//! `datasource.query` (`impl Core`, 0.121.0): executar o que o autor
//! escreveu, como JOB — a rede e o disco esperam; o laco de despacho nao.
//!
//! A recusa `WRITE_CONFIRMATION_REQUIRED` e' SINCRONA, antes do job: a
//! operacao exige confirmacao e o pedido nao trouxe `confirmWrite: true`.
//! Alteracoes filtradas medem em silencio no job antes de escrever.
//! A UI pergunta e reenvia — pelo codigo, nunca pelo texto.

use kinein_protocol::{
    DataSourceQueriedEvent, DataSourceQueryParams, DataSourceTestAccepted, JobRisk, JsonRpcError,
    JsonRpcErrorCode, JsonRpcResponse,
};
use serde_json::{Value, json};

use super::datasource::com_id;
use crate::Core;
use crate::datasource::query;
use crate::jobs::JobOutcome;
use crate::rpc::{jobs_unavailable_response, no_workspace_response, parse_params};

impl Core {
    /// `datasource.query { name, password?, sql, maxRows?, confirmWrite? }`
    /// -> `{ jobId }`; `event.datasource.queried` no fim.
    pub(super) fn datasource_query_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "datasource.query");
        };
        let request = match parse_params::<DataSourceQueryParams>(
            request_id.as_ref(),
            params,
            "datasource.query exige { name, sql } e aceita { password, maxRows, confirmWrite }",
        ) {
            Ok(request) => request,
            Err(response) => return *response,
        };
        if request.sql.trim().is_empty() {
            return JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(JsonRpcErrorCode::InvalidParams, "escreva a instrucao", None),
            );
        }
        let profile = match Self::find_profile(&root, &request.name) {
            Ok(profile) => profile,
            Err(response) => return com_id(*response, request_id),
        };
        // O Mongo escreve desde o 0.155.0; o comando validado diz o que faz
        // (um texto invalido nao escreve — falha no job sem tocar o banco).
        let mongo = profile.engine == kinein_protocol::DataSourceEngine::Mongo;
        let statements = if mongo {
            crate::datasource::mongo_command::parse(&request.sql)
                .map(|c| vec![crate::datasource::mongo_command::impact(&c, &request.sql)])
                .unwrap_or_default()
        } else {
            crate::datasource::impact::classify_all(&request.sql)
        };
        // Classifica TODO o lote: uma leitura inicial nao pode esconder um
        // COMMIT que encerra o READ ONLY e uma remocao de dados logo depois.
        let write = statements
            .iter()
            .any(|s| s.severity != kinein_protocol::SqlImpactSeverity::Read);
        // So' o que REMOVE dados pede o aviso (0.156.0, decisao do autor):
        // inserir e alterar com filtro rodam direto (`datasource::confirm`).
        if crate::datasource::confirm::needs_confirmation(&statements) && !request.confirm_write {
            return JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(
                    JsonRpcErrorCode::WriteConfirmationRequired,
                    "esta instrucao pode remover ou alterar dados; confirme para executar",
                    // A gravidade ja' vai na recusa (pura, sem banco): a tela
                    // sabe na hora se pede o nome do que some; os numeros
                    // vem do `datasource.impact` (0.150.0).
                    Some(json!({
                        "name": profile.name,
                        "severity": crate::datasource::impact::overall(&statements),
                    })),
                ),
            );
        }
        let secret = match Self::resolve_secret(&profile, request.password) {
            Ok(secret) => secret,
            Err(response) => return com_id(*response, request_id),
        };
        let Some(jobs) = self.jobs.as_ref() else {
            return jobs_unavailable_response(request_id, "datasource.query");
        };
        let max_rows = query::clamp_rows(request.max_rows);
        let preflight =
            !request.confirm_write && crate::datasource::confirm::needs_measurement(&statements);
        let sql = request.sql;
        let title = format!(
            "{} em {}",
            if write { "Escrever" } else { "Consultar" },
            profile.name
        );
        let risk = if write { JobRisk::Medium } else { JobRisk::Low };
        let job_id = jobs.spawn("datasource", title, risk, false, move |ctx| {
            if preflight {
                let measured =
                    crate::datasource::measurement::statements(&profile, secret.as_ref(), &sql);
                if crate::datasource::confirm::needs_confirmation(&measured) {
                    ctx.emit_event(
                        "event.datasource.queried",
                        json!(DataSourceQueriedEvent {
                            job_id: ctx.id().to_owned(),
                            name: profile.name,
                            confirmation_sql: Some(sql),
                            message: Some(
                                "o impacto exige confirmacao; nenhuma escrita foi executada"
                                    .to_owned()
                            ),
                            ..DataSourceQueriedEvent::default()
                        }),
                    );
                    return JobOutcome::Failed;
                }
            }
            let resultado = query::run(&profile, secret.as_ref(), &sql, max_rows);
            let evento = evento_da_consulta(ctx, profile.name, resultado);
            let ok = evento.success;
            ctx.emit_event("event.datasource.queried", json!(evento));
            if ok {
                JobOutcome::Success
            } else {
                JobOutcome::Failed
            }
        });
        JsonRpcResponse::success(request_id, json!(DataSourceTestAccepted { job_id }))
    }
}

/// O evento a partir do resultado do motor; a linha na saida do job diz o
/// resumo.
fn evento_da_consulta(
    ctx: &crate::jobs::JobContext,
    name: String,
    resultado: Result<query::QueryResult, (String, bool)>,
) -> DataSourceQueriedEvent {
    match resultado {
        Ok(r) => {
            ctx.emit_output(&match r.affected {
                Some(n) => format!("{n} linha(s) afetada(s) em {} ms", r.elapsed_ms),
                None => format!(
                    "{} linha(s){} em {} ms",
                    r.rows.len(),
                    if r.truncated { " (teto)" } else { "" },
                    r.elapsed_ms
                ),
            });
            DataSourceQueriedEvent {
                job_id: ctx.id().to_owned(),
                name,
                success: true,
                row_count: r.rows.len(),
                columns: r.columns,
                rows: r.rows,
                affected: r.affected,
                truncated: r.truncated,
                elapsed_ms: r.elapsed_ms,
                message: None,
                secret_required: false,
                confirmation_sql: None,
            }
        }
        Err((message, secret_required)) => {
            ctx.emit_output(&message);
            DataSourceQueriedEvent {
                job_id: ctx.id().to_owned(),
                name,
                success: false,
                message: Some(message),
                secret_required,
                ..DataSourceQueriedEvent::default()
            }
        }
    }
}

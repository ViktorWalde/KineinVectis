//! `datasource.query` (`impl Core`, 0.121.0): executar o que o autor
//! escreveu, como JOB — a rede e o disco esperam; o laco de despacho nao.
//!
//! A recusa `WRITE_CONFIRMATION_REQUIRED` e' SINCRONA, antes do job: a
//! instrucao nao e' de leitura e o pedido nao trouxe `confirmWrite: true`.
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
        let pedido = match parse_params::<DataSourceQueryParams>(
            request_id.as_ref(),
            params,
            "datasource.query exige { name, sql } e aceita { password, maxRows, confirmWrite }",
        ) {
            Ok(pedido) => pedido,
            Err(response) => return *response,
        };
        if pedido.sql.trim().is_empty() {
            return JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(JsonRpcErrorCode::InvalidParams, "escreva a instrucao", None),
            );
        }
        let profile = match Self::find_profile(&root, &pedido.name) {
            Ok(profile) => profile,
            Err(response) => return com_id(*response, request_id),
        };
        let escrita = !query::is_read(&pedido.sql)
            && profile.engine != kinein_protocol::DataSourceEngine::Mongo;
        if escrita && !pedido.confirm_write {
            return JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(
                    JsonRpcErrorCode::WriteConfirmationRequired,
                    "esta instrucao ESCREVE no banco; confirme para executar",
                    Some(json!({ "name": profile.name })),
                ),
            );
        }
        let secret = match Self::resolve_secret(&profile, pedido.password) {
            Ok(secret) => secret,
            Err(response) => return com_id(*response, request_id),
        };
        let Some(jobs) = self.jobs.as_ref() else {
            return jobs_unavailable_response(request_id, "datasource.query");
        };
        let max_rows = query::clamp_rows(pedido.max_rows);
        let sql = pedido.sql;
        let titulo = format!(
            "{} em {}",
            if escrita { "Escrever" } else { "Consultar" },
            profile.name
        );
        let risco = if escrita {
            JobRisk::Medium
        } else {
            JobRisk::Low
        };
        let job_id = jobs.spawn("datasource", titulo, risco, false, move |ctx| {
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

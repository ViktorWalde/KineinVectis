//! `datasource.query` (`impl Core`, 0.121.0): executar o que o autor
//! escreveu, como JOB — a rede e o disco esperam; o laco de despacho nao.
//!
//! A recusa `WRITE_CONFIRMATION_REQUIRED` e' SINCRONA, antes do job: a
//! operacao exige confirmacao e o pedido nao trouxe `confirmWrite: true`.
//! Alteracoes filtradas medem em silencio no job antes de escrever.
//! A UI pergunta e reenvia — pelo codigo, nunca pelo texto.

use kinein_protocol::{
    DataSourceQueriedEvent, DataSourceQueryAccepted, DataSourceQueryParams, JobRisk, JsonRpcError,
    JsonRpcErrorCode, JsonRpcResponse,
};
use serde_json::{Value, json};

use super::datasource::com_id;
use crate::Core;
use crate::datasource::{classification, policy, query};
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
        let mut request = match parse_params::<DataSourceQueryParams>(
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
        let statements = classification::classify(profile.engine, &request.sql);
        // Classifica TODO o lote: uma leitura inicial nao pode esconder um
        // COMMIT que encerra o READ ONLY e uma remocao de dados logo depois.
        let write = statements.is_empty()
            || statements
                .iter()
                .any(|s| s.severity != kinein_protocol::SqlImpactSeverity::Read);
        let preview_sql = match query_permission(&root, &profile, &request, &statements) {
            Ok(sql) => sql,
            Err(rejection) => {
                return rejection.query_response(
                    request_id,
                    &profile,
                    &statements,
                    request.client_context.as_deref(),
                );
            }
        };
        if let Err(response) = self.require_odbc_driver(&root, &profile, request_id.clone()) {
            return *response;
        }
        let secret = match Self::resolve_secret(&profile, request.password.take()) {
            Ok(secret) => secret,
            Err(response) => return com_id(*response, request_id),
        };
        let Some(jobs) = self.jobs.as_ref() else {
            return jobs_unavailable_response(request_id, "datasource.query");
        };
        let activity = match self.begin_datasource_operation(
            &root,
            &profile.name,
            request.client_context.as_deref(),
            request_id.clone(),
        ) {
            Ok(activity) => activity,
            Err(response) => return *response,
        };
        let preview = match self.reserve_preview(&request, preview_sql) {
            Ok(preview) => preview,
            Err(rejection) => {
                return rejection.response(
                    request_id,
                    &profile.name,
                    request.client_context.as_deref(),
                );
            }
        };
        let title = format!(
            "{} em {}",
            if write { "Escrever" } else { "Consultar" },
            profile.name
        );
        let risk = if write { JobRisk::Medium } else { JobRisk::Low };
        let accepted_name = profile.name.clone();
        let accepted_context = request.client_context.clone();
        let accepted_preview = request.preview;
        let job_id = jobs.spawn("datasource", title, risk, request.preview, move |ctx| {
            let _activity = activity;
            run_query_job(
                ctx,
                profile,
                secret.as_ref(),
                request,
                &statements,
                write,
                preview,
            )
        });
        JsonRpcResponse::success(
            request_id,
            json!(DataSourceQueryAccepted {
                job_id,
                name: accepted_name,
                client_context: accepted_context,
                preview: accepted_preview
            }),
        )
    }
}

fn query_permission(
    root: &std::path::Path,
    profile: &kinein_protocol::DataSourceProfile,
    request: &DataSourceQueryParams,
    statements: &[kinein_protocol::SqlStatementImpact],
) -> Result<Option<String>, policy::Rejection> {
    policy::check_context(
        root,
        profile,
        request.expected_context.as_ref(),
        request.client_context.as_deref(),
    )
    .and_then(|()| {
        policy::check_query(
            profile,
            statements,
            request.confirm_write,
            request.confirmation.as_ref(),
        )
    })
    .and_then(|()| super::datasource_preview::validate(profile, request))
}

/// Medição/execução esperam no worker; o despacho continua livre.
fn run_query_job(
    ctx: &crate::jobs::JobContext,
    profile: kinein_protocol::DataSourceProfile,
    secret: Option<&crate::datasource::Secret>,
    request: DataSourceQueryParams,
    statements: &[kinein_protocol::SqlStatementImpact],
    write: bool,
    preview: Option<super::datasource_preview::Prepared>,
) -> JobOutcome {
    let max_rows = query::clamp_rows(request.max_rows);
    let preflight = (!request.confirm_write || profile.production)
        && crate::datasource::confirm::needs_measurement(statements);
    let sql = &request.sql;
    if preflight {
        let measured = crate::datasource::measurement::statements(&profile, secret, sql);
        if let Err(rejection) = policy::check_query(
            &profile,
            &measured,
            request.confirm_write,
            request.confirmation.as_ref(),
        ) {
            drop(preview);
            ctx.emit_event(
                "event.datasource.queried",
                json!(DataSourceQueriedEvent {
                    job_id: ctx.id().to_owned(),
                    name: profile.name,
                    confirmation_sql: Some(sql.clone()),
                    client_context: request.client_context,
                    access: kinein_protocol::DataSourceQueryAccess::Write,
                    message: Some(rejection.message.to_owned()),
                    ..DataSourceQueriedEvent::default()
                }),
            );
            return JobOutcome::Failed;
        }
    }
    if let Some(preview) = preview {
        return crate::datasource::preview_postgres::run(
            ctx,
            &profile,
            secret,
            &request,
            preview.lease,
            &preview.sql,
        );
    }
    let catalog_invalidated = classification::invalidates_catalog(profile.engine, statements);
    let resultado = query::run(&profile, secret, sql, max_rows);
    let mut event = evento_da_consulta(ctx, profile.name, resultado);
    event.client_context = request.client_context;
    event.catalog_update = if catalog_invalidated && !event.secret_required {
        kinein_protocol::DataSourceCatalogUpdate::Reload
    } else {
        kinein_protocol::DataSourceCatalogUpdate::None
    };
    event.access = if write {
        kinein_protocol::DataSourceQueryAccess::Write
    } else {
        kinein_protocol::DataSourceQueryAccess::Read
    };
    let ok = event.success;
    ctx.emit_event("event.datasource.queried", json!(event));
    if ok {
        JobOutcome::Success
    } else {
        JobOutcome::Failed
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
                client_context: None,
                access: kinein_protocol::DataSourceQueryAccess::Read,
                preview_outcome: None,
                catalog_update: kinein_protocol::DataSourceCatalogUpdate::None,
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

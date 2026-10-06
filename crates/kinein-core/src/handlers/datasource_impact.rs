//! `datasource.impact` (`0.150.0`): o que uma escrita faria, MEDIDO antes de
//! ela rodar — a tela mostra o comando e a consequencia e pergunta.
//!
//! A classificacao e' pura (`datasource::impact`); aqui ela vira JOB porque
//! as contagens falam com o banco, e a rede espera — o laco de despacho nao.
//! Cada contagem e' uma LEITURA (`SELECT count(*) ...`) e por isso vai pelo
//! caminho read-only do `query::run`: contar nunca escreve, nem com um
//! `WHERE` que chama funcao.

use kinein_protocol::{
    DataSourceImpactEvent, DataSourceImpactParams, DataSourceTestAccepted, JobRisk, JsonRpcResponse,
};
use serde_json::{Value, json};

use super::datasource::com_id;
use crate::Core;
use crate::datasource::{confirm, impact, measurement, policy};
use crate::jobs::JobOutcome;
use crate::rpc::{jobs_unavailable_response, no_workspace_response, parse_params};

impl Core {
    /// `datasource.impact { name, password?, sql }` -> `{ jobId }`;
    /// `event.datasource.impact` no fim.
    pub(super) fn datasource_impact_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "datasource.impact");
        };
        let request = match parse_params::<DataSourceImpactParams>(
            request_id.as_ref(),
            params,
            "datasource.impact exige { name, sql } e aceita { password }",
        ) {
            Ok(request) => request,
            Err(response) => return *response,
        };
        let profile = match Self::find_profile(&root, &request.name) {
            Ok(profile) => profile,
            Err(response) => return com_id(*response, request_id),
        };
        if let Err(rejection) = policy::check_context(
            &root,
            &profile,
            request.expected_context.as_ref(),
            request.client_context.as_deref(),
        ) {
            return rejection.response(
                request_id,
                &profile.name,
                request.client_context.as_deref(),
            );
        }
        let secret = if profile.engine == kinein_protocol::DataSourceEngine::Odbc {
            None
        } else {
            match Self::resolve_secret(&profile, request.password) {
                Ok(secret) => secret,
                Err(response) => return com_id(*response, request_id),
            }
        };
        let Some(jobs) = self.jobs.as_ref() else {
            return jobs_unavailable_response(request_id, "datasource.impact");
        };
        let title = format!("Medir o impacto em {}", profile.name);
        let job_id = jobs.spawn("datasource", title, JobRisk::Low, false, move |ctx| {
            let statements = measurement::statements(&profile, secret.as_ref(), &request.sql);
            let severity = impact::overall(&statements);
            let requires_connection =
                profile.production && confirm::needs_confirmation(&statements);
            let confirmation_target = (severity == kinein_protocol::SqlImpactSeverity::Destructive
                || requires_connection)
                .then(|| policy::confirm_target(&profile, &statements));
            let preview_eligible = profile.engine == kinein_protocol::DataSourceEngine::Postgres
                && !profile.read_only
                && crate::datasource::preview_sql::executed_sql(&request.sql).is_some();
            let event = DataSourceImpactEvent {
                job_id: ctx.id().to_owned(),
                name: profile.name.clone(),
                sql: request.sql,
                severity,
                statements,
                client_context: request.client_context,
                confirmation_target,
                requires_connection,
                preview_eligible,
            };
            ctx.emit_event("event.datasource.impact", json!(event));
            JobOutcome::Success
        });
        JsonRpcResponse::success(request_id, json!(DataSourceTestAccepted { job_id }))
    }
}

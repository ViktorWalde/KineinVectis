//! `datasource.impact` (`0.150.0`): o que uma escrita faria, MEDIDO antes de
//! ela rodar — a tela mostra o comando e a consequencia e pergunta.
//!
//! A classificacao e' pura (`datasource::impact`); aqui ela vira JOB porque
//! as contagens falam com o banco, e a rede espera — o laco de despacho nao.
//! Cada contagem e' uma LEITURA (`SELECT count(*) ...`) e por isso vai pelo
//! caminho read-only do `query::run`: contar nunca escreve, nem com um
//! `WHERE` que chama funcao.

use kinein_protocol::{
    DataSourceImpactEvent, DataSourceImpactParams, DataSourceTestAccepted, JobRisk,
    JsonRpcResponse, SqlImpactSeverity, SqlStatementImpact,
};
use serde_json::{Value, json};

use super::datasource::com_id;
use crate::Core;
use crate::datasource::impact;
use crate::datasource::query;
use crate::datasource::secret::Secret;
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
        let secret = match Self::resolve_secret(&profile, request.password) {
            Ok(secret) => secret,
            Err(response) => return com_id(*response, request_id),
        };
        let Some(jobs) = self.jobs.as_ref() else {
            return jobs_unavailable_response(request_id, "datasource.impact");
        };
        let title = format!("Medir o impacto em {}", profile.name);
        let job_id = jobs.spawn("datasource", title, JobRisk::Low, false, move |ctx| {
            let statements = impact::classify_all(&request.sql)
                .into_iter()
                .map(|statement| measure(&profile, secret.as_ref(), statement))
                .collect::<Vec<_>>();
            let event = DataSourceImpactEvent {
                job_id: ctx.id().to_owned(),
                name: profile.name.clone(),
                sql: request.sql,
                severity: impact::overall(&statements),
                statements,
            };
            ctx.emit_event("event.datasource.impact", json!(event));
            JobOutcome::Success
        });
        JsonRpcResponse::success(request_id, json!(DataSourceTestAccepted { job_id }))
    }
}

/// Roda as contagens de uma instrucao e promove o `WHERE` que pega tudo.
fn measure(
    profile: &kinein_protocol::DataSourceProfile,
    secret: Option<&Secret>,
    mut statement: SqlStatementImpact,
) -> SqlStatementImpact {
    let Some((hit, total)) = impact::count_queries(&statement) else {
        return statement;
    };
    match count(profile, secret, &hit) {
        Ok(rows) => statement.rows = Some(rows),
        Err(message) => {
            statement.note = Some(message);
            return statement;
        }
    }
    if let Some(total) = total {
        statement.total_rows = count(profile, secret, &total).ok();
    }
    // `WHERE 1=1`, um filtro esquecido: pega TODAS as linhas de uma tabela
    // que tem linhas — e' tao destrutivo quanto nao ter WHERE.
    if let (Some(rows), Some(total)) = (statement.rows, statement.total_rows)
        && total > 0
        && rows == total
    {
        statement.severity = SqlImpactSeverity::Destructive;
    }
    statement
}

/// Uma contagem: a primeira celula da primeira linha, como numero.
fn count(
    profile: &kinein_protocol::DataSourceProfile,
    secret: Option<&Secret>,
    sql: &str,
) -> Result<u64, String> {
    let result = query::run(profile, secret, sql, 1).map_err(|(message, _)| message)?;
    result
        .rows
        .first()
        .and_then(|row| row.first().cloned().flatten())
        .and_then(|cell| cell.trim().parse::<u64>().ok())
        .ok_or_else(|| "o motor nao devolveu a contagem".to_owned())
}

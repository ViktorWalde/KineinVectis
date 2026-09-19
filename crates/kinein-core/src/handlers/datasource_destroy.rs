//! `datasource.destroy` (`impl Core`, `0.129.0`): o perfil e, com `data`, os dados.
//!
//! Pelo plano de `crate::datasource::destroy`: arquivo `SQLite` e "so' o
//! perfil" respondem na hora; container e `DROP DATABASE` sao jobs (processo
//! e rede), com `event.datasource.destroyed`.

use std::sync::{Arc, atomic::AtomicBool};

use kinein_protocol::{
    DataSourceDestroyParams, DataSourceDestroyResult, DataSourceDestroyedEvent, JobRisk,
    JsonRpcError, JsonRpcErrorCode, JsonRpcResponse,
};
use serde_json::{Value, json};

use crate::Core;
use crate::datasource::destroy::{self, DestroyPlan};
use crate::datasource::query;
use crate::jobs::JobOutcome;
use crate::rpc::{jobs_unavailable_response, no_workspace_response, parse_params};

impl Core {
    /// `datasource.destroy` — o perfil e, se pedido, os dados.
    pub(crate) fn datasource_destroy_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "datasource.destroy");
        };
        let pedido = match parse_params::<DataSourceDestroyParams>(
            request_id.as_ref(),
            params,
            "datasource.destroy exige { name, data? }",
        ) {
            Ok(pedido) => pedido,
            Err(response) => return *response,
        };
        let Some(profile) = crate::datasource::list(&root)
            .into_iter()
            .find(|p| p.name == pedido.name)
        else {
            return JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(
                    JsonRpcErrorCode::InvalidParams,
                    format!("nao ha perfil chamado {}", pedido.name),
                    None,
                ),
            );
        };
        if !pedido.data {
            return catalogo(
                request_id,
                crate::datasource::remove(&root, &profile.name),
                None,
            );
        }
        // O container existe? So' vale perguntar ao motor quando ha' um.
        let engine = crate::container::detect_with(&self.detector);
        let container_name = format!("kinein-{}", profile.name);
        let container_exists = engine.as_ref().is_some_and(|e| {
            crate::container::list(e, true).containers.iter().any(|c| {
                c.names
                    .iter()
                    .any(|n| n == &container_name || n == &format!("/{container_name}"))
            })
        });
        match destroy::plan(&root, &profile, container_exists) {
            DestroyPlan::ProfileOnly { note } => catalogo(
                request_id,
                crate::datasource::remove(&root, &profile.name),
                Some(note),
            ),
            DestroyPlan::SqliteFile { path } => match destroy::delete_sqlite_file(&path) {
                Ok(()) => catalogo(
                    request_id,
                    crate::datasource::remove(&root, &profile.name),
                    None,
                ),
                Err(mensagem) => JsonRpcResponse::failure(
                    request_id,
                    JsonRpcError::new(JsonRpcErrorCode::InternalError, mensagem, None),
                ),
            },
            DestroyPlan::Container { name } => {
                self.destroy_container_response(request_id, &root, &profile.name, engine, &name)
            }
            DestroyPlan::PostgresDrop {
                maintenance,
                database,
            } => self.destroy_postgres_response(
                request_id,
                &root,
                &profile.name,
                &maintenance,
                &database,
            ),
        }
    }

    /// O container do servidor: `rm -f`, como job.
    fn destroy_container_response(
        &self,
        request_id: Option<Value>,
        root: &std::path::Path,
        profile_name: &str,
        engine: Option<crate::container::Engine>,
        name: &str,
    ) -> JsonRpcResponse {
        let Some(motor) = engine else {
            return jobs_unavailable_response(request_id, "datasource.destroy");
        };
        let Some(jobs) = self.jobs.as_ref() else {
            return jobs_unavailable_response(request_id, "datasource.destroy");
        };
        let mut command = std::process::Command::new(&motor.binary);
        command.args(["rm", "-f", name]);
        let shown = format!(
            "{} rm -f {name}",
            motor
                .binary
                .file_name()
                .map_or_else(String::new, |n| n.to_string_lossy().to_string())
        );
        let mostrado = shown.clone();
        let profile_name = profile_name.to_owned();
        let root = root.to_path_buf();
        let job_id = jobs.spawn(
            "datasource",
            format!("Remover o container {name}"),
            JobRisk::High,
            true,
            move |ctx| {
                ctx.emit_output(&format!("$ {shown}"));
                let cancel: Arc<AtomicBool> = ctx.cancellation();
                let mut ultimas: Vec<String> = Vec::new();
                let mut on_line = |_o: &'static str, linha: String| {
                    ctx.emit_output(&linha);
                    ultimas.push(linha);
                };
                let ok = matches!(
                    crate::process::stream_command_lines_cancelable(command, &cancel, &mut on_line),
                    Ok(status) if status.success()
                );
                fechar(ctx, &root, &profile_name, ok, &ultimas.join("\n"))
            },
        );
        JsonRpcResponse::success(
            request_id,
            json!(DataSourceDestroyResult {
                job_id: Some(job_id),
                command: Some(mostrado),
                ..Default::default()
            }),
        )
    }

    /// O banco dentro do `PostgreSQL`: `DROP DATABASE` pelo `postgres`, como job.
    fn destroy_postgres_response(
        &self,
        request_id: Option<Value>,
        root: &std::path::Path,
        profile_name: &str,
        maintenance: &kinein_protocol::DataSourceProfile,
        database: &str,
    ) -> JsonRpcResponse {
        let Some(jobs) = self.jobs.as_ref() else {
            return jobs_unavailable_response(request_id, "datasource.destroy");
        };
        let secret = match Self::resolve_secret(maintenance, None) {
            Ok(secret) => secret,
            Err(response) => return *response,
        };
        let sql = destroy::drop_statement(database);
        let mostrado = sql.clone();
        let profile_name = profile_name.to_owned();
        let root = root.to_path_buf();
        let maintenance = maintenance.clone();
        let job_id = jobs.spawn(
            "datasource",
            format!("Apagar o banco {database} em {}", maintenance.host),
            JobRisk::High,
            false,
            move |ctx| {
                ctx.emit_output(&sql);
                let (ok, detalhe) = match query::run(&maintenance, secret.as_ref(), &sql, 1) {
                    Ok(_) => (true, String::new()),
                    Err((mensagem, _)) => (false, mensagem),
                };
                fechar(ctx, &root, &profile_name, ok, &detalhe)
            },
        );
        JsonRpcResponse::success(
            request_id,
            json!(DataSourceDestroyResult {
                job_id: Some(job_id),
                command: Some(mostrado),
                ..Default::default()
            }),
        )
    }
}

/// A resposta imediata: o catalogo depois de remover o perfil (com a nota).
fn catalogo(
    request_id: Option<Value>,
    removido: Result<Vec<kinein_protocol::DataSourceProfile>, String>,
    note: Option<String>,
) -> JsonRpcResponse {
    match removido {
        Ok(profiles) => JsonRpcResponse::success(
            request_id,
            json!(DataSourceDestroyResult {
                profiles: Some(profiles),
                note,
                ..Default::default()
            }),
        ),
        Err(mensagem) => JsonRpcResponse::failure(
            request_id,
            JsonRpcError::new(JsonRpcErrorCode::InternalError, mensagem, None),
        ),
    }
}

/// O fecho do job: com sucesso, o perfil sai; o evento diz o desfecho.
fn fechar(
    ctx: &crate::jobs::JobContext,
    root: &std::path::Path,
    profile_name: &str,
    ok: bool,
    detalhe: &str,
) -> JobOutcome {
    let (success, message, profiles) = if ok {
        match crate::datasource::remove(root, profile_name) {
            Ok(profiles) => (true, format!("{profile_name} removido"), Some(profiles)),
            Err(e) => (
                false,
                format!("os dados sumiram, mas o perfil nao saiu: {e}"),
                None,
            ),
        }
    } else {
        (
            false,
            if detalhe.is_empty() {
                "falhou".to_owned()
            } else {
                detalhe.to_owned()
            },
            None,
        )
    };
    ctx.emit_output(&message);
    ctx.emit_event(
        "event.datasource.destroyed",
        json!(DataSourceDestroyedEvent {
            job_id: ctx.id().to_owned(),
            success,
            message,
            profiles
        }),
    );
    if success {
        JobOutcome::Success
    } else {
        JobOutcome::Failed
    }
}

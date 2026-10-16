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
use crate::container::{self, Engine, EngineFailure};
use crate::datasource::destroy::{self, DestroyPlan};
use crate::datasource::query;
use crate::jobs::JobOutcome;
use crate::rpc::{jobs_unavailable_response, no_workspace_response, parse_params};
use crate::tools::ToolDetector;

impl Core {
    /// `datasource.destroy` — o perfil e, se pedido, os dados.
    pub(crate) fn datasource_destroy_response(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "datasource.destroy");
        };
        let mut request = match parse_params::<DataSourceDestroyParams>(
            request_id.as_ref(),
            params,
            "datasource.destroy exige { name, data? }",
        ) {
            Ok(request) => request,
            Err(response) => return *response,
        };
        if let Err(message) = crate::datasource::ensure_writable(&root) {
            return JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(JsonRpcErrorCode::InternalError, message, None),
            );
        }
        let Some(profile) = crate::datasource::list(&root)
            .into_iter()
            .find(|p| p.name == request.name)
        else {
            return JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(
                    JsonRpcErrorCode::InvalidParams,
                    format!("não há perfil chamado {}", request.name),
                    None,
                ),
            );
        };
        if let Err(rejection) = crate::datasource::policy::check_context(
            &root,
            &profile,
            request.expected_context.as_ref(),
            request.client_context.as_deref(),
        )
        .and_then(|()| {
            crate::datasource::policy::check_destroy(
                &profile,
                request.data,
                request.confirmation.as_ref(),
            )
        }) {
            return rejection.response(
                request_id,
                &profile.name,
                request.client_context.as_deref(),
            );
        }
        let token = request.client_context.clone();
        let activity = match self.begin_datasource_operation(
            &root,
            &profile.name,
            request.client_context.as_deref(),
            request_id.clone(),
        ) {
            Ok(activity) => activity,
            Err(response) => return *response,
        };
        // O plano so' pergunta ao motor quando apaga os dados de um servidor;
        // a escolha e a lista rodam fora do laco (16b.1, roadmaps/59 §2.5.2).
        if !(request.data && destroy::depends_on_container(profile.engine)) {
            let response =
                self.execute_destroy(request_id, &root, &profile, &mut request, activity, None);
            return with_client_context(response, token.as_deref());
        }
        let detector = self.detector.clone();
        let container_name = format!("kinein-{}", profile.name);
        self.defer_then(
            request_id,
            move || server_container(&detector, &container_name),
            move |core, request_id, container_engine| match container_engine {
                Ok(container_engine) => with_client_context(
                    core.execute_destroy(
                        request_id,
                        &root,
                        &profile,
                        &mut request,
                        activity,
                        container_engine,
                    ),
                    token.as_deref(),
                ),
                Err(failure) => super::container::engine_failure_response(
                    request_id,
                    &failure,
                    "nada foi apagado, nem o perfil: sem o motor não da' para saber se o \
                     container_engine é um container da IDE. ",
                ),
            },
        )
    }

    fn execute_destroy(
        &self,
        request_id: Option<Value>,
        root: &std::path::Path,
        profile: &kinein_protocol::DataSourceProfile,
        request: &mut DataSourceDestroyParams,
        activity: crate::datasource::activity::Operation,
        container_engine: Option<Engine>,
    ) -> JsonRpcResponse {
        let operation = DestroyContext {
            root: root.to_path_buf(),
            profile: profile.clone(),
            client_context: request.client_context.clone(),
            _activity: activity,
        };
        self.odbc.revoke(root, &profile.name);
        self.previews.revoke(root, &profile.name);
        if !request.data {
            return catalogo(
                request_id,
                crate::datasource::remove(root, &profile.name),
                None,
            );
        }
        match destroy::plan(root, profile, container_engine.is_some()) {
            DestroyPlan::ProfileOnly { note } => catalogo(
                request_id,
                crate::datasource::remove(root, &profile.name),
                Some(note),
            ),
            DestroyPlan::SqliteFile { path } => match destroy::delete_sqlite_file(&path) {
                Ok(()) => catalogo(
                    request_id,
                    crate::datasource::remove(root, &profile.name),
                    None,
                ),
                Err(mensagem) => JsonRpcResponse::failure(
                    request_id,
                    JsonRpcError::new(JsonRpcErrorCode::InternalError, mensagem, None),
                ),
            },
            DestroyPlan::Container { name } => {
                self.destroy_container_response(request_id, operation, container_engine, &name)
            }
            DestroyPlan::PostgresDrop {
                maintenance,
                database,
            } => self.destroy_postgres_response(
                request_id,
                operation,
                &maintenance,
                &database,
                request.password.take(),
            ),
        }
    }

    /// O container do servidor: `rm -f -v`, como job. O `-v` leva os volumes
    /// ANONIMOS (o `VOLUME` das imagens `postgres` e `mongo`), onde moram os
    /// dados; sem ele "Apagar os dados" deixava o banco no disco (defeito 1,
    /// roadmaps/59 §2.5.2). Volume nomeado fica: nao foi a IDE que o criou.
    fn destroy_container_response(
        &self,
        request_id: Option<Value>,
        operation: DestroyContext,
        engine: Option<Engine>,
        name: &str,
    ) -> JsonRpcResponse {
        let Some(motor) = engine else {
            return jobs_unavailable_response(request_id, "datasource.destroy");
        };
        let Some(jobs) = self.jobs.as_ref() else {
            return jobs_unavailable_response(request_id, "datasource.destroy");
        };
        let mut command = std::process::Command::new(&motor.binary);
        command.args(destroy::CONTAINER_REMOVE_ARGS).arg(name);
        let shown = format!(
            "{} {} {name}",
            crate::platform::program_label(&motor.binary),
            destroy::CONTAINER_REMOVE_ARGS.join(" ")
        );
        let mostrado = shown.clone();
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
                fechar(ctx, &operation, ok, &ultimas.join("\n"))
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
        operation: DestroyContext,
        maintenance: &kinein_protocol::DataSourceProfile,
        database: &str,
        password: Option<String>,
    ) -> JsonRpcResponse {
        let Some(jobs) = self.jobs.as_ref() else {
            return jobs_unavailable_response(request_id, "datasource.destroy");
        };
        let secret = match Self::resolve_secret(maintenance, password) {
            Ok(secret) => secret,
            Err(response) => return super::datasource::com_id(*response, request_id),
        };
        let sql = destroy::drop_statement(database);
        let mostrado = sql.clone();
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
                fechar(ctx, &operation, ok, &detalhe)
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

#[derive(Debug)]
struct DestroyContext {
    _activity: crate::datasource::activity::Operation,
    root: std::path::PathBuf,
    profile: kinein_protocol::DataSourceProfile,
    client_context: Option<String>,
}

/// O fecho do job conserva um perfil alterado enquanto a remoção rodava.
fn fechar(
    ctx: &crate::jobs::JobContext,
    operation: &DestroyContext,
    ok: bool,
    detalhe: &str,
) -> JobOutcome {
    let profile_name = &operation.profile.name;
    let (success, message, profiles) = if ok {
        match crate::datasource::remove_unchanged(&operation.root, &operation.profile) {
            Ok(Some(profiles)) => (true, format!("{profile_name} removido"), Some(profiles)),
            Ok(None) => (
                true,
                "Os dados do destino original foram removidos; o perfil alterado foi preservado."
                    .to_owned(),
                Some(crate::datasource::list(&operation.root)),
            ),
            Err(e) => (
                false,
                format!("os dados sumiram, mas o perfil não saiu: {e}"),
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
            profiles,
            client_context: operation.client_context.clone(),
        }),
    );
    if success {
        JobOutcome::Success
    } else {
        JobOutcome::Failed
    }
}

/// O `clientContext` do pedido volta no resultado (`0.158.0`).
fn with_client_context(mut response: JsonRpcResponse, token: Option<&str>) -> JsonRpcResponse {
    if let Some(result) = response.result.as_mut().and_then(Value::as_object_mut) {
        result.insert("clientContext".to_owned(), json!(token));
    }
    response
}

/// O container `kinein-<perfil>` no motor que responde: `Ok(Some(motor))`
/// quando ele existe; `Ok(None)` sem motor no `PATH` (nao ha' container da
/// IDE) ou sem o container; `Err` quando o motor existe e nao responde —
/// nunca "nao existe" por falta de resposta (16b.1, roadmaps/59 §2.5.2).
fn server_container(detector: &ToolDetector, name: &str) -> Result<Option<Engine>, EngineFailure> {
    let Some(selected) = container::answering(detector) else {
        return Ok(None);
    };
    let engine = selected?;
    let exists = container::list(&engine, true)?.containers.iter().any(|c| {
        c.names
            .iter()
            .any(|n| n == name || n.strip_prefix('/') == Some(name))
    });
    Ok(exists.then_some(engine))
}

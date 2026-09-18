//! `datasource.discover` e `datasource.create` (`impl Core`, `0.124.0`).
//!
//! Fino como o `datasource.rs`: valida, delega a `crate::datasource::
//! {discover, create}` e formata. O `discover` roda ADIADO (`defer_work`):
//! ele bate em portas com 200 ms de tolerancia e lista containers — nada
//! disso pode segurar o laco (a licao da F6-b).

use std::sync::{Arc, atomic::AtomicBool};

use kinein_protocol::{
    DataSourceCreateKind, DataSourceCreateParams, DataSourceCreateResult, DataSourceCreatedEvent,
    DataSourceDiscoverParams, JobRisk, JsonRpcError, JsonRpcErrorCode, JsonRpcResponse,
};
use serde_json::{Value, json};

use crate::Core;
use crate::datasource::{create, discover};
use crate::jobs::JobOutcome;
use crate::rpc::{jobs_unavailable_response, no_workspace_response, parse_params};

/// Quantas linhas do motor viajam como motivo na falha.
const LINHAS_DE_MOTIVO: usize = 6;

impl Core {
    /// `datasource.discover` — o que responde nesta maquina, fora do laco.
    pub(crate) fn datasource_discover_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "datasource.discover");
        };
        if let Err(response) = parse_params::<DataSourceDiscoverParams>(
            request_id.as_ref(),
            params,
            "datasource.discover nao aceita parametros",
        ) {
            return *response;
        }
        let detector = self.detector.clone();
        self.defer_work(request_id, move |request_id| {
            JsonRpcResponse::success(request_id, json!(discover::discover(&root, &detector)))
        })
    }

    /// `datasource.create` — um arquivo `SQLite` (imediato) ou um servidor em
    /// container (job, com o comando de volta).
    pub(crate) fn datasource_create_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "datasource.create");
        };
        let pedido = match parse_params::<DataSourceCreateParams>(
            request_id.as_ref(),
            params,
            "datasource.create exige { kind: sqliteFile { name, path? } | containerServer { engine, name, port } }",
        ) {
            Ok(pedido) => pedido,
            Err(response) => return *response,
        };
        match pedido.what {
            DataSourceCreateKind::SqliteFile { name, path } => {
                let criado = create::sqlite_file(&root, &name, path.as_deref())
                    .and_then(|profile| crate::datasource::save(&root, &profile).map(|_| profile));
                match criado {
                    Ok(profile) => JsonRpcResponse::success(
                        request_id,
                        json!(DataSourceCreateResult {
                            profile: Some(profile),
                            ..DataSourceCreateResult::default()
                        }),
                    ),
                    Err(mensagem) => invalid(request_id, mensagem),
                }
            }
            DataSourceCreateKind::ContainerServer { engine, name, port } => {
                self.create_container_server_response(request_id, &root, engine, &name, port)
            }
        }
    }

    /// A metade "servidor em container" do `datasource.create`: acha o motor,
    /// monta o comando pinado e sobe o job.
    fn create_container_server_response(
        &self,
        request_id: Option<Value>,
        root: &std::path::Path,
        engine: kinein_protocol::DataSourceEngine,
        name: &str,
        port: u16,
    ) -> JsonRpcResponse {
        let Some(motor) = crate::container::detect_with(&self.detector) else {
            return JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(
                    JsonRpcErrorCode::ToolNotFound,
                    "nenhum motor de container (docker ou podman) no PATH — sem ele nao ha' como subir um servidor",
                    Some(json!({ "method": "datasource.create" })),
                ),
            );
        };
        let Some(jobs) = self.jobs.as_ref() else {
            return jobs_unavailable_response(request_id, "datasource.create");
        };
        let (command, shown, profile) = match create::container_server(&motor, engine, name, port) {
            Ok(tudo) => tudo,
            Err(mensagem) => return invalid(request_id, mensagem),
        };
        let titulo = format!(
            "Subir {} em container ({})",
            profile.name,
            shown.split_whitespace().next().unwrap_or("motor")
        );
        let comando_mostrado = shown.clone();
        let root = root.to_path_buf();
        let job_id = jobs.spawn("datasource", titulo, JobRisk::High, true, move |ctx| {
            run_server_job(ctx, command, &shown, &root, &profile)
        });
        JsonRpcResponse::success(
            request_id,
            json!(DataSourceCreateResult {
                profile: None,
                job_id: Some(job_id),
                command: Some(comando_mostrado)
            }),
        )
    }
}

/// O job: roda o motor linha a linha, salva o perfil se subiu, e emite
/// `event.datasource.created` com o desfecho.
fn run_server_job(
    ctx: &crate::jobs::JobContext,
    command: std::process::Command,
    shown: &str,
    root: &std::path::Path,
    profile: &kinein_protocol::DataSourceProfile,
) -> JobOutcome {
    ctx.emit_output(&format!("$ {shown}"));
    let cancel: Arc<AtomicBool> = ctx.cancellation();
    let mut ultimas: Vec<String> = Vec::new();
    let mut on_line = |_origem: &'static str, linha: String| {
        ctx.emit_output(&linha);
        if ultimas.len() == LINHAS_DE_MOTIVO {
            ultimas.remove(0);
        }
        ultimas.push(linha);
    };
    let ok = match crate::process::stream_command_lines_cancelable(command, &cancel, &mut on_line) {
        Ok(status) => status.success(),
        Err(crate::process::ProcessError::Spawn(e)) => {
            ultimas.push(format!("nao foi possivel iniciar o motor: {e}"));
            false
        }
        Err(crate::process::ProcessError::Wait(e)) => {
            ultimas.push(format!("nao foi possivel esperar o motor: {e}"));
            false
        }
    };
    let (success, profile, message) = if ok {
        match crate::datasource::save(root, profile) {
            Ok(_) => (
                true,
                Some(profile.clone()),
                format!(
                    "{} no ar em 127.0.0.1:{} — perfil salvo",
                    profile.name, profile.port
                ),
            ),
            Err(e) => (
                false,
                None,
                format!("o container subiu, mas o perfil nao foi salvo: {e}"),
            ),
        }
    } else {
        (false, None, ultimas.join("\n"))
    };
    ctx.emit_output(&message);
    ctx.emit_event(
        "event.datasource.created",
        json!(DataSourceCreatedEvent {
            job_id: ctx.id().to_owned(),
            success,
            profile,
            message
        }),
    );
    if success {
        JobOutcome::Success
    } else {
        JobOutcome::Failed
    }
}

fn invalid(request_id: Option<Value>, mensagem: String) -> JsonRpcResponse {
    JsonRpcResponse::failure(
        request_id,
        JsonRpcError::new(JsonRpcErrorCode::InvalidParams, mensagem, None),
    )
}

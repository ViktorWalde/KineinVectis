//! Handler for `container.*` requests (`impl Core`).
//!
//! Fino: roteia, valida params, delega ao `crate::container` e formata a
//! resposta. As invariantes do `roadmaps/28` §4 aparecem na forma: acao de
//! ciclo de vida e compose sao JOBS com `event.container.finished`; logs e
//! shell abrem NUMA ABA DE TERMINAL pelo mesmo `open_command` do dominio
//! `terminal` — nada e' reimplementado.

use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use kinein_protocol::{
    ContainerActionAccepted, ContainerActionParams, ContainerComposeParams, ContainerEngine,
    ContainerFinishedEvent, ContainerImagesParams, ContainerListParams, ContainerOpenParams,
    ContainerOpenResult, JobRisk, JsonRpcError, JsonRpcErrorCode, JsonRpcResponse,
};
use serde_json::{Value, json};

use crate::Core;
use crate::container::{self, Engine};
use crate::jobs::JobOutcome;
use crate::rpc::{
    jobs_unavailable_response, no_workspace_response, parse_params, terminal_error_response,
    terminal_unavailable_response,
};

/// Quantas linhas finais do motor viajam no evento de termino: e' o motivo
/// quando falhou, e nada quando deu certo.
const LINHAS_DE_MOTIVO: usize = 8;

impl Core {
    /// Roteia os metodos `container.*`.
    pub(crate) fn container_request_response(
        &mut self,
        method: &str,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> Option<JsonRpcResponse> {
        match method {
            "container.status" => Some(Self::container_status_response(request_id, params)),
            "container.list" => Some(Self::container_list_response(request_id, params)),
            "container.images" => Some(Self::container_images_response(request_id, params)),
            "container.action" => Some(self.container_action_response(request_id, params)),
            "container.open" => Some(self.container_open_response(request_id, params)),
            "container.compose" => Some(self.container_compose_response(request_id, params)),
            _ => None,
        }
    }

    /// `container.status` — o motor existe? responde? com que permissao?
    fn container_status_response(
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        if let Err(response) = parse_params::<ContainerImagesParams>(
            request_id.as_ref(),
            params,
            "container.status nao aceita parametros",
        ) {
            return *response;
        }
        let engine = container::detect();
        JsonRpcResponse::success(request_id, json!(container::status(engine.as_ref())))
    }

    /// `container.list` — os containers, parados inclusive por padrao.
    fn container_list_response(
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let pedido = match parse_params::<ContainerListParams>(
            request_id.as_ref(),
            params,
            "container.list aceita so' o campo opcional `all`",
        ) {
            Ok(pedido) => pedido,
            Err(response) => return *response,
        };
        match container::detect() {
            Some(engine) => {
                JsonRpcResponse::success(request_id, json!(container::list(&engine, pedido.all)))
            }
            None => no_engine_response(request_id, "container.list"),
        }
    }

    /// `container.images` — as imagens locais.
    fn container_images_response(
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        if let Err(response) = parse_params::<ContainerImagesParams>(
            request_id.as_ref(),
            params,
            "container.images nao aceita parametros",
        ) {
            return *response;
        }
        match container::detect() {
            Some(engine) => JsonRpcResponse::success(request_id, json!(container::images(&engine))),
            None => no_engine_response(request_id, "container.images"),
        }
    }

    /// `container.action` — start/stop/restart/rm como JOB cancelavel.
    fn container_action_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let pedido = match parse_params::<ContainerActionParams>(
            request_id.as_ref(),
            params,
            "container.action requer os campos id e action (start|stop|restart|remove)",
        ) {
            Ok(pedido) => pedido,
            Err(response) => return *response,
        };
        let Some(engine) = container::detect() else {
            return no_engine_response(request_id, "container.action");
        };
        let Some(jobs) = self.jobs.as_ref() else {
            return jobs_unavailable_response(request_id, "container.action");
        };
        let verbo = pedido.action.verb().to_owned();
        let risco = match pedido.action {
            kinein_protocol::ContainerAction::Remove => JobRisk::High,
            _ => JobRisk::Medium,
        };
        let titulo = format!("{} {}", verbo, pedido.id);
        let command = container::action_command(&engine, pedido.action, &pedido.id);
        let alvo = pedido.id;
        let job_id = jobs.spawn("container", titulo, risco, true, move |ctx| {
            run_and_report(ctx, command, &verbo, &alvo)
        });
        JsonRpcResponse::success(request_id, json!(ContainerActionAccepted { job_id }))
    }

    /// `container.open` — logs ou shell NUMA ABA DE TERMINAL. Exige workspace
    /// porque a aba nasce com o cwd do projeto, como o `terminal.open`.
    fn container_open_response(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let pedido = match parse_params::<ContainerOpenParams>(
            request_id.as_ref(),
            params,
            "container.open requer os campos id e mode (logs|shell)",
        ) {
            Ok(pedido) => pedido,
            Err(response) => return *response,
        };
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "container.open");
        };
        let Some(engine) = container::detect() else {
            return no_engine_response(request_id, "container.open");
        };
        let (programa, args) = container::open_program_args(&engine, pedido.mode, &pedido.id);
        let Some(session) = self.terminal.as_mut() else {
            return terminal_unavailable_response(request_id, "container.open");
        };
        match session.open_command(&root, &programa, &args) {
            Ok(id) => JsonRpcResponse::success(
                request_id,
                json!(ContainerOpenResult {
                    id,
                    command: format!("{programa} {}", args.join(" ")),
                }),
            ),
            Err(error) => terminal_error_response(request_id, &error),
        }
    }

    /// `container.compose` — `up -d` / `down` como JOB, no workspace.
    fn container_compose_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let pedido = match parse_params::<ContainerComposeParams>(
            request_id.as_ref(),
            params,
            "container.compose requer o campo action (up|down) e aceita file",
        ) {
            Ok(pedido) => pedido,
            Err(response) => return *response,
        };
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "container.compose");
        };
        let Some(engine) = container::detect() else {
            return no_engine_response(request_id, "container.compose");
        };
        let Some(compose_tool) = container::compose_tool(&engine) else {
            return JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(
                    JsonRpcErrorCode::ToolNotFound,
                    "nenhum compose respondeu: nem `docker compose`, nem docker-compose, nem \
                     podman-compose",
                    Some(json!({ "engine": engine_name(&engine) })),
                ),
            );
        };
        let Some(jobs) = self.jobs.as_ref() else {
            return jobs_unavailable_response(request_id, "container.compose");
        };
        let verbo = match pedido.action {
            kinein_protocol::ComposeAction::Up => "compose up",
            kinein_protocol::ComposeAction::Down => "compose down",
        }
        .to_owned();
        let alvo = pedido
            .file
            .clone()
            .unwrap_or_else(|| "compose (padrao)".to_owned());
        let command =
            container::compose_command(&compose_tool, pedido.action, pedido.file.as_deref(), &root);
        let titulo = format!("{verbo} {alvo}");
        let job_id = jobs.spawn("container", titulo, JobRisk::Medium, true, move |ctx| {
            run_and_report(ctx, command, &verbo, &alvo)
        });
        JsonRpcResponse::success(request_id, json!(ContainerActionAccepted { job_id }))
    }
}

const fn engine_name(engine: &Engine) -> &'static str {
    match engine.kind {
        ContainerEngine::Docker => "docker",
        ContainerEngine::Podman => "podman",
    }
}

fn no_engine_response(request_id: Option<Value>, method: &str) -> JsonRpcResponse {
    JsonRpcResponse::failure(
        request_id,
        JsonRpcError::new(
            JsonRpcErrorCode::ToolNotFound,
            "nenhum motor de container (docker ou podman) no PATH",
            Some(json!({ "method": method })),
        ),
    )
}

/// Roda o comando como job: cada linha vai para `event.job.output`, as ultimas
/// viajam no `event.container.finished` como motivo, e cancelar mata o filho.
fn run_and_report(
    ctx: &crate::jobs::JobContext,
    command: std::process::Command,
    verbo: &str,
    alvo: &str,
) -> JobOutcome {
    let cancel: Arc<AtomicBool> = ctx.cancellation();
    let mut ultimas: Vec<String> = Vec::new();
    let mut on_line = |_origem: &'static str, linha: String| {
        if linha.contains("Emulate Docker CLI using podman") {
            return;
        }
        ctx.emit_output(&linha);
        if ultimas.len() == LINHAS_DE_MOTIVO {
            ultimas.remove(0);
        }
        ultimas.push(linha);
    };
    let ok = match crate::process::stream_command_lines_cancelable(command, &cancel, &mut on_line) {
        Ok(status) => status.success(),
        Err(erro) => {
            ultimas.push(match erro {
                crate::process::ProcessError::Spawn(e) => format!("nao foi possivel iniciar: {e}"),
                crate::process::ProcessError::Wait(e) => format!("nao foi possivel esperar: {e}"),
            });
            false
        }
    };
    ctx.emit_event(
        "event.container.finished",
        json!(ContainerFinishedEvent {
            job_id: ctx.id().to_owned(),
            action: verbo.to_owned(),
            target: alvo.to_owned(),
            ok,
            message: ultimas.join("\n"),
        }),
    );
    if ok {
        JobOutcome::Success
    } else {
        JobOutcome::Failed
    }
}

//! Handler for `container.*` requests (`impl Core`).
//!
//! Fino: roteia, valida params, delega ao `crate::container` e formata a
//! resposta. As invariantes do `roadmaps/28` §4 aparecem na forma: acao de
//! ciclo de vida e compose sao JOBS com `event.container.finished`; logs e
//! shell abrem NUMA ABA DE TERMINAL pelo mesmo `open_command` do dominio
//! `terminal` — nada e' reimplementado.
//!
//! O motor e' escolhido pelo `select_with` (o que responde ao `ps`, com
//! prazo) e com o detector do `Core` (16b.1, `roadmaps/59` §2.5.2): o que
//! pergunta ao motor roda fora do laco — `status`, `list` e `images` pelo
//! `defer_work`, `action` e `compose` dentro do job. So' o `open` escolhe no
//! laco, porque a aba de terminal nasce dele; a sonda tem prazo.

use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use kinein_protocol::{
    ContainerActionAccepted, ContainerActionParams, ContainerComposeParams, ContainerEngine,
    ContainerFinishedEvent, ContainerImagesParams, ContainerListParams, ContainerOpenParams,
    ContainerOpenResult, ContainerVolumesParams, ContainerVolumesResult, JobRisk, JsonRpcError,
    JsonRpcErrorCode, JsonRpcResponse,
};
use serde_json::{Value, json};

use crate::Core;
use crate::container::{self, Engine, EngineFailure};
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
            "container.status" => Some(self.container_status_response(request_id, params)),
            "container.list" => Some(self.container_list_response(request_id, params)),
            "container.images" => Some(self.container_images_response(request_id, params)),
            "container.volumes" => Some(self.container_volumes_response(request_id, params)),
            "container.action" => Some(self.container_action_response(request_id, params)),
            "container.open" => Some(self.container_open_response(request_id, params)),
            "container.compose" => Some(self.container_compose_response(request_id, params)),
            _ => None,
        }
    }

    /// `container.status` — o motor existe? responde? com que permissao? E o
    /// projeto aberto tem arquivo de compose?
    fn container_status_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        if let Err(response) = parse_params::<ContainerImagesParams>(
            request_id.as_ref(),
            params,
            "container.status não aceita parametros",
        ) {
            return *response;
        }
        // `podman info` + `version` levam ~2,4 s numa maquina (40 §7.64), e
        // a escolha pergunta `ps` a cada motor: fora do laco, como o
        // tools.detect (F6-b).
        let detector = self.detector.clone();
        let root = self.workspace_root();
        self.defer_work(request_id, move |request_id| {
            let selected = container::select_with(&detector);
            JsonRpcResponse::success(
                request_id,
                json!(container::status(selected.as_ref(), root.as_deref())),
            )
        })
    }

    /// `container.list` — os containers, parados inclusive por padrao. Fora
    /// do laco: um motor inalcancavel travava o core (32 s, defeito 4).
    fn container_list_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let pedido = match parse_params::<ContainerListParams>(
            request_id.as_ref(),
            params,
            "container.list aceita só o campo opcional `all`",
        ) {
            Ok(pedido) => pedido,
            Err(response) => return *response,
        };
        let detector = self.detector.clone();
        self.defer_work(request_id, move |request_id| {
            match container::answering(&detector) {
                None => no_engine_response(request_id, "container.list"),
                Some(Err(failure)) => engine_failure_response(request_id, &failure, ""),
                Some(Ok(engine)) => match container::list(&engine, pedido.all) {
                    Ok(listing) => JsonRpcResponse::success(request_id, json!(listing)),
                    Err(failure) => engine_failure_response(request_id, &failure, ""),
                },
            }
        })
    }

    /// `container.images` — as imagens locais, fora do laco como o `list`.
    fn container_images_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        if let Err(response) = parse_params::<ContainerImagesParams>(
            request_id.as_ref(),
            params,
            "container.images não aceita parametros",
        ) {
            return *response;
        }
        let detector = self.detector.clone();
        self.defer_work(request_id, move |request_id| {
            match container::answering(&detector) {
                None => no_engine_response(request_id, "container.images"),
                Some(Err(failure)) => engine_failure_response(request_id, &failure, ""),
                Some(Ok(engine)) => match container::images(&engine) {
                    Ok(images) => JsonRpcResponse::success(request_id, json!(images)),
                    Err(failure) => engine_failure_response(request_id, &failure, ""),
                },
            }
        })
    }

    /// `container.volumes` — os volumes que o container monta, com o tamanho
    /// quando o motor o da' (16b.3): a pergunta do "Remover". Fora do laco.
    fn container_volumes_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let query = match parse_params::<ContainerVolumesParams>(
            request_id.as_ref(),
            params,
            "container.volumes requer o campo id",
        ) {
            Ok(query) => query,
            Err(response) => return *response,
        };
        let detector = self.detector.clone();
        self.defer_work(request_id, move |request_id| {
            match container::answering(&detector) {
                None => no_engine_response(request_id, "container.volumes"),
                Some(Err(failure)) => engine_failure_response(request_id, &failure, ""),
                Some(Ok(engine)) => match container::volumes::volumes(&engine, &query.id) {
                    Ok(volumes) => JsonRpcResponse::success(
                        request_id,
                        json!(ContainerVolumesResult {
                            id: query.id,
                            volumes
                        }),
                    ),
                    Err(reason) => JsonRpcResponse::failure(
                        request_id,
                        JsonRpcError::new(
                            JsonRpcErrorCode::InvalidParams,
                            format!("container.volumes {}: {reason}", query.id),
                            None,
                        ),
                    ),
                },
            }
        })
    }

    /// `container.action` — start/stop/restart/rm como JOB cancelavel.
    fn container_action_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let query = match parse_params::<ContainerActionParams>(
            request_id.as_ref(),
            params,
            "container.action requer os campos id e action (start|stop|restart|remove)",
        ) {
            Ok(query) => query,
            Err(response) => return *response,
        };
        // `rm -v` so' faz sentido no `remove` (16b.3): parar "com os volumes"
        // nao existe, e aceitar calado esconderia o engano de quem pediu.
        if query.volumes && query.action != kinein_protocol::ContainerAction::Remove {
            return JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(
                    JsonRpcErrorCode::InvalidParams,
                    "container.action: `volumes` só vale com `remove`",
                    None,
                ),
            );
        }
        if container::detect_with(&self.detector).is_none() {
            return no_engine_response(request_id, "container.action");
        }
        let Some(jobs) = self.jobs.as_ref() else {
            return jobs_unavailable_response(request_id, "container.action");
        };
        let detector = self.detector.clone();
        let verb = query.action.verb().to_owned();
        let risco = match query.action {
            kinein_protocol::ContainerAction::Remove => JobRisk::High,
            _ => JobRisk::Medium,
        };
        let titulo = format!("{} {}", verb, query.id);
        let (action, volumes) = (query.action, query.volumes);
        let target = query.id;
        let job_id =
            jobs.spawn(
                "container",
                titulo,
                risco,
                true,
                move |ctx| match container::select_with(&detector) {
                    Some((engine, _)) => {
                        let command = container::action_command(&engine, action, &target, volumes);
                        run_and_report(ctx, command, &verb, &target)
                    }
                    None => report_no_engine(ctx, &verb, &target),
                },
            );
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
        let Some((engine, _)) = container::select_with(&self.detector) else {
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
        // Sem `-f` a ferramenta procura o arquivo padrao na raiz; se nao ha'
        // nenhum, o job so' poderia falhar (medido em 2026-09-13: exit 255 do
        // podman-compose) — a recusa vem ANTES, com o que falta.
        if pedido.file.is_none() && container::compose_file_in(&root).is_none() {
            return JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(
                    JsonRpcErrorCode::InvalidParams,
                    "o projeto não tem arquivo de compose (compose.yaml, docker-compose.yml, \
                     container-compose.yml...); crie um na raiz ou passe `file`",
                    Some(json!({ "root": root.display().to_string() })),
                ),
            );
        }
        let Some(engine) = container::detect_with(&self.detector) else {
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
        let verb = match pedido.action {
            kinein_protocol::ComposeAction::Up => "compose up",
            kinein_protocol::ComposeAction::Down => "compose down",
        }
        .to_owned();
        let target = pedido
            .file
            .clone()
            .unwrap_or_else(|| "compose (padrão)".to_owned());
        let titulo = format!("{verb} {target}");
        let detector = self.detector.clone();
        let (action, file) = (pedido.action, pedido.file);
        let job_id = jobs.spawn("container", titulo, JobRisk::Medium, true, move |ctx| {
            // O motor que responde pode ser outro que o primeiro do PATH (o
            // Docker negando, o Podman ao lado): o compose e' o dele.
            let tool = container::select_with(&detector)
                .filter(|(_, check)| check.ok)
                .and_then(|(engine, _)| container::compose_tool(&engine))
                .unwrap_or(compose_tool);
            let command = container::compose_command(&tool, action, file.as_deref(), &root);
            run_and_report(ctx, command, &verb, &target)
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

/// O motor existe e nao respondeu: erro com a dica e a saida crua, que a
/// janela mostra — nunca uma lista vazia (defeito 3). `prefix` diz o que
/// deixou de ser feito (o Banco: "nada foi apagado"), ou vem vazio.
pub(crate) fn engine_failure_response(
    request_id: Option<Value>,
    failure: &EngineFailure,
    prefix: &str,
) -> JsonRpcResponse {
    JsonRpcResponse::failure(
        request_id,
        JsonRpcError::new(
            JsonRpcErrorCode::ToolNotFound,
            format!("{prefix}{}", failure.message()),
            Some(json!({
                "engine": failure.engine,
                "reachable": false,
                "hint": failure.hint,
                "rawOutput": failure.raw_output,
            })),
        ),
    )
}

/// O motor sumiu entre o aceite e o job: o termino diz por que.
fn report_no_engine(ctx: &crate::jobs::JobContext, verb: &str, target: &str) -> JobOutcome {
    ctx.emit_event(
        "event.container.finished",
        json!(ContainerFinishedEvent {
            job_id: ctx.id().to_owned(),
            action: verb.to_owned(),
            target: target.to_owned(),
            ok: false,
            message: "nenhum motor de container (docker ou podman) no PATH".to_owned(),
        }),
    );
    JobOutcome::Failed
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
    verb: &str,
    target: &str,
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
                crate::process::ProcessError::Spawn(e) => format!("não foi possível iniciar: {e}"),
                crate::process::ProcessError::Wait(e) => format!("não foi possível esperar: {e}"),
            });
            false
        }
    };
    ctx.emit_event(
        "event.container.finished",
        json!(ContainerFinishedEvent {
            job_id: ctx.id().to_owned(),
            action: verb.to_owned(),
            target: target.to_owned(),
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

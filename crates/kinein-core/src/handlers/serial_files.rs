//! `serial.files` (`impl Core`): os arquivos na placa `MicroPython` pelo
//! `mpremote fs`, como JOB (C2 do `roadmaps/41` bloco C, 2026-09-17).
//!
//! Fino: valida, recusa antes de tocar a porta, sobe o job; a linha e o
//! parser sao do [`crate::serial::files`], o processo com prazo e' do
//! [`crate::serial::job`]. O desfecho sai em `event.serial.files`.

use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use kinein_protocol::{
    JobRisk, JsonRpcError, JsonRpcErrorCode, JsonRpcResponse, SerialFilesAction, SerialFilesEvent,
    SerialFilesParams, SerialFilesResult,
};
use serde_json::{Value, json};

use super::serial::porta_pronta;
use crate::rpc::{jobs_unavailable_response, parse_params};
use crate::serial::{files, job};
use crate::{Core, jobs};

/// Quanto um `fs` tem: o mpremote copia a ~10 KB/s a 115200 e o raw REPL
/// entra em segundos; dois minutos cobrem um arquivo grande e ainda dizem
/// "nao respondeu" em vez de prender a porta.
const FILES_TIMEOUT: Duration = Duration::from_secs(120);

impl Core {
    /// `serial.files` — nao exige workspace para `list`/`rm`/`mkdir`; para
    /// `get`/`put` um `local` relativo e' sob o workspace (sem workspace,
    /// so' absoluto). Recusa ANTES de tocar a porta: pedido invalido,
    /// porta inexistente ou sem acesso, `mpremote` ausente.
    pub(crate) fn serial_files_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let pedido = match parse_params::<SerialFilesParams>(
            request_id.as_ref(),
            params,
            "serial.files requer device e action (list|get|put|rm|mkdir); aceita path, local e tool",
        ) {
            Ok(pedido) => pedido,
            Err(response) => return *response,
        };
        let local = match self.local_resolvido(pedido.local.as_deref()) {
            Ok(local) => local,
            Err(mensagem) => return recusa(request_id, &mensagem, &pedido),
        };
        // `get` cria a pasta de destino (a tela baixa para `placa/<caminho>`
        // sob o workspace — o espelho da placa, que nunca colide com o codigo
        // do projeto e pode ser baixado de novo).
        if pedido.action == SerialFilesAction::Get
            && let Some(pasta) = local.as_deref().and_then(Path::parent)
            && !pasta.as_os_str().is_empty()
            && let Err(erro) = std::fs::create_dir_all(pasta)
        {
            return recusa(
                request_id,
                &format!("nao consegui criar {}: {erro}", pasta.display()),
                &pedido,
            );
        }
        if let Err(mensagem) =
            files::validate(pedido.action, pedido.path.as_deref(), local.as_deref())
        {
            return recusa(request_id, &mensagem, &pedido);
        }
        if let Err(recusa) = porta_pronta(request_id.as_ref(), "serial.files", &pedido.device) {
            return *recusa;
        }
        let Some(mpremote) = pedido
            .tool
            .as_deref()
            .map(PathBuf::from)
            .filter(|p| p.is_file())
            .or_else(|| self.detector.find_in_path("mpremote"))
        else {
            return JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(
                    JsonRpcErrorCode::ToolNotFound,
                    "mpremote nao esta' nesta maquina: `pipx install mpremote` (o painel de \
                     instalacao mostra o passo oficial)",
                    Some(json!({ "tool": "mpremote" })),
                ),
            );
        };
        let Some(jobs) = self.jobs.as_ref() else {
            return jobs_unavailable_response(request_id, "serial.files");
        };
        let device = pedido.device.clone();
        let action = pedido.action;
        let path = pedido.path.unwrap_or_default().trim().to_owned();
        let (programa, args) =
            files::command_line(&mpremote, &device, action, &path, local.as_deref());
        let comando = job::display(&programa, &args);
        let titulo = match action {
            SerialFilesAction::List => format!("Listar arquivos em {device}"),
            SerialFilesAction::Get => format!("Baixar {path} de {device}"),
            SerialFilesAction::Put => format!("Enviar {path} para {device}"),
            SerialFilesAction::Rm => format!("Apagar {path} em {device}"),
            SerialFilesAction::Mkdir => format!("Criar pasta {path} em {device}"),
        };
        // Medio: interrompe o programa da placa (raw REPL + soft reset);
        // Alto quando escreve na flash dela.
        let risco = if action.writes_board() {
            JobRisk::High
        } else {
            JobRisk::Medium
        };
        let job_id = jobs.spawn("serial.files", titulo, risco, true, move |ctx| {
            executar(
                ctx,
                &mpremote,
                &args,
                &device,
                action,
                &path,
                local.as_deref(),
            )
        });
        JsonRpcResponse::success(
            request_id,
            json!(SerialFilesResult {
                job_id,
                command: comando,
            }),
        )
    }

    /// `local` relativo e' sob o workspace; sem workspace, so' absoluto.
    fn local_resolvido(&self, local: Option<&str>) -> Result<Option<PathBuf>, String> {
        let Some(local) = local.map(str::trim).filter(|l| !l.is_empty()) else {
            return Ok(None);
        };
        let caminho = PathBuf::from(local);
        if caminho.is_absolute() {
            return Ok(Some(caminho));
        }
        self.workspace_root().map_or_else(
            || {
                Err(format!(
                    "`local` relativo ({local}) exige um workspace aberto; sem ele, use um \
                     caminho absoluto"
                ))
            },
            |root| Ok(Some(root.join(caminho))),
        )
    }
}

fn recusa(
    request_id: Option<Value>,
    mensagem: &str,
    pedido: &SerialFilesParams,
) -> JsonRpcResponse {
    JsonRpcResponse::failure(
        request_id,
        JsonRpcError::new(
            JsonRpcErrorCode::InvalidParams,
            format!("serial.files: {mensagem}"),
            Some(json!({ "device": pedido.device, "action": pedido.action })),
        ),
    )
}

/// O corpo do job: roda o `fs`, repete UMA vez se a placa nao deixou entrar
/// no raw REPL (medido: o programa dela inundando a UART), le a saida e
/// emite o desfecho.
fn executar(
    ctx: &jobs::JobContext,
    mpremote: &Path,
    args: &[String],
    device: &str,
    action: SerialFilesAction,
    path: &str,
    local: Option<&Path>,
) -> jobs::JobOutcome {
    let mut tentativa = job::rodar(ctx, mpremote, args, FILES_TIMEOUT, "mpremote");
    if !tentativa.sucesso && files::raw_repl_failed(&tentativa.raw) && !ctx.is_cancelled() {
        ctx.emit_output(
            "a placa nao deixou entrar no raw REPL (o programa dela ocupa a UART): tentando de novo",
        );
        let primeira = tentativa.raw;
        tentativa = job::rodar(ctx, mpremote, args, FILES_TIMEOUT, "mpremote");
        // As duas saidas viajam: a tela mostra por que a primeira morreu.
        tentativa.raw = format!("{primeira}--- segunda tentativa ---\n{}", tentativa.raw);
    }
    let success = tentativa.sucesso;
    let error = if success {
        None
    } else if ctx.is_cancelled() {
        Some("operacao cancelada".to_owned())
    } else if tentativa.expirou {
        Some(format!(
            "o mpremote nao respondeu em {} s: a placa esta' plugada nesta porta e sem outro \
             programa (monitor serial, REPL) segurando-a?",
            FILES_TIMEOUT.as_secs()
        ))
    } else {
        Some(
            files::error_line(&tentativa.raw)
                .unwrap_or_else(|| job::ultimas_linhas(&tentativa.raw, "mpremote")),
        )
    };
    let entries =
        (success && action == SerialFilesAction::List).then(|| files::parse_ls(&tentativa.raw));
    ctx.emit_event(
        "event.serial.files",
        json!(SerialFilesEvent {
            job_id: ctx.id().to_owned(),
            device: device.to_owned(),
            action,
            path: path.to_owned(),
            command: tentativa.comando,
            success,
            error,
            entries,
            local: local.map(|l| l.display().to_string()),
            raw: tentativa.raw,
        }),
    );
    if success {
        jobs::JobOutcome::Success
    } else {
        jobs::JobOutcome::Failed
    }
}

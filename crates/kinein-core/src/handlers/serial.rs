//! Handler for `serial.*` requests (`impl Core`).
//!
//! Fino: roteia, valida params, delega ao `crate::serial` e formata a resposta.

use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use kinein_protocol::{
    JobRisk, JsonRpcError, JsonRpcErrorCode, JsonRpcResponse, SerialAccessParams,
    SerialIdentifiedEvent, SerialIdentifyParams, SerialIdentifyResult, SerialListParams,
    SerialMonitorParams, SerialMonitorResult,
};
use serde_json::{Value, json};

use crate::rpc::{
    jobs_unavailable_response, no_workspace_response, parse_params, terminal_error_response,
    terminal_unavailable_response,
};
use crate::serial::{identify, job, monitor};
use crate::{Core, jobs};

/// Quanto o esptool tem para achar o chip. Ele mesmo tenta sincronizar varias
/// vezes (~7 tentativas de ~1 s com a placa muda); trinta segundos cobrem uma
/// placa em modo estranho e ainda dizem "nao respondeu" em vez de esperar
/// para sempre com a porta presa.
const IDENTIFY_TIMEOUT: Duration = Duration::from_secs(30);

impl Core {
    /// Roteia os metodos `serial.*`.
    pub(crate) fn serial_request_response(
        &mut self,
        method: &str,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> Option<JsonRpcResponse> {
        match method {
            "serial.list" => Some(Self::serial_list_response(request_id, params)),
            "serial.monitor" => Some(self.serial_monitor_response(request_id, params)),
            "serial.identify" => Some(self.serial_identify_response(request_id, params)),
            "serial.access" => Some(Self::serial_access_response(request_id, params)),
            "serial.files" => Some(self.serial_files_response(request_id, params)),
            _ => None,
        }
    }

    /// `serial.monitor` — o monitor como PROCESSO numa aba de terminal (E3 do
    /// `integracoes/38` §6). Exige workspace: a ferramenta vem do kit e a aba
    /// nasce com o cwd do projeto, como o `terminal.open`.
    fn serial_monitor_response(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let pedido = match parse_params::<SerialMonitorParams>(
            request_id.as_ref(),
            params,
            "serial.monitor requer o campo device e aceita baud",
        ) {
            Ok(pedido) => pedido,
            Err(response) => return *response,
        };
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "serial.monitor");
        };
        let toolchain = crate::toolchain::Toolchain::resolve(&root, &self.detected_tools());
        // Num projeto MicroPython o monitor e' o REPL do mpremote (fatia 5 da
        // cadeia Python): a evidencia vem do mesmo detector do project.model.
        let micropython = crate::project::e_micropython(&root);
        // O wrapper do framework (bloco E): o IDF Monitor num projeto ESP-IDF
        // ativavel, o `pio device monitor` num PlatformIO.
        let framework = self.framework_engine(&root).ok().flatten();
        let Some(escolha) = monitor::escolher(&toolchain, micropython, framework.as_ref()) else {
            return JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(
                    JsonRpcErrorCode::ToolNotFound,
                    "nenhum monitor serial nesta maquina: instale tio, picocom, minicom, \
                     espflash ou mpremote — ou fixe um no papel `serialMonitor` do kit",
                    Some(json!({ "method": "serial.monitor" })),
                ),
            );
        };
        // O ELF so' importa ao espflash (decodifica o backtrace); ausente, o
        // monitor e' so' texto — e isso e' dito pela linha de comando da aba.
        let elf = if escolha.id == "espflash" {
            self.workspace
                .as_ref()
                .and_then(|w| crate::dap::resolve_program(w.kind, &root).ok())
                .and_then(|alvo| alvo.program_path().map(std::path::Path::to_path_buf))
        } else {
            None
        };
        let baud = pedido.baud.unwrap_or(monitor::DEFAULT_BAUD);
        let (programa, args) = monitor::command_line(
            &escolha.id,
            &escolha.program,
            &pedido.device,
            baud,
            elf.as_deref(),
            escolha.activation.as_deref(),
        );
        let Some(session) = self.terminal.as_mut() else {
            return terminal_unavailable_response(request_id, "serial.monitor");
        };
        match session.open_command(&root, &programa, &args) {
            Ok(id) => JsonRpcResponse::success(
                request_id,
                json!(SerialMonitorResult {
                    id,
                    command: format!("{programa} {}", args.join(" ")),
                    tool: escolha.id,
                }),
            ),
            Err(error) => terminal_error_response(request_id, &error),
        }
    }

    /// `serial.access` — permissao POR CANAL (E2 do `integracoes/38` §6):
    /// o que falta em cada porta (grupo/ACL, `ModemManager`) e na regra das
    /// sondas, com o passo oficial. Nao exige workspace, como o `serial.list`;
    /// nao roda nada — o passo e' escrito no terminal da IDE pelo usuario.
    fn serial_access_response(
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let pedido = match parse_params::<SerialAccessParams>(
            request_id.as_ref(),
            params,
            "serial.access aceita o campo opcional device",
        ) {
            Ok(pedido) => pedido,
            Err(response) => return *response,
        };
        JsonRpcResponse::success(
            request_id,
            json!(crate::serial::access::diagnose(pedido.device.as_deref())),
        )
    }

    /// `serial.list` — as portas seriais USB desta maquina.
    ///
    /// NAO exige workspace, ao contrario do `probe.list`: nao ha' ferramenta
    /// vinda do kit aqui — e' o sysfs desta maquina, o mesmo com ou sem
    /// projeto aberto. Exigir projeto para dizer o que esta' no USB seria
    /// esconder informacao que a IDE ja' tem.
    fn serial_list_response(request_id: Option<Value>, params: Option<&Value>) -> JsonRpcResponse {
        if let Err(response) = parse_params::<SerialListParams>(
            request_id.as_ref(),
            params,
            "serial.list nao aceita parametros",
        ) {
            return *response;
        }
        JsonRpcResponse::success(request_id, json!(crate::serial::list()))
    }

    /// `serial.identify` — a identidade Espressif PELO CANAL (E5 do
    /// `integracoes/38` §6): `esptool flash-id` num JOB, porque prende a porta
    /// e pode levar segundos; o desfecho sai em `event.serial.identified`.
    ///
    /// Nao exige workspace (como o `serial.list`): o chip e' desta maquina, e
    /// a sugestao de kit que sai com ele so' vira kit no clique do usuario.
    /// Recusa ANTES de abrir a porta o que vai falhar: ferramenta ausente,
    /// no' inexistente, sem permissao — abrir reseta a placa, e resetar por
    /// nada e' defeito.
    fn serial_identify_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let pedido = match parse_params::<SerialIdentifyParams>(
            request_id.as_ref(),
            params,
            "serial.identify requer o campo device e aceita tool",
        ) {
            Ok(pedido) => pedido,
            Err(response) => return *response,
        };
        if let Err(recusa) = porta_pronta(request_id.as_ref(), "serial.identify", &pedido.device) {
            return *recusa;
        }
        let Some(esptool) = pedido
            .tool
            .as_deref()
            .map(PathBuf::from)
            .filter(|p| p.is_file())
            .or_else(|| self.detector.find_in_path("esptool"))
            .or_else(|| self.detector.find_in_path("esptool.py"))
        else {
            return JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(
                    JsonRpcErrorCode::ToolNotFound,
                    "esptool nao esta' nesta maquina: `pipx install esptool` (o painel de \
                     instalacao mostra o passo oficial)",
                    Some(json!({ "tool": "esptool" })),
                ),
            );
        };
        let Some(jobs) = self.jobs.as_ref() else {
            return jobs_unavailable_response(request_id, "serial.identify");
        };
        let device = pedido.device;
        let (programa, args) = identify::command_line(&esptool, &device, identify::FLASH_ID_V5);
        let comando = identify::display(&programa, &args);
        let job_id = jobs.spawn(
            "serial.identify",
            format!("Identificar {device}"),
            // Medio: reseta a placa (DTR/RTS), nao escreve nada nela.
            JobRisk::Medium,
            true,
            move |ctx| identificar(ctx, &esptool, &device),
        );
        JsonRpcResponse::success(
            request_id,
            json!(SerialIdentifyResult {
                job_id,
                command: comando,
            }),
        )
    }
}

/// O corpo do job: roda o `flash-id` (v5; cai para o `flash_id` da v4 se a
/// ferramenta nao conhece o comando), le a saida e emite o desfecho.
fn identificar(ctx: &jobs::JobContext, esptool: &Path, device: &str) -> jobs::JobOutcome {
    let mut tentativa = rodar(ctx, esptool, device, identify::FLASH_ID_V5);
    if !tentativa.sucesso && identify::unknown_subcommand(&tentativa.raw) && !ctx.is_cancelled() {
        ctx.emit_output("a ferramenta nao conhece `flash-id`: tentando `flash_id` (esptool v4)");
        tentativa = rodar(ctx, esptool, device, identify::FLASH_ID_V4);
    }
    let identity = identify::parse_flash_id(&tentativa.raw);
    let leu_chip = identity.chip.is_some();
    let success = tentativa.sucesso && leu_chip;
    let error = if success {
        None
    } else if ctx.is_cancelled() {
        Some("identificacao cancelada".to_owned())
    } else if tentativa.expirou {
        Some(format!(
            "o esptool nao respondeu em {} s: a placa esta' plugada nesta porta e sem outro \
             programa (monitor serial, ModemManager) segurando-a?",
            IDENTIFY_TIMEOUT.as_secs()
        ))
    } else if tentativa.sucesso {
        Some("o esptool terminou sem dizer `Chip type:` — veja a saida crua".to_owned())
    } else {
        Some(job::ultimas_linhas(&tentativa.raw, "esptool"))
    };
    let target = identity
        .chip
        .as_deref()
        .map(|chip| crate::project::alvo_do_chip(chip, "esptool flash-id pela porta serial"));
    ctx.emit_event(
        "event.serial.identified",
        json!(SerialIdentifiedEvent {
            job_id: ctx.id().to_owned(),
            device: device.to_owned(),
            command: tentativa.comando,
            success,
            error,
            identity: leu_chip.then_some(identity),
            target,
            raw: tentativa.raw,
        }),
    );
    if success {
        jobs::JobOutcome::Success
    } else {
        jobs::JobOutcome::Failed
    }
}

/// Uma execucao do esptool com o prazo de identificacao (`serial/job.rs`
/// faz o trabalho; aqui so' a linha de comando do `flash-id`).
fn rodar(ctx: &jobs::JobContext, esptool: &Path, device: &str, subcomando: &str) -> job::Tentativa {
    let (_, args) = identify::command_line(esptool, device, subcomando);
    job::rodar(ctx, esptool, &args, IDENTIFY_TIMEOUT, "esptool")
}

/// A recusa ANTES de abrir a porta, comum a `serial.identify` e
/// `serial.files`: no' inexistente (`INVALID_PARAMS`) e sem leitura/escrita
/// (`INVALID_REQUEST`, com a `hint` do `serial.list`). Abrir a porta reseta
/// ou interrompe a placa; abrir para falhar e' defeito.
pub(super) fn porta_pronta(
    request_id: Option<&Value>,
    method: &str,
    device: &str,
) -> Result<(), Box<JsonRpcResponse>> {
    let no = Path::new(device);
    if device.trim().is_empty() || !no.exists() {
        return Err(Box::new(JsonRpcResponse::failure(
            request_id.cloned(),
            JsonRpcError::new(
                JsonRpcErrorCode::InvalidParams,
                format!("{method}: `{device}` nao existe nesta maquina"),
                Some(json!({ "device": device })),
            ),
        )));
    }
    let acesso = crate::serial::acesso_de(no);
    if !acesso.readable_writable {
        return Err(Box::new(JsonRpcResponse::failure(
            request_id.cloned(),
            JsonRpcError::new(
                JsonRpcErrorCode::InvalidRequest,
                acesso
                    .hint
                    .unwrap_or_else(|| format!("sem permissao de leitura/escrita em {device}")),
                Some(json!({ "device": device, "mode": acesso.mode })),
            ),
        )));
    }
    Ok(())
}

//! Handler for `serial.*` requests (`impl Core`).
//!
//! Fino: roteia, valida params, delega ao `crate::serial` e formata a resposta.

use kinein_protocol::{
    JsonRpcError, JsonRpcErrorCode, JsonRpcResponse, SerialListParams, SerialMonitorParams,
    SerialMonitorResult,
};
use serde_json::{Value, json};

use crate::Core;
use crate::rpc::{
    no_workspace_response, parse_params, terminal_error_response, terminal_unavailable_response,
};
use crate::serial::monitor;

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
        let Some(escolha) = monitor::escolher(&toolchain, micropython) else {
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
}

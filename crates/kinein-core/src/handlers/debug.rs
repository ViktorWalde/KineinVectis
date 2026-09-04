//! Handlers for `debug.*` requests (`impl Core`).
//!
//! O router `debug.*` mais start/setBreakpoints/controle de execucao,
//! dirigindo a sessao DAP gerida por `crate::dap`. `start` e
//! `setBreakpoints` exigem workspace; os controles de sessao (continue,
//! step, pause, stop) so exigem o manager, espelhando `run.stdin`/`run.stop`.

use std::path::{Path, PathBuf};

use kinein_protocol::{
    DebugBreakpointsResult, DebugEvaluateParams, DebugSetBreakpointsParams, DebugStackTraceResult,
    DebugStartParams, DebugStartResult, DebugVariablesParams, DebugVariablesResult, JsonRpcError,
    JsonRpcErrorCode, JsonRpcResponse,
};
use serde_json::{Value, json};

use crate::rpc::{
    debug_error_response, debug_unavailable_response, no_workspace_response, parse_params,
};
use crate::{Core, dap};

impl Core {
    /// Roteia os metodos `debug.*`; `None` quando o metodo nao e de debug.
    pub(crate) fn debug_request_response(
        &mut self,
        method: &str,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> Option<JsonRpcResponse> {
        match method {
            "debug.start" => Some(self.debug_start_response(request_id, params)),
            "debug.setBreakpoints" => Some(self.debug_set_breakpoints_response(request_id, params)),
            "debug.stackTrace" => Some(self.debug_stack_trace_response(request_id)),
            "debug.variables" => Some(self.debug_variables_response(request_id, params)),
            "debug.evaluate" => Some(self.debug_evaluate_response(request_id, params)),
            "debug.continue" | "debug.next" | "debug.stepIn" | "debug.stepOut" | "debug.pause"
            | "debug.stop" => Some(self.debug_session_op_response(request_id, method)),
            _ => None,
        }
    }

    fn debug_start_response(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(workspace) = self.workspace.clone() else {
            return no_workspace_response(request_id, "debug.start");
        };
        let parsed = match parse_params::<DebugStartParams>(
            request_id.as_ref(),
            params,
            "debug.start aceita apenas o campo opcional program",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        if self.debug.is_none() {
            return debug_unavailable_response(request_id, "debug.start");
        }

        let root = Path::new(&workspace.root);
        let program = match parsed.program.filter(|program| !program.trim().is_empty()) {
            Some(explicit) => {
                let path = PathBuf::from(explicit);
                if !path.is_file() {
                    return invalid_debug_params(
                        request_id,
                        &format!("programa nao encontrado: {}", path.display()),
                    );
                }
                path
            }
            None => match dap::resolve_program(workspace.kind, root) {
                Ok(program) => program,
                Err(error) => return debug_error_response(request_id, &error),
            },
        };

        // O kit decide QUAL adaptador sobe. Resolve-se antes de pegar o
        // manager emprestado: `Toolchain::resolve` le o disco e a lista de
        // ferramentas detectadas, e os dois pedem `&self`.
        let toolchain = crate::toolchain::Toolchain::resolve(root, &self.detected_tools());
        let adapter_id = toolchain
            .chosen(kinein_protocol::ToolchainRole::DebugAdapter)
            .map(str::to_owned);
        let adapter_path = toolchain.program_for(kinein_protocol::ToolchainRole::DebugAdapter);
        let chip = toolchain.chip().map(str::to_owned);

        let Some(manager) = self.debug.as_mut() else {
            return debug_unavailable_response(request_id, "debug.start");
        };
        match manager.start(
            root,
            &program,
            adapter_id.as_deref(),
            adapter_path.as_deref(),
            chip.as_deref(),
        ) {
            Ok(()) => JsonRpcResponse::success(
                request_id,
                json!(DebugStartResult {
                    program: program.display().to_string(),
                }),
            ),
            Err(error) => debug_error_response(request_id, &error),
        }
    }

    fn debug_set_breakpoints_response(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(workspace) = self.workspace.clone() else {
            return no_workspace_response(request_id, "debug.setBreakpoints");
        };
        let parsed = match parse_params::<DebugSetBreakpointsParams>(
            request_id.as_ref(),
            params,
            "debug.setBreakpoints requer os campos file e lines",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let Ok(file) = Path::new(&parsed.file).canonicalize() else {
            return invalid_debug_params(
                request_id,
                &format!("arquivo nao encontrado: {}", parsed.file),
            );
        };
        if !file.starts_with(&workspace.root) {
            return invalid_debug_params(request_id, "arquivo fora do workspace aberto");
        }
        let Some(file) = file.to_str().map(str::to_owned) else {
            return invalid_debug_params(request_id, "caminho de arquivo invalido");
        };

        let Some(manager) = self.debug.as_mut() else {
            return debug_unavailable_response(request_id, "debug.setBreakpoints");
        };
        match manager.set_breakpoints(&file, &parsed.breakpoints) {
            Ok(breakpoints) => {
                JsonRpcResponse::success(request_id, json!(DebugBreakpointsResult { breakpoints }))
            }
            Err(error) => debug_error_response(request_id, &error),
        }
    }

    fn debug_stack_trace_response(&self, request_id: Option<Value>) -> JsonRpcResponse {
        let Some(manager) = self.debug.as_ref() else {
            return debug_unavailable_response(request_id, "debug.stackTrace");
        };
        match manager.stack_trace() {
            Ok(frames) => {
                JsonRpcResponse::success(request_id, json!(DebugStackTraceResult { frames }))
            }
            Err(error) => debug_error_response(request_id, &error),
        }
    }

    /// `debug.evaluate` — um watch no frame parado.
    ///
    /// Sem `frameId`, o core usa o frame do topo: e' o que a UI quer quando o
    /// usuario digita a expressao sem ter escolhido um frame. Expressao vazia
    /// e' recusada aqui, no lugar de virar um erro do adapter — a mensagem do
    /// lldb para string vazia nao ajuda ninguem.
    fn debug_evaluate_response(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let parsed = match parse_params::<DebugEvaluateParams>(
            request_id.as_ref(),
            params,
            "debug.evaluate requer expression",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        if parsed.expression.trim().is_empty() {
            return invalid_debug_params(request_id, "debug.evaluate requer expression nao vazia");
        }
        let Some(manager) = self.debug.as_mut() else {
            return debug_unavailable_response(request_id, "debug.evaluate");
        };
        let expression = parsed.expression.trim().to_owned();
        match manager.evaluate(&expression, parsed.frame_id) {
            Ok(result) => JsonRpcResponse::success(request_id, json!(result)),
            // A expressao vai nos `details` porque a UI precisa saber QUAL
            // watch falhou: a resposta de erro nao carrega o pedido, e sem
            // isto a falha de um watch marcaria todos.
            Err(error) => {
                let mut response = debug_error_response(request_id, &error);
                if let Some(rpc_error) = response.error.as_mut() {
                    rpc_error.details = Some(json!({ "expression": expression }));
                }
                response
            }
        }
    }

    fn debug_variables_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let parsed = match parse_params::<DebugVariablesParams>(
            request_id.as_ref(),
            params,
            "debug.variables requer exatamente um de frameId ou ref",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let Some(manager) = self.debug.as_ref() else {
            return debug_unavailable_response(request_id, "debug.variables");
        };
        let outcome = match (parsed.frame_id, parsed.reference) {
            (Some(frame_id), None) => {
                manager
                    .frame_variables(frame_id)
                    .map(|variables| DebugVariablesResult {
                        frame_id: Some(frame_id),
                        reference: None,
                        variables,
                    })
            }
            (None, Some(reference)) => {
                manager
                    .reference_variables(reference)
                    .map(|variables| DebugVariablesResult {
                        frame_id: None,
                        reference: Some(reference),
                        variables,
                    })
            }
            _ => {
                return invalid_debug_params(
                    request_id,
                    "debug.variables requer exatamente um de frameId ou ref",
                );
            }
        };
        match outcome {
            Ok(result) => JsonRpcResponse::success(request_id, json!(result)),
            Err(error) => debug_error_response(request_id, &error),
        }
    }

    fn debug_session_op_response(
        &mut self,
        request_id: Option<Value>,
        method: &str,
    ) -> JsonRpcResponse {
        let Some(manager) = self.debug.as_mut() else {
            return debug_unavailable_response(request_id, method);
        };
        let outcome = match method {
            "debug.continue" => manager.continue_run(),
            "debug.next" => manager.step_over(),
            "debug.stepIn" => manager.step_in(),
            "debug.stepOut" => manager.step_out(),
            "debug.pause" => manager.pause(),
            _ => manager.stop(),
        };
        match outcome {
            Ok(()) => JsonRpcResponse::success(request_id, json!({ "status": "ok" })),
            Err(error) => debug_error_response(request_id, &error),
        }
    }
}

/// Falha `INVALID_PARAMS` com a mensagem de dominio do debug.
fn invalid_debug_params(request_id: Option<Value>, message: &str) -> JsonRpcResponse {
    JsonRpcResponse::failure(
        request_id,
        JsonRpcError::new(JsonRpcErrorCode::InvalidParams, message, None),
    )
}

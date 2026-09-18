//! Handlers for `debug.*` requests (`impl Core`).
//!
//! O router `debug.*` mais start/setBreakpoints/controle de execucao,
//! dirigindo a sessao DAP gerida por `crate::dap`. `start` e
//! `setBreakpoints` exigem workspace; os controles de sessao (continue,
//! step, pause, stop) so exigem o manager, espelhando `run.stdin`/`run.stop`.

use std::path::{Path, PathBuf};

use kinein_protocol::{
    DebugBreakpointsResult, DebugDisassembleParams, DebugDisassembleResult, DebugEvaluateParams,
    DebugReadMemoryParams, DebugScopesParams, DebugScopesResult, DebugSetBreakpointsParams,
    DebugStackTraceResult, DebugStartParams, DebugStartResult, DebugVariablesParams,
    DebugVariablesResult, JsonRpcError, JsonRpcErrorCode, JsonRpcResponse,
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
            "debug.scopes" => Some(self.debug_scopes_response(request_id, params)),
            "debug.readMemory" => Some(self.debug_read_memory_response(request_id, params)),
            "debug.disassemble" => Some(self.debug_disassemble_response(request_id, params)),
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
            "debug.start aceita program ou connect { host, port }, nunca ambos",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        if self.debug.is_none() {
            return debug_unavailable_response(request_id, "debug.start");
        }

        let root = Path::new(&workspace.root);
        let target = if let Some(mut endpoint) = parsed.connect {
            if parsed.program.is_some() {
                return invalid_debug_params(
                    request_id,
                    "program e connect sao mutuamente exclusivos",
                );
            }
            endpoint.host = endpoint.host.trim().to_owned();
            if endpoint.host.is_empty()
                || endpoint
                    .host
                    .chars()
                    .any(|c| c.is_whitespace() || c.is_control())
                || endpoint.host.contains('/')
                || endpoint.port == 0
            {
                return invalid_debug_params(
                    request_id,
                    "connect requer host valido e port entre 1 e 65535",
                );
            }
            dap::DebugTarget::PythonAttach(endpoint)
        } else {
            match parsed.program.filter(|program| !program.trim().is_empty()) {
                Some(explicit) => {
                    let path = PathBuf::from(explicit);
                    if !path.is_file() {
                        return invalid_debug_params(
                            request_id,
                            &format!("programa nao encontrado: {}", path.display()),
                        );
                    }
                    dap::DebugTarget::Program(path)
                }
                None => match dap::resolve_program(workspace.kind, root) {
                    Ok(target) => target,
                    Err(error) => return debug_error_response(request_id, &error),
                },
            }
        };

        let toolchain = crate::toolchain::Toolchain::resolve(root, &self.detected_tools());
        let python = e_um_alvo_python(&target);
        let interpreter = if python && !target.is_attached() {
            match self.python_debug_interpreter(root) {
                Ok(path) => Some(path),
                Err(error) => return debug_error_response(request_id, &error),
            }
        } else {
            None
        };
        // Python never inherits a native kit's remoteTarget/debugServer/chip.
        // TCP attach uses the adapter that debugpy.listen already started.
        let native_path = toolchain.program_for(kinein_protocol::ToolchainRole::DebugAdapter);
        let choice = if python {
            dap::AdapterChoice {
                id: Some(dap::DEBUGPY),
                path: interpreter.as_deref(),
                ..dap::AdapterChoice::default()
            }
        } else {
            dap::AdapterChoice {
                id: toolchain.chosen(kinein_protocol::ToolchainRole::DebugAdapter),
                path: native_path.as_deref(),
                chip: toolchain.chip(),
                remote_target: toolchain.remote_target(),
                debug_server: toolchain.debug_server(),
                svd_file: toolchain.svd_file(),
            }
        };
        let Some(manager) = self.debug.as_mut() else {
            return debug_unavailable_response(request_id, "debug.start");
        };
        match manager.start(root, &target, &choice) {
            Ok(()) => JsonRpcResponse::success(
                request_id,
                json!(DebugStartResult {
                    program: target.display(),
                    attached: target.is_attached(),
                }),
            ),
            Err(error) => debug_error_response(request_id, &error),
        }
    }

    /// Launch needs debugpy in the project's interpreter; TCP attach does not.
    fn python_debug_interpreter(&self, root: &Path) -> Result<PathBuf, dap::DebugError> {
        let mut tools = self.python_tools();
        tools.medir_versao = false;
        let env = crate::python::env::python_env(root, &tools).ok_or_else(|| dap::DebugError::NoTarget {
            message: "sem interpretador Python para este workspace: crie o ambiente (.venv) pela faixa de saude ou instale o python3".to_owned(),
        })?;
        let interpreter = PathBuf::from(env.interpreter);
        crate::python::debug::debugpy_available(&interpreter)
            .map_err(|message| dap::DebugError::MissingAdapterModule { message })?;
        Ok(interpreter)
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

    /// `debug.scopes` (P3): os escopos de um frame, inteiros — o que o
    /// `debug.variables { frameId }` esconde ao escolher um.
    fn debug_scopes_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let parsed = match parse_params::<DebugScopesParams>(
            request_id.as_ref(),
            params,
            "debug.scopes requer frameId",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let Some(manager) = self.debug.as_ref() else {
            return debug_unavailable_response(request_id, "debug.scopes");
        };
        match manager.scopes(parsed.frame_id) {
            Ok(scopes) => JsonRpcResponse::success(
                request_id,
                json!(DebugScopesResult {
                    frame_id: parsed.frame_id,
                    scopes,
                }),
            ),
            Err(error) => debug_error_response(request_id, &error),
        }
    }

    /// `debug.readMemory` (P3): o `readMemory` do DAP, verbatim.
    fn debug_read_memory_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let parsed = match parse_params::<DebugReadMemoryParams>(
            request_id.as_ref(),
            params,
            "debug.readMemory requer memoryReference e count; aceita offset",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        if parsed.memory_reference.trim().is_empty() || parsed.count == 0 {
            return invalid_debug_params(
                request_id,
                "debug.readMemory requer memoryReference nao vazio e count > 0",
            );
        }
        let Some(manager) = self.debug.as_ref() else {
            return debug_unavailable_response(request_id, "debug.readMemory");
        };
        match manager.read_memory(&parsed.memory_reference, parsed.offset, parsed.count) {
            Ok(result) => JsonRpcResponse::success(request_id, json!(result)),
            Err(error) => debug_error_response(request_id, &error),
        }
    }

    /// `debug.disassemble` (P3): o `disassemble` do DAP, verbatim.
    fn debug_disassemble_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let parsed = match parse_params::<DebugDisassembleParams>(
            request_id.as_ref(),
            params,
            "debug.disassemble requer memoryReference e instructionCount; aceita offset e instructionOffset",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        if parsed.memory_reference.trim().is_empty() || parsed.instruction_count == 0 {
            return invalid_debug_params(
                request_id,
                "debug.disassemble requer memoryReference nao vazio e instructionCount > 0",
            );
        }
        let Some(manager) = self.debug.as_ref() else {
            return debug_unavailable_response(request_id, "debug.disassemble");
        };
        match manager.disassemble(&parsed) {
            Ok(instructions) => {
                JsonRpcResponse::success(request_id, json!(DebugDisassembleResult { instructions }))
            }
            Err(error) => debug_error_response(request_id, &error),
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

/// Um `.py` — ou um modulo Python — e' alvo do debugpy, seja qual for o tipo
/// do workspace (um projeto `CMake` com um `tools/gera.py` depura o script
/// com Python).
fn e_um_alvo_python(target: &dap::DebugTarget) -> bool {
    match target {
        dap::DebugTarget::Module(_) | dap::DebugTarget::PythonAttach(_) => true,
        dap::DebugTarget::Program(program) => program
            .extension()
            .and_then(std::ffi::OsStr::to_str)
            .is_some_and(|ext| ext.eq_ignore_ascii_case("py")),
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::e_um_alvo_python;
    use crate::dap::DebugTarget;

    /// So' o `.py` (em qualquer caixa) e o modulo vao para o debugpy; o
    /// binario nativo, o stub `.pyi` e o `.rs` seguem pelo kit.
    #[test]
    fn only_python_files_and_modules_are_python_targets() {
        let programa = |p: &str| DebugTarget::Program(Path::new(p).to_path_buf());
        assert!(e_um_alvo_python(&programa("/w/tools/gera.py")));
        assert!(e_um_alvo_python(&programa("/w/APP.PY")));
        assert!(e_um_alvo_python(&DebugTarget::Module("pacote".to_owned())));
        for outro in [
            "/w/target/debug/app",
            "/w/src/main.rs",
            "/w/stubs/x.pyi",
            "/w/a.pyc",
        ] {
            assert!(!e_um_alvo_python(&programa(outro)), "{outro}");
        }
    }
}

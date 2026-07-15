//! Handlers for `run.*` requests (`impl Core`).
//!
//! The `run.*` router plus start/stdin/stop, driving the user process managed
//! by `crate::run`.

use std::{ffi::OsStr, path::Path};

use kinein_protocol::{
    JsonRpcError, JsonRpcErrorCode, JsonRpcResponse, RunScriptParams, RunStartParams,
    RunStartResult, RunStdinParams,
};
use serde_json::{Value, json};

use crate::rpc::{
    fs_error_response, no_workspace_response, parse_params, run_error_response,
    run_unavailable_response,
};
use crate::{Core, run};

impl Core {
    /// Roteia os metodos `run.*`; `None` quando o metodo nao e de execucao.
    pub(crate) fn run_request_response(
        &mut self,
        method: &str,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> Option<JsonRpcResponse> {
        match method {
            "run.start" => Some(self.run_start_response(request_id, params)),
            "run.script" => Some(self.run_script_response(request_id, params)),
            "run.stdin" => Some(self.run_stdin_response(request_id, params)),
            "run.stop" => Some(self.run_stop_response(request_id)),
            _ => None,
        }
    }

    fn run_script_response(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(workspace) = self.workspace.clone() else {
            return no_workspace_response(request_id, "run.script");
        };
        let parsed = match parse_params::<RunScriptParams>(
            request_id.as_ref(),
            params,
            "run.script requer apenas o campo path",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let root = Path::new(&workspace.root);
        let script = match crate::fsops::confine_file(root, Path::new(&parsed.path)) {
            Ok(script) => script,
            Err(error) => return fs_error_response(request_id, &error),
        };
        let Some(interpreter) = run::script_interpreter(&script) else {
            return JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(
                    JsonRpcErrorCode::InvalidParams,
                    "run.script aceita scripts .sh, .bash ou .zsh",
                    Some(json!({ "path": parsed.path })),
                ),
            );
        };
        let Some(runner) = self.run.as_mut() else {
            return run_unavailable_response(request_id, "run.script");
        };
        let command = run::script_display_command(root, interpreter, &script);
        let args = [OsStr::new("--"), script.as_os_str()];
        match runner.start_program(root, interpreter, &args, &command) {
            Ok(()) => JsonRpcResponse::success(request_id, json!(RunStartResult { command })),
            Err(error) => run_error_response(request_id, &error),
        }
    }

    fn run_start_response(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(workspace) = self.workspace.clone() else {
            return no_workspace_response(request_id, "run.start");
        };
        let parsed = match parse_params::<RunStartParams>(
            request_id.as_ref(),
            params,
            "run.start aceita apenas o campo opcional command",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let Some(runner) = self.run.as_mut() else {
            return run_unavailable_response(request_id, "run.start");
        };

        let root = Path::new(&workspace.root);
        let command = match parsed.command.filter(|command| !command.trim().is_empty()) {
            Some(command) => command,
            None => match crate::runconfig::active_command(root) {
                Some(command) => command,
                None => match run::default_command(workspace.kind, root) {
                    Ok(command) => command,
                    Err(error) => return run_error_response(request_id, &error),
                },
            },
        };

        match runner.start(root, &command) {
            Ok(()) => JsonRpcResponse::success(request_id, json!(RunStartResult { command })),
            Err(error) => run_error_response(request_id, &error),
        }
    }

    fn run_stdin_response(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let parsed = match parse_params::<RunStdinParams>(
            request_id.as_ref(),
            params,
            "run.stdin requer o campo data",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let Some(runner) = self.run.as_mut() else {
            return run_unavailable_response(request_id, "run.stdin");
        };
        match runner.write_stdin(&parsed.data) {
            Ok(()) => JsonRpcResponse::success(request_id, json!({ "status": "ok" })),
            Err(error) => run_error_response(request_id, &error),
        }
    }

    fn run_stop_response(&mut self, request_id: Option<Value>) -> JsonRpcResponse {
        let Some(runner) = self.run.as_mut() else {
            return run_unavailable_response(request_id, "run.stop");
        };
        match runner.stop() {
            Ok(()) => JsonRpcResponse::success(request_id, json!({ "status": "ok" })),
            Err(error) => run_error_response(request_id, &error),
        }
    }
}

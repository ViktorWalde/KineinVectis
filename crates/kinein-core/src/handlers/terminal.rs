//! Handlers for `terminal.*` requests (`impl Core`).
//!
//! The `terminal.*` router plus open/input/close, driving the PTY session
//! managed by `crate::terminal`.

use kinein_protocol::{JsonRpcResponse, TerminalInputParams, TerminalOpenResult};
use serde_json::{Value, json};

use crate::Core;
use crate::rpc::{
    no_workspace_response, parse_params, terminal_error_response, terminal_unavailable_response,
};

impl Core {
    /// Roteia os metodos `terminal.*`; `None` quando o metodo nao e terminal.
    pub(crate) fn terminal_request_response(
        &mut self,
        method: &str,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> Option<JsonRpcResponse> {
        match method {
            "terminal.open" => Some(self.terminal_open_response(request_id)),
            "terminal.input" => Some(self.terminal_input_response(request_id, params)),
            "terminal.close" => Some(self.terminal_close_response(request_id)),
            _ => None,
        }
    }

    fn terminal_open_response(&mut self, request_id: Option<Value>) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "terminal.open");
        };
        let Some(session) = self.terminal.as_mut() else {
            return terminal_unavailable_response(request_id, "terminal.open");
        };
        match session.open(&root) {
            Ok(shell) => JsonRpcResponse::success(request_id, json!(TerminalOpenResult { shell })),
            Err(error) => terminal_error_response(request_id, &error),
        }
    }

    fn terminal_input_response(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let parsed = match parse_params::<TerminalInputParams>(
            request_id.as_ref(),
            params,
            "terminal.input requer o campo data",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let Some(session) = self.terminal.as_mut() else {
            return terminal_unavailable_response(request_id, "terminal.input");
        };
        match session.write(&parsed.data) {
            Ok(()) => JsonRpcResponse::success(request_id, json!({ "status": "ok" })),
            Err(error) => terminal_error_response(request_id, &error),
        }
    }

    fn terminal_close_response(&mut self, request_id: Option<Value>) -> JsonRpcResponse {
        let Some(session) = self.terminal.as_mut() else {
            return terminal_unavailable_response(request_id, "terminal.close");
        };
        match session.close() {
            Ok(()) => JsonRpcResponse::success(request_id, json!({ "status": "ok" })),
            Err(error) => terminal_error_response(request_id, &error),
        }
    }
}

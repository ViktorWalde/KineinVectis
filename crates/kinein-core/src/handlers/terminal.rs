//! Handlers for `terminal.*` requests (`impl Core`).
//!
//! The `terminal.*` router plus open/input/close, driving the PTY session
//! managed by `crate::terminal`.

use kinein_protocol::{
    JsonRpcResponse, TerminalCloseParams, TerminalInputParams, TerminalMouseParams,
    TerminalOpenResult, TerminalResizeParams, TerminalScrollParams,
};
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
            "terminal.resize" => Some(self.terminal_resize_response(request_id, params)),
            "terminal.scroll" => Some(self.terminal_scroll_response(request_id, params)),
            "terminal.mouse" => Some(self.terminal_mouse_response(request_id, params)),
            "terminal.close" => Some(self.terminal_close_response(request_id, params)),
            _ => None,
        }
    }

    /// `terminal.mouse` (R4): repassa o gesto cru ao core, que decide o destino
    /// pelo modo VT. O handler não interpreta o gesto — só o roteia.
    fn terminal_mouse_response(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let parsed = match parse_params::<TerminalMouseParams>(
            request_id.as_ref(),
            params,
            "terminal.mouse requer os campos id, col, row e event",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let Some(session) = self.terminal.as_mut() else {
            return terminal_unavailable_response(request_id, "terminal.mouse");
        };
        match session.mouse(
            &parsed.id,
            parsed.col,
            parsed.row,
            parsed.event,
            parsed.modifiers,
        ) {
            Ok(()) => JsonRpcResponse::success(request_id, json!({ "status": "ok" })),
            Err(error) => terminal_error_response(request_id, &error),
        }
    }

    fn terminal_scroll_response(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let parsed = match parse_params::<TerminalScrollParams>(
            request_id.as_ref(),
            params,
            "terminal.scroll requer os campos id e offset",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let Some(session) = self.terminal.as_mut() else {
            return terminal_unavailable_response(request_id, "terminal.scroll");
        };
        match session.scroll(&parsed.id, parsed.offset) {
            Ok(()) => JsonRpcResponse::success(request_id, json!({ "status": "ok" })),
            Err(error) => terminal_error_response(request_id, &error),
        }
    }

    fn terminal_resize_response(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let parsed = match parse_params::<TerminalResizeParams>(
            request_id.as_ref(),
            params,
            "terminal.resize requer os campos id, cols e rows",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let Some(session) = self.terminal.as_mut() else {
            return terminal_unavailable_response(request_id, "terminal.resize");
        };
        match session.resize(&parsed.id, parsed.cols, parsed.rows) {
            Ok(()) => JsonRpcResponse::success(request_id, json!({ "status": "ok" })),
            Err(error) => terminal_error_response(request_id, &error),
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
            Ok((id, shell)) => {
                JsonRpcResponse::success(request_id, json!(TerminalOpenResult { id, shell }))
            }
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
            "terminal.input requer os campos id e data",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let Some(session) = self.terminal.as_mut() else {
            return terminal_unavailable_response(request_id, "terminal.input");
        };
        match session.write(&parsed.id, &parsed.data) {
            Ok(()) => JsonRpcResponse::success(request_id, json!({ "status": "ok" })),
            Err(error) => terminal_error_response(request_id, &error),
        }
    }

    fn terminal_close_response(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let parsed = match parse_params::<TerminalCloseParams>(
            request_id.as_ref(),
            params,
            "terminal.close requer o campo id",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let Some(session) = self.terminal.as_mut() else {
            return terminal_unavailable_response(request_id, "terminal.close");
        };
        match session.close(&parsed.id) {
            Ok(()) => JsonRpcResponse::success(request_id, json!({ "status": "ok" })),
            Err(error) => terminal_error_response(request_id, &error),
        }
    }
}

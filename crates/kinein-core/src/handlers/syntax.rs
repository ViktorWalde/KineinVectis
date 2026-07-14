//! Handlers for local incremental syntax intelligence (`syntaxTree.*`).

use std::path::Path;

use kinein_protocol::{JsonRpcError, JsonRpcErrorCode, JsonRpcResponse, SyntaxTreeUpdateParams};
use serde_json::{Value, json};

use crate::{
    Core, fsops,
    lang::SyntaxTreeError,
    rpc::{fs_error_response, no_workspace_response, parse_params},
};

impl Core {
    /// Routes local syntax-tree requests.
    pub(crate) fn syntax_request_response(
        &mut self,
        method: &str,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> Option<JsonRpcResponse> {
        match method {
            "syntaxTree.update" => Some(self.syntax_update_response(request_id, params)),
            _ => None,
        }
    }

    fn syntax_update_response(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "syntaxTree.update");
        };
        let parsed = match parse_params::<SyntaxTreeUpdateParams>(
            request_id.as_ref(),
            params,
            "syntaxTree.update requer path, content e version",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let path = match fsops::confine_file(&root, Path::new(&parsed.path)) {
            Ok(path) => path,
            Err(error) => return fs_error_response(request_id, &error),
        };

        match self.syntax.update(&path, &parsed.content, parsed.version) {
            Ok(snapshot) => JsonRpcResponse::success(request_id, json!(snapshot)),
            Err(error) => syntax_error_response(request_id, &path, &error),
        }
    }
}

fn syntax_error_response(
    request_id: Option<Value>,
    path: &Path,
    error: &SyntaxTreeError,
) -> JsonRpcResponse {
    let code = if matches!(error, SyntaxTreeError::TooLarge { .. }) {
        JsonRpcErrorCode::InvalidParams
    } else {
        JsonRpcErrorCode::InternalError
    };
    JsonRpcResponse::failure(
        request_id,
        JsonRpcError::new(
            code,
            error.to_string(),
            Some(json!({ "path": path.display().to_string() })),
        ),
    )
}

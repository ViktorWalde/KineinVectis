//! `datasource.console` (`0.149.0`): garante o console de uma conexao e
//! devolve o caminho, para a UI abri-lo no editor.

use kinein_protocol::{
    DataSourceConsoleParams, DataSourceConsoleResult, JsonRpcError, JsonRpcErrorCode,
    JsonRpcResponse,
};
use serde_json::{Value, json};

use crate::Core;
use crate::rpc::{no_workspace_response, parse_params};

impl Core {
    pub(super) fn datasource_console_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "datasource.console");
        };
        let request = match parse_params::<DataSourceConsoleParams>(
            request_id.as_ref(),
            params,
            "datasource.console exige { name }",
        ) {
            Ok(request) => request,
            Err(response) => return *response,
        };
        match crate::datasource::console::ensure(&root, &request.name) {
            Ok((path, created)) => JsonRpcResponse::success(
                request_id,
                json!(DataSourceConsoleResult {
                    path: path.display().to_string(),
                    created
                }),
            ),
            Err(message) => JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(JsonRpcErrorCode::InvalidParams, message, None),
            ),
        }
    }
}

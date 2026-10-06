//! `datasource.console` (`0.149.0`): garante o console de uma conexao e
//! devolve o caminho, para a UI abri-lo no editor.

use kinein_protocol::{
    DataSourceConsoleParams, DataSourceConsoleResult, DataSourceConsoleStatementParams,
    DataSourceConsoleStatementResult, DataSourceOperationContext, JsonRpcError, JsonRpcErrorCode,
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
        let profile = match Self::find_profile(&root, &request.name) {
            Ok(profile) => profile,
            Err(response) => return super::datasource::com_id(*response, request_id),
        };
        if let Err(rejection) = crate::datasource::policy::check_context(
            &root,
            &profile,
            request.expected_context.as_ref(),
            request.client_context.as_deref(),
        ) {
            return rejection.response(
                request_id,
                &request.name,
                request.client_context.as_deref(),
            );
        }
        match crate::datasource::console::ensure(&root, &request.name) {
            Ok((path, created)) => JsonRpcResponse::success(
                request_id,
                json!(DataSourceConsoleResult {
                    path: path.display().to_string(),
                    created,
                    name: request.name,
                    client_context: request.client_context,
                    expected_context: DataSourceOperationContext {
                        workspace: root.display().to_string(),
                        profile
                    },
                }),
            ),
            Err(message) => JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(JsonRpcErrorCode::InvalidParams, message, None),
            ),
        }
    }
    pub(super) fn datasource_console_statement_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "datasource.console.statement");
        };
        let request = match parse_params::<DataSourceConsoleStatementParams>(
            request_id.as_ref(),
            params,
            "datasource.console.statement exige caminho, texto, posições e contexto",
        ) {
            Ok(request) => request,
            Err(response) => return *response,
        };
        let profile = match Self::find_profile(&root, &request.name) {
            Ok(profile) => profile,
            Err(response) => return super::datasource::com_id(*response, request_id),
        };
        if let Err(rejection) = crate::datasource::policy::check_context(
            &root,
            &profile,
            Some(&request.expected_context),
            Some(&request.client_context),
        ) {
            return rejection.response(request_id, &request.name, Some(&request.client_context));
        }
        let resolved = crate::datasource::console::inspect(
            &root,
            &request.name,
            std::path::Path::new(&request.path),
        )
        .and_then(|()| {
            crate::datasource::console_statement::extract(
                &request.text,
                request.cursor,
                request.selection_start,
                request.selection_end,
                profile.engine == kinein_protocol::DataSourceEngine::Mongo,
            )
            .map_err(str::to_owned)
        });
        match resolved {
            Ok(statement) => JsonRpcResponse::success(
                request_id,
                json!(DataSourceConsoleStatementResult {
                    name: request.name,
                    path: request.path,
                    statement,
                    client_context: request.client_context,
                    expected_context: DataSourceOperationContext {
                        workspace: root.display().to_string(),
                        profile
                    },
                }),
            ),
            Err(message) => JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(
                    JsonRpcErrorCode::InvalidParams,
                    message,
                    Some(json!({"name": request.name, "clientContext": request.client_context})),
                ),
            ),
        }
    }
}

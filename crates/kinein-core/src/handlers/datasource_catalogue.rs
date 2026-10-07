//! Persisted connection catalogue handlers; no driver or database effects.

use kinein_protocol::{
    DataSourceListParams, DataSourceListResult, DataSourceRemoveParams, DataSourceSaveParams,
    DataSourceWriteResult, JsonRpcError, JsonRpcErrorCode, JsonRpcResponse,
};
use serde_json::{Value, json};

use crate::Core;
use crate::rpc::{no_workspace_response, parse_params};

impl Core {
    /// `datasource.list` — os perfis salvos neste workspace, ordenados.
    pub(super) fn datasource_list_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "datasource.list");
        };
        if let Err(response) = parse_params::<DataSourceListParams>(
            request_id.as_ref(),
            params,
            "datasource.list nao aceita parametros",
        ) {
            return *response;
        }
        let profiles = match crate::datasource::try_list(&root) {
            Ok(profiles) => profiles,
            Err(message) => {
                return JsonRpcResponse::failure(
                    request_id,
                    JsonRpcError::new(JsonRpcErrorCode::InternalError, message, None),
                );
            }
        };
        let result = DataSourceListResult {
            providers: crate::datasource::providers::list(),
            console_bindings: crate::datasource::console::bindings(&root, &profiles),
            workspace: root.display().to_string(),
            profiles,
        };
        JsonRpcResponse::success(request_id, json!(result))
    }

    /// `datasource.save` — cria ou substitui o perfil de mesmo nome.
    pub(super) fn datasource_save_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "datasource.save");
        };
        let request = match parse_params::<DataSourceSaveParams>(
            request_id.as_ref(),
            params,
            "datasource.save exige { profile }",
        ) {
            Ok(request) => request,
            Err(response) => return *response,
        };
        match crate::datasource::save(&root, &request.profile) {
            Ok(profiles) => {
                self.previews.revoke(&root, &request.profile.name);
                JsonRpcResponse::success(
                    request_id,
                    json!(DataSourceWriteResult {
                        providers: crate::datasource::providers::list(),
                        console_bindings: crate::datasource::console::bindings(&root, &profiles),
                        workspace: root.display().to_string(),
                        profiles
                    }),
                )
            }
            Err(message) => JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(JsonRpcErrorCode::InvalidParams, message, None),
            ),
        }
    }

    /// `datasource.remove` — tira o perfil do catalogo.
    ///
    /// Remover o que nao existe devolve sucesso com o catalogo atual: a UI nao
    /// precisa tratar "ja tinha sumido" como erro.
    pub(super) fn datasource_remove_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "datasource.remove");
        };
        let request = match parse_params::<DataSourceRemoveParams>(
            request_id.as_ref(),
            params,
            "datasource.remove exige { name }",
        ) {
            Ok(request) => request,
            Err(response) => return *response,
        };
        self.odbc.revoke(&root, &request.name);
        self.previews.revoke(&root, &request.name);
        match crate::datasource::remove(&root, &request.name) {
            Ok(profiles) => JsonRpcResponse::success(
                request_id,
                json!(DataSourceWriteResult {
                    providers: crate::datasource::providers::list(),
                    console_bindings: crate::datasource::console::bindings(&root, &profiles),
                    workspace: root.display().to_string(),
                    profiles
                }),
            ),
            Err(message) => JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(JsonRpcErrorCode::InternalError, message, None),
            ),
        }
    }
}

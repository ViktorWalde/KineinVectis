//! Handlers for `runConfig.*` requests (`impl Core`): CRUD sincrono das run
//! configurations do workspace, todas respondendo a lista completa + ativa.

use kinein_protocol::{
    JsonRpcError, JsonRpcErrorCode, JsonRpcResponse, RunConfigDeleteParams, RunConfigSaveParams,
    RunConfigSetActiveParams, RunConfigsResult,
};
use serde_json::{Value, json};

use crate::rpc::{no_workspace_response, parse_params};
use crate::{Core, runconfig};

impl Core {
    /// Roteia os metodos `runConfig.*`; `None` quando o metodo nao e de
    /// run configurations.
    pub(crate) fn runconfig_request_response(
        &self,
        method: &str,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> Option<JsonRpcResponse> {
        match method {
            "runConfig.list" => Some(self.runconfig_list_response(request_id)),
            "runConfig.save" => Some(self.runconfig_save_response(request_id, params)),
            "runConfig.delete" => Some(self.runconfig_delete_response(request_id, params)),
            "runConfig.setActive" => Some(self.runconfig_set_active_response(request_id, params)),
            _ => None,
        }
    }

    fn runconfig_list_response(&self, request_id: Option<Value>) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "runConfig.list");
        };
        let state = runconfig::load(&root);
        JsonRpcResponse::success(
            request_id,
            json!(RunConfigsResult {
                configs: state.configs,
                active_id: state.active_id,
            }),
        )
    }

    fn runconfig_save_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "runConfig.save");
        };
        let parsed = match parse_params::<RunConfigSaveParams>(
            request_id.as_ref(),
            params,
            "runConfig.save requer name e command (id opcional para editar)",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        match runconfig::save(&root, parsed.id.as_deref(), &parsed.name, &parsed.command) {
            Ok(state) => JsonRpcResponse::success(
                request_id,
                json!(RunConfigsResult {
                    configs: state.configs,
                    active_id: state.active_id,
                }),
            ),
            Err(message) => invalid_runconfig_response(request_id, message),
        }
    }

    fn runconfig_delete_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "runConfig.delete");
        };
        let parsed = match parse_params::<RunConfigDeleteParams>(
            request_id.as_ref(),
            params,
            "runConfig.delete requer o campo id",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        match runconfig::delete(&root, &parsed.id) {
            Ok(state) => JsonRpcResponse::success(
                request_id,
                json!(RunConfigsResult {
                    configs: state.configs,
                    active_id: state.active_id,
                }),
            ),
            Err(message) => invalid_runconfig_response(request_id, message),
        }
    }

    fn runconfig_set_active_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "runConfig.setActive");
        };
        let parsed = match parse_params::<RunConfigSetActiveParams>(
            request_id.as_ref(),
            params,
            "runConfig.setActive aceita o campo opcional id",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        match runconfig::set_active(&root, parsed.id.as_deref()) {
            Ok(state) => JsonRpcResponse::success(
                request_id,
                json!(RunConfigsResult {
                    configs: state.configs,
                    active_id: state.active_id,
                }),
            ),
            Err(message) => invalid_runconfig_response(request_id, message),
        }
    }
}

/// Erros de dominio das run configs viram `INVALID_PARAMS` com a mensagem.
fn invalid_runconfig_response(request_id: Option<Value>, message: String) -> JsonRpcResponse {
    JsonRpcResponse::failure(
        request_id,
        JsonRpcError::new(JsonRpcErrorCode::InvalidParams, message, None),
    )
}

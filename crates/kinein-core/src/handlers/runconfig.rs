//! Handlers for `runConfig.*` requests (`impl Core`): CRUD sincrono das run
//! configurations do workspace, todas respondendo a lista completa + ativa.

use kinein_protocol::{
    FlashProposalParams, JsonRpcError, JsonRpcErrorCode, JsonRpcResponse, RunConfigDeleteParams,
    RunConfigSaveParams, RunConfigSetActiveParams, RunConfigsResult,
};
use serde_json::{Value, json};

use crate::rpc::{no_workspace_response, parse_params};
use crate::{Core, flash, runconfig};

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
            "runConfig.flashProposal" => {
                Some(self.runconfig_flash_proposal_response(request_id, params))
            }
            _ => None,
        }
    }

    /// `runConfig.flashProposal` — "Gravar" como configuracao de execucao (E4
    /// do `integracoes/38` §6, `0.113.0`). PURO: compoe a linha do motor a
    /// partir do modelo do projeto e da porta escolhida; nada roda, nada e'
    /// salvo — salvar e' `runConfig.save`, rodar e' `run.start`.
    fn runconfig_flash_proposal_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let pedido = match parse_params::<FlashProposalParams>(
            request_id.as_ref(),
            params,
            "runConfig.flashProposal aceita os campos opcionais device, engine e flashSizeBytes",
        ) {
            Ok(pedido) => pedido,
            Err(response) => return *response,
        };
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "runConfig.flashProposal");
        };
        let modelo = self.compute_project_model(&root);
        let acha = |binario: &str| self.detector.find_in_path(binario);
        match flash::propose(
            &modelo,
            pedido.engine.as_deref(),
            pedido.device.as_deref(),
            pedido.flash_size_bytes,
            &acha,
        ) {
            Ok(proposta) => JsonRpcResponse::success(request_id, json!(proposta)),
            Err(error) => {
                // Ferramenta ausente e' TOOL_NOT_FOUND (a UI mostra o passo);
                // motor desconhecido e' parametro invalido; o resto e' o
                // estado do projeto (sem build, sem porta, sem motor).
                let code = match &error {
                    flash::FlashError::MissingTool { .. } => JsonRpcErrorCode::ToolNotFound,
                    flash::FlashError::UnknownEngine { .. } => JsonRpcErrorCode::InvalidParams,
                    _ => JsonRpcErrorCode::InvalidRequest,
                };
                JsonRpcResponse::failure(
                    request_id,
                    JsonRpcError::new(
                        code,
                        error.to_string(),
                        Some(json!({ "method": "runConfig.flashProposal" })),
                    ),
                )
            }
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

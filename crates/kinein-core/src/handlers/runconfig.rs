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
    /// do `integracoes/38` §6, `0.113.0`). Sem acesso ao dispositivo ou escrita:
    /// consulta apenas a versao do binario antes do planejamento puro.
    fn runconfig_flash_proposal_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let request = match parse_params::<FlashProposalParams>(
            request_id.as_ref(),
            params,
            "runConfig.flashProposal aceita device, engine, flashSizeBytes, firmware, native e buildDirectory opcionais",
        ) {
            Ok(request) => request,
            Err(response) => return *response,
        };
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "runConfig.flashProposal");
        };
        // Um firmware do catalogo (C5) substitui os artefatos do build; o
        // que nao foi baixado e' recusado antes de compor a linha.
        let firmware = match request
            .firmware
            .as_deref()
            .map(|id| self.firmware_image(id))
        {
            None => None,
            Some(Ok(imagem)) => Some(imagem),
            Some(Err(mensagem)) => {
                return JsonRpcResponse::failure(
                    request_id,
                    JsonRpcError::new(
                        JsonRpcErrorCode::InvalidRequest,
                        format!("runConfig.flashProposal: {mensagem}"),
                        Some(json!({ "method": "runConfig.flashProposal" })),
                    ),
                );
            }
        };
        let model = self.compute_project_model(&root);
        if request.native {
            return self.native_flash_proposal_response(request_id, root, model, request, firmware);
        }
        if request.build_directory.is_some() {
            return JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(
                    JsonRpcErrorCode::InvalidParams,
                    "buildDirectory requer native=true",
                    None,
                ),
            );
        }
        // O wrapper do framework (bloco E), quando esta' nesta maquina.
        let framework = crate::build::engine::resolve(&model, self.sdk_home().as_deref())
            .ok()
            .flatten();
        let detector = self.detector.clone();
        self.defer_work(request_id, move |request_id| {
            let find = |binary: &str| detector.find_in_path(binary);
            let engine =
                match flash::select_engine(&model, request.engine.as_deref(), firmware.as_ref()) {
                    Ok((engine, _)) => engine,
                    Err(error) => return flash_error_response(request_id, &error),
                };
            let mut selected = None;
            if engine == "esptool"
                && let Some(path) = find("esptool").or_else(|| find("esptool.py"))
            {
                match flash::esptool::probe(path, &root, std::time::Duration::from_secs(5)) {
                    Ok(tool) => selected = Some(tool),
                    Err(error) => return flash_error_response(request_id, &error),
                }
            }
            let tools = flash::ProposalTools {
                find: &find,
                esptool: selected,
            };
            match flash::propose(
                &model,
                request.engine.as_deref(),
                request.device.as_deref(),
                request.flash_size_bytes,
                firmware.as_ref(),
                framework.as_ref(),
                &tools,
            ) {
                Ok(proposta) => JsonRpcResponse::success(request_id, json!(proposta)),
                Err(error) => flash_error_response(request_id, &error),
            }
        })
    }

    fn runconfig_list_response(&self, request_id: Option<Value>) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "runConfig.list");
        };
        let state = match runconfig::load(&root) {
            Ok(state) => state,
            Err(error) => return runconfig_error_response(request_id, &error),
        };
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
        match runconfig::save_with_flash(
            &root,
            parsed.id.as_deref(),
            &parsed.name,
            &parsed.command,
            parsed.flash.as_ref(),
        ) {
            Ok(state) => JsonRpcResponse::success(
                request_id,
                json!(RunConfigsResult {
                    configs: state.configs,
                    active_id: state.active_id,
                }),
            ),
            Err(message) => runconfig_error_response(request_id, &message),
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
            Err(message) => runconfig_error_response(request_id, &message),
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
            Err(message) => runconfig_error_response(request_id, &message),
        }
    }
}

/// Version/precondition refusals retain the proposal's existing error mapping.
pub(super) fn flash_error_response(
    request_id: Option<Value>,
    error: &flash::FlashError,
) -> JsonRpcResponse {
    flash_error_response_for("runConfig.flashProposal", request_id, error)
}

pub(super) fn flash_error_response_for(
    method: &str,
    request_id: Option<Value>,
    error: &flash::FlashError,
) -> JsonRpcResponse {
    let code = match error {
        flash::FlashError::MissingTool { .. } => JsonRpcErrorCode::ToolNotFound,
        flash::FlashError::UnknownEngine { .. } => JsonRpcErrorCode::InvalidParams,
        _ => JsonRpcErrorCode::InvalidRequest,
    };
    JsonRpcResponse::failure(
        request_id,
        JsonRpcError::new(code, error.to_string(), Some(json!({ "method": method }))),
    )
}

/// Invalid input and unreadable persisted intent keep distinct error codes.
pub(super) fn runconfig_error_response(
    request_id: Option<Value>,
    error: &runconfig::RunConfigError,
) -> JsonRpcResponse {
    let code = match error {
        runconfig::RunConfigError::Invalid(_) => JsonRpcErrorCode::InvalidParams,
        runconfig::RunConfigError::Storage(_) => JsonRpcErrorCode::InvalidRequest,
    };
    JsonRpcResponse::failure(request_id, JsonRpcError::new(code, error.to_string(), None))
}

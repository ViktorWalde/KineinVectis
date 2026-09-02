//! Handlers for `configAction.*` requests (`impl Core`): lista contextual,
//! preview do plano e aplicacao com o snapshot do preview.
//!
//! Fino de proposito (`ARCHITECTURE.md` §4 regra 2): parse dos params, escopo
//! do workspace, delega ao [`crate::configaction`] e formata a resposta. A
//! unica decisao que mora aqui e a que o dominio nao pode tomar — disparar o
//! job do `cargo.check`, porque o `JobManager` e do `Core` (arquitetura/04 §3).

use kinein_protocol::{
    ConfigActionApplyParams, ConfigActionApplyResult, ConfigActionListParams,
    ConfigActionPreviewParams, JsonRpcError, JsonRpcErrorCode, JsonRpcResponse,
};
use serde_json::{Value, json};

use crate::Core;
use crate::configaction::{self, ConfigActionError, ConfigActionOutcome};
use crate::rpc::{no_workspace_response, parse_params};

impl Core {
    /// Roteia os metodos `configAction.*`; `None` quando o metodo nao e deles.
    pub(crate) fn configaction_request_response(
        &self,
        method: &str,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> Option<JsonRpcResponse> {
        match method {
            "configAction.list" => Some(self.configaction_list_response(request_id, params)),
            "configAction.preview" => Some(self.configaction_preview_response(request_id, params)),
            "configAction.apply" => Some(self.configaction_apply_response(request_id, params)),
            _ => None,
        }
    }

    fn configaction_list_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(workspace) = self.workspace.as_ref() else {
            return no_workspace_response(request_id, "configAction.list");
        };
        let parsed = match parse_params::<ConfigActionListParams>(
            request_id.as_ref(),
            params,
            "configAction.list aceita o campo opcional includeHiddenByScope",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let root = std::path::PathBuf::from(&workspace.root);
        let result = configaction::list(
            &root,
            &workspace.capabilities.build_systems,
            parsed.include_hidden_by_scope,
        );
        JsonRpcResponse::success(request_id, json!(result))
    }

    fn configaction_preview_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(workspace) = self.workspace.as_ref() else {
            return no_workspace_response(request_id, "configAction.preview");
        };
        let parsed = match parse_params::<ConfigActionPreviewParams>(
            request_id.as_ref(),
            params,
            "configAction.preview requer o campo id (params e opcional)",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let root = std::path::PathBuf::from(&workspace.root);
        match configaction::preview(&root, &workspace.capabilities.build_systems, &parsed) {
            Ok(result) => JsonRpcResponse::success(request_id, json!(result)),
            Err(error) => configaction_error_response(request_id, &error),
        }
    }

    fn configaction_apply_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(workspace) = self.workspace.as_ref() else {
            return no_workspace_response(request_id, "configAction.apply");
        };
        let parsed = match parse_params::<ConfigActionApplyParams>(
            request_id.as_ref(),
            params,
            "configAction.apply requer o campo id (params e expected sao opcionais)",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let root = std::path::PathBuf::from(&workspace.root);
        let outcome =
            match configaction::apply(&root, &workspace.capabilities.build_systems, &parsed) {
                Ok(outcome) => outcome,
                Err(error) => return configaction_error_response(request_id, &error),
            };

        match outcome {
            ConfigActionOutcome::Done { message, files } => JsonRpcResponse::success(
                request_id,
                json!(ConfigActionApplyResult {
                    id: parsed.id,
                    message,
                    files,
                    job_id: None,
                }),
            ),
            ConfigActionOutcome::Job { method } => {
                self.configaction_job_response(request_id, &parsed.id, method)
            }
        }
    }

    /// Acoes de efeito `job` reusam o metodo que ja existe, sem executor novo.
    fn configaction_job_response(
        &self,
        request_id: Option<Value>,
        id: &str,
        method: &str,
    ) -> JsonRpcResponse {
        let delegated = self.cargo_request_response(method, request_id.clone(), None);
        let Some(response) = delegated else {
            return JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(
                    JsonRpcErrorCode::InternalError,
                    format!("a acao {id} aponta para o metodo desconhecido {method}"),
                    None,
                ),
            );
        };
        if response.error.is_some() {
            return response;
        }
        let job_id = response
            .result
            .as_ref()
            .and_then(|result| result.get("jobId"))
            .and_then(Value::as_str)
            .map(str::to_owned);
        JsonRpcResponse::success(
            request_id,
            json!(ConfigActionApplyResult {
                id: id.to_owned(),
                message: format!("{method} disparado como job"),
                files: Vec::new(),
                job_id,
            }),
        )
    }
}

/// Erro de dominio vira erro estruturado do protocolo.
fn configaction_error_response(
    request_id: Option<Value>,
    error: &ConfigActionError,
) -> JsonRpcResponse {
    let code = if error.changed_path().is_some() {
        JsonRpcErrorCode::FileChanged
    } else if error.is_invalid_params() {
        JsonRpcErrorCode::InvalidParams
    } else {
        JsonRpcErrorCode::InternalError
    };
    let details = error.changed_path().map(|path| json!({ "path": path }));
    JsonRpcResponse::failure(
        request_id,
        JsonRpcError::new(code, error.to_string(), details),
    )
}

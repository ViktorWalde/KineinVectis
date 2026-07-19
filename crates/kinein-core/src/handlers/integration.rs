//! Handler de `integration.*` (`impl Core`): fino — roteia e delega ao domínio
//! `crate::integration`. Roadmap 28 §2 ("handler fino: parse + delega").

use kinein_protocol::{
    IntegrationChangeKind, IntegrationConfigGetParams, IntegrationConfigResetParams,
    IntegrationConfigResetResult, IntegrationConfigSetParams, JsonRpcError, JsonRpcErrorCode,
    JsonRpcResponse,
};
use serde_json::{Value, json};

use crate::rpc::parse_params;
use crate::{Core, integration};

impl Core {
    /// Roteia `integration.*`; `None` quando o método não é de integração.
    ///
    /// `integration.list` é o inventário read-only (v1 de leitura, 2026-07-17);
    /// `integration.config.get/set/reset` são a metade de ESCRITA do contrato
    /// (fatia 2.2, protocolo 0.62.0). A detecção segue reusada do `tools.rs`.
    pub(crate) fn integration_request_response(
        &mut self,
        method: &str,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> Option<JsonRpcResponse> {
        match method {
            "integration.list" => Some(JsonRpcResponse::success(
                request_id,
                json!(integration::list(&self.detector)),
            )),
            "integration.config.get" => Some(self.integration_config_get(request_id, params)),
            "integration.config.set" => Some(self.integration_config_set(request_id, params)),
            "integration.config.reset" => Some(self.integration_config_reset(request_id, params)),
            _ => None,
        }
    }

    fn integration_config_get(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        match parse_params::<IntegrationConfigGetParams>(
            request_id.as_ref(),
            params,
            "integration.config.get requer o campo id",
        ) {
            Ok(parsed) => {
                if let Some(response) = unknown_integration(request_id.as_ref(), &parsed.id) {
                    return response;
                }
                self.ensure_global_config();
                let stores = integration::config::ConfigStores {
                    global: self.global_config.as_ref(),
                    workspace: self.drafts.as_ref(),
                };
                match integration::config::get(&stores, &parsed.id) {
                    Ok(result) => JsonRpcResponse::success(request_id, json!(result)),
                    Err(error) => config_error_response(request_id, &error),
                }
            }
            Err(response) => *response,
        }
    }

    fn integration_config_set(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        match parse_params::<IntegrationConfigSetParams>(
            request_id.as_ref(),
            params,
            "integration.config.set requer id, key, value e scope",
        ) {
            Ok(parsed) => {
                if let Some(response) = unknown_integration(request_id.as_ref(), &parsed.id) {
                    return response;
                }
                self.ensure_global_config();
                let stores = integration::config::ConfigStores {
                    global: self.global_config.as_ref(),
                    workspace: self.drafts.as_ref(),
                };
                match integration::config::set(
                    &stores,
                    &parsed.id,
                    &parsed.key,
                    &parsed.value,
                    parsed.scope,
                ) {
                    Ok(changed) => {
                        // Set idempotente NAO emite: evento e' mudanca real.
                        if changed {
                            self.emit_integration_changed(
                                &parsed.id,
                                IntegrationChangeKind::Config,
                            );
                        }
                        JsonRpcResponse::success(
                            request_id,
                            json!(kinein_protocol::IntegrationConfigEntry {
                                key: parsed.key,
                                value: parsed.value,
                                scope: parsed.scope,
                            }),
                        )
                    }
                    Err(error) => config_error_response(request_id, &error),
                }
            }
            Err(response) => *response,
        }
    }

    fn integration_config_reset(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        match parse_params::<IntegrationConfigResetParams>(
            request_id.as_ref(),
            params,
            "integration.config.reset requer id, key e scope",
        ) {
            Ok(parsed) => {
                if let Some(response) = unknown_integration(request_id.as_ref(), &parsed.id) {
                    return response;
                }
                self.ensure_global_config();
                let stores = integration::config::ConfigStores {
                    global: self.global_config.as_ref(),
                    workspace: self.drafts.as_ref(),
                };
                match integration::config::reset(&stores, &parsed.id, &parsed.key, parsed.scope) {
                    Ok(removed) => {
                        if removed {
                            self.emit_integration_changed(
                                &parsed.id,
                                IntegrationChangeKind::Config,
                            );
                        }
                        JsonRpcResponse::success(
                            request_id,
                            json!(IntegrationConfigResetResult {
                                id: parsed.id,
                                key: parsed.key,
                                removed,
                            }),
                        )
                    }
                    Err(error) => config_error_response(request_id, &error),
                }
            }
            Err(response) => *response,
        }
    }
}

/// Config de integração desconhecida é erro de parâmetro, não sucesso vazio:
/// a UI descobriria o typo só quando o valor "sumisse".
fn unknown_integration(request_id: Option<&Value>, id: &str) -> Option<JsonRpcResponse> {
    if integration::is_known(id) {
        return None;
    }
    Some(JsonRpcResponse::failure(
        request_id.cloned(),
        JsonRpcError::new(
            JsonRpcErrorCode::InvalidParams,
            "integracao desconhecida",
            Some(json!({ "id": id })),
        ),
    ))
}

fn config_error_response(
    request_id: Option<Value>,
    error: &integration::config::ConfigError,
) -> JsonRpcResponse {
    JsonRpcResponse::failure(
        request_id,
        JsonRpcError::new(JsonRpcErrorCode::InvalidRequest, error.to_string(), None),
    )
}

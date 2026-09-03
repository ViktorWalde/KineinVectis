//! Handlers for `toolchain.*` requests (`impl Core`): ler e fixar qual
//! executavel cumpre cada papel no workspace aberto.
//!
//! Fino, como a §4 regra 2 manda: valida params, chama o dominio e formata a
//! resposta. Quem cruza escolha com maquina e o [`crate::toolchain`]; quem
//! detecta continua sendo o `ToolDetector`.

use kinein_protocol::{
    JsonRpcError, JsonRpcErrorCode, JsonRpcResponse, ToolchainGetParams, ToolchainSetParams,
};
use serde_json::{Value, json};

use crate::rpc::{no_workspace_response, parse_params};
use crate::{Core, toolchain};

impl Core {
    /// Roteia os metodos `toolchain.*`; `None` quando o metodo nao e deles.
    pub(crate) fn toolchain_request_response(
        &self,
        method: &str,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> Option<JsonRpcResponse> {
        match method {
            "toolchain.get" => Some(self.toolchain_get_response(request_id, params)),
            "toolchain.set" => Some(self.toolchain_set_response(request_id, params)),
            _ => None,
        }
    }

    fn toolchain_get_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "toolchain.get");
        };
        if let Err(response) = parse_params::<ToolchainGetParams>(
            request_id.as_ref(),
            params,
            "toolchain.get nao aceita parametros",
        ) {
            return *response;
        }
        let resolvida = toolchain::Toolchain::resolve(&root, &self.detected_tools());
        JsonRpcResponse::success(request_id, json!(resolvida.to_result()))
    }

    fn toolchain_set_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "toolchain.set");
        };
        let parsed = match parse_params::<ToolchainSetParams>(
            request_id.as_ref(),
            params,
            "toolchain.set requer role (id ausente volta para automatico)",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        match toolchain::set(
            &root,
            &self.detected_tools(),
            parsed.role,
            parsed.id.as_deref(),
        ) {
            Ok(resolvida) => JsonRpcResponse::success(request_id, json!(resolvida.to_result())),
            Err(message) => JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(JsonRpcErrorCode::InvalidParams, message, None),
            ),
        }
    }
}

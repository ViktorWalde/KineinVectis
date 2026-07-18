//! Handler de `integration.*` (`impl Core`): fino — roteia e delega ao domínio
//! `crate::integration`. Roadmap 28 §2 ("handler fino: parse + delega").

use kinein_protocol::JsonRpcResponse;
use serde_json::{Value, json};

use crate::{Core, integration};

impl Core {
    /// Roteia `integration.*`; `None` quando o método não é de integração.
    ///
    /// v1 tem só `integration.list` (inventário read-only), sem parâmetros. A
    /// detecção é reusada do `tools.rs` pelo domínio — este handler não a toca.
    pub(crate) fn integration_request_response(
        &self,
        method: &str,
        request_id: Option<Value>,
        _params: Option<&Value>,
    ) -> Option<JsonRpcResponse> {
        match method {
            "integration.list" => Some(JsonRpcResponse::success(
                request_id,
                json!(integration::list(&self.detector)),
            )),
            _ => None,
        }
    }
}

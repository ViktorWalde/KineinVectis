//! `lsp.restart` — ciclo de vida do servidor. Par de `crate::lsp::session`.

use kinein_protocol::{JsonRpcResponse, LspRestartParams, LspRestartResult};
use serde_json::{Value, json};

use crate::Core;
use crate::rpc::{lsp_unavailable_response, parse_params};

impl Core {
    /// Reinicia servidor(es) LSP (M4.3b): `{ language? }` — sem language,
    /// reinicia todos os vivos. Cada um sobe de novo (lazy) no próximo
    /// request; a UI re-sincroniza o arquivo ativo ao ver `event.lsp.restarted`.
    pub(super) fn lsp_restart_response(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        // `params` ausente = reiniciar todos (language None).
        let language = match params {
            Some(value) => match parse_params::<LspRestartParams>(
                request_id.as_ref(),
                Some(value),
                "lsp.restart aceita o campo opcional language",
            ) {
                Ok(parsed) => parsed.language,
                Err(response) => return *response,
            },
            None => None,
        };
        let Some(lsp) = self.lsp.as_mut() else {
            return lsp_unavailable_response(request_id, "lsp.restart");
        };
        let restarted = match language {
            Some(language) => {
                if lsp.restart_language(&language) {
                    vec![language]
                } else {
                    Vec::new()
                }
            }
            None => lsp.restart_all(),
        };
        JsonRpcResponse::success(request_id, json!(LspRestartResult { restarted }))
    }
}

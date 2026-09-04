//! Handlers for `library.*` requests (`impl Core`).
//!
//! Fino, como a `ARCHITECTURE.md` §4 manda: roteia, valida params, delega ao
//! dominio `crate::library` e formata a resposta.

use kinein_protocol::{
    JsonRpcError, JsonRpcErrorCode, JsonRpcResponse, LibraryListParams, LibraryListResult,
    LibraryPlanParams,
};
use serde_json::{Value, json};

use crate::Core;
use crate::library;
use crate::rpc::parse_params;

impl Core {
    /// Roteia os metodos `library.*`; `None` quando o metodo nao e deste dominio.
    ///
    /// Recebe `self` desde 2026-09-04 por UM motivo: alem do catalogo (estatico)
    /// e da disponibilidade (filesystem), a resposta diz o que o PROJETO ABERTO
    /// ja' linka — e isso depende de qual workspace esta aberto.
    pub(crate) fn library_request_response(
        &self,
        method: &str,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> Option<JsonRpcResponse> {
        match method {
            "library.list" => Some(self.library_list_response(request_id, params)),
            "library.plan" => Some(Self::library_plan_response(request_id, params)),
            _ => None,
        }
    }

    /// `library.list` — o catalogo cruzado com esta maquina E com este projeto.
    ///
    /// Nao EXIGE workspace aberto: sem projeto o catalogo ainda vale, e todas
    /// as bibliotecas voltam com `applied: false`, que e' a verdade.
    fn library_list_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        if let Err(response) = parse_params::<LibraryListParams>(
            request_id.as_ref(),
            params,
            "library.list nao aceita parametros",
        ) {
            return *response;
        }
        JsonRpcResponse::success(
            request_id,
            json!(LibraryListResult {
                libraries: library::list(self.workspace_root().as_deref()),
            }),
        )
    }

    /// `library.plan` — o que seria preciso para um alvo usar a biblioteca.
    fn library_plan_response(request_id: Option<Value>, params: Option<&Value>) -> JsonRpcResponse {
        let parsed = match parse_params::<LibraryPlanParams>(
            request_id.as_ref(),
            params,
            "library.plan requer id e target",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        match library::plan(&parsed.id, &parsed.target) {
            Ok(plan) => JsonRpcResponse::success(request_id, json!(plan)),
            Err(message) => JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(JsonRpcErrorCode::InvalidParams, message, None),
            ),
        }
    }
}

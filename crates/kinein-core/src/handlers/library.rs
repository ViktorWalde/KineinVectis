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
    /// Nao recebe `self`, e a ausencia diz algo: este dominio e' STATELESS. O
    /// catalogo e' estatico e a disponibilidade se le do filesystem — nada aqui
    /// depende do estado do `Core`, nem de workspace aberto.
    pub(crate) fn library_request_response(
        method: &str,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> Option<JsonRpcResponse> {
        match method {
            "library.list" => Some(Self::library_list_response(request_id, params)),
            "library.plan" => Some(Self::library_plan_response(request_id, params)),
            _ => None,
        }
    }

    /// `library.list` — o catalogo cruzado com esta maquina.
    ///
    /// Nao exige workspace aberto: o catalogo e o que existe no mundo mais o
    /// que existe na maquina, e nenhum dos dois depende de projeto aberto.
    fn library_list_response(request_id: Option<Value>, params: Option<&Value>) -> JsonRpcResponse {
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
                libraries: library::list(),
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

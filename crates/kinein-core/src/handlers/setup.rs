//! Handler dos pedidos `setup.*` (`impl Core`).
//!
//! Fino: roteia, valida params, delega ao `crate::setup` e formata a resposta.
//!
//! NAO EXIGE WORKSPACE. Instalar `PostgreSQL` nao depende de projeto aberto — e
//! quem esta comecando costuma abrir a IDE antes de ter projeto nenhum, que e'
//! exatamente o momento em que este guia serve.

use kinein_protocol::{JsonRpcResponse, SetupListParams, SetupListResult};
use serde_json::{Value, json};

use crate::Core;
use crate::rpc::parse_params;

impl Core {
    /// Roteia os metodos `setup.*`.
    pub(crate) fn setup_request_response(
        &self,
        method: &str,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> Option<JsonRpcResponse> {
        match method {
            "setup.list" => Some(self.setup_list_response(request_id, params)),
            _ => None,
        }
    }

    /// `setup.list` — o passo a passo oficial para a distro desta maquina.
    fn setup_list_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        if let Err(response) = parse_params::<SetupListParams>(
            request_id.as_ref(),
            params,
            "setup.list nao aceita parametros",
        ) {
            return *response;
        }
        let (distro_id, distro_name, family) = crate::setup::current_distro();
        JsonRpcResponse::success(
            request_id,
            json!(SetupListResult {
                distro_id,
                distro_name,
                family,
                tools: crate::setup::guides(&self.detector),
            }),
        )
    }
}

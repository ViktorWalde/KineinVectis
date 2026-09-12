//! Handler for `serial.*` requests (`impl Core`).
//!
//! Fino: roteia, valida params, delega ao `crate::serial` e formata a resposta.

use kinein_protocol::{JsonRpcResponse, SerialListParams};
use serde_json::{Value, json};

use crate::Core;
use crate::rpc::parse_params;

impl Core {
    /// Roteia os metodos `serial.*`. Funcao associada, sem `self`: o dominio nao
    /// tem estado no `Core` — le o sysfs desta maquina e nada mais.
    pub(crate) fn serial_request_response(
        method: &str,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> Option<JsonRpcResponse> {
        match method {
            "serial.list" => Some(Self::serial_list_response(request_id, params)),
            _ => None,
        }
    }

    /// `serial.list` — as portas seriais USB desta maquina.
    ///
    /// NAO exige workspace, ao contrario do `probe.list`: nao ha' ferramenta
    /// vinda do kit aqui — e' o sysfs desta maquina, o mesmo com ou sem
    /// projeto aberto. Exigir projeto para dizer o que esta' no USB seria
    /// esconder informacao que a IDE ja' tem.
    fn serial_list_response(request_id: Option<Value>, params: Option<&Value>) -> JsonRpcResponse {
        if let Err(response) = parse_params::<SerialListParams>(
            request_id.as_ref(),
            params,
            "serial.list nao aceita parametros",
        ) {
            return *response;
        }
        JsonRpcResponse::success(request_id, json!(crate::serial::list()))
    }
}

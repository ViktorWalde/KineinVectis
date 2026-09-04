//! Handler for `probe.*` requests (`impl Core`).
//!
//! Fino: roteia, valida params, delega ao `crate::probe` e formata a resposta.

use kinein_protocol::{JsonRpcResponse, ProbeListParams};
use serde_json::{Value, json};

use crate::Core;
use crate::rpc::{no_workspace_response, parse_params};

impl Core {
    /// Roteia os metodos `probe.*`.
    pub(crate) fn probe_request_response(
        &self,
        method: &str,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> Option<JsonRpcResponse> {
        match method {
            "probe.list" => Some(self.probe_list_response(request_id, params)),
            _ => None,
        }
    }

    /// `probe.list` — as sondas conectadas nesta maquina.
    ///
    /// Exige workspace porque o EXECUTAVEL vem do kit: sem projeto aberto nao
    /// ha kit, e cair no `PATH` em silencio esconderia do usuario qual
    /// ferramenta respondeu.
    fn probe_list_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "probe.list");
        };
        if let Err(response) = parse_params::<ProbeListParams>(
            request_id.as_ref(),
            params,
            "probe.list nao aceita parametros",
        ) {
            return *response;
        }
        let toolchain = crate::toolchain::Toolchain::resolve(&root, &self.detected_tools());
        let programa = toolchain.program_for(kinein_protocol::ToolchainRole::DebugAdapter);
        JsonRpcResponse::success(request_id, json!(crate::probe::list(programa.as_deref())))
    }
}

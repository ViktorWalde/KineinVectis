//! Handler for `project.*` requests (`impl Core`) — o MODELO do projeto
//! embarcado (pilar 0 do `roadmaps/42`).
//!
//! Fino: valida params, pede o modelo a `crate::project` com o chip do kit e
//! devolve. O mesmo modelo viaja em `event.project.changed` quando o workspace
//! abre e quando um configure/build termina — as telas SEGUEM o modelo em vez
//! de perguntar a cada clique.

use kinein_protocol::{JsonRpcRequest, JsonRpcResponse, ProjectModelParams};
use serde_json::{Value, json};

use crate::Core;
use crate::rpc::{no_workspace_response, parse_params};

impl Core {
    /// Roteia os metodos `project.*`.
    pub(crate) fn project_request_response(
        &self,
        method: &str,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> Option<JsonRpcResponse> {
        match method {
            "project.model" => Some(self.project_model_response(request_id, params)),
            _ => None,
        }
    }

    /// `project.model` — o que o projeto E'.
    fn project_model_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        if let Err(response) = parse_params::<ProjectModelParams>(
            request_id.as_ref(),
            params,
            "project.model nao aceita parametros",
        ) {
            return *response;
        }
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "project.model");
        };
        JsonRpcResponse::success(request_id, json!(self.compute_project_model(&root)))
    }

    /// O modelo com o chip do kit por cima do que os arquivos dizem.
    pub(crate) fn compute_project_model(
        &self,
        root: &std::path::Path,
    ) -> kinein_protocol::ProjectModel {
        let toolchain = crate::toolchain::Toolchain::resolve(root, &self.detected_tools());
        crate::project::model(root, toolchain.chip())
    }

    /// Emite `event.project.changed` com o modelo recomputado. Chamado ao abrir
    /// o workspace e quando um configure/build termina — o que muda os
    /// artefatos e, com eles, o alvo.
    pub(crate) fn emit_project_changed(&self) {
        let (Some(root), Some(events)) = (self.workspace_root(), self.events.as_ref()) else {
            return;
        };
        let modelo = self.compute_project_model(&root);
        drop(events.send(JsonRpcRequest::notification(
            "event.project.changed",
            Some(json!(modelo)),
        )));
    }
}

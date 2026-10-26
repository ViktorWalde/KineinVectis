//! Thin dfu.list handler. Discovery stays outside the core loop.
use crate::{
    Core, dfu,
    rpc::{no_workspace_response, parse_params},
};
use kinein_protocol::{DfuListParams, JsonRpcError, JsonRpcErrorCode, JsonRpcResponse};
use serde_json::{Value, json};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

impl Core {
    pub(crate) fn dfu_request_response(
        &mut self,
        method: &str,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> Option<JsonRpcResponse> {
        match method {
            "dfu.list" => Some(self.dfu_list_response(request_id, params)),
            _ => None,
        }
    }

    fn dfu_list_response(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "dfu.list");
        };
        if let Err(response) = parse_params::<DfuListParams>(
            request_id.as_ref(),
            params,
            "dfu.list não aceita parâmetros",
        ) {
            return *response;
        }
        self.cancel_dfu_query();
        let cancel = Arc::new(AtomicBool::new(false));
        self.dfu_query = Some(cancel.clone());
        let program = self.detector.find_in_path("dfu-util");
        let revision = program
            .as_deref()
            .and_then(|path| crate::platform::file_revision(path).ok());
        let queried_program = program.clone();
        let wait_cancel = cancel.clone();
        self.defer_then(
            request_id,
            move || dfu::list(queried_program.as_deref(), &wait_cancel),
            move |core, request_id, result| {
                if cancel.load(Ordering::SeqCst)
                    || !core
                        .dfu_query
                        .as_ref()
                        .is_some_and(|pending| Arc::ptr_eq(pending, &cancel))
                {
                    return stale(request_id);
                }
                core.dfu_query = None;
                if core.workspace_root().as_ref() != Some(&root)
                    || core.detector.find_in_path("dfu-util") != program
                    || program
                        .as_deref()
                        .and_then(|path| crate::platform::file_revision(path).ok())
                        != revision
                {
                    return stale(request_id);
                }
                JsonRpcResponse::success(request_id, json!(result))
            },
        )
    }

    pub(crate) fn cancel_dfu_query(&mut self) {
        if let Some(cancel) = self.dfu_query.take() {
            cancel.store(true, Ordering::SeqCst);
        }
    }
}

fn stale(request_id: Option<Value>) -> JsonRpcResponse {
    JsonRpcResponse::failure(
        request_id,
        JsonRpcError::new(
            JsonRpcErrorCode::InvalidRequest,
            "A consulta USB DFU foi substituída ou o workspace/ferramenta mudou; solicite uma nova lista",
            None,
        ),
    )
}

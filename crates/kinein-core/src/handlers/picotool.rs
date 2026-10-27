//! Bounded picotool query with the existing automatic reservation and `defer_then`.
use crate::{
    Core, picotool,
    rpc::{no_workspace_response, parse_params},
};
use kinein_protocol::{JsonRpcError, JsonRpcErrorCode, JsonRpcResponse, PicoListParams};
use serde_json::{Value, json};
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

impl Core {
    pub(crate) fn picotool_request_response(
        &mut self,
        method: &str,
        id: Option<Value>,
        params: Option<&Value>,
    ) -> Option<JsonRpcResponse> {
        match method {
            "picotool.list" => Some(self.picotool_list_response(id, params)),
            _ => None,
        }
    }

    fn picotool_list_response(
        &mut self,
        id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(id, "picotool.list");
        };
        if let Err(response) = parse_params::<PicoListParams>(
            id.as_ref(),
            params,
            "picotool.list não aceita parâmetros",
        ) {
            return *response;
        }
        let program = self.detector.find_in_path("picotool");
        if program.is_none() {
            self.cancel_picotool_query();
            return JsonRpcResponse::success(
                id,
                json!(picotool::failed(
                    None,
                    "picotool não encontrado no PATH; instale pelo catálogo antes de consultar"
                )),
            );
        }
        let previous = self.picotool_query.take();
        let (lease, gate) = if let Some(previous) = previous {
            previous.cancel.store(true, Ordering::SeqCst);
            (previous.lease, previous.gate)
        } else {
            let lease = match self.serial_usage.acquire_automatic(
                "picotool.list",
                "Consultar RP2040 BOOTSEL",
                Some(&root),
            ) {
                Ok(lease) => Arc::new(lease),
                Err(error) => return JsonRpcResponse::failure(id, error),
            };
            (lease, Arc::new(Mutex::new(())))
        };
        let cancel = Arc::new(AtomicBool::new(false));
        self.picotool_query = Some(picotool::Pending {
            cancel: cancel.clone(),
            gate: gate.clone(),
            lease: lease.clone(),
        });
        let worker_cancel = cancel.clone();
        let queried = program.clone();
        let deadline = Instant::now() + Duration::from_secs(5);
        let revision = program
            .as_deref()
            .and_then(|p| crate::platform::file_revision(p).ok());
        self.defer_then(
            id,
            move || {
                picotool::exclusive(&gate, &worker_cancel, deadline).map_or_else(
                    || {
                        picotool::failed(
                            queried.as_deref(),
                            "consulta cancelada ou prazo expirou ao aguardar a anterior",
                        )
                    },
                    |_exclusive| {
                        picotool::list(
                            queried.as_deref(),
                            deadline,
                            &worker_cancel,
                            Some(lease.as_ref()),
                        )
                    },
                )
            },
            move |core, id, result| {
                if cancel.load(Ordering::SeqCst)
                    || !core
                        .picotool_query
                        .as_ref()
                        .is_some_and(|q| Arc::ptr_eq(&q.cancel, &cancel))
                {
                    return stale(id);
                }
                core.picotool_query = None;
                if core.workspace_root().as_ref() != Some(&root)
                    || core.detector.find_in_path("picotool") != program
                    || program
                        .as_deref()
                        .and_then(|p| crate::platform::file_revision(p).ok())
                        != revision
                {
                    return stale(id);
                }
                JsonRpcResponse::success(id, json!(result))
            },
        )
    }

    pub(crate) fn cancel_picotool_query(&mut self) {
        if let Some(query) = self.picotool_query.take() {
            query.cancel.store(true, Ordering::SeqCst);
        }
    }
}

fn stale(id: Option<Value>) -> JsonRpcResponse {
    JsonRpcResponse::failure(
        id,
        JsonRpcError::new(
            JsonRpcErrorCode::InvalidRequest,
            "Consulta RP2040 substituída ou workspace/ferramenta alterado; consulte novamente",
            None,
        ),
    )
}

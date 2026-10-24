//! `platformio.homeStart` / `platformio.homeStop` (0.191.0, 52 §14.45) and
//! `platformio.homeRelease` (0.192.0, §14.46): thin, the server lifecycle
//! lives in `crate::platformio::home`.

use std::{sync::Arc, time::Duration};

use kinein_protocol::{
    JsonRpcError, JsonRpcErrorCode, JsonRpcRequest, JsonRpcResponse, PlatformioHomeExitReason,
    PlatformioHomeExitedParams, PlatformioHomeReleaseParams, PlatformioHomeReleaseResult,
    PlatformioHomeStartResult, PlatformioHomeStopResult,
};
use serde_json::{Value, json};

use crate::Core;
use crate::platformio::home::START_DEADLINE;
use crate::rpc::parse_params;

/// `platformio.homeRelease` without `after`.
const RELEASE_DEFAULT_SECONDS: u32 = 300;
const RELEASE_MAX_SECONDS: u32 = 3600;

fn send_exited(
    events: Option<&crate::lsp::EventSender>,
    code: Option<i32>,
    reason: PlatformioHomeExitReason,
) {
    if let Some(events) = events {
        drop(events.send(JsonRpcRequest::notification(
            "event.platformio.homeExited",
            Some(json!(PlatformioHomeExitedParams { code, reason })),
        )));
    }
}

impl Core {
    /// Routes the `platformio.*` methods.
    pub(crate) fn platformio_home_request_response(
        &self,
        method: &str,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> Option<JsonRpcResponse> {
        match method {
            "platformio.homeStart" => Some(self.platformio_home_start_response(request_id)),
            "platformio.homeStop" => Some(self.platformio_home_stop_response(request_id)),
            "platformio.homeRelease" => {
                Some(self.platformio_home_release_response(request_id, params))
            }
            _ => None,
        }
    }

    fn platformio_home_start_response(&self, request_id: Option<Value>) -> JsonRpcResponse {
        let Some(pio) = self
            .detector
            .find_in_path("pio")
            .or_else(|| self.detector.find_in_path("platformio"))
        else {
            return JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(
                    JsonRpcErrorCode::ToolNotFound,
                    "platformio.homeStart: o PlatformIO Core não está nesta máquina (pipx install platformio)",
                    Some(json!({ "tool": "pio" })),
                ),
            );
        };
        let home = Arc::clone(&self.platformio_home);
        let events = self.events.clone();
        self.defer_work(request_id, move |request_id| {
            let agent_events = events.clone();
            let on_exit = move |code: Option<i32>| {
                send_exited(events.as_ref(), code, PlatformioHomeExitReason::Exited);
            };
            match home.start(&pio, START_DEADLINE, agent_events, on_exit) {
                Ok(session) => JsonRpcResponse::success(
                    request_id,
                    json!(PlatformioHomeStartResult {
                        url: session.url,
                        port: session.port,
                    }),
                ),
                Err(message) => JsonRpcResponse::failure(
                    request_id,
                    JsonRpcError::new(
                        JsonRpcErrorCode::InternalError,
                        format!("platformio.homeStart: {message}"),
                        None,
                    ),
                ),
            }
        })
    }

    /// The page left the screen: the core stops the server after `after`
    /// seconds unless a start or a stop comes first.
    fn platformio_home_release_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let hint = "platformio.homeRelease aceita { after? }, de 0 a 3600 s";
        let request =
            match parse_params::<PlatformioHomeReleaseParams>(request_id.as_ref(), params, hint) {
                Ok(request) => request,
                Err(response) => return *response,
            };
        let after = request.after.unwrap_or(RELEASE_DEFAULT_SECONDS);
        if after > RELEASE_MAX_SECONDS {
            return JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(
                    JsonRpcErrorCode::InvalidParams,
                    hint,
                    Some(json!({ "after": after })),
                ),
            );
        }
        let events = self.events.clone();
        let releasing =
            self.platformio_home
                .release(Duration::from_secs(u64::from(after)), move || {
                    send_exited(events.as_ref(), None, PlatformioHomeExitReason::Idle);
                });
        JsonRpcResponse::success(request_id, json!(PlatformioHomeReleaseResult { releasing }))
    }

    fn platformio_home_stop_response(&self, request_id: Option<Value>) -> JsonRpcResponse {
        let home = Arc::clone(&self.platformio_home);
        self.defer_work(request_id, move |request_id| {
            JsonRpcResponse::success(
                request_id,
                json!(PlatformioHomeStopResult {
                    stopped: home.stop(),
                }),
            )
        })
    }
}

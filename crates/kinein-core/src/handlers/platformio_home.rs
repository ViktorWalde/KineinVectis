//! `platformio.homeStart` / `platformio.homeStop` (0.191.0, 52 §14.45): thin,
//! the server lifecycle lives in `crate::platformio::home`.

use std::sync::Arc;

use kinein_protocol::{
    JsonRpcError, JsonRpcErrorCode, JsonRpcRequest, JsonRpcResponse, PlatformioHomeExitedParams,
    PlatformioHomeStartResult, PlatformioHomeStopResult,
};
use serde_json::{Value, json};

use crate::Core;
use crate::platformio::home::START_DEADLINE;

impl Core {
    /// Routes the `platformio.*` methods.
    pub(crate) fn platformio_home_request_response(
        &self,
        method: &str,
        request_id: Option<Value>,
    ) -> Option<JsonRpcResponse> {
        match method {
            "platformio.homeStart" => Some(self.platformio_home_start_response(request_id)),
            "platformio.homeStop" => Some(self.platformio_home_stop_response(request_id)),
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
            let on_exit = move |code: Option<i32>| {
                if let Some(events) = events.as_ref() {
                    drop(events.send(JsonRpcRequest::notification(
                        "event.platformio.homeExited",
                        Some(json!(PlatformioHomeExitedParams { code })),
                    )));
                }
            };
            match home.start(&pio, START_DEADLINE, on_exit) {
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

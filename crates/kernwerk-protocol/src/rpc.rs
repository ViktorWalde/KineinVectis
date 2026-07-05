//! JSON-RPC envelope: requests, responses, errors and stable error codes.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::JSON_RPC_VERSION;

/// JSON-RPC request identifier.
///
/// JSON-RPC allows string, number, or null identifiers. The protocol keeps this
/// as a JSON value so the UI can preserve the exact identifier it sent.
pub type JsonRpcId = Value;

/// JSON-RPC request sent from a client to the core.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    /// JSON-RPC version. Kernwerk currently accepts only `2.0`.
    pub jsonrpc: String,
    /// Request identifier. Events and notifications omit this field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<JsonRpcId>,
    /// Method name, such as `core.ping`.
    pub method: String,
    /// Method parameters.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

impl JsonRpcRequest {
    /// Builds a JSON-RPC request with an identifier.
    #[must_use]
    pub fn new(id: impl Into<JsonRpcId>, method: impl Into<String>, params: Option<Value>) -> Self {
        Self {
            jsonrpc: JSON_RPC_VERSION.to_owned(),
            id: Some(id.into()),
            method: method.into(),
            params,
        }
    }

    /// Builds a JSON-RPC notification without an identifier.
    #[must_use]
    pub fn notification(method: impl Into<String>, params: Option<Value>) -> Self {
        Self {
            jsonrpc: JSON_RPC_VERSION.to_owned(),
            id: None,
            method: method.into(),
            params,
        }
    }

    /// Returns `true` when the request uses the supported JSON-RPC version.
    #[must_use]
    pub fn has_supported_version(&self) -> bool {
        self.jsonrpc == JSON_RPC_VERSION
    }
}

/// JSON-RPC response emitted by the core.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    /// JSON-RPC version. Always `2.0`.
    pub jsonrpc: String,
    /// Identifier copied from the request.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<JsonRpcId>,
    /// Successful response payload.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    /// Error response payload.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

impl JsonRpcResponse {
    /// Builds a successful JSON-RPC response.
    #[must_use]
    pub fn success(id: Option<JsonRpcId>, result: Value) -> Self {
        Self {
            jsonrpc: JSON_RPC_VERSION.to_owned(),
            id,
            result: Some(result),
            error: None,
        }
    }

    /// Builds an error JSON-RPC response.
    #[must_use]
    pub fn failure(id: Option<JsonRpcId>, error: JsonRpcError) -> Self {
        Self {
            jsonrpc: JSON_RPC_VERSION.to_owned(),
            id,
            result: None,
            error: Some(error),
        }
    }
}

/// Error object used by JSON-RPC responses.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct JsonRpcError {
    /// Stable machine-readable error code.
    pub code: JsonRpcErrorCode,
    /// Human-readable message suitable for logs or UI notifications.
    pub message: String,
    /// Structured technical details for diagnostics.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
}

impl JsonRpcError {
    /// Builds a protocol error with optional structured details.
    #[must_use]
    pub fn new(code: JsonRpcErrorCode, message: impl Into<String>, details: Option<Value>) -> Self {
        Self {
            code,
            message: message.into(),
            details,
        }
    }
}

/// Stable error codes used by Kernwerk Studio.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum JsonRpcErrorCode {
    /// The input was not valid JSON.
    ParseError,
    /// The JSON value was not a valid Kernwerk request.
    InvalidRequest,
    /// The requested method is not registered.
    MethodNotFound,
    /// Request parameters failed validation.
    InvalidParams,
    /// The core failed while processing the command.
    InternalError,
    /// A required external tool was not found.
    ToolNotFound,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::{
        CorePingResult, JSON_RPC_VERSION, JsonRpcError, JsonRpcErrorCode, JsonRpcRequest,
        JsonRpcResponse,
    };

    #[test]
    fn request_constructor_uses_json_rpc_version() {
        let request = JsonRpcRequest::new(1_i64, "core.ping", Some(json!({})));

        assert_eq!(request.jsonrpc, JSON_RPC_VERSION);
        assert_eq!(request.method, "core.ping");
        assert!(request.has_supported_version());
    }

    #[test]
    fn response_serializes_success_without_error_field() {
        let response = JsonRpcResponse::success(Some(json!(1)), json!(CorePingResult::default()));
        let value = serde_json::to_value(response).unwrap();

        assert_eq!(value["jsonrpc"], JSON_RPC_VERSION);
        assert_eq!(value["id"], 1);
        assert_eq!(value["result"]["message"], "pong");
        assert!(value.get("error").is_none());
    }

    #[test]
    fn response_serializes_failure_without_result_field() {
        let error = JsonRpcError::new(JsonRpcErrorCode::MethodNotFound, "unknown method", None);
        let response = JsonRpcResponse::failure(Some(json!(7)), error);
        let value = serde_json::to_value(response).unwrap();

        assert_eq!(value["id"], 7);
        assert_eq!(value["error"]["code"], "METHOD_NOT_FOUND");
        assert!(value.get("result").is_none());
    }
}

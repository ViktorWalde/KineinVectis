//! Core lifecycle payloads (`core.ping`).

use serde::{Deserialize, Serialize};

use crate::PROTOCOL_VERSION;

/// Result payload for `core.ping`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CorePingResult {
    /// Status marker used by smoke tests and UI health checks.
    pub status: String,
    /// Human-readable ping answer.
    pub message: String,
    /// Kinein Vectis IPC protocol version.
    pub protocol_version: String,
}

impl Default for CorePingResult {
    fn default() -> Self {
        Self {
            status: "ok".to_owned(),
            message: "pong".to_owned(),
            protocol_version: PROTOCOL_VERSION.to_owned(),
        }
    }
}

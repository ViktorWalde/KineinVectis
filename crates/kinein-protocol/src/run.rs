//! User process payloads (`run.*`).

use serde::{Deserialize, Serialize};

/// Parameters for `run.start`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RunStartParams {
    /// Shell command to execute in the workspace root. When absent, the core
    /// derives a default from the project kind (e.g. `cargo run`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
}

/// Result payload for `run.start`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunStartResult {
    /// Command that is now running.
    pub command: String,
}

/// Parameters for `run.stdin`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RunStdinParams {
    /// Raw bytes forwarded to the child stdin. The UI appends the newline.
    pub data: String,
}

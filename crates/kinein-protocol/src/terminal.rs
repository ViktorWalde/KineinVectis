//! Terminal/PTY payloads (`terminal.*`).

use serde::{Deserialize, Serialize};

/// Result payload for `terminal.open`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalOpenResult {
    /// Shell the session is running (from `$SHELL`).
    pub shell: String,
}

/// Parameters for `terminal.input`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TerminalInputParams {
    /// Raw bytes forwarded to the shell PTY. The UI appends the newline.
    pub data: String,
}

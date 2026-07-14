//! Terminal/PTY payloads (`terminal.*`).

use serde::{Deserialize, Serialize};

/// Result payload for `terminal.open`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalOpenResult {
    /// Id da sessão criada (D2.3): todo comando e evento seguinte usa esse id.
    pub id: String,
    /// Shell the session is running (from `$SHELL`).
    pub shell: String,
}

/// Parameters for `terminal.close` (D2.3): qual sessão fechar.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TerminalCloseParams {
    /// Id da sessão.
    pub id: String,
}

/// Parameters for `terminal.input`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TerminalInputParams {
    /// Id da sessão (D2.3).
    pub id: String,
    /// Raw bytes forwarded to the shell PTY (keys, control chars). The UI
    /// sends each keystroke, not whole lines (D2, docs/24).
    pub data: String,
}

/// Parameters for `terminal.resize` (D2, docs/24): new grid size in cells.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TerminalResizeParams {
    /// Id da sessão (D2.3).
    pub id: String,
    /// Number of columns (cells wide).
    pub cols: u16,
    /// Number of rows (cells tall).
    pub rows: u16,
}

/// Parameters for `terminal.scroll` (D2.2): rows above the live bottom.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TerminalScrollParams {
    /// Id da sessão (D2.3).
    pub id: String,
    /// Scrollback offset in rows (0 = live/bottom; clamped to the buffer).
    pub offset: u16,
}

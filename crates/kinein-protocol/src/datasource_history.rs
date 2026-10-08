//! Query history contracts for `datasource.history` (`0.166.0`, step 13b).
//!
//! The core owns the history because it runs the statements; it lives outside
//! the project, in the user's state directory, never in version control.

use serde::{Deserialize, Serialize};

/// Parameters for `datasource.history` and `datasource.history.clear`.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DataSourceHistoryParams {
    /// The saved profile whose history is read or cleared.
    pub name: String,
}

/// How an executed statement ended.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DataSourceHistoryOutcome {
    /// The engine accepted and ran it.
    Ok,
    /// The engine reported an error.
    Failed,
}

/// One statement that ran.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataSourceHistoryEntry {
    /// The text that was sent to the engine.
    pub sql: String,
    /// When it finished, in seconds since the Unix epoch (UTC).
    pub at: u64,
    /// How it ended.
    pub outcome: DataSourceHistoryOutcome,
    /// Rows read or affected, when the engine reports them.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rows: Option<u64>,
}

/// Result of `datasource.history`: newest first.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataSourceHistoryResult {
    /// The profile.
    pub name: String,
    /// The recorded statements, newest first.
    pub entries: Vec<DataSourceHistoryEntry>,
}

/// Result of `datasource.history.clear`.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataSourceHistoryCleared {
    /// The profile whose history is now empty.
    pub name: String,
}

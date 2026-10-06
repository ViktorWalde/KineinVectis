//! Core-owned console identities and extraction of an editor instruction.
use crate::DataSourceOperationContext;
use serde::{Deserialize, Serialize};

/// Public paths identifying exactly one saved profile.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataSourceConsoleBinding {
    /// Saved profile name.
    pub name: String,
    /// New path plus an unambiguous legacy path, when available.
    pub paths: Vec<String>,
}

/// Ensure the console file of a saved profile.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DataSourceConsoleParams {
    /// Saved profile name.
    pub name: String,
    /// Public correlation token; old clients may omit it.
    #[serde(default)]
    pub client_context: Option<String>,
    /// Public destination snapshot; old clients may omit it.
    #[serde(default)]
    pub expected_context: Option<DataSourceOperationContext>,
}

/// File prepared by `datasource.console`, without executing SQL.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataSourceConsoleResult {
    /// Exact absolute path.
    pub path: String,
    /// Whether this call created the file.
    pub created: bool,
    /// Saved profile name.
    pub name: String,
    /// Public correlation token.
    pub client_context: Option<String>,
    /// Destination used by the core.
    pub expected_context: DataSourceOperationContext,
}

/// Extract an instruction without passwords, jobs or a database connection.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DataSourceConsoleStatementParams {
    /// Saved profile name.
    pub name: String,
    /// Exact console path, checked against core bindings.
    pub path: String,
    /// Current editor buffer, including unsaved changes; at most 1 MiB.
    pub text: String,
    /// Cursor offset in UTF-16 code units, as used by Qt.
    pub cursor: usize,
    /// Selection start in UTF-16 code units.
    pub selection_start: usize,
    /// Selection end in UTF-16 code units.
    pub selection_end: usize,
    /// Public correlation token.
    pub client_context: String,
    /// Public destination snapshot.
    pub expected_context: DataSourceOperationContext,
}

/// Instruction resolved in the core; the existing query policy still applies.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataSourceConsoleStatementResult {
    /// Saved profile name.
    pub name: String,
    /// Exact console path.
    pub path: String,
    /// Empty on a comment or empty buffer; never persisted by this method.
    pub statement: String,
    /// Public correlation token.
    pub client_context: String,
    /// Destination used by the core.
    pub expected_context: DataSourceOperationContext,
}

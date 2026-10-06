//! `datasource.impact` (`0.150.0`): what a write would do, measured BEFORE it
//! runs. The UI shows the command and the consequence and asks; a destructive
//! statement (a `DELETE`/`UPDATE` without `WHERE`, `TRUNCATE`, `DROP TABLE`,
//! `DROP SCHEMA`, `DROP DATABASE`, a dropped column) also asks the author to
//! type the name of what goes away.

use serde::{Deserialize, Serialize};

/// How heavy a statement is. Ordered: `Read < Write < Destructive`.
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "camelCase")]
pub enum SqlImpactSeverity {
    /// Only reads.
    #[default]
    Read,
    /// Writes, but a filter or a definition bounds it.
    Write,
    /// Destroys data that a filter does not bound.
    Destructive,
}

/// One statement of the text and what it would touch.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SqlStatementImpact {
    /// The statement as written.
    pub text: String,
    /// `read`, `insert`, `update`, `delete`, `truncate`, `dropTable`,
    /// `dropView`, `dropIndex`, `dropSchema`, `dropDatabase`, `dropColumn`,
    /// `drop`, `alter`, `create` or `other`; for `MongoDB` (`0.155.0`)
    /// `mongoInsert`, `mongoUpdate`, `mongoDelete` or `dropCollection`.
    pub kind: String,
    /// The tables (or schema, or database) the statement names.
    pub targets: Vec<String>,
    /// The column of a `dropColumn`.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub column: String,
    /// The top-level `WHERE` of a `delete`/`update` (empty = every row).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub filter: String,
    /// How heavy it is (after the count: a `WHERE` that matches every row of
    /// a non-empty table becomes `destructive`).
    pub severity: SqlImpactSeverity,
    /// Rows it would hit (values, for a dropped column; tables, for a
    /// dropped schema), when the count ran.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rows: Option<u64>,
    /// Rows the table has, for a filtered `delete`/`update`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_rows: Option<u64>,
    /// Why there is no count (the engine refused the counting read).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// Parameters for `datasource.impact`.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DataSourceImpactParams {
    /// Which saved profile.
    pub name: String,
    /// The session password, when the profile's policy is `Prompt`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    /// The text the author is about to run.
    pub sql: String,
    /// Public request token; measurement cannot authorize a different operation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_context: Option<String>,
    /// Saved destination shown by the UI.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_context: Option<crate::DataSourceOperationContext>,
}

/// Payload of `event.datasource.impact`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataSourceImpactEvent {
    /// The job that measured it.
    pub job_id: String,
    /// The profile.
    pub name: String,
    /// The text, as it was asked about (the UI runs exactly this).
    pub sql: String,
    /// The heaviest statement.
    pub severity: SqlImpactSeverity,
    /// Statement by statement.
    pub statements: Vec<SqlStatementImpact>,
    /// Public token echoed from the measurement request.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_context: Option<String>,
    /// Literal target name the UI must ask for; absent for an ordinary warning.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confirmation_target: Option<String>,
    /// Production removal also requires the full connection name.
    #[serde(default)]
    pub requires_connection: bool,
}

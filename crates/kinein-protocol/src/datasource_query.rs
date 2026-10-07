//! Query execution contracts for `datasource.query` and its result event.

use serde::{Deserialize, Serialize};

/// Parameters for `datasource.query` — run what the author wrote (`0.121.0`).
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DataSourceQueryParams {
    /// Which saved profile to run against.
    pub name: String,
    /// The session password, when the profile's policy is `Prompt`. Redacted
    /// from the client log exactly like `datasource.test`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    /// SQL, or one `collection.operation(JSON arguments)` command for `MongoDB`.
    /// The legacy `<collection> <JSON filter>` read form also remains valid.
    pub sql: String,
    /// Row ceiling for a read; absent = 500. The result says when it hit it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_rows: Option<u32>,
    /// Acknowledges an operation requiring confirmation (`0.156.0`): deletion,
    /// an unfiltered mass update or an unknown operation. Ordinary writes run
    /// directly; a filtered update affecting every record also requires it.
    #[serde(default)]
    pub confirm_write: bool,
    /// Opaque public request token echoed by the event, never a secret.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_context: Option<String>,
    /// Destination snapshot checked before opening a connection.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_context: Option<crate::DataSourceOperationContext>,
    /// Names typed for production removals or an unknown operation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confirmation: Option<crate::DataSourceConfirmation>,
    /// Execute an eligible `PostgreSQL` write in a transaction awaiting a decision.
    #[serde(default)]
    pub preview: bool,
}

/// Catalogue refresh requested after an execution; not a commit guarantee.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DataSourceCatalogUpdate {
    /// No refresh requested; this does not assert an unchanged database.
    #[default]
    None,
    /// Introspect again because the cached catalogue may be stale.
    Reload,
}

/// Payload of `event.datasource.queried`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataSourceQueriedEvent {
    /// The job that ran it.
    pub job_id: String,
    /// The profile.
    pub name: String,
    /// Whether the engine accepted and ran the statement.
    pub success: bool,
    /// Column names of the result set (empty for a write).
    pub columns: Vec<String>,
    /// Rows as text cells; `null` is SQL `NULL` (or an absent key in Mongo).
    pub rows: Vec<Vec<Option<String>>>,
    /// Rows returned (after the ceiling).
    pub row_count: usize,
    /// Rows a write touched, when the engine reports it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub affected: Option<u64>,
    /// `true` when the ceiling cut the result.
    pub truncated: bool,
    /// Wall time of the statement, in milliseconds.
    pub elapsed_ms: u64,
    /// The engine's error, in its own words, on failure.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// `true` when asking for the password and retrying is the next step.
    #[serde(default)]
    pub secret_required: bool,
    /// Exact text awaiting confirmation after a blocked preflight.
    /// Present only when no write ran and user confirmation is required.
    /// Allows the UI to ignore a result for an older request to the same profile.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confirmation_sql: Option<String>,
    /// Public token of the original request, including a blocked preflight.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_context: Option<String>,
    /// Engine path used, even when zero rows changed or the operation failed.
    #[serde(default)]
    pub access: crate::DataSourceQueryAccess,
    /// Real final transaction outcome, only for a preview query.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preview_outcome: Option<crate::DataSourcePreviewOutcome>,
    /// Catalogue may be stale after execution, including a partially failed batch.
    /// This requests introspection; it does not assert success or a committed write.
    #[serde(default)]
    pub catalog_update: DataSourceCatalogUpdate,
}

#[cfg(test)]
mod tests {
    use super::{DataSourceCatalogUpdate, DataSourceQueriedEvent};

    #[test]
    fn old_query_events_default_to_no_catalogue_invalidation() {
        let mut old = serde_json::to_value(DataSourceQueriedEvent::default()).unwrap();
        old.as_object_mut().unwrap().remove("catalogUpdate");
        let decoded: DataSourceQueriedEvent = serde_json::from_value(old).unwrap();
        assert_eq!(decoded.catalog_update, DataSourceCatalogUpdate::None);
        let invalidated = DataSourceQueriedEvent {
            catalog_update: DataSourceCatalogUpdate::Reload,
            success: false,
            ..DataSourceQueriedEvent::default()
        };
        let json = serde_json::to_value(&invalidated).unwrap();
        assert_eq!(json["catalogUpdate"], "reload");
        assert_eq!(
            serde_json::from_value::<DataSourceQueriedEvent>(json).unwrap(),
            invalidated
        );
    }
}

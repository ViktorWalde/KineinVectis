//! `PostgreSQL` write preview: public messages, never a connection or credential.

use serde::{Deserialize, Serialize};

/// Query job acceptance, including context for cancelling a superseded preview.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataSourceQueryAccepted {
    /// Existing background job identifier.
    pub job_id: String,
    /// Saved connection name.
    pub name: String,
    /// Public token echoed from the request.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_context: Option<String>,
    /// This job may hold a transaction waiting for a decision.
    #[serde(default)]
    pub preview: bool,
}

/// Explicit decision about an executed, still uncommitted write.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DataSourcePreviewDecision {
    /// Persist the transaction's changes.
    Commit,
    /// Discard the transaction's changes.
    Rollback,
}

/// Final transaction outcome; a transport error during commit is unknown.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DataSourcePreviewOutcome {
    /// Server acknowledged COMMIT.
    Committed,
    /// Server acknowledged ROLLBACK.
    RolledBack,
    /// No decision was accepted before the deadline.
    Expired,
    /// Job or destination was cancelled before a decision.
    Cancelled,
    /// Query, connection or transaction failed.
    Failed,
    /// COMMIT was sent but its result could not be established.
    Unknown,
}

/// Required context of `datasource.preview.decide`; one use only.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DataSourcePreviewDecideParams {
    /// Identifier of the ready preview.
    pub preview_id: String,
    /// Author's explicit decision.
    pub decision: DataSourcePreviewDecision,
    /// Saved connection name.
    pub name: String,
    /// Token of the original query.
    pub client_context: String,
    /// Original public destination, checked again when deciding.
    pub expected_context: crate::DataSourceOperationContext,
}

/// Payload of `event.datasource.previewed`, while the job remains running.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataSourcePreviewedEvent {
    /// Job owning the live transaction.
    pub job_id: String,
    /// Saved connection name.
    pub name: String,
    /// Token of the original query.
    pub client_context: String,
    /// Identifier consumed by the decision RPC.
    pub preview_id: String,
    /// Initial decision deadline, measured from the ready event.
    pub expires_in_seconds: u32,
    /// Original command requested by the author.
    pub sql: String,
    /// Executed command, including an added RETURNING clause when necessary.
    pub executed_sql: String,
    /// Column names of the returned sample.
    pub columns: Vec<String>,
    /// Bounded sample; SQL NULL remains null.
    pub rows: Vec<Vec<Option<String>>>,
    /// Full affected count from the server's command tag.
    pub affected: u64,
    /// The sample does not contain every returned row.
    pub truncated: bool,
    /// Time spent executing the write, before awaiting a decision.
    pub elapsed_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn decisions_require_context_and_do_not_accept_authority_or_secrets() {
        let value = json!({"previewId":"preview-1","decision":"commit","name":"local",
            "clientContext":"1:2","expectedContext":{"workspace":"/p","profile":{
            "name":"local","host":"localhost","port":5432,"database":"d","user":"u"}}});
        assert!(serde_json::from_value::<DataSourcePreviewDecideParams>(value.clone()).is_ok());
        for field in ["expectedContext", "clientContext", "previewId", "decision"] {
            let mut wrong = value.clone();
            wrong.as_object_mut().unwrap().remove(field);
            assert!(serde_json::from_value::<DataSourcePreviewDecideParams>(wrong).is_err());
        }
        for field in ["password", "confirmWrite", "timeout", "allow"] {
            let mut wrong = value.clone();
            wrong[field] = json!(true);
            assert!(serde_json::from_value::<DataSourcePreviewDecideParams>(wrong).is_err());
        }
    }
}

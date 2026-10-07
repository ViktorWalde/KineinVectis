//! Disconnect waits for destination workers; saved profiles and data stay intact.

use serde::{Deserialize, Serialize};

use crate::DataSourceOperationContext;

/// Explicit destination and token are mandatory; no credential is requested.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DataSourceDisconnectParams {
    /// Saved destination to drain.
    pub name: String,
    /// Workspace and profile the caller intends to disconnect.
    pub expected_context: DataSourceOperationContext,
    /// Public opaque caller token, echoed in acceptance and terminal event.
    pub client_context: String,
}

/// Acceptance starts draining; only the terminal event confirms disconnection.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataSourceDisconnectAccepted {
    /// Drain job tracked by the existing job infrastructure.
    pub job_id: String,
    /// Destination being drained.
    pub name: String,
    /// Caller token; acceptance does not imply completion.
    pub client_context: String,
}

/// Emitted after all tracked destination workers have released their resources.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataSourceDisconnectedEvent {
    /// Completed drain job.
    pub job_id: String,
    /// Original destination.
    pub name: String,
    /// Original caller token, never a credential.
    pub client_context: String,
    /// True only after the tracked destination workers have exited.
    pub success: bool,
    /// Public completion or failure explanation.
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn disconnect_requires_public_context_and_rejects_credentials() {
        assert!(
            serde_json::from_value::<DataSourceDisconnectParams>(json!({"name":"bank"})).is_err()
        );
        let request = json!({"name":"bank", "clientContext":"close.1",
            "expectedContext":{"workspace":"/project","profile":{"name":"bank", "host":"localhost", "port":5432,"database":"db","user":"user"}}});
        assert!(serde_json::from_value::<DataSourceDisconnectParams>(request.clone()).is_ok());
        let mut secret = request;
        secret["password"] = json!("never");
        assert!(serde_json::from_value::<DataSourceDisconnectParams>(secret).is_err());
    }
}

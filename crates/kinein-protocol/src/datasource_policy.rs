//! Public context and typed confirmation. These types never contain a password.

use serde::{Deserialize, Serialize};

use crate::DataSourceProfile;

/// Engine path used by a query; failure on a write path may have partial effects.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DataSourceQueryAccess {
    /// The engine read path was used; its guarantees depend on the engine.
    #[default]
    Read,
    /// The author authorized the write path.
    Write,
}

/// Destination the UI displayed when the author requested an operation.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DataSourceOperationContext {
    /// Absolute workspace path returned by the core.
    pub workspace: String,
    /// Saved public profile, before any draft edit.
    pub profile: DataSourceProfile,
}

/// Names explicitly typed for a production operation that removes data.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DataSourceConfirmation {
    /// Full saved connection name.
    pub connection: String,
    /// Target shown by the impact policy; connection name if unknown.
    pub target: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn old_profiles_default_to_development_and_context_never_accepts_credentials() {
        let old = json!({"name":"local","host":"localhost","port":5432,"database":"dados","user":"autor"});
        let profile: DataSourceProfile = serde_json::from_value(old.clone()).unwrap();
        assert!(!profile.production && !profile.read_only);
        for field in ["production", "readOnly"] {
            let mut wrong = old.clone();
            wrong[field] = json!("true");
            assert!(serde_json::from_value::<DataSourceProfile>(wrong).is_err());
        }
        let mut context = json!({"workspace":"/projeto","profile":old});
        assert!(serde_json::from_value::<DataSourceOperationContext>(context.clone()).is_ok());
        context["profile"]["password"] = json!("nunca");
        assert!(serde_json::from_value::<DataSourceOperationContext>(context).is_err());
        for extra in ["password", "allow", "confirmWrite"] {
            let mut confirmation = json!({"connection":"local","target":"tabela"});
            confirmation[extra] = json!(true);
            assert!(serde_json::from_value::<DataSourceConfirmation>(confirmation).is_err());
        }
    }
}

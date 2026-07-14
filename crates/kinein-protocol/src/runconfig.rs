//! Run configuration payloads (`runConfig.*`).

use serde::{Deserialize, Serialize};

/// One named run configuration of the workspace.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunConfigInfo {
    /// Stable identifier (`cfg-1`, `cfg-2`, ...).
    pub id: String,
    /// Human-friendly name shown in the toolbar selector.
    pub name: String,
    /// Shell command executed from the workspace root by `run.start`.
    pub command: String,
}

/// Result payload for every `runConfig.*` method.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunConfigsResult {
    /// Every configuration, in creation order.
    pub configs: Vec<RunConfigInfo>,
    /// Active configuration id; absent means "automatic" (heuristic).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_id: Option<String>,
}

/// Parameters for `runConfig.save` (create when `id` is absent).
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RunConfigSaveParams {
    /// Existing configuration id to update; absent creates a new one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Non-empty configuration name.
    pub name: String,
    /// Non-empty shell command.
    pub command: String,
}

/// Parameters for `runConfig.delete`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RunConfigDeleteParams {
    /// Configuration id to remove.
    pub id: String,
}

/// Parameters for `runConfig.setActive`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RunConfigSetActiveParams {
    /// Configuration id to activate; absent selects "automatic".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{RunConfigInfo, RunConfigSaveParams, RunConfigsResult};

    #[test]
    fn configs_result_serializes_camel_case_and_omits_missing_active() {
        let value = serde_json::to_value(RunConfigsResult {
            configs: vec![RunConfigInfo {
                id: "cfg-1".to_owned(),
                name: "Servidor".to_owned(),
                command: "cargo run --bin server".to_owned(),
            }],
            active_id: None,
        })
        .unwrap();

        assert_eq!(value["configs"][0]["id"], "cfg-1");
        assert!(value.get("activeId").is_none());
    }

    #[test]
    fn save_params_accept_missing_id_and_reject_unknown_fields() {
        let created: RunConfigSaveParams =
            serde_json::from_value(json!({ "name": "x", "command": "echo" })).unwrap();
        assert_eq!(created.id, None);

        let rejected = serde_json::from_value::<RunConfigSaveParams>(json!({
            "name": "x",
            "command": "echo",
            "extra": 1,
        }));
        assert!(rejected.is_err());
    }
}

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

/// Parameters for `runConfig.flashProposal` (`0.113.0`, E4 of
/// `integracoes/38` §6): the "Gravar" line as a run configuration PROPOSAL.
///
/// Pure: nothing runs. The core composes the engine's command line from
/// what the project model already read (flash recipe, ELF/UF2/BIN, target)
/// and the UI shows it as a preview; saving it is `runConfig.save`, running
/// it is `run.start` — flashing is a run configuration, not a new domain.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FlashProposalParams {
    /// Serial port for engines that need one (`esptool`): the port chosen in
    /// the Embedded panel. Absent = the engine's own detection, when it has
    /// one; `esptool` refuses without it (flashing the wrong board is worse
    /// than one click).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device: Option<String>,
    /// Engine to use (`esptool`, `probe-rs`, `picotool`, `dfu-util`); absent
    /// = the one the project model suggests (`target.flashEngine`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub engine: Option<String>,
    /// Flash capacity the board reported (`serial.identify`,
    /// `identity.flashSizeBytes`), for the "image larger than the flash"
    /// warning. Absent = no such check.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flash_size_bytes: Option<u64>,
}

/// Result of `runConfig.flashProposal`: a run configuration to save.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlashProposalResult {
    /// Suggested configuration name (`Gravar (esptool)`).
    pub name: String,
    /// The shell line, absolute paths single-quoted; editable by the user
    /// once saved, like any run configuration.
    pub command: String,
    /// The engine the line uses.
    pub engine: String,
    /// Where each piece came from, one line each (engine, recipe, images,
    /// port) — the evidence the preview shows.
    pub source: Vec<String>,
    /// What the user should know before running (encrypted images, image
    /// larger than the reported flash, tool version caveats).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
}

//! Run configuration payloads (`runConfig.*`).

use serde::{Deserialize, Serialize};

/// One named run configuration of the workspace.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RunConfigInfo {
    /// Stable identifier (`cfg-1`, `cfg-2`, ...).
    pub id: String,
    /// Human-friendly name shown in the toolbar selector.
    pub name: String,
    /// Shell command executed from the workspace root by `run.start`.
    pub command: String,
    /// Native flash intent; command is only its derived preview when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flash: Option<FlashIntent>,
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
    /// Preserve the complete native intent while saving or renaming it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flash: Option<FlashIntent>,
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

    use super::{FlashIntent, RunConfigInfo, RunConfigSaveParams, RunConfigsResult};

    #[test]
    fn configs_result_serializes_camel_case_and_omits_missing_active() {
        let value = serde_json::to_value(RunConfigsResult {
            configs: vec![RunConfigInfo {
                id: "cfg-1".to_owned(),
                name: "Servidor".to_owned(),
                command: "cargo run --bin server".to_owned(),
                flash: None,
            }],
            active_id: None,
        })
        .unwrap();

        assert_eq!(value["configs"][0]["id"], "cfg-1");
        assert!(value.get("activeId").is_none());
    }

    #[test]
    fn stored_config_does_not_discard_unsupported_fields() {
        assert!(
            serde_json::from_value::<RunConfigInfo>(json!({
                "id":"cfg-1", "name":"Native", "command":"echo preview", "flash":{"device":"board"}
            }))
            .is_err()
        );
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

    #[test]
    fn native_intent_has_a_strict_round_trip_without_reinterpreting_argv() {
        let value = json!({"version":1,"workspace":"/workspace","selection":{
            "native":true,"engine":"esptool","device":"/dev/fixture","buildDirectory":"/workspace/build"
        },"program":"/tools/esptool","args":["--port","/dev/fixture","a'b c.bin"],"deviceKey":"opaque","fingerprint":"a".repeat(64)});
        let intent: FlashIntent = serde_json::from_value(value.clone()).unwrap();
        let round_trip = serde_json::to_value(&intent).unwrap();
        assert_eq!(round_trip, value);
        let parsed: crate::RunStartParams =
            serde_json::from_value(json!({"flash":intent})).unwrap();
        assert_eq!(parsed.flash.unwrap().args[2], "a'b c.bin");
        let mut unknown = value;
        unknown["selection"]["shell"] = json!("ignored?");
        assert!(serde_json::from_value::<FlashIntent>(unknown).is_err());
    }
}

/// Parameters for `runConfig.flashProposal` (`0.113.0`, E4 of
/// `integracoes/38` §6): the "Gravar" line as a run configuration PROPOSAL.
///
/// No device operation or persistence: the core may query the selected
/// esptool's version off the request loop, with a deadline (`0.186.0`).
/// Unknown or unsupported versions refuse the proposal instead of guessing.
/// The core composes the engine's command line from
/// what the project model already read (flash recipe, ELF/UF2/BIN, target)
/// and the UI shows it as a preview; saving it is `runConfig.save`, running
/// it is `run.start` — flashing is a run configuration, not a new domain.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FlashProposalParams {
    /// Request verified native intent instead of just a manual shell preview.
    #[serde(default)]
    pub native: bool,
    /// Explicit ESP-IDF build directory; native default is the framework's build/.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub build_directory: Option<String>,
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
    /// Catalogue id of an INSTALLED firmware (`toolchain.installable`,
    /// `kind: firmware`, `0.116.0`): the line writes that file at the offset
    /// its source page fixes, instead of the project's build artifacts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub firmware: Option<String>,
    /// `CMake` codemodel target whose single ELF artifact a native probe-rs
    /// intent writes (`0.188.0`); never the newest ELF of the build.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    /// Explicit probe-rs selector `vvvv:pppp[-IF]:SERIAL` for native probe-rs
    /// (`0.188.0`); a listed row index or VID/PID alone is not a destination.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub probe: Option<String>,
    /// Explicit `DfuSe` file and USB alternative for native dfu-util (`0.194.0`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dfu: Option<crate::DfuFlashSelection>,
    /// UF2 verified against target's ELF and a confirmed RP2040 (`0.197.0`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pico: Option<crate::PicoFlashSelection>,
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
    /// Present only after native provenance and snapshot validation succeeded.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flash: Option<FlashIntent>,
}

/// Reviewed native flash snapshot. The core regenerates it before execution.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FlashIntent {
    /// Intent format; only 1 is currently supported.
    pub version: u32,
    /// Canonical workspace that produced the snapshot.
    pub workspace: String,
    /// Resolved engine, device, firmware/build selection; native is true.
    pub selection: FlashProposalParams,
    /// Selected executable, never interpreted as shell text.
    pub program: String,
    /// Literal arguments from the provider plan.
    pub args: Vec<String>,
    /// Opaque observed destination identity; the engine defines its evidence.
    pub device_key: String,
    /// Opaque content/context checksum, regenerated before starting the tool.
    pub fingerprint: String,
}

//! User process payloads (`run.*`).

use serde::{Deserialize, Serialize};

/// Parameters for `run.start`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RunStartParams {
    /// Shell command to execute in the workspace root. When absent, the core
    /// derives a default from the project kind (e.g. `cargo run`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
}

/// Parameters for `run.script`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RunScriptParams {
    /// Shell script path confined to the currently opened workspace.
    pub path: String,
    /// Serial device to run a `.py` ON THE BOARD in a `MicroPython` project
    /// (`mpremote connect <device> run`); omitted = mpremote picks the first
    /// serial device it finds (`0.102.0`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device: Option<String>,
}

/// Result payload for `run.start`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunStartResult {
    /// Command that is now running.
    pub command: String,
}

/// Parameters for `run.stdin`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RunStdinParams {
    /// Raw bytes forwarded to the child stdin. The UI appends the newline.
    pub data: String,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::RunScriptParams;

    #[test]
    fn run_script_params_require_only_a_path() {
        let parsed = serde_json::from_value::<RunScriptParams>(json!({
            "path": "/workspace/scripts/check.sh"
        }))
        .unwrap();

        assert!(parsed.path.ends_with("check.sh"));
        assert!(
            serde_json::from_value::<RunScriptParams>(json!({
                "path": "check.sh",
                "command": "other"
            }))
            .is_err()
        );
    }
}

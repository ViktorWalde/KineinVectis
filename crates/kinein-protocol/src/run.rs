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
    /// Serial device for the DEFAULT launcher of a `MicroPython` project
    /// (`mpremote connect <device> run main.py`); omitted = mpremote picks the
    /// first serial device it finds. Exclusive with `command`: an explicit
    /// command is run verbatim and has no port to receive (`0.110.0`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device: Option<String>,
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

/// Result of `run.capabilities` (`0.107.0`).
///
/// What "Executar" and "Depurar" accept as a file, by extension. Static — the
/// UI asks once per connection and keeps no list of its own (the same
/// invariant as `format.capabilities`).
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunCapabilitiesResult {
    /// Extensions `run.script` accepts (`sh`, `bash`, `zsh`, `py`).
    pub runnable: Vec<String>,
    /// Extensions `debug.start { program }` routes to a language adapter
    /// without the kit (`py`).
    pub debuggable: Vec<String>,
}

/// Result payload for `run.start`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunStartResult {
    /// Command that is now running.
    pub command: String,
    /// The terminal session it runs in (`0.125.0`): the output arrives as
    /// `event.terminal.render` for this id, the end as `event.terminal.closed`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub terminal_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{RunScriptParams, RunStartParams};

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

    /// `run.start` carrega `device` desde `0.110.0`; a exclusividade com
    /// `command` e' regra do handler, nao do parser.
    #[test]
    fn run_start_params_accept_an_optional_device() {
        let parsed = serde_json::from_value::<RunStartParams>(json!({
            "device": "/dev/ttyUSB0"
        }))
        .unwrap();
        assert_eq!(parsed.command, None);
        assert_eq!(parsed.device.as_deref(), Some("/dev/ttyUSB0"));
        assert_eq!(
            serde_json::from_value::<RunStartParams>(json!({})).unwrap(),
            RunStartParams {
                command: None,
                device: None
            }
        );
        assert!(
            serde_json::from_value::<RunStartParams>(json!({ "port": "/dev/ttyUSB0" })).is_err()
        );
    }
}

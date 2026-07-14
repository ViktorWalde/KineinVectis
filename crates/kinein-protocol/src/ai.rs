//! External AI CLI Bridge payloads (`aiBridge.*`).

use serde::{Deserialize, Serialize};

/// Built-in external CLI profiles supported by the first bridge slice.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AiCliProfileId {
    /// Anthropic Claude CLI (`claude` executable).
    #[default]
    Claude,
    /// `OpenAI` Codex CLI (`codex` executable).
    Codex,
}

/// One external AI CLI option displayed by KV Context.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiCliProfile {
    /// Stable profile identifier.
    pub id: AiCliProfileId,
    /// User-facing profile name.
    pub name: String,
    /// Executable searched on `PATH`.
    pub command: String,
    /// Whether the executable is currently available.
    pub available: bool,
}

/// Result of `aiBridge.profiles`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiCliProfilesResult {
    /// Known built-in profiles and their availability.
    pub profiles: Vec<AiCliProfile>,
    /// Preferred profile resolved from settings (Claude by default).
    pub default_profile: AiCliProfileId,
}

/// Parameters for `aiBridge.terminal.open`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AiTerminalOpenParams {
    /// External CLI profile to start in a dedicated PTY.
    pub profile_id: AiCliProfileId,
}

/// Result of `aiBridge.terminal.open`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiTerminalOpenResult {
    /// Terminal session identifier used by `terminal.*` operations/events.
    pub id: String,
    /// Profile running in the PTY.
    pub profile_id: AiCliProfileId,
    /// User-facing profile name.
    pub name: String,
    /// Resolved executable path started by the core.
    pub command: String,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{AiCliProfileId, AiTerminalOpenParams};

    #[test]
    fn open_params_are_strict_and_use_stable_profile_ids() {
        let parsed = serde_json::from_value::<AiTerminalOpenParams>(json!({
            "profileId": "codex"
        }))
        .unwrap();
        assert_eq!(parsed.profile_id, AiCliProfileId::Codex);
        assert!(
            serde_json::from_value::<AiTerminalOpenParams>(json!({
                "profileId": "other"
            }))
            .is_err()
        );
    }
}

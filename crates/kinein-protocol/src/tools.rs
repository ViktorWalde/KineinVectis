//! External tool detection payloads (`tools.detect`, `tools.status`).

use serde::{Deserialize, Serialize};

/// Lifecycle status of an external tool managed by the core.
///
/// The full lifecycle is documented in `DocsPublic/07-tooling-lifecycle.md`. Tool
/// detection uses `Missing`, `Detected`, and `Failed`; the remaining states are
/// reserved for process management.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ToolStatus {
    /// The tool requires manual configuration before it can be used.
    NotConfigured,
    /// The tool was not found on the search path.
    Missing,
    /// The tool was found and responded to a version probe.
    Detected,
    /// The tool is configured and ready to run.
    Ready,
    /// The tool is currently running as a managed process.
    Running,
    /// The tool was found but did not behave as expected.
    Failed,
    /// The tool was explicitly disabled by the user.
    Disabled,
}

/// Structured status of one external tool.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolInfo {
    /// Stable tool identifier, such as `cargo` or `clangd`.
    pub id: String,
    /// Human-readable tool name.
    pub display_name: String,
    /// Current lifecycle status.
    pub status: ToolStatus,
    /// Absolute path of the detected binary.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Version string reported by the tool.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Suggested installation command for CachyOS/Arch. The core never runs
    /// this command; the UI must show it and wait for explicit confirmation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suggested_install: Option<String>,
    /// Human-readable explanation for `Missing` or `Failed` states.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Result payload for `tools.detect` and `tools.status`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolsDetectResult {
    /// Status of every tool known by the core.
    pub tools: Vec<ToolInfo>,
}

#[cfg(test)]
mod tests {
    use crate::{ToolInfo, ToolStatus};

    #[test]
    fn tool_info_serializes_camel_case_and_omits_empty_fields() {
        let missing = ToolInfo {
            id: "clangd".to_owned(),
            display_name: "clangd".to_owned(),
            status: ToolStatus::Missing,
            path: None,
            version: None,
            suggested_install: Some("sudo pacman -S clang".to_owned()),
            message: Some("clangd nao foi encontrado no PATH".to_owned()),
        };
        let value = serde_json::to_value(missing).unwrap();

        assert_eq!(value["status"], "missing");
        assert_eq!(value["displayName"], "clangd");
        assert_eq!(value["suggestedInstall"], "sudo pacman -S clang");
        assert!(value.get("path").is_none());
        assert!(value.get("version").is_none());
    }
}

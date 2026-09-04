//! Types for the `probe.*` domain: connected debug probes.

use serde::{Deserialize, Serialize};

/// One probe seen on this machine.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProbeInfo {
    /// Human-readable name reported by the tool.
    pub name: String,
    /// USB vendor id, as reported (hex, no prefix).
    pub vid: String,
    /// USB product id, as reported.
    pub pid: String,
    /// Serial, when the tool reports one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub serial: Option<String>,
    /// Probe family (CMSIS-DAP, ST-LINK, ...), when reported.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
}

/// Result payload for `probe.list`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProbeListResult {
    /// Probes the parser recognised.
    pub probes: Vec<ProbeInfo>,
    /// Whether the tool could be executed at all.
    pub tool_available: bool,
    /// The tool's raw output.
    ///
    /// Always sent, never only on success: when nothing was recognised, this is
    /// what lets the user see whether a probe is actually there and the format
    /// simply changed. "I did not understand the answer, and here it is" is
    /// actionable; "no probes" when one is plugged in is a lie.
    pub raw_output: String,
    /// What to do about it, when there is nothing to show.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

/// Parameters for `probe.list`.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProbeListParams {}

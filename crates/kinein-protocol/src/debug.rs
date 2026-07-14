//! Debug session payloads (`debug.*`, DAP orchestration).

use serde::{Deserialize, Serialize};

/// Parameters for `debug.start`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DebugStartParams {
    /// Explicit executable to debug; absent resolves the workspace default
    /// (single cargo/cmake binary, mirroring the run heuristic).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub program: Option<String>,
}

/// Result payload for `debug.start`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DebugStartResult {
    /// Absolute path of the executable handed to the debug adapter.
    pub program: String,
}

/// Parameters for `debug.setBreakpoints` (full set per file; empty clears).
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DebugSetBreakpointsParams {
    /// Absolute file path inside the workspace.
    pub file: String,
    /// Every 1-based breakpoint line of the file (replaces the previous set).
    pub lines: Vec<u32>,
}

/// One breakpoint as acknowledged by the core/adapter.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BreakpointInfo {
    /// 1-based line of the breakpoint.
    pub line: u32,
    /// `true` once the live adapter bound the breakpoint to real code.
    pub verified: bool,
}

/// Result payload for `debug.setBreakpoints`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DebugBreakpointsResult {
    /// Breakpoints of the file after the update, sorted by line.
    pub breakpoints: Vec<BreakpointInfo>,
}

/// One stack frame of the paused thread.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StackFrameInfo {
    /// Adapter frame id, used by `debug.variables { frameId }`.
    pub id: i64,
    /// Function/frame display name.
    pub name: String,
    /// Source file of the frame, when the adapter knows it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    /// 1-based line of the frame, when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
}

/// Result payload for `debug.stackTrace`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DebugStackTraceResult {
    /// Frames of the paused thread, top first (capped by the core).
    pub frames: Vec<StackFrameInfo>,
}

/// Parameters for `debug.variables` (exactly one of the two fields).
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DebugVariablesParams {
    /// Frame to inspect (the core resolves the locals scope internally).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frame_id: Option<i64>,
    /// Expansion handle of a structured variable (`ref` from a previous
    /// response).
    #[serde(default, rename = "ref", skip_serializing_if = "Option::is_none")]
    pub reference: Option<i64>,
}

/// One variable (or struct field) of the debuggee.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VariableInfo {
    /// Variable name as the adapter presents it.
    pub name: String,
    /// Rendered value.
    pub value: String,
    /// Type name, when the adapter provides one.
    #[serde(default, rename = "type", skip_serializing_if = "Option::is_none")]
    pub type_name: Option<String>,
    /// Expansion handle: `> 0` means the variable has children.
    #[serde(rename = "ref")]
    pub reference: i64,
}

/// Result payload for `debug.variables`, echoing the request key so the
/// UI can correlate the answer with the tree node that asked.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DebugVariablesResult {
    /// Echo of `frameId`, when the request inspected a frame.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frame_id: Option<i64>,
    /// Echo of `ref`, when the request expanded a variable.
    #[serde(default, rename = "ref", skip_serializing_if = "Option::is_none")]
    pub reference: Option<i64>,
    /// Variables of the frame scope or of the expanded variable.
    pub variables: Vec<VariableInfo>,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{
        BreakpointInfo, DebugBreakpointsResult, DebugSetBreakpointsParams, DebugStartParams,
    };

    #[test]
    fn start_params_accept_missing_program_and_reject_unknown_fields() {
        let empty: DebugStartParams = serde_json::from_value(json!({})).unwrap();
        assert_eq!(empty.program, None);

        let rejected = serde_json::from_value::<DebugStartParams>(json!({ "cmd": "x" }));
        assert!(rejected.is_err());
    }

    #[test]
    fn variables_params_use_ref_wire_name_and_echo_round_trips() {
        let params: super::DebugVariablesParams =
            serde_json::from_value(json!({ "ref": 12 })).unwrap();
        assert_eq!(params.reference, Some(12));
        assert_eq!(params.frame_id, None);

        let value = serde_json::to_value(super::DebugVariablesResult {
            frame_id: Some(3),
            reference: None,
            variables: vec![super::VariableInfo {
                name: "p".to_owned(),
                value: "{...}".to_owned(),
                type_name: Some("point".to_owned()),
                reference: 7,
            }],
        })
        .unwrap();
        assert_eq!(value["frameId"], 3);
        assert_eq!(value["variables"][0]["ref"], 7);
        assert_eq!(value["variables"][0]["type"], "point");
    }

    #[test]
    fn breakpoints_roundtrip_uses_camel_case() {
        let params: DebugSetBreakpointsParams =
            serde_json::from_value(json!({ "file": "/w/main.c", "lines": [3, 7] })).unwrap();
        assert_eq!(params.lines, vec![3, 7]);

        let value = serde_json::to_value(DebugBreakpointsResult {
            breakpoints: vec![BreakpointInfo {
                line: 3,
                verified: true,
            }],
        })
        .unwrap();
        assert_eq!(value["breakpoints"][0]["verified"], true);
    }
}

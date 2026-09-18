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
    /// Attach to an existing debugpy adapter; mutually exclusive with program.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub connect: Option<DebugConnectParams>,
}

/// TCP endpoint already listening for a debugpy DAP client.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DebugConnectParams {
    /// Hostname or IP address (without a URL scheme).
    pub host: String,
    /// TCP port, validated as nonzero by the core.
    pub port: u16,
}

/// Result payload for `debug.start`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DebugStartResult {
    /// Display target: executable, Python module, or debugpy endpoint.
    pub program: String,
    /// The process belongs to the caller; stopping detaches without killing it.
    #[serde(default)]
    pub attached: bool,
}

/// Parameters for `debug.setBreakpoints` (full set per file; empty clears).
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DebugSetBreakpointsParams {
    /// Absolute file path inside the workspace.
    pub file: String,
    /// Every breakpoint of the file (replaces the previous set).
    pub breakpoints: Vec<SourceBreakpointParams>,
}

/// One breakpoint requested by the UI, with its optional stop conditions.
///
/// `condition` and `hit_condition` map 1:1 onto the DAP `SourceBreakpoint`
/// fields of the same name (DAP specification, `setBreakpoints` request):
/// *"Expression for conditional breakpoints; the breakpoint stops only when
/// this evaluates to true"* and *"How many times the breakpoint must be hit
/// before stopping"*. The adapter in use is `lldb-dap`, which advertises both
/// `supportsConditionalBreakpoints` and `supportsHitConditionalBreakpoints`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SourceBreakpointParams {
    /// 1-based line of the breakpoint.
    pub line: u32,
    /// Stops only when this expression evaluates true. `None` = always stops.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
    /// Stops only after this many hits. `None` = stops on every hit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hit_condition: Option<String>,
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
        let params: DebugSetBreakpointsParams = serde_json::from_value(json!({
            "file": "/w/main.c",
            "breakpoints": [{ "line": 3 }, { "line": 7, "condition": "i == 42" }],
        }))
        .unwrap();
        assert_eq!(
            params
                .breakpoints
                .iter()
                .map(|bp| bp.line)
                .collect::<Vec<_>>(),
            vec![3, 7]
        );
        assert_eq!(params.breakpoints[0].condition, None);
        assert_eq!(params.breakpoints[1].condition.as_deref(), Some("i == 42"));

        // `condition`/`hitCondition` ausentes NAO viram `null` no wire: um
        // adapter pode tratar null como expressao vazia (DAP SourceBreakpoint).
        let bare = serde_json::to_value(&params.breakpoints[0]).unwrap();
        assert!(
            bare.get("condition").is_none(),
            "condition nula vazou: {bare}"
        );
        assert!(bare.get("hitCondition").is_none());

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

/// Params for `debug.evaluate` — a watch expression on the paused frame.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DebugEvaluateParams {
    /// Expression to evaluate in the debuggee.
    pub expression: String,
    /// Frame to evaluate in; `None` uses the top frame of the stopped thread.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frame_id: Option<i64>,
}

/// Result payload for `debug.evaluate`.
///
/// Mirrors the DAP `evaluate` response body: `result` is required, `type` is
/// optional, and `variables_reference` is `> 0` when the value can be expanded
/// — the same contract `debug.variables` already uses for `reference`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DebugEvaluateResult {
    /// The expression that was evaluated, echoed back for the UI to match.
    pub expression: String,
    /// Rendered value.
    pub result: String,
    /// Declared type, when the adapter reports one.
    pub type_name: Option<String>,
    /// `> 0` when the value has children; feed it to `debug.variables`.
    pub reference: i64,
}

/// Parameters for `debug.scopes` (`0.118.0`, P3).
///
/// The scopes of a frame — `Locals`, `Registers`, and, with an SVD in the
/// kit, the peripherals the probe-rs exposes. `debug.variables { ref }`
/// expands each one.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DebugScopesParams {
    /// Frame whose scopes are asked.
    pub frame_id: i64,
}

/// One scope, as the adapter names it.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DebugScopeInfo {
    /// `Locals`, `Registers`, `Peripherals`… (the adapter's word).
    pub name: String,
    /// Expansion handle for `debug.variables { ref }`.
    #[serde(rename = "ref")]
    pub reference: i64,
    /// The adapter says fetching it is costly (registers of every peripheral).
    #[serde(default)]
    pub expensive: bool,
}

/// Result of `debug.scopes`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DebugScopesResult {
    /// The frame asked.
    pub frame_id: i64,
    /// Scopes in the adapter's order.
    pub scopes: Vec<DebugScopeInfo>,
}

/// Parameters for `debug.readMemory` (`0.118.0`, P3).
///
/// The DAP `readMemory` request, verbatim — `memoryReference` is an address
/// (`0x3ff00000`) or the reference a variable carries.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DebugReadMemoryParams {
    /// Address or memory reference.
    pub memory_reference: String,
    /// Bytes to skip from the reference.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,
    /// Bytes to read.
    pub count: u64,
}

/// Result of `debug.readMemory`: the DAP body, verbatim.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DebugReadMemoryResult {
    /// The address the data starts at (hex).
    pub address: String,
    /// Bytes the target could not read at the end.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unreadable_bytes: Option<u64>,
    /// The bytes, base64 as the DAP sends them.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
}

/// Parameters for `debug.disassemble` (`0.118.0`, P3): the DAP `disassemble`
/// request, verbatim.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DebugDisassembleParams {
    /// Address or memory reference (a frame's `instructionPointerReference`).
    pub memory_reference: String,
    /// Bytes to skip from the reference.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,
    /// Instructions to skip (negative = before the reference).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instruction_offset: Option<i64>,
    /// Instructions to return.
    pub instruction_count: u64,
}

/// One disassembled instruction.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DebugInstruction {
    /// Address (hex).
    pub address: String,
    /// The instruction text.
    pub instruction: String,
    /// Raw bytes (hex), when the adapter gives them.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instruction_bytes: Option<String>,
    /// Symbol the address belongs to, when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    /// Source file, when the adapter maps it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    /// Source line, when mapped.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<i64>,
}

/// Result of `debug.disassemble`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DebugDisassembleResult {
    /// Instructions in address order.
    pub instructions: Vec<DebugInstruction>,
}

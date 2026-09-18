//! Types for the `coverage.*` domain (`0.119.0`, D8 of `roadmaps/41`): line
//! coverage of the project's tests, as LCOV — the one format `cargo llvm-cov`
//! and `coverage.py` both write, and the one the editor gutters read.

use serde::{Deserialize, Serialize};

/// Parameters for `coverage.run`: no fields (the project kind picks the
/// tool).
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoverageRunParams {}

/// One source file in the LCOV report.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoverageFileSummary {
    /// Absolute path as the tool wrote it (`SF:`).
    pub file: String,
    /// Instrumented lines (`LF:` or the `DA:` count).
    pub lines_found: u64,
    /// Lines hit at least once (`LH:` or the `DA:` count with hits > 0).
    pub lines_hit: u64,
}

/// Payload of `event.coverage.finished`: the job ended.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoverageFinishedEvent {
    /// Job id.
    pub job_id: String,
    /// The tool exited 0 and the LCOV was read.
    pub success: bool,
    /// `cargo llvm-cov` | `coverage.py`.
    pub tool: String,
    /// The LCOV file written (`<root>/.kinein/coverage.lcov`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Per-file totals, in the report's order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub files: Vec<CoverageFileSummary>,
    /// Why it failed, in words.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Parameters for `coverage.lines`: the lines of one file from the last
/// report.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CoverageLinesParams {
    /// Absolute path of the source file.
    pub file: String,
}

/// Result of `coverage.lines`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoverageLinesResult {
    /// The file asked.
    pub file: String,
    /// `true` when the last report has this file.
    pub known: bool,
    /// Lines hit (1-based).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub covered: Vec<u32>,
    /// Instrumented lines never hit (1-based).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub missed: Vec<u32>,
}

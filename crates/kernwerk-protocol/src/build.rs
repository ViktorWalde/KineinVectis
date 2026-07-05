//! Build, quality and test runner payloads (`build.run`, `quality.run`, `test.run`).

use serde::{Deserialize, Serialize};

/// Severity of a structured build diagnostic.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BuildDiagnosticSeverity {
    /// Compilation error.
    Error,
    /// Compiler warning.
    Warning,
    /// Informational note attached to another diagnostic.
    Note,
}

/// Structured diagnostic extracted from build output.
///
/// Streamed as `event.build.diagnostic` notifications while a build runs.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildDiagnostic {
    /// Diagnostic severity.
    pub severity: BuildDiagnosticSeverity,
    /// Human-readable message.
    pub message: String,
    /// Source file, relative to the workspace root when possible.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    /// One-based line number.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<u64>,
    /// One-based column number.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub column: Option<u64>,
}

/// Result payload for `build.run`, sent after `event.build.finished`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildRunResult {
    /// Whether the build finished successfully.
    pub success: bool,
    /// Exit code of the build tool, when it exited normally.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    /// Number of structured diagnostics emitted during the build.
    pub diagnostics: u64,
}

/// Result payload for `quality.run`, sent after `event.quality.finished`.
///
/// Mirrors `BuildRunResult`: quality analysis reuses the same structured
/// diagnostics pipeline as the build, only with a linter as the tool.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QualityRunResult {
    /// Whether the linter finished without erroring out.
    pub success: bool,
    /// Exit code of the linter, when it exited normally.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    /// Number of structured diagnostics emitted.
    pub diagnostics: u64,
}

/// Result payload for `test.run`, sent after `event.test.finished`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestRunResult {
    /// Whether the runner exited successfully (no failing test).
    pub success: bool,
    /// Exit code of the test runner, when it exited normally.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    /// Number of passed test cases.
    pub passed: u64,
    /// Number of failed test cases.
    pub failed: u64,
    /// Number of ignored test cases.
    pub ignored: u64,
}

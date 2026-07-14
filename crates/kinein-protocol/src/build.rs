//! Build, quality and test runner payloads (`build.run`, `quality.run`, `test.run`).

use serde::{Deserialize, Serialize};

use crate::{Diagnostic, DiagnosticSource};

/// Backwards-compatible name for diagnostic severity used by build payloads.
pub use crate::DiagnosticSeverity as BuildDiagnosticSeverity;

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

impl BuildDiagnostic {
    /// Converts this tool-specific diagnostic into the common Problems model.
    #[must_use]
    pub fn to_diagnostic(&self, source: DiagnosticSource, job_id: Option<String>) -> Diagnostic {
        Diagnostic {
            id: None,
            source,
            severity: self.severity,
            category: Some(category_for_source(source).to_owned()),
            message: self.message.clone(),
            file: self.file.clone(),
            line: self.line,
            column: self.column,
            end_line: None,
            end_column: None,
            code: None,
            job_id,
            command: None,
            target: None,
            log_ref: None,
        }
    }
}

const fn category_for_source(source: DiagnosticSource) -> &'static str {
    match source {
        DiagnosticSource::Build => "compiler",
        DiagnosticSource::Quality => "lint",
        DiagnosticSource::Lsp => "lsp",
        DiagnosticSource::Toolchain => "toolchain",
    }
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

#[cfg(test)]
mod tests {
    use crate::{BuildDiagnostic, BuildDiagnosticSeverity, DiagnosticSource};

    #[test]
    fn build_diagnostic_converts_to_common_diagnostic() {
        let build = BuildDiagnostic {
            severity: BuildDiagnosticSeverity::Warning,
            message: "unused variable".to_owned(),
            file: Some("src/main.rs".to_owned()),
            line: Some(12),
            column: Some(8),
        };

        let diagnostic = build.to_diagnostic(DiagnosticSource::Quality, Some("job_7".to_owned()));

        assert_eq!(diagnostic.source, DiagnosticSource::Quality);
        assert_eq!(diagnostic.severity, BuildDiagnosticSeverity::Warning);
        assert_eq!(diagnostic.category.as_deref(), Some("lint"));
        assert_eq!(diagnostic.message, "unused variable");
        assert_eq!(diagnostic.file.as_deref(), Some("src/main.rs"));
        assert_eq!(diagnostic.line, Some(12));
        assert_eq!(diagnostic.column, Some(8));
        assert_eq!(diagnostic.job_id.as_deref(), Some("job_7"));
    }
}

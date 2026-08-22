//! Build, quality and test runner payloads (`build.run`, `quality.run`, `test.run`).

use serde::{Deserialize, Serialize};

use crate::{BuildSystem, Diagnostic, DiagnosticSource};

/// Backwards-compatible name for diagnostic severity used by build payloads.
pub use crate::DiagnosticSeverity as BuildDiagnosticSeverity;

/// Parameters for `build.run`.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BuildRunParams {
    /// Explicit build system in a hybrid workspace; primary kind when absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub build_system: Option<BuildSystem>,
    /// Alvo especifico a compilar (`CMake`); ausente compila tudo.
    ///
    /// So o caminho `CMake` usa: e o `cmake --build --target`. Compilar um
    /// alvo de cada vez e o que torna o seletor da barra util em vez de
    /// decorativo — sem isto ele mostraria uma escolha que nao muda nada.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
}

/// Parameters for `quality.run`.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct QualityRunParams {
    /// Explicit analyzer toolchain. Only Cargo is implemented currently.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub build_system: Option<BuildSystem>,
}

/// Parameters for `test.run`.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TestRunParams {
    /// Optional runner-specific test filter.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filter: Option<String>,
    /// Explicit build system in a hybrid workspace; primary kind when absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub build_system: Option<BuildSystem>,
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
        DiagnosticSource::Audit => "security",
        DiagnosticSource::Memcheck => "memory",
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
    use serde_json::json;

    use crate::{
        BuildDiagnostic, BuildDiagnosticSeverity, BuildRunParams, BuildSystem, DiagnosticSource,
        TestRunParams,
    };

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

    #[test]
    fn hybrid_runner_params_are_strict_and_backwards_compatible() {
        let default_build = serde_json::from_value::<BuildRunParams>(json!({})).unwrap();
        let cmake_tests = serde_json::from_value::<TestRunParams>(json!({
            "filter": "smoke",
            "buildSystem": "cmake"
        }))
        .unwrap();

        assert_eq!(default_build.build_system, None);
        assert_eq!(cmake_tests.build_system, Some(BuildSystem::Cmake));
        assert_eq!(cmake_tests.filter.as_deref(), Some("smoke"));
        assert!(serde_json::from_value::<BuildRunParams>(json!({ "system": "cargo" })).is_err());
    }
}

//! Shared diagnostics model for Problems-like surfaces.
//!
//! Domain events (`event.build.diagnostic`, `event.quality.diagnostic` and
//! `event.lsp.diagnostics`) still keep their existing names, but their payloads
//! can use this common shape so the UI does not need one model per producer.

use serde::{Deserialize, Serialize};

/// Origin that produced a diagnostic.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DiagnosticSource {
    /// Build pipeline (`build.run`).
    Build,
    /// Quality/lint pipeline (`quality.run`).
    Quality,
    /// Language server diagnostics.
    Lsp,
    /// Toolchain/environment checks.
    Toolchain,
}

/// Severity of a structured diagnostic.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DiagnosticSeverity {
    /// Error that should usually block the current operation.
    Error,
    /// Warning that should be visible but may not block.
    Warning,
    /// Informational note attached to a diagnostic or produced by a tool.
    Note,
}

/// Common diagnostic payload emitted by the core.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostic {
    /// Stable id, once a producer can provide one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Producer of the diagnostic.
    pub source: DiagnosticSource,
    /// Severity normalized by the core.
    pub severity: DiagnosticSeverity,
    /// Optional machine-readable category (`compiler`, `lint`, `lsp`, ...).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    /// Human-readable message.
    pub message: String,
    /// Source file, relative to the workspace root when possible.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    /// One-based line number (start of the range).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<u64>,
    /// One-based column number (start of the range).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub column: Option<u64>,
    /// One-based line number of the range end (for editor underlines).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_line: Option<u64>,
    /// One-based column number of the range end (for editor underlines).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_column: Option<u64>,
    /// Machine code/rule of the diagnostic (`E0425`, a lint name, ...).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    /// Job that produced the diagnostic, for job-backed operations.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub job_id: Option<String>,
    /// Command associated with the diagnostic, when useful for UI actions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    /// Target/profile associated with the diagnostic.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    /// Future reference to retained logs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub log_ref: Option<String>,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::{Diagnostic, DiagnosticSeverity, DiagnosticSource};

    #[test]
    fn diagnostic_serializes_camel_case_and_omits_absent_fields() {
        let diagnostic = Diagnostic {
            id: None,
            source: DiagnosticSource::Build,
            severity: DiagnosticSeverity::Error,
            category: Some("compiler".to_owned()),
            message: "expected `;`".to_owned(),
            file: Some("src/main.rs".to_owned()),
            line: Some(7),
            column: Some(12),
            end_line: Some(7),
            end_column: Some(13),
            code: Some("E0425".to_owned()),
            job_id: Some("job_1".to_owned()),
            command: None,
            target: None,
            log_ref: None,
        };

        let value = serde_json::to_value(diagnostic).unwrap();

        assert_eq!(value["source"], "build");
        assert_eq!(value["severity"], "error");
        assert_eq!(value["jobId"], "job_1");
        assert_eq!(value["category"], "compiler");
        assert_eq!(value["endLine"], 7);
        assert_eq!(value["endColumn"], 13);
        assert_eq!(value["code"], "E0425");
        assert!(value.get("command").is_none());
    }

    #[test]
    fn diagnostic_sources_match_protocol_strings() {
        assert_eq!(
            serde_json::to_value(DiagnosticSource::Build).unwrap(),
            json!("build")
        );
        assert_eq!(
            serde_json::to_value(DiagnosticSource::Quality).unwrap(),
            json!("quality")
        );
        assert_eq!(
            serde_json::to_value(DiagnosticSource::Lsp).unwrap(),
            json!("lsp")
        );
        assert_eq!(
            serde_json::to_value(DiagnosticSource::Toolchain).unwrap(),
            json!("toolchain")
        );
    }
}

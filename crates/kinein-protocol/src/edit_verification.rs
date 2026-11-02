//! Evidence from a bounded compiler comparison of one proposed LSP edit.

use serde::{Deserialize, Serialize};

use crate::Diagnostic;

/// Observation, never a guarantee of equivalence or build success.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EditVerificationStatus {
    /// Additional compiler error observed after the edit.
    NewError,
    /// No additional compiler error observed in this comparison.
    NoNewError,
    /// Evidence unavailable, incomplete or obsolete.
    NotVerified,
}

/// Compiler evidence attached to a confirmable workspace-edit preview.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditVerification {
    /// Result of the observation.
    pub status: EditVerificationStatus,
    /// Scope or reason why the edit could not be verified.
    pub reason: String,
    /// Machine-readable recovery action, when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason_code: Option<String>,
    /// The preview identity was invalidated and cannot be applied.
    #[serde(default)]
    pub obsolete: bool,
    /// Total elapsed time including preparation and process cleanup.
    pub duration_ms: u64,
    /// LSP document identity at the time of the comparison.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_version: Option<i64>,
    /// Digest of the CDB and effective command.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_id: Option<String>,
    /// Effective compiler, without substitution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compiler: Option<String>,
    /// Effective syntax-only arguments; transient overlay path omitted.
    pub arguments: Vec<String>,
    /// Original compilation working directory.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub directory: Option<String>,
    /// Number of observed errors before the edit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before_errors: Option<u64>,
    /// Number of observed errors after the edit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after_errors: Option<u64>,
    /// Additional errors, capped at 32 in the preview.
    pub new_errors: Vec<Diagnostic>,
}

impl EditVerification {
    /// An explicit absence of usable compiler evidence.
    pub fn not_verified(reason: impl Into<String>) -> Self {
        Self {
            status: EditVerificationStatus::NotVerified,
            reason: reason.into(),
            reason_code: None,
            obsolete: false,
            duration_ms: 0,
            document_version: None,
            context_id: None,
            compiler: None,
            arguments: Vec::new(),
            directory: None,
            before_errors: None,
            after_errors: None,
            new_errors: Vec::new(),
        }
    }
}

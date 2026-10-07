//! Numeric external errors; free-form text is never public diagnostic data.

use serde::{Deserialize, Serialize};

/// Stable application reason carried in the external error's `data`.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FailureReason {
    /// There is no understood API intersection.
    IncompatibleApi,
    /// The selected adapter cannot implement the requested operation.
    UnsupportedOperation,
    /// The database requires a credential not supplied for this session.
    SecretRequired,
    /// The effective policy forbids the write.
    ReadOnly,
    /// Instance, session or operation generation is stale.
    ContextChanged,
    /// Finite queue or resource capacity is exhausted.
    Busy,
    /// Cancellation reached a terminal result.
    Cancelled,
    /// The operation exceeded its deadline.
    Timeout,
    /// Opening or maintaining the database connection failed.
    ConnectionFailed,
    /// The database confirmed an execution failure.
    ExecutionFailed,
    /// A negotiated budget would be exceeded.
    LimitExceeded,
    /// A submitted write or commit has no established result.
    OutcomeUnknown,
}

/// Known outcome of an unsuccessful operation; never a retry permission.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OperationOutcome {
    /// This operation did not start executing on the database.
    NotStarted,
    /// A failure was confirmed; earlier batch commands may have applied.
    Failed,
    /// The database result could not be established.
    Unknown,
}

/// Allowlisted machine-readable information, with no free-form strings.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(remote = "Self")]
pub struct Failure {
    /// Stable public classification.
    pub reason: FailureReason,
    /// Established result, or explicit uncertainty.
    pub outcome: OperationOutcome,
}

/// JSON-RPC numeric error for the private adapter pipe.
#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(remote = "Self")]
pub struct Error {
    /// Standard JSON-RPC code, or -32000 for the typed application error.
    pub code: i32,
    /// Untrusted adapter text; not forwarded to UI or logs.
    pub message: String,
    /// Known application classification, absent for standard protocol errors.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<Failure>,
}

super::object::object_serde!(Failure, Error);

impl std::fmt::Debug for Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Error")
            .field("code", &self.code)
            .field("message", &"[untrusted text omitted]")
            .field("data", &self.data)
            .finish()
    }
}

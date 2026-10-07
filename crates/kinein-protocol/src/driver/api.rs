//! Initialization metadata and budgets; no connection or credential fields.

use serde::{Deserialize, Serialize};

/// Adapter API version, independent of binary, profile and UI IPC versions.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(remote = "Self", deny_unknown_fields)]
pub struct Version {
    /// Incompatible contract generation.
    pub major: u16,
    /// Additive contract revision.
    pub minor: u16,
}

/// Closed range within a single major; ordering is checked by the consumer.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(remote = "Self", deny_unknown_fields)]
pub struct ApiRange {
    /// Lowest understood API.
    pub min: Version,
    /// Highest understood API.
    pub max: Version,
}

/// Budgets advertised before any credentials or database effects.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(remote = "Self", rename_all = "camelCase", deny_unknown_fields)]
pub struct Limits {
    /// Serialized UTF-8 bytes of one message, excluding the line terminator.
    pub message_bytes: u32,
    /// Requests still awaiting a terminal response.
    pub in_flight: u16,
    /// Rows retained for one operation.
    pub rows: u32,
    /// Cells in one row.
    pub columns: u16,
    /// UTF-8 bytes of one complete cell.
    pub cell_bytes: u32,
    /// Total bytes retained for one result or catalogue.
    pub retained_bytes: u32,
    /// Total catalogue objects, columns and document fields retained.
    pub catalogue_items: u32,
}

/// Operations whose semantics are known to this API generation.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Operation {
    /// Open a validated destination after core authorization.
    Open,
    /// Test an opened destination.
    Test,
    /// Read its catalogue.
    Introspect,
    /// Run authorized text with bounded results.
    Query,
    /// Measure impact without granting write authority.
    Impact,
    /// Hold a write on its original connection and transaction.
    Preview,
    /// Decide that held transaction once.
    Decide,
    /// Request cancellation of an existing operation.
    Cancel,
    /// Close a session after its accepted operations have settled.
    Close,
    /// Shut down the adapter and its owned resources.
    Shutdown,
}

/// Strict parameters of the first request, `driver.initialize`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(remote = "Self", deny_unknown_fields)]
pub struct InitializeParams {
    /// API range understood by the caller.
    pub api: ApiRange,
    /// Caller budgets; the negotiated values cannot exceed these.
    pub limits: Limits,
}

/// Initialization response; additive top-level fields can be ignored.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(remote = "Self", rename_all = "camelCase")]
pub struct InitializeResult {
    /// API range implemented by the adapter.
    pub api: ApiRange,
    /// Implementation identity, distinct from engine and installation.
    pub adapter_id: String,
    /// Adapter binary version; informational, never compatibility proof.
    pub adapter_version: String,
    /// Maintained driver version, when it can be reported without opening.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub driver_version: Option<String>,
    /// Supported engine identities; none is selected implicitly.
    pub engines: Vec<String>,
    /// Implemented operations, independent of effective server permissions.
    pub operations: Vec<Operation>,
    /// Adapter budgets, intersected with the caller's local ceilings.
    pub limits: Limits,
}

super::object::object_serde!(
    Version,
    ApiRange,
    Limits,
    InitializeParams,
    InitializeResult
);

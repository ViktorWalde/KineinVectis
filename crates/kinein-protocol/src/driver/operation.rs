//! Operational adapter messages; these types never execute or persist anything.

use std::{collections::BTreeMap, fmt};

use serde::{Deserialize, Serialize};

use super::{Limits, Operation};
use crate::{
    DataSourceCatalogUpdate, DataSourcePreviewDecision, DataSourcePreviewOutcome, DataSourceSchema,
    MongoCollection, SqlImpactSeverity, SqlStatementImpact,
};

/// Exact operation destination, generated and checked by the core.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(remote = "Self", rename_all = "camelCase", deny_unknown_fields)]
pub struct Context {
    /// Opaque workspace/profile/resolved-installation identity, never an engine key.
    pub instance_id: String,
    /// Opened session; absent only while opening or shutting down the instance.
    pub session_id: Option<String>,
    /// Full configuration/credential generation.
    pub generation: u64,
    /// Correlation with the core's existing operation/job, not a new job system.
    pub operation_id: String,
}

/// Transient credential; deliberately excludes its value from diagnostics.
#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Credential(
    /// Pipe-only value; callers must never persist or log it.
    pub String,
);

impl fmt::Debug for Credential {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Credential([REDACTED])")
    }
}

/// Public scalar option; an adapter schema still validates allowed keys and values.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OptionValue {
    /// Public text, never credentials or executable code.
    Text(String),
    /// Public switch.
    Boolean(bool),
    /// Public integral setting.
    Integer(i64),
}

/// Versioned public options, kept separate from transient credentials.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(remote = "Self", rename_all = "camelCase", deny_unknown_fields)]
pub struct PublicOptions {
    /// Adapter-owned option schema, independent of the API and profile file.
    pub schema_version: u32,
    /// Typed public fields; deserialization alone does not authorize opening.
    #[serde(deserialize_with = "crate::driver::deserialize_unique_map")]
    pub fields: BTreeMap<String, OptionValue>,
}

/// Effective restrictions, after core policy and API negotiation.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(remote = "Self", rename_all = "camelCase", deny_unknown_fields)]
pub struct Restrictions {
    /// Refuse writes and unknown operations.
    pub read_only: bool,
    /// Never greater than the core's negotiated budget.
    pub limits: Limits,
}

/// Opening is the only operational request that can carry a credential.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(remote = "Self", rename_all = "camelCase", deny_unknown_fields)]
pub struct OpenParams {
    /// Instance/operation context, without a session yet.
    pub context: Context,
    /// Previously validated public engine identity.
    pub engine: String,
    /// Public, schema-validated configuration.
    pub options: PublicOptions,
    /// Effective core policy and limits.
    pub restrictions: Restrictions,
    /// Pipe-only transient secret, never a saved option.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub credential: Option<Credential>,
}

/// Session-only test, introspection or close parameters.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(remote = "Self", rename_all = "camelCase", deny_unknown_fields)]
pub struct SessionParams {
    /// Exact existing session and operation.
    pub context: Context,
}

/// Query or preview text authorized by the core.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(remote = "Self", rename_all = "camelCase", deny_unknown_fields)]
pub struct StatementParams {
    /// Exact destination and operation.
    pub context: Context,
    /// SQL or existing Mongo console command, never a callback to the core.
    pub text: String,
    /// Effective row ceiling, no greater than negotiated limits.
    pub max_rows: u32,
}

/// Impact measurement does not authorize the later execution.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(remote = "Self", rename_all = "camelCase", deny_unknown_fields)]
pub struct ImpactParams {
    /// Exact destination and measurement operation.
    pub context: Context,
    /// Existing authorized measurement text.
    pub text: String,
}

/// Decision targets the same session and generation as the ready preview.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(remote = "Self", rename_all = "camelCase", deny_unknown_fields)]
pub struct DecideParams {
    /// Context of this decision request, distinct from the preview request.
    pub context: Context,
    /// Original preview operation.
    pub preview_operation_id: String,
    /// One-use ready identity.
    pub preview_id: String,
    /// Existing explicit transaction decision.
    pub decision: DataSourcePreviewDecision,
}

/// Cancellation has its own reply; the target must still terminate.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(remote = "Self", rename_all = "camelCase", deny_unknown_fields)]
pub struct CancelParams {
    /// Context of this cancellation request.
    pub context: Context,
    /// Existing operation in the same instance/session/generation.
    pub target_operation_id: String,
}

/// Typed method/params portion inside the existing JSON-RPC request envelope.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    remote = "Self",
    tag = "method",
    content = "params",
    deny_unknown_fields
)]
pub enum Request {
    /// Open only after policy and negotiation.
    #[serde(rename = "driver.open")]
    Open(OpenParams),
    /// Probe an opened session.
    #[serde(rename = "driver.test")]
    Test(SessionParams),
    /// Read catalogue chunks.
    #[serde(rename = "driver.introspect")]
    Introspect(SessionParams),
    /// Execute once, within the effective restrictions.
    #[serde(rename = "driver.query")]
    Query(StatementParams),
    /// Measure using the existing impact semantics.
    #[serde(rename = "driver.impact")]
    Impact(ImpactParams),
    /// Execute in an adapter-owned transaction awaiting a decision.
    #[serde(rename = "driver.preview")]
    Preview(StatementParams),
    /// Request a one-use decision; its acceptance is not a commit outcome.
    #[serde(rename = "driver.decide")]
    Decide(DecideParams),
    /// Request cancellation without replacing the target's terminal.
    #[serde(rename = "driver.cancel")]
    Cancel(CancelParams),
    /// Release session resources.
    #[serde(rename = "driver.close")]
    Close(SessionParams),
    /// Release instance resources; session context must be absent.
    #[serde(rename = "driver.shutdown")]
    Shutdown(SessionParams),
}

impl Request {
    /// Known operation semantics of this strictly typed request.
    #[must_use]
    pub const fn operation(&self) -> Operation {
        match self {
            Self::Open(_) => Operation::Open,
            Self::Test(_) => Operation::Test,
            Self::Introspect(_) => Operation::Introspect,
            Self::Query(_) => Operation::Query,
            Self::Impact(_) => Operation::Impact,
            Self::Preview(_) => Operation::Preview,
            Self::Decide(_) => Operation::Decide,
            Self::Cancel(_) => Operation::Cancel,
            Self::Close(_) => Operation::Close,
            Self::Shutdown(_) => Operation::Shutdown,
        }
    }

    /// Exact originating context; contains no credential or SQL text.
    #[must_use]
    pub const fn context(&self) -> &Context {
        match self {
            Self::Open(params) => &params.context,
            Self::Test(params)
            | Self::Introspect(params)
            | Self::Close(params)
            | Self::Shutdown(params) => &params.context,
            Self::Query(params) | Self::Preview(params) => &params.context,
            Self::Impact(params) => &params.context,
            Self::Decide(params) => &params.context,
            Self::Cancel(params) => &params.context,
        }
    }
}

/// Params of the `driver.chunk` notification, correlated with one pending request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(remote = "Self", rename_all = "camelCase")]
pub struct Chunk {
    /// Exact destination and originating operation.
    pub context: Context,
    /// Contiguous sequence beginning at zero.
    pub sequence: u32,
    /// Data whose shape is checked against the originating operation.
    pub payload: ChunkPayload,
}

/// Bounded operation data; SQL NULL and Mongo catalogue forms remain distinct.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum ChunkPayload {
    /// Relational schemas or Mongo collections, never generated instructions.
    Catalogue {
        /// Existing relational shape.
        schemas: Vec<DataSourceSchema>,
        /// Existing document shape, without flattening it into SQL columns.
        collections: Vec<MongoCollection>,
    },
    /// Text rows; absent cells stay `None`, while empty text stays `Some("")`.
    Rows {
        /// Identical headers on every row chunk for this operation.
        columns: Vec<String>,
        /// Complete bounded cells; overlong values must not be silently cut.
        rows: Vec<Vec<Option<String>>>,
    },
    /// One ready indication after the sample; transaction remains open.
    PreviewReady {
        /// One-use decision target.
        preview_id: String,
        /// Actual executed text, for display/comparison, never another execution.
        executed_sql: String,
        /// Remaining decision interval, from 1 to 60 seconds.
        expires_in_seconds: u32,
        /// Full affected count reported by the server.
        affected: u64,
        /// The sample omits rows; no individual value was silently truncated.
        truncated: bool,
    },
}

/// Successful response payload inside `Reply<Terminal>`; errors use numeric errors.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(remote = "Self", rename_all = "camelCase")]
pub struct Terminal {
    /// Exact originating operation context.
    pub context: Context,
    /// Next sequence after all accepted chunks.
    pub sequence: u32,
    /// Single operation-specific success result.
    pub result: TerminalResult,
}

/// Successful terminal shape; failures remain the external numeric error envelope.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum TerminalResult {
    /// Opened session and observed, still policy-limited capabilities.
    Open {
        /// Newly allocated opaque session.
        session_id: String,
        /// Public server version when detected.
        server_version: Option<String>,
        /// Observed known operations, not an authorization grant.
        operations: Vec<Operation>,
    },
    /// Successful connection test.
    Test {
        /// Public server version when detected.
        server_version: Option<String>,
    },
    /// Catalogue finished; the retained catalogue may be partial.
    Introspect {
        /// Whole catalogue truncation, not a partial value.
        truncated: bool,
    },
    /// Query finished once.
    Query {
        /// Reported affected count, when available.
        affected: Option<u64>,
        /// Whole result-set truncation.
        truncated: bool,
        /// Reported execution time.
        elapsed_ms: u64,
        /// Existing refresh request; not a commit guarantee.
        catalog_update: DataSourceCatalogUpdate,
    },
    /// Existing impact data, without transferring confirmation authority.
    Impact {
        /// Existing severity semantics.
        severity: SqlImpactSeverity,
        /// Existing per-statement measurements, bounded by the guard.
        statements: Vec<SqlStatementImpact>,
    },
    /// Original preview's final transaction outcome.
    Preview {
        /// Same identity as ready, absent if it never became ready.
        preview_id: Option<String>,
        /// Existing acknowledged/unknown transaction semantics.
        outcome: DataSourcePreviewOutcome,
        /// Existing refresh request, including partial failures.
        catalog_update: DataSourceCatalogUpdate,
    },
    /// Decision request acceptance, distinct from the original preview terminal.
    Decide {
        /// Original preview request.
        preview_operation_id: String,
        /// One-use preview identity.
        preview_id: String,
        /// Whether the adapter accepted this decision for that preview.
        accepted: bool,
    },
    /// Cancellation request acceptance, distinct from target completion.
    Cancel {
        /// Exact target operation.
        target_operation_id: String,
        /// Whether cancellation was accepted.
        accepted: bool,
    },
    /// Adapter acknowledges release of session resources.
    Close {},
    /// Adapter acknowledges release of instance resources; runtime verifies exit.
    Shutdown {},
}

super::object::object_serde!(
    Context,
    PublicOptions,
    Restrictions,
    OpenParams,
    SessionParams,
    StatementParams,
    ImpactParams,
    DecideParams,
    CancelParams,
    Request,
    Chunk,
    Terminal
);

#[cfg(test)]
#[path = "operation_tests.rs"]
mod tests;

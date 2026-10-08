//! Pure adapter initialization and public error mapping, with no effects.

use kinein_protocol::driver::{
    ApiRange, Error, FailureReason, InitializeParams, InitializeResult, Limits, Operation,
    OperationOutcome, Reply, Version,
};
use kinein_protocol::{JsonRpcError, JsonRpcErrorCode, JsonRpcRequest};
use serde_json::json;

/// Adapter API understood by this core; not the UI IPC version.
pub const API: ApiRange = ApiRange {
    min: Version { major: 1, minor: 0 },
    // 1.1 (2026-10-08): `Failure.engineMessage`, a mensagem do banco (39 §4.4).
    max: Version { major: 1, minor: 1 },
};

/// Local ceilings; a peer can lower these but cannot raise them.
pub const LIMITS: Limits = Limits {
    message_bytes: 1_048_576,
    in_flight: 8,
    rows: super::query::MAX_ROWS,
    columns: 128,
    cell_bytes: 16_384,
    retained_bytes: 8_388_608,
    catalogue_items: 5_000,
};

/// Local queue bound, to be enforced by the process bridge.
pub const QUEUE_CAPACITY: usize = 16;
/// Initialization deadline, to be enforced outside the dispatch loop.
pub const INITIALIZE_TIMEOUT_MS: u64 = 5_000;
/// Shutdown deadline; expiry is not evidence that the process ended.
pub const SHUTDOWN_TIMEOUT_MS: u64 = 5_000;

/// Negotiated data only; this does not establish a database connection.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Negotiated {
    /// Greatest API version in the intersection.
    pub api: Version,
    /// Budgets at or below both participants' ceilings.
    pub limits: Limits,
    /// Known implemented operations, not observed database permissions.
    pub operations: Vec<Operation>,
}

/// Fixed initialization failures, without remote text or raw JSON.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum HandshakeFailure {
    /// Empty, invalid or excessive protocol message.
    InvalidMessage,
    /// Response ID belongs to another request.
    WrongRequest,
    /// Response has invalid version, identity or operation metadata.
    InvalidMetadata,
    /// No API version is understood by both participants.
    IncompatibleApi,
    /// A required operation is absent or preview support is incomplete.
    MissingOperation,
    /// Peer budgets are zero or internally inconsistent.
    InvalidLimits,
    /// Adapter refused initialization; only allowlisted public fields survive.
    Rejected(JsonRpcError),
}

/// Builds the first request without destination, executable or credential data.
#[must_use]
pub fn initialize_request(id: u64) -> JsonRpcRequest {
    JsonRpcRequest::new(
        id,
        "driver.initialize",
        Some(json!(InitializeParams {
            api: API,
            limits: LIMITS,
        })),
    )
}

/// Accepts one bounded JSON line and checks its pending-request correlation.
pub fn accept_initialize(
    bytes: &[u8],
    expected_id: u64,
    adapter_id: &str,
    engine: &str,
) -> Result<Negotiated, HandshakeFailure> {
    let body = bytes.strip_suffix(b"\n").unwrap_or(bytes);
    let body = body.strip_suffix(b"\r").unwrap_or(body);
    if body.len() > LIMITS.message_bytes as usize
        || body.is_empty()
        || body.iter().any(|byte| matches!(byte, b'\n' | b'\r'))
    {
        return Err(HandshakeFailure::InvalidMessage);
    }
    let reply: Reply<InitializeResult> =
        serde_json::from_slice(body).map_err(|_| HandshakeFailure::InvalidMessage)?;
    if reply.id() != &json!(expected_id) {
        return Err(HandshakeFailure::WrongRequest);
    }
    match reply {
        Reply::Success { result, .. } => negotiate(&result, adapter_id, engine),
        Reply::Failure { error, .. } => Err(HandshakeFailure::Rejected(public_error(&error))),
    }
}

/// Validates known semantics before deriving any effective capabilities.
pub fn negotiate(
    peer: &InitializeResult,
    adapter_id: &str,
    engine: &str,
) -> Result<Negotiated, HandshakeFailure> {
    if peer.api.min.major == 0
        || peer.api.min.major != peer.api.max.major
        || peer.api.min.minor > peer.api.max.minor
        || peer.adapter_id != adapter_id
        || !valid_identity(&peer.adapter_id)
        || !valid_version_label(&peer.adapter_version)
        || peer
            .driver_version
            .as_deref()
            .is_some_and(|value| !valid_version_label(value))
        || peer.engines.is_empty()
        || peer.engines.len() > 16
        || peer.engines.iter().any(|value| !valid_identity(value))
        || repeated(&peer.engines)
        || !peer.engines.iter().any(|value| value == engine)
        || peer.operations.len() > 10
        || repeated(&peer.operations)
    {
        return Err(HandshakeFailure::InvalidMetadata);
    }
    let api = intersect_api(API, peer.api).ok_or(HandshakeFailure::IncompatibleApi)?;
    for operation in [
        Operation::Open,
        Operation::Test,
        Operation::Introspect,
        Operation::Query,
        Operation::Cancel,
        Operation::Close,
        Operation::Shutdown,
    ] {
        if !peer.operations.contains(&operation) {
            return Err(HandshakeFailure::MissingOperation);
        }
    }
    if peer.operations.contains(&Operation::Preview) != peer.operations.contains(&Operation::Decide)
    {
        return Err(HandshakeFailure::MissingOperation);
    }
    let limits = negotiate_limits(peer.limits)?;
    Ok(Negotiated {
        api,
        limits,
        operations: peer.operations.clone(),
    })
}

fn valid_identity(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
}

fn intersect_api(local: ApiRange, peer: ApiRange) -> Option<Version> {
    let min = local.min.minor.max(peer.min.minor);
    let max = local.max.minor.min(peer.max.minor);
    if local.min.major != peer.min.major || min > max {
        return None;
    }
    Some(Version {
        major: local.min.major,
        minor: max,
    })
}

fn valid_version_label(value: &str) -> bool {
    !value.is_empty() && value.len() <= 128 && !value.chars().any(char::is_control)
}

fn repeated<T: PartialEq>(values: &[T]) -> bool {
    values
        .iter()
        .enumerate()
        .any(|(index, value)| values[..index].contains(value))
}

/// Intersects budgets without raising local ceilings or accepting zero limits.
///
/// `inFlight` needs at least two slots: a pending preview or query must still
/// leave room for its own decision or cancellation, which also counts.
pub fn negotiate_limits(peer: Limits) -> Result<Limits, HandshakeFailure> {
    let limits = Limits {
        message_bytes: LIMITS.message_bytes.min(peer.message_bytes),
        in_flight: LIMITS.in_flight.min(peer.in_flight),
        rows: LIMITS.rows.min(peer.rows),
        columns: LIMITS.columns.min(peer.columns),
        cell_bytes: LIMITS.cell_bytes.min(peer.cell_bytes),
        retained_bytes: LIMITS.retained_bytes.min(peer.retained_bytes),
        catalogue_items: LIMITS.catalogue_items.min(peer.catalogue_items),
    };
    if limits.message_bytes < 1024
        || limits.in_flight < 2
        || limits.rows == 0
        || limits.columns == 0
        || limits.cell_bytes == 0
        || limits.retained_bytes == 0
        || limits.catalogue_items == 0
        || limits.cell_bytes > limits.message_bytes
        || limits.cell_bytes > limits.retained_bytes
    {
        return Err(HandshakeFailure::InvalidLimits);
    }
    Ok(limits)
}

/// Public text when the database result cannot be established.
const OUTCOME_UNKNOWN: &str = "Não foi possível confirmar o resultado no banco. O resultado é \
    indeterminado; a operação não será repetida automaticamente.";

/// Public text when a started operation failed after earlier commands could apply.
const FAILED_AFTER_START: &str = "A operação falhou depois de iniciada; comandos anteriores \
    de um lote podem ter sido aplicados e não serão repetidos nem desfeitos automaticamente.";

/// Maps only known machine data; adapter message text never leaves this boundary.
///
/// The outcome decides first: an unknown result never receives a code that
/// offers a new credential or a new preparation, and a failure after start
/// only keeps those codes when nothing ran (`notStarted`). The original typed
/// reason stays in the public details.
#[must_use]
pub fn public_error(error: &Error) -> JsonRpcError {
    let Some(data) = error.data.as_ref().filter(|_| error.code == -32000) else {
        return JsonRpcError::new(
            JsonRpcErrorCode::InternalError,
            "O adaptador recusou a operação ou enviou uma resposta incompatível.",
            None,
        );
    };
    let outcome = if data.reason == FailureReason::OutcomeUnknown {
        OperationOutcome::Unknown
    } else {
        data.outcome
    };
    let (code, message) = match (outcome, data.reason) {
        (OperationOutcome::Unknown, _) => (JsonRpcErrorCode::InternalError, OUTCOME_UNKNOWN),
        (
            OperationOutcome::Failed,
            FailureReason::SecretRequired | FailureReason::ContextChanged,
        ) => (JsonRpcErrorCode::InternalError, FAILED_AFTER_START),
        (_, reason) => reason_error(reason),
    };
    JsonRpcError::new(
        code,
        message,
        Some(json!({"driverReason":data.reason,"outcome":outcome})),
    )
}

/// Existing UI code and text for one typed reason, before outcome precedence.
const fn reason_error(reason: FailureReason) -> (JsonRpcErrorCode, &'static str) {
    match reason {
        FailureReason::SecretRequired => (
            JsonRpcErrorCode::SecretRequired,
            "O banco exige uma credencial para esta sessão.",
        ),
        FailureReason::ReadOnly => (
            JsonRpcErrorCode::ReadOnlyViolation,
            "Esta conexão permite somente leitura.",
        ),
        FailureReason::ContextChanged => (
            JsonRpcErrorCode::DataSourceContextChanged,
            "O destino mudou; prepare a operação novamente.",
        ),
        FailureReason::IncompatibleApi => (
            JsonRpcErrorCode::InternalError,
            "O adaptador usa uma API incompatível.",
        ),
        FailureReason::UnsupportedOperation => (
            JsonRpcErrorCode::InternalError,
            "O adaptador não oferece esta operação.",
        ),
        FailureReason::Busy => (
            JsonRpcErrorCode::InternalError,
            "O adaptador está ocupado; aguarde o término das operações.",
        ),
        FailureReason::Cancelled => (JsonRpcErrorCode::InternalError, "A operação foi cancelada."),
        FailureReason::Timeout => (
            JsonRpcErrorCode::InternalError,
            "O adaptador excedeu o prazo da operação.",
        ),
        FailureReason::ConnectionFailed => (
            JsonRpcErrorCode::InternalError,
            "Não foi possível manter a conexão com o banco.",
        ),
        FailureReason::ExecutionFailed => (
            JsonRpcErrorCode::InternalError,
            "O banco informou uma falha na execução.",
        ),
        FailureReason::LimitExceeded => (
            JsonRpcErrorCode::InternalError,
            "A operação excedeu um limite negociado com o adaptador.",
        ),
        FailureReason::OutcomeUnknown => (JsonRpcErrorCode::InternalError, OUTCOME_UNKNOWN),
    }
}

#[cfg(test)]
mod tests;

//! Negotiation fixtures and failures have no process or database effects.

use super::*;
use kinein_protocol::driver::Failure;
use serde_json::Value;

const REQUEST: &str =
    include_str!("../../../../kinein-protocol/src/driver/fixtures/v1/initialize-request.json");
const RESPONSE: &str =
    include_str!("../../../../kinein-protocol/src/driver/fixtures/v1/initialize-reply.json");
const FAILURE: &str =
    include_str!("../../../../kinein-protocol/src/driver/fixtures/v1/unknown-outcome.json");

fn peer() -> InitializeResult {
    let Reply::Success { result, .. } = serde_json::from_str(RESPONSE).unwrap() else {
        panic!("expected initialization fixture");
    };
    result
}

#[test]
fn initialize_request_matches_versioned_fixture_without_destination_or_secret() {
    assert_eq!(
        serde_json::to_value(initialize_request(1)).unwrap(),
        serde_json::from_str::<Value>(REQUEST).unwrap()
    );
}

#[test]
fn compatible_minor_selects_highest_common_and_never_raises_local_limits() {
    let negotiated =
        accept_initialize(RESPONSE.as_bytes(), 1, "native.postgres", "postgres").unwrap();
    assert_eq!(negotiated.api, API.max);
    assert_eq!(negotiated.limits.message_bytes, LIMITS.message_bytes);
    assert_eq!(negotiated.limits.in_flight, 4);
    assert_eq!(negotiated.limits.rows, 1000);
    assert_eq!(negotiated.limits.columns, 64);
    assert_eq!(negotiated.limits.cell_bytes, 8192);
    assert_eq!(negotiated.limits.retained_bytes, 4_194_304);
    assert_eq!(negotiated.limits.catalogue_items, 2500);
    let mut response = peer();
    response.limits = Limits {
        message_bytes: u32::MAX,
        in_flight: u16::MAX,
        rows: u32::MAX,
        columns: u16::MAX,
        cell_bytes: u32::MAX,
        retained_bytes: u32::MAX,
        catalogue_items: u32::MAX,
    };
    assert_eq!(
        negotiate(&response, "native.postgres", "postgres")
            .unwrap()
            .limits,
        LIMITS
    );
}

#[test]
fn intersection_selects_latest_shared_minor_when_both_ranges_span_revisions() {
    let local = ApiRange {
        min: Version { major: 1, minor: 0 },
        max: Version { major: 1, minor: 3 },
    };
    let peer = ApiRange {
        min: Version { major: 1, minor: 1 },
        max: Version { major: 1, minor: 2 },
    };
    assert_eq!(
        intersect_api(local, peer),
        Some(Version { major: 1, minor: 2 })
    );
}

#[test]
fn no_intersection_or_invalid_range_refuses_only_selected_adapter() {
    for (range, expected) in [
        (
            ApiRange {
                min: Version { major: 2, minor: 0 },
                max: Version { major: 2, minor: 0 },
            },
            HandshakeFailure::IncompatibleApi,
        ),
        (
            ApiRange {
                min: Version { major: 1, minor: 1 },
                max: Version { major: 1, minor: 2 },
            },
            HandshakeFailure::IncompatibleApi,
        ),
        (
            ApiRange {
                min: Version { major: 1, minor: 0 },
                max: Version { major: 2, minor: 0 },
            },
            HandshakeFailure::InvalidMetadata,
        ),
        (
            ApiRange {
                min: Version { major: 1, minor: 2 },
                max: Version { major: 1, minor: 0 },
            },
            HandshakeFailure::InvalidMetadata,
        ),
        (
            ApiRange {
                min: Version { major: 0, minor: 0 },
                max: Version { major: 0, minor: 0 },
            },
            HandshakeFailure::InvalidMetadata,
        ),
    ] {
        let mut response = peer();
        response.api = range;
        assert_eq!(
            negotiate(&response, "native.postgres", "postgres"),
            Err(expected)
        );
    }
}

#[test]
fn identity_required_operations_and_preview_pair_cannot_fall_back() {
    let response = peer();
    assert_eq!(
        negotiate(&response, "other.postgres", "postgres"),
        Err(HandshakeFailure::InvalidMetadata)
    );
    assert_eq!(
        negotiate(&response, "native.postgres", "sqlite"),
        Err(HandshakeFailure::InvalidMetadata)
    );
    for operation in [
        Operation::Open,
        Operation::Test,
        Operation::Introspect,
        Operation::Query,
        Operation::Cancel,
        Operation::Close,
        Operation::Shutdown,
        Operation::Decide,
        Operation::Preview,
    ] {
        let mut response = response.clone();
        response.operations.retain(|value| *value != operation);
        assert_eq!(
            negotiate(&response, "native.postgres", "postgres"),
            Err(HandshakeFailure::MissingOperation)
        );
    }
    let mut minimal = response.clone();
    minimal.operations.retain(|value| {
        !matches!(
            value,
            Operation::Impact | Operation::Preview | Operation::Decide
        )
    });
    assert!(negotiate(&minimal, "native.postgres", "postgres").is_ok());
    let mut duplicate = response.clone();
    duplicate.engines.push("postgres".to_owned());
    assert_eq!(
        negotiate(&duplicate, "native.postgres", "postgres"),
        Err(HandshakeFailure::InvalidMetadata)
    );
    let mut duplicate = response;
    duplicate.operations.push(Operation::Open);
    assert_eq!(
        negotiate(&duplicate, "native.postgres", "postgres"),
        Err(HandshakeFailure::InvalidMetadata)
    );
}

#[test]
fn zero_and_inconsistent_budgets_are_not_negotiated_as_unlimited() {
    let response = peer();
    for field in [
        "messageBytes",
        "inFlight",
        "rows",
        "columns",
        "cellBytes",
        "retainedBytes",
        "catalogueItems",
    ] {
        let mut value = serde_json::to_value(&response).unwrap();
        value["limits"][field] = json!(0);
        let wrong = serde_json::from_value(value).unwrap();
        assert_eq!(
            negotiate(&wrong, "native.postgres", "postgres"),
            Err(HandshakeFailure::InvalidLimits),
            "{field}"
        );
    }
    let mut response = response;
    response.limits.message_bytes = 1024;
    assert_eq!(
        negotiate(&response, "native.postgres", "postgres"),
        Err(HandshakeFailure::InvalidLimits)
    );
    response.limits.cell_bytes = 1024;
    assert!(negotiate(&response, "native.postgres", "postgres").is_ok());
    response.limits.retained_bytes = 512;
    assert_eq!(
        negotiate(&response, "native.postgres", "postgres"),
        Err(HandshakeFailure::InvalidLimits)
    );
}

#[test]
fn bounded_single_line_reply_checks_exact_request_without_raw_json_errors() {
    assert_eq!(
        accept_initialize(RESPONSE.as_bytes(), 2, "native.postgres", "postgres"),
        Err(HandshakeFailure::WrongRequest)
    );
    for bytes in [
        b"".as_slice(),
        b"{}",
        b"[]",
        b"{\n}",
        b"PRIVATE_CREDENTIAL_MUST_NOT_ESCAPE",
    ] {
        let error = accept_initialize(bytes, 1, "native.postgres", "postgres").unwrap_err();
        assert_eq!(error, HandshakeFailure::InvalidMessage);
        assert!(!format!("{error:?}").contains("PRIVATE_CREDENTIAL"));
    }
    let large = vec![b' '; LIMITS.message_bytes as usize + 1];
    assert_eq!(
        accept_initialize(&large, 1, "native.postgres", "postgres"),
        Err(HandshakeFailure::InvalidMessage)
    );
    let mut windows_line = RESPONSE.trim_end().as_bytes().to_vec();
    windows_line.extend_from_slice(b"\r\n");
    assert!(accept_initialize(&windows_line, 1, "native.postgres", "postgres").is_ok());
}

#[test]
fn remote_error_mapping_discards_external_text_and_preserves_uncertainty() {
    let error =
        accept_initialize(FAILURE.as_bytes(), 7, "native.postgres", "postgres").unwrap_err();
    let HandshakeFailure::Rejected(error) = error else {
        panic!("expected public refusal");
    };
    assert!(error.message.contains("indeterminado"));
    assert!(
        !serde_json::to_string(&error)
            .unwrap()
            .contains("PRIVATE_CREDENTIAL")
    );
    for (reason, code) in [
        (
            FailureReason::SecretRequired,
            JsonRpcErrorCode::SecretRequired,
        ),
        (FailureReason::ReadOnly, JsonRpcErrorCode::ReadOnlyViolation),
        (
            FailureReason::ContextChanged,
            JsonRpcErrorCode::DataSourceContextChanged,
        ),
        (FailureReason::Timeout, JsonRpcErrorCode::InternalError),
    ] {
        let remote = Error {
            code: -32000,
            message: "PRIVATE_CREDENTIAL_MUST_NOT_ESCAPE".to_owned(),
            data: Some(Failure {
                reason,
                outcome: OperationOutcome::NotStarted,
            }),
        };
        let mapped = public_error(&remote);
        assert_eq!(mapped.code, code);
        assert!(
            !serde_json::to_string(&mapped)
                .unwrap()
                .contains("PRIVATE_CREDENTIAL")
        );
    }
    let standard = Error {
        code: -32603,
        message: "PRIVATE_CREDENTIAL_MUST_NOT_ESCAPE".to_owned(),
        data: None,
    };
    assert!(
        !public_error(&standard)
            .message
            .contains("PRIVATE_CREDENTIAL")
    );
}

#[test]
fn unknown_or_failed_execution_never_requests_a_credential_retry() {
    for reason in [
        FailureReason::SecretRequired,
        FailureReason::ReadOnly,
        FailureReason::ContextChanged,
        FailureReason::Timeout,
        FailureReason::ExecutionFailed,
    ] {
        let mapped = public_error(&Error {
            code: -32000,
            message: "PRIVATE_CREDENTIAL_MUST_NOT_ESCAPE".to_owned(),
            data: Some(Failure {
                reason,
                outcome: OperationOutcome::Unknown,
            }),
        });
        assert_eq!(mapped.code, JsonRpcErrorCode::InternalError, "{reason:?}");
        assert!(mapped.message.contains("indeterminado"));
        assert!(!mapped.message.contains("credencial"));
        assert!(!mapped.message.contains("PRIVATE_CREDENTIAL"));
        assert_eq!(
            mapped.details.as_ref().unwrap()["driverReason"],
            json!(reason)
        );
        assert_eq!(
            mapped.details.as_ref().unwrap()["outcome"],
            json!("unknown")
        );
    }
    let mapped = public_error(&Error {
        code: -32000,
        message: "PRIVATE_CREDENTIAL_MUST_NOT_ESCAPE".to_owned(),
        data: Some(Failure {
            reason: FailureReason::SecretRequired,
            outcome: OperationOutcome::Failed,
        }),
    });
    assert_eq!(mapped.code, JsonRpcErrorCode::InternalError);
    assert!(mapped.message.contains("comandos anteriores"));
    assert!(!mapped.message.contains("PRIVATE_CREDENTIAL"));
}

#[test]
fn a_pending_operation_must_leave_room_for_a_control_request() {
    let mut response = peer();
    response.limits.in_flight = 1;
    assert_eq!(
        negotiate(&response, "native.postgres", "postgres"),
        Err(HandshakeFailure::InvalidLimits)
    );
    response.limits.in_flight = 2;
    assert_eq!(
        negotiate(&response, "native.postgres", "postgres")
            .unwrap()
            .limits
            .in_flight,
        2
    );
}

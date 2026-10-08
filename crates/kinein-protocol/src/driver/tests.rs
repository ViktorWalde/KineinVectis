//! Versioned fixtures exercise the external wire without changing UI IPC.

use serde_json::{Value, json};

use super::{Error, FailureReason, InitializeParams, InitializeResult, OperationOutcome, Reply};

const REQUEST: &str = include_str!("fixtures/v1/initialize-request.json");
const RESPONSE: &str = include_str!("fixtures/v1/initialize-reply.json");
const FAILURE: &str = include_str!("fixtures/v1/unknown-outcome.json");

#[test]
fn initialize_fixture_is_strict_without_credentials_or_paths() {
    let request: crate::JsonRpcRequest = serde_json::from_str(REQUEST).unwrap();
    let params = request.params.unwrap();
    let decoded: InitializeParams = serde_json::from_value(params.clone()).unwrap();
    assert_eq!(serde_json::to_value(decoded).unwrap(), params);
    for path in [vec![], vec!["api"], vec!["api", "min"], vec!["limits"]] {
        let mut wrong = params.clone();
        let mut object = &mut wrong;
        for key in path {
            object = &mut object[key];
        }
        object["password"] = json!("PRIVATE_CREDENTIAL_MUST_NOT_ESCAPE");
        assert!(serde_json::from_value::<InitializeParams>(wrong).is_err());
    }
}

#[test]
fn additive_response_fields_do_not_change_known_api_or_operations() {
    let reply: Reply<InitializeResult> = serde_json::from_str(RESPONSE).unwrap();
    let Reply::Success { id, result } = reply else {
        panic!("expected initialization result");
    };
    assert_eq!(id, json!(1));
    assert_eq!(result.api.max.minor, 2);
    assert_eq!(result.operations.len(), 10);
    let encoded = serde_json::to_value(Reply::Success { id, result }).unwrap();
    assert!(encoded.get("error").is_none());
    assert!(encoded["result"].get("futureOptional").is_none());
}

#[test]
fn numeric_error_remains_distinct_from_textual_ui_error() {
    let reply: Reply<Value> = serde_json::from_str(FAILURE).unwrap();
    assert!(serde_json::from_str::<crate::JsonRpcResponse>(FAILURE).is_err());
    let Reply::Failure { error, id } = reply else {
        panic!("expected external error");
    };
    assert_eq!(id, json!(7));
    assert_eq!(error.code, -32000);
    let data = error.data.as_ref().unwrap();
    assert_eq!(data.reason, FailureReason::OutcomeUnknown);
    assert_eq!(data.outcome, OperationOutcome::Unknown);
    assert!(!format!("{error:?}").contains("PRIVATE_CREDENTIAL"));
    assert!(
        serde_json::to_value(&error)
            .unwrap()
            .get("details")
            .is_none()
    );
}

#[test]
fn reply_rejects_ambiguous_missing_duplicate_and_invalid_envelopes() {
    for body in [
        r#"{"jsonrpc":"2.0","id":1}"#,
        r#"{"jsonrpc":"2.0","id":1,"result":{},"error":{"code":-32603,"message":"x"}}"#,
        r#"{"jsonrpc":"2.0","result":{}}"#,
        r#"{"jsonrpc":"1.0","id":1,"result":{}}"#,
        r#"{"jsonrpc":"2.0","id":{},"result":{}}"#,
        r#"{"jsonrpc":"2.0","id":true,"result":{}}"#,
        r#"{"jsonrpc":"2.0","id":1.5,"result":{}}"#,
        r#"{"jsonrpc":"2.0","id":1,"id":2,"result":{}}"#,
        r#"{"jsonrpc":"2.0","id":1,"result":{},"result":{}}"#,
        r#"{"jsonrpc":"2.0","id":1,"error":null}"#,
        r#"{"jsonrpc":"2.0","id":1,"error":[-32000,"secret"]}"#,
        r#"{"jsonrpc":"2.0","id":1,"error":{"code":-32000,"message":"secret","data":["timeout","unknown"]}}"#,
        r#"[{"jsonrpc":"2.0","id":1,"result":{}}]"#,
    ] {
        assert!(
            serde_json::from_str::<Reply<Value>>(body).is_err(),
            "{body}"
        );
    }
}

#[test]
fn explicit_null_success_and_standard_numeric_errors_preserve_json_rpc_shape() {
    let reply: Reply<Value> =
        serde_json::from_str(r#"{"jsonrpc":"2.0","id":"r","result":null}"#).unwrap();
    assert_eq!(
        serde_json::to_value(reply).unwrap(),
        json!({"jsonrpc":"2.0","id":"r","result":null})
    );
    let reply: Reply<Value> = serde_json::from_str(
        r#"{"jsonrpc":"2.0","id":null,"error":{"code":-32700,"message":"bad"}}"#,
    )
    .unwrap();
    assert!(reply.id().is_null());
    assert!(serde_json::to_value(reply).unwrap().get("result").is_none());
}

#[test]
fn unknown_required_semantics_and_changed_error_types_are_rejected() {
    let mut response: Value = serde_json::from_str(RESPONSE).unwrap();
    response["result"]["operations"][0] = json!("executeCallback");
    assert!(serde_json::from_value::<Reply<InitializeResult>>(response).is_err());
    let mut error: Value = serde_json::from_str(FAILURE).unwrap();
    error["error"]["data"]["reason"] = json!("futureReason");
    assert!(serde_json::from_value::<Reply<Value>>(error).is_err());
    assert!(
        serde_json::from_value::<Error>(json!({"code":"INTERNAL_ERROR","message":"x"})).is_err()
    );
}

#[test]
fn object_contracts_reject_positional_arrays_at_every_initialization_layer() {
    let request: Value = serde_json::from_str(REQUEST).unwrap();
    let params = request["params"].clone();
    let array = json!([params["api"], params["limits"]]);
    assert!(serde_json::from_value::<InitializeParams>(array).is_err());
    for (field, array) in [
        ("api", json!([params["api"]["min"], params["api"]["max"]])),
        (
            "limits",
            json!([1_048_576, 8, 10_000, 128, 16_384, 8_388_608, 5_000]),
        ),
    ] {
        let mut wrong = params.clone();
        wrong[field] = array;
        assert!(
            serde_json::from_value::<InitializeParams>(wrong).is_err(),
            "{field}"
        );
    }
    let mut wrong = params;
    wrong["api"]["min"] = json!([1, 0]);
    assert!(serde_json::from_value::<InitializeParams>(wrong).is_err());
    assert!(serde_json::from_str::<Reply<Value>>(r#"["2.0",1,{}]"#).is_err());
    let response: Value = serde_json::from_str(RESPONSE).unwrap();
    let result = &response["result"];
    let array = json!([
        result["api"],
        result["adapterId"],
        result["adapterVersion"],
        result["driverVersion"],
        result["engines"],
        result["operations"],
        result["limits"]
    ]);
    assert!(serde_json::from_value::<InitializeResult>(array).is_err());
}

#[test]
fn the_engine_message_of_api_one_point_one_is_optional_and_strict() {
    // 1.1 (39 §4.4): a mensagem do BANCO vai no `data`; ausente, o fio e' o
    // mesmo do 1.0; campo adicional numa RESPOSTA e' ignorado (39 §4.4).
    let with_message = json!({
        "jsonrpc": "2.0", "id": 9,
        "error": { "code": -32000, "message": "adapter prose",
                   "data": { "reason": "executionFailed", "outcome": "failed",
                             "engineMessage": "no such table: nada" } }
    });
    let reply: Reply<Value> = serde_json::from_value(with_message.clone()).unwrap();
    let Reply::Failure { error, .. } = &reply else {
        panic!("expected a failure");
    };
    let data = error.data.as_ref().unwrap();
    assert_eq!(data.reason, FailureReason::ExecutionFailed);
    assert_eq!(data.engine_message.as_deref(), Some("no such table: nada"));
    assert_eq!(serde_json::to_value(&reply).unwrap(), with_message);

    let old: Reply<Value> = serde_json::from_str(FAILURE).unwrap();
    let Reply::Failure { error, .. } = &old else {
        panic!("expected a failure");
    };
    assert_eq!(error.data.as_ref().unwrap().engine_message, None);
    assert!(
        !serde_json::to_string(&old)
            .unwrap()
            .contains("engineMessage")
    );

    let mut later = with_message;
    later["error"]["data"]["hint"] = json!("x");
    let Reply::Failure { error, .. } = serde_json::from_value::<Reply<Value>>(later).unwrap()
    else {
        panic!("expected a failure");
    };
    assert_eq!(
        error.data.unwrap().engine_message.as_deref(),
        Some("no such table: nada")
    );
}

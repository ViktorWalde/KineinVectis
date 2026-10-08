use super::*;
use serde_json::{Value, json};

#[test]
fn operational_objects_refuse_positional_arrays() {
    assert!(
        serde_json::from_value::<Context>(json!(["instance", "session", 7, "operation"])).is_err()
    );
    assert!(serde_json::from_value::<PublicOptions>(json!([1, {}])).is_err());
    assert!(
        serde_json::from_value::<SessionParams>(json!([["instance", "session", 7, "operation"]]))
            .is_err()
    );
    let chunk: Value =
        serde_json::from_str(include_str!("fixtures/v1/operation-rows.json")).unwrap();
    assert!(
        serde_json::from_value::<Chunk>(json!([chunk["context"], 0, chunk["payload"]])).is_err()
    );
    let mut nested = chunk;
    nested["context"] = json!(["instance", "session", 7, "operation"]);
    assert!(serde_json::from_value::<Chunk>(nested).is_err());
}

#[test]
fn opening_keeps_public_options_separate_and_hides_nested_debug() {
    let request: Request =
        serde_json::from_str(include_str!("fixtures/v1/operation-open.json")).unwrap();
    let debug = format!("{request:?}");
    assert!(!debug.contains("PRIVATE_CREDENTIAL_FIXTURE"));
    assert!(debug.contains("REDACTED"));
    let value = serde_json::to_value(&request).unwrap();
    assert_eq!(value["params"]["credential"], "PRIVATE_CREDENTIAL_FIXTURE");
    assert!(
        value["params"]["options"]["fields"]
            .get("credential")
            .is_none()
    );
    assert_eq!(serde_json::from_value::<Request>(value).unwrap(), request);
}

#[test]
fn requests_refuse_unknown_fields_authority_and_reverse_requests() {
    let open: Value =
        serde_json::from_str(include_str!("fixtures/v1/operation-open.json")).unwrap();
    for field in ["password", "executeSql", "timeout", "allow", "confirmWrite"] {
        let mut wrong = open.clone();
        wrong["params"][field] = json!(true);
        assert!(serde_json::from_value::<Request>(wrong).is_err(), "{field}");
    }
    let mut wrong = open.clone();
    wrong["params"]["restrictions"]["allow"] = json!(true);
    assert!(serde_json::from_value::<Request>(wrong).is_err());
    let mut wrong = open;
    wrong["method"] = json!("driver.executeSqlCallback");
    assert!(serde_json::from_value::<Request>(wrong).is_err());
}

#[test]
fn every_operational_method_roundtrips_and_only_open_accepts_credential() {
    let open: Value =
        serde_json::from_str(include_str!("fixtures/v1/operation-open.json")).unwrap();
    let mut context = open["params"]["context"].clone();
    context["sessionId"] = json!("session-1");
    for (method, extra, operation) in [
        ("test", json!({}), Operation::Test),
        ("introspect", json!({}), Operation::Introspect),
        (
            "query",
            json!({"text":"SELECT 1","maxRows":500}),
            Operation::Query,
        ),
        (
            "impact",
            json!({"text":"DELETE FROM sample"}),
            Operation::Impact,
        ),
        (
            "preview",
            json!({"text":"UPDATE sample SET value=1","maxRows":500}),
            Operation::Preview,
        ),
        (
            "decide",
            json!({"previewOperationId":"original","previewId":"preview-1","decision":"rollback"}),
            Operation::Decide,
        ),
        (
            "cancel",
            json!({"targetOperationId":"original"}),
            Operation::Cancel,
        ),
        ("close", json!({}), Operation::Close),
        ("shutdown", json!({}), Operation::Shutdown),
    ] {
        let mut params = extra;
        params["context"] = context.clone();
        if method == "shutdown" {
            params["context"]["sessionId"] = Value::Null;
        }
        let value = json!({"method":format!("driver.{method}"),"params":params});
        let request: Request = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(request.operation(), operation);
        assert_eq!(serde_json::to_value(&request).unwrap(), value);
        let mut wrong = value;
        wrong["params"]["credential"] = json!("secret");
        assert!(serde_json::from_value::<Request>(wrong).is_err());
    }
}

#[test]
fn row_fixture_preserves_null_empty_text_and_utf8() {
    let chunk: Chunk =
        serde_json::from_str(include_str!("fixtures/v1/operation-rows.json")).unwrap();
    let ChunkPayload::Rows { rows, .. } = &chunk.payload else {
        panic!("wrong fixture kind");
    };
    assert_eq!(
        rows[0],
        vec![None, Some(String::new()), Some("ação".to_owned())]
    );
    assert_eq!(
        serde_json::from_value::<Chunk>(serde_json::to_value(&chunk).unwrap()).unwrap(),
        chunk
    );
}

#[test]
fn mongo_fixture_keeps_document_shape_and_sample_presence() {
    let chunk: Chunk =
        serde_json::from_str(include_str!("fixtures/v1/operation-mongo.json")).unwrap();
    let ChunkPayload::Catalogue {
        schemas,
        collections,
    } = &chunk.payload
    else {
        panic!("wrong fixture kind");
    };
    assert!(schemas.is_empty());
    assert!(!collections[0].declared);
    assert_eq!(collections[0].fields[0].presence, Some(0.5));
    assert_eq!(collections[0].fields[0].types, ["null", "string"]);
    assert_eq!(
        serde_json::from_value::<Chunk>(serde_json::to_value(&chunk).unwrap()).unwrap(),
        chunk
    );
}

#[test]
fn ready_and_terminal_preserve_unknown_commit_outcome() {
    let ready: Chunk =
        serde_json::from_str(include_str!("fixtures/v1/operation-preview-ready.json")).unwrap();
    assert!(matches!(
        ready.payload,
        ChunkPayload::PreviewReady {
            expires_in_seconds: 60,
            ..
        }
    ));
    let terminal: Terminal =
        serde_json::from_str(include_str!("fixtures/v1/operation-preview-terminal.json")).unwrap();
    assert!(matches!(
        terminal.result,
        TerminalResult::Preview {
            outcome: DataSourcePreviewOutcome::Unknown,
            ..
        }
    ));
    let mut additive = serde_json::to_value(&terminal).unwrap();
    additive["futureMetadata"] = json!({"safe":"ignored"});
    assert_eq!(
        serde_json::from_value::<Terminal>(additive).unwrap(),
        terminal
    );
}

#[test]
fn public_options_refuse_a_repeated_field_name() {
    // A map would keep the last value silently; the first one could be what the
    // author reviewed. Bytes, not a Value: a Value already lost the duplicate.
    let text = r#"{"schemaVersion":1,"fields":{"host":"reviewed","host":"other"}}"#;
    assert!(serde_json::from_str::<PublicOptions>(text).is_err());
    let unique = r#"{"schemaVersion":1,"fields":{"host":"reviewed","port":5432}}"#;
    assert_eq!(
        serde_json::from_str::<PublicOptions>(unique)
            .unwrap()
            .fields["port"],
        OptionValue::Integer(5432)
    );
}

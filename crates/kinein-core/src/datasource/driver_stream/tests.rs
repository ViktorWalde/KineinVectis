use super::*;
use kinein_protocol::driver::operation::{CancelParams, DecideParams, StatementParams};
use kinein_protocol::{
    DataSourceCatalogUpdate, DataSourcePreviewDecision, DataSourcePreviewOutcome,
};
use serde_json::json;

const ROWS: &str =
    include_str!("../../../../kinein-protocol/src/driver/fixtures/v1/operation-rows.json");
const MONGO: &str =
    include_str!("../../../../kinein-protocol/src/driver/fixtures/v1/operation-mongo.json");
const READY: &str =
    include_str!("../../../../kinein-protocol/src/driver/fixtures/v1/operation-preview-ready.json");
const PREVIEW_END: &str = include_str!(
    "../../../../kinein-protocol/src/driver/fixtures/v1/operation-preview-terminal.json"
);

fn rows() -> Chunk {
    serde_json::from_str(ROWS).unwrap()
}
fn guard(operation: Operation) -> StreamGuard {
    let mut context = rows().context;
    if matches!(operation, Operation::Open | Operation::Shutdown) {
        context.session_id = None;
    }
    StreamGuard::new(context, operation, super::super::driver_contract::LIMITS).unwrap()
}
fn query_end(context: Context, sequence: u32) -> Terminal {
    Terminal {
        context,
        sequence,
        result: TerminalResult::Query {
            affected: None,
            truncated: false,
            elapsed_ms: 1,
            catalog_update: DataSourceCatalogUpdate::None,
        },
    }
}
fn serialized(value: &impl Serialize) -> Vec<u8> {
    serde_json::to_vec(value).unwrap()
}

#[test]
fn exact_context_and_contiguous_sequence_precede_state_changes() {
    let original = rows();
    for field in ["instanceId", "sessionId", "generation", "operationId"] {
        let mut wrong = serde_json::to_value(&original).unwrap();
        wrong["context"][field] = if field == "generation" {
            json!(8)
        } else {
            json!("another")
        };
        let mut stream = guard(Operation::Query);
        assert_eq!(
            stream.accept_chunk(&serde_json::from_value(wrong).unwrap()),
            Err(StreamFailure::WrongContext)
        );
        assert_eq!(stream.statistics(), Statistics::default());
        stream.accept_chunk(&original).unwrap();
    }
    let mut stream = guard(Operation::Query);
    let mut wrong = original.clone();
    wrong.sequence = 1;
    assert_eq!(
        stream.accept_chunk(&wrong),
        Err(StreamFailure::WrongSequence)
    );
    stream.accept_chunk(&original).unwrap();
    assert_eq!(
        stream.accept_chunk(&original),
        Err(StreamFailure::WrongSequence)
    );
}

#[test]
fn null_values_and_full_cells_survive_but_bad_dimensions_never_advance() {
    let original = rows();
    let mut stream = guard(Operation::Query);
    let mut wrong = original.clone();
    let ChunkPayload::Rows { rows, .. } = &mut wrong.payload else {
        unreachable!()
    };
    rows[0].pop();
    assert_eq!(stream.accept_chunk(&wrong), Err(StreamFailure::InvalidRows));
    let accepted = stream.accept_chunk_bytes(&serialized(&original)).unwrap();
    assert_eq!(accepted, original);
    let mut changed = original;
    changed.sequence = 1;
    let ChunkPayload::Rows { columns, .. } = &mut changed.payload else {
        unreachable!()
    };
    columns[0] = "different".to_owned();
    let previous = stream.statistics();
    assert_eq!(
        stream.accept_chunk(&changed),
        Err(StreamFailure::ChangedColumns)
    );
    assert_eq!(stream.statistics(), previous);
}

#[test]
fn utf8_cell_and_column_ceilings_refuse_complete_message_without_cutting() {
    let mut original = rows();
    let mut limits = super::super::driver_contract::LIMITS;
    limits.cell_bytes = 4;
    let ChunkPayload::Rows {
        columns,
        rows: values,
    } = &mut original.payload
    else {
        unreachable!()
    };
    *columns = vec!["x".to_owned()];
    *values = vec![vec![Some("ééé".to_owned())]];
    let mut stream = StreamGuard::new(original.context.clone(), Operation::Query, limits).unwrap();
    assert_eq!(
        stream.accept_chunk(&original),
        Err(StreamFailure::CellLimit)
    );
    assert_eq!(stream.statistics(), Statistics::default());
    assert!(
        serialized(&original)
            .windows(6)
            .any(|part| part == "ééé".as_bytes())
    );
    limits.columns = 1;
    let mut stream = StreamGuard::new(original.context, Operation::Query, limits).unwrap();
    assert_eq!(
        stream.accept_chunk(&rows()),
        Err(StreamFailure::InvalidRows)
    );
}

#[test]
fn row_counts_and_requested_ceiling_accumulate_across_chunks() {
    let original = rows();
    let request = Request::Query(StatementParams {
        context: original.context.clone(),
        text: "SELECT 1".to_owned(),
        max_rows: 1,
    });
    let mut stream =
        StreamGuard::for_request(&request, super::super::driver_contract::LIMITS).unwrap();
    stream.accept_chunk(&original).unwrap();
    let mut another = original;
    another.sequence = 1;
    let previous = stream.statistics();
    assert_eq!(stream.accept_chunk(&another), Err(StreamFailure::RowLimit));
    assert_eq!(stream.statistics(), previous);
}

#[test]
fn byte_budgets_include_raw_additive_fields_and_terminal() {
    let original = rows();
    let mut limits = super::super::driver_contract::LIMITS;
    limits.message_bytes = 1024;
    limits.cell_bytes = 64;
    limits.retained_bytes = 500;
    let mut stream = StreamGuard::new(original.context.clone(), Operation::Query, limits).unwrap();
    let mut padded = serde_json::to_value(&original).unwrap();
    padded["future"] = json!("p".repeat(600));
    assert_eq!(
        stream.accept_chunk_bytes(&serialized(&padded)),
        Err(StreamFailure::RetainedLimit)
    );
    assert_eq!(stream.statistics(), Statistics::default());
    padded["future"] = json!("p".repeat(1024));
    assert_eq!(
        stream.accept_chunk_bytes(&serialized(&padded)),
        Err(StreamFailure::MessageLimit)
    );
    limits.retained_bytes = u32::try_from(serialized(&original).len()).unwrap();
    let mut stream = StreamGuard::new(original.context.clone(), Operation::Query, limits).unwrap();
    stream.accept_chunk(&original).unwrap();
    let before = stream.statistics();
    assert_eq!(
        stream.accept_terminal(&query_end(original.context, 1)),
        Err(StreamFailure::RetainedLimit)
    );
    assert!(!stream.is_finished());
    assert_eq!(stream.statistics(), before);
}

#[test]
fn oversized_or_non_json_messages_fail_before_retention() {
    let mut stream = guard(Operation::Query);
    assert_eq!(
        stream.accept_chunk_bytes(&vec![b'x'; 1_048_577]),
        Err(StreamFailure::MessageLimit)
    );
    assert_eq!(
        stream.accept_chunk_bytes(b"not JSON"),
        Err(StreamFailure::InvalidJson)
    );
    assert_eq!(
        stream.accept_chunk_bytes(b"{}\n{}"),
        Err(StreamFailure::InvalidJson)
    );
    assert_eq!(stream.statistics(), Statistics::default());
}

#[test]
fn terminal_kind_and_unique_completion_are_enforced_for_success_and_failure() {
    let mut stream = guard(Operation::Query);
    let context = rows().context;
    let wrong = Terminal {
        context: context.clone(),
        sequence: 0,
        result: TerminalResult::Close {},
    };
    assert_eq!(
        stream.accept_terminal(&wrong),
        Err(StreamFailure::InvalidTerminal)
    );
    stream
        .accept_terminal(&query_end(context.clone(), 0))
        .unwrap();
    assert_eq!(
        stream.finish_failure(&context),
        Err(StreamFailure::Finished)
    );
    assert_eq!(stream.accept_chunk(&rows()), Err(StreamFailure::Finished));
    let mut stream = guard(Operation::Query);
    stream.finish_failure(&context).unwrap();
    assert_eq!(
        stream.accept_terminal(&query_end(context, 0)),
        Err(StreamFailure::Finished)
    );
}

#[test]
fn mongo_presence_shape_and_cumulative_catalogue_items_are_checked() {
    let original: Chunk = serde_json::from_str(MONGO).unwrap();
    let mut limits = super::super::driver_contract::LIMITS;
    limits.catalogue_items = 3;
    let mut stream =
        StreamGuard::new(original.context.clone(), Operation::Introspect, limits).unwrap();
    stream.accept_chunk(&original).unwrap();
    assert_eq!(stream.statistics().catalogue_items, 2);
    let mut another = original.clone();
    another.sequence = 1;
    assert_eq!(
        stream.accept_chunk(&another),
        Err(StreamFailure::CatalogueLimit)
    );
    for presence in [f64::NAN, f64::INFINITY, -0.1, 1.1] {
        let mut wrong = original.clone();
        let ChunkPayload::Catalogue { collections, .. } = &mut wrong.payload else {
            unreachable!()
        };
        collections[0].fields[0].presence = Some(presence);
        assert_eq!(
            guard(Operation::Introspect).accept_chunk(&wrong),
            Err(StreamFailure::InvalidCatalogue)
        );
    }
    let mut mixed = original;
    let ChunkPayload::Catalogue {
        schemas,
        collections,
    } = &mut mixed.payload
    else {
        unreachable!()
    };
    schemas.push(kinein_protocol::DataSourceSchema::default());
    collections[0].statements = Some(kinein_protocol::DataSourceStatements::default());
    assert_eq!(
        guard(Operation::Introspect).accept_chunk(&mixed),
        Err(StreamFailure::InvalidCatalogue)
    );
}

#[test]
fn relational_catalogue_counts_nested_columns_and_refuses_adapter_instructions() {
    let mut chunk = Chunk {
        context: rows().context,
        sequence: 0,
        payload: ChunkPayload::Catalogue {
            schemas: vec![kinein_protocol::DataSourceSchema {
                name: "public".to_owned(),
                tables: vec![kinein_protocol::DataSourceTable {
                    name: "sample".to_owned(),
                    kind: "table".to_owned(),
                    columns: vec![kinein_protocol::DataSourceColumn::default()],
                    ..Default::default()
                }],
            }],
            collections: vec![],
        },
    };
    let mut stream = guard(Operation::Introspect);
    stream.accept_chunk(&chunk).unwrap();
    assert_eq!(stream.statistics().catalogue_items, 3);
    let ChunkPayload::Catalogue { schemas, .. } = &mut chunk.payload else {
        unreachable!()
    };
    schemas[0].tables[0].read_sql = Some("SQL FROM ADAPTER".to_owned());
    assert_eq!(
        guard(Operation::Introspect).accept_chunk(&chunk),
        Err(StreamFailure::InvalidCatalogue)
    );
    let mut mongo: Chunk = serde_json::from_str(MONGO).unwrap();
    mongo.sequence = 1;
    assert_eq!(
        stream.accept_chunk(&mongo),
        Err(StreamFailure::InvalidCatalogue)
    );
}

#[test]
fn catalogue_object_kinds_are_closed_to_known_relational_and_document_shapes() {
    let mut mongo: Chunk = serde_json::from_str(MONGO).unwrap();
    let ChunkPayload::Catalogue { collections, .. } = &mut mongo.payload else {
        unreachable!()
    };
    collections[0].kind = "table".to_owned();
    assert_eq!(
        guard(Operation::Introspect).accept_chunk(&mongo),
        Err(StreamFailure::InvalidCatalogue)
    );
    let relational = Chunk {
        context: rows().context,
        sequence: 0,
        payload: ChunkPayload::Catalogue {
            schemas: vec![kinein_protocol::DataSourceSchema {
                name: "public".to_owned(),
                tables: vec![kinein_protocol::DataSourceTable {
                    name: "sample".to_owned(),
                    kind: "collection".to_owned(),
                    ..Default::default()
                }],
            }],
            collections: vec![],
        },
    };
    assert_eq!(
        guard(Operation::Introspect).accept_chunk(&relational),
        Err(StreamFailure::InvalidCatalogue)
    );
}

#[test]
fn preview_ready_is_once_and_precedes_only_a_coherent_terminal() {
    let mut stream = guard(Operation::Preview);
    stream.accept_chunk(&rows()).unwrap();
    let ready: Chunk = serde_json::from_str(READY).unwrap();
    stream.accept_chunk(&ready).unwrap();
    let mut duplicate = ready.clone();
    duplicate.sequence = 2;
    assert_eq!(
        stream.accept_chunk(&duplicate),
        Err(StreamFailure::InvalidPreview)
    );
    let mut late_rows = rows();
    late_rows.sequence = 2;
    assert_eq!(
        stream.accept_chunk(&late_rows),
        Err(StreamFailure::InvalidPreview)
    );
    let terminal: Terminal = serde_json::from_str(PREVIEW_END).unwrap();
    let mut wrong = terminal.clone();
    let TerminalResult::Preview { preview_id, .. } = &mut wrong.result else {
        unreachable!()
    };
    *preview_id = Some("another".to_owned());
    assert_eq!(
        stream.accept_terminal(&wrong),
        Err(StreamFailure::InvalidPreview)
    );
    stream.accept_terminal(&terminal).unwrap();
    assert!(stream.is_finished());
    let mut premature = terminal;
    premature.sequence = 0;
    let TerminalResult::Preview {
        preview_id,
        outcome,
        ..
    } = &mut premature.result
    else {
        unreachable!()
    };
    *preview_id = None;
    *outcome = DataSourcePreviewOutcome::Committed;
    assert_eq!(
        guard(Operation::Preview).accept_terminal(&premature),
        Err(StreamFailure::InvalidPreview)
    );
}

#[test]
fn preview_ready_sample_count_and_text_cannot_claim_a_complete_false_sample() {
    for (affected, truncated, text) in [
        (0, true, "UPDATE x"),
        (3, false, "UPDATE x"),
        (1, false, " "),
    ] {
        let mut stream = guard(Operation::Preview);
        stream.accept_chunk(&rows()).unwrap();
        let mut ready: Chunk = serde_json::from_str(READY).unwrap();
        let ChunkPayload::PreviewReady {
            affected: total,
            truncated: cut,
            executed_sql,
            ..
        } = &mut ready.payload
        else {
            unreachable!()
        };
        *total = affected;
        *cut = truncated;
        *executed_sql = text.to_owned();
        let before = stream.statistics();
        assert_eq!(
            stream.accept_chunk(&ready),
            Err(StreamFailure::InvalidPreview)
        );
        assert_eq!(stream.statistics(), before);
    }
}

#[test]
fn decision_and_cancel_replies_must_echo_the_exact_requested_target() {
    let context = rows().context;
    let limits = super::super::driver_contract::LIMITS;
    let request = Request::Decide(DecideParams {
        context: context.clone(),
        preview_operation_id: "original".to_owned(),
        preview_id: "preview-1".to_owned(),
        decision: DataSourcePreviewDecision::Rollback,
    });
    let mut stream = StreamGuard::for_request(&request, limits).unwrap();
    let mut terminal = Terminal {
        context: context.clone(),
        sequence: 0,
        result: TerminalResult::Decide {
            preview_operation_id: "wrong".to_owned(),
            preview_id: "preview-1".to_owned(),
            accepted: true,
        },
    };
    assert_eq!(
        stream.accept_terminal(&terminal),
        Err(StreamFailure::InvalidTerminal)
    );
    let TerminalResult::Decide {
        preview_operation_id,
        ..
    } = &mut terminal.result
    else {
        unreachable!()
    };
    *preview_operation_id = "original".to_owned();
    stream.accept_terminal(&terminal).unwrap();
    let request = Request::Cancel(CancelParams {
        context: context.clone(),
        target_operation_id: "original".to_owned(),
    });
    let mut stream = StreamGuard::for_request(&request, limits).unwrap();
    let wrong = Terminal {
        context,
        sequence: 0,
        result: TerminalResult::Cancel {
            target_operation_id: "wrong".to_owned(),
            accepted: true,
        },
    };
    assert_eq!(
        stream.accept_terminal(&wrong),
        Err(StreamFailure::InvalidTerminal)
    );
}

#[test]
fn other_operations_accept_only_their_minimal_terminal_shape() {
    for (operation, result) in [
        (
            Operation::Open,
            TerminalResult::Open {
                session_id: "new".to_owned(),
                server_version: Some("16".to_owned()),
                operations: vec![Operation::Query],
            },
        ),
        (
            Operation::Test,
            TerminalResult::Test {
                server_version: None,
            },
        ),
        (
            Operation::Introspect,
            TerminalResult::Introspect { truncated: true },
        ),
        (
            Operation::Impact,
            TerminalResult::Impact {
                severity: kinein_protocol::SqlImpactSeverity::Read,
                statements: vec![],
            },
        ),
        (Operation::Close, TerminalResult::Close {}),
        (Operation::Shutdown, TerminalResult::Shutdown {}),
    ] {
        let mut stream = guard(operation);
        let terminal = Terminal {
            context: stream.context.clone(),
            sequence: 0,
            result,
        };
        stream
            .accept_terminal_bytes(&serialized(&terminal))
            .unwrap();
        assert!(stream.is_finished());
    }
}

#[test]
fn invalid_session_budget_and_payload_kind_are_refused() {
    let mut context = rows().context;
    context.session_id = None;
    assert!(matches!(
        StreamGuard::new(
            context,
            Operation::Query,
            super::super::driver_contract::LIMITS
        ),
        Err(StreamFailure::InvalidContext)
    ));
    let mut limits = super::super::driver_contract::LIMITS;
    limits.rows += 1;
    assert!(matches!(
        StreamGuard::new(rows().context, Operation::Query, limits),
        Err(StreamFailure::InvalidLimits)
    ));
    assert_eq!(
        guard(Operation::Test).accept_chunk(&rows()),
        Err(StreamFailure::UnexpectedPayload)
    );
    let mut ready: Chunk = serde_json::from_str(READY).unwrap();
    ready.sequence = 0;
    let ChunkPayload::PreviewReady {
        expires_in_seconds, ..
    } = &mut ready.payload
    else {
        unreachable!()
    };
    *expires_in_seconds = 61;
    assert_eq!(
        guard(Operation::Preview).accept_chunk(&ready),
        Err(StreamFailure::InvalidPreview)
    );
}

//! Core dispatch basics: ping, command list, error handling and the stdio loop.

use std::io::{BufReader, Cursor};

use serde_json::{Value, json};

use crate::{Core, RequestOutcome, run_json_lines};
use kernwerk_protocol::JsonRpcRequest;

#[test]
fn ping_returns_pong() {
    let mut core = Core::new();
    let request = JsonRpcRequest::new(1_i64, "core.ping", Some(json!({})));
    let outcome = core.handle_request(&request);
    let result = outcome.response().result.as_ref().unwrap();

    assert!(!outcome.should_shutdown());
    assert_eq!(result["status"], "ok");
    assert_eq!(result["message"], "pong");
}

#[test]
fn unknown_command_returns_structured_error() {
    let mut core = Core::new();
    let request = JsonRpcRequest::new(9_i64, "missing.command", Some(json!({})));
    let outcome = core.handle_request(&request);
    let error = outcome.response().error.as_ref().unwrap();

    assert_eq!(
        error.code,
        kernwerk_protocol::JsonRpcErrorCode::MethodNotFound
    );
    assert_eq!(error.details.as_ref().unwrap()["method"], "missing.command");
}

#[test]
fn command_list_includes_lsp_navigation_commands() {
    let mut core = Core::new();
    let request = JsonRpcRequest::new(8_i64, "command.list", Some(json!({})));
    let outcome = core.handle_request(&request);
    let commands = outcome.response().result.as_ref().unwrap()["commands"]
        .as_array()
        .unwrap();
    let ids = commands
        .iter()
        .filter_map(|command| command["id"].as_str())
        .collect::<Vec<_>>();

    assert!(ids.contains(&"workspace.createFolder"));
    assert!(ids.contains(&"workspace.createProject"));
    assert!(ids.contains(&"fs.createFile"));
    assert!(ids.contains(&"fs.createDirectory"));
    assert!(ids.contains(&"fs.rename"));
    assert!(ids.contains(&"fs.delete"));
    assert!(ids.contains(&"fs.findFiles"));
    assert!(ids.contains(&"lsp.didChange"));
    assert!(ids.contains(&"lsp.definition"));
    assert!(ids.contains(&"lsp.hover"));
    assert!(ids.contains(&"lsp.completion"));
    assert!(ids.contains(&"lsp.references"));
    assert!(ids.contains(&"lsp.rename"));
}

#[test]
fn invalid_json_returns_parse_error() {
    let mut core = Core::new();
    let outcome = core.handle_json_line("{not-json");
    let error = outcome.response().error.as_ref().unwrap();

    assert_eq!(error.code, kernwerk_protocol::JsonRpcErrorCode::ParseError);
}

#[test]
fn shutdown_outcome_is_explicit() {
    let mut core = Core::new();
    let request = JsonRpcRequest::new(2_i64, "core.shutdown", Some(json!({})));
    let outcome = core.handle_request(&request);

    assert!(matches!(outcome, RequestOutcome::Shutdown(_)));
    assert!(outcome.should_shutdown());
}

#[test]
fn stdio_loop_writes_one_response_per_line_and_stops() {
    let input = concat!(
        r#"{"jsonrpc":"2.0","id":1,"method":"core.ping","params":{}}"#,
        "\n",
        r#"{"jsonrpc":"2.0","id":2,"method":"core.shutdown","params":{}}"#,
        "\n",
        r#"{"jsonrpc":"2.0","id":3,"method":"core.ping","params":{}}"#,
        "\n",
    );
    let reader = BufReader::new(Cursor::new(input.as_bytes()));
    let mut output = Vec::new();

    run_json_lines(reader, &mut output).unwrap();

    let text = String::from_utf8(output).unwrap();
    let responses = text
        .lines()
        .map(serde_json::from_str::<Value>)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    assert_eq!(responses.len(), 2);
    assert_eq!(responses[0]["result"]["message"], "pong");
    assert_eq!(responses[1]["result"]["message"], "shutdown requested");
}

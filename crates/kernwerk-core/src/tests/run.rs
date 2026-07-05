//! User process dispatch (`run.*`).

use serde_json::json;

use super::core_with_empty_search_path;
use kernwerk_protocol::JsonRpcRequest;

#[test]
fn run_start_requires_workspace_and_enabled_manager() {
    let mut core = core_with_empty_search_path("run-no-workspace");
    let denied = core.handle_request(&JsonRpcRequest::new(40_i64, "run.start", None));
    assert!(denied.response().error.is_some());

    let dir = std::env::temp_dir()
        .join("kernwerk-core-tests")
        .join(format!("{}-run-start", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let opened = core.handle_request(&JsonRpcRequest::new(
        41_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none());

    let unavailable = core.handle_request(&JsonRpcRequest::new(42_i64, "run.start", None));
    let error = unavailable.response().error.as_ref().unwrap();
    assert_eq!(
        error.code,
        kernwerk_protocol::JsonRpcErrorCode::InternalError
    );
}

#[test]
fn run_start_executes_command_and_emits_events() {
    let (sender, receiver) = std::sync::mpsc::channel();
    let mut core = core_with_empty_search_path("run-e2e");
    core.enable_lsp(sender);

    let dir = std::env::temp_dir()
        .join("kernwerk-core-tests")
        .join(format!("{}-run-e2e", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let opened = core.handle_request(&JsonRpcRequest::new(
        43_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none());

    let no_default = core.handle_request(&JsonRpcRequest::new(44_i64, "run.start", None));
    assert!(no_default.response().error.is_some());

    let started = core.handle_request(&JsonRpcRequest::new(
        45_i64,
        "run.start",
        Some(json!({ "command": "printf 'executado\\n'" })),
    ));
    let result = started.response().result.as_ref().unwrap();
    assert_eq!(result["command"], "printf 'executado\\n'");

    let mut saw_output = false;
    loop {
        let event = receiver
            .recv_timeout(std::time::Duration::from_secs(5))
            .expect("evento run dentro do timeout");
        if event.method == "event.run.output" {
            saw_output = event.params.as_ref().unwrap()["line"] == "executado";
        }
        if event.method == "event.run.finished" {
            assert_eq!(event.params.as_ref().unwrap()["success"], true);
            break;
        }
    }
    assert!(saw_output);

    let stopped = core.handle_request(&JsonRpcRequest::new(46_i64, "run.stop", None));
    assert!(stopped.response().error.is_some());
}

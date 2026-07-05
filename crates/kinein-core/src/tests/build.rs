//! Build runner dispatch (`build.run`), including its streamed events.

use serde_json::json;

use super::core_with_empty_search_path;
use kinein_protocol::JsonRpcRequest;

#[test]
fn build_run_requires_open_workspace() {
    let mut core = core_with_empty_search_path("build-no-workspace");
    let request = JsonRpcRequest::new(30_i64, "build.run", Some(json!({})));
    let outcome = core.handle_request_streaming(&request, &mut |_| {});
    let error = outcome.response().error.as_ref().unwrap();

    assert_eq!(
        error.code,
        kinein_protocol::JsonRpcErrorCode::InvalidRequest
    );
    assert_eq!(error.message, "nenhum workspace aberto");
}

#[test]
fn build_run_rejects_unsupported_project_kind() {
    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-build-unsupported", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("pyproject.toml"), "[project]\n").unwrap();
    let mut core = core_with_empty_search_path("build-unsupported");

    let opened = core.handle_request(&JsonRpcRequest::new(
        31_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none());

    let mut events = Vec::new();
    let request = JsonRpcRequest::new(32_i64, "build.run", Some(json!({})));
    let outcome = core.handle_request_streaming(&request, &mut |notification| {
        events.push(notification.method.clone());
    });
    let error = outcome.response().error.as_ref().unwrap();

    assert_eq!(
        error.code,
        kinein_protocol::JsonRpcErrorCode::InvalidRequest
    );
    assert!(error.message.contains("python"));
    assert!(events.is_empty());
}

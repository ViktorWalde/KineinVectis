//! Tool detection dispatch (`tools.detect`, `tools.status`).

use serde_json::json;

use super::core_with_empty_search_path;
use kernwerk_protocol::JsonRpcRequest;

#[test]
fn tools_detect_returns_structured_status_for_all_known_tools() {
    let mut core = core_with_empty_search_path("detect");
    let request = JsonRpcRequest::new(4_i64, "tools.detect", Some(json!({})));
    let outcome = core.handle_request(&request);
    let result = outcome.response().result.as_ref().unwrap();
    let tools = result["tools"].as_array().unwrap();

    assert_eq!(tools.len(), crate::tools::KNOWN_TOOLS.len());
    assert!(tools.iter().all(|tool| tool["status"] == "missing"));
    assert!(tools[0].get("suggestedInstall").is_none());
}

#[test]
fn tools_status_runs_detection_once_and_reuses_registry() {
    let mut core = core_with_empty_search_path("status");
    let first = core.handle_request(&JsonRpcRequest::new(5_i64, "tools.status", Some(json!({}))));
    let second = core.handle_request(&JsonRpcRequest::new(6_i64, "tools.status", Some(json!({}))));

    assert_eq!(
        first.response().result.as_ref().unwrap()["tools"],
        second.response().result.as_ref().unwrap()["tools"]
    );
}

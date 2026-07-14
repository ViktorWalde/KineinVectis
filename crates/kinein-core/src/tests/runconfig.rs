//! Run configuration dispatch (`runConfig.*`) e integracao com `run.start`.

use serde_json::json;

use super::core_with_empty_search_path;
use kinein_protocol::JsonRpcRequest;

#[test]
fn runconfig_crud_flows_through_dispatch() {
    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-runconfig-crud", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("Cargo.toml"), "[package]\n").unwrap();
    let mut core = core_with_empty_search_path("runconfig-crud");

    let no_workspace = core.handle_request(&JsonRpcRequest::new(100_i64, "runConfig.list", None));
    assert_eq!(
        no_workspace.response().error.as_ref().unwrap().code,
        kinein_protocol::JsonRpcErrorCode::InvalidRequest
    );

    let opened = core.handle_request(&JsonRpcRequest::new(
        101_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none());

    let saved = core.handle_request(&JsonRpcRequest::new(
        102_i64,
        "runConfig.save",
        Some(json!({ "name": "Echo", "command": "echo oi" })),
    ));
    let result = saved.response().result.as_ref().unwrap().clone();
    assert_eq!(result["configs"][0]["id"], "cfg-1");
    assert_eq!(result["activeId"], "cfg-1");

    let cleared = core.handle_request(&JsonRpcRequest::new(
        103_i64,
        "runConfig.setActive",
        Some(json!({})),
    ));
    assert!(
        cleared
            .response()
            .result
            .as_ref()
            .unwrap()
            .get("activeId")
            .is_none()
    );

    let invalid = core.handle_request(&JsonRpcRequest::new(
        104_i64,
        "runConfig.setActive",
        Some(json!({ "id": "cfg-9" })),
    ));
    assert_eq!(
        invalid.response().error.as_ref().unwrap().code,
        kinein_protocol::JsonRpcErrorCode::InvalidParams
    );

    let deleted = core.handle_request(&JsonRpcRequest::new(
        105_i64,
        "runConfig.delete",
        Some(json!({ "id": "cfg-1" })),
    ));
    assert!(
        deleted.response().result.as_ref().unwrap()["configs"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}

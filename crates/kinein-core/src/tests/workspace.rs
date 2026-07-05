//! Workspace lifecycle dispatch (`workspace.*`).

use serde_json::json;

use super::core_with_empty_search_path;
use kinein_protocol::JsonRpcRequest;

#[test]
fn workspace_open_status_close_cycle_works() {
    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-workspace-cycle", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("Cargo.toml"), "[package]\n").unwrap();
    let mut core = core_with_empty_search_path("workspace-cycle");

    let opened = core.handle_request(&JsonRpcRequest::new(
        10_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    let opened_result = opened.response().result.as_ref().unwrap();
    assert_eq!(opened_result["kind"], "rustCargo");

    let status = core.handle_request(&JsonRpcRequest::new(
        11_i64,
        "workspace.status",
        Some(json!({})),
    ));
    let status_result = status.response().result.as_ref().unwrap();
    assert_eq!(status_result["workspace"]["kind"], "rustCargo");

    let closed = core.handle_request(&JsonRpcRequest::new(
        12_i64,
        "workspace.close",
        Some(json!({})),
    ));
    let closed_result = closed.response().result.as_ref().unwrap();
    assert_eq!(closed_result["status"], "ok");
    assert!(closed_result["closed"].is_string());

    let after = core.handle_request(&JsonRpcRequest::new(
        13_i64,
        "workspace.status",
        Some(json!({})),
    ));
    let after_result = after.response().result.as_ref().unwrap();
    assert!(after_result["workspace"].is_null());
}

#[test]
fn workspace_open_without_path_returns_invalid_params() {
    let mut core = core_with_empty_search_path("workspace-bad-params");
    let outcome = core.handle_request(&JsonRpcRequest::new(
        14_i64,
        "workspace.open",
        Some(json!({})),
    ));
    let error = outcome.response().error.as_ref().unwrap();

    assert_eq!(error.code, kinein_protocol::JsonRpcErrorCode::InvalidParams);
}

#[test]
fn workspace_browse_returns_directory_entries() {
    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-workspace-browse", std::process::id()));
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("Cargo.toml"), "[package]\n").unwrap();
    let mut core = core_with_empty_search_path("workspace-browse");

    let outcome = core.handle_request(&JsonRpcRequest::new(
        16_i64,
        "workspace.browse",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    let result = outcome.response().result.as_ref().unwrap();
    let entries = result["entries"].as_array().unwrap();

    assert_eq!(
        result["path"],
        dir.canonicalize().unwrap().display().to_string()
    );
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0]["name"], "src");
}

#[test]
fn workspace_browse_without_path_returns_invalid_params() {
    let mut core = core_with_empty_search_path("workspace-browse-bad-params");
    let outcome = core.handle_request(&JsonRpcRequest::new(
        17_i64,
        "workspace.browse",
        Some(json!({})),
    ));
    let error = outcome.response().error.as_ref().unwrap();

    assert_eq!(error.code, kinein_protocol::JsonRpcErrorCode::InvalidParams);
}

#[test]
fn workspace_create_folder_creates_directory() {
    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-workspace-create-folder", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let mut core = core_with_empty_search_path("workspace-create-folder");

    let outcome = core.handle_request(&JsonRpcRequest::new(
        18_i64,
        "workspace.createFolder",
        Some(json!({ "parent": dir.to_str().unwrap(), "name": "novo modulo" })),
    ));
    let result = outcome.response().result.as_ref().unwrap();
    let created = std::path::PathBuf::from(result["path"].as_str().unwrap());

    assert!(created.is_dir());
    assert_eq!(created.file_name().unwrap(), "novo modulo");
}

#[test]
fn workspace_create_project_opens_created_cpp_project() {
    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-workspace-create-project", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let mut core = core_with_empty_search_path("workspace-create-project");

    let outcome = core.handle_request(&JsonRpcRequest::new(
        19_i64,
        "workspace.createProject",
        Some(json!({
            "parent": dir.to_str().unwrap(),
            "name": "demo_cpp",
            "template": "cppCmake",
        })),
    ));
    let result = outcome.response().result.as_ref().unwrap();

    assert_eq!(result["kind"], "cmake");
    assert_eq!(result["name"], "demo_cpp");
    assert!(dir.join("demo_cpp/CMakeLists.txt").is_file());
    assert!(dir.join("demo_cpp/src/main.cpp").is_file());
    assert_eq!(
        core.handle_request(&JsonRpcRequest::new(
            20_i64,
            "workspace.status",
            Some(json!({}))
        ))
        .response()
        .result
        .as_ref()
        .unwrap()["workspace"]["name"],
        "demo_cpp"
    );
}

#[test]
fn workspace_open_with_missing_directory_returns_invalid_params() {
    let mut core = core_with_empty_search_path("workspace-missing-dir");
    let outcome = core.handle_request(&JsonRpcRequest::new(
        15_i64,
        "workspace.open",
        Some(json!({ "path": "/definitely/not/a/real/path" })),
    ));
    let error = outcome.response().error.as_ref().unwrap();

    assert_eq!(error.code, kinein_protocol::JsonRpcErrorCode::InvalidParams);
    assert_eq!(
        error.details.as_ref().unwrap()["path"],
        "/definitely/not/a/real/path"
    );
}

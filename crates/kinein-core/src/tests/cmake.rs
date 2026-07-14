//! `CMake` service dispatch (`cmake.*`).

use serde_json::json;

use super::core_with_empty_search_path;
use kinein_protocol::JsonRpcRequest;

fn cmake_workspace(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-cmake-{name}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("CMakeLists.txt"),
        "cmake_minimum_required(VERSION 3.20)\nproject(demo CXX)\n",
    )
    .unwrap();
    dir
}

#[test]
fn cmake_methods_require_workspace_and_cmake_kind() {
    let mut core = core_with_empty_search_path("cmake-guards");
    for method in [
        "cmake.configure",
        "cmake.presets.list",
        "cmake.targets.list",
        "cmake.status",
    ] {
        let outcome = core.handle_request(&JsonRpcRequest::new(80_i64, method, None));
        let error = outcome.response().error.as_ref().unwrap();
        assert_eq!(
            error.code,
            kinein_protocol::JsonRpcErrorCode::InvalidRequest,
            "{method}"
        );
    }

    let rust_dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-cmake-rust", std::process::id()));
    std::fs::create_dir_all(&rust_dir).unwrap();
    std::fs::write(rust_dir.join("Cargo.toml"), "[package]\n").unwrap();
    let opened = core.handle_request(&JsonRpcRequest::new(
        81_i64,
        "workspace.open",
        Some(json!({ "path": rust_dir.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none());

    let wrong_kind = core.handle_request(&JsonRpcRequest::new(82_i64, "cmake.status", None));
    let error = wrong_kind.response().error.as_ref().unwrap();
    assert_eq!(error.code, kinein_protocol::JsonRpcErrorCode::InvalidParams);
    assert!(error.message.contains("workspace CMake"));
}

#[test]
fn cmake_status_and_presets_work_without_jobs() {
    let dir = cmake_workspace("status");
    std::fs::write(
        dir.join("CMakePresets.json"),
        r#"{ "version": 6, "configurePresets": [ { "name": "release" } ] }"#,
    )
    .unwrap();
    let mut core = core_with_empty_search_path("cmake-status");
    let opened = core.handle_request(&JsonRpcRequest::new(
        83_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none());

    let status = core.handle_request(&JsonRpcRequest::new(84_i64, "cmake.status", None));
    let result = status.response().result.as_ref().unwrap().clone();
    assert_eq!(result["configured"], false);
    assert_eq!(result["hasCompileCommands"], false);
    assert!(
        result["buildDir"]
            .as_str()
            .unwrap()
            .ends_with(".kinein/build")
    );

    let presets = core.handle_request(&JsonRpcRequest::new(85_i64, "cmake.presets.list", None));
    let listed = presets.response().result.as_ref().unwrap()["presets"]
        .as_array()
        .unwrap()
        .clone();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0]["name"], "release");

    let targets = core.handle_request(&JsonRpcRequest::new(86_i64, "cmake.targets.list", None));
    assert!(
        targets.response().result.as_ref().unwrap()["targets"]
            .as_array()
            .unwrap()
            .is_empty()
    );

    let configure = core.handle_request(&JsonRpcRequest::new(87_i64, "cmake.configure", None));
    let error = configure.response().error.as_ref().unwrap();
    assert_eq!(error.code, kinein_protocol::JsonRpcErrorCode::InternalError);
    assert!(error.message.contains("jobs"));
}

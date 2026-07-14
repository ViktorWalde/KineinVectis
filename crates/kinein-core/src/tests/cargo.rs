//! Cargo service dispatch (`cargo.*`).

use serde_json::json;

use super::core_with_empty_search_path;
use kinein_protocol::JsonRpcRequest;

#[test]
fn cargo_methods_require_workspace_and_rust_kind() {
    let mut core = core_with_empty_search_path("cargo-guards");
    for method in ["cargo.metadata", "cargo.check"] {
        let outcome = core.handle_request(&JsonRpcRequest::new(90_i64, method, None));
        let error = outcome.response().error.as_ref().unwrap();
        assert_eq!(
            error.code,
            kinein_protocol::JsonRpcErrorCode::InvalidRequest,
            "{method}"
        );
    }

    let cmake_dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-cargo-cmake", std::process::id()));
    std::fs::create_dir_all(&cmake_dir).unwrap();
    std::fs::write(
        cmake_dir.join("CMakeLists.txt"),
        "cmake_minimum_required(VERSION 3.20)\nproject(x)\n",
    )
    .unwrap();
    let opened = core.handle_request(&JsonRpcRequest::new(
        91_i64,
        "workspace.open",
        Some(json!({ "path": cmake_dir.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none());

    let wrong_kind = core.handle_request(&JsonRpcRequest::new(92_i64, "cargo.metadata", None));
    let error = wrong_kind.response().error.as_ref().unwrap();
    assert_eq!(error.code, kinein_protocol::JsonRpcErrorCode::InvalidParams);
    assert!(error.message.contains("Rust/Cargo"));
}

#[test]
fn cargo_check_requires_jobs_enabled() {
    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-cargo-check", std::process::id()));
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    std::fs::write(dir.join("src/main.rs"), "fn main() {}\n").unwrap();
    let mut core = core_with_empty_search_path("cargo-check");
    let opened = core.handle_request(&JsonRpcRequest::new(
        93_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none());

    let outcome = core.handle_request(&JsonRpcRequest::new(94_i64, "cargo.check", None));
    let error = outcome.response().error.as_ref().unwrap();
    assert_eq!(error.code, kinein_protocol::JsonRpcErrorCode::InternalError);
    assert!(error.message.contains("jobs"));
}

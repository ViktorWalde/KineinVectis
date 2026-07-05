//! Language-server dispatch (`lsp.*`), covering the no-workspace and
//! no-manager guard paths (real servers are exercised elsewhere).

use serde_json::json;

use super::core_with_empty_search_path;
use kinein_protocol::JsonRpcRequest;

#[test]
fn lsp_did_change_requires_open_workspace() {
    let mut core = core_with_empty_search_path("lsp-no-workspace");
    let request = JsonRpcRequest::new(
        33_i64,
        "lsp.didChange",
        Some(json!({ "path": "/tmp/main.rs", "content": "fn main() {}\n" })),
    );
    let outcome = core.handle_request(&request);
    let error = outcome.response().error.as_ref().unwrap();

    assert_eq!(
        error.code,
        kinein_protocol::JsonRpcErrorCode::InvalidRequest
    );
    assert_eq!(error.message, "nenhum workspace aberto");
}

#[test]
fn lsp_navigation_requires_open_workspace() {
    let mut core = core_with_empty_search_path("lsp-navigation-no-workspace");
    for method in [
        "lsp.definition",
        "lsp.hover",
        "lsp.completion",
        "lsp.references",
    ] {
        let request = JsonRpcRequest::new(
            36_i64,
            method,
            Some(json!({
                "path": "/tmp/main.rs",
                "content": "fn main() {}\n",
                "line": 1,
                "column": 1,
            })),
        );
        let outcome = core.handle_request(&request);
        let error = outcome.response().error.as_ref().unwrap();

        assert_eq!(
            error.code,
            kinein_protocol::JsonRpcErrorCode::InvalidRequest
        );
        assert_eq!(error.message, "nenhum workspace aberto");
    }
}

#[test]
fn lsp_rename_requires_open_workspace_and_non_empty_name() {
    let mut core = core_with_empty_search_path("lsp-rename-no-workspace");
    let request = JsonRpcRequest::new(
        40_i64,
        "lsp.rename",
        Some(json!({
            "path": "/tmp/main.rs",
            "content": "fn main() {}\n",
            "line": 1,
            "column": 4,
            "newName": "start",
        })),
    );
    let outcome = core.handle_request(&request);
    let error = outcome.response().error.as_ref().unwrap();
    assert_eq!(
        error.code,
        kinein_protocol::JsonRpcErrorCode::InvalidRequest
    );
    assert_eq!(error.message, "nenhum workspace aberto");

    let workspace = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-lsp-rename-empty-name", std::process::id()));
    std::fs::create_dir_all(workspace.join("src")).unwrap();
    std::fs::write(workspace.join("Cargo.toml"), "[package]\n").unwrap();
    std::fs::write(workspace.join("src/main.rs"), "fn main() {}\n").unwrap();
    let opened = core.handle_request(&JsonRpcRequest::new(
        41_i64,
        "workspace.open",
        Some(json!({ "path": workspace.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none());

    let empty_name = core.handle_request(&JsonRpcRequest::new(
        42_i64,
        "lsp.rename",
        Some(json!({
            "path": workspace.join("src/main.rs").to_str().unwrap(),
            "content": "fn main() {}\n",
            "line": 1,
            "column": 4,
            "newName": "   ",
        })),
    ));
    let empty_error = empty_name.response().error.as_ref().unwrap();
    assert_eq!(
        empty_error.code,
        kinein_protocol::JsonRpcErrorCode::InvalidParams
    );
}

#[test]
fn lsp_navigation_reports_unavailable_without_lsp_manager() {
    let workspace = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-lsp-unavailable", std::process::id()));
    std::fs::create_dir_all(workspace.join("src")).unwrap();
    std::fs::write(workspace.join("Cargo.toml"), "[package]\n").unwrap();
    std::fs::write(workspace.join("src/main.rs"), "fn main() {}\n").unwrap();
    let mut core = core_with_empty_search_path("lsp-unavailable");

    let opened = core.handle_request(&JsonRpcRequest::new(
        37_i64,
        "workspace.open",
        Some(json!({ "path": workspace.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none());

    let file_path = workspace.join("src/main.rs");
    let outcome = core.handle_request(&JsonRpcRequest::new(
        38_i64,
        "lsp.definition",
        Some(json!({
            "path": file_path.to_str().unwrap(),
            "content": "fn main() {}\n",
            "line": 1,
            "column": 1,
        })),
    ));
    let error = outcome.response().error.as_ref().unwrap();

    assert_eq!(error.code, kinein_protocol::JsonRpcErrorCode::InternalError);
    assert_eq!(error.message, "LSP nao esta habilitado neste loop do core");
}

#[test]
fn lsp_did_change_rejects_paths_outside_workspace() {
    let workspace = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-lsp-workspace", std::process::id()));
    let outside = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-lsp-outside", std::process::id()));
    std::fs::create_dir_all(&workspace).unwrap();
    std::fs::create_dir_all(&outside).unwrap();
    std::fs::write(workspace.join("Cargo.toml"), "[package]\n").unwrap();
    std::fs::write(outside.join("main.rs"), "fn main() {}\n").unwrap();
    let mut core = core_with_empty_search_path("lsp-outside");

    let opened = core.handle_request(&JsonRpcRequest::new(
        34_i64,
        "workspace.open",
        Some(json!({ "path": workspace.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none());

    let outcome = core.handle_request(&JsonRpcRequest::new(
        35_i64,
        "lsp.didChange",
        Some(json!({
            "path": outside.join("main.rs").to_str().unwrap(),
            "content": "fn main() {}\n",
        })),
    ));
    let error = outcome.response().error.as_ref().unwrap();

    assert_eq!(error.code, kinein_protocol::JsonRpcErrorCode::InvalidParams);
}

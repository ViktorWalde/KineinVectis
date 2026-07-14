//! Incremental syntax-tree IPC dispatch.

use kinein_protocol::{JsonRpcRequest, SyntaxTreeSnapshotResult};
use serde_json::json;

use super::core_with_empty_search_path;

#[test]
fn syntax_update_returns_structural_layers_without_lsp() {
    let root = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-syntax-update", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join("main.rs");
    std::fs::write(&path, "fn main() {}\n").unwrap();
    let mut core = core_with_empty_search_path("syntax-update");
    let opened = core.handle_request(&JsonRpcRequest::new(
        801_i64,
        "workspace.open",
        Some(json!({ "path": root })),
    ));
    assert!(opened.response().error.is_none());

    let response = core.handle_request(&JsonRpcRequest::new(
        802_i64,
        "syntaxTree.update",
        Some(json!({
            "path": path,
            "content": "fn main() {\n    let state = 1;\n}\n",
            "version": 4
        })),
    ));
    let snapshot = serde_json::from_value::<SyntaxTreeSnapshotResult>(
        response.response().result.clone().unwrap(),
    )
    .unwrap();

    assert_eq!(snapshot.language, "rust");
    assert_eq!(snapshot.version, 4);
    assert!(snapshot.outline.iter().any(|item| item.name == "main"));
    assert!(!snapshot.folding_ranges.is_empty());
    assert!(!snapshot.highlights.is_empty());
}

#[test]
fn syntax_update_rejects_paths_outside_workspace() {
    let root = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-syntax-confine", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    let mut core = core_with_empty_search_path("syntax-confine");
    let _ = core.handle_request(&JsonRpcRequest::new(
        803_i64,
        "workspace.open",
        Some(json!({ "path": root })),
    ));

    let response = core.handle_request(&JsonRpcRequest::new(
        804_i64,
        "syntaxTree.update",
        Some(json!({
            "path": "/etc/hosts",
            "content": "127.0.0.1 localhost\n",
            "version": 1
        })),
    ));

    assert_eq!(
        response.response().error.as_ref().unwrap().code,
        kinein_protocol::JsonRpcErrorCode::InvalidParams
    );
}

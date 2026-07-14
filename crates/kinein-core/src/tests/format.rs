//! Buffer formatting dispatch (`format.text`).

use serde_json::json;

use super::core_with_empty_search_path;
use kinein_protocol::{JsonRpcErrorCode, JsonRpcRequest};

fn workspace_dir(test_name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-format-{test_name}", std::process::id()));
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("Cargo.toml"), "[package]\n").unwrap();
    std::fs::write(dir.join("src/main.rs"), "fn main() {}\n").unwrap();
    std::fs::write(dir.join("notas.txt"), "texto\n").unwrap();
    dir
}

fn open_workspace(core: &mut crate::Core, dir: &std::path::Path) -> String {
    let opened = core.handle_request(&JsonRpcRequest::new(
        1_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    opened.response().result.as_ref().unwrap()["root"]
        .as_str()
        .unwrap()
        .to_owned()
}

#[test]
fn format_text_requires_open_workspace() {
    let mut core = core_with_empty_search_path("format-no-workspace");
    let outcome = core.handle_request(&JsonRpcRequest::new(
        2_i64,
        "format.text",
        Some(json!({ "path": "/tmp/a.rs", "text": "fn main(){}" })),
    ));
    let error = outcome.response().error.as_ref().unwrap();
    assert_eq!(error.code, JsonRpcErrorCode::InvalidRequest);
    assert_eq!(error.message, "nenhum workspace aberto");
}

#[test]
fn format_text_formats_a_rust_buffer_with_rustfmt() {
    let dir = workspace_dir("rustfmt");
    let mut core = core_with_empty_search_path("format-rustfmt");
    let root = open_workspace(&mut core, &dir);

    let outcome = core.handle_request(&JsonRpcRequest::new(
        3_i64,
        "format.text",
        Some(json!({
            "path": format!("{root}/src/main.rs"),
            "text": "fn main(   ){ let x=1;println!(\"{x}\") ; }",
        })),
    ));
    let result = outcome.response().result.as_ref().unwrap().clone();
    assert_eq!(result["formatter"], "rustfmt");
    assert_eq!(result["changed"], true);
    assert!(
        result["path"].as_str().unwrap().ends_with("src/main.rs"),
        "path ecoado: {}",
        result["path"]
    );
    let text = result["text"].as_str().unwrap();
    assert!(text.contains("fn main() {"), "texto formatado: {text}");
    assert!(text.contains("let x = 1;"), "texto formatado: {text}");
}

#[test]
fn format_text_reports_unchanged_buffers() {
    let dir = workspace_dir("unchanged");
    let mut core = core_with_empty_search_path("format-unchanged");
    let root = open_workspace(&mut core, &dir);

    let outcome = core.handle_request(&JsonRpcRequest::new(
        4_i64,
        "format.text",
        Some(json!({
            "path": format!("{root}/src/main.rs"),
            "text": "fn main() {}\n",
        })),
    ));
    let result = outcome.response().result.as_ref().unwrap().clone();
    assert_eq!(result["changed"], false);
    assert_eq!(result["text"], "fn main() {}\n");
}

#[test]
fn format_text_rejects_extensions_without_formatter() {
    let dir = workspace_dir("unsupported");
    let mut core = core_with_empty_search_path("format-unsupported");
    let root = open_workspace(&mut core, &dir);

    let outcome = core.handle_request(&JsonRpcRequest::new(
        5_i64,
        "format.text",
        Some(json!({ "path": format!("{root}/notas.txt"), "text": "texto" })),
    ));
    let error = outcome.response().error.as_ref().unwrap();
    assert_eq!(error.code, JsonRpcErrorCode::InvalidParams);
    assert!(error.message.contains("nenhum formatter"));
}

#[test]
fn format_text_rejects_paths_outside_workspace() {
    let dir = workspace_dir("outside");
    let mut core = core_with_empty_search_path("format-outside");
    let _root = open_workspace(&mut core, &dir);

    let outcome = core.handle_request(&JsonRpcRequest::new(
        6_i64,
        "format.text",
        Some(json!({ "path": "/etc/hostname", "text": "x" })),
    ));
    let error = outcome.response().error.as_ref().unwrap();
    assert_eq!(error.code, JsonRpcErrorCode::InvalidParams);
}

#[test]
fn command_list_includes_format_text() {
    let mut core = core_with_empty_search_path("format-command-list");
    let outcome = core.handle_request(&JsonRpcRequest::new(7_i64, "command.list", None));
    let commands = outcome.response().result.as_ref().unwrap()["commands"]
        .as_array()
        .unwrap()
        .clone();
    let format_command = commands
        .iter()
        .find(|command| command["id"] == "format.text")
        .expect("format.text deveria estar em command.list");
    assert_eq!(format_command["defaultShortcut"], "Ctrl+Alt+L");
    assert_eq!(format_command["requiresWorkspace"], true);
}

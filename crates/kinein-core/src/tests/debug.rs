//! Debug dispatch (`debug.*`): guardas, confinamento e store de breakpoints.

use serde_json::json;

use super::core_with_empty_search_path;
use kinein_protocol::{JsonRpcErrorCode, JsonRpcRequest};

#[test]
fn debug_methods_require_workspace_or_manager() {
    let mut core = core_with_empty_search_path("debug-guards");

    let start = core.handle_request(&JsonRpcRequest::new(1_i64, "debug.start", None));
    assert_eq!(
        start.response().error.as_ref().unwrap().code,
        JsonRpcErrorCode::InvalidRequest
    );
    let breakpoints = core.handle_request(&JsonRpcRequest::new(
        2_i64,
        "debug.setBreakpoints",
        Some(json!({ "file": "/x/main.c", "lines": [1] })),
    ));
    assert_eq!(
        breakpoints.response().error.as_ref().unwrap().code,
        JsonRpcErrorCode::InvalidRequest
    );

    // Sem enable_lsp nao ha manager: controles de sessao indisponiveis.
    let resume = core.handle_request(&JsonRpcRequest::new(3_i64, "debug.continue", None));
    assert_eq!(
        resume.response().error.as_ref().unwrap().code,
        JsonRpcErrorCode::InternalError
    );
}

#[test]
fn breakpoint_flow_and_session_guards_through_dispatch() {
    let base = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-debug-flow", std::process::id()));
    let root = base.join("ws");
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("Cargo.toml"), "[package]\n").unwrap();
    std::fs::write(root.join("main.rs"), "fn main() {}\n").unwrap();
    std::fs::write(base.join("fora.rs"), "fn main() {}\n").unwrap();

    let (sender, _receiver) = std::sync::mpsc::channel();
    let mut core = core_with_empty_search_path("debug-flow");
    core.enable_lsp(sender);
    let opened = core.handle_request(&JsonRpcRequest::new(
        10_i64,
        "workspace.open",
        Some(json!({ "path": root.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none());

    let outside = core.handle_request(&JsonRpcRequest::new(
        11_i64,
        "debug.setBreakpoints",
        Some(json!({ "file": base.join("fora.rs").to_str().unwrap(), "lines": [1] })),
    ));
    assert_eq!(
        outside.response().error.as_ref().unwrap().code,
        JsonRpcErrorCode::InvalidParams
    );

    let missing = core.handle_request(&JsonRpcRequest::new(
        12_i64,
        "debug.setBreakpoints",
        Some(json!({ "file": root.join("nao-existe.rs").to_str().unwrap(), "lines": [1] })),
    ));
    assert_eq!(
        missing.response().error.as_ref().unwrap().code,
        JsonRpcErrorCode::InvalidParams
    );

    let stored = core.handle_request(&JsonRpcRequest::new(
        13_i64,
        "debug.setBreakpoints",
        Some(json!({ "file": root.join("main.rs").to_str().unwrap(), "lines": [7, 3, 7] })),
    ));
    let result = stored.response().result.as_ref().unwrap().clone();
    assert_eq!(result["breakpoints"][0]["line"], 3);
    assert_eq!(result["breakpoints"][1]["line"], 7);
    assert_eq!(result["breakpoints"][0]["verified"], false);

    let cleared = core.handle_request(&JsonRpcRequest::new(
        14_i64,
        "debug.setBreakpoints",
        Some(json!({ "file": root.join("main.rs").to_str().unwrap(), "lines": [] })),
    ));
    let result = cleared.response().result.as_ref().unwrap().clone();
    assert_eq!(result["breakpoints"].as_array().unwrap().len(), 0);

    // Nenhuma sessao viva: controles respondem estado invalido, nao panico.
    let resume = core.handle_request(&JsonRpcRequest::new(15_i64, "debug.continue", None));
    assert_eq!(
        resume.response().error.as_ref().unwrap().code,
        JsonRpcErrorCode::InvalidRequest
    );

    let ghost_program = core.handle_request(&JsonRpcRequest::new(
        16_i64,
        "debug.start",
        Some(json!({ "program": root.join("fantasma").to_str().unwrap() })),
    ));
    assert_eq!(
        ghost_program.response().error.as_ref().unwrap().code,
        JsonRpcErrorCode::InvalidParams
    );

    // Automatico num cargo sem binario compilado: erro claro, nao spawn.
    let no_target = core.handle_request(&JsonRpcRequest::new(17_i64, "debug.start", None));
    let error = no_target.response().error.as_ref().unwrap().clone();
    assert_eq!(error.code, JsonRpcErrorCode::InvalidRequest);
    assert!(error.message.contains("compile antes"));
}

#[test]
fn inspection_methods_validate_params_and_session_state() {
    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-debug-inspect", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let (sender, _receiver) = std::sync::mpsc::channel();
    let mut core = core_with_empty_search_path("debug-inspect");
    core.enable_lsp(sender);
    let opened = core.handle_request(&JsonRpcRequest::new(
        30_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none());

    // Sem sessao viva: INVALID_REQUEST com a mensagem de dominio.
    let stack = core.handle_request(&JsonRpcRequest::new(31_i64, "debug.stackTrace", None));
    assert_eq!(
        stack.response().error.as_ref().unwrap().code,
        JsonRpcErrorCode::InvalidRequest
    );

    // frameId e ref juntos (ou ausentes) e INVALID_PARAMS antes de tudo.
    let both = core.handle_request(&JsonRpcRequest::new(
        32_i64,
        "debug.variables",
        Some(json!({ "frameId": 1, "ref": 2 })),
    ));
    assert_eq!(
        both.response().error.as_ref().unwrap().code,
        JsonRpcErrorCode::InvalidParams
    );
    let neither = core.handle_request(&JsonRpcRequest::new(33_i64, "debug.variables", None));
    assert_eq!(
        neither.response().error.as_ref().unwrap().code,
        JsonRpcErrorCode::InvalidParams
    );

    let by_ref = core.handle_request(&JsonRpcRequest::new(
        34_i64,
        "debug.variables",
        Some(json!({ "ref": 5 })),
    ));
    assert_eq!(
        by_ref.response().error.as_ref().unwrap().code,
        JsonRpcErrorCode::InvalidRequest
    );
}

#[test]
fn command_list_includes_debug_start() {
    let mut core = core_with_empty_search_path("debug-command-list");
    let outcome = core.handle_request(&JsonRpcRequest::new(20_i64, "command.list", None));
    let result = outcome.response().result.as_ref().unwrap().clone();
    let has_debug_start = result["commands"]
        .as_array()
        .unwrap()
        .iter()
        .any(|command| command["id"] == "debug.start");
    assert!(has_debug_start);
}

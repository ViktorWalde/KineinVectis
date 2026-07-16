//! Settings dispatch: get read-only, set workspace (isolado em `.kinein`),
//! validacao de fonte e guarda de workspace. O escopo GLOBAL escreve em
//! `~/.config` real, entao so e exercitado na sonda e2e (subprocesso com
//! `XDG_CONFIG_HOME` proprio) — nunca nos testes em processo.

use serde_json::json;

use super::core_with_empty_search_path;
use kinein_protocol::{JsonRpcErrorCode, JsonRpcRequest};

fn temp_workspace(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-settings-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn settings_get_returns_effective_global_and_workspace() {
    let mut core = core_with_empty_search_path("settings-get");
    let outcome = core.handle_request(&JsonRpcRequest::new(1_i64, "settings.get", None));
    let result = outcome.response().result.as_ref().unwrap().clone();
    // Estrutura presente e efetivo com fonte num range plausivel (o valor
    // exato depende do ~/.config real, entao nao afirmamos igualdade).
    assert!(result["settings"]["editorFontSize"].as_u64().unwrap() >= 8);
    assert!(result.get("global").is_some());
    assert_eq!(result["workspace"], json!({}));
}

#[test]
fn settings_set_workspace_overrides_and_persists() {
    let dir = temp_workspace("set-workspace");
    let mut core = core_with_empty_search_path("settings-set-workspace");
    let opened = core.handle_request(&JsonRpcRequest::new(
        10_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none());

    // Set parcial no workspace: sobrepoe o efetivo, independente do global.
    let set = core.handle_request(&JsonRpcRequest::new(
        11_i64,
        "settings.set",
        Some(json!({
            "scope": "workspace",
            "values": {
                "editorFontSize": 20,
                "autoClosePairs": false,
                "explorerWidth": 320,
                "contextWidth": 400,
                "assistantTerminalWidth": 680,
                "bottomPanelHeight": 300,
                "outlineWidth": 240,
                "outlineCollapsed": true,
            },
        })),
    ));
    let result = set.response().result.as_ref().unwrap().clone();
    assert_eq!(result["settings"]["editorFontSize"], 20);
    assert_eq!(result["settings"]["autoClosePairs"], false);
    assert_eq!(result["settings"]["explorerWidth"], 320);
    assert_eq!(result["settings"]["contextWidth"], 400);
    assert_eq!(result["settings"]["assistantTerminalWidth"], 680);
    assert_eq!(result["settings"]["bottomPanelHeight"], 300);
    assert_eq!(result["settings"]["outlineWidth"], 240);
    assert_eq!(result["settings"]["outlineCollapsed"], true);
    assert_eq!(result["workspace"]["editorFontSize"], 20);

    // Segundo set parcial NAO apaga o campo anterior (merge).
    let merge = core.handle_request(&JsonRpcRequest::new(
        12_i64,
        "settings.set",
        Some(json!({ "scope": "workspace", "values": { "formatOnSave": true } })),
    ));
    let merged = merge.response().result.as_ref().unwrap().clone();
    assert_eq!(merged["settings"]["editorFontSize"], 20);
    assert_eq!(merged["settings"]["formatOnSave"], true);
    assert_eq!(merged["settings"]["autoClosePairs"], false);

    // Persistiu em disco: um core novo no mesmo workspace le o override.
    let mut reopened = core_with_empty_search_path("settings-reopen");
    let _ = reopened.handle_request(&JsonRpcRequest::new(
        13_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    let reget = reopened.handle_request(&JsonRpcRequest::new(14_i64, "settings.get", None));
    let reread = reget.response().result.as_ref().unwrap().clone();
    assert_eq!(reread["settings"]["editorFontSize"], 20);
    assert_eq!(reread["settings"]["formatOnSave"], true);
    assert_eq!(reread["settings"]["outlineWidth"], 240);
    assert_eq!(reread["settings"]["assistantTerminalWidth"], 680);
    assert_eq!(reread["settings"]["outlineCollapsed"], true);
}

#[test]
fn settings_set_rejects_out_of_range_font() {
    let dir = temp_workspace("font-range");
    let mut core = core_with_empty_search_path("settings-font-range");
    let _ = core.handle_request(&JsonRpcRequest::new(
        20_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));

    for size in [5, 100] {
        let bad = core.handle_request(&JsonRpcRequest::new(
            21_i64,
            "settings.set",
            Some(json!({ "scope": "workspace", "values": { "editorFontSize": size } })),
        ));
        assert_eq!(
            bad.response().error.as_ref().unwrap().code,
            JsonRpcErrorCode::InvalidParams
        );
    }
}

#[test]
fn settings_set_rejects_out_of_range_layout_dimensions() {
    let dir = temp_workspace("layout-range");
    let mut core = core_with_empty_search_path("settings-layout-range");
    let _ = core.handle_request(&JsonRpcRequest::new(
        25_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));

    for values in [
        json!({ "explorerWidth": 100 }),
        json!({ "contextWidth": 900 }),
        json!({ "assistantTerminalWidth": 200 }),
        json!({ "assistantTerminalWidth": 900 }),
        json!({ "outlineWidth": 80 }),
        json!({ "bottomPanelHeight": 900 }),
    ] {
        let bad = core.handle_request(&JsonRpcRequest::new(
            26_i64,
            "settings.set",
            Some(json!({ "scope": "workspace", "values": values })),
        ));
        assert_eq!(
            bad.response().error.as_ref().unwrap().code,
            JsonRpcErrorCode::InvalidParams
        );
    }
}

#[test]
fn settings_set_workspace_requires_open_workspace() {
    let mut core = core_with_empty_search_path("settings-no-workspace");
    let no_workspace = core.handle_request(&JsonRpcRequest::new(
        30_i64,
        "settings.set",
        Some(json!({ "scope": "workspace", "values": { "formatOnSave": true } })),
    ));
    assert_eq!(
        no_workspace.response().error.as_ref().unwrap().code,
        JsonRpcErrorCode::InvalidRequest
    );
}

#[test]
fn command_list_includes_settings() {
    let mut core = core_with_empty_search_path("settings-command-list");
    let outcome = core.handle_request(&JsonRpcRequest::new(40_i64, "command.list", None));
    let result = outcome.response().result.as_ref().unwrap().clone();
    let found = result["commands"]
        .as_array()
        .unwrap()
        .iter()
        .any(|command| command["id"] == "settings.get");
    assert!(found);
}

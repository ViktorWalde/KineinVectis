//! Exact console identity and public context through the real dispatcher.
use super::core_with_empty_search_path;
use kinein_protocol::{JsonRpcErrorCode, JsonRpcRequest};
use serde_json::{Value, json};

fn request(
    core: &mut crate::Core,
    method: &str,
    params: Value,
) -> kinein_protocol::JsonRpcResponse {
    core.handle_request(&JsonRpcRequest::new(1_i64, method, Some(params)))
        .response()
        .clone()
}

fn fixture(label: &str) -> (crate::Core, std::path::PathBuf, Value, String) {
    let root =
        std::env::temp_dir().join(format!("kinein-console-rpc-{label}-{}", std::process::id()));
    drop(std::fs::remove_dir_all(&root));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("Cargo.toml"), "[package]\n").unwrap();
    let mut core = core_with_empty_search_path(&format!("console-{label}"));
    assert!(
        request(&mut core, "workspace.open", json!({"path": root}))
            .error
            .is_none()
    );
    let result = request(&mut core, "datasource.save", json!({"profile": {
        "name": "loja", "engine": "sqlite", "database": root.join("a.db"), "host": "", "port": 0, "user": ""
    }})).result.unwrap();
    let context = json!({"workspace": root, "profile": result["profiles"][0]});
    assert_eq!(result["workspace"], root.display().to_string());
    let created = request(
        &mut core,
        "datasource.console",
        json!({
            "name": "loja", "clientContext": "console.0:1", "expectedContext": context
        }),
    )
    .result
    .unwrap();
    assert_eq!(created["clientContext"], "console.0:1");
    assert_eq!(created["expectedContext"], context);
    let path = created["path"].as_str().unwrap().to_owned();
    assert!(
        result["consoleBindings"][0]["paths"]
            .as_array()
            .unwrap()
            .contains(&json!(path))
    );
    (core, root, context, path)
}

#[test]
fn a_statement_is_resolved_without_execution_and_respects_utf16() {
    let (mut core, root, context, path) = fixture("utf16");
    let text = "-- header\nSELECT '😀;\n\nDELETE FROM t';\nSELECT 2;";
    let cursor = text[..text.find("DELETE").unwrap()].encode_utf16().count();
    let params = json!({"name": "loja", "path": path, "text": text, "cursor": cursor,
        "selectionStart": cursor, "selectionEnd": cursor,
        "clientContext": "console.0:2", "expectedContext": context});
    let resolved = request(&mut core, "datasource.console.statement", params.clone())
        .result
        .unwrap();
    assert_eq!(resolved["statement"], "SELECT '😀;\n\nDELETE FROM t'");
    assert_eq!(resolved["expectedContext"], context);
    assert_eq!(resolved["clientContext"], "console.0:2");
    assert!(
        !root.join("a.db").exists(),
        "resolver não abre/cria o banco"
    );
    let mut malformed = params.clone();
    malformed["cursor"] = json!(text[..text.find('😀').unwrap()].encode_utf16().count() + 1);
    assert_eq!(
        request(&mut core, "datasource.console.statement", malformed)
            .error
            .unwrap()
            .code,
        JsonRpcErrorCode::InvalidParams
    );
    for extra in ["password", "confirmWrite", "preview"] {
        let mut malformed = params.clone();
        malformed[extra] = json!("proibido");
        assert_eq!(
            request(&mut core, "datasource.console.statement", malformed)
                .error
                .unwrap()
                .code,
            JsonRpcErrorCode::InvalidParams
        );
    }
    let mut malformed = params;
    malformed.as_object_mut().unwrap().remove("expectedContext");
    assert_eq!(
        request(&mut core, "datasource.console.statement", malformed)
            .error
            .unwrap()
            .code,
        JsonRpcErrorCode::InvalidParams
    );
    drop(std::fs::remove_dir_all(root));
}

#[test]
fn stale_destinations_and_wrong_paths_are_refused_before_extracting() {
    let (mut core, root, context, path) = fixture("context");
    let params = json!({"name": "loja", "path": path, "text": "SELECT 1;", "cursor": 0,
        "selectionStart": 0, "selectionEnd": 0, "clientContext": "console.0:2", "expectedContext": context});
    let mut changed = params.clone();
    changed["expectedContext"]["profile"]["production"] = json!(true);
    let rejected = request(&mut core, "datasource.console.statement", changed)
        .error
        .unwrap();
    assert_eq!(rejected.code, JsonRpcErrorCode::DataSourceContextChanged);
    assert_eq!(rejected.details.unwrap()["clientContext"], "console.0:2");
    for wrong in [
        format!("{path}/child.sql"),
        format!("{path}.bak"),
        root.join(".kinein/consoles/../datasources.json")
            .display()
            .to_string(),
    ] {
        let mut changed = params.clone();
        changed["path"] = json!(wrong);
        assert_eq!(
            request(&mut core, "datasource.console.statement", changed)
                .error
                .unwrap()
                .code,
            JsonRpcErrorCode::InvalidParams
        );
    }
    let mut profile = context["profile"].clone();
    profile["database"] = json!(root.join("other.db"));
    assert!(
        request(&mut core, "datasource.save", json!({"profile": profile}))
            .error
            .is_none()
    );
    assert_eq!(
        request(&mut core, "datasource.console.statement", params)
            .error
            .unwrap()
            .code,
        JsonRpcErrorCode::DataSourceContextChanged
    );
    let rejected = request(
        &mut core,
        "datasource.console",
        json!({"name": "loja", "expectedContext": context, "clientContext": "console.0:3"}),
    );
    assert_eq!(
        rejected.error.unwrap().code,
        JsonRpcErrorCode::DataSourceContextChanged
    );
    drop(std::fs::remove_dir_all(root));
}

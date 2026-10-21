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

#[test]
fn invalid_runconfig_storage_is_refused_without_replacing_the_original_bytes() {
    for (name, contents) in [
        ("broken", "{not-json"),
        (
            "future",
            r#"{"schemaVersion":99,"configs":[{"id":"cfg-1","name":"Old","command":"echo old"}]}"#,
        ),
        (
            "dangling",
            r#"{"schemaVersion":1,"activeId":"cfg-9","configs":[]}"#,
        ),
        (
            "unsupported-entry",
            r#"{"schemaVersion":1,"configs":[{"id":"cfg-1","name":"Native","command":"echo preview","flash":{"device":"board"}}]}"#,
        ),
        (
            "duplicates",
            r#"{"schemaVersion":1,"configs":[{"id":"cfg-1","name":"A","command":"echo a"},{"id":"cfg-1","name":"B","command":"echo b"}]}"#,
        ),
    ] {
        let dir = std::env::temp_dir()
            .join("kinein-core-tests")
            .join(format!("{}-runconfig-preserve-{name}", std::process::id()));
        std::fs::create_dir_all(dir.join(".kinein")).unwrap();
        let path = dir.join(".kinein/runconfigs.json");
        std::fs::write(&path, contents).unwrap();
        let mut core = core_with_empty_search_path(&format!("runconfig-preserve-{name}"));
        assert!(
            core.handle_request(&JsonRpcRequest::new(
                1_i64,
                "workspace.open",
                Some(json!({"path":dir}))
            ))
            .response()
            .error
            .is_none()
        );
        for (method, params) in [
            ("runConfig.list", json!({})),
            ("runConfig.save", json!({"name":"New","command":"echo new"})),
            ("runConfig.delete", json!({"id":"cfg-1"})),
            ("runConfig.setActive", json!({})),
        ] {
            let response = core.handle_request(&JsonRpcRequest::new(2_i64, method, Some(params)));
            let error = response
                .response()
                .error
                .as_ref()
                .expect("invalid storage must not become an empty configuration list");
            assert_eq!(
                error.code,
                kinein_protocol::JsonRpcErrorCode::InvalidRequest,
                "{name} {method}: {error:?}"
            );
            assert!(error.message.contains("runconfigs.json"), "{error:?}");
            assert_eq!(
                std::fs::read_to_string(&path).unwrap(),
                contents,
                "{name} {method} replaced user data"
            );
        }
    }
}

#[test]
fn saving_must_not_silently_replace_a_corrupt_runconfig_file() {
    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-runconfig-save-preserve", std::process::id()));
    std::fs::create_dir_all(dir.join(".kinein")).unwrap();
    let path = dir.join(".kinein/runconfigs.json");
    let original = "{user data requiring repair";
    std::fs::write(&path, original).unwrap();
    let response = crate::runconfig::save(&dir, None, "New", "echo new");
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        original,
        "save must preserve an unreadable configuration file"
    );
    assert!(response.is_err());
}

#[test]
fn default_run_refuses_invalid_config_storage_but_explicit_command_remains_available() {
    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-runconfig-default-preserve", std::process::id()));
    std::fs::create_dir_all(dir.join(".kinein")).unwrap();
    let path = dir.join(".kinein/runconfigs.json");
    std::fs::write(&path, "{broken active intent").unwrap();
    let mut core = core_with_empty_search_path("runconfig-default-preserve");
    assert!(
        core.handle_request(&JsonRpcRequest::new(
            1_i64,
            "workspace.open",
            Some(json!({"path":dir}))
        ))
        .response()
        .error
        .is_none()
    );
    let refused = core.handle_request(&JsonRpcRequest::new(2_i64, "run.start", Some(json!({}))));
    let error = refused.response().error.as_ref().unwrap();
    assert_eq!(
        error.code,
        kinein_protocol::JsonRpcErrorCode::InvalidRequest
    );
    assert!(error.message.contains("runconfigs.json"), "{error:?}");
    let accepted = core.handle_request(&JsonRpcRequest::new(
        3_i64,
        "run.start",
        Some(json!({"command":"echo manual-recovery"})),
    ));
    let terminal = accepted.response().error.as_ref().unwrap();
    assert!(
        !terminal.message.contains("runconfigs.json"),
        "explicit command tried to load broken configuration: {terminal:?}"
    );
    assert!(
        terminal.message.to_lowercase().contains("terminal"),
        "explicit command did not reach the executor: {terminal:?}"
    );
    assert_eq!(
        std::fs::read_to_string(path).unwrap(),
        "{broken active intent"
    );
}

#[test]
fn configuration_action_checks_storage_between_preview_and_apply() {
    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-runconfig-action-preserve", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"fixture\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    let mut core = core_with_empty_search_path("runconfig-action-preserve");
    assert!(
        core.handle_request(&JsonRpcRequest::new(
            1_i64,
            "workspace.open",
            Some(json!({"path": dir})),
        ))
        .response()
        .error
        .is_none()
    );
    let params =
        json!({"id":"cargo.createRunConfig","params":{"name":"Fixture","command":"cargo run"}});
    let preview = core.handle_request(&JsonRpcRequest::new(
        2_i64,
        "configAction.preview",
        Some(params.clone()),
    ));
    assert!(
        preview.response().error.is_none(),
        "{:?}",
        preview.response()
    );
    let path = dir.join(".kinein/runconfigs.json");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let original = "{storage changed after preview";
    std::fs::write(&path, original).unwrap();
    for method in ["configAction.apply", "configAction.preview"] {
        let refused =
            core.handle_request(&JsonRpcRequest::new(3_i64, method, Some(params.clone())));
        let error = refused
            .response()
            .error
            .as_ref()
            .expect("the action must not replace unreadable intent");
        assert!(error.message.contains("runconfigs.json"), "{error:?}");
        assert_eq!(std::fs::read_to_string(&path).unwrap(), original);
    }
}

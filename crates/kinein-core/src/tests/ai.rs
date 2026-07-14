use kinein_protocol::{AiCliProfileId, JsonRpcErrorCode, JsonRpcRequest};
use serde_json::json;

use super::core_with_empty_search_path;

#[cfg(unix)]
use crate::{Core, tools::ToolDetector};

#[test]
fn profiles_are_deterministic_and_expose_saved_default() {
    let mut core = core_with_empty_search_path("ai-profiles");
    let request = JsonRpcRequest::new(1_i64, "aiBridge.profiles", Some(json!({})));
    let outcome = core.handle_request(&request);
    let result = outcome.response().result.as_ref().unwrap();
    let profiles = result["profiles"].as_array().unwrap();

    assert!(matches!(
        result["defaultProfile"].as_str(),
        Some("claude" | "codex")
    ));
    assert_eq!(profiles.len(), 2);
    assert_eq!(profiles[0]["id"], "claude");
    assert_eq!(profiles[0]["available"], false);
    assert_eq!(profiles[1]["id"], "codex");
}

#[test]
fn opening_ai_terminal_requires_workspace() {
    let mut core = core_with_empty_search_path("ai-no-workspace");
    let request = JsonRpcRequest::new(
        2_i64,
        "aiBridge.terminal.open",
        Some(json!({ "profileId": AiCliProfileId::Claude })),
    );
    let outcome = core.handle_request(&request);

    assert_eq!(
        outcome.response().error.as_ref().map(|error| error.code),
        Some(JsonRpcErrorCode::InvalidRequest)
    );
}

#[cfg(unix)]
#[test]
fn installed_profile_starts_in_the_existing_terminal_manager() {
    use std::os::unix::fs::PermissionsExt;

    let root = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-ai-terminal", std::process::id()));
    let bin = root.join("bin");
    let workspace = root.join("workspace");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&bin).unwrap();
    std::fs::create_dir_all(&workspace).unwrap();
    let executable = bin.join("claude");
    std::fs::write(&executable, "#!/bin/sh\nexec /bin/sleep 30\n").unwrap();
    let mut permissions = std::fs::metadata(&executable).unwrap().permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(&executable, permissions).unwrap();

    let (events, _notifications) = std::sync::mpsc::channel();
    let mut core = Core::with_detector(ToolDetector::with_search_path(&bin));
    core.enable_lsp(events);
    let opened_workspace = core.handle_request(&JsonRpcRequest::new(
        10_i64,
        "workspace.open",
        Some(json!({ "path": workspace.to_str().unwrap() })),
    ));
    assert!(opened_workspace.response().error.is_none());

    let profiles = core.handle_request(&JsonRpcRequest::new(
        11_i64,
        "aiBridge.profiles",
        Some(json!({})),
    ));
    assert_eq!(
        profiles.response().result.as_ref().unwrap()["profiles"][0]["available"],
        true
    );

    let opened = core.handle_request(&JsonRpcRequest::new(
        12_i64,
        "aiBridge.terminal.open",
        Some(json!({ "profileId": "claude" })),
    ));
    let result = opened.response().result.as_ref().unwrap();
    assert_eq!(result["profileId"], "claude");
    assert_eq!(result["command"], executable.to_string_lossy().as_ref());
    let id = result["id"].as_str().unwrap().to_owned();

    let closed = core.handle_request(&JsonRpcRequest::new(
        13_i64,
        "terminal.close",
        Some(json!({ "id": id })),
    ));
    assert!(closed.response().error.is_none());
    let _ = core.handle_request(&JsonRpcRequest::new(
        14_i64,
        "workspace.close",
        Some(json!({})),
    ));
}

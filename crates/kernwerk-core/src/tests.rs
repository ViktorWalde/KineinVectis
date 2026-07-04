// Unit tests for Core request dispatch and the stdio loop.
// Split out of lib.rs; `super` still refers to the crate root.

use std::io::{BufReader, Cursor};

use serde_json::{Value, json};

use super::{Core, RequestOutcome, run_json_lines};
use crate::tools::ToolDetector;
use kernwerk_protocol::JsonRpcRequest;

fn core_with_empty_search_path(test_name: &str) -> Core {
    let dir = std::env::temp_dir()
        .join("kernwerk-core-tests")
        .join(format!("{}-dispatch-{test_name}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    Core::with_detector(ToolDetector::with_search_path(dir))
}

#[test]
fn ping_returns_pong() {
    let mut core = Core::new();
    let request = JsonRpcRequest::new(1_i64, "core.ping", Some(json!({})));
    let outcome = core.handle_request(&request);
    let result = outcome.response().result.as_ref().unwrap();

    assert!(!outcome.should_shutdown());
    assert_eq!(result["status"], "ok");
    assert_eq!(result["message"], "pong");
}

#[test]
fn unknown_command_returns_structured_error() {
    let mut core = Core::new();
    let request = JsonRpcRequest::new(9_i64, "missing.command", Some(json!({})));
    let outcome = core.handle_request(&request);
    let error = outcome.response().error.as_ref().unwrap();

    assert_eq!(
        error.code,
        kernwerk_protocol::JsonRpcErrorCode::MethodNotFound
    );
    assert_eq!(error.details.as_ref().unwrap()["method"], "missing.command");
}

#[test]
fn command_list_includes_lsp_navigation_commands() {
    let mut core = Core::new();
    let request = JsonRpcRequest::new(8_i64, "command.list", Some(json!({})));
    let outcome = core.handle_request(&request);
    let commands = outcome.response().result.as_ref().unwrap()["commands"]
        .as_array()
        .unwrap();
    let ids = commands
        .iter()
        .filter_map(|command| command["id"].as_str())
        .collect::<Vec<_>>();

    assert!(ids.contains(&"workspace.createFolder"));
    assert!(ids.contains(&"workspace.createProject"));
    assert!(ids.contains(&"fs.createFile"));
    assert!(ids.contains(&"fs.createDirectory"));
    assert!(ids.contains(&"fs.rename"));
    assert!(ids.contains(&"fs.delete"));
    assert!(ids.contains(&"fs.findFiles"));
    assert!(ids.contains(&"lsp.didChange"));
    assert!(ids.contains(&"lsp.definition"));
    assert!(ids.contains(&"lsp.hover"));
    assert!(ids.contains(&"lsp.completion"));
    assert!(ids.contains(&"lsp.references"));
    assert!(ids.contains(&"lsp.rename"));
}

#[test]
fn invalid_json_returns_parse_error() {
    let mut core = Core::new();
    let outcome = core.handle_json_line("{not-json");
    let error = outcome.response().error.as_ref().unwrap();

    assert_eq!(error.code, kernwerk_protocol::JsonRpcErrorCode::ParseError);
}

#[test]
fn shutdown_outcome_is_explicit() {
    let mut core = Core::new();
    let request = JsonRpcRequest::new(2_i64, "core.shutdown", Some(json!({})));
    let outcome = core.handle_request(&request);

    assert!(matches!(outcome, RequestOutcome::Shutdown(_)));
    assert!(outcome.should_shutdown());
}

#[test]
fn tools_detect_returns_structured_status_for_all_known_tools() {
    let mut core = core_with_empty_search_path("detect");
    let request = JsonRpcRequest::new(4_i64, "tools.detect", Some(json!({})));
    let outcome = core.handle_request(&request);
    let result = outcome.response().result.as_ref().unwrap();
    let tools = result["tools"].as_array().unwrap();

    assert_eq!(tools.len(), crate::tools::KNOWN_TOOLS.len());
    assert!(tools.iter().all(|tool| tool["status"] == "missing"));
    assert!(tools[0].get("suggestedInstall").is_none());
}

#[test]
fn tools_status_runs_detection_once_and_reuses_registry() {
    let mut core = core_with_empty_search_path("status");
    let first = core.handle_request(&JsonRpcRequest::new(5_i64, "tools.status", Some(json!({}))));
    let second = core.handle_request(&JsonRpcRequest::new(6_i64, "tools.status", Some(json!({}))));

    assert_eq!(
        first.response().result.as_ref().unwrap()["tools"],
        second.response().result.as_ref().unwrap()["tools"]
    );
}

#[test]
fn workspace_open_status_close_cycle_works() {
    let dir = std::env::temp_dir()
        .join("kernwerk-core-tests")
        .join(format!("{}-workspace-cycle", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("Cargo.toml"), "[package]\n").unwrap();
    let mut core = core_with_empty_search_path("workspace-cycle");

    let opened = core.handle_request(&JsonRpcRequest::new(
        10_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    let opened_result = opened.response().result.as_ref().unwrap();
    assert_eq!(opened_result["kind"], "rustCargo");

    let status = core.handle_request(&JsonRpcRequest::new(
        11_i64,
        "workspace.status",
        Some(json!({})),
    ));
    let status_result = status.response().result.as_ref().unwrap();
    assert_eq!(status_result["workspace"]["kind"], "rustCargo");

    let closed = core.handle_request(&JsonRpcRequest::new(
        12_i64,
        "workspace.close",
        Some(json!({})),
    ));
    let closed_result = closed.response().result.as_ref().unwrap();
    assert_eq!(closed_result["status"], "ok");
    assert!(closed_result["closed"].is_string());

    let after = core.handle_request(&JsonRpcRequest::new(
        13_i64,
        "workspace.status",
        Some(json!({})),
    ));
    let after_result = after.response().result.as_ref().unwrap();
    assert!(after_result["workspace"].is_null());
}

#[test]
fn workspace_open_without_path_returns_invalid_params() {
    let mut core = core_with_empty_search_path("workspace-bad-params");
    let outcome = core.handle_request(&JsonRpcRequest::new(
        14_i64,
        "workspace.open",
        Some(json!({})),
    ));
    let error = outcome.response().error.as_ref().unwrap();

    assert_eq!(
        error.code,
        kernwerk_protocol::JsonRpcErrorCode::InvalidParams
    );
}

#[test]
fn workspace_browse_returns_directory_entries() {
    let dir = std::env::temp_dir()
        .join("kernwerk-core-tests")
        .join(format!("{}-workspace-browse", std::process::id()));
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("Cargo.toml"), "[package]\n").unwrap();
    let mut core = core_with_empty_search_path("workspace-browse");

    let outcome = core.handle_request(&JsonRpcRequest::new(
        16_i64,
        "workspace.browse",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    let result = outcome.response().result.as_ref().unwrap();
    let entries = result["entries"].as_array().unwrap();

    assert_eq!(
        result["path"],
        dir.canonicalize().unwrap().display().to_string()
    );
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0]["name"], "src");
}

#[test]
fn workspace_browse_without_path_returns_invalid_params() {
    let mut core = core_with_empty_search_path("workspace-browse-bad-params");
    let outcome = core.handle_request(&JsonRpcRequest::new(
        17_i64,
        "workspace.browse",
        Some(json!({})),
    ));
    let error = outcome.response().error.as_ref().unwrap();

    assert_eq!(
        error.code,
        kernwerk_protocol::JsonRpcErrorCode::InvalidParams
    );
}

#[test]
fn workspace_create_folder_creates_directory() {
    let dir = std::env::temp_dir()
        .join("kernwerk-core-tests")
        .join(format!("{}-workspace-create-folder", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let mut core = core_with_empty_search_path("workspace-create-folder");

    let outcome = core.handle_request(&JsonRpcRequest::new(
        18_i64,
        "workspace.createFolder",
        Some(json!({ "parent": dir.to_str().unwrap(), "name": "novo modulo" })),
    ));
    let result = outcome.response().result.as_ref().unwrap();
    let created = std::path::PathBuf::from(result["path"].as_str().unwrap());

    assert!(created.is_dir());
    assert_eq!(created.file_name().unwrap(), "novo modulo");
}

#[test]
fn workspace_create_project_opens_created_cpp_project() {
    let dir = std::env::temp_dir()
        .join("kernwerk-core-tests")
        .join(format!("{}-workspace-create-project", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let mut core = core_with_empty_search_path("workspace-create-project");

    let outcome = core.handle_request(&JsonRpcRequest::new(
        19_i64,
        "workspace.createProject",
        Some(json!({
            "parent": dir.to_str().unwrap(),
            "name": "demo_cpp",
            "template": "cppCmake",
        })),
    ));
    let result = outcome.response().result.as_ref().unwrap();

    assert_eq!(result["kind"], "cmake");
    assert_eq!(result["name"], "demo_cpp");
    assert!(dir.join("demo_cpp/CMakeLists.txt").is_file());
    assert!(dir.join("demo_cpp/src/main.cpp").is_file());
    assert_eq!(
        core.handle_request(&JsonRpcRequest::new(
            20_i64,
            "workspace.status",
            Some(json!({}))
        ))
        .response()
        .result
        .as_ref()
        .unwrap()["workspace"]["name"],
        "demo_cpp"
    );
}

#[test]
fn workspace_open_with_missing_directory_returns_invalid_params() {
    let mut core = core_with_empty_search_path("workspace-missing-dir");
    let outcome = core.handle_request(&JsonRpcRequest::new(
        15_i64,
        "workspace.open",
        Some(json!({ "path": "/definitely/not/a/real/path" })),
    ));
    let error = outcome.response().error.as_ref().unwrap();

    assert_eq!(
        error.code,
        kernwerk_protocol::JsonRpcErrorCode::InvalidParams
    );
    assert_eq!(
        error.details.as_ref().unwrap()["path"],
        "/definitely/not/a/real/path"
    );
}

#[test]
fn fs_methods_require_open_workspace() {
    let mut core = core_with_empty_search_path("fs-no-workspace");
    let outcome = core.handle_request(&JsonRpcRequest::new(
        20_i64,
        "fs.list",
        Some(json!({ "path": "/tmp" })),
    ));
    let error = outcome.response().error.as_ref().unwrap();

    assert_eq!(
        error.code,
        kernwerk_protocol::JsonRpcErrorCode::InvalidRequest
    );
    assert_eq!(error.message, "nenhum workspace aberto");
}

#[test]
fn fs_list_read_write_cycle_inside_workspace() {
    let dir = std::env::temp_dir()
        .join("kernwerk-core-tests")
        .join(format!("{}-fs-cycle", std::process::id()));
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("Cargo.toml"), "[package]\n").unwrap();
    std::fs::write(dir.join("src/main.rs"), "fn main() {}\n").unwrap();
    let mut core = core_with_empty_search_path("fs-cycle");

    let opened = core.handle_request(&JsonRpcRequest::new(
        21_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    let root = opened.response().result.as_ref().unwrap()["root"]
        .as_str()
        .unwrap()
        .to_owned();

    let listed = core.handle_request(&JsonRpcRequest::new(
        22_i64,
        "fs.list",
        Some(json!({ "path": root })),
    ));
    let entries = listed.response().result.as_ref().unwrap()["entries"]
        .as_array()
        .unwrap()
        .clone();
    assert_eq!(entries[0]["name"], ".kernwerk");
    assert_eq!(entries[1]["name"], "src");
    assert_eq!(entries[1]["kind"], "directory");

    let file_path = format!("{root}/src/main.rs");
    let read = core.handle_request(&JsonRpcRequest::new(
        23_i64,
        "fs.read",
        Some(json!({ "path": file_path })),
    ));
    assert_eq!(
        read.response().result.as_ref().unwrap()["content"],
        "fn main() {}\n"
    );

    let written = core.handle_request(&JsonRpcRequest::new(
        24_i64,
        "fs.write",
        Some(json!({ "path": file_path, "content": "// editado\n" })),
    ));
    assert_eq!(
        written.response().result.as_ref().unwrap()["bytesWritten"],
        11
    );

    let escape = core.handle_request(&JsonRpcRequest::new(
        25_i64,
        "fs.read",
        Some(json!({ "path": "/etc/hostname" })),
    ));
    let error = escape.response().error.as_ref().unwrap();
    assert_eq!(
        error.code,
        kernwerk_protocol::JsonRpcErrorCode::InvalidParams
    );
}

#[test]
fn fs_create_file_creates_once_and_stays_inside_workspace() {
    let dir = std::env::temp_dir()
        .join("kernwerk-core-tests")
        .join(format!("{}-fs-create-file", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("Cargo.toml"), "[package]\n").unwrap();
    let mut core = core_with_empty_search_path("fs-create-file");

    let opened = core.handle_request(&JsonRpcRequest::new(
        30_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    let root = opened.response().result.as_ref().unwrap()["root"]
        .as_str()
        .unwrap()
        .to_owned();
    let file_path = format!("{root}/src/lib.rs");

    let created = core.handle_request(&JsonRpcRequest::new(
        31_i64,
        "fs.createFile",
        Some(json!({ "path": file_path, "content": "pub fn answer() -> u8 { 42 }\n" })),
    ));
    let result = created.response().result.as_ref().unwrap();
    assert_eq!(result["path"], format!("{root}/src/lib.rs"));
    assert_eq!(result["bytesWritten"], 29);

    let duplicate = core.handle_request(&JsonRpcRequest::new(
        32_i64,
        "fs.createFile",
        Some(json!({ "path": format!("{root}/src/lib.rs") })),
    ));
    assert_eq!(
        duplicate.response().error.as_ref().unwrap().code,
        kernwerk_protocol::JsonRpcErrorCode::InvalidParams
    );

    let escape = core.handle_request(&JsonRpcRequest::new(
        33_i64,
        "fs.createFile",
        Some(json!({ "path": format!("{root}/../escape.rs") })),
    ));
    assert_eq!(
        escape.response().error.as_ref().unwrap().code,
        kernwerk_protocol::JsonRpcErrorCode::InvalidParams
    );
}

#[test]
fn fs_create_directory_creates_once_and_stays_inside_workspace() {
    let dir = std::env::temp_dir()
        .join("kernwerk-core-tests")
        .join(format!("{}-fs-create-directory", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("Cargo.toml"), "[package]\n").unwrap();
    let mut core = core_with_empty_search_path("fs-create-directory");

    let opened = core.handle_request(&JsonRpcRequest::new(
        34_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    let root = opened.response().result.as_ref().unwrap()["root"]
        .as_str()
        .unwrap()
        .to_owned();
    let directory_path = format!("{root}/src/features");

    let created = core.handle_request(&JsonRpcRequest::new(
        35_i64,
        "fs.createDirectory",
        Some(json!({ "path": directory_path })),
    ));
    assert_eq!(
        created.response().result.as_ref().unwrap()["path"],
        format!("{root}/src/features")
    );

    let duplicate = core.handle_request(&JsonRpcRequest::new(
        36_i64,
        "fs.createDirectory",
        Some(json!({ "path": format!("{root}/src/features") })),
    ));
    assert_eq!(
        duplicate.response().error.as_ref().unwrap().code,
        kernwerk_protocol::JsonRpcErrorCode::InvalidParams
    );

    let escape = core.handle_request(&JsonRpcRequest::new(
        37_i64,
        "fs.createDirectory",
        Some(json!({ "path": format!("{root}/../outside") })),
    ));
    assert_eq!(
        escape.response().error.as_ref().unwrap().code,
        kernwerk_protocol::JsonRpcErrorCode::InvalidParams
    );
}

#[test]
fn fs_rename_and_delete_stay_inside_workspace() {
    let dir = std::env::temp_dir()
        .join("kernwerk-core-tests")
        .join(format!("{}-fs-rename-delete", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("Cargo.toml"), "[package]\n").unwrap();
    std::fs::write(dir.join("src/old.rs"), "fn old() {}\n").unwrap();
    let mut core = core_with_empty_search_path("fs-rename-delete");

    let opened = core.handle_request(&JsonRpcRequest::new(
        40_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    let root = opened.response().result.as_ref().unwrap()["root"]
        .as_str()
        .unwrap()
        .to_owned();

    let renamed = core.handle_request(&JsonRpcRequest::new(
        41_i64,
        "fs.rename",
        Some(json!({
            "from": format!("{root}/src/old.rs"),
            "to": format!("{root}/src/new.rs"),
        })),
    ));
    let result = renamed.response().result.as_ref().unwrap();
    assert_eq!(result["from"], format!("{root}/src/old.rs"));
    assert_eq!(result["to"], format!("{root}/src/new.rs"));
    assert!(std::path::Path::new(&format!("{root}/src/new.rs")).exists());

    let escape = core.handle_request(&JsonRpcRequest::new(
        42_i64,
        "fs.rename",
        Some(json!({
            "from": format!("{root}/src/new.rs"),
            "to": format!("{root}/../escape.rs"),
        })),
    ));
    assert_eq!(
        escape.response().error.as_ref().unwrap().code,
        kernwerk_protocol::JsonRpcErrorCode::InvalidParams
    );

    let deleted = core.handle_request(&JsonRpcRequest::new(
        43_i64,
        "fs.delete",
        Some(json!({ "path": format!("{root}/src/new.rs") })),
    ));
    assert_eq!(
        deleted.response().result.as_ref().unwrap()["path"],
        format!("{root}/src/new.rs")
    );
    assert!(!std::path::Path::new(&format!("{root}/src/new.rs")).exists());

    let escape_delete = core.handle_request(&JsonRpcRequest::new(
        44_i64,
        "fs.delete",
        Some(json!({ "path": "/etc/hostname" })),
    ));
    assert_eq!(
        escape_delete.response().error.as_ref().unwrap().code,
        kernwerk_protocol::JsonRpcErrorCode::InvalidParams
    );
}

#[test]
fn fs_search_finds_matches_and_requires_workspace() {
    let mut core = core_with_empty_search_path("fs-search-no-workspace");
    let denied = core.handle_request(&JsonRpcRequest::new(
        26_i64,
        "fs.search",
        Some(json!({ "query": "main" })),
    ));
    assert!(denied.response().error.is_some());

    let dir = std::env::temp_dir()
        .join("kernwerk-core-tests")
        .join(format!("{}-fs-search", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("Cargo.toml"), "[package]\n").unwrap();
    std::fs::write(
        dir.join("src/main.rs"),
        "fn main() {\n    Encontrar();\n}\n",
    )
    .unwrap();
    let opened = core.handle_request(&JsonRpcRequest::new(
        27_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none());

    let searched = core.handle_request(&JsonRpcRequest::new(
        28_i64,
        "fs.search",
        Some(json!({ "query": "encontrar" })),
    ));
    let result = searched.response().result.as_ref().unwrap();
    assert_eq!(result["truncated"], false);
    assert_eq!(result["matches"][0]["path"], "src/main.rs");
    assert_eq!(result["matches"][0]["line"], 2);
    assert_eq!(result["matches"][0]["column"], 5);

    let empty = core.handle_request(&JsonRpcRequest::new(
        29_i64,
        "fs.search",
        Some(json!({ "query": "" })),
    ));
    assert!(empty.response().error.is_some());
}

#[test]
fn fs_find_files_requires_workspace_and_non_empty_query() {
    let mut core = core_with_empty_search_path("fs-find-files-no-workspace");
    let denied = core.handle_request(&JsonRpcRequest::new(
        38_i64,
        "fs.findFiles",
        Some(json!({ "query": "main" })),
    ));
    assert_eq!(
        denied.response().error.as_ref().unwrap().code,
        kernwerk_protocol::JsonRpcErrorCode::InvalidRequest
    );

    let dir = std::env::temp_dir()
        .join("kernwerk-core-tests")
        .join(format!("{}-fs-find-files", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("Cargo.toml"), "[package]\n").unwrap();
    let opened = core.handle_request(&JsonRpcRequest::new(
        39_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none());

    let empty = core.handle_request(&JsonRpcRequest::new(
        40_i64,
        "fs.findFiles",
        Some(json!({ "query": "" })),
    ));
    assert_eq!(
        empty.response().error.as_ref().unwrap().code,
        kernwerk_protocol::JsonRpcErrorCode::InvalidParams
    );
}

#[test]
fn run_start_requires_workspace_and_enabled_manager() {
    let mut core = core_with_empty_search_path("run-no-workspace");
    let denied = core.handle_request(&JsonRpcRequest::new(40_i64, "run.start", None));
    assert!(denied.response().error.is_some());

    let dir = std::env::temp_dir()
        .join("kernwerk-core-tests")
        .join(format!("{}-run-start", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let opened = core.handle_request(&JsonRpcRequest::new(
        41_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none());

    let unavailable = core.handle_request(&JsonRpcRequest::new(42_i64, "run.start", None));
    let error = unavailable.response().error.as_ref().unwrap();
    assert_eq!(
        error.code,
        kernwerk_protocol::JsonRpcErrorCode::InternalError
    );
}

#[test]
fn run_start_executes_command_and_emits_events() {
    let (sender, receiver) = std::sync::mpsc::channel();
    let mut core = core_with_empty_search_path("run-e2e");
    core.enable_lsp(sender);

    let dir = std::env::temp_dir()
        .join("kernwerk-core-tests")
        .join(format!("{}-run-e2e", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let opened = core.handle_request(&JsonRpcRequest::new(
        43_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none());

    let no_default = core.handle_request(&JsonRpcRequest::new(44_i64, "run.start", None));
    assert!(no_default.response().error.is_some());

    let started = core.handle_request(&JsonRpcRequest::new(
        45_i64,
        "run.start",
        Some(json!({ "command": "printf 'executado\\n'" })),
    ));
    let result = started.response().result.as_ref().unwrap();
    assert_eq!(result["command"], "printf 'executado\\n'");

    let mut saw_output = false;
    loop {
        let event = receiver
            .recv_timeout(std::time::Duration::from_secs(5))
            .expect("evento run dentro do timeout");
        if event.method == "event.run.output" {
            saw_output = event.params.as_ref().unwrap()["line"] == "executado";
        }
        if event.method == "event.run.finished" {
            assert_eq!(event.params.as_ref().unwrap()["success"], true);
            break;
        }
    }
    assert!(saw_output);

    let stopped = core.handle_request(&JsonRpcRequest::new(46_i64, "run.stop", None));
    assert!(stopped.response().error.is_some());
}

#[test]
fn build_run_requires_open_workspace() {
    let mut core = core_with_empty_search_path("build-no-workspace");
    let request = JsonRpcRequest::new(30_i64, "build.run", Some(json!({})));
    let outcome = core.handle_request_streaming(&request, &mut |_| {});
    let error = outcome.response().error.as_ref().unwrap();

    assert_eq!(
        error.code,
        kernwerk_protocol::JsonRpcErrorCode::InvalidRequest
    );
    assert_eq!(error.message, "nenhum workspace aberto");
}

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
        kernwerk_protocol::JsonRpcErrorCode::InvalidRequest
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
            kernwerk_protocol::JsonRpcErrorCode::InvalidRequest
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
        kernwerk_protocol::JsonRpcErrorCode::InvalidRequest
    );
    assert_eq!(error.message, "nenhum workspace aberto");

    let workspace = std::env::temp_dir()
        .join("kernwerk-core-tests")
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
        kernwerk_protocol::JsonRpcErrorCode::InvalidParams
    );
}

#[test]
fn lsp_navigation_reports_unavailable_without_lsp_manager() {
    let workspace = std::env::temp_dir()
        .join("kernwerk-core-tests")
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

    assert_eq!(
        error.code,
        kernwerk_protocol::JsonRpcErrorCode::InternalError
    );
    assert_eq!(error.message, "LSP nao esta habilitado neste loop do core");
}

#[test]
fn lsp_did_change_rejects_paths_outside_workspace() {
    let workspace = std::env::temp_dir()
        .join("kernwerk-core-tests")
        .join(format!("{}-lsp-workspace", std::process::id()));
    let outside = std::env::temp_dir()
        .join("kernwerk-core-tests")
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

    assert_eq!(
        error.code,
        kernwerk_protocol::JsonRpcErrorCode::InvalidParams
    );
}

#[test]
fn build_run_rejects_unsupported_project_kind() {
    let dir = std::env::temp_dir()
        .join("kernwerk-core-tests")
        .join(format!("{}-build-unsupported", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("pyproject.toml"), "[project]\n").unwrap();
    let mut core = core_with_empty_search_path("build-unsupported");

    let opened = core.handle_request(&JsonRpcRequest::new(
        31_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none());

    let mut events = Vec::new();
    let request = JsonRpcRequest::new(32_i64, "build.run", Some(json!({})));
    let outcome = core.handle_request_streaming(&request, &mut |notification| {
        events.push(notification.method.clone());
    });
    let error = outcome.response().error.as_ref().unwrap();

    assert_eq!(
        error.code,
        kernwerk_protocol::JsonRpcErrorCode::InvalidRequest
    );
    assert!(error.message.contains("python"));
    assert!(events.is_empty());
}

#[test]
fn stdio_loop_writes_one_response_per_line_and_stops() {
    let input = concat!(
        r#"{"jsonrpc":"2.0","id":1,"method":"core.ping","params":{}}"#,
        "\n",
        r#"{"jsonrpc":"2.0","id":2,"method":"core.shutdown","params":{}}"#,
        "\n",
        r#"{"jsonrpc":"2.0","id":3,"method":"core.ping","params":{}}"#,
        "\n",
    );
    let reader = BufReader::new(Cursor::new(input.as_bytes()));
    let mut output = Vec::new();

    run_json_lines(reader, &mut output).unwrap();

    let text = String::from_utf8(output).unwrap();
    let responses = text
        .lines()
        .map(serde_json::from_str::<Value>)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    assert_eq!(responses.len(), 2);
    assert_eq!(responses[0]["result"]["message"], "pong");
    assert_eq!(responses[1]["result"]["message"], "shutdown requested");
}

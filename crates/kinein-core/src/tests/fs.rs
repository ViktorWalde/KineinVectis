//! Workspace-confined filesystem dispatch (`fs.*`).

use serde_json::json;

use super::core_with_empty_search_path;
use kinein_protocol::JsonRpcRequest;

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
        kinein_protocol::JsonRpcErrorCode::InvalidRequest
    );
    assert_eq!(error.message, "nenhum workspace aberto");
}

#[test]
fn fs_list_read_write_cycle_inside_workspace() {
    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
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
    assert_eq!(entries[0]["name"], ".kinein");
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
        kinein_protocol::JsonRpcErrorCode::InvalidParams
    );
}

#[test]
fn fs_create_file_creates_once_and_stays_inside_workspace() {
    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
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
        kinein_protocol::JsonRpcErrorCode::InvalidParams
    );

    let escape = core.handle_request(&JsonRpcRequest::new(
        33_i64,
        "fs.createFile",
        Some(json!({ "path": format!("{root}/../escape.rs") })),
    ));
    assert_eq!(
        escape.response().error.as_ref().unwrap().code,
        kinein_protocol::JsonRpcErrorCode::InvalidParams
    );
}

#[test]
fn fs_create_directory_creates_once_and_stays_inside_workspace() {
    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
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
        kinein_protocol::JsonRpcErrorCode::InvalidParams
    );

    let escape = core.handle_request(&JsonRpcRequest::new(
        37_i64,
        "fs.createDirectory",
        Some(json!({ "path": format!("{root}/../outside") })),
    ));
    assert_eq!(
        escape.response().error.as_ref().unwrap().code,
        kinein_protocol::JsonRpcErrorCode::InvalidParams
    );
}

#[test]
fn fs_rename_and_delete_stay_inside_workspace() {
    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
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
        kinein_protocol::JsonRpcErrorCode::InvalidParams
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
        kinein_protocol::JsonRpcErrorCode::InvalidParams
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
        .join("kinein-core-tests")
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
        kinein_protocol::JsonRpcErrorCode::InvalidRequest
    );

    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
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
        kinein_protocol::JsonRpcErrorCode::InvalidParams
    );
}

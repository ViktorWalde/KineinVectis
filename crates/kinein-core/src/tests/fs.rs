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
    assert_eq!(error.message, "nenhum projeto aberto");
    let trash = core.handle_request(&JsonRpcRequest::new(
        21_i64,
        "fs.trash",
        Some(json!({ "path": "/tmp" })),
    ));
    assert_eq!(
        trash.response().error.as_ref().unwrap().code,
        kinein_protocol::JsonRpcErrorCode::InvalidRequest
    );
    let external = core.handle_request(&JsonRpcRequest::new(
        22_i64,
        "fs.readExternal",
        Some(json!({ "path": "/tmp/nao-importar.txt" })),
    ));
    assert_eq!(
        external.response().error.as_ref().unwrap().code,
        kinein_protocol::JsonRpcErrorCode::InvalidRequest
    );
}

#[test]
#[cfg(unix)]
fn fs_external_preview_reads_only_selected_text_without_importing() {
    let base =
        std::env::temp_dir().join(format!("kinein-fs-external-preview-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let workspace = base.join("workspace");
    let external = base.join("external");
    std::fs::create_dir_all(&workspace).unwrap();
    std::fs::create_dir_all(&external).unwrap();
    let file = external.join("arquivo.txt");
    std::fs::write(&file, "texto externo\n").unwrap();
    let link = external.join("link.txt");
    std::os::unix::fs::symlink(&file, &link).unwrap();
    let binary = external.join("binary.bin");
    std::fs::write(&binary, [0xff, 0x00]).unwrap();
    let large = external.join("large.txt");
    std::fs::write(&large, vec![b'a'; 1024 * 1024 + 1]).unwrap();

    let mut core = core_with_empty_search_path("fs-external-preview");
    let opened = core.handle_request(&JsonRpcRequest::new(
        30_i64,
        "workspace.open",
        Some(json!({ "path": workspace })),
    ));
    assert!(opened.response().error.is_none());

    let preview = core.handle_request(&JsonRpcRequest::new(
        31_i64,
        "fs.readExternal",
        Some(json!({ "path": file })),
    ));
    let result = preview.response().result.as_ref().unwrap();
    assert_eq!(result["path"], file.to_str().unwrap());
    assert_eq!(result["content"], "texto externo\n");
    assert_eq!(std::fs::read_to_string(&file).unwrap(), "texto externo\n");
    assert!(!workspace.join("arquivo.txt").exists());

    for (id, path) in [(32, &link), (33, &binary), (34, &large), (35, &external)] {
        let rejected = core.handle_request(&JsonRpcRequest::new(
            id,
            "fs.readExternal",
            Some(json!({ "path": path })),
        ));
        assert!(
            rejected.response().error.is_some(),
            "path={}",
            path.display()
        );
    }
    for method in ["fs.read", "fs.write"] {
        let rejected = core.handle_request(&JsonRpcRequest::new(
            36_i64,
            method,
            Some(json!({ "path": file, "content": "alterado" })),
        ));
        assert!(rejected.response().error.is_some(), "method={method}");
    }
    assert_eq!(std::fs::read_to_string(&file).unwrap(), "texto externo\n");
    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn fs_replace_flows_through_dispatch_and_changes_all_matching_files() {
    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-fs-replace-dispatch", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::create_dir_all(dir.join("target")).unwrap();
    std::fs::write(dir.join("src/a.txt"), "alpha beta alpha\n").unwrap();
    std::fs::write(dir.join("src/b.txt"), "ALPHA\n").unwrap();
    std::fs::write(dir.join("target/skipped.txt"), "alpha\n").unwrap();
    let mut core = core_with_empty_search_path("fs-replace-dispatch");
    let opened = core.handle_request(&JsonRpcRequest::new(
        200_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none());

    let empty = core.handle_request(&JsonRpcRequest::new(
        201_i64,
        "fs.replace",
        Some(json!({ "query": "", "replacement": "x" })),
    ));
    assert_eq!(
        empty.response().error.as_ref().unwrap().code,
        kinein_protocol::JsonRpcErrorCode::InvalidParams
    );

    let replaced = core.handle_request(&JsonRpcRequest::new(
        202_i64,
        "fs.replace",
        Some(json!({
            "query": "alpha",
            "replacement": "omega",
            "caseSensitive": false
        })),
    ));
    let result = replaced.response().result.as_ref().unwrap();
    assert_eq!(result["replacements"], 3);
    assert_eq!(result["files"].as_array().unwrap().len(), 2);
    assert_eq!(
        std::fs::read_to_string(dir.join("src/a.txt")).unwrap(),
        "omega beta omega\n"
    );
    assert_eq!(
        std::fs::read_to_string(dir.join("src/b.txt")).unwrap(),
        "omega\n"
    );
    assert_eq!(
        std::fs::read_to_string(dir.join("target/skipped.txt")).unwrap(),
        "alpha\n"
    );
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
        Some(json!({
            "path": file_path,
            "content": "// editado\n",
            "expectedContent": "fn main() {}\n"
        })),
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
    assert_eq!(error.code, kinein_protocol::JsonRpcErrorCode::InvalidParams);
}

#[test]
fn fs_write_rejects_stale_editor_snapshot() {
    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-fs-write-conflict", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("main.rs");
    std::fs::write(&file, "disk v1\n").unwrap();
    let mut core = core_with_empty_search_path("fs-write-conflict");
    let opened = core.handle_request(&JsonRpcRequest::new(
        26_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none());

    std::fs::write(&file, "external v2\n").unwrap();
    let outcome = core.handle_request(&JsonRpcRequest::new(
        27_i64,
        "fs.write",
        Some(json!({
            "path": file.to_str().unwrap(),
            "content": "local buffer\n",
            "expectedContent": "disk v1\n"
        })),
    ));
    let error = outcome.response().error.as_ref().unwrap();

    assert_eq!(error.code, kinein_protocol::JsonRpcErrorCode::FileChanged);
    assert_eq!(
        error.details.as_ref().unwrap()["path"],
        file.to_str().unwrap()
    );
    assert_eq!(std::fs::read_to_string(file).unwrap(), "external v2\n");
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

    let escape_trash = core.handle_request(&JsonRpcRequest::new(
        45_i64,
        "fs.trash",
        Some(json!({ "path": "/etc/hostname" })),
    ));
    assert_eq!(
        escape_trash.response().error.as_ref().unwrap().code,
        kinein_protocol::JsonRpcErrorCode::InvalidParams
    );
}

#[test]
fn fs_copy_uses_workspace_and_keeps_source() {
    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-fs-copy", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("original.txt"), "conteudo\n").unwrap();
    let mut core = core_with_empty_search_path("fs-copy");
    let denied = core.handle_request(&JsonRpcRequest::new(
        400_i64,
        "fs.copy",
        Some(json!({ "from": "/tmp/a", "to": "/tmp/b" })),
    ));
    assert_eq!(
        denied.response().error.as_ref().unwrap().code,
        kinein_protocol::JsonRpcErrorCode::InvalidRequest
    );

    let opened = core.handle_request(&JsonRpcRequest::new(
        401_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    let root = opened.response().result.as_ref().unwrap()["root"]
        .as_str()
        .unwrap();
    let source = format!("{root}/original.txt");
    let target = format!("{root}/copia.txt");
    let copied = core.handle_request(&JsonRpcRequest::new(
        402_i64,
        "fs.copy",
        Some(json!({ "from": source, "to": target })),
    ));
    assert!(copied.response().error.is_none());
    assert_eq!(copied.response().result.as_ref().unwrap()["to"], target);
    assert_eq!(std::fs::read_to_string(&source).unwrap(), "conteudo\n");
    assert_eq!(std::fs::read_to_string(&target).unwrap(), "conteudo\n");
}

#[test]
fn fs_copy_batch_and_import_use_existing_job_manager_when_services_are_enabled() {
    use std::{sync::mpsc, time::Duration};

    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-fs-copy-job", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("origem.bin"), vec![7_u8; 2 * 1024 * 1024]).unwrap();
    let (events_tx, events_rx) = mpsc::channel();
    let (responses_tx, responses_rx) = mpsc::channel();
    let mut core = core_with_empty_search_path("fs-copy-job");
    core.enable_lsp(events_tx);
    core.enable_deferred_responses(responses_tx);
    let opened = core.handle_request(&JsonRpcRequest::new(
        410_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none());
    let root = opened.response().result.as_ref().unwrap()["root"]
        .as_str()
        .unwrap();
    let source = format!("{root}/origem.bin");
    let target = format!("{root}/copia.bin");
    let deferred = core.handle_request(&JsonRpcRequest::new(
        411_i64,
        "fs.copy",
        Some(json!({ "from": source, "to": target })),
    ));
    assert_eq!(
        deferred.response().result.as_ref().unwrap()["kineinDeferred"],
        true
    );

    let response = responses_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    assert!(response.error.is_none(), "{:?}", response.error);
    assert_eq!(response.result.as_ref().unwrap()["to"], target);
    assert_cancelable_job_lifecycle(&events_rx, "fs.copy");
    assert_eq!(std::fs::metadata(&target).unwrap().len(), 2 * 1024 * 1024);
    assert_eq!(std::fs::metadata(&source).unwrap().len(), 2 * 1024 * 1024);

    std::fs::write(dir.join("outro.txt"), "segundo").unwrap();
    let batch_target = format!("{root}/lote.bin");
    let second_target = format!("{root}/lote.txt");
    let deferred = core.handle_request(&JsonRpcRequest::new(
        412_i64,
        "fs.transferBatch",
        Some(json!({
            "operation": "copy",
            "items": [
                { "from": source, "to": batch_target },
                { "from": format!("{root}/outro.txt"), "to": second_target }
            ]
        })),
    ));
    assert_eq!(
        deferred.response().result.as_ref().unwrap()["kineinDeferred"],
        true
    );
    let response = responses_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    assert!(response.error.is_none(), "{:?}", response.error);
    let result = response.result.as_ref().unwrap();
    assert_eq!(result["items"][0]["status"], "success");
    assert_eq!(result["items"][1]["status"], "success");
    assert_eq!(std::fs::read_to_string(&second_target).unwrap(), "segundo");

    assert_cancelable_job_lifecycle(&events_rx, "fs.transferBatch");

    let external = dir.with_extension("external-source");
    let _ = std::fs::remove_dir_all(&external);
    std::fs::create_dir(&external).unwrap();
    let external_source = external.join("arquivo externo.txt");
    std::fs::write(&external_source, "externo").unwrap();
    let import_target = format!("{root}/importado.txt");
    let deferred = core.handle_request(&JsonRpcRequest::new(
        413_i64,
        "fs.transferBatch",
        Some(json!({
            "operation": "import",
            "items": [{ "from": external_source, "to": import_target }]
        })),
    ));
    assert_eq!(
        deferred.response().result.as_ref().unwrap()["kineinDeferred"],
        true
    );
    let response = responses_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    assert!(response.error.is_none(), "{:?}", response.error);
    let result = response.result.as_ref().unwrap();
    assert_eq!(result["operation"], "import");
    assert_eq!(result["items"][0]["status"], "success");
    assert_eq!(std::fs::read_to_string(&import_target).unwrap(), "externo");
    assert_eq!(
        std::fs::read_to_string(&external_source).unwrap(),
        "externo"
    );
    assert_cancelable_job_lifecycle(&events_rx, "fs.transferBatch");
}

fn assert_cancelable_job_lifecycle(events: &std::sync::mpsc::Receiver<JsonRpcRequest>, kind: &str) {
    let mut job_id = String::new();
    let mut progress = false;
    loop {
        let event = events
            .recv_timeout(std::time::Duration::from_secs(5))
            .unwrap();
        let params = event.params.as_ref().unwrap();
        match event.method.as_str() {
            "event.job.created" if params["kind"] == kind => {
                job_id = params["id"].as_str().unwrap().to_owned();
                assert_eq!(params["canCancel"], true);
            }
            "event.job.progress" if params["jobId"] == job_id => progress = true,
            "event.job.finished" if !job_id.is_empty() && params["jobId"] == job_id => {
                assert_eq!(params["status"], "success");
                assert!(progress);
                return;
            }
            _ => {}
        }
    }
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

/// Busca e substituição MULTI-LINHA, e a invariante que as amarra.
///
/// Até 2026-09-02 uma query com `\n` era **recusada**, e pela razão certa: a
/// busca casava LINHA A LINHA enquanto a substituição casava no conteúdo
/// inteiro. A query multi-linha era, portanto, invisível para o preview e ativa
/// para a escrita — o usuário via "0 resultados", mandava substituir e arquivos
/// eram reescritos. `fs.replace` é destrutivo: transação, snapshot e rollback
/// protegem contra falha de ESCRITA, não contra aprovar o que não se viu.
///
/// A etapa 9 do `roadmaps/30` não removeu a guarda — ela **consertou a busca**.
/// O que este teste trava é a igualdade que tornou a remoção legítima: o número
/// de resultados da busca é o número de substituições do replace, também
/// quando o casamento atravessa linhas.
#[test]
fn multiline_search_previews_exactly_what_replace_will_rewrite() {
    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-fs-multilinha", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("Cargo.toml"), "[package]\n").unwrap();
    std::fs::write(
        dir.join("a.txt"),
        "antes\nprimeira\nsegunda\ndepois\nprimeira\nsegunda\nfim\n",
    )
    .unwrap();

    let mut core = core_with_empty_search_path("fs-multilinha");
    let aberto = core.handle_request(&JsonRpcRequest::new(
        190_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    assert!(aberto.response().error.is_none());

    // A busca ACHA as duas ocorrências, e diz onde cada uma começa.
    let busca = core.handle_request(&JsonRpcRequest::new(
        191_i64,
        "fs.search",
        Some(json!({ "query": "primeira\nsegunda" })),
    ));
    let encontrados = busca.response().result.as_ref().unwrap()["matches"]
        .as_array()
        .unwrap()
        .clone();
    assert_eq!(encontrados.len(), 2);
    assert_eq!(encontrados[0]["line"], 2);
    assert_eq!(encontrados[0]["column"], 1);
    assert_eq!(encontrados[1]["line"], 5);

    // E o PREVIEW mostra a quebra: sem isso, um casamento de duas linhas
    // pareceria de uma, e o usuário aprovaria mais do que viu.
    assert_eq!(encontrados[0]["preview"], "primeira ⏎ segunda");

    let troca = core.handle_request(&JsonRpcRequest::new(
        192_i64,
        "fs.replace",
        Some(json!({ "query": "primeira\nsegunda", "replacement": "unica" })),
    ));
    let resultado = troca.response().result.as_ref().unwrap();

    // A INVARIANTE: o preview contou exatamente o que a escrita fez.
    assert_eq!(resultado["replacements"], encontrados.len());
    assert_eq!(
        std::fs::read_to_string(dir.join("a.txt")).unwrap(),
        "antes\nunica\ndepois\nunica\nfim\n"
    );
}

/// Substituir por texto MULTI-LINHA também vale, e a próxima busca enxerga.
///
/// O replacement com `\n` era recusado junto com a query, porque quebraria o
/// casamento linha a linha da busca seguinte sobre o mesmo arquivo. Com a busca
/// casando no conteúdo, isso deixou de ser verdade — e este teste é quem prova,
/// buscando DEPOIS de escrever.
#[test]
fn a_multiline_replacement_is_found_by_the_next_search() {
    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-fs-multilinha-repl", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("Cargo.toml"), "[package]\n").unwrap();
    std::fs::write(dir.join("a.txt"), "alfa\n").unwrap();

    let mut core = core_with_empty_search_path("fs-multilinha-repl");
    let aberto = core.handle_request(&JsonRpcRequest::new(
        193_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    assert!(aberto.response().error.is_none());

    let troca = core.handle_request(&JsonRpcRequest::new(
        194_i64,
        "fs.replace",
        Some(json!({ "query": "alfa", "replacement": "um\ndois" })),
    ));
    assert!(troca.response().error.is_none());
    assert_eq!(
        std::fs::read_to_string(dir.join("a.txt")).unwrap(),
        "um\ndois\n"
    );

    let busca = core.handle_request(&JsonRpcRequest::new(
        195_i64,
        "fs.search",
        Some(json!({ "query": "um\ndois" })),
    ));
    let encontrados = busca.response().result.as_ref().unwrap()["matches"]
        .as_array()
        .unwrap()
        .clone();
    assert_eq!(encontrados.len(), 1, "o que foi escrito tem de ser achavel");
    assert_eq!(encontrados[0]["line"], 1);
}

/// O rascunho acompanha o `fs.rename`.
///
/// A chave da store é o caminho absoluto. Renomear sem mover deixava o rascunho
/// órfão: o caminho antigo não existe mais e o novo não tem autosave. Na
/// abertura seguinte o rascunho era descartado — perda silenciosa dentro da
/// própria rede de segurança (DocsPublic/seguranca/23).
#[test]
fn renaming_a_file_moves_its_draft_with_it() {
    let base = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-fs-rename-draft", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let projeto = base.join("projeto");
    std::fs::create_dir_all(&projeto).unwrap();
    std::fs::write(projeto.join("Cargo.toml"), "[package]\n").unwrap();
    std::fs::write(projeto.join("velho.txt"), "no disco\n").unwrap();
    let global = base.join("global");
    std::fs::create_dir_all(&global).unwrap();

    let (sender, _receiver) = std::sync::mpsc::channel();
    let mut core = core_with_empty_search_path("fs-rename-draft");
    core.enable_lsp(sender);
    core.enable_persistence(global);
    let aberto = core.handle_request(&JsonRpcRequest::new(
        200_i64,
        "workspace.open",
        Some(json!({ "path": projeto.to_str().unwrap() })),
    ));
    assert!(aberto.response().error.is_none());

    let salvo = core.handle_request(&JsonRpcRequest::new(
        201_i64,
        "draft.save",
        Some(json!({
            "path": projeto.join("velho.txt").to_str().unwrap(),
            "content": "editado, nao salvo\n",
        })),
    ));
    assert!(salvo.response().error.is_none());

    let renomeado = core.handle_request(&JsonRpcRequest::new(
        202_i64,
        "fs.rename",
        Some(json!({
            "from": projeto.join("velho.txt").to_str().unwrap(),
            "to": projeto.join("novo.txt").to_str().unwrap(),
        })),
    ));
    assert!(renomeado.response().error.is_none());

    // Reabrir e' o gesto que recupera: o rascunho tem de voltar no caminho NOVO.
    let fechado = core.handle_request(&JsonRpcRequest::new(203_i64, "workspace.close", None));
    assert!(fechado.response().error.is_none());
    let reaberto = core.handle_request(&JsonRpcRequest::new(
        204_i64,
        "workspace.open",
        Some(json!({ "path": projeto.to_str().unwrap() })),
    ));
    let drafts = reaberto.response().result.as_ref().unwrap()["drafts"]
        .as_array()
        .cloned()
        .unwrap_or_default();

    let caminhos: Vec<&str> = drafts
        .iter()
        .filter_map(|item| item["path"].as_str())
        .collect();
    assert_eq!(caminhos.len(), 1, "recuperou {caminhos:?}");
    assert!(
        caminhos[0].ends_with("novo.txt"),
        "o rascunho ficou em {}",
        caminhos[0]
    );
    assert_eq!(drafts[0]["content"], "editado, nao salvo\n");
}

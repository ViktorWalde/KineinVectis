//! Headers antes das fontes: comando comprovado, ambiguidades e buffer vivo.

use super::*;
use crate::index::context::{CompileContext, ContextTools};

fn configured(name: &str) -> Project {
    let mut project = Project::new(name);
    project.fixture();
    project.open();
    project.call("workspace.trust", json!({}));
    let job = project.call("cmake.configure", json!({}))["jobId"].clone();
    assert_eq!(
        project.wait("event.cmake.finished", |value| value["jobId"] == job)["success"],
        true
    );
    project.wait_context("include/c_api.h", "c");
    project
}

fn hover(project: &mut Project, file: &str, buffer: &str) -> Value {
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        let response = project.core.handle_request(&JsonRpcRequest::new(
            2_i64,
            "lsp.hover",
            Some(json!({"path":project.root.join(file),"content":buffer,"line":4,"column":25})),
        ));
        if let Some(result) = &response.response().result {
            return result.clone();
        }
        assert!(Instant::now() < deadline, "{:?}", response.response());
        std::thread::sleep(Duration::from_millis(30));
    }
}

#[test]
fn c_identity_header_references_refuse_ambiguity_and_manual_databases() {
    let project = configured("header-reference");
    let load = || CompileContext::load(&project.root, &ContextTools::default());
    let header = project.root.join("include/cpp_api.h");
    let command = load().header_command(&project.root, &header).unwrap();
    assert_eq!(command.source, project.root.join("src/main.cpp"));
    assert_eq!(command.target, "mixed");
    assert!(
        load()
            .for_file(&project.root, &header, "cpp")
            .unit
            .is_none()
    );
    for file in ["include/shared.h", "include/orphan.h", "src/main.cpp"] {
        assert!(
            load()
                .header_command(&project.root, &project.root.join(file))
                .is_none()
        );
    }
    project.put(".clangd", "CompileFlags:\n  Add: [-DKEPT=1]\n");
    assert_eq!(load().header_command(&project.root, &header), Some(command));
    project.put(".clangd", "CompileFlags:\n  CompilationDatabase: None\n");
    assert!(load().header_command(&project.root, &header).is_none());
    project.put(".clangd", "");
    let cdb_path = project.root.join(".kinein/build/compile_commands.json");
    let mut cdb: Vec<Value> =
        serde_json::from_str(&std::fs::read_to_string(&cdb_path).unwrap()).unwrap();
    cdb.push(
        cdb.iter()
            .find(|entry| entry["file"].as_str().unwrap().ends_with("main.cpp"))
            .unwrap()
            .clone(),
    );
    std::fs::write(&cdb_path, json!(cdb).to_string()).unwrap();
    assert!(load().header_command(&project.root, &header).is_none());
    cdb.push(
        json!({"directory":project.root,"file":header,"arguments":["clang","-xc-header",header]}),
    );
    std::fs::write(&cdb_path, json!(cdb).to_string()).unwrap();
    assert!(load().header_command(&project.root, &header).is_none());
    assert_eq!(load().document_language(&header), "c");
}

fn change_reference_flags(project: &Project) {
    let cdb_path = project.root.join(".kinein/build/compile_commands.json");
    let mut entries: Vec<Value> =
        serde_json::from_str(&std::fs::read_to_string(&cdb_path).unwrap()).unwrap();
    let source = entries
        .iter_mut()
        .find(|entry| entry["file"].as_str().unwrap().ends_with("main.cpp"))
        .unwrap();
    source["command"] = json!(format!(
        "{} -DHEADER_REVISION=2",
        source["command"].as_str().unwrap()
    ));
    std::fs::write(cdb_path, json!(entries).to_string()).unwrap();
}

fn wait_reference_flag(project: &mut Project, path: &Path, flag: &str) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        project.pump();
        let updated = project
            .core
            .index
            .lock()
            .unwrap()
            .context
            .as_ref()
            .and_then(|context| context.header_command(&project.root, path))
            .is_some_and(|command| command.arguments.iter().any(|arg| arg == flag));
        if updated {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "header reference context deadline"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn c_identity_header_wire_preserves_argv_and_invalidates_unsaved_buffers() {
    let mut project = configured("header-wire");
    wire::fake_server(&mut project);
    let file = "include/cpp_api.h";
    let path = project.root.join(file);
    let buffer = "// unsaved\nnamespace unsaved { int live; }\n";
    hover(&mut project, file, buffer);
    let messages = wire::messages(&project.root.join("wire.jsonl"));
    let config = messages
        .iter()
        .position(|value| {
            value
                .pointer("/params/settings/compilationDatabaseChanges")
                .is_some()
        })
        .unwrap();
    let open = messages
        .iter()
        .position(|value| value["method"] == "textDocument/didOpen")
        .unwrap();
    assert!(config < open);
    let reference = project
        .core
        .index
        .lock()
        .unwrap()
        .context
        .as_ref()
        .unwrap()
        .header_command(&project.root, &path)
        .unwrap();
    let sent = &messages[config]["params"]["settings"]["compilationDatabaseChanges"]
        [path.to_str().unwrap()];
    assert_eq!(sent["compilationCommand"], json!(reference.arguments));
    assert_eq!(sent["workingDirectory"], reference.directory);
    assert_eq!(messages[open]["params"]["textDocument"]["text"], buffer);
    let generation = project.core.lsp.as_ref().unwrap().code_action_generation();
    // Same family/text with changed per-target flags is a new context too.
    change_reference_flags(&project);
    project.core.reload_index_context();
    // A recarga e' um job: familia igual/Ready nao comprova flags novas.
    wait_reference_flag(&mut project, &path, "-DHEADER_REVISION=2");
    assert!(
        !project
            .core
            .lsp
            .as_ref()
            .unwrap()
            .document_matches(&path, buffer)
    );
    hover(&mut project, file, buffer);
    let changed = wire::messages(&project.root.join("wire.jsonl"));
    let updated = changed
        .iter()
        .rfind(|value| {
            value
                .pointer("/params/settings/compilationDatabaseChanges")
                .is_some()
        })
        .unwrap();
    assert!(updated.to_string().contains("-DHEADER_REVISION=2"));
    // A manual CDB choice removes the override, requiring a fresh server.
    project.put(".clangd", "CompileFlags:\n  CompilationDatabase: None\n");
    assert!(
        !project
            .core
            .lsp
            .as_ref()
            .unwrap()
            .document_matches(&path, buffer)
    );
    hover(&mut project, file, buffer);
    let messages = wire::messages(&project.root.join("wire.jsonl"));
    assert_eq!(
        messages
            .iter()
            .filter(|value| value["method"] == "initialize")
            .count(),
        3
    );
    let second = messages
        .iter()
        .rposition(|value| value["method"] == "initialize")
        .unwrap();
    assert!(messages[second..].iter().all(|value| {
        value
            .pointer("/params/settings/compilationDatabaseChanges")
            .is_none()
    }));
    assert_eq!(
        messages
            .iter()
            .rfind(|value| value["method"] == "textDocument/didOpen")
            .unwrap()["params"]["textDocument"]["text"],
        buffer
    );
    assert!(project.core.lsp.as_ref().unwrap().code_action_generation() > generation);
    assert!(!std::fs::read_to_string(path).unwrap().contains("unsaved"));
}

#[test]
fn c_identity_header_cpp_first_clangd_uses_target_and_clangd_edits() {
    let mut project = configured("header-clangd");
    project.put(
        ".clangd",
        "CompileFlags:\n  Add: [-DHEADER_CONFIGURATION=73]\n",
    );
    assert!(
        project
            .core
            .use_language_server_command("cpp", "clangd", &["--background-index"])
    );
    let file = "include/cpp_api.h";
    let path = project.root.join(file);
    let buffer = "#if HEADER_CONFIGURATION != 73\n#error manual configuration lost\n#endif\nnamespace cpp_api { int live(); }\n";
    assert!(
        hover(&mut project, file, buffer)["content"]
            .as_str()
            .unwrap()
            .contains("live")
    );
    project.wait("event.lsp.diagnostics", |value| {
        value["path"] == json!(path)
    });
    // Request a distinct unsaved version and observe a real error, then recover.
    let broken = format!("{buffer}#error UNSAVED_HEADER\n");
    hover(&mut project, file, &broken);
    let diagnostic = project.wait("event.lsp.diagnostics", |value| {
        value["path"] == json!(path)
            && value["diagnostics"]
                .as_array()
                .unwrap()
                .iter()
                .any(|entry| {
                    entry["message"]
                        .as_str()
                        .unwrap()
                        .contains("UNSAVED_HEADER")
                })
    });
    assert_eq!(
        diagnostic["diagnostics"].as_array().unwrap().len(),
        1,
        "{diagnostic}"
    );
    hover(&mut project, file, buffer);
    project.wait("event.lsp.diagnostics", |value| {
        value["path"] == json!(path) && value["diagnostics"] == json!([])
    });
    assert!(!std::fs::read_to_string(path).unwrap().contains("live"));
}

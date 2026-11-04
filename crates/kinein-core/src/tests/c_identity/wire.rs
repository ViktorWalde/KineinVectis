//! Wire real do transporte contra servidor controlado; identidade muda sem editar.

use super::*;

pub(super) fn messages(path: &Path) -> Vec<Value> {
    std::fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

fn wait_document(p: &mut Project, file: &str, language: &str, count: usize) -> Vec<Value> {
    let path = p.root.join(file);
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        p.call("fs.read", json!({"path":path}));
        let wire = messages(&p.root.join("wire.jsonl"));
        let opened = wire
            .iter()
            .filter(|m| {
                m["method"] == "textDocument/didOpen"
                    && m["params"]["textDocument"]["uri"]
                        .as_str()
                        .unwrap()
                        .ends_with(file)
                    && m["params"]["textDocument"]["languageId"] == language
            })
            .count();
        if opened >= count {
            return wire;
        }
        assert!(Instant::now() < deadline, "{file} {language}: {wire:?}");
        std::thread::sleep(Duration::from_millis(30));
    }
}

pub(super) fn fake_server(p: &mut Project) {
    assert!(p.core.use_language_server_command(
        "cpp",
        super::super::lsp_server::python3(),
        &[
            super::super::lsp_server::fake_server().to_str().unwrap(),
            p.root.join("wire.jsonl").to_str().unwrap()
        ]
    ));
}

#[test]
fn c_identity_wire_uses_one_clangd_and_reopens_when_only_context_changes() {
    let mut p = Project::new("wire");
    for file in ["main.c", "main.cpp", "header.h"] {
        p.put(file, "int value(void);\n");
    }
    let cdb = |cpp: bool| json!([{"directory":p.root,"file":p.root.join("main.c"),"arguments":[if cpp {"clang++"} else {"clang"}]}]);
    p.put("compile_commands.json", &cdb(false).to_string());
    p.open();
    fake_server(&mut p);
    wait_document(&mut p, "main.c", "c", 1);
    wait_document(&mut p, "main.cpp", "cpp", 1);
    wait_document(&mut p, "header.h", "c", 1);
    p.syntax("header.h", "c");
    let old_context = p.core.index.lock().unwrap().context.clone().unwrap();
    // No mutation of this buffer: grammar, wire and generation must still change.
    let generation = p.core.lsp.as_ref().unwrap().code_action_generation();
    let entries =
        json!([{"directory":p.root,"file":p.root.join("main.c"),"arguments":["clang++"]}]);
    p.put("compile_commands.json", &entries.to_string());
    p.core.reload_index_context();
    p.wait_context("header.h", "cpp");
    let path = p.root.join("header.h");
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(
        !p.core
            .lsp
            .as_ref()
            .unwrap()
            .document_matches(&path, &content)
    );
    p.syntax("header.h", "cpp");
    // Exercise didChange's existing-document path, then read without duplicate sync.
    p.call("lsp.didChange", json!({"path":path,"content":content}));
    let wire = wait_document(&mut p, "header.h", "cpp", 1);
    assert!(p.core.lsp.as_ref().unwrap().code_action_generation() > generation);
    assert_eq!(
        wire.iter().filter(|m| m["method"] == "initialize").count(),
        1
    );
    let closed = wire
        .iter()
        .position(|m| m["method"] == "textDocument/didClose")
        .unwrap();
    let reopened = &wire[closed + 1];
    assert_eq!(reopened["method"], "textDocument/didOpen");
    assert_eq!(reopened["params"]["textDocument"]["languageId"], "cpp");
    assert_eq!(reopened["params"]["textDocument"]["version"], 1);
    let before = wire.len();
    p.call("fs.read", json!({"path":path}));
    // A following query acts as a transport barrier for redundant notifications.
    p.call(
        "lsp.hover",
        json!({"path":path,"content":content,"line":1,"column":5}),
    );
    let after = messages(&p.root.join("wire.jsonl"));
    assert!(
        after[before..]
            .iter()
            .all(|m| m["method"] != "textDocument/didOpen"
                && m["method"] != "textDocument/didChange")
    );
    let mut other = p.root.clone();
    other.push("another-workspace");
    std::fs::create_dir_all(&other).unwrap();
    std::fs::write(other.join("header.h"), &content).unwrap();
    p.call("workspace.open", json!({"path":other}));
    p.wait_ready();
    // Workspace activation deliberately reapplies the real toolchain command.
    fake_server(&mut p);
    let result = p.call(
        "syntaxTree.update",
        json!({"path":other.join("header.h"),"content":content,"version":1}),
    );
    assert_eq!(result["language"], "cpp");
    // A delayed producer for the old workspace cannot determine this document.
    {
        let mut index = p.core.index.lock().unwrap();
        index.root.clone_from(&p.root);
        index.context = Some(old_context);
    }
    let result = p.call(
        "syntaxTree.update",
        json!({"path":other.join("header.h"),"content":content,"version":2}),
    );
    assert_eq!(result["language"], "cpp");
    assert_eq!(
        p.call("index.context", json!({"path":other.join("header.h")}))["language"],
        "cpp"
    );
    wait_document(&mut p, "another-workspace/header.h", "cpp", 1);
}

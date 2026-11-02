//! Code actions through real IPC dispatch, fake LSP suggestions and real compiler.

use kinein_protocol::{JsonRpcRequest, JsonRpcResponse};
use serde_json::{Value, json};
use std::{
    path::PathBuf,
    sync::mpsc,
    time::{Duration, Instant},
};

struct Project {
    core: crate::Core,
    root: PathBuf,
    path: PathBuf,
    before: String,
    _events: mpsc::Receiver<JsonRpcRequest>,
}

impl Project {
    fn new(name: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "kinein-edit-integration-{}-{name}",
            std::process::id()
        ));
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join("main.cpp");
        let before = "/* 😀 */ int value = 1;\n".to_owned();
        std::fs::write(&path, &before).unwrap();
        std::fs::write(root.join("other.cpp"), "int other;\n").unwrap();
        std::fs::write(
            root.join("compile_commands.json"),
            json!([{
                "directory": root, "file": path, "arguments": ["clang++", "-DFLAG=7", "-c", path]
            }])
            .to_string(),
        )
        .unwrap();
        let start = before[..before.find('1').unwrap()].encode_utf16().count();
        let uri = format!("file://{}", path.display());
        let edit = |text: &str| {
            json!({"changes": {uri.clone(): [{
                "range": {"start": {"line":0, "character":start}, "end":{"line":0,"character":start+1}}, "newText":text
            }]}})
        };
        let actions = json!([
            {"title":"Valid Unicode fix", "kind":"quickfix", "edit":edit("FLAG")},
            {"title":"Broken fix", "kind":"quickfix", "edit":edit("missing")},
            {"title":"Multiple files", "kind":"quickfix", "edit":{"changes": {
                uri: [{"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":0}},"newText":"// a\n"}],
                format!("file://{}",root.join("other.cpp").display()): [{"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":0}},"newText":"// b\n"}]
            }}}
        ]);
        let action_file = root.join("actions.json");
        std::fs::write(&action_file, actions.to_string()).unwrap();
        let (sender, events) = mpsc::channel();
        let mut core = super::core_with_empty_search_path(&format!("edit-{name}"));
        core.enable_lsp(sender);
        let mut project = Self {
            core,
            root,
            path,
            before,
            _events: events,
        };
        let response = project.call("workspace.open", json!({"path":project.root}));
        assert!(response.error.is_none(), "{response:?}");
        project.core.use_language_server_command(
            "cpp",
            super::lsp_server::python3(),
            &[
                super::lsp_server::fake_server().to_str().unwrap(),
                project.root.join("wire.jsonl").to_str().unwrap(),
                "--code-actions",
                action_file.to_str().unwrap(),
            ],
        );
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            project.call("fs.read", json!({"path": project.path}));
            if std::fs::read_to_string(project.root.join("wire.jsonl"))
                .unwrap_or_default()
                .contains("textDocument/didOpen")
            {
                break;
            }
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(50));
        }
        project
    }
    fn call(&mut self, method: &str, params: Value) -> JsonRpcResponse {
        self.core
            .handle_request(&JsonRpcRequest::new(1_i64, method, Some(params)))
            .response()
            .clone()
    }
    fn preview(&mut self, index: u64) -> Value {
        let response = self.call(
            "lsp.codeActions",
            json!({"path":self.path,"content":self.before,"line":1,"column":22}),
        );
        assert!(response.error.is_none(), "{response:?}");
        let response = self.call(
            "lsp.applyCodeAction",
            json!({"path":self.path,"content":self.before,"actionIndex":index}),
        );
        assert!(response.error.is_none(), "{response:?}");
        response.result.unwrap()
    }
    fn trust(&mut self) {
        assert!(self.call("workspace.trust", json!({})).error.is_none());
    }
}
impl Drop for Project {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

#[test]
fn edit_verification_preview_and_explicit_apply_use_the_real_transaction() {
    let mut p = Project::new("apply");
    let unknown = p.preview(0);
    assert_eq!(unknown["verification"]["status"], "notVerified");
    assert_eq!(unknown["verification"]["reasonCode"], "untrustedWorkspace");
    p.trust();
    let preview = p.preview(0);
    assert_eq!(preview["verification"]["status"], "noNewError", "{preview}");
    assert!(
        preview["files"][0]["afterContent"]
            .as_str()
            .unwrap()
            .contains("= FLAG;")
    );
    assert_eq!(std::fs::read_to_string(&p.path).unwrap(), p.before);
    let response = p.call(
        "lsp.workspaceEdit.apply",
        json!({"transactionId":preview["transactionId"]}),
    );
    assert!(response.error.is_none(), "{response:?}");
    assert_eq!(
        std::fs::read_to_string(&p.path).unwrap(),
        p.before.replace('1', "FLAG")
    );
}

#[test]
fn edit_verification_new_error_and_multiple_files_remain_reviewable() {
    let mut p = Project::new("broken");
    p.trust();
    let preview = p.preview(1);
    assert_eq!(preview["verification"]["status"], "newError", "{preview}");
    assert!(
        !preview["verification"]["newErrors"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert_eq!(std::fs::read_to_string(&p.path).unwrap(), p.before);
    p.call(
        "lsp.workspaceEdit.cancel",
        json!({"transactionId":preview["transactionId"]}),
    );
    let preview = p.preview(2);
    assert_eq!(preview["verification"]["status"], "notVerified");
    assert!(
        preview["verification"]["reason"]
            .as_str()
            .unwrap()
            .contains("múltiplos")
    );
}

#[test]
fn edit_verification_obsolete_buffer_and_context_refuse_apply() {
    let mut p = Project::new("stale");
    p.trust();
    let preview = p.preview(0);
    p.call(
        "lsp.didChange",
        json!({"path":p.path,"content":format!("{}// edit\n",p.before)}),
    );
    assert!(
        p.call(
            "lsp.workspaceEdit.apply",
            json!({"transactionId":preview["transactionId"]})
        )
        .error
        .is_some()
    );
    assert_eq!(std::fs::read_to_string(&p.path).unwrap(), p.before);
    let preview = p.preview(0);
    let cdb = p.root.join("compile_commands.json");
    let text = std::fs::read_to_string(&cdb)
        .unwrap()
        .replace("-DFLAG=7", "-DFLAG=8");
    std::fs::write(cdb, text).unwrap();
    assert!(
        p.call(
            "lsp.workspaceEdit.apply",
            json!({"transactionId":preview["transactionId"]})
        )
        .error
        .is_some()
    );
    assert_eq!(std::fs::read_to_string(&p.path).unwrap(), p.before);
}

#[test]
fn edit_verification_action_content_must_match_the_query() {
    let mut p = Project::new("query");
    p.trust();
    p.call(
        "lsp.codeActions",
        json!({"path":p.path,"content":p.before,"line":1,"column":22}),
    );
    let response = p.call(
        "lsp.applyCodeAction",
        json!({"path":p.path,"content":format!("{}// changed",p.before),"actionIndex":0}),
    );
    assert!(response.error.is_some());
    assert_eq!(std::fs::read_to_string(&p.path).unwrap(), p.before);
}

#[test]
fn edit_verification_worker_does_not_block_ipc_and_obsolete_reply_cannot_apply() {
    let mut p = Project::new("worker");
    p.trust();
    let slow = crate::write_executable(p.root.join("clang"), "#!/bin/sh\nsleep 20\n");
    let cdb = p.root.join("compile_commands.json");
    let mut entries: Value = serde_json::from_slice(&std::fs::read(&cdb).unwrap()).unwrap();
    entries[0]["arguments"][0] = json!(slow);
    std::fs::write(&cdb, entries.to_string()).unwrap();
    let query = p.call(
        "lsp.codeActions",
        json!({"path":p.path,"content":p.before,"line":1,"column":22}),
    );
    assert!(query.error.is_none(), "{query:?}");
    let (sender, receiver) = mpsc::channel::<crate::Continuation>();
    p.core.enable_continuations(sender);
    let started = Instant::now();
    let response = p.core.handle_request(&JsonRpcRequest::new(
        2_i64,
        "lsp.applyCodeAction",
        Some(json!({"path":p.path,"content":p.before,"actionIndex":0})),
    ));
    assert!(response.is_deferred());
    assert!(p.call("fs.list", json!({"path":p.root})).error.is_none());
    assert!(started.elapsed() < Duration::from_millis(500));
    p.call(
        "lsp.didChange",
        json!({"path":p.path,"content":format!("{}// changed\n",p.before)}),
    );
    let finish = receiver
        .recv_timeout(Duration::from_secs(3))
        .expect("cancelled worker must finish promptly");
    let response = finish(&mut p.core);
    let preview = response.result.unwrap();
    assert_eq!(preview["verification"]["status"], "notVerified");
    assert!(
        p.call(
            "lsp.workspaceEdit.apply",
            json!({"transactionId":preview["transactionId"]})
        )
        .error
        .is_some()
    );
    assert_eq!(std::fs::read_to_string(&p.path).unwrap(), p.before);
}

#[test]
fn edit_verification_late_lsp_query_cannot_restore_actions_for_an_old_version() {
    let mut p = Project::new("late-query");
    p.trust();
    let (sender, receiver) = mpsc::channel::<crate::Continuation>();
    p.core.enable_continuations(sender);
    let query = p.core.handle_request(&JsonRpcRequest::new(
        2_i64,
        "lsp.codeActions",
        Some(json!({"path":p.path,"content":p.before,"line":1,"column":22})),
    ));
    assert!(query.is_deferred());
    p.call(
        "lsp.didChange",
        json!({"path":p.path,"content":format!("{}// changed\n",p.before)}),
    );
    let finish = receiver.recv_timeout(Duration::from_secs(3)).unwrap();
    assert!(finish(&mut p.core).error.is_some());
    assert!(
        p.call(
            "lsp.applyCodeAction",
            json!({"path":p.path,"content":p.before,"actionIndex":0})
        )
        .error
        .is_some()
    );
}

#[test]
fn edit_verification_another_query_cannot_keep_actions_for_old_content() {
    let mut p = Project::new("other-query");
    p.trust();
    p.call(
        "lsp.codeActions",
        json!({"path": p.path, "content": p.before, "line": 1, "column": 22}),
    );
    p.call("lsp.hover", json!({"path": p.path, "content": format!("{}// newer\n", p.before), "line": 1, "column": 1}));
    let response = p.call(
        "lsp.applyCodeAction",
        json!({"path": p.path, "content": p.before, "actionIndex": 0}),
    );
    assert!(
        response.error.is_some(),
        "old actions survived a query that synced newer content"
    );
}

#[test]
fn edit_verification_explicit_cancel_stops_the_worker_and_apply_waits_for_it() {
    let mut p = Project::new("explicit-cancel");
    p.trust();
    let marker = p.root.join("started");
    let program = format!(
        "#!/usr/bin/env python3\nimport pathlib,time\npathlib.Path({}).write_text('started')\ntime.sleep(20)\n",
        serde_json::to_string(&marker).unwrap()
    );
    let slow = crate::write_executable(p.root.join("clang"), program);
    let cdb = p.root.join("compile_commands.json");
    let mut entries: Value = serde_json::from_slice(&std::fs::read(&cdb).unwrap()).unwrap();
    entries[0]["arguments"][0] = json!(slow);
    std::fs::write(cdb, entries.to_string()).unwrap();
    p.call(
        "lsp.codeActions",
        json!({"path":p.path,"content":p.before,"line":1,"column":22}),
    );
    let (sender, receiver) = mpsc::channel::<crate::Continuation>();
    p.core.enable_continuations(sender);
    let response = p.core.handle_request(&JsonRpcRequest::new(
        2_i64,
        "lsp.applyCodeAction",
        Some(json!({"path":p.path,"content":p.before,"actionIndex":0})),
    ));
    assert!(response.is_deferred());
    let deadline = Instant::now() + Duration::from_secs(3);
    while !marker.exists() {
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(
        p.call(
            "lsp.workspaceEdit.apply",
            json!({"transactionId":"workspace-edit-1"})
        )
        .error
        .is_some()
    );
    let cancelled = p.call(
        "lsp.workspaceEdit.cancel",
        json!({"transactionId":"workspace-edit-1"}),
    );
    assert!(cancelled.error.is_none());
    let finished = receiver
        .recv_timeout(Duration::from_secs(2))
        .expect("explicit cancellation must terminate the worker");
    let response = finished(&mut p.core);
    let preview = response.result.unwrap();
    assert_eq!(preview["verification"]["status"], "notVerified");
    assert_eq!(preview["verification"]["obsolete"], true);
    assert_eq!(std::fs::read_to_string(&p.path).unwrap(), p.before);
}

#[test]
fn edit_verification_malformed_utf16_action_cannot_be_verified_or_applied() {
    let mut p = Project::new("invalid-range");
    let path = p.root.join("actions.json");
    let mut actions: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    actions.as_array_mut().unwrap().push(json!({"title":"Invalid range","edit":{"changes":{
        format!("file://{}",p.path.display()):[{"range":{"start":{"line":0,"character":4},"end":{"line":0,"character":5}},"newText":"x"}]
    }}}));
    std::fs::write(path, actions.to_string()).unwrap();
    for trusted in [false, true] {
        if trusted {
            p.trust();
        }
        // Unsupported scope and missing trust must not bypass edit validation.
        for index in [3, 2] {
            if index == 2 {
                let path = p.root.join("actions.json");
                let mut actions: Value =
                    serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
                let uri = format!("file://{}", p.path.display());
                actions[2]["edit"]["changes"][&uri] = actions[3]["edit"]["changes"][&uri].clone();
                std::fs::write(path, actions.to_string()).unwrap();
            }
            let preview = p.preview(index);
            assert_eq!(preview["verification"]["status"], "notVerified");
            assert_eq!(preview["verification"]["obsolete"], true);
            assert!(
                p.call(
                    "lsp.workspaceEdit.apply",
                    json!({"transactionId":preview["transactionId"]})
                )
                .error
                .is_some()
            );
            assert_eq!(std::fs::read_to_string(&p.path).unwrap(), p.before);
        }
    }
}

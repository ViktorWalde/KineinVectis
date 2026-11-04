//! Identidade C/C++ compartilhada por contexto, estrutura local e clangd.

mod context;
mod headers;
mod wire;

use kinein_protocol::JsonRpcRequest;
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    sync::mpsc,
    time::{Duration, Instant},
};

struct Project {
    core: crate::Core,
    root: PathBuf,
    events: mpsc::Receiver<JsonRpcRequest>,
}

impl Project {
    fn new(name: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "kinein C identidade ação {} {name}",
            std::process::id()
        ));
        std::fs::create_dir_all(&root).unwrap();
        let (sender, events) = mpsc::channel();
        let mut core = super::core_with_empty_search_path(&format!("c-identity-{name}"));
        core.enable_lsp(sender);
        Self { core, root, events }
    }

    fn put(&self, path: &str, content: &str) {
        let path = self.root.join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, content).unwrap();
    }

    fn open(&mut self) {
        self.call("workspace.open", json!({"path":self.root}));
        self.wait_ready();
    }

    fn pump(&mut self) {
        while let Ok(event) = self.events.try_recv() {
            self.core.observe_notification(&event);
        }
    }

    fn wait_ready(&mut self) {
        let deadline = Instant::now() + Duration::from_secs(30);
        loop {
            self.pump();
            let root = self.core.workspace_root().unwrap();
            let ready = self.core.index.lock().is_ok_and(|index| {
                index.root == root && matches!(index.state, kinein_protocol::IndexState::Ready)
            });
            if ready {
                return;
            }
            assert!(
                Instant::now() < deadline,
                "current workspace index deadline"
            );
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    fn wait_context(&mut self, file: &str, expected: &str) {
        self.wait_ready();
        let deadline = Instant::now() + Duration::from_secs(30);
        loop {
            self.pump();
            let result = self.call("index.context", json!({"path":self.root.join(file)}));
            if result["language"] == expected {
                return;
            }
            assert!(Instant::now() < deadline, "document context: {result}");
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    fn call(&mut self, method: &str, params: Value) -> Value {
        let response = self
            .core
            .handle_request(&JsonRpcRequest::new(1_i64, method, Some(params)))
            .response()
            .clone();
        assert!(response.error.is_none(), "{method}: {response:?}");
        response.result.unwrap_or_default()
    }

    fn wait(&mut self, method: &str, matches: impl Fn(&Value) -> bool) -> Value {
        let deadline = Instant::now() + Duration::from_secs(30);
        loop {
            let left = deadline
                .checked_duration_since(Instant::now())
                .expect("C identity event deadline");
            let event = self.events.recv_timeout(left).unwrap();
            self.core.observe_notification(&event);
            let params = event.params.unwrap_or_default();
            if event.method == method && matches(&params) {
                return params;
            }
        }
    }

    fn syntax(&mut self, path: &str, expected: &str) -> (Value, Value) {
        let path = self.root.join(path);
        let buffer = std::fs::read_to_string(&path).unwrap();
        let syntax = self.call(
            "syntaxTree.update",
            json!({"path":path,"content":buffer,"version":1}),
        );
        assert_eq!(syntax["language"], expected, "{syntax}");
        let context = self.call("index.context", json!({"path":path}));
        assert_eq!(context["language"], expected, "{context}");
        let relative = path.strip_prefix(&self.root).unwrap().to_str().unwrap();
        assert_eq!(
            self.core.index.lock().unwrap().files[relative].language,
            expected
        );
        (syntax, context)
    }

    fn fixture(&self) {
        let fixture =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/fixtures/c-identity");
        for path in [
            "CMakeLists.txt",
            "include/c_api.h",
            "include/cpp_api.h",
            "include/shared.h",
            "include/orphan.h",
            "src/c_api.c",
            "src/main.cpp",
        ] {
            self.put(path, &std::fs::read_to_string(fixture.join(path)).unwrap());
        }
    }
}

impl Drop for Project {
    fn drop(&mut self) {
        self.core.lsp.as_mut().unwrap().shutdown_all();
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

#[test]
fn c_identity_mixed_targets_grammars_and_clangd_use_real_producers() {
    for (c, cpp) in [("gcc", "g++"), ("clang", "clang++")] {
        let mut p = Project::new(c);
        p.fixture();
        p.open();
        p.call("workspace.trust", json!({}));
        // The existing configure owner reads these explicit cache choices.
        p.put("CMakePresets.json", &json!({"version":6,"configurePresets":[{
            "name":"identity", "generator":"Ninja", "binaryDir":"${sourceDir}/.kinein/build",
            "cacheVariables":{"CMAKE_C_COMPILER":c,"CMAKE_CXX_COMPILER":cpp,"CMAKE_EXPORT_COMPILE_COMMANDS":"ON"}
        }]}).to_string());
        let job = p.call("cmake.configure", json!({"preset":"identity"}))["jobId"].clone();
        assert_eq!(
            p.wait("event.cmake.finished", |v| v["jobId"] == job)["success"],
            true
        );
        p.wait_context("include/c_api.h", "c");
        let job = p.call("build.run", json!({}))["jobId"].clone();
        assert_eq!(
            p.wait("event.build.finished", |v| v["jobId"] == job)["success"],
            true
        );
        let job = p.call("test.run", json!({}))["jobId"].clone();
        assert_eq!(
            p.wait("event.test.finished", |v| v["jobId"] == job)["passed"],
            1
        );
        let (modern_syntax, c_context) = p.syntax("src/c_api.c", "c");
        // Identity is separate from the pinned upstream grammar's C23 coverage.
        eprintln!("{c}: C23 local hasErrors={}", modern_syntax["hasErrors"]);
        assert!(matches!(
            c_context["unit"]["standard"].as_str(),
            Some("c23" | "c2x")
        ));
        p.syntax("src/main.cpp", "cpp");
        let (header_syntax, header) = p.syntax("include/c_api.h", "c");
        assert_eq!(header_syntax["hasErrors"], false, "{header_syntax}");
        assert!(header["unit"].is_null());
        assert_eq!(p.syntax("include/cpp_api.h", "cpp").0["hasErrors"], false);
        let (shared_syntax, shared) = p.syntax("include/shared.h", "cpp");
        assert_eq!(shared_syntax["hasErrors"], false);
        assert!(shared["hint"].as_str().unwrap().contains("ambíguo"));
        p.syntax("include/orphan.h", "cpp");
        assert!(
            p.core
                .use_language_server_command("cpp", "clangd", &["--background-index"])
        );
        let path = p.root.join("include/c_api.h");
        let content = std::fs::read_to_string(&path).unwrap();
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            let outcome = p.core.handle_request(&JsonRpcRequest::new(
                2_i64,
                "lsp.hover",
                Some(json!({"path":path,"content":content,"line":7,"column":6})),
            ));
            if let Some(result) = &outcome.response().result {
                assert!(
                    result["content"]
                        .as_str()
                        .unwrap_or_default()
                        .contains("c_value"),
                    "{result}"
                );
                break;
            }
            assert!(Instant::now() < deadline, "{:?}", outcome.response());
            std::thread::sleep(Duration::from_millis(50));
        }
    }
}

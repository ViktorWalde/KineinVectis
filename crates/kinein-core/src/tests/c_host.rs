//! Contexto C gerado pelo `CMake`, build/teste e verificador existente pelo despacho.

mod standard;

use kinein_protocol::{JsonRpcRequest, JsonRpcResponse};
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
    before: String,
}

impl Project {
    fn new(name: &str) -> Self {
        for tool in ["cmake", "clang", "ninja"] {
            assert!(
                std::process::Command::new(tool)
                    .arg("--version")
                    .output()
                    .is_ok_and(|o| o.status.success()),
                "C project requires {tool}"
            );
        }
        let root =
            std::env::temp_dir().join(format!("kinein C ação {} {name}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/fixtures/c-host");
        for relative in [
            "CMakeLists.txt",
            "CMakePresets.json",
            "include/sample.h",
            "include/sample_config.h.in",
            "src/sum.c",
            "src/scale.c",
            "app/main.c",
            "tests/sample_test.c",
        ] {
            let destination = root.join(relative);
            std::fs::create_dir_all(destination.parent().unwrap()).unwrap();
            std::fs::copy(fixture.join(relative), destination).unwrap();
        }
        let before = std::fs::read_to_string(root.join("app/main.c")).unwrap();
        let uri = format!("file://{}", root.join("app/main.c").display());
        let line = before
            .lines()
            .position(|line| line.contains("sample_sum(3"))
            .unwrap();
        let column = before.lines().nth(line).unwrap().find('3').unwrap();
        let action = |title: &str, text: &str| {
            json!({
                "title": title, "kind": "quickfix", "edit": {"changes": {uri.clone(): [{
                    "range": {"start": {"line":line,"character":column},
                              "end": {"line":line,"character":column+1}}, "newText":text
                }]}}
            })
        };
        std::fs::write(
            root.join("actions.json"),
            json!([
                action("Equivalent C expression", "1 + 2"),
                action("Unknown C identifier", "missing_value")
            ])
            .to_string(),
        )
        .unwrap();
        let (sender, events) = mpsc::channel();
        let mut core = super::core_with_empty_search_path("c-project");
        core.enable_lsp(sender);
        let mut project = Self {
            core,
            root,
            events,
            before,
        };
        project.call("workspace.open", json!({"path": project.root}));
        project.wait("event.index.finished", |_| true);
        project.core.use_language_server_command(
            "cpp",
            super::lsp_server::python3(),
            &[
                super::lsp_server::fake_server().to_str().unwrap(),
                project.root.join("wire.jsonl").to_str().unwrap(),
                "--code-actions",
                project.root.join("actions.json").to_str().unwrap(),
            ],
        );
        project.call("workspace.trust", json!({}));
        project.sync();
        project
    }

    fn response(&mut self, method: &str, params: Value) -> JsonRpcResponse {
        self.core
            .handle_request(&JsonRpcRequest::new(1_i64, method, Some(params)))
            .response()
            .clone()
    }

    fn call(&mut self, method: &str, params: Value) -> Value {
        let response = self.response(method, params);
        assert!(response.error.is_none(), "{method}: {response:?}");
        response.result.unwrap_or(Value::Null)
    }

    fn wait(&mut self, method: &str, matches: impl Fn(&Value) -> bool) -> Value {
        let deadline = Instant::now() + Duration::from_secs(30);
        while let Some(left) = deadline.checked_duration_since(Instant::now()) {
            let event = self
                .events
                .recv_timeout(left)
                .expect("C project event deadline");
            self.core.observe_notification(&event);
            let params = event.params.unwrap_or_default();
            if event.method == method && matches(&params) {
                return params;
            }
        }
        panic!("missing {method}");
    }

    fn pump(&mut self) {
        while let Ok(event) = self.events.try_recv() {
            self.core.observe_notification(&event);
        }
    }

    fn sync(&mut self) {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            self.pump();
            self.call("fs.read", json!({"path": self.root.join("app/main.c")}));
            if self
                .core
                .lsp
                .as_ref()
                .unwrap()
                .document_version(&self.root.join("app/main.c"), None)
                .is_some()
            {
                break;
            }
            assert!(Instant::now() < deadline, "C document not synchronized");
            std::thread::sleep(Duration::from_millis(50));
        }
    }

    fn build(&mut self, success: bool) -> Value {
        let job = self.call("build.run", json!({}))["jobId"].clone();
        let finished = self.wait("event.build.finished", |p| p["jobId"] == job);
        assert_eq!(finished["success"], success, "{finished}");
        finished
    }

    fn context(&mut self, define: &str) -> Value {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            self.pump();
            let value = self.call(
                "index.context",
                json!({"path": self.root.join("app/main.c")}),
            );
            if value["unit"]["defines"]
                .as_array()
                .is_some_and(|defines| defines.contains(&json!(define)))
            {
                return value;
            }
            assert!(
                Instant::now() < deadline,
                "context must follow build: {value}"
            );
            std::thread::sleep(Duration::from_millis(50));
        }
    }

    fn preview(&mut self, index: u64) -> Value {
        self.sync();
        let path = self.root.join("app/main.c");
        self.call(
            "lsp.codeActions",
            json!({"path":path,"content":self.before,"line":5,"column":33}),
        );
        self.call(
            "lsp.applyCodeAction",
            json!({"path":path,"content":self.before,"actionIndex":index}),
        )
    }

    fn test(&mut self) {
        let job = self.call("test.run", json!({}))["jobId"].clone();
        let result = self.wait("event.test.finished", |p| p["jobId"] == job);
        assert_eq!(result["success"], true, "{result}");
        assert_eq!(result["passed"], 1, "{result}");
    }
}

impl Drop for Project {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

#[test]
fn c_host_keeps_build_lsp_context_and_verification_on_the_same_configuration() {
    let mut p = Project::new("cycle");
    p.core.index.lock().unwrap().context = None;
    p.build(true);
    let first = p.context("SAMPLE_BIAS=1");
    assert_eq!(first["language"], "c");
    assert_eq!(first["unit"]["standard"], "c11");
    assert_eq!(first["targets"], json!(["c_host"]));
    assert!(
        first["unit"]["compiler"]
            .as_str()
            .unwrap()
            .contains("clang")
    );
    assert_eq!(p.call("cmake.status", json!({}))["preset"], "host");
    let targets = p.call("cmake.targets.list", json!({}));
    assert_eq!(targets["targets"].as_array().unwrap().len(), 3, "{targets}");
    p.test();
    p.sync();
    let version = p
        .core
        .lsp
        .as_ref()
        .unwrap()
        .document_version(&p.root.join("app/main.c"), None);
    p.build(true);
    p.pump();
    assert_eq!(
        p.core
            .lsp
            .as_ref()
            .unwrap()
            .document_version(&p.root.join("app/main.c"), None),
        version,
        "unchanged CDB must preserve open documents"
    );
    let valid = p.preview(0);
    assert_eq!(valid["verification"]["status"], "noNewError", "{valid}");
    assert_eq!(
        std::fs::read_to_string(p.root.join("app/main.c")).unwrap(),
        p.before
    );
    let broken = p.preview(1);
    assert_eq!(broken["verification"]["status"], "newError", "{broken}");
    let pending = p.preview(0);
    let cmake = p.root.join("CMakeLists.txt");
    let content = std::fs::read_to_string(&cmake).unwrap();
    std::fs::write(
        &cmake,
        content.replace("set(SAMPLE_BIAS 1)", "set(SAMPLE_BIAS 2)"),
    )
    .unwrap();
    assert!(
        p.response(
            "lsp.workspaceEdit.apply",
            json!({"transactionId":pending["transactionId"]})
        )
        .error
        .is_some()
    );
    p.build(true);
    p.context("SAMPLE_BIAS=2");
    p.test();
    let changed = p.preview(0);
    assert_eq!(changed["verification"]["status"], "noNewError", "{changed}");
    assert_ne!(
        changed["verification"]["contextId"],
        valid["verification"]["contextId"]
    );
    let header = p.root.join("include/sample.h");
    let original = std::fs::read_to_string(&header).unwrap();
    std::fs::write(&header, format!("{original}\n#error changed header\n")).unwrap();
    assert!(
        p.response(
            "lsp.workspaceEdit.apply",
            json!({"transactionId":changed["transactionId"]})
        )
        .error
        .is_some()
    );
    std::fs::write(&header, original).unwrap();
    let apply = p.preview(0);
    p.call(
        "lsp.workspaceEdit.apply",
        json!({"transactionId":apply["transactionId"]}),
    );
    assert!(
        std::fs::read_to_string(p.root.join("app/main.c"))
            .unwrap()
            .contains("sample_sum(1 + 2, 4)")
    );
    p.build(true);
    p.test();
    recover_link_error(&mut p);
}

fn recover_link_error(p: &mut Project) {
    let cmake = p.root.join("CMakeLists.txt");
    let source = p.root.join("src/sum.c");
    let original = std::fs::read_to_string(&source).unwrap();
    std::fs::write(
        &source,
        original.replace("int sample_sum(", "int sample_missing("),
    )
    .unwrap();
    let cmake_content = std::fs::read_to_string(&cmake).unwrap();
    std::fs::write(
        &cmake,
        cmake_content.replace("set(SAMPLE_BIAS 2)", "set(SAMPLE_BIAS 3)"),
    )
    .unwrap();
    p.build(false);
    p.context("SAMPLE_BIAS=3");
    std::fs::write(&source, original).unwrap();
    p.build(true);
    p.test();
    p.call("workspace.open", json!({"path":p.root}));
    p.wait("event.index.finished", |_| true);
    p.context("SAMPLE_BIAS=3");
}

#[test]
fn c_host_refreshes_an_initial_server_even_when_the_index_already_has_the_new_cdb() {
    let mut p = Project::new("early-index");
    // Sem CDB em lugar nenhum, o servidor subiu sem pasta e nada mudou ainda:
    // so' a CDB que o build cria o torna desatualizado (C1-16, 2026-11-09).
    assert!(!p.core.lsp.as_ref().unwrap().needs_compile_commands("cpp"));
    let job = p.call("build.run", json!({}))["jobId"].clone();
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        let left = deadline
            .checked_duration_since(Instant::now())
            .expect("build deadline");
        let event = p.events.recv_timeout(left).unwrap();
        if event.method == "event.build.finished" && event.params.as_ref().unwrap()["jobId"] == job
        {
            assert_eq!(event.params.as_ref().unwrap()["success"], true);
            let context = crate::index::context::CompileContext::load_cdb(&p.root);
            assert!(context.has_current_cdb(&p.root));
            p.core.index.lock().unwrap().context = Some(context);
            p.core.observe_notification(&event);
            break;
        }
        p.core.observe_notification(&event);
    }
    assert!(
        !p.core.lsp.as_ref().unwrap().needs_compile_commands("cpp"),
        "server readiness must not be inferred from the index snapshot"
    );
    p.sync();
}

#[test]
fn c_host_refuses_ambiguous_presets_before_configuring() {
    let mut p = Project::new("ambiguous");
    std::fs::write(
        p.root.join("CMakePresets.json"),
        json!({
            "version":3, "configurePresets":[{"name":"first"},{"name":"second"}]
        })
        .to_string(),
    )
    .unwrap();
    let result = p.build(false);
    assert!(
        result["error"].as_str().unwrap().contains("escolha um"),
        "{result}"
    );
    assert!(!p.root.join(".kinein/build/CMakeCache.txt").exists());
    assert!(!p.root.join(".kinein/presets").exists());
}

#[test]
fn c_host_tracks_cdb_content_and_origin_with_restored_metadata() {
    let root = std::env::temp_dir().join(format!("kinein-c-context-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join("compile_commands.json");
    std::fs::write(&path, "[]\n").unwrap();
    let context = crate::index::context::CompileContext::load_cdb(&root);
    assert!(context.has_current_cdb(&root));
    let modified = std::fs::metadata(&path).unwrap().modified().unwrap();
    std::fs::write(&path, " []").unwrap();
    std::fs::File::options()
        .write(true)
        .open(&path)
        .unwrap()
        .set_times(std::fs::FileTimes::new().set_modified(modified))
        .unwrap();
    assert!(
        !context.has_current_cdb(&root),
        "same size/date cannot conceal new bytes"
    );
    std::fs::write(&path, "[]\n").unwrap();
    assert!(context.has_current_cdb(&root));
    std::fs::create_dir_all(root.join(".kinein/build")).unwrap();
    std::fs::write(root.join(".kinein/build/compile_commands.json"), "[]\n").unwrap();
    assert!(
        !context.has_current_cdb(&root),
        "higher-priority CDB changes origin"
    );
    std::fs::remove_dir_all(root).unwrap();
}

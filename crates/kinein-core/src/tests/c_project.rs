//! Criacao C pelo dispatch e dialetos pelo contexto/compiladores reais.

mod case_environment;
mod configuration_types;
mod configurations;
mod dialects;
mod discovered_cases;
mod environment;
mod existing;
mod existing_cdb;
mod existing_kit;
mod existing_recovery;
mod inner_cases;
mod preset_condition;
mod preset_include;
mod preset_macros;
mod preset_schema;

use kinein_protocol::JsonRpcRequest;
use serde_json::{Value, json};
use std::{
    path::PathBuf,
    sync::mpsc,
    time::{Duration, Instant},
};

struct Project {
    core: crate::Core,
    events: mpsc::Receiver<JsonRpcRequest>,
    parent: PathBuf,
    root: PathBuf,
}

impl Project {
    fn create(compiler: &str) -> Self {
        let parent = std::env::temp_dir().join(format!(
            "kinein novo C ação {} {compiler}",
            std::process::id()
        ));
        std::fs::create_dir_all(&parent).unwrap();
        let root = parent.join("starter");
        let (sender, events) = mpsc::channel();
        let mut core = super::core_with_empty_search_path("c-project");
        core.enable_lsp(sender);
        let mut project = Self {
            core,
            events,
            parent,
            root,
        };
        let created = project.call(
            "workspace.createProject",
            json!({"parent":project.parent,"name":"starter","template":"cCmake","options":{"standard":"23"}}),
        );
        assert_eq!(created["kind"], "cmake");
        assert_eq!(created["trusted"], true);
        assert!(project.root.join("src/main.c").is_file());
        assert!(!project.root.join("src/main.cpp").exists());
        project.wait("event.index.finished", |_| true);
        let presets = project.root.join("CMakePresets.json");
        let mut value: Value =
            serde_json::from_str(&std::fs::read_to_string(&presets).unwrap()).unwrap();
        for preset in value["configurePresets"].as_array_mut().unwrap() {
            preset["cacheVariables"]["CMAKE_C_COMPILER"] = json!(compiler);
        }
        std::fs::write(&presets, serde_json::to_string_pretty(&value).unwrap()).unwrap();
        project
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
                .expect("C project deadline");
            let event = self.events.recv_timeout(left).unwrap();
            self.core.observe_notification(&event);
            let params = event.params.unwrap_or_default();
            if event.method == method && matches(&params) {
                return params;
            }
        }
    }

    fn build(&mut self, success: bool) {
        let job = self.call("build.run", json!({}))["jobId"].clone();
        let result = self.wait("event.build.finished", |p| p["jobId"] == job);
        assert_eq!(result["success"], success, "{result}");
    }

    fn standard(&mut self, value: &str) {
        let params = json!({"target":"starter", "standard":value});
        let before = std::fs::read_to_string(self.root.join("CMakeLists.txt")).unwrap();
        let plan = self.call(
            "configAction.preview",
            json!({"id":"cmake.setCStandard", "params":params}),
        );
        assert_eq!(
            std::fs::read_to_string(self.root.join("CMakeLists.txt")).unwrap(),
            before
        );
        self.call(
            "configAction.apply",
            json!({"id":"cmake.setCStandard", "params":params,
            "expected":[{"path":"CMakeLists.txt", "content":plan["files"][0]["before"]}]}),
        );
    }

    fn context(&mut self, standard: &str) {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            while let Ok(event) = self.events.try_recv() {
                self.core.observe_notification(&event);
            }
            let context = self.call(
                "index.context",
                json!({"path":self.root.join("src/main.c")}),
            );
            let observed = context["unit"]["standard"].as_str().unwrap_or_default();
            if observed == format!("c{standard}") || (standard == "23" && observed == "c2x") {
                assert_eq!(context["language"], "c");
                assert_eq!(context["targets"], json!(["starter"]));
                return;
            }
            assert!(Instant::now() < deadline, "context: {context}");
            std::thread::sleep(Duration::from_millis(25));
        }
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
        let _ = std::fs::remove_dir_all(&self.parent);
    }
}

#[test]
fn c_project_creation_build_context_standards_and_recovery_use_real_tools() {
    for compiler in ["gcc", "clang"] {
        let mut project = Project::create(compiler);
        let configured =
            project.call("cmake.configure", json!({"preset":"debug"}))["jobId"].clone();
        let result = project.wait("event.cmake.finished", |p| p["jobId"] == configured);
        assert_eq!(result["success"], true, "{result}");
        project.build(true);
        project.context("23");
        project.test();
        for standard in ["90", "99", "11", "17", "23"] {
            project.standard(standard);
            project.build(true);
            project.context(standard);
            project.test();
        }
        // A IDE aceita o pedido futuro; o produtor atual deve recusa-lo sem reduzir.
        project.standard("2099");
        project.build(false);
        assert!(
            std::fs::read_to_string(project.root.join("CMakeLists.txt"))
                .unwrap()
                .contains("C_STANDARD 2099")
        );
        project.standard("23");
        project.build(true);
        project.context("23");
        project.test();
        let source = project.root.join("src/main.c");
        let original = std::fs::read_to_string(&source).unwrap();
        let collision = project.core.handle_request(&JsonRpcRequest::new(
            2_i64,
            "workspace.createProject",
            Some(json!({"parent":project.parent,"name":"starter","template":"cCmake","options":{"standard":"23"}})),
        ));
        assert!(collision.response().error.is_some());
        assert_eq!(std::fs::read_to_string(source).unwrap(), original);
    }
}

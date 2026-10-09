//! Logical run identity and single live run, through actual PTY processes.
use std::{path::PathBuf, sync::mpsc, time::Duration};

use kinein_protocol::{JsonRpcErrorCode, JsonRpcRequest};
use serde_json::{Value, json};

use super::{core_with_empty_search_path, terminal_run_until_closed};

struct Fixture {
    core: crate::Core,
    root: PathBuf,
    events: mpsc::Receiver<JsonRpcRequest>,
}

impl Fixture {
    fn new(tag: &str) -> Self {
        let root = std::env::temp_dir().join(format!("kinein-run-{tag}-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let root = crate::platform::canonicalize(&root).unwrap();
        let (sender, events) = mpsc::channel();
        let mut core = core_with_empty_search_path(tag);
        core.enable_lsp(sender);
        let opened = core.handle_request(&JsonRpcRequest::new(
            1_i64,
            "workspace.open",
            Some(json!({"path": root})),
        ));
        assert!(opened.response().error.is_none());
        Self { core, root, events }
    }

    fn run(&mut self, method: &str, params: Value) -> Value {
        let result = self
            .core
            .handle_request(&JsonRpcRequest::new(2_i64, method, Some(params)));
        assert!(
            result.response().error.is_none(),
            "{:?}",
            result.response().error
        );
        result.response().result.as_ref().unwrap().clone()
    }

    fn finish(&self, result: &Value) {
        let (lines, code) = terminal_run_until_closed(
            &self.events,
            result["terminalId"].as_str().unwrap(),
            Duration::from_secs(10),
        );
        assert_eq!(code, Some(0), "{lines:?}");
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = self
            .core
            .handle_request(&JsonRpcRequest::new(99_i64, "workspace.close", None));
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

#[test]
#[cfg(unix)]
fn canonical_file_identity_survives_retries_but_pty_ids_do_not() {
    let mut fixture = Fixture::new("reuse-script");
    std::fs::create_dir(fixture.root.join("other")).unwrap();
    let script = fixture.root.join("check it's;safe.sh");
    std::fs::write(&script, "printf 'tentativa segura\\n'\n").unwrap();
    std::fs::write(fixture.root.join("other/check it's;safe.sh"), "exit 0\n").unwrap();
    let first = fixture.run("run.script", json!({"path": script}));
    fixture.finish(&first);
    std::os::unix::fs::symlink(&script, fixture.root.join("alias.sh")).unwrap();
    let again = fixture.run("run.script", json!({"path": fixture.root.join("alias.sh")}));
    fixture.finish(&again);
    assert_ne!(first["terminalId"], again["terminalId"]);
    assert_eq!(first["executionKey"], again["executionKey"]);
    assert_eq!(first["workspace"], fixture.root.to_str().unwrap());
    assert_eq!(
        serde_json::from_str::<Value>(first["executionKey"].as_str().unwrap()).unwrap(),
        json!(["script", fixture.root, script])
    );
    let other = fixture.run(
        "run.script",
        json!({"path": fixture.root.join("other/check it's;safe.sh")}),
    );
    fixture.finish(&other);
    assert_ne!(first["executionKey"], other["executionKey"]);
}

#[test]
fn rejects_a_second_live_execution_before_spawn() {
    let mut fixture = Fixture::new("reuse-live");
    let script = fixture.root.join("wait.sh");
    std::fs::write(
        &script,
        "printf 'primeira unica\\n'\nread -r answer\nprintf 'fim\\n'\n",
    )
    .unwrap();
    let first = fixture.run("run.script", json!({"path": script}));
    for (method, params) in [
        ("run.script", json!({"path": script})),
        ("run.start", json!({"command": "touch segunda-proibida"})),
    ] {
        let denied = fixture
            .core
            .handle_request(&JsonRpcRequest::new(3_i64, method, Some(params)));
        assert_eq!(
            denied.response().error.as_ref().map(|error| error.code),
            Some(JsonRpcErrorCode::InvalidRequest)
        );
    }
    assert!(!fixture.root.join("segunda-proibida").exists());
    fixture.run(
        "terminal.input",
        json!({"id": first["terminalId"], "data": "pronto\n"}),
    );
    let (lines, code) = terminal_run_until_closed(
        &fixture.events,
        first["terminalId"].as_str().unwrap(),
        Duration::from_secs(10),
    );
    assert_eq!(code, Some(0));
    assert_eq!(
        lines
            .iter()
            .filter(|line| line.contains("primeira unica"))
            .count(),
        1
    );
    let next = fixture.run("run.start", json!({"command": "printf 'nova\\n'"}));
    fixture.finish(&next);
}

#[test]
fn config_command_and_workspace_have_separate_stable_identities() {
    let mut fixture = Fixture::new("reuse-config");
    let config =
        crate::runconfig::save(&fixture.root, None, "Minha configuração", "printf 'v1\\n'")
            .unwrap();
    let id = config.active_id.as_ref().unwrap();
    let first = fixture.run("run.start", json!({}));
    fixture.finish(&first);
    crate::runconfig::save(&fixture.root, Some(id), "Outro nome", "printf 'v2\\n'").unwrap();
    let again = fixture.run("run.start", json!({}));
    fixture.finish(&again);
    assert_eq!(first["executionKey"], again["executionKey"]);
    assert_ne!(first["command"], again["command"]);
    let command = fixture.run("run.start", json!({"command": "printf 'v2\\n'"}));
    fixture.finish(&command);
    assert_ne!(again["executionKey"], command["executionKey"]);
    let repeated = fixture.run("run.start", json!({"command": "printf 'v2\\n'"}));
    fixture.finish(&repeated);
    assert_eq!(command["executionKey"], repeated["executionKey"]);
    let other = fixture.run("run.start", json!({"command": "printf 'v3\\n'"}));
    fixture.finish(&other);
    assert_ne!(command["executionKey"], other["executionKey"]);
    let mut workspace = Fixture::new("reuse-other-workspace");
    let elsewhere = workspace.run("run.start", json!({"command": "printf 'v2\\n'"}));
    workspace.finish(&elsewhere);
    assert_ne!(command["executionKey"], elsewhere["executionKey"]);
}

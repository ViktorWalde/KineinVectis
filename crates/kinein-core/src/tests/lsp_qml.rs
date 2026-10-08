//! O `qmlls` do projeto (59 §2.4, 2026-10-08): o core passa o build como `-b`
//! e o `.qml` abre no servidor. O `qmlls` daqui e' falso: grava os proprios
//! argumentos e entrega a conversa ao `scripts/fake_lsp_server.py`, que grava o
//! wire. Assim o teste olha o que o processo RECEBEU, nao a resposta do IPC.

use std::{
    path::PathBuf,
    sync::mpsc,
    time::{Duration, Instant},
};

use kinein_protocol::JsonRpcRequest;
use serde_json::{Value, json};

use crate::lsp;

const DEADLINE: Duration = Duration::from_secs(10);

struct Setup {
    core: crate::Core,
    root: PathBuf,
    args_file: PathBuf,
    log: PathBuf,
    _events: mpsc::Receiver<JsonRpcRequest>,
}

fn setup(name: &str, configured: bool) -> Setup {
    let root = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-qml-{name}", std::process::id()));
    if root.exists() {
        std::fs::remove_dir_all(&root).unwrap();
    }
    std::fs::create_dir_all(root.join("ui")).unwrap();
    std::fs::write(root.join("ui/Main.qml"), "import QtQuick\n\nItem {}\n").unwrap();
    if configured {
        std::fs::create_dir_all(root.join(".kinein/build")).unwrap();
        std::fs::write(root.join(".kinein/build/CMakeCache.txt"), "# teste\n").unwrap();
    }
    let root = root.canonicalize().unwrap();
    let args_file = root.join("qmlls-args.txt");
    let log = root.join("qml-wire.jsonl");

    let detector_name = format!("qml-{name}");
    let mut core = super::core_with_empty_search_path(&detector_name);
    // O PATH do detector e' a pasta que `core_with_empty_search_path` criou.
    let bin = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-dispatch-{detector_name}", std::process::id()));
    crate::write_executable(
        bin.join("qmlls"),
        format!(
            "#!/bin/sh\nprintf '%s\\n' \"$@\" > '{}'\nexec {} '{}' '{}'\n",
            args_file.display(),
            super::lsp_server::python3(),
            super::lsp_server::fake_server().display(),
            log.display()
        ),
    );
    let (sender, receiver) = mpsc::channel::<JsonRpcRequest>();
    core.enable_lsp(sender as lsp::EventSender);
    let opened = core.handle_request(&JsonRpcRequest::new(
        1_i64,
        "workspace.open",
        Some(json!({ "path": root.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none(), "workspace.open falhou");
    Setup {
        core,
        root,
        args_file,
        log,
        _events: receiver,
    }
}

impl Setup {
    fn open_main_qml(&mut self) {
        let path = self.root.join("ui/Main.qml").display().to_string();
        let outcome = self.core.handle_request(&JsonRpcRequest::new(
            2_i64,
            "fs.read",
            Some(json!({ "path": path })),
        ));
        assert!(outcome.response().error.is_none(), "fs.read falhou");
    }

    /// Os argumentos da ultima subida do `qmlls`, quando ela gravou `want`.
    fn args_when(&self, want: impl Fn(&[String]) -> bool) -> Vec<String> {
        let deadline = Instant::now() + DEADLINE;
        let mut last = Vec::new();
        while Instant::now() < deadline {
            if let Ok(body) = std::fs::read_to_string(&self.args_file) {
                last = body
                    .lines()
                    .filter(|line| !line.is_empty())
                    .map(str::to_owned)
                    .collect();
                if want(&last) {
                    return last;
                }
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        panic!("o qmlls nao subiu com os argumentos esperados; ultimos: {last:?}");
    }

    fn did_open(&self) -> Value {
        let deadline = Instant::now() + DEADLINE;
        while Instant::now() < deadline {
            let body = std::fs::read_to_string(&self.log).unwrap_or_default();
            if let Some(message) = body
                .lines()
                .filter_map(|line| serde_json::from_str::<Value>(line).ok())
                .find(|m| m["method"] == "textDocument/didOpen")
            {
                return message;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        panic!("o .qml nao chegou ao servidor em {DEADLINE:?}");
    }
}

#[test]
fn a_configured_project_gives_qmlls_its_build_directory() {
    let mut setup = setup("b", true);
    setup.open_main_qml();
    let build = setup.root.join(".kinein/build").display().to_string();
    assert_eq!(setup.args_when(|args| !args.is_empty()), ["-b", &build]);
    let document = &setup.did_open()["params"]["textDocument"];
    assert_eq!(document["languageId"], "qml");
    assert!(document["uri"].as_str().unwrap().ends_with("/ui/Main.qml"));
}

#[test]
fn without_a_build_qmlls_starts_with_the_qt_modules_only() {
    let mut setup = setup("sem-build", false);
    setup.open_main_qml();
    assert_eq!(
        setup.did_open()["params"]["textDocument"]["languageId"],
        "qml"
    );
    assert!(setup.args_when(|_| true).is_empty());
}

#[test]
fn a_finished_configure_restarts_qmlls_with_the_new_build() {
    let mut setup = setup("configure", false);
    setup.open_main_qml();
    setup.did_open();
    assert!(setup.args_when(|_| true).is_empty());

    std::fs::create_dir_all(setup.root.join(".kinein/build")).unwrap();
    std::fs::write(setup.root.join(".kinein/build/CMakeCache.txt"), "# teste\n").unwrap();
    // Pelo loop real, como o job do configure: a reacao mora no loop.
    let (sender, inbox) = mpsc::channel::<crate::runtime::LoopEvent>();
    sender
        .send(crate::runtime::LoopEvent::Notification(Box::new(
            JsonRpcRequest::notification(
                "event.cmake.finished",
                Some(json!({ "jobId": "job-1", "success": true })),
            ),
        )))
        .unwrap();
    drop(sender);
    let mut out = Vec::new();
    crate::runtime::drain_loop_events(&mut setup.core, &mut out, &inbox).unwrap();
    setup.open_main_qml();
    let build = setup.root.join(".kinein/build").display().to_string();
    assert_eq!(setup.args_when(|args| !args.is_empty()), ["-b", &build]);
}

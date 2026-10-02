//! O stderr do language server (2026-09-17; lacuna do `roadmaps/40` §4).
//!
//! Ate' aqui o servidor nascia com o stderr em /dev/null: um clangd dizendo
//! "`compile_commands.json` not found" ou um basedpyright que morria no
//! `import` eram, para a IDE, um `status: failed` sem motivo. O que se prova
//! com o servidor falso: cada linha do stderr sai como `event.lsp.log`; o
//! servidor que morre antes do `initialize` deixa a cauda no `status: failed`
//! (e no erro), e o que sai depois de subir deixa a cauda no `status: exited`.
//!
//! E (2026-10-01, F0 da 0.3.6) o contrario: o servidor que o PROPRIO core
//! encerrou — workspace fechado, `lsp.restart` — nao e' queda. Ate' aqui a
//! thread leitora via o fim do stdout depois do `stopped` e anunciava
//! `exited`, e a barra de status pintava "LSP ✗" na tela inicial.

use std::{sync::mpsc, time::Duration};

use kinein_protocol::JsonRpcRequest;
use serde_json::{Value, json};

use super::lsp_server::{fake_server, python3};
use crate::lsp;

struct Scenario {
    core: crate::Core,
    app: String,
    events: mpsc::Receiver<JsonRpcRequest>,
}

fn scenario(name: &str, extra: &[&str]) -> Scenario {
    let root = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-lspstderr-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("pyproject.toml"), "[project]\nname = \"demo\"\n").unwrap();
    std::fs::write(root.join("app.py"), "import os\n").unwrap();
    let root = root.canonicalize().unwrap();
    let log = root.join("principal.jsonl");

    let (sender, receiver) = mpsc::channel::<JsonRpcRequest>();
    let mut core = super::core_with_empty_search_path(&format!("lspstderr-{name}"));
    core.enable_lsp(sender as lsp::EventSender);
    let opened = core.handle_request(&JsonRpcRequest::new(
        1_i64,
        "workspace.open",
        Some(json!({ "path": root.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none(), "workspace.open falhou");
    let script = fake_server();
    let mut args = vec![
        script.to_str().unwrap().to_owned(),
        log.display().to_string(),
    ];
    args.extend(extra.iter().map(|a| (*a).to_owned()));
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    assert!(core.use_language_server_command("python", python3(), &refs));
    Scenario {
        core,
        app: root.join("app.py").display().to_string(),
        events: receiver,
    }
}

impl Scenario {
    /// Abre o arquivo: e' o que sobe o servidor.
    fn open_app(&mut self) -> kinein_protocol::JsonRpcResponse {
        self.core
            .handle_request(&JsonRpcRequest::new(
                2_i64,
                "fs.read",
                Some(json!({ "path": self.app })),
            ))
            .response()
            .clone()
    }

    /// Eventos `event.lsp.*` recebidos ate' um deles satisfazer `pronto`.
    fn events_until(&self, done: impl Fn(&JsonRpcRequest) -> bool) -> Vec<(String, Value)> {
        let deadline = std::time::Instant::now() + Duration::from_secs(10);
        let mut seen = Vec::new();
        while std::time::Instant::now() < deadline {
            match self.events.recv_timeout(Duration::from_millis(50)) {
                Ok(event)
                    if event
                        .method
                        .strip_prefix("event.")
                        .is_some_and(|m| m.starts_with("lsp")) =>
                {
                    let finished = done(&event);
                    seen.push((
                        event.method.clone(),
                        event.params.clone().unwrap_or(Value::Null),
                    ));
                    if finished {
                        return seen;
                    }
                }
                Ok(_) | Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
        panic!("o evento esperado nao chegou; vistos: {seen:?}");
    }

    /// Todos os `event.lsp.status` que chegam durante `janela`.
    fn statuses_during(&self, window: Duration) -> Vec<String> {
        let deadline = std::time::Instant::now() + window;
        let mut seen = Vec::new();
        while let Some(left) = deadline.checked_duration_since(std::time::Instant::now()) {
            match self
                .events
                .recv_timeout(left.min(Duration::from_millis(50)))
            {
                Ok(event) if event.method == "event.lsp.status" => {
                    let params = event.params.unwrap_or(Value::Null);
                    seen.push(params["status"].as_str().unwrap_or("").to_owned());
                }
                Ok(_) | Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
        seen
    }

    fn wait_running(&self) {
        self.events_until(|e| {
            e.method == "event.lsp.status" && e.params.as_ref().unwrap()["status"] == "running"
        });
    }
}

/// Cada linha do stderr vira `event.lsp.log { language, line }`, na ordem, e
/// o servidor sobe normalmente (`status: running`).
#[test]
fn stderr_lines_become_lsp_log_events_and_the_server_still_runs() {
    let mut c = scenario("log", &["--stderr", "3"]);
    assert!(c.open_app().error.is_none());
    // O stderr e' lido por OUTRA thread: a terceira linha pode chegar depois
    // do `running` (medido em 2026-09-17: falhava 1 em ~3 rodadas). Espera
    // as duas coisas — o servidor de pe' E as tres linhas.
    let logs_seen = std::cell::Cell::new(0_u32);
    let running_seen = std::cell::Cell::new(false);
    let seen = c.events_until(|e| {
        if e.method == "event.lsp.log" {
            logs_seen.set(logs_seen.get() + 1);
        }
        if e.method == "event.lsp.status" && e.params.as_ref().unwrap()["status"] == "running" {
            running_seen.set(true);
        }
        running_seen.get() && logs_seen.get() >= 3
    });
    let logs: Vec<String> = seen
        .iter()
        .filter(|(m, _)| m == "event.lsp.log")
        .map(|(_, p)| {
            assert_eq!(p["language"], "python");
            p["line"].as_str().unwrap().to_owned()
        })
        .collect();
    assert_eq!(
        logs,
        [
            "fake-lsp stderr 1",
            "fake-lsp stderr 2",
            "fake-lsp stderr 3"
        ]
    );
}

/// O servidor que morre no berco: `status: failed` leva a cauda do stderr
/// (o motivo), e a resposta do `fs.read` nao falha — o LSP e' companhia,
/// nao pre-requisito de abrir um arquivo.
#[test]
fn a_server_that_dies_before_initialize_leaves_its_stderr_in_the_failed_status() {
    let mut c = scenario("morre", &["--stderr", "2", "--morre"]);
    assert!(c.open_app().error.is_none());
    let seen = c.events_until(|e| {
        e.method == "event.lsp.status" && e.params.as_ref().unwrap()["status"] == "failed"
    });
    let (_, failure) = seen.last().unwrap();
    let message = failure["message"].as_str().unwrap();
    assert!(
        message.contains("--- stderr do servidor ---")
            && message.contains("fake-lsp stderr 1")
            && message.contains("fake-lsp stderr 2"),
        "{message}"
    );
    // As linhas tambem sairam ao vivo, antes do status.
    assert_eq!(seen.iter().filter(|(m, _)| m == "event.lsp.log").count(), 2);
}

/// O servidor que CAI depois de subir (sozinho: `--cai`) deixa a cauda no
/// `status: exited` — e' o `message` que a faixa de saude mostra. Ate'
/// 2026-10-01 este teste matava o servidor pelo `lsp.restart` e chamava isso
/// de saida: era o defeito, travado como contrato.
#[test]
fn a_server_that_exits_leaves_its_stderr_in_the_exited_status() {
    let mut c = scenario("sai", &["--stderr", "1", "--cai"]);
    assert!(c.open_app().error.is_none());
    let seen = c.events_until(|e| {
        e.method == "event.lsp.status" && e.params.as_ref().unwrap()["status"] == "exited"
    });
    let (_, exited) = seen.last().unwrap();
    assert_eq!(exited["language"], "python");
    assert_eq!(exited["message"], "fake-lsp stderr 1");
}

/// Fechar o workspace encerra o servidor DE PROPOSITO: `stopped`, e nada de
/// `exited` nem `failed` depois — nem com o fim do stdout chegando a' thread
/// leitora quando o `stopped` ja' saiu.
#[test]
fn closing_the_workspace_stops_the_server_without_reporting_a_crash() {
    let mut c = scenario("fecha", &["--stderr", "1"]);
    assert!(c.open_app().error.is_none());
    c.wait_running();
    let closed = c
        .core
        .handle_request(&JsonRpcRequest::new(4_i64, "workspace.close", None))
        .response()
        .clone();
    assert!(closed.error.is_none(), "{:?}", closed.error);
    let statuses = c.statuses_during(Duration::from_millis(1500));
    assert_eq!(statuses, ["stopped"], "depois do workspace.close");
}

/// `lsp.restart` tambem: `restarting` e o novo `running`; a morte do antigo
/// nao vira `exited`.
#[test]
fn restarting_the_server_does_not_report_the_old_one_as_crashed() {
    let mut c = scenario("reinicia", &["--stderr", "1"]);
    assert!(c.open_app().error.is_none());
    c.wait_running();
    let restarted = c
        .core
        .handle_request(&JsonRpcRequest::new(
            5_i64,
            "lsp.restart",
            Some(json!({ "language": "python" })),
        ))
        .response()
        .clone();
    assert!(restarted.error.is_none(), "{:?}", restarted.error);
    let statuses = c.statuses_during(Duration::from_millis(1500));
    assert!(
        !statuses.iter().any(|s| s == "exited" || s == "failed"),
        "depois do lsp.restart: {statuses:?}"
    );
    assert_eq!(statuses.first().map(String::as_str), Some("restarting"));
}

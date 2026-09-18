//! O stderr do language server (2026-09-17; lacuna do `roadmaps/40` §4).
//!
//! Ate' aqui o servidor nascia com o stderr em /dev/null: um clangd dizendo
//! "`compile_commands.json` not found" ou um basedpyright que morria no
//! `import` eram, para a IDE, um `status: failed` sem motivo. O que se prova
//! com o servidor falso: cada linha do stderr sai como `event.lsp.log`; o
//! servidor que morre antes do `initialize` deixa a cauda no `status: failed`
//! (e no erro), e o que sai depois de subir deixa a cauda no `status: exited`.

use std::{sync::mpsc, time::Duration};

use kinein_protocol::JsonRpcRequest;
use serde_json::{Value, json};

use super::lsp_server::{fake_server, python3};
use crate::lsp;

struct Cenario {
    core: crate::Core,
    app: String,
    events: mpsc::Receiver<JsonRpcRequest>,
}

fn cenario(nome: &str, extra: &[&str]) -> Cenario {
    let root = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-lspstderr-{nome}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("pyproject.toml"), "[project]\nname = \"demo\"\n").unwrap();
    std::fs::write(root.join("app.py"), "import os\n").unwrap();
    let root = root.canonicalize().unwrap();
    let log = root.join("principal.jsonl");

    let (sender, receiver) = mpsc::channel::<JsonRpcRequest>();
    let mut core = super::core_with_empty_search_path(&format!("lspstderr-{nome}"));
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
    Cenario {
        core,
        app: root.join("app.py").display().to_string(),
        events: receiver,
    }
}

impl Cenario {
    /// Abre o arquivo: e' o que sobe o servidor.
    fn abrir(&mut self) -> kinein_protocol::JsonRpcResponse {
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
    fn eventos_ate(&self, pronto: impl Fn(&JsonRpcRequest) -> bool) -> Vec<(String, Value)> {
        let prazo = std::time::Instant::now() + Duration::from_secs(10);
        let mut vistos = Vec::new();
        while std::time::Instant::now() < prazo {
            match self.events.recv_timeout(Duration::from_millis(50)) {
                Ok(event)
                    if event
                        .method
                        .strip_prefix("event.")
                        .is_some_and(|m| m.starts_with("lsp")) =>
                {
                    let fim = pronto(&event);
                    vistos.push((
                        event.method.clone(),
                        event.params.clone().unwrap_or(Value::Null),
                    ));
                    if fim {
                        return vistos;
                    }
                }
                Ok(_) | Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
        panic!("o evento esperado nao chegou; vistos: {vistos:?}");
    }
}

/// Cada linha do stderr vira `event.lsp.log { language, line }`, na ordem, e
/// o servidor sobe normalmente (`status: running`).
#[test]
fn stderr_lines_become_lsp_log_events_and_the_server_still_runs() {
    let mut c = cenario("log", &["--stderr", "3"]);
    assert!(c.abrir().error.is_none());
    // O stderr e' lido por OUTRA thread: a terceira linha pode chegar depois
    // do `running` (medido em 2026-09-17: falhava 1 em ~3 rodadas). Espera
    // as duas coisas — o servidor de pe' E as tres linhas.
    let logs_vistos = std::cell::Cell::new(0_u32);
    let running_visto = std::cell::Cell::new(false);
    let vistos = c.eventos_ate(|e| {
        if e.method == "event.lsp.log" {
            logs_vistos.set(logs_vistos.get() + 1);
        }
        if e.method == "event.lsp.status" && e.params.as_ref().unwrap()["status"] == "running" {
            running_visto.set(true);
        }
        running_visto.get() && logs_vistos.get() >= 3
    });
    let logs: Vec<String> = vistos
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
    let mut c = cenario("morre", &["--stderr", "2", "--morre"]);
    assert!(c.abrir().error.is_none());
    let vistos = c.eventos_ate(|e| {
        e.method == "event.lsp.status" && e.params.as_ref().unwrap()["status"] == "failed"
    });
    let (_, falha) = vistos.last().unwrap();
    let mensagem = falha["message"].as_str().unwrap();
    assert!(
        mensagem.contains("--- stderr do servidor ---")
            && mensagem.contains("fake-lsp stderr 1")
            && mensagem.contains("fake-lsp stderr 2"),
        "{mensagem}"
    );
    // As linhas tambem sairam ao vivo, antes do status.
    assert_eq!(
        vistos.iter().filter(|(m, _)| m == "event.lsp.log").count(),
        2
    );
}

/// O servidor que SAI depois de subir (aqui: morto pelo `lsp.restart`) deixa
/// a cauda no `status: exited` — e' o `message` que a faixa de saude mostra.
#[test]
fn a_server_that_exits_leaves_its_stderr_in_the_exited_status() {
    let mut c = cenario("sai", &["--stderr", "1"]);
    assert!(c.abrir().error.is_none());
    c.eventos_ate(|e| {
        e.method == "event.lsp.status" && e.params.as_ref().unwrap()["status"] == "running"
    });
    let reiniciado = c
        .core
        .handle_request(&JsonRpcRequest::new(
            3_i64,
            "lsp.restart",
            Some(json!({ "language": "python" })),
        ))
        .response()
        .clone();
    assert!(reiniciado.error.is_none(), "{:?}", reiniciado.error);
    let vistos = c.eventos_ate(|e| {
        e.method == "event.lsp.status" && e.params.as_ref().unwrap()["status"] == "exited"
    });
    let (_, saiu) = vistos.last().unwrap();
    assert_eq!(saiu["language"], "python");
    assert_eq!(saiu["message"], "fake-lsp stderr 1");
}

//! Integration tests for `Core` request dispatch, grouped by domain.
//!
//! Declared as `#[cfg(test)] mod tests;` in `lib.rs`, so `crate::` reaches the
//! core surface and every submodule shares [`core_with_empty_search_path`].

use crate::EXECUTAVEIS;

mod build;
mod cargo;
mod cmake;
mod cmake_model;
mod configaction;
mod container;
mod coverage;
mod datasource;
mod datasource_discover;
mod datasource_query;
mod debug;
mod debug_attach;
mod debug_inspect;
mod dispatch;
mod flash_proposal;
mod format;
mod frameworks;
mod fs;
mod git;
mod grafana;
mod index;
mod index_context;
mod jobs;
mod lsp;
mod lsp_companion;
mod lsp_deferred;
mod lsp_server;
mod lsp_stderr;
mod project;
mod python;
mod remote;
mod remote_mirror;
mod run;
mod runconfig;
mod runners;
mod serial;
mod serial_files;
mod serial_identify;
mod settings;
mod syntax;
mod terminal;
mod toolchain;
mod tools;
mod workspace;

use crate::Core;
use crate::tools::ToolDetector;

/// Builds a `Core` whose tool detector searches an empty temp directory, so
/// detection is deterministic and never finds host tools.
/// As linhas de um `event.terminal.render`, DESENROLADAS: uma linha logica
/// que passou das colunas do grid continua na linha seguinte sem quebra —
/// aqui as duas voltam a ser uma (o caminho longo de um `mpremote run
/// /tmp/.../util.py` e' o caso).
fn render_lines(event: &kinein_protocol::JsonRpcRequest) -> Vec<String> {
    let params = event.params.as_ref();
    let cols = params
        .and_then(|p| p["cols"].as_u64())
        .and_then(|c| usize::try_from(c).ok())
        .unwrap_or(80);
    let raw: Vec<String> = params
        .and_then(|p| p["lines"].as_array())
        .map(|lines| {
            lines
                .iter()
                .map(|line| {
                    line.as_array()
                        .map(|spans| {
                            spans
                                .iter()
                                .filter_map(|s| s["text"].as_str())
                                .collect::<String>()
                        })
                        .unwrap_or_default()
                })
                .collect()
        })
        .unwrap_or_default();
    let mut linhas: Vec<String> = Vec::new();
    let mut continua = false;
    for row in raw {
        let cheia = row.chars().count() >= cols && !row.ends_with(' ');
        if continua {
            if let Some(last) = linhas.last_mut() {
                last.push_str(row.trim_end());
            }
        } else {
            linhas.push(row.trim_end().to_owned());
        }
        continua = cheia;
    }
    while linhas.last().is_some_and(String::is_empty) {
        linhas.pop();
    }
    linhas
}

/// O que uma EXECUCAO (`run.start`/`run.script`, uma sessao de terminal desde
/// 2026-09-18) mostrou ate' fechar: as linhas do ULTIMO `event.terminal.render`
/// da sessao `terminal_id` e o `exitCode` do `event.terminal.closed`. Outros
/// eventos no canal sao ignorados; um render que chegue logo depois do
/// closed ainda conta.
fn terminal_run_until_closed(
    receiver: &std::sync::mpsc::Receiver<kinein_protocol::JsonRpcRequest>,
    terminal_id: &str,
    timeout: std::time::Duration,
) -> (Vec<String>, Option<i64>) {
    let deadline = std::time::Instant::now() + timeout;
    let mut linhas: Vec<String> = Vec::new();
    let da_sessao = |e: &kinein_protocol::JsonRpcRequest| {
        e.params.as_ref().and_then(|p| p["id"].as_str()) == Some(terminal_id)
    };
    while std::time::Instant::now() < deadline {
        let Ok(event) = receiver.recv_timeout(std::time::Duration::from_millis(100)) else {
            continue;
        };
        if !da_sessao(&event) {
            continue;
        }
        if event.method == "event.terminal.render" {
            linhas = render_lines(&event);
        }
        if event.method == "event.terminal.closed" {
            let code = event.params.as_ref().and_then(|p| p["exitCode"].as_i64());
            let mais = std::time::Instant::now() + std::time::Duration::from_millis(400);
            while std::time::Instant::now() < mais {
                if let Ok(e) = receiver.recv_timeout(std::time::Duration::from_millis(50))
                    && e.method == "event.terminal.render"
                    && da_sessao(&e)
                {
                    linhas = render_lines(&e);
                }
            }
            return (linhas, code);
        }
    }
    panic!("a sessao {terminal_id} nao fechou em {timeout:?}; visto: {linhas:?}");
}

fn core_with_empty_search_path(test_name: &str) -> Core {
    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-dispatch-{test_name}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    Core::with_detector(ToolDetector::with_search_path(dir))
}

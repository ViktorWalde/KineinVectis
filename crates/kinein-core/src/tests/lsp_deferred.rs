//! As consultas LSP ADIADAS (Etapa 2, F6, 2026-09-18): o laco do core nao
//! para esperando o servidor de linguagem.
//!
//! Medido na F3 com o rust-analyzer real: um `lsp.semanticTokens` segurava
//! o laco ate' 4 s (15 s no initialize) e o `fs.list` seguinte so' saia
//! depois — a IDE inteira muda. Aqui o servidor falso demora 3 s no hover;
//! com as respostas adiadas ligadas, o `lsp.hover` volta como Deferred, um
//! `fs.list` pedido logo depois responde em < 100 ms, e a resposta real do
//! hover chega pelo canal quando o servidor responde. Sem o canal (o modo
//! dos outros testes), a espera e' inline como sempre.

use std::{
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    sync::mpsc,
    time::{Duration, Instant},
};

use kinein_protocol::{JsonRpcRequest, JsonRpcResponse};
use serde_json::json;

use super::lsp_server::{fake_server, python3};

fn workspace(nome: &str) -> PathBuf {
    let root = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-lspdeferred-{nome}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(root.join("Cargo.toml"), "[package]\nname = \"demo\"\n").unwrap();
    std::fs::write(root.join("src/main.rs"), "fn main() {}\n").unwrap();
    root.canonicalize().unwrap()
}

/// Um envoltorio que liga o atraso do hover no servidor falso.
fn slow_server(root: &std::path::Path, delay_ms: u32) -> PathBuf {
    let script = root.join("lsp-lento.sh");
    std::fs::write(
        &script,
        format!(
            "#!/bin/sh\nFAKE_LSP_HOVER_DELAY_MS={delay_ms} exec {} {} \"$@\"\n",
            python3(),
            fake_server().display()
        ),
    )
    .unwrap();
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
    script
}

#[test]
#[cfg(unix)]
fn a_slow_hover_does_not_block_the_loop_and_answers_later_through_the_channel() {
    let _serial = super::EXECUTAVEIS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let root = workspace("lento");
    let log = root.join("wire.jsonl");
    let (events, _inbox) = mpsc::channel::<JsonRpcRequest>();
    let (responses, respostas) = mpsc::channel::<JsonRpcResponse>();
    let mut core = super::core_with_empty_search_path("lsp-deferred");
    core.enable_lsp(events);
    core.enable_deferred_responses(responses);
    let script = slow_server(&root, 3000);
    assert!(core.use_language_server_command(
        "rust",
        script.to_str().unwrap(),
        &[log.to_str().unwrap()]
    ));
    let opened = core.handle_request(&JsonRpcRequest::new(
        1_i64,
        "workspace.open",
        Some(json!({ "path": root.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none());
    let main_rs = root.join("src/main.rs").display().to_string();

    // O hover volta ADIADO na hora (o servidor sobe e recebe o pedido; a
    // espera e' de outra thread).
    let t0 = Instant::now();
    let hover = core.handle_request(&JsonRpcRequest::new(
        2_i64,
        "lsp.hover",
        Some(json!({ "path": &main_rs, "content": "fn main() {}\n", "line": 1, "column": 4 })),
    ));
    assert!(hover.is_deferred(), "{:?}", hover.response());
    assert!(
        t0.elapsed() < Duration::from_millis(2500),
        "o handler esperou: {:?}",
        t0.elapsed()
    );

    // O laco continua: um fs.list responde em bem menos que o atraso do hover.
    let t1 = Instant::now();
    let listed = core.handle_request(&JsonRpcRequest::new(
        3_i64,
        "fs.list",
        Some(json!({ "path": root.to_str().unwrap() })),
    ));
    assert!(listed.response().error.is_none());
    assert!(!listed.is_deferred());
    assert!(
        t1.elapsed() < Duration::from_millis(500),
        "fs.list demorou {:?}",
        t1.elapsed()
    );

    // E a resposta real do hover chega pelo canal, com o id do pedido.
    let resposta = respostas
        .recv_timeout(Duration::from_secs(10))
        .expect("a resposta adiada do hover nao chegou");
    assert_eq!(resposta.id, Some(json!(2)));
    assert_eq!(
        resposta.result.as_ref().unwrap()["content"],
        "fake hover",
        "{resposta:?}"
    );
    assert!(
        t0.elapsed() >= Duration::from_millis(2500),
        "chegou cedo demais: {:?}",
        t0.elapsed()
    );
}

/// Sem o canal, a espera e' inline — o comportamento dos demais testes.
#[test]
#[cfg(unix)]
fn without_the_channel_the_hover_is_answered_inline() {
    let _serial = super::EXECUTAVEIS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let root = workspace("inline");
    let log = root.join("wire.jsonl");
    let (events, _inbox) = mpsc::channel::<JsonRpcRequest>();
    let mut core = super::core_with_empty_search_path("lsp-inline");
    core.enable_lsp(events);
    let script = slow_server(&root, 300);
    assert!(core.use_language_server_command(
        "rust",
        script.to_str().unwrap(),
        &[log.to_str().unwrap()]
    ));
    let opened = core.handle_request(&JsonRpcRequest::new(
        1_i64,
        "workspace.open",
        Some(json!({ "path": root.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none());
    let main_rs = root.join("src/main.rs").display().to_string();
    let hover = core.handle_request(&JsonRpcRequest::new(
        2_i64,
        "lsp.hover",
        Some(json!({ "path": &main_rs, "content": "fn main() {}\n", "line": 1, "column": 4 })),
    ));
    assert!(!hover.is_deferred());
    assert_eq!(
        hover.response().result.as_ref().unwrap()["content"],
        "fake hover"
    );
}

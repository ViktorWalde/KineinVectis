//! As tres threads de uma sessao viva: a leitora (bytes do PTY para o
//! emulador, e as respostas do emulador de volta ao PTY), a emissora (um frame
//! por `FRAME`, so' quando ha' mudanca) e a que espera o processo sair.

// Os metodos sao do `session`, e nada daqui e' publico: `pub(super)` e' a
// visibilidade certa, e `pub` seria reprovado pelo `unreachable_pub` do
// workspace. O mesmo conflito com o clippy esta' resolvido assim no
// `platform/mod.rs`.
#![allow(clippy::redundant_pub_crate)]

use std::{
    io::{Read, Write},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};

use kinein_protocol::JsonRpcRequest;
use portable_pty::Child;
use serde_json::json;

use super::super::render::emit_render;
use super::super::state::TerminalState;
use super::TerminalManager;

/// Tamanho de cada leitura do PTY, em bytes.
const READ_CHUNK_BYTES: usize = 8192;
/// Cadência máxima de render (~30fps): coalesce rajadas de saída.
const FRAME: Duration = Duration::from_millis(33);

impl TerminalManager {
    /// Thread leitora: bytes crus do PTY → `parser.process` → marca sujo.
    pub(super) fn spawn_reader(
        &self,
        id: &str,
        mut reader: Box<dyn Read + Send>,
        state: &Arc<Mutex<TerminalState>>,
        dirty: &Arc<AtomicBool>,
        writer: &Arc<Mutex<Box<dyn Write + Send>>>,
    ) {
        let state = Arc::clone(state);
        let dirty = Arc::clone(dirty);
        let writer = Arc::clone(writer);
        let events = self.events.clone();
        let id = id.to_owned();
        thread::spawn(move || {
            let mut buffer = [0_u8; READ_CHUNK_BYTES];
            while let Ok(bytes_read) = reader.read(&mut buffer) {
                if bytes_read == 0 {
                    break;
                }
                let bytes = &buffer[..bytes_read];
                let replies = state.lock().map_or_else(
                    |_poisoned| Vec::new(),
                    |mut state| {
                        state.process(bytes);
                        state.take_replies()
                    },
                );
                if !replies.is_empty()
                    && let Ok(mut writer) = writer.lock()
                {
                    // Se o processo ja' saiu, nao ha' quem leia a resposta.
                    drop(writer.write_all(&replies).and_then(|()| writer.flush()));
                }
                dirty.store(true, Ordering::SeqCst);
            }
            // EOF: garante o render do estado final antes do `closed`.
            emit_render(&events, &id, &state);
        });
    }

    /// Thread emissora: a cada FRAME, se sujo, manda o grid pra UI (throttle).
    pub(super) fn spawn_emitter(
        &self,
        id: &str,
        state: &Arc<Mutex<TerminalState>>,
        dirty: &Arc<AtomicBool>,
        running: &Arc<AtomicBool>,
    ) {
        let state = Arc::clone(state);
        let dirty = Arc::clone(dirty);
        let running = Arc::clone(running);
        let events = self.events.clone();
        let id = id.to_owned();
        thread::spawn(move || {
            while running.load(Ordering::SeqCst) {
                thread::sleep(FRAME);
                if dirty.swap(false, Ordering::SeqCst) {
                    emit_render(&events, &id, &state);
                }
            }
        });
    }

    /// Thread waiter: espera o shell sair, zera `running` e emite `closed`.
    pub(super) fn spawn_waiter(
        &self,
        id: &str,
        mut child: Box<dyn Child + Send + Sync>,
        running: &Arc<AtomicBool>,
    ) {
        let running = Arc::clone(running);
        let events = self.events.clone();
        let id = id.to_owned();
        thread::spawn(move || {
            let code = child
                .wait()
                .ok()
                .and_then(|status| i32::try_from(status.exit_code()).ok());
            running.store(false, Ordering::SeqCst);
            drop(events.send(JsonRpcRequest::notification(
                "event.terminal.closed",
                Some(json!({ "id": id, "exitCode": code })),
            )));
        });
    }
}

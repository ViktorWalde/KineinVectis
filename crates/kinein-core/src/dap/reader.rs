//! A thread LEITORA do adapter e a traducao de evento DAP para `event.debug.*`.
//!
//! Separado em 2026-09-03 (etapa 15 do `roadmaps/34`). A regra que este arquivo
//! guarda: a leitora NUNCA espera resposta de request — e' ela quem as entrega.
//! Por isso o enriquecimento do `stopped` (stackTrace -> file/line) roda em
//! thread propria, e nao aqui dentro.

use std::{
    io::{BufReader, Read},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread,
};

use kinein_protocol::JsonRpcRequest;
use serde_json::{Value, json};

use super::session::REQUEST_TIMEOUT;
use super::wire::{Pending, Wire};
use crate::lsp::EventSender;
use crate::lsp::framing::read_message;

/// Sobe a thread leitora do stdout ou socket do adapter.
pub(super) fn spawn_reader(
    stdout: impl Read + Send + 'static,
    wire: &Wire,
    events: &EventSender,
    alive: &Arc<AtomicBool>,
    stopped_thread: &Arc<Mutex<Option<i64>>>,
    initialized: mpsc::Sender<()>,
) {
    let wire = wire.clone();
    let events = events.clone();
    let alive = Arc::clone(alive);
    let stopped_thread = Arc::clone(stopped_thread);
    thread::spawn(move || {
        let exit_code = Arc::new(Mutex::new(None));
        let mut reader = BufReader::new(stdout);
        while let Ok(Some(message)) = read_message(&mut reader) {
            match message.get("type").and_then(Value::as_str) {
                Some("response") => deliver_response(&wire.pending, &message),
                Some("event") => handle_adapter_event(
                    &message,
                    &wire,
                    &events,
                    &alive,
                    &stopped_thread,
                    &exit_code,
                    &initialized,
                ),
                _ => {}
            }
        }
        // EOF/erro: destrava quem espera resposta e fecha a sessao uma vez.
        if let Ok(mut guard) = wire.pending.lock() {
            guard.clear();
        }
        finish_once(&alive, &events, &exit_code);
    });
}

/// Entrega uma resposta ao request que a aguarda.
fn deliver_response(pending: &Pending, message: &Value) {
    let Some(seq) = message.get("request_seq").and_then(Value::as_i64) else {
        return;
    };
    let sender = match pending.lock() {
        Ok(mut guard) => guard.remove(&seq),
        Err(_poisoned) => None,
    };
    if let Some(sender) = sender {
        drop(sender.send(message.clone()));
    }
}

/// Traduz um evento do adapter em `event.debug.*` da UI.
fn handle_adapter_event(
    message: &Value,
    wire: &Wire,
    events: &EventSender,
    alive: &Arc<AtomicBool>,
    stopped_thread: &Arc<Mutex<Option<i64>>>,
    exit_code: &Arc<Mutex<Option<i64>>>,
    initialized: &mpsc::Sender<()>,
) {
    let name = message
        .get("event")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let body = message.get("body").cloned().unwrap_or(Value::Null);
    match name {
        "initialized" => drop(initialized.send(())),
        "output" => emit_output(events, &body),
        "stopped" => {
            let thread_id = body.get("threadId").and_then(Value::as_i64).unwrap_or(0);
            let reason = body
                .get("reason")
                .and_then(Value::as_str)
                .unwrap_or("stopped")
                .to_owned();
            if let Ok(mut guard) = stopped_thread.lock() {
                *guard = Some(thread_id);
            }
            let wire = wire.clone();
            let events = events.clone();
            // A leitora nao pode esperar o stackTrace que ela mesma entrega.
            thread::spawn(move || emit_enriched_stopped(&wire, &events, thread_id, &reason));
        }
        "continued" => note_continued(stopped_thread, events),
        "exited" => {
            if let Ok(mut guard) = exit_code.lock() {
                *guard = body.get("exitCode").and_then(Value::as_i64);
            }
        }
        "terminated" => finish_once(alive, events, exit_code),
        _ => {}
    }
}

/// Emite `event.debug.output` linha a linha (telemetry e ignorada).
fn emit_output(events: &EventSender, body: &Value) {
    let category = body
        .get("category")
        .and_then(Value::as_str)
        .unwrap_or("console");
    if category == "telemetry" {
        return;
    }
    let Some(output) = body.get("output").and_then(Value::as_str) else {
        return;
    };
    for line in output.lines() {
        send_event(
            events,
            "event.debug.output",
            json!({ "category": category, "line": line }),
        );
    }
}

/// Consulta o frame do topo e emite `event.debug.stopped` com file/line.
fn emit_enriched_stopped(wire: &Wire, events: &EventSender, thread_id: i64, reason: &str) {
    let mut file = Value::Null;
    let mut line = Value::Null;
    if let Ok(body) = wire.request(
        "stackTrace",
        &json!({ "threadId": thread_id, "startFrame": 0, "levels": 1 }),
        REQUEST_TIMEOUT,
    ) {
        if let Some(frame) = body
            .get("stackFrames")
            .and_then(Value::as_array)
            .and_then(|frames| frames.first())
        {
            file = frame
                .get("source")
                .and_then(|source| source.get("path"))
                .cloned()
                .unwrap_or(Value::Null);
            line = frame.get("line").cloned().unwrap_or(Value::Null);
        }
    }
    send_event(
        events,
        "event.debug.stopped",
        json!({
            "reason": reason,
            "file": file,
            "line": line,
            "threadId": thread_id,
        }),
    );
}

/// Emite `event.debug.continued` uma unica vez por retomada.
///
/// Chamada tanto apos um continue/step bem-sucedido quanto no evento
/// `continued` do adapter: o `take()` do estado pausado deduplica.
pub(super) fn note_continued(stopped_thread: &Arc<Mutex<Option<i64>>>, events: &EventSender) {
    let was_stopped = stopped_thread
        .lock()
        .is_ok_and(|mut guard| guard.take().is_some());
    if was_stopped {
        send_event(events, "event.debug.continued", json!({}));
    }
}

/// Emite `event.debug.finished` exatamente uma vez por sessao.
fn finish_once(alive: &Arc<AtomicBool>, events: &EventSender, exit_code: &Arc<Mutex<Option<i64>>>) {
    if alive.swap(false, Ordering::SeqCst) {
        let code = exit_code.lock().ok().and_then(|guard| *guard);
        send_event(events, "event.debug.finished", json!({ "exitCode": code }));
    }
}

/// Serializa uma notificacao `event.debug.*` no canal assincrono.
pub(super) fn send_event(events: &EventSender, method: &str, params: Value) {
    drop(events.send(JsonRpcRequest::notification(method, Some(params))));
}

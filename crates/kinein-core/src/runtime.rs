//! The stdio JSON-RPC loop that drives `Core` as a subprocess.
//!
//! `run_stdio` wires up the async event channel and multiplexes stdin requests
//! with LSP/run/terminal notifications; `run_json_lines` is the pure,
//! testable line-by-line driver used by the tests.

mod services;

use std::io::{self, BufRead, Write};

use kinein_protocol::{JsonRpcRequest, JsonRpcResponse};

use crate::{Core, CoreError};

/// Runs a line-delimited JSON-RPC loop over arbitrary IO streams.
///
/// This function is used by `kinein-core` over stdin/stdout and by tests over
/// in-memory buffers.
pub fn run_json_lines<R, W>(reader: R, mut writer: W) -> Result<(), CoreError>
where
    R: BufRead,
    W: Write,
{
    let mut core = Core::new();

    for line in reader.lines() {
        let line = line.map_err(CoreError::Read)?;

        if line.trim().is_empty() {
            continue;
        }

        let outcome = core.handle_json_line(&line);
        write_json_line(&mut writer, outcome.response())?;

        if outcome.should_shutdown() {
            break;
        }
    }

    Ok(())
}

fn write_json_line<W, T>(writer: &mut W, payload: &T) -> Result<(), CoreError>
where
    W: Write,
    T: serde::Serialize,
{
    serde_json::to_writer(&mut *writer, payload).map_err(CoreError::Serialize)?;
    writer.write_all(b"\n").map_err(CoreError::Write)?;
    writer.flush().map_err(CoreError::Write)
}

/// Evento interno do loop stdio.
///
/// `pub(crate)` para que o [`drain_loop_events`] seja testavel: ate 2026-09-02
/// o corpo do loop so existia dentro do `run_stdio`, entao a fiacao "o core
/// observa o evento ANTES de a UI ve-lo" nao tinha como reprovar. Ligar no
/// lugar errado nao quebra build — deixa de funcionar em silencio
/// (`ARCHITECTURE.md` §8).
pub(crate) enum LoopEvent {
    /// One request line arrived on stdin.
    Line(String),
    /// An async notification (LSP diagnostics, server status) must be sent.
    Notification(Box<JsonRpcRequest>),
    /// Uma resposta ADIADA chegou (consulta LSP esperada fora do laco).
    Response(Box<JsonRpcResponse>),
    /// A CONTINUACAO de um pedido adiado que precisa do `Core` para fechar
    /// (rename, code actions: a espera foi fora do laco; o plano e a
    /// transacao nascem aqui, com `&mut Core`).
    Continue(crate::Continuation),
    /// Stdin closed; the core should stop.
    Eof,
}

/// Runs the core over stdin/stdout with support for async notifications.
///
/// A reader thread feeds stdin lines into a channel; LSP reader threads feed
/// notifications into the same channel. The main loop serializes everything
/// to stdout, so responses and events never interleave mid-line.
pub fn run_stdio() -> Result<(), CoreError> {
    let (events, inbox) = std::sync::mpsc::channel::<LoopEvent>();

    let line_events = events.clone();
    std::thread::spawn(move || {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            match line {
                Ok(line) => {
                    if line_events.send(LoopEvent::Line(line)).is_err() {
                        return;
                    }
                }
                Err(_error) => break,
            }
        }
        drop(line_events.send(LoopEvent::Eof));
    });

    let (lsp_events, lsp_inbox) = std::sync::mpsc::channel::<JsonRpcRequest>();
    let notification_events = events.clone();
    std::thread::spawn(move || {
        for notification in lsp_inbox {
            if notification_events
                .send(LoopEvent::Notification(Box::new(notification)))
                .is_err()
            {
                return;
            }
        }
    });

    // As respostas adiadas (Etapa 2 F6) entram no MESMO laco, pela mesma
    // ponte: serializadas no stdout junto com as outras linhas.
    let (response_events, response_inbox) = std::sync::mpsc::channel::<JsonRpcResponse>();
    let response_loop_for_continuations = events.clone();
    let response_loop = events;
    std::thread::spawn(move || {
        for response in response_inbox {
            if response_loop
                .send(LoopEvent::Response(Box::new(response)))
                .is_err()
            {
                return;
            }
        }
    });

    // As continuacoes (rename/code actions adiados) voltam ao laco pela
    // mesma ponte: rodam com `&mut Core` e a resposta sai em ordem.
    let (continuation_events, continuation_inbox) =
        std::sync::mpsc::channel::<crate::Continuation>();
    let continuation_loop = response_loop_for_continuations;
    std::thread::spawn(move || {
        for continuation in continuation_inbox {
            if continuation_loop
                .send(LoopEvent::Continue(continuation))
                .is_err()
            {
                return;
            }
        }
    });

    let mut core = Core::new();
    core.enable_lsp(lsp_events);
    core.enable_deferred_responses(response_events);
    core.enable_continuations(continuation_events);
    // Persistencia local (rascunhos + historico global) so no processo real: o
    // caminho do estado global entra por aqui e nunca e' deduzido la dentro.
    core.enable_persistence(crate::settings::global_dir());

    let stdout = io::stdout();
    let mut writer = stdout.lock();

    drain_loop_events(&mut core, &mut writer, &inbox)
}

/// Drena os eventos do loop aplicando a politica do processo real.
///
/// Separado do [`run_stdio`] para ser testavel sem stdio: o que se prova aqui e
/// que **toda notificacao passa por [`Core::observe_notification`] antes de ir
/// para a UI**. E o ponto em que a fronteira de thread do `arquitetura/04` §3
/// se fecha sem estado compartilhado — o job emite o evento, e quem possui o
/// `Core` e este loop.
pub(crate) fn drain_loop_events<W>(
    core: &mut Core,
    writer: &mut W,
    inbox: &std::sync::mpsc::Receiver<LoopEvent>,
) -> Result<(), CoreError>
where
    W: Write,
{
    for event in inbox {
        match event {
            LoopEvent::Line(line) => {
                if line.trim().is_empty() {
                    continue;
                }

                let outcome = core.handle_json_line(&line);
                if !outcome.is_deferred() {
                    write_json_line(writer, outcome.response())?;
                }

                if outcome.should_shutdown() {
                    break;
                }
            }
            LoopEvent::Notification(notification) => {
                core.observe_notification(notification.as_ref());
                write_json_line(writer, notification.as_ref())?;
            }
            LoopEvent::Response(response) => {
                write_json_line(writer, response.as_ref())?;
            }
            LoopEvent::Continue(finish) => {
                let response = finish(core);
                write_json_line(writer, &response)?;
            }
            LoopEvent::Eof => break,
        }
    }

    Ok(())
}

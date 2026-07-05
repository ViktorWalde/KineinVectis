//! The stdio JSON-RPC loop that drives `Core` as a subprocess.
//!
//! `run_stdio` wires up the async event channel and multiplexes stdin requests
//! with LSP/run/terminal notifications; `run_json_lines` is the pure,
//! testable line-by-line driver used by the tests.

use std::io::{self, BufRead, Write};

use kinein_protocol::JsonRpcRequest;

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

        let mut emit_error: Option<CoreError> = None;
        let outcome = {
            let mut emit = |notification: &JsonRpcRequest| {
                if emit_error.is_some() {
                    return;
                }
                emit_error = write_json_line(&mut writer, notification).err();
            };
            core.handle_json_line_streaming(&line, &mut emit)
        };
        if let Some(error) = emit_error {
            return Err(error);
        }

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

/// Internal event of the stdio loop.
enum LoopEvent {
    /// One request line arrived on stdin.
    Line(String),
    /// An async notification (LSP diagnostics, server status) must be sent.
    Notification(Box<JsonRpcRequest>),
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
    let notification_events = events;
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

    let mut core = Core::new();
    core.enable_lsp(lsp_events);

    let stdout = io::stdout();
    let mut writer = stdout.lock();

    for event in inbox {
        match event {
            LoopEvent::Line(line) => {
                if line.trim().is_empty() {
                    continue;
                }

                let mut emit_error: Option<CoreError> = None;
                let outcome = {
                    let mut emit = |notification: &JsonRpcRequest| {
                        if emit_error.is_some() {
                            return;
                        }
                        emit_error = write_json_line(&mut writer, notification).err();
                    };
                    core.handle_json_line_streaming(&line, &mut emit)
                };
                if let Some(error) = emit_error {
                    return Err(error);
                }

                write_json_line(&mut writer, outcome.response())?;

                if outcome.should_shutdown() {
                    break;
                }
            }
            LoopEvent::Notification(notification) => {
                write_json_line(&mut writer, notification.as_ref())?;
            }
            LoopEvent::Eof => break,
        }
    }

    Ok(())
}

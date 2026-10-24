//! The IDE agent of PIO Home (52 §14.46, `03-dominios/platformio.md`).
//!
//! PIO Home asks the IDE to open what it creates. `agent.py`, run in
//! `PlatformIO`'s interpreter, keeps one `ide.listen_commands` pending on the
//! session's WebSocket and hands each command to this reader, which answers
//! the page and, for the two commands that are actions, tells the UI through
//! `event.platformio.homeCommand`. The agent exits when its stdin closes.

use std::{
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::{ChildStdin, Command, ExitStatus},
    sync::{
        Arc, Mutex, PoisonError,
        atomic::AtomicBool,
        mpsc::{self, RecvTimeoutError},
    },
    time::{Duration, Instant},
};

use kinein_protocol::JsonRpcRequest;
use serde_json::{Value, json};

use crate::owned_child::{Env, OwnedChild};

/// The agent `PlatformIO`'s interpreter runs.
const AGENT: &str = include_str!("agent.py");
const INFO_DEADLINE: Duration = Duration::from_secs(30);

/// What the agent's commands need from the core: where events go and the
/// workspace that is open now.
#[derive(Debug, Clone, Default)]
pub struct Context {
    /// The core's event channel (`None` in tests that do not listen).
    pub events: Option<crate::lsp::EventSender>,
    /// The open workspace root, kept current by the core.
    pub workspace: Arc<Mutex<Option<PathBuf>>>,
}

/// The running agent: its process and the pipe its answers go through.
///
/// The pipe lives here, not in the `OwnedChild`: the reader thread writes
/// the answers to it, and closing it is what tells the agent to leave.
#[derive(Debug)]
pub(super) struct Agent {
    child: OwnedChild,
    replies: Arc<Mutex<Option<ChildStdin>>>,
}

impl Agent {
    pub(super) fn exited(&mut self) -> Option<ExitStatus> {
        self.child.exited()
    }

    /// Close its stdin, so it closes the WebSocket and leaves, then reap it
    /// (the group is ended if `grace` passes first).
    pub(super) fn close(self, grace: Duration) {
        drop(
            self.replies
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .take(),
        );
        let _ = self.child.close(grace);
    }
}

/// `PlatformIO`'s interpreter, from `pio system info --json-output`.
///
/// # Errors
/// The query fails, times out or names no interpreter.
pub(super) fn interpreter(pio: &Path) -> Result<PathBuf, String> {
    let mut command = Command::new(pio);
    command
        .args(["system", "info", "--json-output"])
        .current_dir(
            pio.parent()
                .map_or_else(std::env::temp_dir, Path::to_path_buf),
        )
        .env("PLATFORMIO_NO_ANSI", "1");
    let mut output = String::new();
    let result = crate::process::stream_command_lines_with_timeout_cancelable(
        command,
        INFO_DEADLINE,
        Arc::new(AtomicBool::new(false)),
        &mut |_, line| {
            output.push_str(&line);
            output.push('\n');
        },
    );
    match result {
        Ok((status, false)) if status.success() => output
            .lines()
            .rev()
            .find_map(|line| serde_json::from_str::<Value>(line.trim()).ok())
            .and_then(|info| info["python_exe"]["value"].as_str().map(PathBuf::from))
            .filter(|path| !path.as_os_str().is_empty())
            .ok_or_else(|| {
                format!(
                    "pio system info não informou o interpretador: {}",
                    output.trim()
                )
            }),
        Ok((_, true)) => Err("o prazo de pio system info expirou".to_owned()),
        Ok((status, false)) => Err(format!("pio system info terminou com {status}")),
        Err(error) => Err(format!("falha em pio system info: {error:?}")),
    }
}

/// Spawn the agent on `socket` (`ws://127.0.0.1:<port>/session/<id>/wsrpc`)
/// and wait for it to connect.
///
/// # Errors
/// It cannot start, says it failed, ends, or `deadline` passes first.
pub(super) fn spawn(
    python: &Path,
    socket: &str,
    context: Context,
    deadline: Instant,
) -> Result<Agent, String> {
    let mut command = Command::new(python);
    command.args(["-I", "-c", AGENT, socket]);
    let writer: Arc<Mutex<Option<ChildStdin>>> = Arc::default();
    let replies = Arc::clone(&writer);
    let (ready, connected) = mpsc::channel();
    let mut owned = OwnedChild::spawn(command, Env::Inherit, None, move |stdout| {
        for line in BufReader::new(stdout).lines().map_while(Result::ok) {
            let Ok(message) = serde_json::from_str::<Value>(&line) else {
                continue;
            };
            if message["ready"] == true {
                let _ = ready.send(Ok(()));
            } else if let Some(error) = message["error"].as_str() {
                let _ = ready.send(Err(error.to_owned()));
            } else if message["command"].is_object() {
                let value = handle(&message["command"], &context);
                let reply = json!({"result": {"id": message["command"]["id"], "value": value}});
                if let Some(stdin) = replies
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .as_mut()
                {
                    let _ = writeln!(stdin, "{reply}").and_then(|()| stdin.flush());
                }
            }
        }
    })
    .map_err(|error| format!("não foi possível iniciar o agente do PIO Home: {error}"))?;
    *writer.lock().unwrap_or_else(PoisonError::into_inner) = owned.take_stdin();
    loop {
        let wait = deadline
            .saturating_duration_since(Instant::now())
            .min(Duration::from_millis(200));
        match connected.recv_timeout(wait) {
            Ok(Ok(())) => {
                return Ok(Agent {
                    child: owned,
                    replies: writer,
                });
            }
            Ok(Err(error)) => {
                let _ = owned.close(Duration::ZERO);
                return Err(format!("o agente do PIO Home não conectou: {error}"));
            }
            Err(RecvTimeoutError::Timeout) if Instant::now() < deadline => {
                if let Some(status) = owned.exited() {
                    let tail = owned.stderr_tail();
                    return Err(format!(
                        "o agente do PIO Home terminou ({status}): {}",
                        tail.trim()
                    ));
                }
            }
            Err(_) => {
                let _ = owned.close(Duration::ZERO);
                return Err("o agente do PIO Home não conectou a tempo".to_owned());
            }
        }
    }
}

/// The page's answer, and the UI event for the commands that are actions.
fn handle(command: &Value, context: &Context) -> Value {
    let params = &command["params"];
    let (method, path) = match command["method"].as_str() {
        Some("get_pio_project_dirs") => {
            let root = context
                .workspace
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .clone();
            return json!(
                root.filter(|root| root.join("platformio.ini").is_file())
                    .map(|root| vec![root])
                    .unwrap_or_default()
            );
        }
        Some("open_project") => (
            "open_project",
            params.as_str().or_else(|| params["path"].as_str()),
        ),
        Some("open_text_document") => ("open_text_document", params["path"].as_str()),
        _ => return Value::Null,
    };
    let Some(path) = path.filter(|path| Path::new(path).is_absolute()) else {
        return json!(false);
    };
    let event = if method == "open_project" {
        json!({"method": method, "params": {"path": path}})
    } else {
        json!({"method": method, "params": {
            "path": path,
            "line": params["line"].as_u64().unwrap_or(1),
            "column": params["column"].as_u64().unwrap_or(1),
        }})
    };
    if let Some(events) = context.events.as_ref() {
        drop(events.send(JsonRpcRequest::notification(
            "event.platformio.homeCommand",
            Some(event),
        )));
    }
    json!(true)
}

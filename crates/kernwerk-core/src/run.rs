//! Non-blocking execution of user commands inside the workspace.
//!
//! `run.start` spawns one child process per workspace via `sh -c`, streams
//! `event.run.output` lines through the same async notification channel used
//! by the LSP manager, and reports `event.run.finished` when the process
//! exits. The core stays responsive during the whole run; `run.stdin` and
//! `run.stop` talk to the live process.
//!
//! This is NOT a full terminal: there is no TTY, so full-screen interactive
//! programs will not behave. Line-based programs (test runners, servers,
//! simple prompts) work.

use std::{
    error::Error,
    fmt,
    io::{BufRead, BufReader, Write},
    path::Path,
    process::{Child, ChildStdin, Command, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

use kernwerk_protocol::{JsonRpcRequest, ProjectKind};
use serde_json::json;

use crate::lsp::EventSender;

/// Interval between `try_wait` polls while the process is alive.
const WAIT_POLL_INTERVAL: Duration = Duration::from_millis(50);

/// After the process exits, how long to wait for the output readers to
/// drain before emitting `finished`. Grandchildren keeping the pipes open
/// (e.g. a killed shell whose child survives) must not delay the event.
const READER_DRAIN_DEADLINE: Duration = Duration::from_secs(2);

/// Error produced by the run manager.
#[derive(Debug)]
pub enum RunError {
    /// A process is already running in this workspace.
    AlreadyRunning,
    /// No process is currently running.
    NotRunning,
    /// The project kind has no default run command.
    NoDefaultCommand {
        /// Human explanation of what to do instead.
        message: String,
    },
    /// The child process could not be spawned or reached.
    Process {
        /// Underlying failure description.
        message: String,
    },
}

impl fmt::Display for RunError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyRunning => {
                write!(
                    formatter,
                    "ja existe um processo em execucao; pare-o antes (run.stop)"
                )
            }
            Self::NotRunning => write!(formatter, "nenhum processo em execucao"),
            Self::NoDefaultCommand { message } | Self::Process { message } => {
                write!(formatter, "{message}")
            }
        }
    }
}

impl Error for RunError {}

/// Owns the single child process a workspace may run at a time.
#[derive(Debug)]
pub struct RunManager {
    events: EventSender,
    child: Arc<Mutex<Option<Child>>>,
    stdin: Option<ChildStdin>,
}

impl RunManager {
    /// Creates a manager that pushes `event.run.*` through `events`.
    #[must_use]
    pub fn new(events: EventSender) -> Self {
        Self {
            events,
            child: Arc::new(Mutex::new(None)),
            stdin: None,
        }
    }

    /// Returns `true` while a child process is alive.
    #[must_use]
    pub fn is_running(&self) -> bool {
        self.child.lock().is_ok_and(|guard| guard.is_some())
    }

    /// Spawns `command` via `sh -c` in `root` and streams its output.
    pub fn start(&mut self, root: &Path, command: &str) -> Result<(), RunError> {
        if self.is_running() {
            return Err(RunError::AlreadyRunning);
        }

        let mut child = Command::new("sh")
            .arg("-c")
            .arg(command)
            .current_dir(root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|source| RunError::Process {
                message: format!("falha ao iniciar `{command}`: {source}"),
            })?;

        self.stdin = child.stdin.take();
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        send_event(
            &self.events,
            "event.run.started",
            json!({ "command": command }),
        );

        let pending_readers = Arc::new(AtomicUsize::new(0));
        if let Some(stdout) = stdout {
            let events = self.events.clone();
            let pending = Arc::clone(&pending_readers);
            pending.fetch_add(1, Ordering::SeqCst);
            thread::spawn(move || {
                for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                    send_event(
                        &events,
                        "event.run.output",
                        json!({ "stream": "stdout", "line": line }),
                    );
                }
                pending.fetch_sub(1, Ordering::SeqCst);
            });
        }
        if let Some(stderr) = stderr {
            let events = self.events.clone();
            let pending = Arc::clone(&pending_readers);
            pending.fetch_add(1, Ordering::SeqCst);
            thread::spawn(move || {
                for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                    send_event(
                        &events,
                        "event.run.output",
                        json!({ "stream": "stderr", "line": line }),
                    );
                }
                pending.fetch_sub(1, Ordering::SeqCst);
            });
        }

        if let Ok(mut guard) = self.child.lock() {
            *guard = Some(child);
        }

        let slot = Arc::clone(&self.child);
        let events = self.events.clone();
        thread::spawn(move || {
            let status = wait_for_exit(&slot);
            drain_readers(&pending_readers);
            if let Ok(mut guard) = slot.lock() {
                *guard = None;
            }
            send_event(
                &events,
                "event.run.finished",
                json!({
                    "success": status.as_ref().is_some_and(std::process::ExitStatus::success),
                    "exitCode": status.and_then(|status| status.code()),
                }),
            );
        });

        Ok(())
    }

    /// Forwards raw `data` to the child stdin.
    pub fn write_stdin(&mut self, data: &str) -> Result<(), RunError> {
        if !self.is_running() {
            return Err(RunError::NotRunning);
        }
        let Some(stdin) = self.stdin.as_mut() else {
            return Err(RunError::Process {
                message: "stdin do processo nao esta disponivel".to_owned(),
            });
        };
        stdin
            .write_all(data.as_bytes())
            .and_then(|()| stdin.flush())
            .map_err(|source| RunError::Process {
                message: format!("falha ao escrever no stdin: {source}"),
            })
    }

    /// Kills the running child. The waiter thread reports `event.run.finished`.
    pub fn stop(&mut self) -> Result<(), RunError> {
        let Ok(mut guard) = self.child.lock() else {
            return Err(RunError::NotRunning);
        };
        let Some(child) = guard.as_mut() else {
            return Err(RunError::NotRunning);
        };
        child.kill().map_err(|source| RunError::Process {
            message: format!("falha ao encerrar o processo: {source}"),
        })
    }
}

/// Serializes one `event.run.*` notification into the async channel.
fn send_event(events: &EventSender, method: &str, params: serde_json::Value) {
    drop(events.send(JsonRpcRequest::notification(method, Some(params))));
}

/// Polls the shared child slot until the process exits.
///
/// Returns `None` only when the slot or the wait syscall is unusable; the
/// caller then reports a failed run instead of hanging.
pub(crate) fn wait_for_exit(slot: &Arc<Mutex<Option<Child>>>) -> Option<std::process::ExitStatus> {
    loop {
        let poll = match slot.lock() {
            Ok(mut guard) => guard
                .as_mut()
                .map_or(Err(()), |child| child.try_wait().map_err(|_error| ())),
            Err(_poisoned) => Err(()),
        };
        match poll {
            Ok(Some(status)) => return Some(status),
            Ok(None) => thread::sleep(WAIT_POLL_INTERVAL),
            Err(()) => return None,
        }
    }
}

/// Waits until every output reader finished or the drain deadline passes.
pub(crate) fn drain_readers(pending: &Arc<AtomicUsize>) {
    let deadline = Instant::now() + READER_DRAIN_DEADLINE;
    while pending.load(Ordering::SeqCst) > 0 && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(20));
    }
}

/// Derives the default run command for the magic Run button.
pub fn default_command(kind: ProjectKind, root: &Path) -> Result<String, RunError> {
    match kind {
        ProjectKind::RustCargo => Ok("cargo run".to_owned()),
        ProjectKind::Cmake => cmake_binary_command(root),
        ProjectKind::Maven | ProjectKind::Gradle | ProjectKind::Python | ProjectKind::Unknown => {
            Err(RunError::NoDefaultCommand {
                message: "este tipo de projeto ainda nao tem comando de execucao padrao; \
                          digite o comando no painel Terminal"
                    .to_owned(),
            })
        }
    }
}

/// Finds the single executable produced by the `CMake` build, if any.
fn cmake_binary_command(root: &Path) -> Result<String, RunError> {
    let build_dir = root.join(".kernwerk").join("build");
    let mut executables = Vec::new();
    collect_executables(&build_dir, &mut executables);

    match executables.as_slice() {
        [] => Err(RunError::NoDefaultCommand {
            message: "nenhum executavel encontrado em .kernwerk/build; \
                      compile antes (Ctrl+F9)"
                .to_owned(),
        }),
        [single] => Ok(format!("'{}'", single.replace('\'', "'\\''"))),
        _multiple => Err(RunError::NoDefaultCommand {
            message: format!(
                "mais de um executavel em .kernwerk/build ({}); \
                 digite o comando no painel Terminal",
                executables.join(", ")
            ),
        }),
    }
}

/// Collects executable regular files under `dir`, skipping `CMakeFiles`.
fn collect_executables(dir: &Path, found: &mut Vec<String>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        let name = entry.file_name().to_string_lossy().into_owned();
        if file_type.is_dir() {
            if name != "CMakeFiles" {
                collect_executables(&entry.path(), found);
            }
        } else if file_type.is_file() && is_executable(&entry.path()) {
            found.push(entry.path().display().to_string());
        }
    }
}

/// Returns `true` when the file has any execute permission bit set.
#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(path).is_ok_and(|metadata| metadata.permissions().mode() & 0o111 != 0)
}

/// Non-Unix platforms have no execute bit; nothing is auto-runnable.
#[cfg(not(unix))]
fn is_executable(_path: &Path) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, sync::mpsc, time::Duration};

    use kernwerk_protocol::{JsonRpcRequest, ProjectKind};

    use super::{RunError, RunManager, default_command};

    fn temp_root(test_name: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("kernwerk-run-tests")
            .join(format!("{}-{test_name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir.canonicalize().unwrap()
    }

    fn drain_until_finished(receiver: &mpsc::Receiver<JsonRpcRequest>) -> Vec<JsonRpcRequest> {
        let mut events = Vec::new();
        loop {
            let event = receiver
                .recv_timeout(Duration::from_secs(5))
                .expect("evento run dentro do timeout");
            let done = event.method == "event.run.finished";
            events.push(event);
            if done {
                return events;
            }
        }
    }

    #[test]
    fn start_streams_output_and_finishes_with_success() {
        let (sender, receiver) = mpsc::channel();
        let mut manager = RunManager::new(sender);
        let root = temp_root("echo");

        manager.start(&root, "printf 'ola\\n'").unwrap();
        let events = drain_until_finished(&receiver);

        assert_eq!(events[0].method, "event.run.started");
        let output = events
            .iter()
            .find(|event| event.method == "event.run.output")
            .expect("evento de output");
        let params = output.params.as_ref().unwrap();
        assert_eq!(params["line"], "ola");
        let finished = events.last().unwrap().params.as_ref().unwrap();
        assert_eq!(finished["success"], true);
        assert_eq!(finished["exitCode"], 0);
        assert!(!manager.is_running());
    }

    #[test]
    fn stdin_reaches_the_child_process() {
        let (sender, receiver) = mpsc::channel();
        let mut manager = RunManager::new(sender);
        let root = temp_root("stdin");

        manager
            .start(&root, "read nome && printf 'oi %s\\n' \"$nome\"")
            .unwrap();
        manager.write_stdin("kernwerk\n").unwrap();
        let events = drain_until_finished(&receiver);

        let output = events
            .iter()
            .find(|event| event.method == "event.run.output")
            .expect("evento de output");
        assert_eq!(output.params.as_ref().unwrap()["line"], "oi kernwerk");
    }

    #[test]
    fn second_start_is_rejected_and_stop_kills_the_child() {
        let (sender, receiver) = mpsc::channel();
        let mut manager = RunManager::new(sender);
        let root = temp_root("stop");

        manager.start(&root, "sleep 30").unwrap();
        assert!(matches!(
            manager.start(&root, "true"),
            Err(RunError::AlreadyRunning)
        ));

        manager.stop().unwrap();
        let events = drain_until_finished(&receiver);
        let finished = events.last().unwrap().params.as_ref().unwrap();
        assert_eq!(finished["success"], false);
        assert!(!manager.is_running());
        assert!(matches!(manager.stop(), Err(RunError::NotRunning)));
        assert!(matches!(
            manager.write_stdin("x\n"),
            Err(RunError::NotRunning)
        ));
    }

    #[test]
    fn default_command_covers_rust_and_rejects_kinds_without_default() {
        let root = temp_root("default");

        assert_eq!(
            default_command(ProjectKind::RustCargo, &root).unwrap(),
            "cargo run"
        );
        assert!(matches!(
            default_command(ProjectKind::Unknown, &root),
            Err(RunError::NoDefaultCommand { .. })
        ));
        assert!(matches!(
            default_command(ProjectKind::Cmake, &root),
            Err(RunError::NoDefaultCommand { .. })
        ));
    }

    #[test]
    #[cfg(unix)]
    fn default_command_finds_single_cmake_executable() {
        use std::os::unix::fs::PermissionsExt;

        let root = temp_root("cmake-bin");
        let build = root.join(".kernwerk").join("build");
        std::fs::create_dir_all(build.join("CMakeFiles")).unwrap();
        std::fs::write(build.join("CMakeFiles/ignorado"), "#!/bin/sh\n").unwrap();
        let mut ignored_permissions = std::fs::metadata(build.join("CMakeFiles/ignorado"))
            .unwrap()
            .permissions();
        ignored_permissions.set_mode(0o755);
        std::fs::set_permissions(build.join("CMakeFiles/ignorado"), ignored_permissions).unwrap();

        let binary = build.join("app");
        std::fs::write(&binary, "#!/bin/sh\n").unwrap();
        let mut permissions = std::fs::metadata(&binary).unwrap().permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(&binary, permissions).unwrap();

        let command = default_command(ProjectKind::Cmake, &root).unwrap();
        assert!(command.contains("app"));
        assert!(command.starts_with('\''));
    }
}

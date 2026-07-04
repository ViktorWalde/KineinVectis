//! Persistent shell session for the IDE Terminal panel.
//!
//! The core does not implement a terminal emulator. It orchestrates the
//! mature `script(1)` tool (util-linux) to allocate a real PTY and run the
//! user's own `$SHELL` interactively — profile, aliases and prompt included.
//! Output is streamed in chunks as `event.terminal.data`, with ANSI escape
//! sequences stripped by a small stateful sanitizer so the QML panel can
//! render plain text.
//!
//! Known limitation (registered): the panel renders sanitized text, so
//! full-screen TUI programs (vim, htop) will not draw correctly even though
//! they see a real TTY.

use std::{
    error::Error,
    fmt,
    io::{Read, Write},
    path::Path,
    process::{Child, ChildStdin, Command, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    thread,
};

use kernwerk_protocol::JsonRpcRequest;
use serde_json::json;

use crate::{
    lsp::EventSender,
    run::{drain_readers, wait_for_exit},
};

/// Size of each PTY read, in bytes.
const READ_CHUNK_BYTES: usize = 4096;

/// Error produced by the terminal session manager.
#[derive(Debug)]
pub enum TerminalError {
    /// A session is already open.
    AlreadyOpen,
    /// No session is currently open.
    NotOpen,
    /// The session process could not be spawned or reached.
    Process {
        /// Underlying failure description.
        message: String,
    },
}

impl fmt::Display for TerminalError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyOpen => write!(formatter, "o terminal ja esta aberto"),
            Self::NotOpen => write!(formatter, "nenhuma sessao de terminal aberta"),
            Self::Process { message } => write!(formatter, "{message}"),
        }
    }
}

impl Error for TerminalError {}

/// Owns the single shell session a workspace may keep open at a time.
#[derive(Debug)]
pub struct TerminalManager {
    events: EventSender,
    child: Arc<Mutex<Option<Child>>>,
    stdin: Option<ChildStdin>,
}

impl TerminalManager {
    /// Creates a manager that pushes `event.terminal.*` through `events`.
    #[must_use]
    pub fn new(events: EventSender) -> Self {
        Self {
            events,
            child: Arc::new(Mutex::new(None)),
            stdin: None,
        }
    }

    /// Returns `true` while the shell session is alive.
    #[must_use]
    pub fn is_open(&self) -> bool {
        self.child.lock().is_ok_and(|guard| guard.is_some())
    }

    /// Opens the user's shell (from `$SHELL`) inside a PTY at `root`.
    ///
    /// Returns the shell path on success.
    pub fn open(&mut self, root: &Path) -> Result<String, TerminalError> {
        let shell = std::env::var("SHELL").unwrap_or_else(|_absent| "/bin/bash".to_owned());
        self.open_with_shell(root, &shell)?;
        Ok(shell)
    }

    /// Opens `shell` inside a PTY at `root`. Split out for tests.
    pub fn open_with_shell(&mut self, root: &Path, shell: &str) -> Result<(), TerminalError> {
        if self.is_open() {
            return Err(TerminalError::AlreadyOpen);
        }

        let mut child = Command::new("script")
            .arg("-qfc")
            .arg(format!("{shell} -i"))
            .arg("/dev/null")
            .current_dir(root)
            .env("TERM", "dumb")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|source| TerminalError::Process {
                message: format!("falha ao abrir o terminal ({shell}): {source}"),
            })?;

        self.stdin = child.stdin.take();
        let stdout = child.stdout.take();

        let pending_readers = Arc::new(AtomicUsize::new(0));
        if let Some(mut stdout) = stdout {
            let events = self.events.clone();
            let pending = Arc::clone(&pending_readers);
            pending.fetch_add(1, Ordering::SeqCst);
            thread::spawn(move || {
                let mut sanitizer = AnsiSanitizer::default();
                let mut buffer = [0_u8; READ_CHUNK_BYTES];
                while let Ok(bytes_read) = stdout.read(&mut buffer) {
                    if bytes_read == 0 {
                        break;
                    }
                    let raw = String::from_utf8_lossy(&buffer[..bytes_read]);
                    let clean = sanitizer.push(&raw);
                    if clean.is_empty() {
                        continue;
                    }
                    let event = JsonRpcRequest::notification(
                        "event.terminal.data",
                        Some(json!({ "data": clean })),
                    );
                    if events.send(event).is_err() {
                        break;
                    }
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
            drop(events.send(JsonRpcRequest::notification(
                "event.terminal.closed",
                Some(json!({ "exitCode": status.and_then(|status| status.code()) })),
            )));
        });

        Ok(())
    }

    /// Forwards raw `data` to the shell PTY.
    pub fn write(&mut self, data: &str) -> Result<(), TerminalError> {
        if !self.is_open() {
            return Err(TerminalError::NotOpen);
        }
        let Some(stdin) = self.stdin.as_mut() else {
            return Err(TerminalError::Process {
                message: "stdin do terminal nao esta disponivel".to_owned(),
            });
        };
        stdin
            .write_all(data.as_bytes())
            .and_then(|()| stdin.flush())
            .map_err(|source| TerminalError::Process {
                message: format!("falha ao escrever no terminal: {source}"),
            })
    }

    /// Kills the shell session. The waiter thread reports `closed`.
    pub fn close(&mut self) -> Result<(), TerminalError> {
        let Ok(mut guard) = self.child.lock() else {
            return Err(TerminalError::NotOpen);
        };
        let Some(child) = guard.as_mut() else {
            return Err(TerminalError::NotOpen);
        };
        child.kill().map_err(|source| TerminalError::Process {
            message: format!("falha ao fechar o terminal: {source}"),
        })
    }
}

/// Parser state of the ANSI sanitizer, kept across chunks.
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
enum SanitizerState {
    /// Plain text.
    #[default]
    Normal,
    /// Saw `ESC`, deciding the sequence kind.
    Escape,
    /// Inside `ESC [ ...` (CSI); ends at a byte in `@`..=`~`.
    Csi,
    /// Inside `ESC ] ...` (OSC); ends at BEL or `ESC \`.
    Osc,
    /// Inside OSC and saw `ESC`, expecting `\` to terminate.
    OscEscape,
}

/// Removes ANSI escape sequences and carriage returns from PTY output.
///
/// The state survives chunk boundaries, so a sequence split across two
/// reads is still stripped correctly.
#[derive(Debug, Default)]
pub struct AnsiSanitizer {
    state: SanitizerState,
}

impl AnsiSanitizer {
    /// Sanitizes one chunk, returning only printable text.
    pub fn push(&mut self, chunk: &str) -> String {
        let mut clean = String::with_capacity(chunk.len());
        for character in chunk.chars() {
            match self.state {
                SanitizerState::Normal => match character {
                    '\u{1b}' => self.state = SanitizerState::Escape,
                    '\r' => {}
                    _ => clean.push(character),
                },
                SanitizerState::Escape => {
                    self.state = match character {
                        '[' => SanitizerState::Csi,
                        ']' => SanitizerState::Osc,
                        _other => SanitizerState::Normal,
                    };
                }
                SanitizerState::Csi => {
                    if ('\u{40}'..='\u{7e}').contains(&character) {
                        self.state = SanitizerState::Normal;
                    }
                }
                SanitizerState::Osc => match character {
                    '\u{07}' => self.state = SanitizerState::Normal,
                    '\u{1b}' => self.state = SanitizerState::OscEscape,
                    _other => {}
                },
                SanitizerState::OscEscape => {
                    self.state = if character == '\\' {
                        SanitizerState::Normal
                    } else {
                        SanitizerState::Osc
                    };
                }
            }
        }
        clean
    }
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, sync::mpsc, time::Duration};

    use kernwerk_protocol::JsonRpcRequest;

    use super::{AnsiSanitizer, TerminalError, TerminalManager};

    fn temp_root(test_name: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("kernwerk-terminal-tests")
            .join(format!("{}-{test_name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir.canonicalize().unwrap()
    }

    fn collect_data_until(
        receiver: &mpsc::Receiver<JsonRpcRequest>,
        needle: &str,
    ) -> Option<String> {
        let mut seen = String::new();
        while let Ok(event) = receiver.recv_timeout(Duration::from_secs(10)) {
            if event.method == "event.terminal.data" {
                if let Some(params) = event.params.as_ref() {
                    if let Some(data) = params["data"].as_str() {
                        seen.push_str(data);
                    }
                }
            }
            if seen.contains(needle) {
                return Some(seen);
            }
            if event.method == "event.terminal.closed" {
                break;
            }
        }
        None
    }

    #[test]
    fn sanitizer_strips_csi_osc_and_carriage_returns() {
        let mut sanitizer = AnsiSanitizer::default();

        let clean = sanitizer.push("\u{1b}[01;32muser\u{1b}[00m$ ola\r\n");
        assert_eq!(clean, "user$ ola\n");

        let clean = sanitizer.push("\u{1b}]0;titulo\u{07}texto");
        assert_eq!(clean, "texto");
    }

    #[test]
    fn sanitizer_survives_sequences_split_across_chunks() {
        let mut sanitizer = AnsiSanitizer::default();

        let mut clean = sanitizer.push("antes\u{1b}[01;3");
        clean.push_str(&sanitizer.push("2mdepois"));

        assert_eq!(clean, "antesdepois");
    }

    #[test]
    fn shell_session_echoes_commands_and_closes() {
        let (sender, receiver) = mpsc::channel();
        let mut manager = TerminalManager::new(sender);
        let root = temp_root("session");

        if manager.open_with_shell(&root, "sh").is_err() {
            // Sem PTY disponivel neste ambiente (ex.: sandbox de CI).
            return;
        }
        assert!(manager.is_open());
        assert!(matches!(
            manager.open_with_shell(&root, "sh"),
            Err(TerminalError::AlreadyOpen)
        ));

        manager.write("echo terminal-ok $((3+4))\n").unwrap();
        let seen = collect_data_until(&receiver, "terminal-ok 7");
        assert!(seen.is_some(), "saida do shell nao chegou: {seen:?}");

        manager.write("exit\n").unwrap();
        let mut closed = false;
        while let Ok(event) = receiver.recv_timeout(Duration::from_secs(10)) {
            if event.method == "event.terminal.closed" {
                closed = true;
                break;
            }
        }
        assert!(closed);
        assert!(!manager.is_open());
        assert!(matches!(manager.write("x\n"), Err(TerminalError::NotOpen)));
    }
}

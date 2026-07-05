//! Synchronous line streaming for child processes.
//!
//! Both `build` and `test` spawn an external tool, drain its stdout/stderr
//! line by line while it runs, and inspect each line. This module owns that
//! plumbing (piping, reader threads, and the exit-status wait) so the callers
//! only provide a per-line callback and their own parsing.

use std::{
    io::{self, BufRead},
    process::{Command, ExitStatus, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread,
    time::{Duration, Instant},
};

/// Failure while spawning or waiting on a streamed child process.
///
/// The two variants are kept apart because a spawn failure usually means the
/// tool is missing (surfaced as `TOOL_NOT_FOUND`), while a wait failure is an
/// internal error.
#[derive(Debug)]
pub enum ProcessError {
    /// The command could not be started (often a missing binary).
    Spawn(io::Error),
    /// Waiting for the process to finish failed.
    Wait(io::Error),
}

/// Spawns `command`, forwarding each stdout/stderr line to `on_line` as it
/// arrives, and returns the process exit status once it finishes.
///
/// stdin is closed; stdout and stderr are piped and drained by dedicated
/// reader threads, so large output never deadlocks on a full pipe. Lines are
/// delivered without their trailing newline.
pub fn stream_command_lines(
    mut command: Command,
    on_line: &mut dyn FnMut(&'static str, String),
) -> Result<ExitStatus, ProcessError> {
    command.stdin(Stdio::null());
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    let mut child = command.spawn().map_err(ProcessError::Spawn)?;

    let (sender, receiver) = mpsc::channel::<(&'static str, String)>();
    let mut readers = Vec::new();

    if let Some(stdout) = child.stdout.take() {
        let sender = sender.clone();
        readers.push(thread::spawn(move || {
            for line in io::BufReader::new(stdout).lines().map_while(Result::ok) {
                if sender.send(("stdout", line)).is_err() {
                    break;
                }
            }
        }));
    }
    if let Some(stderr) = child.stderr.take() {
        let sender = sender.clone();
        readers.push(thread::spawn(move || {
            for line in io::BufReader::new(stderr).lines().map_while(Result::ok) {
                if sender.send(("stderr", line)).is_err() {
                    break;
                }
            }
        }));
    }
    drop(sender);

    for (stream, line) in receiver {
        on_line(stream, line);
    }

    for reader in readers {
        // Reader threads end when their pipe closes; a join error carries no
        // useful information here.
        drop(reader.join());
    }

    child.wait().map_err(ProcessError::Wait)
}

/// After the child exits, how long to keep draining output before returning.
///
/// A killed process may leave grandchildren holding the pipes open (a killed
/// shell whose child survives), so the reader threads would block forever. We
/// give them this window, then return regardless — the detached readers finish
/// when the pipe finally closes.
const CANCEL_DRAIN_DEADLINE: Duration = Duration::from_secs(2);

/// Like [`stream_command_lines`], but kills the child when `cancel` flips to
/// `true`, so long jobs (build, quality) can be cancelled.
///
/// The loop polls the output channel, the cancel flag and the child's exit; it
/// never blocks waiting on lingering grandchildren, so cancel returns promptly.
/// A cancelled process yields a non-success status.
pub fn stream_command_lines_cancelable(
    mut command: Command,
    cancel: &Arc<AtomicBool>,
    on_line: &mut dyn FnMut(&'static str, String),
) -> Result<ExitStatus, ProcessError> {
    command.stdin(Stdio::null());
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    let mut child = command.spawn().map_err(ProcessError::Spawn)?;

    let (sender, receiver) = mpsc::channel::<(&'static str, String)>();
    if let Some(stdout) = child.stdout.take() {
        let sender = sender.clone();
        thread::spawn(move || {
            for line in io::BufReader::new(stdout).lines().map_while(Result::ok) {
                if sender.send(("stdout", line)).is_err() {
                    break;
                }
            }
        });
    }
    if let Some(stderr) = child.stderr.take() {
        let sender = sender.clone();
        thread::spawn(move || {
            for line in io::BufReader::new(stderr).lines().map_while(Result::ok) {
                if sender.send(("stderr", line)).is_err() {
                    break;
                }
            }
        });
    }
    drop(sender);

    let mut killed = false;
    let mut exited: Option<ExitStatus> = None;
    let mut drain_deadline: Option<Instant> = None;

    loop {
        match receiver.recv_timeout(Duration::from_millis(50)) {
            Ok((stream, line)) => {
                on_line(stream, line);
                continue;
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
            Err(mpsc::RecvTimeoutError::Timeout) => {}
        }

        if !killed && cancel.load(Ordering::SeqCst) {
            drop(child.kill());
            killed = true;
        }
        if exited.is_none() {
            if let Ok(Some(status)) = child.try_wait() {
                exited = Some(status);
                drain_deadline = Some(Instant::now() + CANCEL_DRAIN_DEADLINE);
            }
        }
        if drain_deadline.is_some_and(|deadline| Instant::now() >= deadline) {
            break;
        }
    }

    exited.map_or_else(|| child.wait().map_err(ProcessError::Wait), Ok)
}

#[cfg(test)]
mod tests {
    use std::process::Command;

    use super::{ProcessError, stream_command_lines, stream_command_lines_cancelable};

    #[cfg(unix)]
    #[test]
    fn streams_stdout_and_stderr_lines_then_returns_status() {
        let mut command = Command::new("sh");
        command.arg("-c").arg("echo saida; echo erro 1>&2; exit 3");

        let mut lines = Vec::new();
        let status = stream_command_lines(command, &mut |stream, line| {
            lines.push((stream, line));
        })
        .unwrap();

        assert_eq!(status.code(), Some(3));
        assert!(lines.contains(&("stdout", "saida".to_owned())));
        assert!(lines.contains(&("stderr", "erro".to_owned())));
    }

    #[test]
    fn missing_binary_is_a_spawn_error() {
        let command = Command::new("kinein-binario-que-nao-existe");

        let error = stream_command_lines(command, &mut |_stream, _line| {}).unwrap_err();

        assert!(matches!(error, ProcessError::Spawn(_)));
    }

    #[cfg(unix)]
    #[test]
    fn cancel_kills_a_running_child_quickly() {
        use std::sync::{
            Arc,
            atomic::{AtomicBool, Ordering},
        };
        use std::time::{Duration, Instant};

        let cancel = Arc::new(AtomicBool::new(false));
        let flipper = Arc::clone(&cancel);
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(200));
            flipper.store(true, Ordering::SeqCst);
        });

        let mut command = Command::new("sh");
        command.arg("-c").arg("sleep 30");

        let start = Instant::now();
        let status =
            stream_command_lines_cancelable(command, &cancel, &mut |_stream, _line| {}).unwrap();

        assert!(
            start.elapsed() < Duration::from_secs(10),
            "o cancel deve matar o processo em vez de esperar o sleep inteiro"
        );
        assert!(!status.success());
    }
}

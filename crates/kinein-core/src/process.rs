//! Synchronous line streaming for child processes.
//!
//! Both `build` and `test` spawn an external tool, drain its stdout/stderr
//! line by line while it runs, and inspect each line. This module owns that
//! plumbing (piping, reader threads, and the exit-status wait) so the callers
//! only provide a per-line callback and their own parsing.

use std::{
    io::{self, BufRead},
    process::{Command, ExitStatus, Stdio},
    sync::mpsc,
    thread,
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

#[cfg(test)]
mod tests {
    use std::process::Command;

    use super::{ProcessError, stream_command_lines};

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
}

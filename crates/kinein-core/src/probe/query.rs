//! Deadline, cancellation and retained-output limits for probe-rs queries:
//! the listing shown on screen and the native flash validation share them.
use kinein_protocol::ProbeListResult;
use std::{
    path::{Path, PathBuf},
    process::Command,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};

const MAX_OUTPUT_BYTES: usize = 65_536;

/// Retained output of one bounded query and why it is not evidence, if so.
#[derive(Debug)]
pub struct Answer {
    /// Retained output without ANSI escapes.
    pub output: String,
    /// `None` only when the program finished successfully within the limits.
    pub failure: Option<Failure>,
}

/// Why a bounded query's output does not count.
#[derive(Debug)]
pub enum Failure {
    /// The program could not start.
    Spawn(String),
    /// Status, deadline, size or cancellation left the output partial.
    Incomplete(String),
}

/// Run `command` within the deadline and 64 KiB of retained output.
///
/// `cancel` belongs to the caller and is only read: an overflow stops this
/// query without marking the caller's whole operation as cancelled.
#[must_use]
pub fn run(command: Command, timeout: Duration, cancel: &Arc<AtomicBool>) -> Answer {
    let stop = Arc::new(AtomicBool::new(false));
    let done = Arc::new(AtomicBool::new(false));
    {
        let (cancel, stop, done) = (Arc::clone(cancel), Arc::clone(&stop), Arc::clone(&done));
        thread::spawn(move || {
            while !done.load(Ordering::SeqCst) {
                if cancel.load(Ordering::SeqCst) {
                    stop.store(true, Ordering::SeqCst);
                    break;
                }
                thread::sleep(Duration::from_millis(25));
            }
        });
    }
    let mut output = String::new();
    let mut overflow = false;
    let result = crate::process::stream_command_lines_with_timeout_cancelable(
        command,
        timeout,
        Arc::clone(&stop),
        &mut |_, line| {
            if output.len().saturating_add(line.len()).saturating_add(1) <= MAX_OUTPUT_BYTES {
                output.push_str(&line);
                output.push('\n');
            } else {
                overflow = true;
                stop.store(true, Ordering::SeqCst);
            }
        },
    );
    done.store(true, Ordering::SeqCst);
    let incomplete = |reason: &str| Some(Failure::Incomplete(reason.to_owned()));
    let failure = match result {
        Err(crate::process::ProcessError::Spawn(error)) => Some(Failure::Spawn(error.to_string())),
        Err(error) => incomplete(&format!("falha ao aguardar a consulta: {error:?}")),
        Ok(_) if cancel.load(Ordering::SeqCst) => incomplete("a consulta foi cancelada"),
        Ok((_, true)) => incomplete("o prazo da consulta expirou"),
        Ok(_) if overflow => incomplete("a saída excedeu 64 KiB e a consulta foi interrompida"),
        Ok((status, false)) if !status.success() => {
            incomplete(&format!("a consulta terminou com {status}"))
        }
        Ok(_) => None,
    };
    Answer {
        output: super::strip_ansi(&output),
        failure,
    }
}

pub(super) fn list(program: Option<&Path>, timeout: Duration) -> ProbeListResult {
    let program = program.map_or_else(
        || PathBuf::from(super::DEFAULT_PROBE_TOOL),
        Path::to_path_buf,
    );
    let mut command = Command::new(&program);
    command.arg("list").env("NO_COLOR", "1");
    let answer = run(command, timeout, &Arc::new(AtomicBool::new(false)));
    match answer.failure {
        Some(Failure::Spawn(error)) => failed(
            false,
            answer.output,
            format!(
                "{} não pôde iniciar: {error}. Confira a instalação e o adaptador fixado no kit; a lista não foi verificada.",
                program.display()
            ),
        ),
        Some(Failure::Incomplete(reason)) => failed(
            true,
            answer.output,
            format!(
                "{}: {reason}; a lista não foi verificada. Confira a saída e solicite uma nova lista.",
                program.display()
            ),
        ),
        None => {
            let probes = super::parse_list(&answer.output);
            let hint = super::hint_for(&probes, &answer.output);
            ProbeListResult {
                probes,
                tool_available: true,
                raw_output: answer.output,
                hint,
            }
        }
    }
}

const fn failed(tool_available: bool, raw_output: String, hint: String) -> ProbeListResult {
    ProbeListResult {
        probes: Vec::new(),
        tool_available,
        raw_output,
        hint: Some(hint),
    }
}

#[cfg(test)]
mod tests {
    use super::list;
    use std::{
        fs,
        path::PathBuf,
        time::{Duration, Instant},
    };

    fn tool(name: &str, body: &str) -> PathBuf {
        let root =
            std::env::temp_dir().join(format!("kinein-probe-query-{}-{name}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        crate::write_executable(root.join("query.sh"), format!("#!/bin/sh\n{body}\n"))
    }

    #[test]
    fn deadline_refuses_partial_output_and_terminates_the_query() {
        let tool = tool("deadline", "echo '[0]: Partial -- 1234:5678:S1'\nsleep 30");
        let start = Instant::now();
        let result = list(Some(&tool), Duration::from_millis(100));
        assert!(start.elapsed() < Duration::from_secs(3));
        assert!(
            result.tool_available && result.probes.is_empty(),
            "{result:?}"
        );
        assert!(result.raw_output.contains("Partial"));
        assert!(result.hint.unwrap().contains("prazo"));
    }

    #[test]
    fn excess_output_stops_the_query_and_never_publishes_partial_probes() {
        let tool = tool(
            "overflow",
            "echo '[0]: Partial -- 1234:5678:S1'\ni=0; while [ $i -lt 3000 ]; do echo '0123456789012345678901234567890123456789'; i=$((i+1)); done\nsleep 30",
        );
        let start = Instant::now();
        let result = list(Some(&tool), Duration::from_secs(4));
        assert!(start.elapsed() < Duration::from_secs(3));
        assert!(result.tool_available && result.probes.is_empty());
        assert!(result.raw_output.len() <= 65_536);
        assert!(result.hint.unwrap().contains("64 KiB"));
    }

    #[test]
    fn missing_program_reports_the_launch_failure() {
        let result = list(
            Some(std::path::Path::new("/missing/kinein/probe-query")),
            Duration::from_secs(1),
        );
        assert!(!result.tool_available);
        assert!(result.probes.is_empty() && result.raw_output.is_empty());
        assert!(result.hint.unwrap().contains("não pôde iniciar"));
    }
}

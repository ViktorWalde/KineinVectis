//! Deadline and retained-output limits for the existing probe listing query.
use kinein_protocol::ProbeListResult;
use std::{
    path::{Path, PathBuf},
    process::Command,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

const MAX_OUTPUT_BYTES: usize = 65_536;

pub(super) fn list(program: Option<&Path>, timeout: Duration) -> ProbeListResult {
    let program = program.map_or_else(
        || PathBuf::from(super::DEFAULT_PROBE_TOOL),
        Path::to_path_buf,
    );
    let mut command = Command::new(&program);
    command.arg("list").env("NO_COLOR", "1");
    let cancel = Arc::new(AtomicBool::new(false));
    let mut output = String::new();
    let mut overflow = false;
    let result = crate::process::stream_command_lines_with_timeout_cancelable(
        command,
        timeout,
        cancel.clone(),
        &mut |_, line| {
            if output.len().saturating_add(line.len()).saturating_add(1) <= MAX_OUTPUT_BYTES {
                output.push_str(&line);
                output.push('\n');
            } else {
                overflow = true;
                cancel.store(true, Ordering::SeqCst);
            }
        },
    );
    let output = super::strip_ansi(&output);
    let failure = match result {
        Err(crate::process::ProcessError::Spawn(error)) => {
            return failed(
                false,
                output,
                format!(
                    "{} não pôde iniciar: {error}. Confira a instalação e o adaptador fixado no kit; a lista não foi verificada.",
                    program.display()
                ),
            );
        }
        Err(error) => Some(format!("falha ao aguardar a consulta: {error:?}")),
        Ok((_, true)) => Some("o prazo da consulta expirou; a lista não foi verificada".to_owned()),
        Ok(_) if overflow => Some(
            "a saída excedeu 64 KiB; a consulta foi interrompida e a lista não foi verificada"
                .to_owned(),
        ),
        Ok((status, false)) if !status.success() => Some(format!(
            "a consulta terminou com {status}; a lista não foi verificada"
        )),
        Ok(_) => None,
    };
    if let Some(reason) = failure {
        return failed(
            true,
            output,
            format!(
                "{}: {reason}. Confira a saída e solicite uma nova lista.",
                program.display()
            ),
        );
    }
    let probes = super::parse_list(&output);
    let hint = super::hint_for(&probes, &output);
    ProbeListResult {
        probes,
        tool_available: true,
        raw_output: output,
        hint,
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

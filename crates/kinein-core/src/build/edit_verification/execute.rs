//! Isolated compiler invocation and bounded collection of its diagnostics.

use crate::{
    build::syntax_check::{SyntaxCheck, diagnostics_by_file},
    cdb::verification::ContextGuard,
};
use kinein_protocol::{Diagnostic, DiagnosticSeverity};
use std::{
    io::Read,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

pub(super) struct Invocation<'a> {
    pub check: &'a SyntaxCheck,
    pub overlay: &'a Path,
    pub mappings: &'a [(PathBuf, PathBuf)],
    pub sandbox: Option<&'a Path>,
    pub dependency: &'a Path,
    pub original: &'a Path,
}

pub(super) struct Output {
    pub errors: Vec<Diagnostic>,
    pub complete: bool,
}

fn command(input: &Invocation<'_>) -> Result<std::process::Command, String> {
    let check = input.check;
    let mut command =
        std::process::Command::new(input.sandbox.unwrap_or_else(|| Path::new(&check.program)));
    if input.sandbox.is_some() {
        command.args(["--die-with-parent", "--ro-bind", "/", "/"]);
        command.args(["--dev", "/dev"]);
        let private = input
            .dependency
            .parent()
            .ok_or("Área de dependências ausente.")?;
        command.arg("--bind").arg(private).arg(private);
        for (virtual_path, physical) in input.mappings {
            let canonical =
                crate::platform::canonicalize(virtual_path).map_err(|e| e.to_string())?;
            command.arg("--ro-bind").arg(physical).arg(canonical);
        }
        command.arg("--").arg(&check.program).args(&check.args);
    } else {
        command
            .args(&check.args)
            .arg("-ivfsoverlay")
            .arg(input.overlay);
    }
    command
        .arg("-MD")
        .arg("-MF")
        .arg(input.dependency)
        .args(["-MT", "kinein"])
        .current_dir(&check.directory)
        .env("LC_ALL", "C")
        .env("LANG", "C");
    Ok(command)
}

struct Collector {
    lines: Vec<String>,
    bytes: usize,
    overflow: bool,
    error: Option<String>,
    search: bool,
    roots: Vec<PathBuf>,
}

impl Collector {
    fn line(
        &mut self,
        line: String,
        input: &Invocation<'_>,
        guard: &mut ContextGuard,
        stop: &AtomicBool,
    ) {
        self.bytes = self.bytes.saturating_add(line.len());
        if self.bytes > 1_048_576 {
            self.overflow = true;
            stop.store(true, Ordering::SeqCst);
            return;
        }
        if line.starts_with("#include") && line.ends_with("search starts here:") {
            self.search = true;
            return;
        }
        if self.search {
            if line == "End of search list." {
                self.search = false;
            } else {
                self.roots.push(
                    input
                        .check
                        .directory
                        .join(line.trim().trim_end_matches(" (framework directory)")),
                );
            }
            return;
        }
        let trace = line.trim_start_matches('.');
        if trace.len() != line.len() && trace.starts_with(' ') {
            let path = input.check.directory.join(trace.trim_start());
            if crate::platform::canonicalize(&path).ok().as_deref() != Some(input.original)
                && let Err(error) = guard.observe_input(&path)
            {
                self.error = Some(error);
                stop.store(true, Ordering::SeqCst);
            }
        } else {
            self.lines.push(line);
        }
    }
}

pub(super) fn run(
    input: &Invocation<'_>,
    cancel: &Arc<AtomicBool>,
    deadline: Instant,
    guard: &mut ContextGuard,
) -> Result<Output, String> {
    if cancel.load(Ordering::SeqCst) || Instant::now() >= deadline {
        return Err("Verificação cancelada ou prazo excedido.".into());
    }
    let command = command(input)?;
    if input.dependency.exists() {
        std::fs::remove_file(input.dependency).map_err(|e| e.to_string())?;
    }
    let stop = Arc::new(AtomicBool::new(false));
    let finished = Arc::new(AtomicBool::new(false));
    let watchdog = {
        let (stop, finished, cancel) = (stop.clone(), finished.clone(), cancel.clone());
        std::thread::spawn(move || {
            while !finished.load(Ordering::SeqCst) {
                if cancel.load(Ordering::SeqCst) || Instant::now() >= deadline {
                    stop.store(true, Ordering::SeqCst);
                    break;
                }
                std::thread::sleep(Duration::from_millis(10));
            }
        })
    };
    let mut collected = Collector {
        lines: Vec::new(),
        bytes: 0,
        overflow: false,
        error: None,
        search: false,
        roots: Vec::new(),
    };
    let outcome =
        crate::process::stream_command_lines_bounded(command, &stop, 1_048_576, &mut |_, line| {
            collected.line(line, input, guard, &stop);
        });
    finished.store(true, Ordering::SeqCst);
    let _ = watchdog.join();
    if cancel.load(Ordering::SeqCst) {
        return Err("Verificação cancelada.".into());
    }
    if Instant::now() >= deadline {
        return Err("Prazo total da verificação excedido.".into());
    }
    if collected.overflow {
        return Err("Saída do compilador excede 1 MiB.".into());
    }
    if let Some(error) = collected.error {
        return Err(error);
    }
    let (status, truncated) = outcome.map_err(|e| format!("Compilador indisponível: {e:?}"))?;
    if truncated {
        return Err("Saída excede 1 MiB ou contém bytes inválidos.".into());
    }
    collected.roots.sort_by_key(|p| p.components().count());
    for root in collected.roots {
        guard.observe_directory(&root)?;
    }
    if let Ok(file) = std::fs::File::open(input.dependency) {
        let mut bytes = Vec::new();
        file.take(1_048_577)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        if bytes.len() > 1_048_576 {
            return Err("Lista de dependências excede 1 MiB.".into());
        }
        let text = std::str::from_utf8(&bytes).map_err(|_| "Lista de dependências não é UTF-8.")?;
        for path in super::dependencies::parse(text, &input.check.directory)? {
            if crate::platform::canonicalize(&path).ok().as_deref() != Some(input.original) {
                guard.observe_input(&path)?;
            }
        }
    } else if status.success() {
        return Err("Compilador não forneceu a lista de dependências.".into());
    }
    let errors = diagnostics_by_file(&collected.lines, &input.check.directory, None)
        .into_values()
        .flatten()
        .filter(|d| d.severity == DiagnosticSeverity::Error)
        .collect::<Vec<_>>();
    let incomplete = collected
        .lines
        .iter()
        .any(|line| line.contains(": fatal error: "));
    if status.code().is_none() || (!status.success() && errors.is_empty()) {
        return Err(format!(
            "Compilador sem evidência interpretável: {}",
            collected
                .lines
                .join("\n")
                .chars()
                .take(500)
                .collect::<String>()
        ));
    }
    Ok(Output {
        errors,
        complete: !incomplete,
    })
}

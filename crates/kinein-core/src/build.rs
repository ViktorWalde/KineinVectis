//! Build execution with streamed output and structured diagnostics.
//!
//! The core spawns the build tool (`cargo` or `cmake`), streams every output
//! line as an event, and extracts structured diagnostics: Cargo via
//! `--message-format=json`, CMake/compilers via the classic
//! `file:line:column: level: message` format.

use std::{
    error::Error,
    fmt, io,
    path::Path,
    process::Command,
    sync::{Arc, atomic::AtomicBool},
};

use kinein_protocol::{BuildDiagnostic, BuildDiagnosticSeverity, ProjectKind, RigorProfile};
use serde::Deserialize;

use crate::process::{self, ProcessError};

/// Flags de lint do clippy por perfil de rigor (fatia M4.5). Vão DEPOIS do
/// `--` do `cargo clippy`. Balanced usa o clippy default (sem extras).
#[must_use]
pub fn clippy_profile_args(profile: RigorProfile) -> Vec<&'static str> {
    match profile {
        RigorProfile::Strict => vec![
            "--",
            "-W",
            "clippy::pedantic",
            "-W",
            "clippy::nursery",
            "-D",
            "warnings",
        ],
        RigorProfile::Balanced => Vec::new(),
        RigorProfile::Relaxed => vec!["--", "-A", "clippy::all", "-W", "clippy::correctness"],
    }
}

/// `RUSTFLAGS` do `cargo build` por perfil: Strict trata warning como erro;
/// os outros deixam o cargo decidir. `None` = não setar o env.
#[must_use]
pub const fn rust_build_rustflags(profile: RigorProfile) -> Option<&'static str> {
    match profile {
        RigorProfile::Strict => Some("-D warnings"),
        RigorProfile::Balanced | RigorProfile::Relaxed => None,
    }
}

/// Event emitted while a build runs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuildEvent {
    /// A build command is about to run.
    Started {
        /// Human-readable command line.
        command: String,
    },
    /// One line of raw build output.
    Output {
        /// `stdout` or `stderr`.
        stream: &'static str,
        /// Output line without the trailing newline.
        line: String,
    },
    /// One structured diagnostic extracted from the output.
    Diagnostic(BuildDiagnostic),
}

/// Final result of a build.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuildOutcome {
    /// Whether every build step succeeded.
    pub success: bool,
    /// Exit code of the last build step, when it exited normally.
    pub exit_code: Option<i32>,
    /// Number of diagnostics emitted.
    pub diagnostics: u64,
}

/// Error produced while starting or driving a build.
#[derive(Debug)]
pub enum BuildError {
    /// The workspace project kind has no build integration yet.
    Unsupported {
        /// Project kind name as exposed by the protocol.
        kind: String,
    },
    /// The build tool could not be started.
    Spawn {
        /// Command that failed to start.
        command: String,
        /// Underlying IO error.
        source: io::Error,
    },
    /// IO failure while reading build output.
    Io(io::Error),
}

impl fmt::Display for BuildError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unsupported { kind } => {
                write!(
                    formatter,
                    "build ainda nao e suportado para projetos do tipo {kind}"
                )
            }
            Self::Spawn { command, source } => {
                write!(formatter, "falha ao iniciar '{command}': {source}")
            }
            Self::Io(error) => write!(formatter, "falha de IO durante o build: {error}"),
        }
    }
}

impl Error for BuildError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Spawn { source, .. } => Some(source),
            Self::Io(error) => Some(error),
            Self::Unsupported { .. } => None,
        }
    }
}

impl BuildError {
    /// Returns `true` when the failure is a missing build tool.
    #[must_use]
    pub const fn is_missing_tool(&self) -> bool {
        matches!(self, Self::Spawn { .. })
    }
}

/// How diagnostics are extracted from the tool output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DiagnosticFormat {
    /// Cargo `--message-format=json` on stdout.
    CargoJson,
    /// `file:line:column: level: message` lines from compilers and `CMake`.
    GccLike,
}

/// Serialized (camelCase) name of a project kind, for error messages.
pub(crate) fn project_kind_name(kind: ProjectKind) -> String {
    serde_json::to_value(kind)
        .ok()
        .and_then(|value| value.as_str().map(ToOwned::to_owned))
        .unwrap_or_else(|| format!("{kind:?}"))
}

/// Runs the build pipeline for the workspace project kind.
///
/// `cancel` is polled while the tool runs; flipping it to `true` kills the
/// build process (see [`crate::process::stream_command_lines_cancelable`]).
pub fn run_build(
    root: &Path,
    kind: ProjectKind,
    profile: RigorProfile,
    cancel: &Arc<AtomicBool>,
    sink: &mut dyn FnMut(BuildEvent),
) -> Result<BuildOutcome, BuildError> {
    match kind {
        ProjectKind::RustCargo => run_cargo_build(root, profile, cancel, sink),
        // C++ CMake -Werror por perfil fica para uma fatia futura (injetar
        // flag no build do usuario e invasivo — ver docs/diario/18 M4.5).
        ProjectKind::Cmake => run_cmake_build(root, cancel, sink),
        other => Err(BuildError::Unsupported {
            kind: project_kind_name(other),
        }),
    }
}

/// Runs `cargo check` for fast type/borrow feedback without codegen.
///
/// Same JSON stream as `cargo build`/`cargo clippy`, so diagnostics reuse
/// the existing parser and flow into the Problems panel unchanged.
pub fn run_cargo_check(
    root: &Path,
    cancel: &Arc<AtomicBool>,
    sink: &mut dyn FnMut(BuildEvent),
) -> Result<BuildOutcome, BuildError> {
    let mut command = Command::new("cargo");
    command
        .arg("check")
        .arg("--workspace")
        .arg("--all-targets")
        .arg("--message-format=json")
        .current_dir(root);
    stream_command(
        command,
        "cargo check",
        DiagnosticFormat::CargoJson,
        cancel,
        sink,
    )
}

/// Runs the lint/quality tool for the workspace project kind.
///
/// Reuses the build streaming and diagnostic parsing: for Rust, `cargo
/// clippy --message-format=json` speaks the exact JSON that `cargo build`
/// does, so its lints flow into the Problems panel with no new parser.
pub fn run_quality(
    root: &Path,
    kind: ProjectKind,
    profile: RigorProfile,
    cancel: &Arc<AtomicBool>,
    sink: &mut dyn FnMut(BuildEvent),
) -> Result<BuildOutcome, BuildError> {
    match kind {
        ProjectKind::RustCargo => {
            let mut command = Command::new("cargo");
            command
                .arg("clippy")
                .arg("--all-targets")
                .arg("--message-format=json")
                .args(clippy_profile_args(profile))
                .current_dir(root);
            stream_command(
                command,
                "cargo clippy",
                DiagnosticFormat::CargoJson,
                cancel,
                sink,
            )
        }
        other => Err(BuildError::Unsupported {
            kind: project_kind_name(other),
        }),
    }
}

fn run_cargo_build(
    root: &Path,
    profile: RigorProfile,
    cancel: &Arc<AtomicBool>,
    sink: &mut dyn FnMut(BuildEvent),
) -> Result<BuildOutcome, BuildError> {
    let mut command = Command::new("cargo");
    command
        .arg("build")
        .arg("--message-format=json")
        .current_dir(root);
    // Strict: warning vira erro no build do usuario (invalida o cache do
    // cargo ao trocar de perfil — aceito, ver docs/diario/18 M4.5).
    if let Some(flags) = rust_build_rustflags(profile) {
        command.env("RUSTFLAGS", flags);
    }

    stream_command(
        command,
        "cargo build",
        DiagnosticFormat::CargoJson,
        cancel,
        sink,
    )
}

fn run_cmake_build(
    root: &Path,
    cancel: &Arc<AtomicBool>,
    sink: &mut dyn FnMut(BuildEvent),
) -> Result<BuildOutcome, BuildError> {
    let build_dir = root.join(".kinein").join("build");

    if !build_dir.join("CMakeCache.txt").is_file() {
        let mut configure = Command::new("cmake");
        configure.arg("-S").arg(root).arg("-B").arg(&build_dir);

        let outcome = stream_command(
            configure,
            "cmake (configure)",
            DiagnosticFormat::GccLike,
            cancel,
            sink,
        )?;
        if !outcome.success {
            return Ok(outcome);
        }
    }

    let mut build = Command::new("cmake");
    build.arg("--build").arg(&build_dir);

    stream_command(
        build,
        "cmake --build",
        DiagnosticFormat::GccLike,
        cancel,
        sink,
    )
}

fn stream_command(
    command: Command,
    display_name: &str,
    format: DiagnosticFormat,
    cancel: &Arc<AtomicBool>,
    sink: &mut dyn FnMut(BuildEvent),
) -> Result<BuildOutcome, BuildError> {
    sink(BuildEvent::Started {
        command: display_name.to_owned(),
    });

    let mut diagnostics: u64 = 0;
    let mut on_line = |stream: &'static str, line: String| match format {
        DiagnosticFormat::CargoJson if stream == "stdout" => {
            // Linhas JSON do cargo nao vao para a saida bruta; o texto
            // renderizado do diagnostico e emitido no lugar.
            if let Some(diagnostic) = parse_cargo_json_line(&line) {
                if let Some(rendered) = &diagnostic.rendered {
                    for rendered_line in rendered.lines() {
                        sink(BuildEvent::Output {
                            stream: "stdout",
                            line: rendered_line.to_owned(),
                        });
                    }
                }
                if let Some(structured) = diagnostic.structured {
                    diagnostics += 1;
                    sink(BuildEvent::Diagnostic(structured));
                }
            }
        }
        _ => {
            if let Some(structured) = parse_gcc_like_line(&line) {
                diagnostics += 1;
                sink(BuildEvent::Diagnostic(structured));
            }
            sink(BuildEvent::Output { stream, line });
        }
    };

    let status = process::stream_command_lines_cancelable(command, cancel, &mut on_line).map_err(
        |error| match error {
            ProcessError::Spawn(source) => BuildError::Spawn {
                command: display_name.to_owned(),
                source,
            },
            ProcessError::Wait(source) => BuildError::Io(source),
        },
    )?;

    Ok(BuildOutcome {
        success: status.success(),
        exit_code: status.code(),
        diagnostics,
    })
}

/// Parsed cargo JSON message: rendered text plus optional structured data.
struct CargoParsed {
    rendered: Option<String>,
    structured: Option<BuildDiagnostic>,
}

#[derive(Deserialize)]
struct CargoMessage {
    reason: String,
    message: Option<CargoCompilerMessage>,
}

#[derive(Deserialize)]
struct CargoCompilerMessage {
    level: String,
    message: String,
    rendered: Option<String>,
    #[serde(default)]
    spans: Vec<CargoSpan>,
}

#[derive(Deserialize)]
struct CargoSpan {
    file_name: String,
    line_start: u64,
    column_start: u64,
    is_primary: bool,
}

fn parse_cargo_json_line(line: &str) -> Option<CargoParsed> {
    let message = serde_json::from_str::<CargoMessage>(line).ok()?;
    if message.reason != "compiler-message" {
        return None;
    }
    let compiler_message = message.message?;

    let severity = match compiler_message.level.as_str() {
        "error" | "error: internal compiler error" => BuildDiagnosticSeverity::Error,
        "warning" => BuildDiagnosticSeverity::Warning,
        _ => {
            return Some(CargoParsed {
                rendered: compiler_message.rendered,
                structured: None,
            });
        }
    };

    let primary_span = compiler_message
        .spans
        .iter()
        .find(|span| span.is_primary)
        .or_else(|| compiler_message.spans.first());

    Some(CargoParsed {
        rendered: compiler_message.rendered.clone(),
        structured: Some(BuildDiagnostic {
            severity,
            message: compiler_message.message,
            file: primary_span.map(|span| span.file_name.clone()),
            line: primary_span.map(|span| span.line_start),
            column: primary_span.map(|span| span.column_start),
        }),
    })
}

/// Parses `file:line[:column]: level: message` lines from compilers and `CMake`.
fn parse_gcc_like_line(line: &str) -> Option<BuildDiagnostic> {
    const MARKERS: [(&str, BuildDiagnosticSeverity); 4] = [
        (": fatal error: ", BuildDiagnosticSeverity::Error),
        (": error: ", BuildDiagnosticSeverity::Error),
        (": warning: ", BuildDiagnosticSeverity::Warning),
        (": note: ", BuildDiagnosticSeverity::Note),
    ];

    for (marker, severity) in MARKERS {
        let Some(position) = line.find(marker) else {
            continue;
        };
        let location = &line[..position];
        let message = line[position + marker.len()..].trim().to_owned();
        if message.is_empty() {
            return None;
        }

        // Tenta arquivo:linha:coluna, depois arquivo:linha.
        let mut pieces = location.rsplitn(3, ':');
        let last = pieces.next()?;
        let middle = pieces.next();
        let head = pieces.next();

        if let (Some(head), Some(middle)) = (head, middle) {
            if let (Ok(line_number), Ok(column)) = (middle.parse::<u64>(), last.parse::<u64>()) {
                return Some(BuildDiagnostic {
                    severity,
                    message,
                    file: Some(head.to_owned()),
                    line: Some(line_number),
                    column: Some(column),
                });
            }
        }

        let mut pieces = location.rsplitn(2, ':');
        let last = pieces.next()?;
        let head = pieces.next();
        if let (Some(head), Ok(line_number)) = (head, last.parse::<u64>()) {
            return Some(BuildDiagnostic {
                severity,
                message,
                file: Some(head.to_owned()),
                line: Some(line_number),
                column: None,
            });
        }

        return Some(BuildDiagnostic {
            severity,
            message,
            file: None,
            line: None,
            column: None,
        });
    }

    None
}

#[cfg(test)]
mod tests {
    use kinein_protocol::BuildDiagnosticSeverity;

    use super::{BuildEvent, parse_cargo_json_line, parse_gcc_like_line};

    #[test]
    fn gcc_line_with_column_is_parsed() {
        let diagnostic =
            parse_gcc_like_line("src/main.cpp:42:13: error: expected ';' after expression")
                .unwrap();

        assert_eq!(diagnostic.severity, BuildDiagnosticSeverity::Error);
        assert_eq!(diagnostic.file.as_deref(), Some("src/main.cpp"));
        assert_eq!(diagnostic.line, Some(42));
        assert_eq!(diagnostic.column, Some(13));
        assert!(diagnostic.message.contains("expected"));
    }

    #[test]
    fn gcc_line_without_column_is_parsed() {
        let diagnostic =
            parse_gcc_like_line("CMakeLists.txt:7: error: unknown command foo").unwrap();

        assert_eq!(diagnostic.file.as_deref(), Some("CMakeLists.txt"));
        assert_eq!(diagnostic.line, Some(7));
        assert_eq!(diagnostic.column, None);
    }

    #[test]
    fn ordinary_lines_are_not_diagnostics() {
        assert!(parse_gcc_like_line("[2/10] Building CXX object foo.o").is_none());
        assert!(parse_gcc_like_line("Compiling kinein-core v0.1.0").is_none());
    }

    #[test]
    fn cargo_compiler_message_becomes_diagnostic() {
        let line = r#"{"reason":"compiler-message","message":{"level":"error","message":"mismatched types","rendered":"error[E0308]: mismatched types\n","spans":[{"file_name":"src/lib.rs","line_start":10,"column_start":5,"is_primary":true}]}}"#;

        let parsed = parse_cargo_json_line(line).unwrap();
        let diagnostic = parsed.structured.unwrap();

        assert_eq!(diagnostic.severity, BuildDiagnosticSeverity::Error);
        assert_eq!(diagnostic.file.as_deref(), Some("src/lib.rs"));
        assert_eq!(diagnostic.line, Some(10));
        assert!(parsed.rendered.unwrap().contains("E0308"));
    }

    #[test]
    fn cargo_non_compiler_messages_are_ignored() {
        assert!(parse_cargo_json_line(r#"{"reason":"build-finished","success":true}"#).is_none());
        assert!(parse_cargo_json_line("nao e json").is_none());
    }

    #[cfg(unix)]
    #[test]
    fn stream_command_emits_output_and_diagnostics() {
        use std::process::Command;

        let mut command = Command::new("sh");
        command.arg("-c").arg(concat!(
            "echo 'linha normal'; ",
            "echo 'main.c:3:1: error: algo errado' 1>&2; ",
            "exit 2"
        ));

        let cancel = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let mut events = Vec::new();
        let outcome = super::stream_command(
            command,
            "sh de teste",
            super::DiagnosticFormat::GccLike,
            &cancel,
            &mut |event| events.push(event),
        )
        .unwrap();

        assert!(!outcome.success);
        assert_eq!(outcome.exit_code, Some(2));
        assert_eq!(outcome.diagnostics, 1);
        assert!(events.iter().any(|event| matches!(
            event,
            BuildEvent::Output { stream: "stdout", line } if line == "linha normal"
        )));
        assert!(events.iter().any(|event| matches!(
            event,
            BuildEvent::Diagnostic(diagnostic) if diagnostic.line == Some(3)
        )));
    }

    #[test]
    fn run_quality_rejects_kinds_without_linter() {
        use kinein_protocol::{ProjectKind, RigorProfile};

        // Cmake tem build integrado mas ainda nao tem analise de qualidade.
        let root = std::env::temp_dir();
        let cancel = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let error = super::run_quality(
            &root,
            ProjectKind::Cmake,
            RigorProfile::Strict,
            &cancel,
            &mut |_event| {},
        )
        .unwrap_err();

        assert!(matches!(error, super::BuildError::Unsupported { .. }));
        assert!(!error.is_missing_tool());
    }

    #[test]
    fn clippy_and_build_flags_vary_by_rigor_profile() {
        use kinein_protocol::RigorProfile;

        // Strict: pedantic/nursery + deny warnings; RUSTFLAGS deny warnings.
        let strict = super::clippy_profile_args(RigorProfile::Strict);
        assert!(strict.contains(&"clippy::pedantic"));
        assert!(strict.contains(&"warnings"));
        assert_eq!(
            super::rust_build_rustflags(RigorProfile::Strict),
            Some("-D warnings")
        );
        // Balanced: clippy default (sem flags extras); build sem RUSTFLAGS.
        assert!(super::clippy_profile_args(RigorProfile::Balanced).is_empty());
        assert_eq!(super::rust_build_rustflags(RigorProfile::Balanced), None);
        // Relaxed: allow-all + so correctness; build sem RUSTFLAGS.
        let relaxed = super::clippy_profile_args(RigorProfile::Relaxed);
        assert!(relaxed.contains(&"clippy::correctness"));
        assert_eq!(super::rust_build_rustflags(RigorProfile::Relaxed), None);
    }
}

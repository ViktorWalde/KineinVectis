//! Build execution with streamed output and structured diagnostics.
//!
//! The core spawns the build tool (`cargo` or `cmake`), streams every output
//! line as an event, and extracts structured diagnostics: Cargo via
//! `--message-format=json`, CMake/compilers via the classic
//! `file:line:column: level: message` format.

mod parse;

use std::{
    error::Error,
    fmt, io,
    path::Path,
    process::Command,
    sync::{Arc, atomic::AtomicBool},
};

use kinein_protocol::{BuildDiagnostic, ProjectKind, RigorProfile, ToolchainRole};

use parse::{parse_cargo_json_line, parse_gcc_like_line};

use crate::toolchain::Toolchain;

use crate::process::{self, ProcessError};

/// Regras do ruff por perfil de rigor — SO' quando o projeto nao as declara.
///
/// `ruff.toml`, `.ruff.toml` ou `[tool.ruff]` no `pyproject.toml` vencem o
/// perfil, como o `.clang-tidy` vence. Balanced = o padrao do ruff (E4, E7,
/// E9, F); Strict acrescenta os conjuntos que o mercado liga (W, I, UP, B, N);
/// Relaxed so' o que quebra (`E9` sintaxe, `F63`/`F7`/`F82` nomes e
/// comparacoes indefinidos).
#[must_use]
pub fn ruff_profile_args(root: &Path, profile: RigorProfile) -> Vec<&'static str> {
    let projeto_declara = root.join("ruff.toml").is_file()
        || root.join(".ruff.toml").is_file()
        || std::fs::read_to_string(root.join("pyproject.toml"))
            .is_ok_and(|t| t.contains("[tool.ruff"));
    if projeto_declara {
        return Vec::new();
    }
    match profile {
        RigorProfile::Strict => vec!["--select", "E,F,W,I,UP,B,N"],
        RigorProfile::Balanced => Vec::new(),
        RigorProfile::Relaxed => vec!["--select", "E9,F63,F7,F82"],
    }
}

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
    /// The tool the project kind needs is not on this machine.
    ToolMissing {
        /// Tool id (`ruff`).
        tool: String,
        /// How to get it, with the official step.
        hint: String,
    },
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
            Self::ToolMissing { tool, hint } => {
                write!(formatter, "{tool} nao foi detectado nesta maquina: {hint}")
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
            Self::Unsupported { .. } | Self::ToolMissing { .. } => None,
        }
    }
}

impl BuildError {
    /// Returns `true` when the failure is a missing build tool.
    #[must_use]
    pub const fn is_missing_tool(&self) -> bool {
        matches!(self, Self::Spawn { .. } | Self::ToolMissing { .. })
    }
}

/// How diagnostics are extracted from the tool output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DiagnosticFormat {
    /// Cargo `--message-format=json` on stdout.
    CargoJson,
    /// `file:line:column: level: message` lines from compilers and `CMake`.
    GccLike,
    /// `ruff check --output-format concise`: `file:line:column: CODE message`.
    RuffConcise,
}

/// Serialized (camelCase) name of a project kind, for error messages.
pub(crate) fn project_kind_name(kind: ProjectKind) -> String {
    serde_json::to_value(kind)
        .ok()
        .and_then(|value| value.as_str().map(ToOwned::to_owned))
        .unwrap_or_else(|| format!("{kind:?}"))
}

/// Acrescenta `--target <triple>` quando o kit escolheu um alvo.
///
/// Vale para check, clippy e build: compilar o binario para o alvo e checar
/// para o host daria diagnostico do host — que e' pior que nao ter, porque
/// parece certo. Sem alvo escolhido nada muda e o cargo decide como sempre.
fn aplica_alvo(command: &mut Command, toolchain: &Toolchain) {
    if let Some(triple) = toolchain.target_triple() {
        command.arg("--target").arg(triple);
    }
}

/// Executavel de um papel da toolchain, caindo no nome nu quando nao ha
/// escolha fixada — que e o padrao e mantem o `PATH` no comando.
fn programa(toolchain: &Toolchain, role: ToolchainRole, padrao: &str) -> std::path::PathBuf {
    toolchain
        .program_for(role)
        .unwrap_or_else(|| std::path::PathBuf::from(padrao))
}

/// Runs the build pipeline for the workspace project kind.
///
/// `cancel` is polled while the tool runs; flipping it to `true` kills the
/// build process (see [`crate::process::stream_command_lines_cancelable`]).
pub fn run_build(
    root: &Path,
    kind: ProjectKind,
    profile: RigorProfile,
    toolchain: &Toolchain,
    cancel: &Arc<AtomicBool>,
    sink: &mut dyn FnMut(BuildEvent),
) -> Result<BuildOutcome, BuildError> {
    match kind {
        ProjectKind::RustCargo => run_cargo_build(root, profile, toolchain, cancel, sink),
        // C++ CMake -Werror por perfil fica para uma fatia futura (injetar
        // flag no build do usuario e invasivo — ver DocsPrivate/diario/18 M4.5).
        ProjectKind::Cmake => run_cmake_build(root, toolchain, cancel, sink),
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
    toolchain: &Toolchain,
    cancel: &Arc<AtomicBool>,
    sink: &mut dyn FnMut(BuildEvent),
) -> Result<BuildOutcome, BuildError> {
    let mut command = Command::new(programa(toolchain, ToolchainRole::Cargo, "cargo"));
    command
        .arg("check")
        .arg("--workspace")
        .arg("--all-targets")
        .arg("--message-format=json")
        .current_dir(root);
    aplica_alvo(&mut command, toolchain);
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
    toolchain: &Toolchain,
    ruff: Option<&Path>,
    cancel: &Arc<AtomicBool>,
    sink: &mut dyn FnMut(BuildEvent),
) -> Result<BuildOutcome, BuildError> {
    match kind {
        // Python (cadeia do roadmaps/41 bloco B, fatia 2, 2026-09-12): o ruff
        // DETECTADO, `check` no root com a configuracao do projeto
        // (ruff.toml / pyproject [tool.ruff]); o perfil de rigor escolhe as
        // regras so' quando o projeto nao as declara.
        ProjectKind::Python => {
            let Some(ruff) = ruff else {
                return Err(BuildError::ToolMissing {
                    tool: "ruff".to_owned(),
                    hint: "instale o ruff (o painel de instalacao mostra o passo oficial: pipx \
                           install ruff)"
                        .to_owned(),
                });
            };
            let mut command = Command::new(ruff);
            command
                .args(["check", "--output-format", "concise", "--no-fix"])
                .args(ruff_profile_args(root, profile))
                .arg(".")
                .current_dir(root);
            stream_command(
                command,
                "ruff check",
                DiagnosticFormat::RuffConcise,
                cancel,
                sink,
            )
        }
        ProjectKind::RustCargo => {
            let mut command = Command::new(programa(toolchain, ToolchainRole::Cargo, "cargo"));
            command
                .arg("clippy")
                .arg("--all-targets")
                .arg("--message-format=json")
                .args(clippy_profile_args(profile))
                .current_dir(root);
            aplica_alvo(&mut command, toolchain);
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
    toolchain: &Toolchain,
    cancel: &Arc<AtomicBool>,
    sink: &mut dyn FnMut(BuildEvent),
) -> Result<BuildOutcome, BuildError> {
    let mut command = Command::new(programa(toolchain, ToolchainRole::Cargo, "cargo"));
    command
        .arg("build")
        .arg("--message-format=json")
        .current_dir(root);
    aplica_alvo(&mut command, toolchain);
    // Strict: warning vira erro no build do usuario (invalida o cache do
    // cargo ao trocar de perfil — aceito, ver DocsPrivate/diario/18 M4.5).
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
    toolchain: &Toolchain,
    cancel: &Arc<AtomicBool>,
    sink: &mut dyn FnMut(BuildEvent),
) -> Result<BuildOutcome, BuildError> {
    let build_dir = root.join(".kinein").join("build");

    if !build_dir.join("CMakeCache.txt").is_file() {
        let mut configure = Command::new(programa(toolchain, ToolchainRole::Cmake, "cmake"));
        configure.arg("-S").arg(root).arg("-B").arg(&build_dir);
        configure.args(toolchain.cmake_arguments());

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

    let mut build = Command::new(programa(toolchain, ToolchainRole::Cmake, "cmake"));
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
        DiagnosticFormat::RuffConcise => {
            if let Some(structured) = parse::parse_ruff_concise_line(&line) {
                diagnostics += 1;
                sink(BuildEvent::Diagnostic(structured));
            }
            sink(BuildEvent::Output { stream, line });
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

#[cfg(test)]
mod tests {
    use super::BuildEvent;

    /// Perfil de rigor do ruff (fatia Python 2): a IDE so' escolhe regras quando
    /// o PROJETO nao declarou as suas — `ruff.toml`, `.ruff.toml` ou
    /// `[tool.ruff*]` no `pyproject.toml` vencem sempre. Sem declaracao,
    /// Balanced = default do ruff, Strict amplia, Relaxed so' o que quebra.
    #[test]
    fn ruff_profile_args_defer_to_the_project_when_it_declares_rules() {
        use super::ruff_profile_args;
        use kinein_protocol::RigorProfile;

        let dir = std::env::temp_dir()
            .join("kinein-core-tests")
            .join(format!("{}-ruff-profile", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        // Sem declaracao: o perfil manda.
        assert_eq!(
            ruff_profile_args(&dir, RigorProfile::Strict),
            vec!["--select", "E,F,W,I,UP,B,N"]
        );
        assert!(ruff_profile_args(&dir, RigorProfile::Balanced).is_empty());
        assert_eq!(
            ruff_profile_args(&dir, RigorProfile::Relaxed),
            vec!["--select", "E9,F63,F7,F82"]
        );

        // pyproject sem [tool.ruff]: continua sendo o perfil.
        std::fs::write(dir.join("pyproject.toml"), "[project]\nname = \"x\"\n").unwrap();
        assert!(!ruff_profile_args(&dir, RigorProfile::Strict).is_empty());
        // [tool.ruff.lint] (subtabela) tambem conta como declaracao.
        std::fs::write(
            dir.join("pyproject.toml"),
            "[project]\nname = \"x\"\n[tool.ruff.lint]\nselect = [\"E\"]\n",
        )
        .unwrap();
        assert!(ruff_profile_args(&dir, RigorProfile::Strict).is_empty());
        assert!(ruff_profile_args(&dir, RigorProfile::Relaxed).is_empty());

        // ruff.toml e .ruff.toml, cada um sozinho.
        std::fs::remove_file(dir.join("pyproject.toml")).unwrap();
        std::fs::write(dir.join("ruff.toml"), "").unwrap();
        assert!(ruff_profile_args(&dir, RigorProfile::Strict).is_empty());
        std::fs::remove_file(dir.join("ruff.toml")).unwrap();
        std::fs::write(dir.join(".ruff.toml"), "").unwrap();
        assert!(ruff_profile_args(&dir, RigorProfile::Strict).is_empty());
        let _ = std::fs::remove_dir_all(&dir);
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
            &crate::toolchain::Toolchain::resolve(&root, &[]),
            None,
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

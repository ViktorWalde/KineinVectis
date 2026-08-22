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
    target: Option<&str>,
    cancel: &Arc<AtomicBool>,
    sink: &mut dyn FnMut(BuildEvent),
) -> Result<BuildOutcome, BuildError> {
    match kind {
        // Cargo nao recebe alvo nesta fatia: `cmake.targets.list` e a fonte da
        // lista, e ela e do CMake. Aceitar um alvo aqui e ignora-lo seria pior
        // que recusar — o usuario veria uma escolha sem efeito.
        ProjectKind::RustCargo => run_cargo_build(root, profile, cancel, sink),
        // C++ CMake -Werror por perfil fica para uma fatia futura (injetar
        // flag no build do usuario e invasivo — ver docsprivate/diario/18 M4.5).
        ProjectKind::Cmake => run_cmake_build(root, target, cancel, sink),
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
    extras: &QualityExtras,
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
        ProjectKind::Cmake => run_cmake_quality(root, profile, extras, cancel, sink),
        other => Err(BuildError::Unsupported {
            kind: project_kind_name(other),
        }),
    }
}

/// Argumentos extras vindos da configuracao das integracoes, por ferramenta.
///
/// Um struct, e nao mais parametros posicionais: o funil de C/C++ passou a ter
/// DOIS analisadores na fatia 4 do L2, e `&[String]` solto ja nao dizia de
/// quem eram os argumentos. Cada campo vem de `integration.config` (fatia
/// 2.2), chave `args`, com o escopo de workspace sobrepondo o global.
#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct QualityExtras {
    /// Extras do Cppcheck (id `cppcheck`).
    pub cppcheck: Vec<String>,
    /// Extras do clang-tidy (id `clang-tidy`).
    pub clang_tidy: Vec<String>,
}

/// Analise estatica de C/C++: Cppcheck e, quando ha `compile_commands.json`,
/// o Clang Static Analyzer via `clang-tidy`.
///
/// Os dois no MESMO job de propósito. Eles nao competem: o Cppcheck le o
/// codigo sem saber como ele e compilado (roda sempre, ate antes de
/// configurar), e o clang-tidy usa o comando de compilacao REAL de cada
/// arquivo, entao enxerga macro, include e flag que o outro nao ve. Rodar so
/// um dos dois deixaria metade dos achados na mesa.
///
/// L2 fatia 1 (2026-07-19) trouxe o Cppcheck; a fatia 4 (2026-08-21) somou o
/// clang-tidy. Ambos falam `GccLike` no parser que ja existia — zero parser
/// novo, e os achados caem na aba Problems como os do build.
fn run_cmake_quality(
    root: &Path,
    profile: RigorProfile,
    extras: &QualityExtras,
    cancel: &Arc<AtomicBool>,
    sink: &mut dyn FnMut(BuildEvent),
) -> Result<BuildOutcome, BuildError> {
    let mut command = Command::new("cppcheck");
    command
        .args(cppcheck_args(profile, &extras.cppcheck))
        .current_dir(root);
    let cppcheck = stream_command(command, "cppcheck", DiagnosticFormat::GccLike, cancel, sink)?;

    let build_dir = root.join(".kinein").join("build");
    let arquivos = clang_tidy_sources(&build_dir, root);
    if arquivos.is_empty() {
        // Sem `compile_commands.json` o clang-tidy nao sabe COMO cada arquivo
        // e compilado, e um analisador que adivinha flags produz achado que
        // nao existe. Pular dizendo o motivo e melhor que analisar mentindo.
        sink(BuildEvent::Output {
            stream: "stdout",
            line: "clang-tidy pulado: configure o CMake para gerar \
                   .kinein/build/compile_commands.json"
                .to_owned(),
        });
        return Ok(cppcheck);
    }

    let mut tidy = Command::new("clang-tidy");
    tidy.args(clang_tidy_args(
        profile,
        &extras.clang_tidy,
        &build_dir,
        root.join(".clang-tidy").is_file(),
    ))
    .args(&arquivos)
    .current_dir(root);
    let tidy_outcome = stream_command(tidy, "clang-tidy", DiagnosticFormat::GccLike, cancel, sink);

    // clang-tidy ausente NAO invalida o que o Cppcheck ja achou. Ele e o
    // segundo analisador, nao um pre-requisito: quem so tem um instalado
    // continua tendo analise, e a falta aparece na saida do job.
    let tidy_outcome = match tidy_outcome {
        Ok(outcome) => outcome,
        Err(error) if error.is_missing_tool() => {
            sink(BuildEvent::Output {
                stream: "stderr",
                line: "clang-tidy nao encontrado no PATH: analise seguiu so com o Cppcheck"
                    .to_owned(),
            });
            return Ok(cppcheck);
        }
        Err(error) => return Err(error),
    };

    Ok(BuildOutcome {
        success: cppcheck.success && tidy_outcome.success,
        // O ultimo passo executado e o que responde pelo codigo de saida,
        // mesma regra do run_cmake_build (configure -> build).
        exit_code: tidy_outcome.exit_code,
        diagnostics: cppcheck.diagnostics + tidy_outcome.diagnostics,
    })
}

/// Arquivos a analisar, lidos do `compile_commands.json` do build dir.
///
/// So entram os que ficam DENTRO da raiz do workspace: o banco de comandos
/// tambem lista codigo gerado e dependencia de terceiros, e apontar o
/// analisador para o que o usuario nao escreveu enche a aba Problems de ruido
/// que ele nao pode corrigir. Duplicatas caem fora (um mesmo .cpp aparece uma
/// vez por alvo que o compila).
#[must_use]
fn clang_tidy_sources(build_dir: &Path, root: &Path) -> Vec<String> {
    let Ok(raw) = std::fs::read_to_string(build_dir.join("compile_commands.json")) else {
        return Vec::new();
    };
    let Ok(entradas) = serde_json::from_str::<Vec<serde_json::Value>>(&raw) else {
        return Vec::new();
    };
    let mut vistos = std::collections::BTreeSet::new();
    for entrada in entradas {
        let Some(arquivo) = entrada.get("file").and_then(serde_json::Value::as_str) else {
            continue;
        };
        let caminho = Path::new(arquivo);
        let absoluto = if caminho.is_absolute() {
            caminho.to_path_buf()
        } else {
            root.join(caminho)
        };
        if !absoluto.starts_with(root) || absoluto.starts_with(build_dir) {
            continue;
        }
        if let Some(texto) = absoluto.to_str() {
            vistos.insert(texto.to_owned());
        }
    }
    vistos.into_iter().collect()
}

/// Argumentos do `clang-tidy` por perfil de rigor + extras da configuracao.
///
/// `tem_config_do_projeto` decide o unico ponto delicado: se o projeto tem o
/// PROPRIO `.clang-tidy`, nao passamos `--checks`. Sobrepor a escolha do
/// projeto seria a mesma invasao que a IDE recusa em `--coverage` e `-Werror`
/// (docsprivate/diario/18 M4.5) — rigor e decisao de quem escreve o codigo.
/// Sem config no projeto, o padrao e o Clang Static Analyzer, que e o que o
/// roadmap 28 pede no L2.
#[must_use]
pub fn clang_tidy_args(
    profile: RigorProfile,
    extra_args: &[String],
    build_dir: &Path,
    tem_config_do_projeto: bool,
) -> Vec<String> {
    let mut args: Vec<String> = vec![
        "-p".to_owned(),
        build_dir.to_string_lossy().into_owned(),
        // Sem isto o clang-tidy imprime "N warnings generated." por arquivo, e
        // o parser GccLike ignoraria — mas a saida bruta do job viraria ruido.
        "--quiet".to_owned(),
    ];
    if !tem_config_do_projeto {
        args.push(format!("--checks=-*,{}", clang_tidy_checks(profile)));
    }
    args.extend(extra_args.iter().cloned());
    args
}

/// Conjunto de checks por perfil, quando o projeto nao tem `.clang-tidy`.
const fn clang_tidy_checks(profile: RigorProfile) -> &'static str {
    match profile {
        // O analisador de caminho: use-after-free, null deref, vazamento.
        RigorProfile::Relaxed => "clang-analyzer-*",
        RigorProfile::Balanced => "clang-analyzer-*,bugprone-*",
        RigorProfile::Strict => "clang-analyzer-*,bugprone-*,cert-*,performance-*",
    }
}

/// Argumentos do Cppcheck por perfil de rigor + extras da configuracao.
///
/// Os extras vem da config da integracao (id `cppcheck`, chave `args`) — a
/// primeira consumidora real do `integration.config` da fatia 2.2 — e entram
/// DEPOIS dos flags de perfil, entao o usuario pode sobrepor o que quiser.
#[must_use]
pub fn cppcheck_args(profile: RigorProfile, extra_args: &[String]) -> Vec<String> {
    let mut args: Vec<String> = vec![
        "--template=gcc".to_owned(),
        "--quiet".to_owned(),
        match profile {
            RigorProfile::Strict => "--enable=warning,style,performance,portability".to_owned(),
            RigorProfile::Balanced => "--enable=warning,style".to_owned(),
            RigorProfile::Relaxed => "--enable=warning".to_owned(),
        },
        // Sem isto o cppcheck reprovaria o job por achado; quem decide
        // severidade e a UI, pelo diagnostico — nao o exit code.
        "--error-exitcode=0".to_owned(),
    ];
    args.extend(extra_args.iter().cloned());
    args.push(".".to_owned());
    args
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
    // cargo ao trocar de perfil — aceito, ver docsprivate/diario/18 M4.5).
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
    target: Option<&str>,
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
    // O comando ANUNCIADO nomeia o alvo: o log do job precisa mostrar o que
    // foi realmente pedido, ou o usuario nao tem como conferir a escolha.
    let display = target.map_or_else(
        || "cmake --build".to_owned(),
        |target| {
            build.arg("--target").arg(target);
            format!("cmake --build --target {target}")
        },
    );

    stream_command(build, &display, DiagnosticFormat::GccLike, cancel, sink)
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
    fn cppcheck_args_respeita_perfil_e_aplica_extras_da_config_por_ultimo() {
        use kinein_protocol::RigorProfile;

        let extras = vec![
            "--std=c11".to_owned(),
            "--suppress=missingInclude".to_owned(),
        ];
        let args = super::cppcheck_args(RigorProfile::Strict, &extras);

        // O contrato do funil: template gcc (parser GccLike) e exit code 0
        // (severidade e da UI, nao do processo).
        assert!(args.contains(&"--template=gcc".to_owned()));
        assert!(args.contains(&"--error-exitcode=0".to_owned()));
        assert!(args.contains(&"--enable=warning,style,performance,portability".to_owned()));
        // Extras da integracao (2.2) DEPOIS do perfil, "." por ultimo.
        let pos_extra = args.iter().position(|a| a == "--std=c11").unwrap();
        let pos_perfil = args
            .iter()
            .position(|a| a.starts_with("--enable="))
            .unwrap();
        assert!(pos_extra > pos_perfil);
        assert_eq!(args.last().unwrap(), ".");

        // Perfis mais leves reduzem o --enable.
        let leve = super::cppcheck_args(RigorProfile::Relaxed, &[]);
        assert!(leve.contains(&"--enable=warning".to_owned()));
    }

    #[test]
    fn clang_tidy_args_respeitam_perfil_config_do_projeto_e_extras() {
        use kinein_protocol::RigorProfile;
        use std::path::Path;

        let build_dir = Path::new("/ws/.kinein/build");

        // Sem .clang-tidy no projeto: nos escolhemos os checks, e o piso e o
        // Clang Static Analyzer que o roadmap 28 pede no L2.
        let padrao = super::clang_tidy_args(RigorProfile::Balanced, &[], build_dir, false);
        assert_eq!(padrao[0], "-p");
        assert_eq!(padrao[1], "/ws/.kinein/build");
        let checks = padrao
            .iter()
            .find(|a| a.starts_with("--checks="))
            .expect("sem config do projeto, os checks sao nossos");
        assert!(checks.contains("clang-analyzer-*"));
        assert!(
            checks.starts_with("--checks=-*,"),
            "lista fechada, nao aditiva"
        );

        // COM .clang-tidy no projeto: NAO passamos --checks. Sobrepor a
        // escolha do projeto seria a mesma invasao recusada em --coverage.
        let com_config = super::clang_tidy_args(RigorProfile::Strict, &[], build_dir, true);
        assert!(!com_config.iter().any(|a| a.starts_with("--checks=")));

        // Perfil mais rigoroso amplia os checks.
        let estrito = super::clang_tidy_args(RigorProfile::Strict, &[], build_dir, false);
        let estritos = estrito.iter().find(|a| a.starts_with("--checks=")).unwrap();
        assert!(estritos.contains("bugprone-*"));
        assert!(estritos.contains("performance-*"));
        let leve = super::clang_tidy_args(RigorProfile::Relaxed, &[], build_dir, false);
        let leves = leve.iter().find(|a| a.starts_with("--checks=")).unwrap();
        assert!(!leves.contains("performance-*"));

        // Extras da config entram DEPOIS, entao o usuario sobrepoe.
        let extras = vec!["--warnings-as-errors=".to_owned()];
        let com_extras = super::clang_tidy_args(RigorProfile::Balanced, &extras, build_dir, false);
        let pos_extra = com_extras
            .iter()
            .position(|a| a == "--warnings-as-errors=")
            .unwrap();
        let pos_checks = com_extras
            .iter()
            .position(|a| a.starts_with("--checks="))
            .unwrap();
        assert!(pos_extra > pos_checks);
    }

    #[test]
    fn clang_tidy_sources_filtra_gerado_terceiro_e_duplicata() {
        let base = std::env::temp_dir()
            .join("kinein-core-tests")
            .join(format!("{}-tidy-sources", std::process::id()));
        let build_dir = base.join(".kinein").join("build");
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&build_dir).unwrap();

        let raiz = base.to_str().unwrap();
        let build = build_dir.to_str().unwrap();
        let banco = format!(
            r#"[
              {{"file": "{raiz}/src/main.cpp"}},
              {{"file": "{raiz}/src/main.cpp"}},
              {{"file": "{build}/gerado.cpp"}},
              {{"file": "/fora/do/workspace/terceiro.cpp"}}
            ]"#
        );
        std::fs::write(build_dir.join("compile_commands.json"), banco).unwrap();

        let arquivos = super::clang_tidy_sources(&build_dir, &base);
        assert_eq!(
            arquivos.len(),
            1,
            "duplicata, gerado no build dir e codigo de terceiro nao entram"
        );
        assert!(arquivos[0].ends_with("src/main.cpp"));

        // Sem o banco de comandos, lista VAZIA — o chamador pula o clang-tidy
        // em vez de rodar um analisador que adivinha as flags.
        std::fs::remove_file(build_dir.join("compile_commands.json")).unwrap();
        assert!(super::clang_tidy_sources(&build_dir, &base).is_empty());

        // JSON invalido tambem nao vira analise: silencio, nao chute.
        std::fs::write(build_dir.join("compile_commands.json"), "nao e json").unwrap();
        assert!(super::clang_tidy_sources(&build_dir, &base).is_empty());

        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn run_quality_rejects_kinds_without_linter() {
        use kinein_protocol::{ProjectKind, RigorProfile};

        // Maven tem deteccao de workspace mas nao tem analise de qualidade.
        // (Cmake SAIU daqui em 2026-07-19: ganhou o Cppcheck no L2.)
        let root = std::env::temp_dir();
        let cancel = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let error = super::run_quality(
            &root,
            ProjectKind::Maven,
            RigorProfile::Strict,
            &super::QualityExtras::default(),
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

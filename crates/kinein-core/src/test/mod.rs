//! Test execution with streamed, per-case results.
//!
//! The core runs the project's test tool (`cargo test` for Rust, `ctest` for
//! `CMake`, `pytest` for Python — with the project's own interpreter, fatia 3
//! of the Python chain, 2026-09-13), streams every output line, and parses the
//! individual test outcomes from the tool's normal text output. Nothing here
//! reimplements a test framework: it orchestrates the mature runners and
//! structures their output.

use std::{
    error::Error,
    fmt,
    path::Path,
    process::Command,
    sync::{Arc, atomic::AtomicBool},
    time::Duration,
};

pub mod discover;
mod entries;
pub mod frameworks;
mod launch;
pub mod listing;
mod parse;
mod runners;

pub use discover::discover_tests;

use kinein_protocol::ProjectKind;

use crate::{
    build::project_kind_name,
    process::{self, ProcessError},
    python::run::PythonLauncher,
};

/// Outcome of a single test case.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaseStatus {
    /// The test passed.
    Passed,
    /// The test failed.
    Failed,
    /// The test was ignored / skipped.
    Ignored,
}

impl CaseStatus {
    /// Protocol string used in `event.test.case`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Failed => "failed",
            Self::Ignored => "ignored",
        }
    }
}

/// Event emitted while a test run progresses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TestEvent {
    /// A test command is about to run.
    Started {
        /// Human-readable command line.
        command: String,
    },
    /// One line of raw output.
    Output {
        /// `stdout` or `stderr`.
        stream: &'static str,
        /// Output line without the trailing newline.
        line: String,
    },
    /// One test case finished.
    Case {
        /// Test case name as reported by the runner.
        name: String,
        /// Outcome of the case.
        status: CaseStatus,
    },
}

/// Final tally of a test run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TestOutcome {
    /// Whether the runner exited successfully (no failing test).
    pub success: bool,
    /// Exit code of the runner, when it exited normally.
    pub exit_code: Option<i32>,
    /// Number of passed cases counted from the output.
    pub passed: u64,
    /// Number of failed cases counted from the output.
    pub failed: u64,
    /// Number of ignored cases counted from the output.
    pub ignored: u64,
}

/// Error produced while starting or driving a test run.
#[derive(Debug)]
pub enum TestError {
    /// The workspace project kind has no test integration yet.
    Unsupported {
        /// Project kind name as exposed by the protocol.
        kind: String,
    },
    /// The test tool could not be started.
    Spawn {
        /// Command that failed to start.
        command: String,
        /// Underlying IO error.
        source: std::io::Error,
    },
    /// A ferramenta que o projeto exige nao existe onde deveria (o pytest fora
    /// do ambiente, o interpretador ausente): o erro diz qual e o passo.
    ToolMissing {
        /// Nome da ferramenta (`pytest`, `python`).
        tool: String,
        /// O que fazer, como a fonte oficial escreve.
        hint: String,
    },
    /// IO failure while waiting for the runner.
    Io(std::io::Error),
    /// The `CMake` project has no active configuration (C1-21): the reason
    /// the configure gives.
    NoConfiguration {
        /// The sentence for the person.
        reason: String,
    },
    /// The runner cannot run only what was asked (C1-31): the reason, before
    /// running anything.
    Refused {
        /// The sentence for the person.
        reason: String,
    },
    /// The job was cancelled while it listed (C1-43): no tree is better than
    /// half of one counted as whole.
    ListingCancelled,
}

impl fmt::Display for TestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unsupported { kind } => {
                write!(
                    formatter,
                    "execução de testes ainda não é suportada para projetos do tipo {kind}"
                )
            }
            Self::Spawn { command, source } => {
                write!(formatter, "falha ao iniciar '{command}': {source}")
            }
            Self::ToolMissing { tool, hint } => write!(formatter, "{tool} ausente: {hint}"),
            Self::Io(error) => write!(formatter, "falha de IO durante os testes: {error}"),
            Self::NoConfiguration { reason } | Self::Refused { reason } => {
                formatter.write_str(reason)
            }
            Self::ListingCancelled => formatter.write_str("listagem cancelada"),
        }
    }
}

impl Error for TestError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Spawn { source, .. } | Self::Io(source) => Some(source),
            Self::Unsupported { .. }
            | Self::ToolMissing { .. }
            | Self::NoConfiguration { .. }
            | Self::Refused { .. }
            | Self::ListingCancelled => None,
        }
    }
}

/// The folder ctest reads: the one of the active configuration (C1-21).
pub(crate) fn cmake_test_dir(root: &Path) -> Result<std::path::PathBuf, TestError> {
    crate::cmake::active_dir(root).map_err(|reason| TestError::NoConfiguration { reason })
}

/// `-C` with the configuration clangd reads, in a multi-config build
/// (C1-22): without it `ctest` runs no test ("Not Run") and its JSON lists
/// none.
fn ctest_configuration(command: &mut Command, build_dir: &Path) {
    if let Some(configuration) = crate::cmake::build_configuration::of(build_dir) {
        command.arg("-C").arg(&configuration.name);
    }
}

/// The command shown of a `ctest` that runs, from the same arguments, with
/// the quotes the shell of the system asks: run at the root, it runs the same
/// tests (C1-41).
fn ctest_shown(command: &Command) -> String {
    std::iter::once("ctest".to_owned())
        .chain(
            command
                .get_args()
                .map(|argument| crate::platform::shell_argument(&argument.to_string_lossy())),
        )
        .collect::<Vec<_>>()
        .join(" ")
}

impl TestError {
    /// Returns `true` when the failure is a missing test tool.
    #[must_use]
    pub const fn is_missing_tool(&self) -> bool {
        matches!(self, Self::Spawn { .. } | Self::ToolMissing { .. })
    }
}

/// Runs the test suite for the workspace project kind.
///
/// `filter` narrows the run to matching test names when the runner supports
/// it (cargo's positional filter, ctest's `-R`, pytest's `-k`). `python` e' o
/// lancador do projeto (interpretador ou `uv run`), resolvido por quem chama;
/// so' o Python o usa.
pub fn run_tests(
    root: &Path,
    kind: ProjectKind,
    selection: Selection<'_>,
    python: Option<&PythonLauncher>,
    cancel: &Arc<AtomicBool>,
    sink: &mut dyn FnMut(TestEvent),
) -> Result<TestOutcome, TestError> {
    match kind {
        ProjectKind::RustCargo => runners::run_cargo_test(root, selection, cancel, sink),
        // Um caso de DENTRO de um binario gtest/Catch2 (`teste::caso`, D7)
        // roda pelo proprio binario; o resto e' o ctest de sempre.
        ProjectKind::Cmake => {
            let build_dir = cmake_test_dir(root)?;
            if let Selection::Exact(id) = selection
                && let Some(resultado) =
                    frameworks::run_inner_case(Path::new("ctest"), &build_dir, id, cancel, sink)
            {
                return resultado;
            }
            runners::run_ctest(root, &build_dir, selection, cancel, sink)
        }
        ProjectKind::Python => runners::run_pytest(root, selection, python, cancel, sink),
        other => Err(TestError::Unsupported {
            kind: project_kind_name(other),
        }),
    }
}

/// O que rodar: tudo, um filtro do runner, ou UM teste pelo id exato que o
/// `test.discover` deu (2026-09-13).
#[derive(Debug, Clone, Copy, Default)]
pub enum Selection<'a> {
    /// A suite inteira.
    #[default]
    All,
    /// O filtro na forma do runner (posicional do cargo, `-R` do ctest,
    /// `-k` do pytest).
    Filter(&'a str),
    /// Exatamente um: node id do pytest, nome do libtest com `--exact`, nome
    /// do ctest ancorado.
    Exact(&'a str),
}

impl<'a> Selection<'a> {
    /// A selecao a partir dos dois campos do `test.run`: `testId` vence.
    #[must_use]
    pub fn from_params(filter: Option<&'a str>, test_id: Option<&'a str>) -> Self {
        let limpo = |s: Option<&'a str>| s.map(str::trim).filter(|s| !s.is_empty());
        match (limpo(test_id), limpo(filter)) {
            (Some(id), _) => Self::Exact(id),
            (None, Some(f)) => Self::Filter(f),
            (None, None) => Self::All,
        }
    }
}

pub(super) fn sem_interpretador() -> TestError {
    TestError::ToolMissing {
        tool: "python".to_owned(),
        hint: "sem interpretador para este projeto: crie o ambiente (.venv) pela faixa de \
               saude ou instale o python3"
            .to_owned(),
    }
}

pub(super) fn sem_modulo_pytest() -> TestError {
    TestError::ToolMissing {
        tool: "pytest".to_owned(),
        hint: "instale-o NO ambiente do projeto: `uv add --dev pytest` (projeto do uv) ou \
               `.venv/bin/python -m pip install pytest`"
            .to_owned(),
    }
}

/// Spawns the runner, streams output, and tallies parsed cases.
pub(super) fn stream_command(
    command: Command,
    display_name: &str,
    parse_case: fn(&str) -> Option<(String, CaseStatus)>,
    cancel: &Arc<AtomicBool>,
    sink: &mut dyn FnMut(TestEvent),
) -> Result<TestOutcome, TestError> {
    stream_command_until(command, display_name, parse_case, cancel, None, sink)
        .map(|(outcome, _)| outcome)
}

/// [`stream_command`] stopped at the `timeout` of a test of the ctest; the
/// bool says the deadline stopped it (C1-40).
pub(super) fn stream_command_until(
    command: Command,
    display_name: &str,
    parse_case: fn(&str) -> Option<(String, CaseStatus)>,
    cancel: &Arc<AtomicBool>,
    timeout: Option<Duration>,
    sink: &mut dyn FnMut(TestEvent),
) -> Result<(TestOutcome, bool), TestError> {
    sink(TestEvent::Started {
        command: display_name.to_owned(),
    });

    let mut passed: u64 = 0;
    let mut failed: u64 = 0;
    let mut ignored: u64 = 0;
    let mut on_line = |stream: &'static str, line: String| {
        if let Some((name, status)) = parse_case(&line) {
            match status {
                CaseStatus::Passed => passed += 1,
                CaseStatus::Failed => failed += 1,
                CaseStatus::Ignored => ignored += 1,
            }
            sink(TestEvent::Case { name, status });
        }
        sink(TestEvent::Output { stream, line });
    };

    let run = match timeout {
        Some(timeout) => process::stream_command_lines_with_timeout_cancelable(
            command,
            timeout,
            Arc::clone(cancel),
            &mut on_line,
        ),
        None => process::stream_command_lines_cancelable(command, cancel, &mut on_line)
            .map(|status| (status, false)),
    };
    let (status, expired) = run.map_err(|error| match error {
        ProcessError::Spawn(source) => TestError::Spawn {
            command: display_name.to_owned(),
            source,
        },
        ProcessError::Wait(source) => TestError::Io(source),
    })?;

    Ok((
        TestOutcome {
            success: status.success(),
            exit_code: status.code(),
            passed,
            failed,
            ignored,
        },
        expired,
    ))
}

#[cfg(test)]
mod tests {
    use super::parse::{parse_cargo_case, parse_ctest_case, parse_pytest_case};
    use super::runners::regex_literal;
    use super::{CaseStatus, Selection};
    use super::{TestEvent, stream_command, stream_command_until};

    /// `testId` vence `filter`; vazios e espacos nao contam.
    #[test]
    fn selection_prefers_the_exact_id_and_ignores_blanks() {
        assert!(matches!(Selection::from_params(None, None), Selection::All));
        assert!(matches!(
            Selection::from_params(Some("  "), Some("")),
            Selection::All
        ));
        assert!(matches!(
            Selection::from_params(Some(" soma "), None),
            Selection::Filter("soma")
        ));
        assert!(matches!(
            Selection::from_params(Some("soma"), Some("tests/a.py::x")),
            Selection::Exact("tests/a.py::x")
        ));
    }

    /// O nome exato do ctest vira regex ANCORADA com os metacaracteres
    /// escapados: `Broken.Case` nao pode casar `BrokenXCase`.
    #[test]
    fn ctest_exact_name_is_an_anchored_escaped_regex() {
        assert_eq!(regex_literal("Broken.Case"), "Broken\\.Case");
        assert_eq!(regex_literal("a+b(c)[d]"), "a\\+b\\(c\\)\\[d\\]");
        assert_eq!(regex_literal("CoreParsing"), "CoreParsing");
    }

    fn args_de(command: &std::process::Command) -> Vec<String> {
        command
            .get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect()
    }

    /// Os comandos por selecao, lidos sem rodar nada: o `--exact` do libtest
    /// vai DEPOIS do `--`; o ctest exato e' `-R ^id$` escapado.
    #[test]
    fn the_runner_commands_carry_the_selection() {
        use std::path::Path;
        let root = Path::new("/w");
        assert_eq!(
            args_de(&super::runners::cargo_command(root, Selection::All).0),
            ["test"]
        );
        assert_eq!(
            args_de(&super::runners::cargo_command(root, Selection::Filter("alpha")).0),
            ["test", "alpha"]
        );
        let (c, d) = super::runners::cargo_command(root, Selection::Exact("tests::alpha"));
        assert_eq!(args_de(&c), ["test", "tests::alpha", "--", "--exact"]);
        assert_eq!(d, "cargo test tests::alpha -- --exact");

        let build_dir = crate::cmake::ide_build_dir(root);
        let (c, d) =
            super::runners::ctest_command(root, &build_dir, Selection::Exact("Broken.Case"));
        let build = build_dir.display().to_string();
        assert_eq!(
            args_de(&c),
            [
                "--test-dir",
                build.as_str(),
                "--output-on-failure",
                "-R",
                "^Broken\\.Case$"
            ]
        );
        // C1-41: the command shown is the one that runs, with the quotes;
        // the pattern needs them in the POSIX shell and in the PowerShell.
        assert_eq!(
            d,
            format!(
                "ctest --test-dir {} --output-on-failure -R '^Broken\\.Case$'",
                crate::platform::shell_argument(&build)
            )
        );
        assert_eq!(
            args_de(&super::runners::ctest_command(root, &build_dir, Selection::Filter("Core")).0)
                [3..],
            ["-R", "Core"]
        );
    }

    /// A descoberta do pytest com um interpretador FALSO: so' o stdout conta
    /// (um id que aparece no stderr — o pytest escreve avisos la' — nao e'
    /// teste), e o exit 5 do pytest ("no tests ran") e' lista vazia, nao erro.
    #[test]
    #[cfg_attr(
        windows,
        ignore = "W7: o Python do projeto no Windows (venv com Scripts\\, uv, debugpy) e' da W7 (60 §3.3)"
    )]
    fn pytest_discovery_reads_stdout_only_and_treats_exit_5_as_empty() {
        // Este teste ESCREVE um executavel e o roda; sem o lock do crate ele
        // corre com os outros que fazem o mesmo e o `exec` do filho volta
        // `ETXTBSY` ("Text file busy"), porque outra thread ainda tem o
        // arquivo aberto para escrita no momento do `fork`. Visto acontecer no
        // gate em 2026-09-24. O motivo do lock esta' no `lib.rs`.
        let _serial = crate::serialize_executables();
        let dir = std::env::temp_dir()
            .join("kinein-core-tests")
            .join(format!("{}-pytest-discover", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let fake = dir.join("python");
        crate::write_executable(
            &fake,
            "#!/bin/sh\necho 'tests/a.py::no_stderr' >&2\necho 'tests/a.py::test_x'\necho '1 test collected in 0.00s'\nexit 0\n",
        );
        let launcher = crate::python::run::PythonLauncher::Interpreter(fake.clone());
        let cancel = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let casos = super::discover_tests(
            &dir,
            kinein_protocol::ProjectKind::Python,
            Some(&launcher),
            &cancel,
            &mut |_| {},
        )
        .unwrap();
        assert_eq!(
            casos.iter().map(|c| c.id.as_str()).collect::<Vec<_>>(),
            ["tests/a.py::test_x"]
        );

        // Regravar pelo `write_executable`, nunca abrindo o executavel para
        // escrita AQUI: um `fork` de outro teste herdaria o descritor e o
        // `exec` daria ETXTBSY (falhou assim no gate de 2026-10-08).
        crate::write_executable(&fake, "#!/bin/sh\necho 'no tests ran in 0.01s'\nexit 5\n");
        let casos = super::discover_tests(
            &dir,
            kinein_protocol::ProjectKind::Python,
            Some(&launcher),
            &cancel,
            &mut |_| {},
        )
        .unwrap();
        assert!(casos.is_empty(), "exit 5 = sem testes, nao erro");

        crate::write_executable(&fake, "#!/bin/sh\necho 'erro de coleta' >&2\nexit 2\n");
        let error = super::discover_tests(
            &dir,
            kinein_protocol::ProjectKind::Python,
            Some(&launcher),
            &cancel,
            &mut |_| {},
        )
        .unwrap_err();
        assert!(error.to_string().contains("saiu com"), "{error}");

        // C1-43: the cancel in the middle of the command of the runner says
        // so, and not the signal that stopped it.
        let pid = dir.join("pid");
        crate::write_executable(
            &fake,
            format!(
                "#!/bin/sh\nsleep 30 &\n{}\nwait\n",
                crate::sh_save_background_pid(&pid)
            ),
        );
        let cancel = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let canceller = {
            let (cancel, pid) = (std::sync::Arc::clone(&cancel), pid.clone());
            std::thread::spawn(move || {
                let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
                while std::fs::read_to_string(&pid)
                    .unwrap_or_default()
                    .trim()
                    .is_empty()
                {
                    assert!(std::time::Instant::now() < deadline, "o runner não começou");
                    std::thread::sleep(std::time::Duration::from_millis(20));
                }
                cancel.store(true, std::sync::atomic::Ordering::SeqCst);
            })
        };
        let error = super::discover_tests(
            &dir,
            kinein_protocol::ProjectKind::Python,
            Some(&launcher),
            &cancel,
            &mut |_| {},
        )
        .unwrap_err();
        canceller.join().unwrap();
        crate::kill_process(&std::fs::read_to_string(&pid).unwrap());
        assert!(
            matches!(error, super::TestError::ListingCancelled),
            "{error}"
        );
        assert_eq!(error.to_string(), "listagem cancelada");
    }

    /// A descoberta contra o `ctest` REAL: um projeto minimo de dois testes
    /// (um deles com `.` no nome) configurado pelo cmake real; e rodar UM pelo
    /// id exato roda so' ele. Sem cmake na maquina, o teste nao prova nada e
    /// diz isso.
    #[test]
    fn ctest_discovery_and_exact_run_against_the_real_ctest() {
        use std::process::Command;
        // No Windows, o cmake e o ctest vem do Visual Studio, com o ambiente
        // dele e o Ninja, como no produto (60 §3.3, W6a).
        let tool = |name: &str| {
            let mut command = Command::new(name);
            crate::msvc::apply_environment(&mut command);
            command
        };
        let cmake = tool("cmake")
            .arg("--version")
            .output()
            .is_ok_and(|o| o.status.success());
        let ctest = tool("ctest")
            .arg("--version")
            .output()
            .is_ok_and(|o| o.status.success());
        assert!(
            cmake && ctest || !cfg!(windows),
            "o gate do Windows exige o Visual Studio, que traz o cmake e o ctest"
        );
        if !cmake || !ctest {
            eprintln!("cmake/ctest ausentes: descoberta do ctest NAO provada aqui");
            return;
        }
        let root = std::env::temp_dir()
            .join("kinein-core-tests")
            .join(format!("{}-ctest-discover", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join(".kinein")).unwrap();
        std::fs::write(
            root.join("CMakeLists.txt"),
            "cmake_minimum_required(VERSION 3.24)\nproject(t NONE)\nenable_testing()\n\
             add_test(NAME Core COMMAND ${CMAKE_COMMAND} -E true)\n\
             add_test(NAME CoreParsing COMMAND ${CMAKE_COMMAND} -E true)\n\
             add_test(NAME Broken.Case COMMAND ${CMAKE_COMMAND} -E false)\n",
        )
        .unwrap();
        let mut configure = tool("cmake");
        crate::msvc::prefer_ninja(&mut configure, false);
        let configurado = configure
            .args(["-S", ".", "-B", ".kinein/build"])
            .current_dir(&root)
            .output()
            .unwrap();
        assert!(
            configurado.status.success(),
            "{}",
            String::from_utf8_lossy(&configurado.stderr)
        );
        let cancel = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let mut eventos = Vec::new();
        let casos = super::discover_tests(
            &root,
            kinein_protocol::ProjectKind::Cmake,
            None,
            &cancel,
            &mut |e| eventos.push(e),
        )
        .unwrap();
        let ids: Vec<&str> = casos.iter().map(|c| c.id.as_str()).collect();
        assert_eq!(ids, vec!["Core", "CoreParsing", "Broken.Case"]);
        // C1-41: the command shown is the one that runs at the root.
        let build = root.join(".kinein/build").display().to_string();
        let listing = format!(
            "ctest --test-dir {} -N",
            crate::platform::shell_argument(&build)
        );
        assert!(
            eventos
                .iter()
                .any(|e| matches!(e, TestEvent::Started { command } if *command == listing))
        );

        let ran = |events: &[TestEvent], test: &str| {
            events
                .iter()
                .any(|e| matches!(e, TestEvent::Case { name, .. } if name == test))
        };
        // Rodar so' o Broken.Case: um caso, falhou; o CoreParsing NAO rodou.
        let mut eventos = Vec::new();
        let outcome = super::run_tests(
            &root,
            kinein_protocol::ProjectKind::Cmake,
            Selection::Exact("Broken.Case"),
            None,
            &cancel,
            &mut |e| eventos.push(e),
        )
        .unwrap();
        assert_eq!((outcome.passed, outcome.failed), (0, 1), "{eventos:?}");
        assert!(eventos.iter().any(|e| matches!(e, TestEvent::Case { name, status: CaseStatus::Failed } if name == "Broken.Case")));
        assert!(!ran(&eventos, "CoreParsing"));
        // "Core" exato NAO arrasta "CoreParsing": a regex e' ancorada.
        let mut eventos = Vec::new();
        let outcome = super::run_tests(
            &root,
            kinein_protocol::ProjectKind::Cmake,
            Selection::Exact("Core"),
            None,
            &cancel,
            &mut |e| eventos.push(e),
        )
        .unwrap();
        assert_eq!((outcome.passed, outcome.failed), (1, 0), "{eventos:?}");
        assert!(!ran(&eventos, "CoreParsing"));
    }

    /// As linhas do `pytest -v` (medidas na doc do pytest 9, 2026-09-13):
    /// caso com estado e percentual, parametro com espaco, SKIPPED com motivo,
    /// XFAIL/XPASS/ERROR; e o que NAO e' caso: o resumo curto (estado na
    /// frente), a linha de sessao, o cabecalho do arquivo no modo `-q`.
    #[test]
    fn pytest_verbose_lines_are_parsed_and_summary_is_ignored() {
        assert_eq!(
            parse_pytest_case("tests/test_a.py::test_soma PASSED                 [ 50%]"),
            Some(("tests/test_a.py::test_soma".to_owned(), CaseStatus::Passed))
        );
        assert_eq!(
            parse_pytest_case("tests/test_a.py::test_x[a b] FAILED [100%]"),
            Some((
                "tests/test_a.py::test_x[a b]".to_owned(),
                CaseStatus::Failed
            ))
        );
        assert_eq!(
            parse_pytest_case("tests/test_a.py::test_lento SKIPPED (precisa de rede) [ 33%]"),
            Some((
                "tests/test_a.py::test_lento".to_owned(),
                CaseStatus::Ignored
            ))
        );
        assert_eq!(
            parse_pytest_case("tests/test_a.py::test_bug XFAIL [ 66%]")
                .unwrap()
                .1,
            CaseStatus::Ignored
        );
        assert_eq!(
            parse_pytest_case("tests/test_a.py::test_bug XPASS [ 66%]")
                .unwrap()
                .1,
            CaseStatus::Passed
        );
        assert_eq!(
            parse_pytest_case("tests/test_a.py::test_fixture ERROR [ 10%]")
                .unwrap()
                .1,
            CaseStatus::Failed
        );
        // Parametro que CONTEM uma palavra de estado: o estado e' o ULTIMO.
        assert_eq!(
            parse_pytest_case("tests/t.py::test_p[caso FAILED antes] PASSED [ 10%]"),
            Some((
                "tests/t.py::test_p[caso FAILED antes]".to_owned(),
                CaseStatus::Passed
            ))
        );
        for ruido in [
            "FAILED tests/test_a.py::test_x[a b] - assert 1 == 2",
            "PASSED tests/test_a.py::test_soma",
            // O print de um programa sob teste (com -s) nao e' caso: sem `::`.
            "tudo PASSED",
            "resultado: 3 FAILED [ 50%]",
            "============ 1 failed, 1 passed in 0.02s ============",
            "tests/test_a.py .F                                    [100%]",
            "collected 2 items",
            "PASSED",
            "",
        ] {
            assert!(parse_pytest_case(ruido).is_none(), "nao e' caso: {ruido:?}");
        }
    }

    #[test]
    fn cargo_case_lines_are_parsed_and_summary_is_ignored() {
        assert_eq!(
            parse_cargo_case("test tests::alpha ... ok"),
            Some(("tests::alpha".to_owned(), CaseStatus::Passed))
        );
        assert_eq!(
            parse_cargo_case("test mod::beta ... FAILED"),
            Some(("mod::beta".to_owned(), CaseStatus::Failed))
        );
        assert_eq!(
            parse_cargo_case("test slow ... ignored, precisa de rede"),
            Some(("slow".to_owned(), CaseStatus::Ignored))
        );
        // Doctest names carry spaces and parentheses.
        assert_eq!(
            parse_cargo_case("test src/lib.rs - foo (line 12) ... ok"),
            Some(("src/lib.rs - foo (line 12)".to_owned(), CaseStatus::Passed))
        );
        assert!(parse_cargo_case("test result: ok. 3 passed; 0 failed; 0 ignored;").is_none());
        assert!(parse_cargo_case("running 3 tests").is_none());
    }

    #[test]
    fn ctest_case_lines_are_parsed() {
        assert_eq!(
            parse_ctest_case("1/2 Test #1: CoreParsing ...........   Passed    0.01 sec"),
            Some(("CoreParsing".to_owned(), CaseStatus::Passed))
        );
        assert_eq!(
            parse_ctest_case("2/2 Test #2: BrokenCase ...........***Failed    0.02 sec"),
            Some(("BrokenCase".to_owned(), CaseStatus::Failed))
        );
        // C1-35, as CTest 4.4.5 writes them: the disabled test is not run and
        // is not a failure; the missing executable is.
        assert_eq!(
            parse_ctest_case(
                "1/2 Test #2: Text.Off .........................***Not Run (Disabled)   0.00 sec"
            ),
            Some(("Text.Off".to_owned(), CaseStatus::Ignored))
        );
        assert_eq!(
            parse_ctest_case(
                "2/2 Test #5: kv_missing .......................***Not Run   0.00 sec"
            ),
            Some(("kv_missing".to_owned(), CaseStatus::Failed))
        );
        // C1-36: the number aligned with ten tests or more.
        assert_eq!(
            parse_ctest_case(
                " 1/11 Test  #1: t1 ...............................   Passed    0.01 sec"
            ),
            Some(("t1".to_owned(), CaseStatus::Passed))
        );
        assert_eq!(
            parse_ctest_case(
                "10/11 Test #10: t10 ..............................   Passed    0.01 sec"
            ),
            Some(("t10".to_owned(), CaseStatus::Passed))
        );
        assert!(parse_ctest_case("      Start  1: t1").is_none());
        assert!(parse_ctest_case("Test project /tmp/build").is_none());
        // C1-42: each line of CTest 4.4.5 for a test that crashed, was
        // killed or passed its `TIMEOUT`; and the ones of the outcome by the
        // output and by the return code.
        for (line, name, status) in [
            (
                " 1/14 Test  #1: segv .............................***Exception: SegFault  0.13 sec",
                "segv",
                CaseStatus::Failed,
            ),
            (
                " 2/14 Test  #2: abort ............................Subprocess aborted***Exception:   0.10 sec",
                "abort",
                CaseStatus::Failed,
            ),
            (
                " 3/14 Test  #3: fpe ..............................***Exception: Numerical  0.11 sec",
                "fpe",
                CaseStatus::Failed,
            ),
            (
                " 4/14 Test  #4: ill ..............................***Exception: Illegal  0.11 sec",
                "ill",
                CaseStatus::Failed,
            ),
            (
                " 5/14 Test  #5: term .............................Subprocess terminated***Exception:   0.00 sec",
                "term",
                CaseStatus::Failed,
            ),
            (
                " 6/14 Test  #6: kill .............................Subprocess killed***Exception:   0.00 sec",
                "kill",
                CaseStatus::Failed,
            ),
            (
                " 9/14 Test  #9: sleep ............................***Timeout   1.01 sec",
                "sleep",
                CaseStatus::Failed,
            ),
            (
                "10/14 Test #10: passre ...........................***Failed  Required regular expression not found. Regex=[never",
                "passre",
                CaseStatus::Failed,
            ),
            (
                "11/14 Test #11: failre ...........................***Failed  Error regular expression found in output. Regex=[boom]  0.00 sec",
                "failre",
                CaseStatus::Failed,
            ),
            (
                "13/14 Test #13: skip77 ...........................***Skipped   0.00 sec",
                "skip77",
                CaseStatus::Ignored,
            ),
        ] {
            assert_eq!(
                parse_ctest_case(line),
                Some((name.to_owned(), status)),
                "{line}"
            );
        }
    }

    #[test]
    fn stream_command_tallies_cases_from_output() {
        let command = crate::sh_command(concat!(
            "echo 'running 2 tests'; ",
            "echo 'test a ... ok'; ",
            "echo 'test b ... FAILED'; ",
            "exit 101"
        ));

        let cancel = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let mut events = Vec::new();
        let outcome = stream_command(
            command,
            "sh de teste",
            parse_cargo_case,
            &cancel,
            &mut |event| {
                events.push(event);
            },
        )
        .unwrap();

        assert!(!outcome.success);
        assert_eq!(outcome.passed, 1);
        assert_eq!(outcome.failed, 1);
        assert!(events.iter().any(|event| matches!(
            event,
            TestEvent::Case { name, status: CaseStatus::Failed } if name == "b"
        )));
    }

    /// C1-40: the `TIMEOUT` of a test stops the case and says so; a case that
    /// ends before it is not stopped.
    #[test]
    fn the_timeout_stops_the_command_and_says_it() {
        let cancel = std::sync::Arc::default();
        let started = std::time::Instant::now();
        let (outcome, expired) = stream_command_until(
            crate::sh_command("echo 'test a ... ok'; exec sleep 30"),
            "sh de teste",
            parse_cargo_case,
            &cancel,
            Some(std::time::Duration::from_millis(300)),
            &mut |_| {},
        )
        .unwrap();
        assert!(expired);
        assert!(!outcome.success);
        assert_eq!(outcome.passed, 1);
        assert!(started.elapsed() < std::time::Duration::from_secs(20));
        let (outcome, expired) = stream_command_until(
            crate::sh_command("echo 'test a ... ok'"),
            "sh de teste",
            parse_cargo_case,
            &cancel,
            Some(std::time::Duration::from_secs(60)),
            &mut |_| {},
        )
        .unwrap();
        assert!(!expired);
        assert!(outcome.success);
    }
}

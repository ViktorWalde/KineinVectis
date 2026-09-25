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
};

pub mod discover;
pub mod frameworks;
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
}

impl fmt::Display for TestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unsupported { kind } => {
                write!(
                    formatter,
                    "execucao de testes ainda nao e suportada para projetos do tipo {kind}"
                )
            }
            Self::Spawn { command, source } => {
                write!(formatter, "falha ao iniciar '{command}': {source}")
            }
            Self::ToolMissing { tool, hint } => write!(formatter, "{tool} ausente: {hint}"),
            Self::Io(error) => write!(formatter, "falha de IO durante os testes: {error}"),
        }
    }
}

impl Error for TestError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Spawn { source, .. } | Self::Io(source) => Some(source),
            Self::Unsupported { .. } | Self::ToolMissing { .. } => None,
        }
    }
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
            if let Selection::Exact(id) = selection
                && let Some(resultado) = frameworks::run_inner_case(
                    Path::new("ctest"),
                    &root.join(".kinein/build"),
                    id,
                    cancel,
                    sink,
                )
            {
                return resultado;
            }
            runners::run_ctest(root, selection, cancel, sink)
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

    let status = process::stream_command_lines_cancelable(command, cancel, &mut on_line).map_err(
        |error| match error {
            ProcessError::Spawn(source) => TestError::Spawn {
                command: display_name.to_owned(),
                source,
            },
            ProcessError::Wait(source) => TestError::Io(source),
        },
    )?;

    Ok(TestOutcome {
        success: status.success(),
        exit_code: status.code(),
        passed,
        failed,
        ignored,
    })
}

#[cfg(test)]
mod tests {
    use super::parse::{parse_cargo_case, parse_ctest_case, parse_pytest_case};
    use super::runners::regex_literal;
    use super::{CaseStatus, Selection, TestEvent, stream_command};

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

        let (c, d) = super::runners::ctest_command(root, Selection::Exact("Broken.Case"));
        assert_eq!(
            args_de(&c),
            [
                "--test-dir",
                "/w/.kinein/build",
                "--output-on-failure",
                "-R",
                "^Broken\\.Case$"
            ]
        );
        assert!(d.ends_with("-R ^Broken\\.Case$"), "{d}");
        assert_eq!(
            args_de(&super::runners::ctest_command(root, Selection::Filter("Core")).0)[3..],
            ["-R", "Core"]
        );
    }

    /// A descoberta do pytest com um interpretador FALSO: so' o stdout conta
    /// (um id que aparece no stderr — o pytest escreve avisos la' — nao e'
    /// teste), e o exit 5 do pytest ("no tests ran") e' lista vazia, nao erro.
    #[test]
    #[cfg(unix)]
    fn pytest_discovery_reads_stdout_only_and_treats_exit_5_as_empty() {
        use std::os::unix::fs::PermissionsExt;

        // Este teste ESCREVE um executavel e o roda; sem o lock do crate ele
        // corre com os outros que fazem o mesmo e o `exec` do filho volta
        // `ETXTBSY` ("Text file busy"), porque outra thread ainda tem o
        // arquivo aberto para escrita no momento do `fork`. Visto acontecer no
        // gate em 2026-09-24. O motivo do lock esta' no `lib.rs`.
        let _serial = crate::serializar_executaveis();
        let dir = std::env::temp_dir()
            .join("kinein-core-tests")
            .join(format!("{}-pytest-discover", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let falso = dir.join("python");
        std::fs::write(
            &falso,
            "#!/bin/sh\necho 'tests/a.py::no_stderr' >&2\necho 'tests/a.py::test_x'\necho '1 test collected in 0.00s'\nexit 0\n",
        )
        .unwrap();
        std::fs::set_permissions(&falso, std::fs::Permissions::from_mode(0o755)).unwrap();
        let launcher = crate::python::run::PythonLauncher::Interpreter(falso.clone());
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

        std::fs::write(&falso, "#!/bin/sh\necho 'no tests ran in 0.01s'\nexit 5\n").unwrap();
        let casos = super::discover_tests(
            &dir,
            kinein_protocol::ProjectKind::Python,
            Some(&launcher),
            &cancel,
            &mut |_| {},
        )
        .unwrap();
        assert!(casos.is_empty(), "exit 5 = sem testes, nao erro");

        std::fs::write(&falso, "#!/bin/sh\necho 'erro de coleta' >&2\nexit 2\n").unwrap();
        let erro = super::discover_tests(
            &dir,
            kinein_protocol::ProjectKind::Python,
            Some(&launcher),
            &cancel,
            &mut |_| {},
        )
        .unwrap_err();
        assert!(erro.to_string().contains("saiu com"), "{erro}");
    }

    /// A descoberta contra o `ctest` REAL: um projeto minimo de dois testes
    /// (um deles com `.` no nome) configurado pelo cmake real; e rodar UM pelo
    /// id exato roda so' ele. Sem cmake na maquina, o teste nao prova nada e
    /// diz isso.
    #[test]
    #[cfg(unix)]
    fn ctest_discovery_and_exact_run_against_the_real_ctest() {
        use std::process::Command;
        let cmake = Command::new("cmake")
            .arg("--version")
            .output()
            .is_ok_and(|o| o.status.success());
        let ctest = Command::new("ctest")
            .arg("--version")
            .output()
            .is_ok_and(|o| o.status.success());
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
             add_test(NAME Core COMMAND true)\nadd_test(NAME CoreParsing COMMAND true)\n\
             add_test(NAME Broken.Case COMMAND false)\n",
        )
        .unwrap();
        let configurado = Command::new("cmake")
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
        assert!(
            eventos
                .iter()
                .any(|e| matches!(e, TestEvent::Started { command } if command == "ctest -N"))
        );

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
        assert!(
            !eventos
                .iter()
                .any(|e| matches!(e, TestEvent::Case { name, .. } if name == "CoreParsing"))
        );
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
        assert!(
            !eventos
                .iter()
                .any(|e| matches!(e, TestEvent::Case { name, .. } if name == "CoreParsing"))
        );
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
        assert!(parse_ctest_case("Test project /tmp/build").is_none());
    }

    #[cfg(unix)]
    #[test]
    fn stream_command_tallies_cases_from_output() {
        use std::process::Command;

        let mut command = Command::new("sh");
        command.arg("-c").arg(concat!(
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
}

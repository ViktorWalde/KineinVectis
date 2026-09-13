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
    filter: Option<&str>,
    python: Option<&PythonLauncher>,
    cancel: &Arc<AtomicBool>,
    sink: &mut dyn FnMut(TestEvent),
) -> Result<TestOutcome, TestError> {
    match kind {
        ProjectKind::RustCargo => run_cargo_test(root, filter, cancel, sink),
        ProjectKind::Cmake => run_ctest(root, filter, cancel, sink),
        ProjectKind::Python => run_pytest(root, filter, python, cancel, sink),
        other => Err(TestError::Unsupported {
            kind: project_kind_name(other),
        }),
    }
}

/// `python -m pytest -v` com o Python DO PROJETO: o pytest tem de ser o do
/// ambiente (e' la' que os pacotes do projeto estao). `-v` da' uma linha por
/// caso (`arquivo::caso PASSED [ 50%]`), que `parse_pytest_case` le. Sem o
/// modulo, o pytest nao existe naquele ambiente — e o erro diz como instalar.
fn run_pytest(
    root: &Path,
    filter: Option<&str>,
    python: Option<&PythonLauncher>,
    cancel: &Arc<AtomicBool>,
    sink: &mut dyn FnMut(TestEvent),
) -> Result<TestOutcome, TestError> {
    let Some(launcher) = python else {
        return Err(TestError::ToolMissing {
            tool: "python".to_owned(),
            hint: "sem interpretador para este projeto: crie o ambiente (.venv) pela faixa de \
                   saude ou instale o python3"
                .to_owned(),
        });
    };
    let (program, prefix) = launcher.program();
    let mut command = Command::new(program);
    command
        .args(prefix)
        .args(["-m", "pytest", "-v"])
        .current_dir(root);
    let mut display = format!("{} -m pytest -v", launcher.display(root));
    if let Some(filter) = filter.map(str::trim).filter(|filter| !filter.is_empty()) {
        command.arg("-k").arg(filter);
        display.push_str(" -k ");
        display.push_str(filter);
    }

    let mut sem_pytest = false;
    let mut observando = |event: TestEvent| {
        if let TestEvent::Output { line, .. } = &event {
            if line.contains("No module named pytest") {
                sem_pytest = true;
            }
        }
        sink(event);
    };
    let outcome = stream_command(
        command,
        &display,
        parse_pytest_case,
        cancel,
        &mut observando,
    )?;
    if sem_pytest {
        return Err(TestError::ToolMissing {
            tool: "pytest".to_owned(),
            hint: "instale-o NO ambiente do projeto: `uv add --dev pytest` (projeto do uv) ou \
                   `.venv/bin/python -m pip install pytest`"
                .to_owned(),
        });
    }
    Ok(outcome)
}

fn run_cargo_test(
    root: &Path,
    filter: Option<&str>,
    cancel: &Arc<AtomicBool>,
    sink: &mut dyn FnMut(TestEvent),
) -> Result<TestOutcome, TestError> {
    let mut command = Command::new("cargo");
    command.arg("test").current_dir(root);
    let mut display = String::from("cargo test");
    if let Some(filter) = filter.map(str::trim).filter(|filter| !filter.is_empty()) {
        command.arg(filter);
        display.push(' ');
        display.push_str(filter);
    }

    stream_command(command, &display, parse_cargo_case, cancel, sink)
}

fn run_ctest(
    root: &Path,
    filter: Option<&str>,
    cancel: &Arc<AtomicBool>,
    sink: &mut dyn FnMut(TestEvent),
) -> Result<TestOutcome, TestError> {
    let build_dir = root.join(".kinein").join("build");
    let mut command = Command::new("ctest");
    command
        .arg("--test-dir")
        .arg(&build_dir)
        .arg("--output-on-failure")
        .current_dir(root);
    let mut display = String::from("ctest --output-on-failure");
    if let Some(filter) = filter.map(str::trim).filter(|filter| !filter.is_empty()) {
        command.arg("-R").arg(filter);
        display.push_str(" -R ");
        display.push_str(filter);
    }

    stream_command(command, &display, parse_ctest_case, cancel, sink)
}

/// Spawns the runner, streams output, and tallies parsed cases.
fn stream_command(
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

/// Parses a libtest case line: `test <name> ... <outcome>`.
///
/// The summary line `test result: ...` is intentionally not matched: it lacks
/// the ` ... ` separator that every case line carries.
fn parse_cargo_case(line: &str) -> Option<(String, CaseStatus)> {
    let rest = line.trim().strip_prefix("test ")?;
    let separator = rest.rfind(" ... ")?;
    let name = rest[..separator].trim();
    if name.is_empty() {
        return None;
    }
    let outcome = rest[separator + " ... ".len()..].trim();
    let status = match outcome {
        "ok" => CaseStatus::Passed,
        "FAILED" => CaseStatus::Failed,
        // "ignored" or "ignored, <reason>".
        _ if outcome == "ignored" || outcome.starts_with("ignored,") => CaseStatus::Ignored,
        // Benchmarks and anything else are not pass/fail cases.
        _ => return None,
    };
    Some((name.to_owned(), status))
}

/// Parses a pytest `-v` case line: `tests/test_x.py::test_a PASSED [ 50%]`.
///
/// O nome vem ANTES do estado e sempre tem `::` (`arquivo::caso`, com
/// parametros entre colchetes, que podem ter espaco); o estado e' a ULTIMA
/// palavra-chave da linha antes do `[ nn%]`. Linhas do resumo (`FAILED
/// tests/x.py::a - assert`) comecam pelo estado e nao casam — senao cada
/// falha contaria duas vezes.
fn parse_pytest_case(line: &str) -> Option<(String, CaseStatus)> {
    const ESTADOS: [(&str, CaseStatus); 6] = [
        ("PASSED", CaseStatus::Passed),
        ("XPASS", CaseStatus::Passed),
        ("FAILED", CaseStatus::Failed),
        ("ERROR", CaseStatus::Failed),
        ("SKIPPED", CaseStatus::Ignored),
        ("XFAIL", CaseStatus::Ignored),
    ];
    let line = line.trim();
    let mut melhor: Option<(usize, CaseStatus)> = None;
    for (palavra, status) in ESTADOS {
        let marcador = format!(" {palavra}");
        if let Some(pos) = line.rfind(&marcador) {
            if melhor.is_none_or(|(p, _)| pos > p) {
                melhor = Some((pos, status));
            }
        }
    }
    let (pos, status) = melhor?;
    let name = line[..pos].trim();
    if name.is_empty() || !name.contains("::") {
        return None;
    }
    Some((name.to_owned(), status))
}

/// Parses a ctest case line: `1/3 Test #1: Name .... Passed|***Failed`.
fn parse_ctest_case(line: &str) -> Option<(String, CaseStatus)> {
    let marker = line.find("Test #")?;
    let after_hash = line[marker + "Test #".len()..].split_once(':')?.1;
    // `after_hash` is like `  Name .................   Passed    0.01 sec`.
    let status = if line.contains("***Failed") || line.contains("***Not Run") {
        CaseStatus::Failed
    } else if line.contains("***Skipped") || line.contains("***Disabled") {
        CaseStatus::Ignored
    } else if line.contains("   Passed") {
        CaseStatus::Passed
    } else {
        return None;
    };
    let name = after_hash
        .trim_start()
        .split(" ..")
        .next()?
        .split("   ")
        .next()?
        .trim();
    if name.is_empty() {
        return None;
    }
    Some((name.to_owned(), status))
}

#[cfg(test)]
mod tests {
    use super::{
        CaseStatus, TestEvent, parse_cargo_case, parse_ctest_case, parse_pytest_case,
        stream_command,
    };

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

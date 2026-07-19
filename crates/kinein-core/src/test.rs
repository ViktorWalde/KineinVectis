//! Test execution with streamed, per-case results.
//!
//! The core runs the project's test tool (`cargo test` for Rust, `ctest` for
//! `CMake`), streams every output line, and parses the individual test outcomes
//! from the tool's normal text output. Nothing here reimplements a test
//! framework: it orchestrates the mature runners and structures their output.

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
            Self::Io(error) => write!(formatter, "falha de IO durante os testes: {error}"),
        }
    }
}

impl Error for TestError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Spawn { source, .. } | Self::Io(source) => Some(source),
            Self::Unsupported { .. } => None,
        }
    }
}

impl TestError {
    /// Returns `true` when the failure is a missing test tool.
    #[must_use]
    pub const fn is_missing_tool(&self) -> bool {
        matches!(self, Self::Spawn { .. })
    }
}

/// Runs the test suite for the workspace project kind.
///
/// `filter` narrows the run to matching test names when the runner supports
/// it (cargo's positional filter, ctest's `-R`).
pub fn run_tests(
    root: &Path,
    kind: ProjectKind,
    filter: Option<&str>,
    cancel: &Arc<AtomicBool>,
    sink: &mut dyn FnMut(TestEvent),
) -> Result<TestOutcome, TestError> {
    match kind {
        ProjectKind::RustCargo => run_cargo_test(root, filter, cancel, sink),
        ProjectKind::Cmake => run_ctest(root, filter, cancel, sink),
        other => Err(TestError::Unsupported {
            kind: project_kind_name(other),
        }),
    }
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

    stream_command(command, &display, parse_cmake_case, cancel, sink)
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

/// Chain parser do caminho `CMake` (L2 fatia 2, 2026-07-19): ctest primeiro
/// (autoritativo por executavel), depois os frameworks C/C++ cujo output
/// atravessa o `--output-on-failure` — `GTest` e `Unity`. Com isso um executavel
/// que FALHA lista os casos internos nomeados no painel, nao so o binario.
/// Assimetria documentada: executavel que passa conta 1 caso (ctest); o que
/// falha soma os casos internos que o framework imprimir. Criterion ficou de
/// fora de proposito: formato por-caso nao confirmado na fonte — entra
/// quando for medido, nao de memoria.
fn parse_cmake_case(line: &str) -> Option<(String, CaseStatus)> {
    parse_ctest_case(line)
        .or_else(|| parse_gtest_case(line))
        .or_else(|| parse_unity_case(line))
}

/// `GTest`: `[       OK ] Suite.Case (0 ms)` / `[  FAILED  ] Suite.Case` /
/// `[  SKIPPED ] Suite.Case`. O sufixo de duracao e descartado.
fn parse_gtest_case(line: &str) -> Option<(String, CaseStatus)> {
    let trimmed = line.trim();
    let (status, rest) = if let Some(rest) = trimmed.strip_prefix("[       OK ]") {
        (CaseStatus::Passed, rest)
    } else if let Some(rest) = trimmed.strip_prefix("[  FAILED  ]") {
        (CaseStatus::Failed, rest)
    } else if let Some(rest) = trimmed.strip_prefix("[  SKIPPED ]") {
        (CaseStatus::Ignored, rest)
    } else {
        return None;
    };
    let name = rest.trim().split(" (").next()?.trim();
    // Linhas de resumo ("N tests, listed below") nao tem Suite.Case.
    if name.is_empty() || !name.contains('.') || name.contains(' ') {
        return None;
    }
    Some((name.to_owned(), status))
}

/// `Unity`: `arquivo.c:12:nome_do_teste:PASS|FAIL[: mensagem]|IGNORE`.
fn parse_unity_case(line: &str) -> Option<(String, CaseStatus)> {
    let mut parts = line.trim().splitn(4, ':');
    let file = parts.next()?;
    let line_number = parts.next()?;
    let name = parts.next()?.trim();
    let outcome = parts.next()?.trim();
    // O 2o campo TEM que ser numero de linha, senao qualquer frase com
    // dois-pontos viraria caso de teste.
    if !file.contains('.') || line_number.parse::<u64>().is_err() || name.is_empty() {
        return None;
    }
    let status = if outcome == "PASS" || outcome.starts_with("PASS:") {
        CaseStatus::Passed
    } else if outcome == "FAIL" || outcome.starts_with("FAIL:") {
        CaseStatus::Failed
    } else if outcome == "IGNORE" || outcome.starts_with("IGNORE:") {
        CaseStatus::Ignored
    } else {
        return None;
    };
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
        CaseStatus, TestEvent, parse_cargo_case, parse_cmake_case, parse_ctest_case,
        parse_gtest_case, parse_unity_case, stream_command,
    };

    #[test]
    fn parse_gtest_case_cobre_ok_failed_skipped_e_recusa_resumo() {
        assert_eq!(
            parse_gtest_case("[       OK ] SuiteA.CasoBom (12 ms)"),
            Some(("SuiteA.CasoBom".to_owned(), CaseStatus::Passed))
        );
        assert_eq!(
            parse_gtest_case("[  FAILED  ] SuiteA.CasoRuim (3 ms)"),
            Some(("SuiteA.CasoRuim".to_owned(), CaseStatus::Failed))
        );
        assert_eq!(
            parse_gtest_case("[  SKIPPED ] SuiteA.CasoPulado"),
            Some(("SuiteA.CasoPulado".to_owned(), CaseStatus::Ignored))
        );
        // Linhas de resumo e moldura do GTest NAO viram caso.
        assert_eq!(
            parse_gtest_case("[  FAILED  ] 2 tests, listed below:"),
            None
        );
        assert_eq!(
            parse_gtest_case("[==========] 5 tests ran. (20 ms total)"),
            None
        );
        assert_eq!(parse_gtest_case("[ RUN      ] SuiteA.CasoBom"), None);
    }

    #[test]
    fn parse_unity_case_exige_arquivo_linha_e_recusa_frase_com_dois_pontos() {
        assert_eq!(
            parse_unity_case("test_led.c:21:test_liga_led:PASS"),
            Some(("test_liga_led".to_owned(), CaseStatus::Passed))
        );
        assert_eq!(
            parse_unity_case("src/test_io.c:7:test_le_pino:FAIL: Expected 1 Was 0"),
            Some(("test_le_pino".to_owned(), CaseStatus::Failed))
        );
        assert_eq!(
            parse_unity_case("test_led.c:30:test_futuro:IGNORE"),
            Some(("test_futuro".to_owned(), CaseStatus::Ignored))
        );
        // Frase qualquer com dois-pontos: o 2o campo nao e numero de linha.
        assert_eq!(parse_unity_case("nota: veja o log: erro: FAIL"), None);
        assert_eq!(parse_unity_case("Makefile:12: warning: PASS"), None);
        // ISOLA a guarda do numero de linha: arquivo com ponto, campo 2 nao
        // numerico — sem esta fixture a mutacao "guarda removida" sobrevivia
        // (2026-07-19), porque os casos acima ja caiam pela guarda do ponto.
        assert_eq!(parse_unity_case("log.txt:aviso:contexto:FAIL"), None);
    }

    #[test]
    fn parse_cmake_case_prioriza_ctest_e_encadeia_frameworks() {
        // Linha do ctest continua autoritativa...
        assert_eq!(
            parse_cmake_case("1/3 Test #1: unidade ............   Passed    0.01 sec"),
            Some(("unidade".to_owned(), CaseStatus::Passed))
        );
        // ...e os casos internos (que so aparecem no output de executavel
        // FALHO com --output-on-failure) ganham nome no painel.
        assert_eq!(
            parse_cmake_case("[  FAILED  ] SuiteA.CasoRuim (3 ms)"),
            Some(("SuiteA.CasoRuim".to_owned(), CaseStatus::Failed))
        );
        assert_eq!(
            parse_cmake_case("test_led.c:21:test_liga_led:FAIL"),
            Some(("test_liga_led".to_owned(), CaseStatus::Failed))
        );
        assert_eq!(parse_cmake_case("saida qualquer do build"), None);
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

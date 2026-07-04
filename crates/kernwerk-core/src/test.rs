//! Test execution with streamed, per-case results.
//!
//! The core runs the project's test tool (`cargo test` for Rust, `ctest` for
//! `CMake`), streams every output line, and parses the individual test outcomes
//! from the tool's normal text output. Nothing here reimplements a test
//! framework: it orchestrates the mature runners and structures their output.

use std::{error::Error, fmt, path::Path, process::Command};

use kernwerk_protocol::ProjectKind;

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
    sink: &mut dyn FnMut(TestEvent),
) -> Result<TestOutcome, TestError> {
    match kind {
        ProjectKind::RustCargo => run_cargo_test(root, filter, sink),
        ProjectKind::Cmake => run_ctest(root, filter, sink),
        other => Err(TestError::Unsupported {
            kind: project_kind_name(other),
        }),
    }
}

fn run_cargo_test(
    root: &Path,
    filter: Option<&str>,
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

    stream_command(command, &display, parse_cargo_case, sink)
}

fn run_ctest(
    root: &Path,
    filter: Option<&str>,
    sink: &mut dyn FnMut(TestEvent),
) -> Result<TestOutcome, TestError> {
    let build_dir = root.join(".kernwerk").join("build");
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

    stream_command(command, &display, parse_ctest_case, sink)
}

/// Spawns the runner, streams output, and tallies parsed cases.
fn stream_command(
    command: Command,
    display_name: &str,
    parse_case: fn(&str) -> Option<(String, CaseStatus)>,
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

    let status =
        process::stream_command_lines(command, &mut on_line).map_err(|error| match error {
            ProcessError::Spawn(source) => TestError::Spawn {
                command: display_name.to_owned(),
                source,
            },
            ProcessError::Wait(source) => TestError::Io(source),
        })?;

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
    use super::{CaseStatus, TestEvent, parse_cargo_case, parse_ctest_case, stream_command};

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

        let mut events = Vec::new();
        let outcome = stream_command(command, "sh de teste", parse_cargo_case, &mut |event| {
            events.push(event);
        })
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

//! DESCOBRIR os testes antes de rodar.
//!
//! A arvore que o `JetBrains` mostra antes do primeiro run (41 B6, o que
//! faltou; 2026-09-13): cada runner tem um modo de listar, e cada linha vira
//! um `TestCaseInfo` com o ID exato que o mesmo runner aceita para rodar UM
//! teste.
//!
//! ```text
//! pytest   python -m pytest --collect-only -q     tests/test_a.py::test_x[a b]
//!          (medido no pytest 9.1.1: um node id por linha, linhas vazias e o
//!          resumo "N tests collected in 0.00s" no fim; avisos de <frozen
//!          site> vao para o stderr)            -> roda: pytest <node id>
//! cargo    cargo test -- --list                  tests::alpha: test
//!          (libtest: `<nome>: test` por caso, `<nome>: benchmark` para
//!          benches, e o resumo "N tests, M benchmarks"; COMPILA os testes,
//!          por isso e' JOB)                    -> roda: cargo test <nome> -- --exact
//! ctest    ctest --test-dir <build> -N           Test #1: CoreParsing
//!          (medido no ctest 4.x: "  Test #N: Nome" e "Total Tests: N")
//!                                               -> roda: ctest -R ^Nome$
//! ```

use std::path::Path;
use std::process::Command;
use std::sync::{Arc, atomic::AtomicBool};

use kinein_protocol::{ProjectKind, TestCaseInfo};

use crate::build::project_kind_name;
use crate::process::{self, ProcessError};
use crate::python::run::PythonLauncher;

use super::{TestError, TestEvent, sem_interpretador, sem_modulo_pytest};

/// Que runner lista os testes deste tipo de projeto, e com que comando.
#[must_use]
pub const fn runner_name(kind: ProjectKind) -> Option<&'static str> {
    match kind {
        ProjectKind::RustCargo => Some("cargo"),
        ProjectKind::Cmake => Some("ctest"),
        ProjectKind::Python => Some("pytest"),
        ProjectKind::Maven
        | ProjectKind::Gradle
        | ProjectKind::Make
        | ProjectKind::PlatformIo
        | ProjectKind::Unknown => None,
    }
}

/// Uma linha do `pytest --collect-only -q`: o node id, ou nada.
#[must_use]
pub fn parse_pytest_collect_line(line: &str) -> Option<TestCaseInfo> {
    let l = line.trim();
    // O node id tem `::`; o resumo ("6 tests collected in 0.00s"), avisos e
    // linhas vazias nao.
    if l.is_empty() || !l.contains("::") {
        return None;
    }
    let (file, resto) = l.split_once("::")?;
    if file.is_empty()
        || !std::path::Path::new(file)
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("py"))
            && !file.contains('/')
    {
        return None;
    }
    Some(TestCaseInfo {
        id: l.to_owned(),
        name: resto.to_owned(),
        file: Some(file.to_owned()),
    })
}

/// Uma linha do `cargo test -- --list`: `<nome>: test`, ou nada (benchmarks,
/// resumo, "running N tests" nao sao casos).
#[must_use]
pub fn parse_cargo_list_line(line: &str) -> Option<TestCaseInfo> {
    let l = line.trim();
    let nome = l.strip_suffix(": test")?.trim();
    if nome.is_empty() || nome.contains(' ') {
        return None;
    }
    Some(TestCaseInfo {
        id: nome.to_owned(),
        name: nome.to_owned(),
        file: None,
    })
}

/// Uma linha do `ctest -N`: `  Test #N: Nome`, ou nada.
#[must_use]
pub fn parse_ctest_list_line(line: &str) -> Option<TestCaseInfo> {
    let l = line.trim();
    let resto = l.strip_prefix("Test #")?;
    let (_numero, nome) = resto.split_once(':')?;
    let nome = nome.trim();
    if nome.is_empty() {
        return None;
    }
    Some(TestCaseInfo {
        id: nome.to_owned(),
        name: nome.to_owned(),
        file: None,
    })
}

/// O parser de cada runner.
#[must_use]
pub fn parser_for(kind: ProjectKind) -> Option<fn(&str) -> Option<TestCaseInfo>> {
    match kind {
        ProjectKind::RustCargo => Some(parse_cargo_list_line),
        ProjectKind::Cmake => Some(parse_ctest_list_line),
        ProjectKind::Python => Some(parse_pytest_collect_line),
        ProjectKind::Maven
        | ProjectKind::Gradle
        | ProjectKind::Make
        | ProjectKind::PlatformIo
        | ProjectKind::Unknown => None,
    }
}

/// LISTA os testes sem rodar.
///
/// O comando de descoberta do runner, uma linha por caso pelo parser do
/// `discover`. `Err` quando o tipo nao tem runner, a ferramenta falta, ou o
/// comando sai com erro — e cargo COMPILA os testes para listar, por isso
/// quem chama roda isto como job.
pub fn discover_tests(
    root: &Path,
    kind: ProjectKind,
    python: Option<&PythonLauncher>,
    cancel: &Arc<AtomicBool>,
    sink: &mut dyn FnMut(TestEvent),
) -> Result<Vec<TestCaseInfo>, TestError> {
    let Some(parse) = parser_for(kind) else {
        return Err(TestError::Unsupported {
            kind: project_kind_name(kind),
        });
    };
    let (command, display) = match kind {
        ProjectKind::RustCargo => {
            let mut c = Command::new("cargo");
            c.args(["test", "--", "--list"]).current_dir(root);
            (c, "cargo test -- --list".to_owned())
        }
        ProjectKind::Cmake => {
            let build_dir = root.join(".kinein").join("build");
            let mut c = Command::new("ctest");
            c.arg("--test-dir")
                .arg(&build_dir)
                .arg("-N")
                .current_dir(root);
            (c, "ctest -N".to_owned())
        }
        ProjectKind::Python => {
            let Some(launcher) = python else {
                return Err(sem_interpretador());
            };
            let (program, prefix) = launcher.program();
            let mut c = Command::new(program);
            c.args(prefix)
                .args(["-m", "pytest", "--collect-only", "-q"])
                .current_dir(root);
            (
                c,
                format!("{} -m pytest --collect-only -q", launcher.display(root)),
            )
        }
        ProjectKind::Maven
        | ProjectKind::Gradle
        | ProjectKind::Make
        | ProjectKind::PlatformIo
        | ProjectKind::Unknown => {
            return Err(TestError::Unsupported {
                kind: project_kind_name(kind),
            });
        }
    };
    sink(TestEvent::Started {
        command: display.clone(),
    });
    let mut casos = Vec::new();
    let mut sem_pytest = false;
    let mut on_line = |stream: &'static str, line: String| {
        if line.contains("No module named pytest") {
            sem_pytest = true;
        }
        if stream == "stdout" {
            if let Some(caso) = parse(&line) {
                casos.push(caso);
            }
        }
        sink(TestEvent::Output { stream, line });
    };
    let status = process::stream_command_lines_cancelable(command, cancel, &mut on_line).map_err(
        |error| match error {
            ProcessError::Spawn(source) => TestError::Spawn {
                command: display.clone(),
                source,
            },
            ProcessError::Wait(source) => TestError::Io(source),
        },
    )?;
    if sem_pytest {
        return Err(sem_modulo_pytest());
    }
    // pytest sai com 5 quando NAO HA' testes ("no tests ran"): lista vazia,
    // nao erro — um projeto novo nao tem testes ainda.
    let vazio_do_pytest = kind == ProjectKind::Python && status.code() == Some(5);
    if !status.success() && !vazio_do_pytest {
        return Err(TestError::Io(std::io::Error::other(format!(
            "`{display}` saiu com {status}"
        ))));
    }
    // CMake (D7 do roadmaps/41, 2026-09-17): os casos de DENTRO de cada
    // binario gtest/Catch2, perguntados ao proprio binario, entram depois
    // das linhas do ctest — com o id `teste::caso`.
    if kind == ProjectKind::Cmake {
        let internos = super::frameworks::discover_inner_cases(
            Path::new("ctest"),
            &root.join(".kinein").join("build"),
            cancel,
        );
        if !internos.is_empty() {
            sink(TestEvent::Output {
                stream: "stdout",
                line: format!(
                    "{} caso(s) de gtest/Catch2 dentro dos binarios do ctest",
                    internos.len()
                ),
            });
            casos.extend(internos);
        }
    }
    Ok(casos)
}

#[cfg(test)]
mod tests {
    use super::{parse_cargo_list_line, parse_ctest_list_line, parse_pytest_collect_line};

    /// As linhas medidas em 2026-09-13 (pytest 9.1.1, libtest, ctest 4.x) e o
    /// ruido de cada um: resumo, cabecalho, benchmark, aviso.
    #[test]
    fn each_runner_listing_becomes_ids_the_same_runner_accepts() {
        let p = parse_pytest_collect_line("tests/test_a.py::test_param[a b]").unwrap();
        assert_eq!(
            (p.id.as_str(), p.name.as_str(), p.file.as_deref()),
            (
                "tests/test_a.py::test_param[a b]",
                "test_param[a b]",
                Some("tests/test_a.py")
            )
        );
        assert_eq!(
            parse_pytest_collect_line("tests/test_a.py::TestX::test_m")
                .unwrap()
                .name,
            "TestX::test_m"
        );
        for ruido in [
            "",
            "6 tests collected in 0.00s",
            "no tests ran in 0.01s",
            "<frozen site>:101: RuntimeWarning: x",
            "ERROR tests/test_b.py",
        ] {
            assert!(parse_pytest_collect_line(ruido).is_none(), "{ruido:?}");
        }

        let c = parse_cargo_list_line("tests::alpha: test").unwrap();
        assert_eq!((c.id.as_str(), c.file), ("tests::alpha", None));
        for ruido in [
            "",
            "bench::x: benchmark",
            "3 tests, 0 benchmarks",
            "running 3 tests",
            "test result: ok",
        ] {
            assert!(parse_cargo_list_line(ruido).is_none(), "{ruido:?}");
        }

        let t = parse_ctest_list_line("  Test #2: Broken.Case").unwrap();
        assert_eq!(t.id, "Broken.Case");
        for ruido in ["Test project /tmp/b", "Total Tests: 2", "", "  Test #3:"] {
            assert!(parse_ctest_list_line(ruido).is_none(), "{ruido:?}");
        }
    }
}

//! Os RUNNERS: como cada ferramenta de teste e' invocada para rodar tudo, um
//! filtro, ou UM teste pelo id exato (`Selection`), e o comando de cada uma
//! como funcao pura — para o teste ler os argumentos sem compilar nada.
//! Saiu do `mod.rs` em 2026-09-13, quando a descoberta e a selecao exata o
//! levaram a 585 linhas: rodar e' outra responsabilidade que orquestrar.

use std::path::Path;
use std::process::Command;
use std::sync::{Arc, atomic::AtomicBool};

use crate::python::run::PythonLauncher;

use super::parse::{parse_cargo_case, parse_ctest_case, parse_pytest_case};
use super::{
    Selection, TestError, TestEvent, TestOutcome, sem_interpretador, sem_modulo_pytest,
    stream_command,
};

pub(super) fn run_cargo_test(
    root: &Path,
    selection: Selection<'_>,
    cancel: &Arc<AtomicBool>,
    sink: &mut dyn FnMut(TestEvent),
) -> Result<TestOutcome, TestError> {
    let (command, display) = cargo_command(root, selection);
    stream_command(command, &display, parse_cargo_case, cancel, sink)
}

/// `cargo test [filtro | <id> -- --exact]`: funcao pura, para o teste ler os
/// argumentos sem compilar nada.
pub(super) fn cargo_command(root: &Path, selection: Selection<'_>) -> (Command, String) {
    let mut command = Command::new("cargo");
    command.arg("test").current_dir(root);
    let mut display = String::from("cargo test");
    match selection {
        Selection::All => {}
        Selection::Filter(filter) => {
            command.arg(filter);
            display.push(' ');
            display.push_str(filter);
        }
        // `--exact` depois do `--`: e' do libtest, nao do cargo.
        Selection::Exact(id) => {
            command.args([id, "--", "--exact"]);
            display.push(' ');
            display.push_str(id);
            display.push_str(" -- --exact");
        }
    }
    (command, display)
}

pub(super) fn run_ctest(
    root: &Path,
    selection: Selection<'_>,
    cancel: &Arc<AtomicBool>,
    sink: &mut dyn FnMut(TestEvent),
) -> Result<TestOutcome, TestError> {
    let (command, display) = ctest_command(root, selection);
    stream_command(command, &display, parse_ctest_case, cancel, sink)
}

/// `ctest --test-dir .kinein/build --output-on-failure [-R filtro | -R ^id$]`.
pub(super) fn ctest_command(root: &Path, selection: Selection<'_>) -> (Command, String) {
    let build_dir = root.join(".kinein").join("build");
    let mut command = Command::new("ctest");
    command
        .arg("--test-dir")
        .arg(&build_dir)
        .arg("--output-on-failure")
        .current_dir(root);
    let mut display = String::from("ctest --output-on-failure");
    match selection {
        Selection::All => {}
        Selection::Filter(filter) => {
            command.arg("-R").arg(filter);
            display.push_str(" -R ");
            display.push_str(filter);
        }
        // Um nome exato e' uma regex ancorada com os metacaracteres escapados.
        Selection::Exact(id) => {
            let ancorado = format!("^{}$", regex_literal(id));
            command.arg("-R").arg(&ancorado);
            display.push_str(" -R ");
            display.push_str(&ancorado);
        }
    }
    (command, display)
}

/// Escapa o que e' metacaractere na regex do ctest (`.` de `Broken.Case`).
pub(super) fn regex_literal(texto: &str) -> String {
    let mut saida = String::with_capacity(texto.len() + 4);
    for c in texto.chars() {
        if r"\.^$|()[]{}*+?".contains(c) {
            saida.push('\\');
        }
        saida.push(c);
    }
    saida
}

/// `python -m pytest -v` com o Python DO PROJETO: o pytest tem de ser o do
/// ambiente (e' la' que os pacotes do projeto estao). `-v` da' uma linha por
/// caso (`arquivo::caso PASSED [ 50%]`), que `parse_pytest_case` le. Sem o
/// modulo, o pytest nao existe naquele ambiente — e o erro diz como instalar.
pub(super) fn run_pytest(
    root: &Path,
    selection: Selection<'_>,
    python: Option<&PythonLauncher>,
    cancel: &Arc<AtomicBool>,
    sink: &mut dyn FnMut(TestEvent),
) -> Result<TestOutcome, TestError> {
    let Some(launcher) = python else {
        return Err(sem_interpretador());
    };
    let (program, prefix) = launcher.program();
    let mut command = Command::new(program);
    command
        .args(prefix)
        .args(["-m", "pytest", "-v"])
        .current_dir(root);
    let mut display = format!("{} -m pytest -v", launcher.display(root));
    match selection {
        Selection::All => {}
        Selection::Filter(filter) => {
            command.arg("-k").arg(filter);
            display.push_str(" -k ");
            display.push_str(filter);
        }
        // O node id e' posicional (`pytest tests/x.py::caso`), nao `-k`.
        Selection::Exact(id) => {
            command.arg(id);
            display.push(' ');
            display.push_str(id);
        }
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
        return Err(sem_modulo_pytest());
    }
    Ok(outcome)
}

//! O perfil de rigor (M4.5) num projeto C/C++ de verdade (2026-10-03).
//!
//! O defeito que o autor achou: abrir um projeto, escolher "Estrito", depois
//! "Equilibrado" ou "Relaxado" — e nada mudava no build C/C++. O perfil nunca
//! chegava ao `CMake`, e o build so' configurava quando faltava o cache. Aqui
//! roda o `cmake` real num projeto com `#warning` (aviso em todo compilador,
//! sem depender de -Wall), trocando o perfil entre builds como o usuario faz.
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use kinein_protocol::{ProjectKind, RigorProfile};

use crate::build::{BuildEvent, BuildTools, run_build};
use crate::toolchain::Toolchain;

/// A ferramenta existe, procurada como o build a procura: no Windows, no
/// ambiente do Visual Studio (60 §3.3, W6a).
fn tool_exists(name: &str) -> bool {
    let mut command = std::process::Command::new(name);
    crate::msvc::apply_environment(&mut command);
    command.arg("--version").output().is_ok()
}

/// O compilador de C da maquina: o `cl` do Visual Studio no Windows.
const C_COMPILER: &str = if cfg!(windows) { "cl" } else { "cc" };

/// Sem `cmake` ou compilador, o teste nao prova nada. No Windows o gate exige
/// o Visual Studio, que traz os dois: la' faltar e' falha, e nao pulo.
fn tools_present() -> bool {
    let present = tool_exists("cmake") && tool_exists(C_COMPILER);
    assert!(
        present || !cfg!(windows),
        "o gate do Windows exige o Visual Studio, que traz o cmake e o cl"
    );
    present
}

fn project(name: &str, project_wants_werror: bool) -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!("kinein-rigor-{name}-{}", std::process::id()));
    drop(std::fs::remove_dir_all(&root));
    std::fs::create_dir_all(&root).unwrap();
    let property = if project_wants_werror {
        "set_target_properties(app PROPERTIES COMPILE_WARNING_AS_ERROR ON)\n"
    } else {
        ""
    };
    std::fs::write(
        root.join("CMakeLists.txt"),
        format!(
            "cmake_minimum_required(VERSION 3.24)\nproject(rigor C)\nadd_executable(app main.c)\n{property}"
        ),
    )
    .unwrap();
    // Um deslocamento maior que o tipo: aviso nos tres compiladores no nivel
    // PADRAO, sem `-Wall` (`-Wshift-count-overflow` no GCC e no Clang, C4293
    // no MSVC). Medido em 2026-10-10: com o `cmake_minimum_required` 3.24 o
    // CMake nao poe mais o `/W3` (CMP0092), o `cl` fica no `/W1`, e o
    // `#warning` de antes nao e' aviso no C do MSVC (60 §3.3, W6b2).
    std::fs::write(
        root.join("main.c"),
        "int main(void) { return (1 << 40) != 0; }\n",
    )
    .unwrap();
    root
}

fn build(root: &Path, profile: RigorProfile) -> (bool, Vec<String>) {
    let toolchain = Toolchain::resolve(root, &[]);
    let cancel = Arc::new(AtomicBool::new(false));
    let mut lines = Vec::new();
    let mut sink = |event: BuildEvent| {
        if let BuildEvent::Output { line, .. } = event {
            lines.push(line);
        }
    };
    let outcome = run_build(
        root,
        ProjectKind::Cmake,
        profile,
        &toolchain,
        &BuildTools::default(),
        &cancel,
        &mut sink,
    )
    .expect("o build roda");
    (outcome.success, lines)
}

#[test]
fn changing_the_profile_with_the_project_open_changes_the_c_build() {
    if !tools_present() {
        eprintln!("NAO PROVADO: cmake ou cc ausente nesta maquina");
        return;
    }
    let root = project("troca", false);

    // Estrito: o aviso vira erro e o build para.
    let (ok, _) = build(&root, RigorProfile::Strict);
    assert!(!ok, "no Estrito o aviso deveria parar o build");

    // Equilibrado, SEM fechar nada: o build reconfigura e passa.
    let (ok, lines) = build(&root, RigorProfile::Balanced);
    assert!(ok, "no Equilibrado o aviso nao para o build: {lines:?}");
    assert!(
        lines
            .iter()
            .any(|line| line.contains("perfil de rigor mudou (strict -> balanced)")),
        "a troca tem de reconfigurar e dizer: {lines:?}"
    );

    // De volta ao Estrito: volta a parar.
    let (ok, _) = build(&root, RigorProfile::Strict);
    assert!(!ok, "de volta ao Estrito o build para de novo");
    drop(std::fs::remove_dir_all(&root));
}

#[test]
fn relaxed_wins_even_when_the_project_asks_for_werror() {
    if !tools_present() {
        eprintln!("NAO PROVADO: cmake ou cc ausente nesta maquina");
        return;
    }
    let root = project("relaxado", true);
    // Equilibrado: o projeto decide, e ele pediu aviso como erro.
    let (ok, _) = build(&root, RigorProfile::Balanced);
    assert!(
        !ok,
        "no Equilibrado vale o COMPILE_WARNING_AS_ERROR do projeto"
    );
    // Relaxado: o aviso nunca para o build.
    let (ok, lines) = build(&root, RigorProfile::Relaxed);
    assert!(ok, "no Relaxado o aviso nao para o build: {lines:?}");
    drop(std::fs::remove_dir_all(&root));
}

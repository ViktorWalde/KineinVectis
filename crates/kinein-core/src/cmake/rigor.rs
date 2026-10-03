//! O PERFIL DE RIGOR no configure do `CMake` (M4.5, 2026-10-03).
//!
//! Ate' aqui o perfil (Estrito/Equilibrado/Relaxado) nao chegava ao C/C++: o
//! build ignorava-o e so' configurava quando faltava o cache, entao trocar o
//! perfil com o projeto aberto nao mudava nada. Este modulo e' o dono de
//! tudo o que o perfil muda no `CMake`: os argumentos, a deteccao da versao,
//! a anotacao do perfil aplicado e a decisao de reconfigurar.

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use kinein_protocol::{RigorProfile, ToolchainRole};

use super::{build_dir, configure_command};

/// O arquivo em que a IDE anota o perfil de rigor do ultimo configure
/// (M4.5): o build so' reconfigura quando o perfil escolhido mudou.
const RIGOR_MARKER: &str = ".kinein-rigor";

/// A chave estavel de um perfil, a mesma do `settings.json`.
#[must_use]
pub const fn rigor_key(profile: RigorProfile) -> &'static str {
    match profile {
        RigorProfile::Strict => "strict",
        RigorProfile::Balanced => "balanced",
        RigorProfile::Relaxed => "relaxed",
    }
}

/// O que o perfil de rigor muda no configure de um projeto C/C++ (M4.5),
/// pelo mecanismo oficial do `CMake` (>= 3.24), sem tocar no `CMakeLists`
/// nem nas flags do usuario:
///
/// - Estrito: `CMAKE_COMPILE_WARNING_AS_ERROR=ON` — aviso para o build.
/// - Equilibrado: `OFF` — o projeto decide (a propriedade dele vale).
/// - Relaxado: `OFF` e `--compile-no-warning-as-error` — aviso nunca para o
///   build, nem quando o projeto pede.
///
/// `has_flag` diz se o `cmake` conhece o `--compile-no-warning-as-error`
/// (3.24+); sem ele, o Relaxado fica igual ao Equilibrado.
#[must_use]
pub(super) fn rigor_arguments(profile: RigorProfile, has_flag: bool) -> Vec<String> {
    match profile {
        RigorProfile::Strict => vec!["-DCMAKE_COMPILE_WARNING_AS_ERROR=ON".to_owned()],
        RigorProfile::Balanced => vec!["-DCMAKE_COMPILE_WARNING_AS_ERROR=OFF".to_owned()],
        RigorProfile::Relaxed => {
            let mut arguments = vec!["-DCMAKE_COMPILE_WARNING_AS_ERROR=OFF".to_owned()];
            if has_flag {
                arguments.push("--compile-no-warning-as-error".to_owned());
            }
            arguments
        }
    }
}

/// O `cmake` em `program` e' 3.24 ou mais novo (o `CMAKE_COMPILE_WARNING_AS_ERROR`
/// e o `--compile-no-warning-as-error` nasceram ali)? Versao ilegivel = nao.
#[must_use]
fn supports_warning_as_error(program: &Path) -> bool {
    Command::new(program)
        .arg("--version")
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .is_some_and(|text| version_at_least(&text, 3, 24))
}

fn version_at_least(version_text: &str, major: u32, minor: u32) -> bool {
    let Some(numbers) = version_text
        .split_whitespace()
        .find(|word| word.chars().next().is_some_and(|c| c.is_ascii_digit()))
    else {
        return false;
    };
    let mut parts = numbers
        .split('.')
        .map(|part| part.parse::<u32>().unwrap_or(0));
    let found = (parts.next().unwrap_or(0), parts.next().unwrap_or(0));
    found >= (major, minor)
}

/// O perfil com que o build dir foi configurado da ultima vez, se anotado.
#[must_use]
pub fn applied_rigor(root: &Path) -> Option<String> {
    fs::read_to_string(build_dir(root).join(RIGOR_MARKER))
        .ok()
        .map(|text| text.trim().to_owned())
        .filter(|text| !text.is_empty())
}

/// Anota o perfil de um configure que deu certo.
pub fn record_rigor(root: &Path, profile: RigorProfile) {
    drop(fs::write(
        build_dir(root).join(RIGOR_MARKER),
        format!("{}\n", rigor_key(profile)),
    ));
}

/// O build precisa (re)configurar.
///
/// Sem cache, ou com outro perfil de rigor que o escolhido agora. Ate'
/// 2026-10-03 so' a falta de cache contava, e trocar o perfil com o projeto
/// aberto nao mudava nada no C/C++.
#[must_use]
pub fn needs_configure(root: &Path, profile: RigorProfile) -> bool {
    !build_dir(root).join("CMakeCache.txt").is_file()
        || applied_rigor(root).as_deref() != Some(rigor_key(profile))
}

/// O configure completo de um perfil de rigor: o [`configure_command`] mais
/// o que o perfil muda. O build e o `cmake.configure` usam este, o mesmo.
#[must_use]
pub fn configure_with_rigor(
    root: &Path,
    preset: Option<&str>,
    toolchain: &crate::toolchain::Toolchain,
    extra: &[String],
    profile: RigorProfile,
) -> Command {
    let program = toolchain
        .program_for(ToolchainRole::Cmake)
        .unwrap_or_else(|| PathBuf::from("cmake"));
    let rigor = rigor_arguments(profile, supports_warning_as_error(&program));
    configure_command(root, preset, toolchain, extra, &rigor)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::{build_dir, needs_configure, record_rigor, rigor_arguments, version_at_least};

    #[test]
    fn each_rigor_profile_says_what_it_changes_in_the_configure() {
        use kinein_protocol::RigorProfile;
        assert_eq!(
            rigor_arguments(RigorProfile::Strict, true),
            ["-DCMAKE_COMPILE_WARNING_AS_ERROR=ON"]
        );
        assert_eq!(
            rigor_arguments(RigorProfile::Balanced, true),
            ["-DCMAKE_COMPILE_WARNING_AS_ERROR=OFF"]
        );
        assert_eq!(
            rigor_arguments(RigorProfile::Relaxed, true),
            [
                "-DCMAKE_COMPILE_WARNING_AS_ERROR=OFF",
                "--compile-no-warning-as-error"
            ]
        );
        // Um cmake antigo nao conhece a flag: o Relaxado vira o Equilibrado.
        assert_eq!(
            rigor_arguments(RigorProfile::Relaxed, false),
            ["-DCMAKE_COMPILE_WARNING_AS_ERROR=OFF"]
        );
    }

    #[test]
    fn the_cmake_version_is_read_from_its_own_banner() {
        assert!(version_at_least("cmake version 4.2.3\n", 3, 24));
        assert!(version_at_least("cmake version 3.24.0", 3, 24));
        assert!(!version_at_least("cmake version 3.22.1", 3, 24));
        assert!(!version_at_least("sem versao", 3, 24));
    }

    #[test]
    fn a_new_profile_asks_for_a_new_configure() {
        use kinein_protocol::RigorProfile;
        let root = std::env::temp_dir().join(format!("kinein-rigor-marker-{}", std::process::id()));
        drop(fs::remove_dir_all(&root));
        fs::create_dir_all(build_dir(&root)).unwrap();
        assert!(needs_configure(&root, RigorProfile::Strict), "sem cache");
        fs::write(build_dir(&root).join("CMakeCache.txt"), "").unwrap();
        assert!(
            needs_configure(&root, RigorProfile::Strict),
            "cache sem perfil anotado"
        );
        record_rigor(&root, RigorProfile::Strict);
        assert!(!needs_configure(&root, RigorProfile::Strict));
        assert!(
            needs_configure(&root, RigorProfile::Balanced),
            "o perfil mudou"
        );
        drop(fs::remove_dir_all(&root));
    }
}

//! O Visual Studio do Windows, para o build de C/C++.
//!
//! A instalacao achada pelo `vswhere`, o ambiente do `vcvars64.bat` e as
//! pastas de ferramenta que ele traz: o `CMake`, o `Ninja` e o `LLVM`
//! (DocsPublic/roadmaps/60 §3.3, W6a).
//!
//! Medido em 2026-10-10: fora do ambiente do Visual Studio, nada de C/C++ esta'
//! no `PATH` do Windows, e sem o `vcvars64.bat` o `cl` nao acha o `INCLUDE` nem
//! o `LIB`. Quem roda o `CMake`, o `Ninja` e o `ctest` recebe o ambiente que o
//! "Developer `PowerShell`" teria. Fora do Windows nada disto existe: as
//! funcoes devolvem vazio e nao mexem no comando.

use std::{
    path::{Path, PathBuf},
    process::Command,
};

/// A instalacao mais nova do Visual Studio com as ferramentas de C++ (o mesmo
/// pedido do `verificar-windows.ps1`), achada uma vez por processo.
#[must_use]
pub fn installation() -> Option<&'static Path> {
    imp::installation()
}

/// As pastas de ferramenta do Visual Studio que existem nesta maquina.
///
/// O `cmake`/`ctest`, o `ninja` e o `LLVM` que ele traz (`clang-format`,
/// `clang-tidy`). O detector de ferramentas procura nelas depois do `PATH`.
#[must_use]
pub fn tool_dirs() -> Vec<PathBuf> {
    installation().map_or_else(Vec::new, |vs| {
        let cmake = vs
            .join("Common7")
            .join("IDE")
            .join("CommonExtensions")
            .join("Microsoft")
            .join("CMake");
        [
            cmake.join("CMake").join("bin"),
            cmake.join("Ninja"),
            vs.join("VC")
                .join("Tools")
                .join("Llvm")
                .join("x64")
                .join("bin"),
        ]
        .into_iter()
        .filter(|dir| dir.is_dir())
        .collect()
    })
}

/// Poe no `command` o ambiente do `vcvars64.bat`.
///
/// Assim o `CMake`, o `Ninja` e o `cl` se acham como no "Developer
/// `PowerShell`". Nao faz nada fora do Windows, sem Visual Studio, ou quando o
/// processo ja' esta' num ambiente dele (`VSCMD_VER`, que o `vcvars` define).
pub fn apply_environment(command: &mut Command) {
    imp::apply_environment(command);
}

/// Escolhe o `Ninja` para o configure quando ninguem escolheu gerador.
///
/// No Windows o padrao do `CMake` e' o gerador do Visual Studio, que nao gera
/// o `compile_commands.json` de que o indice e o `clangd` dependem. A variavel
/// `CMAKE_GENERATOR` so' vale sem `-G` e sem gerador no preset, e o usuario
/// que a definiu continua mandando. Fora do Windows, nada.
pub fn prefer_ninja(command: &mut Command, generator_chosen: bool) {
    imp::prefer_ninja(command, generator_chosen);
}

#[cfg(windows)]
mod imp {
    use std::{
        collections::HashMap,
        ffi::OsString,
        os::windows::process::CommandExt,
        path::{Path, PathBuf},
        process::Command,
        sync::OnceLock,
    };

    pub(super) fn installation() -> Option<&'static Path> {
        static FOUND: OnceLock<Option<PathBuf>> = OnceLock::new();
        FOUND.get_or_init(find_installation).as_deref()
    }

    fn find_installation() -> Option<PathBuf> {
        let vswhere = PathBuf::from(std::env::var_os("ProgramFiles(x86)")?)
            .join("Microsoft Visual Studio")
            .join("Installer")
            .join("vswhere.exe");
        let output = Command::new(vswhere)
            .args([
                "-latest",
                "-products",
                "*",
                "-requires",
                "Microsoft.VisualStudio.Component.VC.Tools.x86.x64",
                "-property",
                "installationPath",
                "-utf8",
            ])
            .output()
            .ok()?;
        let text = String::from_utf8(output.stdout).ok()?;
        let path = PathBuf::from(text.lines().next()?.trim());
        path.is_dir().then_some(path)
    }

    /// O ambiente do `vcvars64.bat`, capturado uma vez: so' o que difere do
    /// ambiente deste processo.
    fn environment() -> Option<&'static [(OsString, OsString)]> {
        static CAPTURED: OnceLock<Option<Vec<(OsString, OsString)>>> = OnceLock::new();
        CAPTURED.get_or_init(capture_environment).as_deref()
    }

    fn capture_environment() -> Option<Vec<(OsString, OsString)>> {
        let vcvars = installation()?
            .join("VC")
            .join("Auxiliary")
            .join("Build")
            .join("vcvars64.bat");
        if !vcvars.is_file() {
            return None;
        }
        // `/U`: o `set` sai em UTF-16, e um caminho com acento chega inteiro.
        // `raw_arg`: o `cmd` nao entende as aspas escapadas com `\"` que o
        // `std` poria; com `/S`, ele tira so' o primeiro e o ultimo `"`.
        let output = Command::new("cmd")
            .raw_arg(format!(
                "/U /D /S /C \"call \"{}\" >nul 2>&1 && set\"",
                vcvars.display()
            ))
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let units: Vec<u16> = output
            .stdout
            .chunks_exact(2)
            .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
            .collect();
        let text = String::from_utf16_lossy(&units);
        // O Windows nao distingue maiuscula no nome da variavel.
        let current: HashMap<String, OsString> = std::env::vars_os()
            .map(|(name, value)| (name.to_string_lossy().to_uppercase(), value))
            .collect();
        let changed: Vec<(OsString, OsString)> = text
            .lines()
            .filter_map(|line| line.split_once('='))
            // `=C:=C:\...`: as variaveis do diretorio de cada drive, que o `cmd`
            // guarda com nome vazio.
            .filter(|(name, _value)| !name.is_empty())
            .filter(|(name, value)| {
                current
                    .get(&name.to_uppercase())
                    .is_none_or(|known| known != &OsString::from(*value))
            })
            .map(|(name, value)| (OsString::from(name), OsString::from(value)))
            .collect();
        (!changed.is_empty()).then_some(changed)
    }

    pub(super) fn apply_environment(command: &mut Command) {
        if std::env::var_os("VSCMD_VER").is_some() {
            return;
        }
        if let Some(vars) = environment() {
            command.envs(vars.iter().map(|(name, value)| (name, value)));
        }
    }

    pub(super) fn prefer_ninja(command: &mut Command, generator_chosen: bool) {
        if !generator_chosen && std::env::var_os("CMAKE_GENERATOR").is_none() {
            command.env("CMAKE_GENERATOR", "Ninja");
        }
    }
}

// Nao `const`: a assinatura e' a do Windows, onde cada uma faz chamada ao
// sistema; o `const` daqui faria o clippy pedir `const` a quem as chama.
#[cfg(not(windows))]
#[allow(clippy::missing_const_for_fn)]
mod imp {
    use std::{path::Path, process::Command};

    pub(super) fn installation() -> Option<&'static Path> {
        None
    }

    pub(super) fn apply_environment(_command: &mut Command) {}

    pub(super) fn prefer_ninja(_command: &mut Command, _generator_chosen: bool) {}
}

#[cfg(test)]
#[cfg(windows)]
mod tests {
    /// O gate do Windows exige o Visual Studio com o C++: aqui ele e' achado,
    /// e as pastas de ferramenta dele existem.
    #[test]
    fn the_visual_studio_and_its_tool_folders_are_found() {
        let vs = super::installation().expect("o gate do Windows exige o Visual Studio");
        assert!(vs.join("VC").is_dir(), "{}", vs.display());
        let dirs = super::tool_dirs();
        assert!(
            dirs.iter().any(|dir| dir.join("cmake.exe").is_file()),
            "{dirs:?}"
        );
        assert!(
            dirs.iter().any(|dir| dir.join("ninja.exe").is_file()),
            "{dirs:?}"
        );
    }

    /// Com o ambiente do `vcvars64.bat`, um `cmd` acha o `cl` e o `INCLUDE`
    /// do MSVC, que fora dele nao existem.
    #[test]
    fn the_vcvars_environment_brings_cl_and_the_include_folders() {
        let mut command = std::process::Command::new("cmd");
        command.args(["/D", "/C", "where cl && echo INCLUDE=%INCLUDE%"]);
        super::apply_environment(&mut command);
        let output = command.output().unwrap();
        let text = String::from_utf8_lossy(&output.stdout);
        assert!(output.status.success(), "{text}");
        assert!(text.to_lowercase().contains("cl.exe"), "{text}");
        assert!(text.contains("MSVC"), "{text}");
    }

    #[test]
    fn ninja_is_preferred_only_when_nobody_chose_a_generator() {
        let read = |command: &std::process::Command| {
            command
                .get_envs()
                .find(|(name, _)| *name == "CMAKE_GENERATOR")
                .and_then(|(_, value)| value.map(ToOwned::to_owned))
        };
        let mut free = std::process::Command::new("cmake");
        super::prefer_ninja(&mut free, false);
        assert_eq!(read(&free), Some("Ninja".into()));
        let mut chosen = std::process::Command::new("cmake");
        super::prefer_ninja(&mut chosen, true);
        assert_eq!(read(&chosen), None);
    }
}

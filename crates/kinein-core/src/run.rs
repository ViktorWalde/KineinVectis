//! O que "Executar" roda, e como se mostra.
//!
//! A EXECUCAO em si e' uma sessao de terminal (PTY) desde 2026-09-18, a
//! pedido do autor: "ja' temos o terminal integrado, nao precisamos de mais
//! nada para executar". O `run.start`/`run.script` resolvem o comando
//! (configuracao ativa, lancador padrao do tipo de projeto, lancador
//! Python/MicroPython) e o abrem numa aba de terminal real — stdin, cores e
//! programas de tela cheia funcionam de graca, e a saida chega por
//! `event.terminal.render` como qualquer outra. O que ficou aqui e' o que
//! NAO e' execucao: o erro, o catalogo de extensoes, o comando padrao por
//! tipo e a linha que a tela mostra.

use std::{error::Error, ffi::OsStr, fmt, path::Path};

use kinein_protocol::ProjectKind;

/// Error produced while resolving or starting a run.
#[derive(Debug)]
pub enum RunError {
    /// No run is currently open.
    NotRunning,
    /// The project kind has no default run command.
    NoDefaultCommand {
        /// Human explanation of what to do instead.
        message: String,
    },
    /// The terminal session could not be opened.
    Process {
        /// Underlying failure description.
        message: String,
    },
}

impl fmt::Display for RunError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotRunning => write!(formatter, "nenhuma execução aberta"),
            Self::NoDefaultCommand { message } | Self::Process { message } => {
                write!(formatter, "{message}")
            }
        }
    }
}

impl Error for RunError {}

/// Extensoes de shell que `run.script` aceita, e o interpretador de cada uma.
/// **Fonte unica** com `script_interpreter` e com `run.capabilities`: a UI
/// nao mantem lista propria (a mesma regra do `format.capabilities`, que
/// nasceu de duas listas divergindo em silencio).
#[cfg(unix)]
const SHELL_SCRIPTS: [(&str, &str); 3] = [("sh", "bash"), ("bash", "bash"), ("zsh", "zsh")];

/// No Windows (60 §3.3, W2b): o `.sh` e o `.bash` rodam no bash do Git for
/// Windows; os scripts do proprio sistema, no PowerShell e no `cmd`. O `zsh`
/// nao existe la'. Quem resolve o nome no programa e' o
/// `platform::script_command`.
#[cfg(windows)]
const SHELL_SCRIPTS: [(&str, &str); 5] = [
    ("sh", "bash"),
    ("bash", "bash"),
    ("ps1", "powershell"),
    ("cmd", "cmd"),
    ("bat", "cmd"),
];

/// Extensoes que `run.script` entrega ao Python do projeto (`handlers/run`).
pub const PYTHON_SCRIPTS: [&str; 1] = ["py"];

/// Interpreter for shell-script file types intentionally exposed by the UI.
#[must_use]
pub fn script_interpreter(path: &Path) -> Option<&'static str> {
    let extension = path.extension().and_then(OsStr::to_str)?;
    SHELL_SCRIPTS
        .iter()
        .find(|(ext, _)| *ext == extension)
        .map(|(_, interpreter)| *interpreter)
}

/// O que "Executar" e "Depurar" aceitam por extensao (`run.capabilities`):
/// os shells e o Python; so' o Python se depura (o debugpy do projeto).
#[must_use]
pub fn capabilities() -> (Vec<&'static str>, Vec<&'static str>) {
    let runnable = SHELL_SCRIPTS
        .iter()
        .map(|(ext, _)| *ext)
        .chain(PYTHON_SCRIPTS)
        .collect();
    (runnable, PYTHON_SCRIPTS.to_vec())
}

/// Shell-like label used only for display in the Run panel and events.
#[must_use]
pub fn script_display_command(root: &Path, interpreter: &str, script: &Path) -> String {
    let display_path = script.strip_prefix(root).map_or_else(
        |_| script.display().to_string(),
        crate::platform::portable_relative,
    );
    format!("{interpreter} -- '{}'", display_path.replace('\'', "'\\''"))
}

/// Derives the default run command for the magic Run button.
///
/// Python nao passa por aqui: precisa do lancador do projeto (interpretador
/// ou `uv run`), que o handler resolve — `crate::python::run::default_command`.
pub fn default_command(kind: ProjectKind, root: &Path) -> Result<String, RunError> {
    match kind {
        ProjectKind::RustCargo => Ok("cargo run".to_owned()),
        ProjectKind::Cmake => cmake_binary_command(root),
        ProjectKind::Maven
        | ProjectKind::Gradle
        | ProjectKind::Python
        | ProjectKind::Make
        | ProjectKind::PlatformIo
        | ProjectKind::Unknown => Err(RunError::NoDefaultCommand {
            message: "este tipo de projeto ainda não tem comando de execução padrão; \
                          digite o comando no painel Terminal"
                .to_owned(),
        }),
    }
}

/// Finds the single executable the `CMake` build produced in the folder of
/// the active configuration (C1-21), if any.
fn cmake_binary_command(root: &Path) -> Result<String, RunError> {
    let build_dir = crate::cmake_configuration::active_dir(root)
        .map_err(|message| RunError::NoDefaultCommand { message })?;
    let location = crate::cmake_configuration::label(root, &build_dir);
    let executables = cmake_executables(root, &build_dir)
        .map_err(|message| RunError::NoDefaultCommand { message })?;

    match executables.as_slice() {
        [] => Err(RunError::NoDefaultCommand {
            message: format!("nenhum executável encontrado em {location}; compile antes (Ctrl+F9)"),
        }),
        [single] => Ok(crate::platform::program_invocation(single)),
        _multiple => Err(RunError::NoDefaultCommand {
            message: format!(
                "mais de um executável em {location} ({}); \
                 digite o comando no painel Terminal",
                executables.join(", ")
            ),
        }),
    }
}

/// The executables of a `CMake` build folder: the artifacts of the
/// `EXECUTABLE` targets in the codemodel, which in a multi-config build are
/// those of the configuration clangd reads (C1-22). A shared library has the
/// bit too (C1-30). Without the model, a single-config folder is walked.
pub(crate) fn cmake_executables(root: &Path, build_dir: &Path) -> Result<Vec<String>, String> {
    let Some(model) = crate::cmake::model::CmakeModel::load(build_dir) else {
        if let Some(configuration) = crate::cmake::build_configuration::of(build_dir) {
            return Err(format!(
                "o build multi-config em {} não tem o modelo do file-api, que diz quais \
                 executáveis são da configuração {}; configure pela IDE (Configurar)",
                crate::cmake_configuration::label(root, build_dir),
                configuration.name
            ));
        }
        let mut found = Vec::new();
        collect_executables(build_dir, &mut found);
        return Ok(found);
    };
    Ok(model
        .targets
        .iter()
        .filter(|target| target.kind == "EXECUTABLE")
        .flat_map(|target| &target.artifacts)
        .filter(|artifact| artifact.is_file() && crate::platform::is_executable(artifact))
        .map(|artifact| artifact.display().to_string())
        .collect())
}

/// `libm.so`, `libm.so.1.2.3` and `libm.1.dylib` (C1-30).
fn is_shared_library(name: &str) -> bool {
    Path::new(name).extension().is_some_and(|extension| {
        extension.eq_ignore_ascii_case("so") || extension.eq_ignore_ascii_case("dylib")
    }) || name.split_once(".so.").is_some_and(|(_, version)| {
        version
            .split('.')
            .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
    })
}

fn collect_executables(dir: &Path, found: &mut Vec<String>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        let name = entry.file_name().to_string_lossy().into_owned();
        if file_type.is_dir() {
            if name != "CMakeFiles" {
                collect_executables(&entry.path(), found);
            }
        } else if file_type.is_file()
            && !is_shared_library(&name)
            && crate::platform::is_executable(&entry.path())
        {
            found.push(entry.path().display().to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use kinein_protocol::ProjectKind;

    use super::{
        PYTHON_SCRIPTS, RunError, capabilities, default_command, script_display_command,
        script_interpreter,
    };

    fn temp_root(test_name: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("kinein-run-tests")
            .join(format!("{}-{test_name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        crate::platform::canonicalize(&dir).unwrap()
    }

    /// O catalogo que a UI recebe e' a decisao do `run.script`: toda extensao
    /// runnable de shell resolve um interpretador, o `py` e' o do Python, e
    /// so' o Python e' debuggable. Uma segunda lista em QML diverge por
    /// construcao — foi assim com o format.capabilities.
    #[test]
    fn the_published_catalogue_matches_the_decision() {
        let (runnable, debuggable) = capabilities();
        if cfg!(windows) {
            assert_eq!(runnable, ["sh", "bash", "ps1", "cmd", "bat", "py"]);
        } else {
            assert_eq!(runnable, ["sh", "bash", "zsh", "py"]);
        }
        assert_eq!(debuggable, ["py"]);
        for ext in &runnable {
            let caminho = PathBuf::from(format!("x.{ext}"));
            let e_shell = script_interpreter(&caminho).is_some();
            let e_python = PYTHON_SCRIPTS.contains(ext);
            assert!(e_shell != e_python, ".{ext} e' shell OU python");
        }
        assert!(script_interpreter(Path::new("x.py")).is_none());
        assert!(script_interpreter(Path::new("x.txt")).is_none());
    }

    #[test]
    fn shell_script_detection_and_display_are_explicit() {
        let root = PathBuf::from("/workspace");
        let script = root.join("scripts/check it's.sh");

        assert_eq!(script_interpreter(&script), Some("bash"));
        assert_eq!(
            script_display_command(&root, "bash", &script),
            "bash -- 'scripts/check it'\\''s.sh'"
        );
        assert_eq!(script_interpreter(&root.join("script.py")), None);
    }

    #[test]
    fn default_command_covers_rust_and_rejects_kinds_without_default() {
        let root = temp_root("default");

        assert_eq!(
            default_command(ProjectKind::RustCargo, &root).unwrap(),
            "cargo run"
        );
        assert!(matches!(
            default_command(ProjectKind::Unknown, &root),
            Err(RunError::NoDefaultCommand { .. })
        ));
        assert!(matches!(
            default_command(ProjectKind::Cmake, &root),
            Err(RunError::NoDefaultCommand { .. })
        ));
    }

    /// C1-30: the names of a shared library, which has the bit too.
    #[test]
    fn a_shared_library_is_not_an_executable() {
        for (name, expected) in [
            ("libkv.so", true),
            ("libkv.so.1", true),
            ("libkv.so.1.2.3", true),
            ("libkv.dylib", true),
            ("libkv.1.dylib", true),
            ("libkv.SO", true),
            ("app", false),
            ("solver", false),
            ("app.sonar", false),
            ("libkv.so.", false),
            ("libkv.so.1.", false),
            ("libkv.so.1.bak", false),
            ("libkv.so.d", false),
            ("libkv.socket", false),
        ] {
            assert_eq!(super::is_shared_library(name), expected, "{name}");
        }
    }

    /// O executavel do build e' o que o sistema executa: no Unix o arquivo com
    /// bit de execucao, no Windows o `.exe` (60 §3.3, W6a). O comando e' o
    /// do shell do Run (D11).
    #[test]
    fn default_command_finds_single_cmake_executable() {
        let root = temp_root("cmake-bin");
        let build = root.join(".kinein").join("build");
        std::fs::create_dir_all(build.join("CMakeFiles")).unwrap();
        crate::write_executable(build.join("CMakeFiles/ignorado"), "#!/bin/sh\n");

        let binary = crate::write_executable(build.join("app"), "#!/bin/sh\n");
        // C1-30: a shared library has the bit too.
        crate::write_executable(build.join("libkv.so"), "\x7fELF");
        crate::write_executable(build.join("libkv.so.1.2.3"), "\x7fELF");

        let command = default_command(ProjectKind::Cmake, &root).unwrap();
        assert_eq!(
            command,
            crate::platform::program_invocation(&binary.display().to_string())
        );
    }
}

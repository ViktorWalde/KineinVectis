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
            Self::NotRunning => write!(formatter, "nenhuma execucao aberta"),
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
const SHELL_SCRIPTS: [(&str, &str); 3] = [("sh", "bash"), ("bash", "bash"), ("zsh", "zsh")];

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
    let display_path = script
        .strip_prefix(root)
        .unwrap_or(script)
        .display()
        .to_string();
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
            message: "este tipo de projeto ainda nao tem comando de execucao padrao; \
                          digite o comando no painel Terminal"
                .to_owned(),
        }),
    }
}

/// Finds the single executable produced by the `CMake` build, if any.
fn cmake_binary_command(root: &Path) -> Result<String, RunError> {
    let build_dir = root.join(".kinein").join("build");
    let mut executables = Vec::new();
    collect_executables(&build_dir, &mut executables);

    match executables.as_slice() {
        [] => Err(RunError::NoDefaultCommand {
            message: "nenhum executavel encontrado em .kinein/build; \
                      compile antes (Ctrl+F9)"
                .to_owned(),
        }),
        [single] => Ok(format!("'{}'", single.replace('\'', "'\\''"))),
        _multiple => Err(RunError::NoDefaultCommand {
            message: format!(
                "mais de um executavel em .kinein/build ({}); \
                 digite o comando no painel Terminal",
                executables.join(", ")
            ),
        }),
    }
}

/// Collects executable regular files under `dir`, skipping `CMakeFiles`.
pub(crate) fn collect_executables(dir: &Path, found: &mut Vec<String>) {
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
        } else if file_type.is_file() && is_executable(&entry.path()) {
            found.push(entry.path().display().to_string());
        }
    }
}

/// Returns `true` when the file has any execute permission bit set.
#[cfg(unix)]
pub(crate) fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(path).is_ok_and(|metadata| metadata.permissions().mode() & 0o111 != 0)
}

/// Non-Unix platforms have no execute bit; nothing is auto-runnable.
#[cfg(not(unix))]
pub(crate) fn is_executable(_path: &Path) -> bool {
    false
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
        dir.canonicalize().unwrap()
    }

    /// O catalogo que a UI recebe e' a decisao do `run.script`: toda extensao
    /// runnable de shell resolve um interpretador, o `py` e' o do Python, e
    /// so' o Python e' debuggable. Uma segunda lista em QML diverge por
    /// construcao — foi assim com o format.capabilities.
    #[test]
    fn the_published_catalogue_matches_the_decision() {
        let (runnable, debuggable) = capabilities();
        assert_eq!(runnable, ["sh", "bash", "zsh", "py"]);
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

    #[test]
    #[cfg(unix)]
    fn default_command_finds_single_cmake_executable() {
        use std::os::unix::fs::PermissionsExt;

        let root = temp_root("cmake-bin");
        let build = root.join(".kinein").join("build");
        std::fs::create_dir_all(build.join("CMakeFiles")).unwrap();
        std::fs::write(build.join("CMakeFiles/ignorado"), "#!/bin/sh\n").unwrap();
        let mut ignored_permissions = std::fs::metadata(build.join("CMakeFiles/ignorado"))
            .unwrap()
            .permissions();
        ignored_permissions.set_mode(0o755);
        std::fs::set_permissions(build.join("CMakeFiles/ignorado"), ignored_permissions).unwrap();

        let binary = build.join("app");
        std::fs::write(&binary, "#!/bin/sh\n").unwrap();
        let mut permissions = std::fs::metadata(&binary).unwrap().permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(&binary, permissions).unwrap();

        let command = default_command(ProjectKind::Cmake, &root).unwrap();
        assert!(command.contains("app"));
        assert!(command.starts_with('\''));
    }
}

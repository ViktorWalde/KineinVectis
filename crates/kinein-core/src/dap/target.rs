//! Resolucao do binario a depurar ("Automatico", espelho da heuristica do
//! run): cargo usa o unico executavel no topo de `target/debug`, cmake o
//! unico executavel de `.kinein/build`; Python (fatia 4 da cadeia do
//! `roadmaps/41`, 2026-09-13) usa o MESMO ponto de entrada do botao Executar
//! (`python::run::entry_point`). O Target selector visual da spec substitui
//! isso na C5+.

use std::path::{Path, PathBuf};

use kinein_protocol::{DebugConnectParams, ProjectKind};

use super::DebugError;

/// O que o adaptador vai depurar.
///
/// Um executavel (ou um `.py`), ou um MODULO Python (`python -m pacote`, o
/// `module` do launch do debugpy — medido no 1.8.21 em 2026-09-13: para no
/// breakpoint dentro do pacote e a saida vem por `output`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DebugTarget {
    /// Caminho absoluto de um executavel ou script.
    Program(PathBuf),
    /// Nome de um pacote/modulo Python com `__main__.py`.
    Module(String),
    /// Existing Python process, exposed by debugpy.listen / --listen.
    PythonAttach(DebugConnectParams),
}

impl DebugTarget {
    /// Only the Python TCP attach leaves the debuggee owned externally.
    #[must_use]
    pub const fn is_attached(&self) -> bool {
        matches!(self, Self::PythonAttach(_))
    }

    /// O caminho, quando o alvo e' um arquivo (o servidor de debug e o
    /// `build.size` precisam de um ELF; um modulo nao tem).
    #[must_use]
    pub fn program_path(&self) -> Option<&Path> {
        match self {
            Self::Program(p) => Some(p),
            Self::Module(_) | Self::PythonAttach(_) => None,
        }
    }

    /// Como a tela mostra: o caminho, ou `-m <pacote>`.
    #[must_use]
    pub fn display(&self) -> String {
        match self {
            Self::Program(p) => p.display().to_string(),
            Self::Module(m) => format!("-m {m}"),
            Self::PythonAttach(endpoint) => format!("debugpy {}:{}", endpoint.host, endpoint.port),
        }
    }
}

/// Resolves the target `debug.start` should hand to the adapter.
///
/// Mirrors `run::default_command`: a single obvious binary or a clear,
/// actionable error — never a guess between candidates.
pub fn resolve_program(kind: ProjectKind, root: &Path) -> Result<DebugTarget, DebugError> {
    match kind {
        ProjectKind::RustCargo => cargo_binary(root).map(DebugTarget::Program),
        ProjectKind::Cmake => cmake_binary(root).map(DebugTarget::Program),
        ProjectKind::Python => python_entry(root),
        ProjectKind::Maven | ProjectKind::Gradle | ProjectKind::Unknown => {
            Err(DebugError::NoTarget {
                message: "este tipo de projeto ainda nao tem alvo de debug automatico; \
                          informe o executavel em debug.start { program }"
                    .to_owned(),
            })
        }
    }
}

/// O ponto de entrada do Executar, como alvo: `main.py`/`app.py`/
/// `__main__.py` na raiz ou o script de `[project.scripts]` instalado (um
/// arquivo Python com shebang — o debugpy o lanca como `program`); um pacote
/// com `__main__.py` vira `Module` (o `module` do launch, desde 2026-09-13).
fn python_entry(root: &Path) -> Result<DebugTarget, DebugError> {
    use crate::python::run::EntryPoint;

    match crate::python::run::entry_point(root) {
        Some(EntryPoint::File(arquivo)) => Ok(DebugTarget::Program(root.join(arquivo))),
        Some(EntryPoint::InstalledScript(nome)) => {
            Ok(DebugTarget::Program(root.join(".venv/bin").join(nome)))
        }
        Some(EntryPoint::Module(pacote)) => Ok(DebugTarget::Module(pacote)),
        None => Err(DebugError::NoTarget {
            message: "nenhum ponto de entrada Python (main.py, app.py, __main__.py na raiz; um \
                      pacote com __main__.py; um script de [project.scripts] instalado); \
                      informe o arquivo em debug.start { program }"
                .to_owned(),
        }),
    }
}

/// Finds the single cargo binary at the top level of `target/debug`.
///
/// Only extension-less executables count: helper artifacts (`*.d`, `*.so`,
/// `build/`, `deps/`) never become debug targets by accident.
fn cargo_binary(root: &Path) -> Result<PathBuf, DebugError> {
    let debug_dir = root.join("target").join("debug");
    let mut binaries = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&debug_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let is_file = entry.file_type().is_ok_and(|kind| kind.is_file());
            if is_file && path.extension().is_none() && crate::run::is_executable(&path) {
                binaries.push(path);
            }
        }
    }
    single_binary(binaries, "target/debug")
}

/// Finds the single executable produced by the `CMake` build.
fn cmake_binary(root: &Path) -> Result<PathBuf, DebugError> {
    let build_dir = root.join(".kinein").join("build");
    let mut found = Vec::new();
    crate::run::collect_executables(&build_dir, &mut found);
    single_binary(
        found.into_iter().map(PathBuf::from).collect(),
        ".kinein/build",
    )
}

/// Accepts exactly one candidate; zero or many become actionable errors.
fn single_binary(mut binaries: Vec<PathBuf>, location: &str) -> Result<PathBuf, DebugError> {
    binaries.sort();
    match binaries.as_slice() {
        [] => Err(DebugError::NoTarget {
            message: format!("nenhum executavel em {location}; compile antes (Ctrl+F9)"),
        }),
        [single] => Ok(single.clone()),
        _multiple => Err(DebugError::NoTarget {
            message: format!(
                "mais de um executavel em {location} ({}); \
                 informe o alvo em debug.start {{ program }}",
                binaries
                    .iter()
                    .map(|path| path.display().to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }),
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use kinein_protocol::ProjectKind;

    use super::{DebugError, resolve_program};

    fn temp_root(test_name: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("kinein-dap-target-tests")
            .join(format!("{}-{test_name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir.canonicalize().unwrap()
    }

    #[cfg(unix)]
    fn write_executable(path: &Path) {
        use std::os::unix::fs::PermissionsExt;
        std::fs::write(path, "#!/bin/sh\n").unwrap();
        let mut permissions = std::fs::metadata(path).unwrap().permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(path, permissions).unwrap();
    }

    #[test]
    #[cfg(unix)]
    fn cargo_target_wants_exactly_one_extensionless_executable() {
        let root = temp_root("cargo");
        let debug_dir = root.join("target").join("debug");
        std::fs::create_dir_all(debug_dir.join("deps")).unwrap();

        assert!(matches!(
            resolve_program(ProjectKind::RustCargo, &root),
            Err(DebugError::NoTarget { .. })
        ));

        write_executable(&debug_dir.join("app"));
        std::fs::write(debug_dir.join("app.d"), "dep info").unwrap();
        write_executable(&debug_dir.join("deps").join("ignorado"));
        let program = resolve_program(ProjectKind::RustCargo, &root).unwrap();
        assert!(
            program
                .program_path()
                .unwrap()
                .ends_with("target/debug/app")
        );

        write_executable(&debug_dir.join("outro"));
        let error = resolve_program(ProjectKind::RustCargo, &root).unwrap_err();
        assert!(error.to_string().contains("mais de um executavel"));
    }

    #[test]
    #[cfg(unix)]
    fn cmake_target_reuses_the_build_dir_scan() {
        let root = temp_root("cmake");
        let build = root.join(".kinein").join("build");
        std::fs::create_dir_all(&build).unwrap();
        write_executable(&build.join("app"));

        let program = resolve_program(ProjectKind::Cmake, &root).unwrap();
        assert!(program.program_path().unwrap().ends_with("app"));
    }

    /// Python: o alvo e' o MESMO ponto de entrada do Executar — arquivo na
    /// raiz ou script instalado; um pacote nao e' arquivo e o erro aponta o
    /// `__main__.py`; sem nada, o erro diz o que procurou.
    #[test]
    fn python_target_is_the_run_entry_point_as_a_file() {
        let root = temp_root("python");
        let error = resolve_program(ProjectKind::Python, &root).unwrap_err();
        assert!(error.to_string().contains("main.py"), "{error}");

        std::fs::create_dir_all(root.join("pacote")).unwrap();
        std::fs::write(root.join("pacote/__main__.py"), "").unwrap();
        // Um pacote e' um alvo: `-m pacote` (o launch por `module`).
        let alvo = resolve_program(ProjectKind::Python, &root).unwrap();
        assert_eq!(alvo, super::DebugTarget::Module("pacote".to_owned()));
        assert_eq!(alvo.display(), "-m pacote");
        assert!(alvo.program_path().is_none());

        std::fs::create_dir_all(root.join(".venv/bin")).unwrap();
        std::fs::write(
            root.join("pyproject.toml"),
            "[project.scripts]\ncli = \"p:m\"\n",
        )
        .unwrap();
        std::fs::write(root.join(".venv/bin/cli"), "#!/w/.venv/bin/python\n").unwrap();
        // O script instalado NAO vence o pacote (o run tambem nao o faz)...
        assert!(matches!(
            resolve_program(ProjectKind::Python, &root).unwrap(),
            super::DebugTarget::Module(_)
        ));
        std::fs::remove_dir_all(root.join("pacote")).unwrap();
        // ...mas vale quando e' o unico.
        assert_eq!(
            resolve_program(ProjectKind::Python, &root).unwrap(),
            super::DebugTarget::Program(root.join(".venv/bin/cli"))
        );

        std::fs::write(root.join("app.py"), "").unwrap();
        assert_eq!(
            resolve_program(ProjectKind::Python, &root).unwrap(),
            super::DebugTarget::Program(root.join("app.py"))
        );
    }

    #[test]
    fn kinds_without_default_point_to_explicit_program() {
        let root = temp_root("unknown");
        let error = resolve_program(ProjectKind::Unknown, &root).unwrap_err();
        assert!(error.to_string().contains("debug.start"));
    }
}

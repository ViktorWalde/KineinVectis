//! Resolucao do binario a depurar ("Automatico", espelho da heuristica do
//! run): cargo usa o unico executavel no topo de `target/debug`, cmake o
//! unico executavel de `.kinein/build`. O Target selector visual da spec
//! substitui isso na C5+.

use std::path::{Path, PathBuf};

use kinein_protocol::ProjectKind;

use super::DebugError;

/// Resolves the executable `debug.start` should hand to the adapter.
///
/// Mirrors `run::default_command`: a single obvious binary or a clear,
/// actionable error — never a guess between candidates.
pub fn resolve_program(kind: ProjectKind, root: &Path) -> Result<PathBuf, DebugError> {
    match kind {
        ProjectKind::RustCargo => cargo_binary(root),
        ProjectKind::Cmake => cmake_binary(root),
        ProjectKind::Maven | ProjectKind::Gradle | ProjectKind::Python | ProjectKind::Unknown => {
            Err(DebugError::NoTarget {
                message: "este tipo de projeto ainda nao tem alvo de debug automatico; \
                          informe o executavel em debug.start { program }"
                    .to_owned(),
            })
        }
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
        assert!(program.ends_with("target/debug/app"));

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
        assert!(program.ends_with("app"));
    }

    #[test]
    fn kinds_without_default_point_to_explicit_program() {
        let root = temp_root("unknown");
        let error = resolve_program(ProjectKind::Unknown, &root).unwrap_err();
        assert!(error.to_string().contains("debug.start"));
    }
}

//! Project kind detection by build system markers.

use std::path::Path;

use kinein_protocol::{BuildSystem, ProjectKind, WorkspaceCapabilities};

/// Build system markers in precedence order. The first match decides the
/// primary project kind; every match is reported to the UI.
const MARKERS: &[(&str, ProjectKind, BuildSystem)] = &[
    ("Cargo.toml", ProjectKind::RustCargo, BuildSystem::Cargo),
    ("CMakeLists.txt", ProjectKind::Cmake, BuildSystem::Cmake),
    ("pom.xml", ProjectKind::Maven, BuildSystem::Maven),
    ("build.gradle", ProjectKind::Gradle, BuildSystem::Gradle),
    ("build.gradle.kts", ProjectKind::Gradle, BuildSystem::Gradle),
    ("settings.gradle", ProjectKind::Gradle, BuildSystem::Gradle),
    (
        "settings.gradle.kts",
        ProjectKind::Gradle,
        BuildSystem::Gradle,
    ),
    // Um Makefile puro (P0 do 40 §4.1, 2026-09-17): depois do CMake, porque
    // uma arvore CMake tambem pode carregar um Makefile na raiz.
    ("Makefile", ProjectKind::Make, BuildSystem::Make),
    ("GNUmakefile", ProjectKind::Make, BuildSystem::Make),
    ("pyproject.toml", ProjectKind::Python, BuildSystem::Python),
    ("setup.py", ProjectKind::Python, BuildSystem::Python),
    ("requirements.txt", ProjectKind::Python, BuildSystem::Python),
];

/// Detects the project kind of a workspace root.
///
/// Returns the primary kind (first marker in precedence order) and every
/// recognized marker file present in the root.
#[must_use]
pub fn detect_project(root: &Path) -> (ProjectKind, Vec<String>, WorkspaceCapabilities) {
    let found = MARKERS
        .iter()
        .filter(|(marker, _, _)| root.join(marker).is_file())
        .collect::<Vec<_>>();

    let kind = found
        .first()
        .map_or(ProjectKind::Unknown, |(_, kind, _)| *kind);
    let markers = found
        .iter()
        .map(|(marker, _, _)| (*marker).to_owned())
        .collect();
    let mut build_systems = Vec::new();
    for (_, _, build_system) in found {
        if !build_systems.contains(build_system) {
            build_systems.push(*build_system);
        }
    }

    (kind, markers, WorkspaceCapabilities { build_systems })
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use kinein_protocol::{BuildSystem, ProjectKind};

    use super::detect_project;

    fn temp_workspace(test_name: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("kinein-workspace-detect-tests")
            .join(format!("{}-{test_name}", std::process::id()));
        if dir.exists() {
            fs::remove_dir_all(&dir).unwrap();
        }
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn empty_directory_is_unknown() {
        let dir = temp_workspace("unknown");

        let (kind, markers, capabilities) = detect_project(&dir);

        assert_eq!(kind, ProjectKind::Unknown);
        assert!(markers.is_empty());
        assert!(capabilities.build_systems.is_empty());
    }

    #[test]
    fn cargo_marker_wins_over_cmake_marker() {
        let dir = temp_workspace("precedence");
        fs::write(dir.join("Cargo.toml"), "[package]\n").unwrap();
        fs::write(dir.join("CMakeLists.txt"), "project(x)\n").unwrap();

        let (kind, markers, capabilities) = detect_project(&dir);

        assert_eq!(kind, ProjectKind::RustCargo);
        assert_eq!(markers, ["Cargo.toml", "CMakeLists.txt"]);
        assert_eq!(
            capabilities.build_systems,
            [BuildSystem::Cargo, BuildSystem::Cmake]
        );
    }

    #[test]
    fn each_build_system_is_detected() {
        let cases = [
            ("pom.xml", ProjectKind::Maven),
            ("build.gradle.kts", ProjectKind::Gradle),
            ("pyproject.toml", ProjectKind::Python),
        ];

        for (marker, expected) in cases {
            let dir = temp_workspace(marker);
            fs::write(dir.join(marker), "x\n").unwrap();

            let (kind, _, _) = detect_project(&dir);

            assert_eq!(kind, expected, "marker {marker}");
        }
    }
}

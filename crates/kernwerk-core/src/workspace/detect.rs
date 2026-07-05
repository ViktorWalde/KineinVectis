//! Project kind detection by build system markers.

use std::path::Path;

use kernwerk_protocol::ProjectKind;

/// Build system markers in precedence order. The first match decides the
/// primary project kind; every match is reported to the UI.
const MARKERS: &[(&str, ProjectKind)] = &[
    ("Cargo.toml", ProjectKind::RustCargo),
    ("CMakeLists.txt", ProjectKind::Cmake),
    ("pom.xml", ProjectKind::Maven),
    ("build.gradle", ProjectKind::Gradle),
    ("build.gradle.kts", ProjectKind::Gradle),
    ("settings.gradle", ProjectKind::Gradle),
    ("settings.gradle.kts", ProjectKind::Gradle),
    ("pyproject.toml", ProjectKind::Python),
    ("setup.py", ProjectKind::Python),
    ("requirements.txt", ProjectKind::Python),
];

/// Detects the project kind of a workspace root.
///
/// Returns the primary kind (first marker in precedence order) and every
/// recognized marker file present in the root.
#[must_use]
pub fn detect_project(root: &Path) -> (ProjectKind, Vec<String>) {
    let found = MARKERS
        .iter()
        .filter(|(marker, _)| root.join(marker).is_file())
        .collect::<Vec<_>>();

    let kind = found
        .first()
        .map_or(ProjectKind::Unknown, |(_, kind)| *kind);
    let markers = found
        .iter()
        .map(|(marker, _)| (*marker).to_owned())
        .collect();

    (kind, markers)
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use kernwerk_protocol::ProjectKind;

    use super::detect_project;

    fn temp_workspace(test_name: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("kernwerk-workspace-detect-tests")
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

        let (kind, markers) = detect_project(&dir);

        assert_eq!(kind, ProjectKind::Unknown);
        assert!(markers.is_empty());
    }

    #[test]
    fn cargo_marker_wins_over_cmake_marker() {
        let dir = temp_workspace("precedence");
        fs::write(dir.join("Cargo.toml"), "[package]\n").unwrap();
        fs::write(dir.join("CMakeLists.txt"), "project(x)\n").unwrap();

        let (kind, markers) = detect_project(&dir);

        assert_eq!(kind, ProjectKind::RustCargo);
        assert_eq!(markers, ["Cargo.toml", "CMakeLists.txt"]);
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

            let (kind, _) = detect_project(&dir);

            assert_eq!(kind, expected, "marker {marker}");
        }
    }
}

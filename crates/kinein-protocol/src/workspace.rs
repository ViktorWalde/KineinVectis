//! Workspace lifecycle payloads (`workspace.*`).

use serde::{Deserialize, Serialize};

/// Project kind detected when a workspace is opened.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ProjectKind {
    /// Rust project driven by Cargo (`Cargo.toml`).
    RustCargo,
    /// C/C++ project driven by `CMake` (`CMakeLists.txt`).
    Cmake,
    /// Java project driven by Maven (`pom.xml`).
    Maven,
    /// Java project driven by Gradle (`build.gradle`, `settings.gradle`).
    Gradle,
    /// Python project (`pyproject.toml`, `setup.py`, `requirements.txt`).
    Python,
    /// C/C++ driven by a plain `Makefile`/`GNUmakefile` (`0.115.0`): built by
    /// `make`, with `bear -- make` producing the compilation database when
    /// `bear` is installed. Loses to `CMakeLists.txt` in precedence — a `CMake`
    /// tree may also carry a Makefile.
    Make,
    /// `PlatformIO` project (`platformio.ini`, `0.117.0`): built, uploaded and
    /// monitored by `pio`. Loses to `CMakeLists.txt` (a `PlatformIO` project
    /// with `framework = espidf` is also a `CMake` tree — the build engine
    /// still picks `pio`, by the framework); wins over a plain Makefile.
    PlatformIo,
    /// No known build system marker was found.
    Unknown,
}

/// Build system capability detected in a workspace root.
///
/// This is deliberately independent from [`ProjectKind`]: `kind` keeps the
/// primary, backwards-compatible classification while a hybrid workspace can
/// expose more than one actionable build system.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BuildSystem {
    /// Cargo (`Cargo.toml`).
    Cargo,
    /// `CMake` (`CMakeLists.txt`).
    Cmake,
    /// Maven (`pom.xml`).
    Maven,
    /// Gradle (`build.gradle*`, `settings.gradle*`).
    Gradle,
    /// Python project metadata (`pyproject.toml`, `setup.py`, requirements).
    Python,
    /// Plain `Makefile` (`0.115.0`).
    Make,
    /// `PlatformIO` (`platformio.ini`, `0.117.0`).
    PlatformIo,
}

impl BuildSystem {
    /// Primary project kind associated with this build system.
    #[must_use]
    pub const fn project_kind(self) -> ProjectKind {
        match self {
            Self::Cargo => ProjectKind::RustCargo,
            Self::Cmake => ProjectKind::Cmake,
            Self::Maven => ProjectKind::Maven,
            Self::Gradle => ProjectKind::Gradle,
            Self::Python => ProjectKind::Python,
            Self::Make => ProjectKind::Make,
            Self::PlatformIo => ProjectKind::PlatformIo,
        }
    }
}

/// Actionable project capabilities detected in one pass over root markers.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceCapabilities {
    /// Detected build systems in marker-precedence order, without duplicates.
    pub build_systems: Vec<BuildSystem>,
}

impl WorkspaceCapabilities {
    /// Whether this workspace can route actions to `build_system`.
    #[must_use]
    pub fn supports(&self, build_system: BuildSystem) -> bool {
        self.build_systems.contains(&build_system)
    }
}

/// Workspace opened by the core.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceInfo {
    /// Workspace display name, derived from the root directory name.
    pub name: String,
    /// Canonical absolute path of the workspace root.
    pub root: String,
    /// Primary project kind, chosen by marker precedence.
    pub kind: ProjectKind,
    /// Every recognized build system marker found in the root.
    pub markers: Vec<String>,
    /// All actionable project systems found in the same detection pass.
    pub capabilities: WorkspaceCapabilities,
}

/// Parameters for `workspace.open`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorkspaceOpenParams {
    /// Directory to open as workspace root.
    pub path: String,
}

/// One globally persisted workspace shown in the Start Screen and File menu.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecentWorkspaceInfo {
    /// Display name captured from the last successful workspace opening.
    pub name: String,
    /// Canonical absolute workspace root.
    pub root: String,
    /// Last successful opening time as Unix epoch milliseconds.
    pub last_opened_at: u64,
    /// Whether the user pinned this entry above ordinary recent workspaces.
    pub pinned: bool,
    /// Whether the root still exists as a directory.
    pub available: bool,
}

/// Empty parameters accepted by `workspace.recent.list` and
/// `workspace.recent.clear`.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecentWorkspacesParams {}

/// Parameters for `workspace.recent.pin`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RecentWorkspacePinParams {
    /// Canonical root already present in the recent-workspace list.
    pub root: String,
    /// New pinned state.
    pub pinned: bool,
}

/// Parameters for `workspace.recent.remove`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RecentWorkspaceRemoveParams {
    /// Canonical root to remove, even when it no longer exists on disk.
    pub root: String,
}

/// Complete recent-workspace snapshot returned by every `workspace.recent.*`
/// operation.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecentWorkspacesResult {
    /// Pinned-first, last-opened-descending workspace list.
    pub workspaces: Vec<RecentWorkspaceInfo>,
}

/// Editor session restored with `workspace.open` (open tabs + active tab).
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceSession {
    /// Canonical absolute paths of files to reopen, in tab order.
    pub open_files: Vec<String>,
    /// Canonical absolute path of the tab that was active, when still valid.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_file: Option<String>,
}

/// Parameters for `workspace.saveSession`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorkspaceSaveSessionParams {
    /// Absolute paths of the open tabs, in order.
    pub open_files: Vec<String>,
    /// Absolute path of the active tab, when any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_file: Option<String>,
}

/// Result payload for `workspace.saveSession`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceSaveSessionResult {
    /// Number of entries actually persisted (invalid paths are skipped).
    pub files: u64,
}

/// Result payload for `workspace.status`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceStatusResult {
    /// Currently opened workspace, if any.
    #[serde(default)]
    pub workspace: Option<WorkspaceInfo>,
}

/// Parameters for `workspace.browse`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorkspaceBrowseParams {
    /// Directory to list in the IDE-owned workspace picker.
    pub path: String,
}

/// One directory entry returned by `workspace.browse`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceBrowseEntry {
    /// Directory name without the parent path.
    pub name: String,
    /// Canonical absolute path of the directory.
    pub path: String,
}

/// Result payload for `workspace.browse`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceBrowseResult {
    /// Canonical path that was listed.
    pub path: String,
    /// Canonical parent directory, absent for filesystem roots.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    /// Child directories sorted case-insensitively.
    pub entries: Vec<WorkspaceBrowseEntry>,
}

/// Project template supported by `workspace.createProject`.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WorkspaceProjectTemplate {
    /// Empty directory opened as an unknown project.
    Empty,
    /// Strict C++ console project using `CMake`.
    CppCmake,
    /// Rust binary project created through `cargo new`.
    RustCargo,
    /// Python project: `pyproject.toml` (PEP 621), `main.py`, a `tests/` with
    /// pytest — no tool is run; the environment is one click away in the IDE
    /// (`0.105.0`, 2026-09-13).
    Python,
}

/// Parameters for `workspace.createFolder`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorkspaceCreateFolderParams {
    /// Existing parent directory.
    pub parent: String,
    /// New child directory name, not a path.
    pub name: String,
}

/// Result payload for `workspace.createFolder`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceCreateFolderResult {
    /// Canonical path of the created directory.
    pub path: String,
}

/// Parameters for `workspace.createProject`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorkspaceCreateProjectParams {
    /// Existing parent directory where the project directory will be created.
    pub parent: String,
    /// Project directory name, not a path.
    pub name: String,
    /// Template/scaffold to create.
    pub template: WorkspaceProjectTemplate,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::{
        BuildSystem, ProjectKind, RecentWorkspaceInfo, RecentWorkspacePinParams,
        RecentWorkspaceRemoveParams, RecentWorkspacesParams, WorkspaceBrowseParams,
        WorkspaceCapabilities, WorkspaceCreateFolderParams, WorkspaceCreateProjectParams,
        WorkspaceInfo, WorkspaceOpenParams, WorkspaceProjectTemplate,
    };

    #[test]
    fn workspace_info_serializes_camel_case_kind() {
        let workspace = WorkspaceInfo {
            name: "demo".to_owned(),
            root: "/home/user/demo".to_owned(),
            kind: ProjectKind::RustCargo,
            markers: vec!["Cargo.toml".to_owned()],
            capabilities: WorkspaceCapabilities {
                build_systems: vec![BuildSystem::Cargo],
            },
        };
        let value = serde_json::to_value(workspace).unwrap();

        assert_eq!(value["kind"], "rustCargo");
        assert_eq!(value["markers"][0], "Cargo.toml");
        assert_eq!(value["capabilities"]["buildSystems"][0], "cargo");
    }

    #[test]
    fn workspace_open_params_reject_unknown_fields() {
        let valid = serde_json::from_value::<WorkspaceOpenParams>(json!({ "path": "/tmp" }));
        let invalid =
            serde_json::from_value::<WorkspaceOpenParams>(json!({ "path": "/tmp", "x": 1 }));

        assert_eq!(valid.unwrap().path, "/tmp");
        assert!(invalid.is_err());
    }

    #[test]
    fn recent_workspace_payloads_use_camel_case_and_strict_params() {
        let info = RecentWorkspaceInfo {
            name: "demo".to_owned(),
            root: "/tmp/demo".to_owned(),
            last_opened_at: 42,
            pinned: true,
            available: false,
        };
        let value = serde_json::to_value(info).unwrap();
        assert_eq!(value["lastOpenedAt"], 42);
        assert_eq!(value["pinned"], true);

        assert!(serde_json::from_value::<RecentWorkspacesParams>(json!({})).is_ok());
        assert!(serde_json::from_value::<RecentWorkspacesParams>(json!({ "x": 1 })).is_err());
        assert!(
            serde_json::from_value::<RecentWorkspacePinParams>(json!({
                "root": "/tmp/demo",
                "pinned": true,
            }))
            .is_ok()
        );
        assert!(
            serde_json::from_value::<RecentWorkspaceRemoveParams>(json!({
                "root": "/tmp/demo",
                "extra": false,
            }))
            .is_err()
        );
    }

    #[test]
    fn workspace_browse_params_reject_unknown_fields() {
        let valid = serde_json::from_value::<WorkspaceBrowseParams>(json!({ "path": "/tmp" }));
        let invalid =
            serde_json::from_value::<WorkspaceBrowseParams>(json!({ "path": "/tmp", "x": 1 }));

        assert_eq!(valid.unwrap().path, "/tmp");
        assert!(invalid.is_err());
    }

    #[test]
    fn workspace_create_folder_params_reject_unknown_fields() {
        let valid = serde_json::from_value::<WorkspaceCreateFolderParams>(json!({
            "parent": "/tmp",
            "name": "demo",
        }));
        let invalid = serde_json::from_value::<WorkspaceCreateFolderParams>(json!({
            "parent": "/tmp",
            "name": "demo",
            "path": "/tmp/demo",
        }));

        assert_eq!(valid.unwrap().name, "demo");
        assert!(invalid.is_err());
    }

    #[test]
    fn workspace_project_template_serializes_camel_case() {
        let params = WorkspaceCreateProjectParams {
            parent: "/tmp".to_owned(),
            name: "demo".to_owned(),
            template: WorkspaceProjectTemplate::CppCmake,
        };
        let value = serde_json::to_value(params).unwrap();

        assert_eq!(value["template"], "cppCmake");
    }
}

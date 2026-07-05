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
    /// No known build system marker was found.
    Unknown,
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
}

/// Parameters for `workspace.open`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorkspaceOpenParams {
    /// Directory to open as workspace root.
    pub path: String,
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
        ProjectKind, WorkspaceBrowseParams, WorkspaceCreateFolderParams,
        WorkspaceCreateProjectParams, WorkspaceInfo, WorkspaceOpenParams, WorkspaceProjectTemplate,
    };

    #[test]
    fn workspace_info_serializes_camel_case_kind() {
        let workspace = WorkspaceInfo {
            name: "demo".to_owned(),
            root: "/home/user/demo".to_owned(),
            kind: ProjectKind::RustCargo,
            markers: vec!["Cargo.toml".to_owned()],
        };
        let value = serde_json::to_value(workspace).unwrap();

        assert_eq!(value["kind"], "rustCargo");
        assert_eq!(value["markers"][0], "Cargo.toml");
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

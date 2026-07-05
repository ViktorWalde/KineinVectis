//! Opening a workspace, browsing directories, and persisting metadata.

use std::{
    fs,
    path::{Path, PathBuf},
};

use kinein_protocol::{WorkspaceBrowseEntry, WorkspaceBrowseResult, WorkspaceInfo};
use serde::{Deserialize, Serialize};

use super::detect_project;
use super::{WORKSPACE_DIR, WORKSPACE_FILE, WORKSPACE_SCHEMA_VERSION, WorkspaceError};

/// On-disk representation of `.kinein/workspace.json`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PersistedWorkspace {
    /// Schema version of this file.
    schema_version: String,
    /// Workspace metadata as exposed over IPC.
    #[serde(flatten)]
    workspace: WorkspaceInfo,
}

/// Opens a directory as workspace and persists `.kinein/workspace.json`.
pub fn open_workspace(path: &Path) -> Result<WorkspaceInfo, WorkspaceError> {
    let root = fs::canonicalize(path).map_err(|source| WorkspaceError::InvalidRoot {
        path: path.display().to_string(),
        source,
    })?;

    if !root.is_dir() {
        return Err(WorkspaceError::NotADirectory {
            path: root.display().to_string(),
        });
    }

    let name = root.file_name().map_or_else(
        || root.display().to_string(),
        |file_name| file_name.to_string_lossy().into_owned(),
    );
    let (kind, markers) = detect_project(&root);
    let workspace = WorkspaceInfo {
        name,
        root: root.display().to_string(),
        kind,
        markers,
    };

    persist(&root, &workspace)?;

    Ok(workspace)
}

/// Lists child directories for the IDE-owned workspace picker.
pub fn browse_directories(path: &Path) -> Result<WorkspaceBrowseResult, WorkspaceError> {
    let directory = fs::canonicalize(path).map_err(|source| WorkspaceError::InvalidRoot {
        path: path.display().to_string(),
        source,
    })?;

    if !directory.is_dir() {
        return Err(WorkspaceError::NotADirectory {
            path: directory.display().to_string(),
        });
    }

    let read_dir = fs::read_dir(&directory).map_err(|source| WorkspaceError::ListDirectory {
        path: directory.display().to_string(),
        source,
    })?;

    let mut entries = Vec::new();
    for dir_entry in read_dir {
        let dir_entry = dir_entry.map_err(|source| WorkspaceError::ListDirectory {
            path: directory.display().to_string(),
            source,
        })?;
        let path = dir_entry.path();
        let metadata = dir_entry
            .metadata()
            .map_err(|source| WorkspaceError::ListDirectory {
                path: path.display().to_string(),
                source,
            })?;

        if metadata.is_dir() {
            let canonical_path =
                fs::canonicalize(&path).map_err(|source| WorkspaceError::ListDirectory {
                    path: path.display().to_string(),
                    source,
                })?;
            entries.push(WorkspaceBrowseEntry {
                name: dir_entry.file_name().to_string_lossy().into_owned(),
                path: canonical_path.display().to_string(),
            });
        }
    }

    entries.sort_by_key(|entry| entry.name.to_lowercase());

    let parent = directory
        .parent()
        .map(|parent| parent.display().to_string());

    Ok(WorkspaceBrowseResult {
        path: directory.display().to_string(),
        parent,
        entries,
    })
}

/// Returns the path of the persisted metadata file for a workspace root.
#[must_use]
pub fn metadata_path(root: &Path) -> PathBuf {
    root.join(WORKSPACE_DIR).join(WORKSPACE_FILE)
}

fn persist(root: &Path, workspace: &WorkspaceInfo) -> Result<(), WorkspaceError> {
    let directory = root.join(WORKSPACE_DIR);
    fs::create_dir_all(&directory).map_err(|source| WorkspaceError::Persist {
        path: directory.display().to_string(),
        source,
    })?;

    let persisted = PersistedWorkspace {
        schema_version: WORKSPACE_SCHEMA_VERSION.to_owned(),
        workspace: workspace.clone(),
    };
    let mut json = serde_json::to_string_pretty(&persisted).map_err(WorkspaceError::Serialize)?;
    json.push('\n');

    let file = metadata_path(root);
    fs::write(&file, json).map_err(|source| WorkspaceError::Persist {
        path: file.display().to_string(),
        source,
    })
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use kinein_protocol::ProjectKind;
    use serde_json::Value;

    use super::{browse_directories, metadata_path, open_workspace};

    fn temp_workspace(test_name: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("kinein-workspace-open-tests")
            .join(format!("{}-{test_name}", std::process::id()));
        if dir.exists() {
            fs::remove_dir_all(&dir).unwrap();
        }
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn open_workspace_persists_metadata_json() {
        let dir = temp_workspace("persist");
        fs::write(dir.join("pyproject.toml"), "[project]\n").unwrap();

        let workspace = open_workspace(&dir).unwrap();

        assert_eq!(workspace.kind, ProjectKind::Python);
        assert_eq!(
            workspace.root,
            dir.canonicalize().unwrap().display().to_string()
        );

        let raw = fs::read_to_string(metadata_path(&dir)).unwrap();
        let value = serde_json::from_str::<Value>(&raw).unwrap();
        assert_eq!(value["schemaVersion"], "0.1.0");
        assert_eq!(value["kind"], "python");
        assert_eq!(value["name"], dir.file_name().unwrap().to_str().unwrap());
    }

    #[test]
    fn open_workspace_rejects_missing_path() {
        let dir = temp_workspace("missing").join("does-not-exist");

        let error = open_workspace(&dir).unwrap_err();

        assert!(error.is_invalid_path());
        assert!(error.to_string().contains("does-not-exist"));
    }

    #[test]
    fn browse_directories_lists_only_directories() {
        let dir = temp_workspace("browse");
        fs::create_dir(dir.join("zeta")).unwrap();
        fs::create_dir(dir.join("Alpha")).unwrap();
        fs::write(dir.join("file.txt"), "x").unwrap();

        let result = browse_directories(&dir).unwrap();

        assert_eq!(
            result.path,
            dir.canonicalize().unwrap().display().to_string()
        );
        assert!(result.parent.is_some());
        let names = result
            .entries
            .iter()
            .map(|entry| entry.name.as_str())
            .collect::<Vec<_>>();
        assert_eq!(names, ["Alpha", "zeta"]);
    }

    #[test]
    fn browse_directories_rejects_files() {
        let dir = temp_workspace("browse-file");
        let file = dir.join("file.txt");
        fs::write(&file, "x").unwrap();

        let error = browse_directories(&file).unwrap_err();

        assert!(error.is_invalid_path());
        assert!(error.to_string().contains("nao e um diretorio"));
    }
}

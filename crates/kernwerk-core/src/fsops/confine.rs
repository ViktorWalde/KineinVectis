//! Path canonicalization and workspace-root confinement primitives.

use std::path::{Path, PathBuf};

use std::fs;

use super::FsError;

/// Canonicalizes `path` and ensures it stays inside `root`.
pub(super) fn confine(root: &Path, path: &Path) -> Result<PathBuf, FsError> {
    let canonical = fs::canonicalize(path).map_err(|source| FsError::InvalidPath {
        path: path.display().to_string(),
        source,
    })?;

    if !canonical.starts_with(root) {
        return Err(FsError::OutsideRoot {
            path: canonical.display().to_string(),
        });
    }

    Ok(canonical)
}

/// Canonicalizes a path and ensures it is an existing regular file inside the workspace root.
pub fn confine_file(root: &Path, path: &Path) -> Result<PathBuf, FsError> {
    let file = confine(root, path)?;

    if !file.is_file() {
        return Err(FsError::NotAFile {
            path: file.display().to_string(),
        });
    }

    Ok(file)
}

/// Resolves the parent of a would-be new child and joins its file name.
///
/// The parent must already exist and stay inside `root`; the child itself is
/// not required to exist yet.
pub(super) fn new_child_path(root: &Path, path: &Path) -> Result<PathBuf, FsError> {
    let parent = path.parent().ok_or_else(|| FsError::InvalidFileName {
        path: path.display().to_string(),
    })?;
    let file_name = path.file_name().ok_or_else(|| FsError::InvalidFileName {
        path: path.display().to_string(),
    })?;
    if file_name.is_empty() {
        return Err(FsError::InvalidFileName {
            path: path.display().to_string(),
        });
    }

    let directory = confine(root, parent)?;
    if !directory.is_dir() {
        return Err(FsError::NotADirectory {
            path: directory.display().to_string(),
        });
    }

    Ok(directory.join(file_name))
}

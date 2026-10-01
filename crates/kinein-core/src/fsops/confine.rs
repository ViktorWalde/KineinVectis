//! Path canonicalization and workspace-root confinement primitives.

use std::{
    io,
    path::{Path, PathBuf},
};

use std::fs;

use super::FsError;

/// Which transfer policy applies to a source/destination pair.
#[derive(Clone, Copy)]
pub(super) enum TransferKind {
    Copy,
    Move,
    Import,
}

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

/// Resolves an existing absolute workspace directory for a terminal session.
pub fn confine_directory(root: &Path, path: &Path) -> Result<PathBuf, FsError> {
    if !path.is_absolute() {
        return Err(FsError::InvalidFileName {
            path: path.display().to_string(),
        });
    }
    let directory = confine(root, path)?;
    if !directory.is_dir() {
        return Err(FsError::NotADirectory {
            path: directory.display().to_string(),
        });
    }
    Ok(directory)
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

/// Resolves one transfer pair before a copy or move touches the destination.
/// Batch planning calls the same rule for each pair before executing any one.
pub(super) fn transfer_paths(
    root: &Path,
    from: &Path,
    to: &Path,
    kind: TransferKind,
) -> Result<(PathBuf, PathBuf), FsError> {
    let source = if matches!(kind, TransferKind::Import) {
        if !from.is_absolute() {
            return Err(FsError::InvalidFileName {
                path: from.display().to_string(),
            });
        }
        from.to_path_buf()
    } else {
        confine(root, from)?
    };
    if source == root && !matches!(kind, TransferKind::Import) {
        return Err(FsError::WorkspaceRoot {
            path: source.display().to_string(),
        });
    }
    let target = new_child_path(root, to)?;
    if source.is_dir() && target.starts_with(&source) {
        return Err(FsError::InvalidFileName {
            path: target.display().to_string(),
        });
    }
    match fs::symlink_metadata(&target) {
        Ok(_) => {
            return Err(FsError::AlreadyExists {
                path: target.display().to_string(),
            });
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(FsError::Io {
                path: target.display().to_string(),
                source: error,
            });
        }
    }
    if matches!(kind, TransferKind::Copy | TransferKind::Import)
        && fs::symlink_metadata(from)
            .map_err(|source| FsError::Io {
                path: from.display().to_string(),
                source,
            })?
            .file_type()
            .is_symlink()
    {
        return Err(FsError::UnsupportedEntry {
            path: from.display().to_string(),
        });
    }
    Ok((source, target))
}

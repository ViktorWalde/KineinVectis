//! Basic file operations: list, read, create, write, rename and delete.

use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

use kernwerk_protocol::{FsEntry, FsEntryKind};

use super::confine::{confine, confine_file, new_child_path};
use super::{FsError, MAX_READ_BYTES};

/// Lists a directory inside the workspace root.
///
/// Entries are sorted directories-first, then case-insensitive by name.
pub fn list_dir(root: &Path, path: &Path) -> Result<(PathBuf, Vec<FsEntry>), FsError> {
    let directory = confine(root, path)?;

    if !directory.is_dir() {
        return Err(FsError::NotADirectory {
            path: directory.display().to_string(),
        });
    }

    let read_dir = fs::read_dir(&directory).map_err(|source| FsError::Io {
        path: directory.display().to_string(),
        source,
    })?;

    let mut entries = Vec::new();
    for dir_entry in read_dir {
        let dir_entry = dir_entry.map_err(|source| FsError::Io {
            path: directory.display().to_string(),
            source,
        })?;
        let metadata = dir_entry.metadata().map_err(|source| FsError::Io {
            path: dir_entry.path().display().to_string(),
            source,
        })?;

        let kind = if metadata.is_dir() {
            FsEntryKind::Directory
        } else if metadata.is_file() {
            FsEntryKind::File
        } else {
            FsEntryKind::Other
        };
        entries.push(FsEntry {
            name: dir_entry.file_name().to_string_lossy().into_owned(),
            kind,
            size: (kind == FsEntryKind::File).then_some(metadata.len()),
        });
    }

    entries.sort_by(|left, right| {
        let left_is_dir = left.kind == FsEntryKind::Directory;
        let right_is_dir = right.kind == FsEntryKind::Directory;
        right_is_dir
            .cmp(&left_is_dir)
            .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase()))
    });

    Ok((directory, entries))
}

/// Reads a UTF-8 text file inside the workspace root.
pub fn read_file(root: &Path, path: &Path) -> Result<(PathBuf, String), FsError> {
    let file = confine_file(root, path)?;

    let metadata = fs::metadata(&file).map_err(|source| FsError::Io {
        path: file.display().to_string(),
        source,
    })?;
    if metadata.len() > MAX_READ_BYTES {
        return Err(FsError::TooLarge {
            path: file.display().to_string(),
            size: metadata.len(),
        });
    }

    let bytes = fs::read(&file).map_err(|source| FsError::Io {
        path: file.display().to_string(),
        source,
    })?;
    let content = String::from_utf8(bytes).map_err(|_utf8_error| FsError::NotText {
        path: file.display().to_string(),
    })?;

    Ok((file, content))
}

/// Creates a new UTF-8 text file inside the workspace root.
///
/// Parent directories must already exist and the operation never overwrites an
/// existing file.
pub fn create_file(root: &Path, path: &Path, content: &str) -> Result<(PathBuf, u64), FsError> {
    let file = new_child_path(root, path)?;
    if file.exists() {
        return Err(FsError::AlreadyExists {
            path: file.display().to_string(),
        });
    }

    let mut handle = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&file)
        .map_err(|source| {
            if source.kind() == io::ErrorKind::AlreadyExists {
                FsError::AlreadyExists {
                    path: file.display().to_string(),
                }
            } else {
                FsError::Io {
                    path: file.display().to_string(),
                    source,
                }
            }
        })?;
    handle
        .write_all(content.as_bytes())
        .map_err(|source| FsError::Io {
            path: file.display().to_string(),
            source,
        })?;

    Ok((file, content.len() as u64))
}

/// Creates a new directory inside the workspace root.
///
/// The parent directory must already exist and the operation never overwrites
/// an existing path.
pub fn create_directory(root: &Path, path: &Path) -> Result<PathBuf, FsError> {
    let directory = new_child_path(root, path)?;
    if directory.exists() {
        return Err(FsError::AlreadyExists {
            path: directory.display().to_string(),
        });
    }

    fs::create_dir(&directory).map_err(|source| {
        if source.kind() == io::ErrorKind::AlreadyExists {
            FsError::AlreadyExists {
                path: directory.display().to_string(),
            }
        } else {
            FsError::Io {
                path: directory.display().to_string(),
                source,
            }
        }
    })?;

    Ok(directory)
}

/// Overwrites an existing UTF-8 text file inside the workspace root.
///
/// This function only saves files that already exist; use [`create_file`] for
/// explicit new-file creation.
pub fn write_file(root: &Path, path: &Path, content: &str) -> Result<(PathBuf, u64), FsError> {
    let file = confine_file(root, path)?;

    fs::write(&file, content).map_err(|source| FsError::Io {
        path: file.display().to_string(),
        source,
    })?;

    Ok((file, content.len() as u64))
}

/// Renames or moves a file or directory inside the workspace root.
///
/// `from` must exist and stay inside the workspace. `to` must resolve to a new
/// child path whose parent already exists inside the workspace and must not
/// already exist. The workspace root itself cannot be renamed. Returns the
/// canonical source and destination paths.
pub fn rename(root: &Path, from: &Path, to: &Path) -> Result<(PathBuf, PathBuf), FsError> {
    let source = confine(root, from)?;
    if source == root {
        return Err(FsError::WorkspaceRoot {
            path: source.display().to_string(),
        });
    }

    let target = new_child_path(root, to)?;
    if target.exists() {
        return Err(FsError::AlreadyExists {
            path: target.display().to_string(),
        });
    }

    fs::rename(&source, &target).map_err(|io_error| FsError::Io {
        path: source.display().to_string(),
        source: io_error,
    })?;

    Ok((source, target))
}

/// Deletes a file or directory inside the workspace root.
///
/// Directories are removed recursively. The path must stay inside the workspace
/// and cannot be the workspace root itself. Returns the canonical deleted path.
pub fn delete(root: &Path, path: &Path) -> Result<PathBuf, FsError> {
    let target = confine(root, path)?;
    if target == root {
        return Err(FsError::WorkspaceRoot {
            path: target.display().to_string(),
        });
    }

    if target.is_dir() {
        fs::remove_dir_all(&target)
    } else {
        fs::remove_file(&target)
    }
    .map_err(|io_error| FsError::Io {
        path: target.display().to_string(),
        source: io_error,
    })?;

    Ok(target)
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use kernwerk_protocol::FsEntryKind;

    use super::super::{FsError, MAX_READ_BYTES};
    use super::{delete, list_dir, read_file, rename, write_file};

    fn temp_root(test_name: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("kernwerk-fsops-ops-tests")
            .join(format!("{}-{test_name}", std::process::id()));
        if dir.exists() {
            fs::remove_dir_all(&dir).unwrap();
        }
        fs::create_dir_all(&dir).unwrap();
        dir.canonicalize().unwrap()
    }

    #[test]
    fn list_dir_sorts_directories_first() {
        let root = temp_root("list");
        fs::create_dir(root.join("zeta-dir")).unwrap();
        fs::write(root.join("alpha.txt"), "a").unwrap();
        fs::write(root.join("Beta.txt"), "b").unwrap();

        let (path, entries) = list_dir(&root, &root).unwrap();

        assert_eq!(path, root);
        let names = entries
            .iter()
            .map(|entry| entry.name.as_str())
            .collect::<Vec<_>>();
        assert_eq!(names, ["zeta-dir", "alpha.txt", "Beta.txt"]);
        assert_eq!(entries[0].kind, FsEntryKind::Directory);
        assert_eq!(entries[1].size, Some(1));
    }

    #[test]
    fn paths_outside_root_are_rejected() {
        let root = temp_root("outside");

        let error = list_dir(&root, &std::env::temp_dir()).unwrap_err();

        assert!(matches!(error, FsError::OutsideRoot { .. }));
        assert!(error.is_invalid_path());
    }

    #[test]
    fn dot_dot_traversal_is_rejected() {
        let root = temp_root("traversal");
        let sneaky = root.join("..").join("..");

        let error = list_dir(&root, &sneaky).unwrap_err();

        assert!(matches!(error, FsError::OutsideRoot { .. }));
    }

    #[test]
    fn read_and_write_roundtrip() {
        let root = temp_root("roundtrip");
        let file = root.join("main.rs");
        fs::write(&file, "fn main() {}\n").unwrap();

        let (path, content) = read_file(&root, &file).unwrap();
        assert_eq!(content, "fn main() {}\n");

        let (_, bytes) = write_file(&root, &path, "fn main() { println!(); }\n").unwrap();
        assert_eq!(bytes, 26);

        let (_, reread) = read_file(&root, &file).unwrap();
        assert_eq!(reread, "fn main() { println!(); }\n");
    }

    #[test]
    fn read_rejects_oversized_files() {
        let root = temp_root("oversized");
        let file = root.join("big.bin");
        fs::write(
            &file,
            vec![b'x'; usize::try_from(MAX_READ_BYTES).unwrap() + 1],
        )
        .unwrap();

        let error = read_file(&root, &file).unwrap_err();

        assert!(matches!(error, FsError::TooLarge { .. }));
    }

    #[test]
    fn read_rejects_binary_files() {
        let root = temp_root("binary");
        let file = root.join("blob.bin");
        fs::write(&file, [0xFF, 0xFE, 0x00, 0x81]).unwrap();

        let error = read_file(&root, &file).unwrap_err();

        assert!(matches!(error, FsError::NotText { .. }));
    }

    #[test]
    fn write_refuses_to_create_new_files() {
        let root = temp_root("no-create");

        let error = write_file(&root, &root.join("novo.txt"), "x").unwrap_err();

        assert!(matches!(error, FsError::InvalidPath { .. }));
    }

    #[test]
    fn rename_moves_file_within_workspace() {
        let root = temp_root("rename-file");
        fs::create_dir(root.join("src")).unwrap();
        fs::write(root.join("old.rs"), "conteudo\n").unwrap();

        let (from, to) = rename(&root, &root.join("old.rs"), &root.join("src/new.rs")).unwrap();

        assert_eq!(from, root.join("old.rs"));
        assert!(to.starts_with(&root));
        assert!(to.ends_with("new.rs"));
        assert!(!root.join("old.rs").exists());
        assert_eq!(
            fs::read_to_string(root.join("src/new.rs")).unwrap(),
            "conteudo\n"
        );
    }

    #[test]
    fn rename_refuses_to_clobber_existing_target() {
        let root = temp_root("rename-clobber");
        fs::write(root.join("a.rs"), "a\n").unwrap();
        fs::write(root.join("b.rs"), "b\n").unwrap();

        let error = rename(&root, &root.join("a.rs"), &root.join("b.rs")).unwrap_err();

        assert!(matches!(error, FsError::AlreadyExists { .. }));
        assert_eq!(fs::read_to_string(root.join("a.rs")).unwrap(), "a\n");
    }

    #[test]
    fn rename_rejects_source_outside_root() {
        let root = temp_root("rename-outside");

        let error = rename(&root, &std::env::temp_dir(), &root.join("here")).unwrap_err();

        assert!(matches!(error, FsError::OutsideRoot { .. }));
    }

    #[test]
    fn rename_rejects_the_workspace_root() {
        let root = temp_root("rename-root");

        let error = rename(&root, &root, &root.join("renamed")).unwrap_err();

        assert!(matches!(error, FsError::WorkspaceRoot { .. }));
        assert!(error.is_invalid_path());
    }

    #[test]
    fn delete_removes_a_file() {
        let root = temp_root("delete-file");
        fs::write(root.join("gone.rs"), "x\n").unwrap();

        let path = delete(&root, &root.join("gone.rs")).unwrap();

        assert!(path.ends_with("gone.rs"));
        assert!(!root.join("gone.rs").exists());
    }

    #[test]
    fn delete_removes_a_directory_recursively() {
        let root = temp_root("delete-dir");
        fs::create_dir_all(root.join("mod/inner")).unwrap();
        fs::write(root.join("mod/inner/a.rs"), "a\n").unwrap();

        delete(&root, &root.join("mod")).unwrap();

        assert!(!root.join("mod").exists());
    }

    #[test]
    fn delete_rejects_paths_outside_root() {
        let root = temp_root("delete-outside");

        let error = delete(&root, &std::env::temp_dir()).unwrap_err();

        assert!(matches!(error, FsError::OutsideRoot { .. }));
    }

    #[test]
    fn delete_rejects_the_workspace_root() {
        let root = temp_root("delete-root");

        let error = delete(&root, &root).unwrap_err();

        assert!(matches!(error, FsError::WorkspaceRoot { .. }));
    }
}

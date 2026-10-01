//! Basic file operations: list, read, create, write, rename and delete.

use std::{
    fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

use kinein_protocol::{FsEntry, FsEntryKind};

use super::confine::{TransferKind, confine, confine_file, new_child_path, transfer_paths};
use super::copy_ops::open_source;
use super::publish::{publish_noreplace, temp_sibling};
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
    let content = read_utf8_regular(&file)?;
    Ok((file, content))
}

/// Reads one explicitly selected local file for an external, read-only tab.
/// No workspace mutation, watcher, or LSP state is created by this operation.
pub fn read_external_file(path: &Path) -> Result<(PathBuf, String), FsError> {
    let content = read_utf8_regular(path)?;
    Ok((path.to_path_buf(), content))
}

fn read_utf8_regular(path: &Path) -> Result<String, FsError> {
    // Shared with import: descriptor-relative traversal rejects symlinked
    // components, and NONBLOCK prevents a replaced FIFO from hanging the UI.
    let handle = open_source(path)?;
    let metadata = handle.metadata().map_err(|source| FsError::Io {
        path: path.display().to_string(),
        source,
    })?;
    if !metadata.is_file() {
        return Err(FsError::NotAFile {
            path: path.display().to_string(),
        });
    }
    if metadata.len() > MAX_READ_BYTES {
        return Err(FsError::TooLarge {
            path: path.display().to_string(),
            size: metadata.len(),
        });
    }

    let mut bytes = Vec::new();
    handle
        .take(MAX_READ_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|source| FsError::Io {
            path: path.display().to_string(),
            source,
        })?;
    if bytes.len() as u64 > MAX_READ_BYTES {
        return Err(FsError::TooLarge {
            path: path.display().to_string(),
            size: bytes.len() as u64,
        });
    }
    let content = String::from_utf8(bytes).map_err(|_utf8_error| FsError::NotText {
        path: path.display().to_string(),
    })?;
    Ok(content)
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
    atomic_write(&file, content.as_bytes())?;
    Ok((file, content.len() as u64))
}

/// Atomically overwrites a UTF-8 file only when its disk content still
/// matches the snapshot last observed by the caller.
pub fn write_file_if_unchanged(
    root: &Path,
    path: &Path,
    content: &str,
    expected_content: &str,
) -> Result<(PathBuf, u64), FsError> {
    let file = confine_file(root, path)?;
    let current = fs::read(&file).map_err(|source| FsError::Io {
        path: file.display().to_string(),
        source,
    })?;
    if current != expected_content.as_bytes() {
        return Err(FsError::ChangedOnDisk {
            path: file.display().to_string(),
        });
    }
    atomic_write(&file, content.as_bytes())?;
    Ok((file, content.len() as u64))
}

/// Grava `bytes` em `target` de forma ATÔMICA (rede de segurança da fatia S1,
/// ver `DocsPublic/seguranca/23`): escreve num arquivo temporário no MESMO diretório, faz
/// `fsync`, e `rename` por cima do alvo. Como o `rename` no mesmo filesystem
/// é atômico, um crash/kill no meio da escrita nunca deixa o alvo truncado ou
/// zerado — ele fica com o conteúdo ANTIGO ou o NOVO, jamais pela metade.
fn atomic_write(target: &Path, bytes: &[u8]) -> Result<(), FsError> {
    let io_err = |path: &Path, source: io::Error| FsError::Io {
        path: path.display().to_string(),
        source,
    };
    let temp = temp_sibling(target);

    let mut file = fs::File::create(&temp).map_err(|source| io_err(&temp, source))?;
    if let Err(source) = file.write_all(bytes) {
        drop(fs::remove_file(&temp));
        return Err(io_err(&temp, source));
    }
    // `fsync` garante que os bytes chegaram ao disco antes do rename.
    if let Err(source) = file.sync_all() {
        drop(fs::remove_file(&temp));
        return Err(io_err(&temp, source));
    }
    drop(file); // fecha o handle antes do rename.

    fs::rename(&temp, target).map_err(|source| {
        drop(fs::remove_file(&temp));
        io_err(target, source)
    })
}

/// Renames or moves a file or directory inside the workspace root.
///
/// `from` must exist and stay inside the workspace. `to` must resolve to a new
/// child path whose parent already exists inside the workspace and must not
/// already exist. The workspace root itself cannot be renamed. Returns the
/// canonical source and destination paths.
pub fn rename(root: &Path, from: &Path, to: &Path) -> Result<(PathBuf, PathBuf), FsError> {
    let (source, target) = transfer_paths(root, from, to, TransferKind::Move)?;
    publish_noreplace(&source, &target)?;

    Ok((source, target))
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        sync::atomic::{AtomicBool, Ordering},
    };

    use kinein_protocol::FsEntryKind;

    use super::super::{FsError, MAX_READ_BYTES, copy, copy_with_progress, delete};
    use super::{list_dir, read_file, rename, write_file, write_file_if_unchanged};

    #[test]
    fn copy_preserves_binary_file_and_refuses_collision() {
        let root = temp_root("copy-binary");
        let source = root.join("imagem.bin");
        let target = root.join("imagem-copia.bin");
        fs::write(&source, [0, 255, 31, 128]).unwrap();

        assert_eq!(
            copy(&root, &source, &target).unwrap(),
            (source.clone(), target.clone())
        );
        assert_eq!(fs::read(&target).unwrap(), [0, 255, 31, 128]);
        assert_eq!(fs::read(&source).unwrap(), [0, 255, 31, 128]);

        let error = copy(&root, &source, &target).unwrap_err();
        assert!(matches!(error, FsError::AlreadyExists { .. }));
        assert_eq!(fs::read(&target).unwrap(), [0, 255, 31, 128]);
    }

    #[test]
    fn copy_directory_refuses_descendant_and_nested_symlink() {
        let root = temp_root("copy-directory");
        let source = root.join("src");
        fs::create_dir_all(source.join("sub")).unwrap();
        fs::write(source.join("sub/main.rs"), b"fn main() {}\n").unwrap();

        let target = root.join("src-copia");
        copy(&root, &source, &target).unwrap();
        assert_eq!(
            fs::read(target.join("sub/main.rs")).unwrap(),
            b"fn main() {}\n"
        );

        let error = copy(&root, &source, &source.join("sub/loop")).unwrap_err();
        assert!(matches!(error, FsError::InvalidFileName { .. }));
        assert!(!source.join("sub/loop").exists());

        std::os::unix::fs::symlink(root.join("outside"), source.join("sub/link")).unwrap();
        let blocked = root.join("bloqueada");
        let error = copy(&root, &source, &blocked).unwrap_err();
        assert!(matches!(error, FsError::UnsupportedEntry { .. }));
        assert!(!blocked.exists(), "nao publicar pasta parcialmente copiada");
        assert!(
            fs::read_dir(&root).unwrap().all(|entry| !entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .contains("kinein-tmp")),
            "nao deixar staging depois da falha"
        );
    }

    #[test]
    fn copy_cancelled_mid_file_removes_staging_and_keeps_source() {
        let root = temp_root("copy-cancelled");
        let source = root.join("grande.bin");
        let target = root.join("cancelada.bin");
        fs::write(&source, vec![42_u8; 2 * 1024 * 1024]).unwrap();
        let cancelled = AtomicBool::new(false);
        let mut progress = 0;

        let error = copy_with_progress(
            &root,
            &source,
            &target,
            || cancelled.load(Ordering::SeqCst),
            |done, total| {
                assert_eq!(total, 2 * 1024 * 1024);
                progress = done;
                cancelled.store(true, Ordering::SeqCst);
            },
        )
        .unwrap_err();
        assert!(matches!(error, FsError::Cancelled { .. }));
        assert!(progress > 0);
        assert_eq!(fs::metadata(&source).unwrap().len(), 2 * 1024 * 1024);
        assert!(!target.exists());
        assert!(fs::read_dir(&root).unwrap().all(|entry| {
            !entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .contains("kinein-tmp")
        }));
    }

    #[test]
    fn rename_does_not_replace_dangling_symlink() {
        let root = temp_root("rename-symlink-collision");
        let source = root.join("origem.txt");
        let target = root.join("destino.txt");
        fs::write(&source, "preservar origem").unwrap();
        std::os::unix::fs::symlink(root.join("ausente.txt"), &target).unwrap();

        let error = rename(&root, &source, &target).unwrap_err();
        assert!(matches!(error, FsError::AlreadyExists { .. }));
        assert_eq!(fs::read_to_string(&source).unwrap(), "preservar origem");
        assert!(
            fs::symlink_metadata(&target)
                .unwrap()
                .file_type()
                .is_symlink()
        );
    }

    #[test]
    fn rename_directory_into_its_descendant_is_rejected_before_mutation() {
        let root = temp_root("rename-descendant");
        let source = root.join("src");
        let child = source.join("child");
        fs::create_dir_all(&child).unwrap();
        fs::write(child.join("keep.txt"), "preservar").unwrap();

        let error = rename(&root, &source, &child.join("moved")).unwrap_err();
        assert!(matches!(error, FsError::InvalidFileName { .. }));
        assert_eq!(
            fs::read_to_string(child.join("keep.txt")).unwrap(),
            "preservar"
        );
        assert!(!child.join("moved").exists());
    }

    fn temp_root(test_name: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("kinein-fsops-ops-tests")
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
    fn write_is_atomic_and_leaves_no_temp_behind() {
        // Escrita atômica (S1/docs/seguranca/23): substitui o conteúdo e não deixa
        // nenhum arquivo temporário `.kinein-tmp-*` no diretório.
        let root = temp_root("atomic");
        let file = root.join("data.txt");
        fs::write(&file, "antigo\n").unwrap();

        write_file(&root, &file, "novo conteudo\n").unwrap();

        let (_, reread) = read_file(&root, &file).unwrap();
        assert_eq!(reread, "novo conteudo\n");

        let leftovers: Vec<_> = fs::read_dir(&root)
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_name().to_string_lossy().contains("kinein-tmp"))
            .collect();
        assert!(leftovers.is_empty(), "temp não foi renomeado/limpo");
    }

    #[test]
    fn conditional_write_rejects_external_change() {
        let root = temp_root("conditional-conflict");
        let file = root.join("data.txt");
        fs::write(&file, "primeira versao\n").unwrap();

        let error = write_file_if_unchanged(&root, &file, "buffer local\n", "snapshot antigo\n")
            .unwrap_err();

        assert!(matches!(error, FsError::ChangedOnDisk { .. }));
        assert_eq!(fs::read_to_string(&file).unwrap(), "primeira versao\n");
    }

    #[test]
    fn conditional_write_accepts_matching_snapshot() {
        let root = temp_root("conditional-match");
        let file = root.join("data.txt");
        fs::write(&file, "snapshot\n").unwrap();

        let (_, bytes) =
            write_file_if_unchanged(&root, &file, "buffer local\n", "snapshot\n").unwrap();

        assert_eq!(bytes, 13);
        assert_eq!(fs::read_to_string(&file).unwrap(), "buffer local\n");
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

//! Workspace-confined copy, with one motor for direct calls and cancelable jobs.

use std::{
    ffi::{OsStr, OsString},
    fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

use crate::platform::{self, NoFollow};

use super::FsError;
use super::confine::{TransferKind, transfer_paths};
use super::publish::{publish_noreplace, temp_sibling};

/// Copies one regular file or directory without replacing the destination.
pub fn copy(root: &Path, from: &Path, to: &Path) -> Result<(PathBuf, PathBuf), FsError> {
    copy_with_progress(root, from, to, || false, |_, _| {})
}

/// The same copy motor with cooperative cancellation and byte progress.
///
/// The source is scanned before staging begins. The scan rejects symlinks and
/// special entries, counts bytes and checks cancellation. The copy checks
/// cancellation between chunks; a failed or canceled attempt removes staging.
pub fn copy_with_progress<C, P>(
    root: &Path,
    from: &Path,
    to: &Path,
    is_cancelled: C,
    report: P,
) -> Result<(PathBuf, PathBuf), FsError>
where
    C: Fn() -> bool,
    P: FnMut(u64, u64),
{
    copy_with_policy(root, from, to, TransferKind::Copy, is_cancelled, report)
}

/// Imports a local source selected outside the workspace through the same
/// staging and no-replace copy motor. The source is never moved or removed.
pub(super) fn import_with_progress<C, P>(
    root: &Path,
    from: &Path,
    to: &Path,
    is_cancelled: C,
    report: P,
) -> Result<(PathBuf, PathBuf), FsError>
where
    C: Fn() -> bool,
    P: FnMut(u64, u64),
{
    copy_with_policy(root, from, to, TransferKind::Import, is_cancelled, report)
}

fn copy_with_policy<C, P>(
    root: &Path,
    from: &Path,
    to: &Path,
    kind: TransferKind,
    is_cancelled: C,
    mut report: P,
) -> Result<(PathBuf, PathBuf), FsError>
where
    C: Fn() -> bool,
    P: FnMut(u64, u64),
{
    let (source, target) = transfer_paths(root, from, to, kind)?;
    let descriptor = open_source(&source)?;

    let total = scan_open_entry(&descriptor, &source, 0, &is_cancelled)?;
    check_cancelled(&source, &is_cancelled)?;
    let staging = temp_sibling(&target);
    let mut completed = 0;
    if let Err(error) = copy_entry(
        &descriptor,
        &source,
        &staging,
        0,
        &is_cancelled,
        &mut report,
        &mut completed,
        total,
    ) {
        remove_staging(&staging);
        return Err(error);
    }
    if let Err(error) = check_cancelled(&source, &is_cancelled) {
        remove_staging(&staging);
        return Err(error);
    }
    if let Err(error) = publish_noreplace(&staging, &target) {
        remove_staging(&staging);
        return Err(error);
    }
    report(total, total);
    Ok((source, target))
}

fn check_cancelled<C: Fn() -> bool>(path: &Path, is_cancelled: &C) -> Result<(), FsError> {
    if is_cancelled() {
        Err(FsError::Cancelled {
            path: path.display().to_string(),
        })
    } else {
        Ok(())
    }
}

pub(super) fn open_source(source: &Path) -> Result<fs::File, FsError> {
    if !source.is_absolute() {
        return Err(FsError::InvalidFileName {
            path: source.display().to_string(),
        });
    }
    platform::open_nofollow(source).map_err(|error| nofollow_error(error, source))
}

/// Abre o filho `name` da pasta `parent`, aberta em `parent_path`.
fn open_child(
    parent: &fs::File,
    parent_path: &Path,
    name: &OsStr,
    path: &Path,
) -> Result<fs::File, FsError> {
    platform::open_child_nofollow(parent, parent_path, name)
        .map_err(|error| nofollow_error(error, path))
}

fn nofollow_error(error: NoFollow, path: &Path) -> FsError {
    let path = path.display().to_string();
    match error {
        NoFollow::Link => FsError::UnsupportedEntry { path },
        NoFollow::InvalidPath => FsError::InvalidFileName { path },
        NoFollow::Io(source) => FsError::Io { path, source },
    }
}

fn copyable_metadata<C: Fn() -> bool>(
    descriptor: &fs::File,
    source: &Path,
    depth: usize,
    cancel: &C,
) -> Result<fs::Metadata, FsError> {
    check_cancelled(source, cancel)?;
    if depth > 256 {
        return Err(FsError::InvalidPath {
            path: source.display().to_string(),
            source: io::Error::other("copia excedeu 256 niveis de pastas"),
        });
    }
    let metadata = descriptor.metadata().map_err(|error| FsError::Io {
        path: source.display().to_string(),
        source: error,
    })?;
    if !metadata.is_file() && !metadata.is_dir() {
        return Err(FsError::UnsupportedEntry {
            path: source.display().to_string(),
        });
    }
    Ok(metadata)
}

pub(super) fn scan_entry<C: Fn() -> bool>(
    source: &Path,
    depth: usize,
    cancel: &C,
) -> Result<u64, FsError> {
    let descriptor = open_source(source)?;
    scan_open_entry(&descriptor, source, depth, cancel)
}

fn scan_open_entry<C: Fn() -> bool>(
    descriptor: &fs::File,
    source: &Path,
    depth: usize,
    cancel: &C,
) -> Result<u64, FsError> {
    let metadata = copyable_metadata(descriptor, source, depth, cancel)?;
    if metadata.is_file() {
        return Ok(metadata.len());
    }
    let mut total = 0u64;
    for name in child_names(descriptor, source)? {
        let child_path = source.join(&name);
        let child = open_child(descriptor, source, &name, &child_path)?;
        total = total.saturating_add(scan_open_entry(&child, &child_path, depth + 1, cancel)?);
    }
    Ok(total)
}

/// Os nomes dentro de um diretorio aberto, sem `.` e `..`. So' os NOMES: o
/// chamador abre um filho de cada vez, e uma pasta enorme nao acumula
/// descritores abertos.
fn child_names(descriptor: &fs::File, source: &Path) -> Result<Vec<OsString>, FsError> {
    platform::child_names(descriptor, source).map_err(|error| FsError::Io {
        path: source.display().to_string(),
        source: error,
    })
}

#[allow(clippy::too_many_arguments)]
fn copy_entry<C, P>(
    descriptor: &fs::File,
    source: &Path,
    target: &Path,
    depth: usize,
    cancel: &C,
    report: &mut P,
    completed: &mut u64,
    total: u64,
) -> Result<(), FsError>
where
    C: Fn() -> bool,
    P: FnMut(u64, u64),
{
    let metadata = copyable_metadata(descriptor, source, depth, cancel)?;
    if metadata.is_file() {
        let mut input = descriptor;
        let mut output = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(target)
            .map_err(|error| FsError::Io {
                path: target.display().to_string(),
                source: error,
            })?;
        let mut buffer = vec![0_u8; 256 * 1024];
        loop {
            check_cancelled(source, cancel)?;
            let count = input.read(&mut buffer).map_err(|error| FsError::Io {
                path: source.display().to_string(),
                source: error,
            })?;
            if count == 0 {
                break;
            }
            output
                .write_all(&buffer[..count])
                .map_err(|error| FsError::Io {
                    path: target.display().to_string(),
                    source: error,
                })?;
            *completed = completed.saturating_add(count as u64);
            report(*completed, total);
        }
        output.sync_all().map_err(|error| FsError::Io {
            path: target.display().to_string(),
            source: error,
        })?;
    } else {
        fs::create_dir(target).map_err(|error| FsError::Io {
            path: target.display().to_string(),
            source: error,
        })?;
        for name in child_names(descriptor, source)? {
            let child_path = source.join(&name);
            let child = open_child(descriptor, source, &name, &child_path)?;
            copy_entry(
                &child,
                &child_path,
                &target.join(&name),
                depth + 1,
                cancel,
                report,
                completed,
                total,
            )?;
        }
    }
    fs::set_permissions(target, metadata.permissions()).map_err(|error| FsError::Io {
        path: target.display().to_string(),
        source: error,
    })?;
    Ok(())
}

fn remove_staging(path: &Path) {
    if path.is_dir() {
        make_directories_writable(path);
        drop(fs::remove_dir_all(path));
    } else {
        drop(fs::remove_file(path));
    }
}

fn make_directories_writable(path: &Path) {
    let Ok(metadata) = fs::symlink_metadata(path) else {
        return;
    };
    if !metadata.is_dir() {
        return;
    }
    drop(fs::set_permissions(
        path,
        platform::owner_writable(metadata.permissions()),
    ));
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            make_directories_writable(&entry.path());
        }
    }
}

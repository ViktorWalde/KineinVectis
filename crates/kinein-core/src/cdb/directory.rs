//! Identity of include-search namespaces, including previously absent headers.

use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, path::Path, time::Instant};

pub(super) fn signature(root: &Path, deadline: Instant) -> Option<String> {
    if !root.exists() {
        return Some("absent".into());
    }
    let mut pending = vec![root.to_path_buf()];
    let mut seen = BTreeSet::new();
    let mut hasher = Sha256::new();
    let mut count = 0_usize;
    while let Some(directory) = pending.pop() {
        if Instant::now() >= deadline {
            return None;
        }
        let real = crate::platform::canonicalize(&directory).ok()?;
        if !seen.insert(real) {
            continue;
        }
        let mut entries = std::fs::read_dir(&directory)
            .ok()?
            .collect::<Result<Vec<_>, _>>()
            .ok()?;
        entries.sort_by_key(std::fs::DirEntry::file_name);
        for entry in entries {
            count += 1;
            if count > 200_000 || Instant::now() >= deadline {
                return None;
            }
            let path = entry.path();
            hasher.update(path.as_os_str().as_encoded_bytes());
            let kind = entry.file_type().ok()?;
            hasher.update([u8::from(kind.is_dir()), u8::from(kind.is_symlink()), 0]);
            if kind.is_symlink() {
                hasher.update(
                    std::fs::read_link(&path)
                        .ok()?
                        .as_os_str()
                        .as_encoded_bytes(),
                );
            }
            if kind.is_dir() || (kind.is_symlink() && path.is_dir()) {
                pending.push(path);
            }
        }
    }
    Some(super::verification::hex(hasher.finalize().as_ref()))
}

pub(super) fn covered(paths: impl Iterator<Item = impl AsRef<Path>>, path: &Path) -> bool {
    paths
        .into_iter()
        .any(|root| path.starts_with(root.as_ref()))
}

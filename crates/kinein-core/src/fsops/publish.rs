//! Shared staging and no-replace publication for workspace file mutations.

use std::{
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use super::FsError;

/// Hidden, unique sibling on the same filesystem as the destination.
pub(super) fn temp_sibling(target: &Path) -> PathBuf {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
    let name = target
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("file");
    target.with_file_name(format!(".{name}.kinein-tmp-{}-{seq}", std::process::id()))
}

/// Atomically publishes `source` at `target`, refusing even a late collision.
pub(super) fn publish_noreplace(source: &Path, target: &Path) -> Result<(), FsError> {
    crate::platform::rename_noreplace(source, target).map_err(|error| {
        if error.kind() == std::io::ErrorKind::AlreadyExists {
            FsError::AlreadyExists {
                path: target.display().to_string(),
            }
        } else {
            FsError::Io {
                path: target.display().to_string(),
                source: error,
            }
        }
    })
}

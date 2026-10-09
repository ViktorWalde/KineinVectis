//! Batch orchestration over the existing copy and rename motors.

use std::path::{Path, PathBuf};

use kinein_protocol::{
    FsCopyParams, FsTransferBatchParams, FsTransferBatchResult, FsTransferItemResult,
    FsTransferOperation, FsTransferStatus,
};

use super::FsError;
use super::confine::{TransferKind, transfer_paths};
use super::copy_ops::{copy_with_progress, import_with_progress, scan_entry};
use super::ops::rename;

const MAX_TRANSFER_ITEMS: usize = 128;

/// Validates the entire set before touching any destination, then processes it
/// in request order. Runtime failures are reported per item; the batch does not
/// promise rollback of already completed items.
pub fn transfer_batch_with_progress<C, P>(
    root: &Path,
    params: &FsTransferBatchParams,
    is_cancelled: C,
    mut report: P,
) -> Result<FsTransferBatchResult, FsError>
where
    C: Fn() -> bool,
    P: FnMut(usize, usize, u64, u64),
{
    let planned = plan_batch(root, params)?;
    let count = planned.len();
    let mut items = Vec::with_capacity(count);
    let mut cancelled = false;

    for (index, ((source, target), requested)) in planned.iter().zip(&params.items).enumerate() {
        if is_cancelled() {
            cancelled = true;
            items.extend(params.items[index..].iter().map(not_started));
            break;
        }
        let outcome = match params.operation {
            FsTransferOperation::Copy => {
                copy_with_progress(root, source, target, &is_cancelled, |done, total| {
                    report(index, count, done, total);
                })
            }
            FsTransferOperation::Move => rename(root, source, target),
            FsTransferOperation::Import => {
                import_with_progress(root, source, target, &is_cancelled, |done, total| {
                    report(index, count, done, total);
                })
            }
        };
        let (status, error) = match outcome {
            Ok(_) => (FsTransferStatus::Success, None),
            Err(FsError::Cancelled { path }) => {
                cancelled = true;
                (
                    FsTransferStatus::Cancelled,
                    Some(format!("copia cancelada: {path}")),
                )
            }
            Err(error) => (FsTransferStatus::Failed, Some(error.to_string())),
        };
        items.push(FsTransferItemResult {
            from: requested.from.clone(),
            to: requested.to.clone(),
            status,
            error,
        });
        if cancelled {
            items.extend(params.items[index + 1..].iter().map(not_started));
            break;
        }
        report(index + 1, count, 0, 0);
    }
    Ok(FsTransferBatchResult {
        operation: params.operation,
        items,
        cancelled,
    })
}

fn not_started(requested: &FsCopyParams) -> FsTransferItemResult {
    FsTransferItemResult {
        from: requested.from.clone(),
        to: requested.to.clone(),
        status: FsTransferStatus::NotStarted,
        error: None,
    }
}

fn plan_batch(
    root: &Path,
    params: &FsTransferBatchParams,
) -> Result<Vec<(PathBuf, PathBuf)>, FsError> {
    if params.items.is_empty() || params.items.len() > MAX_TRANSFER_ITEMS {
        return Err(FsError::InvalidBatch {
            message: format!("o lote requer entre 1 e {MAX_TRANSFER_ITEMS} itens"),
        });
    }
    let kind = match params.operation {
        FsTransferOperation::Copy => TransferKind::Copy,
        FsTransferOperation::Move => TransferKind::Move,
        FsTransferOperation::Import => TransferKind::Import,
    };
    let mut planned = Vec::with_capacity(params.items.len());
    for requested in &params.items {
        let pair = transfer_paths(
            root,
            Path::new(&requested.from),
            Path::new(&requested.to),
            kind,
        )?;
        if matches!(
            params.operation,
            FsTransferOperation::Copy | FsTransferOperation::Import
        ) {
            scan_entry(&pair.0, 0, &|| false)?;
        }
        planned.push(pair);
    }
    for (index, (source, target)) in planned.iter().enumerate() {
        for (other_index, (other_source, other_target)) in planned.iter().enumerate() {
            if index == other_index {
                continue;
            }
            if source.starts_with(other_source) || target == other_target {
                return Err(FsError::InvalidBatch {
                    message: format!(
                        "origens ou destinos sobrepostos no lote: {} e {}",
                        source.display(),
                        other_source.display()
                    ),
                });
            }
            if other_source.is_dir() && target.starts_with(other_source) {
                return Err(FsError::InvalidBatch {
                    message: format!(
                        "destino {} cai dentro de outra origem do lote",
                        target.display()
                    ),
                });
            }
        }
    }
    Ok(planned)
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        sync::atomic::{AtomicBool, Ordering},
    };

    use kinein_protocol::{
        FsCopyParams, FsTransferBatchParams, FsTransferOperation, FsTransferStatus,
    };

    use super::transfer_batch_with_progress;
    use crate::fsops::FsError;

    fn temp_root(name: &str) -> PathBuf {
        let root = std::env::temp_dir()
            .join("kinein-transfer-batch-tests")
            .join(format!("{}-{name}", std::process::id()));
        drop(fs::remove_dir_all(&root));
        fs::create_dir_all(&root).unwrap();
        crate::platform::canonicalize(&root).unwrap()
    }

    fn pair(from: &std::path::Path, to: &std::path::Path) -> FsCopyParams {
        FsCopyParams {
            from: from.display().to_string(),
            to: to.display().to_string(),
        }
    }

    #[test]
    fn preflight_rejects_ancestor_sources_without_touching_disk() {
        let root = temp_root("nested");
        let folder = root.join("folder");
        fs::create_dir(&folder).unwrap();
        fs::write(folder.join("child.txt"), "keep").unwrap();
        let params = FsTransferBatchParams {
            operation: FsTransferOperation::Copy,
            items: vec![
                pair(&folder, &root.join("folder-copy")),
                pair(&folder.join("child.txt"), &root.join("child-copy.txt")),
            ],
        };
        let error =
            transfer_batch_with_progress(&root, &params, || false, |_, _, _, _| {}).unwrap_err();
        assert!(matches!(error, FsError::InvalidBatch { .. }));
        assert!(!root.join("folder-copy").exists());
        assert!(!root.join("child-copy.txt").exists());
    }

    #[test]
    #[cfg(unix)]
    fn preflight_checks_every_source_before_first_copy() {
        let root = temp_root("symlink");
        fs::write(root.join("good.txt"), "good").unwrap();
        let folder = root.join("bad");
        fs::create_dir(&folder).unwrap();
        std::os::unix::fs::symlink(root.join("good.txt"), folder.join("link")).unwrap();
        let params = FsTransferBatchParams {
            operation: FsTransferOperation::Copy,
            items: vec![
                pair(&root.join("good.txt"), &root.join("good-copy.txt")),
                pair(&folder, &root.join("bad-copy")),
            ],
        };
        let error =
            transfer_batch_with_progress(&root, &params, || false, |_, _, _, _| {}).unwrap_err();
        assert!(matches!(error, FsError::UnsupportedEntry { .. }));
        assert!(!root.join("good-copy.txt").exists());
    }

    #[test]
    fn runtime_collision_reports_partial_outcomes() {
        let root = temp_root("partial");
        fs::write(root.join("one.txt"), "one").unwrap();
        fs::write(root.join("two.txt"), "two").unwrap();
        let second_target = root.join("two-copy.txt");
        let params = FsTransferBatchParams {
            operation: FsTransferOperation::Copy,
            items: vec![
                pair(&root.join("one.txt"), &root.join("one-copy.txt")),
                pair(&root.join("two.txt"), &second_target),
            ],
        };
        let result = transfer_batch_with_progress(
            &root,
            &params,
            || false,
            |index, _, _, _| {
                if index == 1 && !second_target.exists() {
                    fs::write(&second_target, "other").unwrap();
                }
            },
        )
        .unwrap();
        assert_eq!(result.items[0].status, FsTransferStatus::Success);
        assert_eq!(result.items[1].status, FsTransferStatus::Failed);
        assert_eq!(
            fs::read_to_string(root.join("one-copy.txt")).unwrap(),
            "one"
        );
        assert_eq!(fs::read_to_string(&second_target).unwrap(), "other");
    }

    #[test]
    fn cancellation_keeps_unstarted_items_and_sources() {
        let root = temp_root("cancel");
        fs::write(root.join("large.bin"), vec![7_u8; 2 * 1024 * 1024]).unwrap();
        fs::write(root.join("later.txt"), "later").unwrap();
        let params = FsTransferBatchParams {
            operation: FsTransferOperation::Copy,
            items: vec![
                pair(&root.join("large.bin"), &root.join("copy.bin")),
                pair(&root.join("later.txt"), &root.join("later-copy.txt")),
            ],
        };
        let cancelled = AtomicBool::new(false);
        let result = transfer_batch_with_progress(
            &root,
            &params,
            || cancelled.load(Ordering::SeqCst),
            |index, _, done, _| {
                if index == 0 && done > 0 {
                    cancelled.store(true, Ordering::SeqCst);
                }
            },
        )
        .unwrap();
        assert!(result.cancelled);
        assert_eq!(result.items[0].status, FsTransferStatus::Cancelled);
        assert_eq!(result.items[1].status, FsTransferStatus::NotStarted);
        assert!(!root.join("copy.bin").exists());
        assert!(!root.join("later-copy.txt").exists());
        assert_eq!(fs::read_to_string(root.join("later.txt")).unwrap(), "later");
    }

    #[test]
    fn move_batch_reuses_rename_without_leaving_sources() {
        let root = temp_root("move");
        fs::write(root.join("one.txt"), "one").unwrap();
        fs::write(root.join("two.txt"), "two").unwrap();
        let params = FsTransferBatchParams {
            operation: FsTransferOperation::Move,
            items: vec![
                pair(&root.join("one.txt"), &root.join("moved-one.txt")),
                pair(&root.join("two.txt"), &root.join("moved-two.txt")),
            ],
        };
        let result =
            transfer_batch_with_progress(&root, &params, || false, |_, _, _, _| {}).unwrap();
        assert!(
            result
                .items
                .iter()
                .all(|item| item.status == FsTransferStatus::Success)
        );
        assert_eq!(
            fs::read_to_string(root.join("moved-one.txt")).unwrap(),
            "one"
        );
        assert_eq!(
            fs::read_to_string(root.join("moved-two.txt")).unwrap(),
            "two"
        );
        assert!(!root.join("one.txt").exists());
        assert!(!root.join("two.txt").exists());
    }

    #[test]
    fn import_copies_external_folder_and_keeps_source() {
        let root = temp_root("import-folder");
        let external = root.with_extension("external");
        drop(fs::remove_dir_all(&external));
        fs::create_dir(&external).unwrap();
        fs::create_dir(external.join("sub")).unwrap();
        fs::write(external.join("sub/ação #1.txt"), "conteúdo").unwrap();
        let destination = root.join("imported");
        let params = FsTransferBatchParams {
            operation: FsTransferOperation::Import,
            items: vec![pair(&external, &destination)],
        };
        let result =
            transfer_batch_with_progress(&root, &params, || false, |_, _, _, _| {}).unwrap();
        assert_eq!(result.items[0].status, FsTransferStatus::Success);
        assert_eq!(
            fs::read_to_string(destination.join("sub/ação #1.txt")).unwrap(),
            "conteúdo"
        );
        assert_eq!(
            fs::read_to_string(external.join("sub/ação #1.txt")).unwrap(),
            "conteúdo"
        );
    }

    #[test]
    #[cfg(unix)]
    fn import_rejects_symlinked_ancestor_before_copying_any_item() {
        let root = temp_root("import-symlink");
        let external = root.with_extension("external");
        drop(fs::remove_dir_all(&external));
        fs::create_dir(&external).unwrap();
        fs::write(external.join("good.txt"), "good").unwrap();
        std::os::unix::fs::symlink(&external, external.join("linked")).unwrap();
        let params = FsTransferBatchParams {
            operation: FsTransferOperation::Import,
            items: vec![
                pair(&external.join("good.txt"), &root.join("good.txt")),
                pair(&external.join("linked/good.txt"), &root.join("bad.txt")),
            ],
        };
        assert!(transfer_batch_with_progress(&root, &params, || false, |_, _, _, _| {}).is_err());
        assert!(!root.join("good.txt").exists());
    }
}

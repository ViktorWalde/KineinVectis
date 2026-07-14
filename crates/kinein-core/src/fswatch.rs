//! Debounced, workspace-confined observation of external file-system changes.
//!
//! The watcher is deliberately lazy: the workspace root is observed when it
//! opens, then each directory is added non-recursively when the explorer lists
//! it or an editor reads a file inside it. This keeps large build trees out of
//! the hot path while still protecting every opened buffer.

use std::{
    collections::{BTreeMap, HashSet},
    error::Error,
    fmt,
    path::{Path, PathBuf},
    sync::mpsc,
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use kinein_protocol::{FsChange, FsChangeKind, FsChangedEvent, FsWatchErrorEvent, JsonRpcRequest};
use notify::{
    Config, Event, EventKind, PollWatcher, RecommendedWatcher, RecursiveMode, Watcher,
    event::ModifyKind,
};
use serde_json::json;

use crate::lsp::EventSender;

const DEBOUNCE: Duration = Duration::from_millis(180);
const POLL_INTERVAL: Duration = Duration::from_secs(2);
const SKIP_DIRS: &[&str] = &[
    ".git",
    ".kinein",
    ".idea",
    ".cache",
    "target",
    "build",
    "node_modules",
];

/// Error produced while starting or extending the workspace watcher.
#[derive(Debug)]
pub(crate) enum WatchError {
    /// Both the native backend and polling fallback failed to start.
    Start {
        /// Native backend failure.
        native: String,
        /// Polling fallback failure.
        fallback: String,
    },
    /// A new directory could not be registered.
    Directory {
        /// Directory that failed.
        path: String,
        /// Backend failure.
        source: notify::Error,
    },
}

impl fmt::Display for WatchError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Start { native, fallback } => write!(
                formatter,
                "watcher nativo falhou ({native}) e fallback por polling falhou ({fallback})"
            ),
            Self::Directory { path, source } => {
                write!(formatter, "nao foi possivel observar {path}: {source}")
            }
        }
    }
}

impl Error for WatchError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Directory { source, .. } => Some(source),
            Self::Start { .. } => None,
        }
    }
}

/// Active watcher for one open workspace.
pub(crate) struct WorkspaceWatcher {
    root: PathBuf,
    watcher: Option<Box<dyn Watcher + Send>>,
    watched_directories: HashSet<PathBuf>,
    worker: Option<JoinHandle<()>>,
}

impl fmt::Debug for WorkspaceWatcher {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WorkspaceWatcher")
            .field("root", &self.root)
            .field("watched_directories", &self.watched_directories)
            .finish_non_exhaustive()
    }
}

impl WorkspaceWatcher {
    /// Starts a native watcher, falling back to periodic content polling.
    pub(crate) fn new(root: &Path, events: EventSender) -> Result<Self, WatchError> {
        let root = root.to_path_buf();
        let (raw_sender, raw_receiver) = mpsc::channel();
        let config = Config::default().with_follow_symlinks(false);
        let native_sender = raw_sender.clone();
        let watcher: Box<dyn Watcher + Send> =
            match RecommendedWatcher::new(move |event| drop(native_sender.send(event)), config) {
                Ok(watcher) => Box::new(watcher),
                Err(native) => {
                    let poll_sender = raw_sender;
                    let poll_config = Config::default()
                        .with_follow_symlinks(false)
                        .with_poll_interval(POLL_INTERVAL)
                        .with_compare_contents(true);
                    match PollWatcher::new(move |event| drop(poll_sender.send(event)), poll_config)
                    {
                        Ok(watcher) => Box::new(watcher),
                        Err(fallback) => {
                            return Err(WatchError::Start {
                                native: native.to_string(),
                                fallback: fallback.to_string(),
                            });
                        }
                    }
                }
            };

        let worker_root = root.clone();
        let worker = thread::Builder::new()
            .name("kinein-fs-watch".to_owned())
            .spawn(move || watch_loop(&worker_root, &events, &raw_receiver))
            .map_err(|source| WatchError::Start {
                native: "backend iniciado, mas a thread de debounce falhou".to_owned(),
                fallback: source.to_string(),
            })?;

        let mut result = Self {
            root,
            watcher: Some(watcher),
            watched_directories: HashSet::new(),
            worker: Some(worker),
        };
        let root_to_watch = result.root.clone();
        result.watch_directory(&root_to_watch)?;
        Ok(result)
    }

    /// Adds one directory non-recursively if it is inside this workspace.
    pub(crate) fn watch_directory(&mut self, directory: &Path) -> Result<(), WatchError> {
        if !directory.starts_with(&self.root)
            || is_ignored(&self.root, directory)
            || self.watched_directories.contains(directory)
        {
            return Ok(());
        }
        let Some(watcher) = self.watcher.as_mut() else {
            return Ok(());
        };
        watcher
            .watch(directory, RecursiveMode::NonRecursive)
            .map_err(|source| WatchError::Directory {
                path: directory.display().to_string(),
                source,
            })?;
        self.watched_directories.insert(directory.to_path_buf());
        Ok(())
    }
}

impl Drop for WorkspaceWatcher {
    fn drop(&mut self) {
        drop(self.watcher.take());
        if let Some(worker) = self.worker.take() {
            drop(worker.join());
        }
    }
}

fn watch_loop(root: &Path, events: &EventSender, receiver: &mpsc::Receiver<notify::Result<Event>>) {
    while let Ok(first) = receiver.recv() {
        let mut batch = vec![first];
        let deadline = Instant::now() + DEBOUNCE;
        let disconnected = loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            match receiver.recv_timeout(remaining) {
                Ok(event) => batch.push(event),
                Err(mpsc::RecvTimeoutError::Timeout) => break false,
                Err(mpsc::RecvTimeoutError::Disconnected) => break true,
            }
        };

        let (changes, errors) = changes_from_batch(root, batch);
        if !changes.is_empty() {
            drop(events.send(JsonRpcRequest::notification(
                "event.fs.changed",
                Some(json!(FsChangedEvent { changes })),
            )));
        }
        for message in errors {
            drop(events.send(JsonRpcRequest::notification(
                "event.fs.watchError",
                Some(json!(FsWatchErrorEvent { message })),
            )));
        }
        if disconnected {
            break;
        }
    }
}

fn changes_from_batch(
    root: &Path,
    batch: Vec<notify::Result<Event>>,
) -> (Vec<FsChange>, Vec<String>) {
    let mut observed = BTreeMap::new();
    let mut errors = Vec::new();
    for item in batch {
        match item {
            Ok(event) => record_event(&mut observed, event),
            Err(error) => errors.push(error.to_string()),
        }
    }

    let changes = observed
        .into_iter()
        .filter(|(path, _)| path.starts_with(root) && !is_ignored(root, path))
        .map(|(path, kind)| FsChange {
            path: path.display().to_string(),
            kind,
        })
        .collect();
    (changes, errors)
}

fn record_event(observed: &mut BTreeMap<PathBuf, FsChangeKind>, event: Event) {
    match event.kind {
        EventKind::Create(_) => {
            for path in event.paths {
                record_change(observed, &path, FsChangeKind::Created);
            }
        }
        EventKind::Modify(ModifyKind::Name(_)) if event.paths.len() > 1 => {
            let last = event.paths.len() - 1;
            for (index, path) in event.paths.into_iter().enumerate() {
                let kind = if index == last {
                    FsChangeKind::Created
                } else {
                    FsChangeKind::Deleted
                };
                record_change(observed, &path, kind);
            }
        }
        EventKind::Modify(ModifyKind::Name(_)) => {
            for path in event.paths {
                let kind = if path.exists() {
                    FsChangeKind::Created
                } else {
                    FsChangeKind::Deleted
                };
                record_change(observed, &path, kind);
            }
        }
        EventKind::Modify(_) => {
            for path in event.paths {
                record_change(observed, &path, FsChangeKind::Modified);
            }
        }
        EventKind::Remove(_) => {
            for path in event.paths {
                record_change(observed, &path, FsChangeKind::Deleted);
            }
        }
        EventKind::Access(_) | EventKind::Other | EventKind::Any => {}
    }
}

fn record_change(
    observed: &mut BTreeMap<PathBuf, FsChangeKind>,
    path: &Path,
    new_kind: FsChangeKind,
) {
    observed
        .entry(path.to_path_buf())
        .and_modify(|current| {
            *current = merge_change(*current, new_kind, path.exists());
        })
        .or_insert(new_kind);
}

const fn merge_change(current: FsChangeKind, incoming: FsChangeKind, exists: bool) -> FsChangeKind {
    match (current, incoming) {
        (FsChangeKind::Deleted, FsChangeKind::Created)
        | (FsChangeKind::Created, FsChangeKind::Deleted) => {
            if exists {
                FsChangeKind::Modified
            } else {
                FsChangeKind::Deleted
            }
        }
        (FsChangeKind::Created, FsChangeKind::Modified) => FsChangeKind::Created,
        (_, FsChangeKind::Deleted) => FsChangeKind::Deleted,
        (FsChangeKind::Modified, FsChangeKind::Created) => FsChangeKind::Modified,
        (_, kind) => kind,
    }
}

fn is_ignored(root: &Path, path: &Path) -> bool {
    let temporary = path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.contains(".kinein-tmp-"));
    let relative = path.strip_prefix(root).unwrap_or(path);
    temporary
        || relative.components().any(|component| {
            component
                .as_os_str()
                .to_str()
                .is_some_and(|name| SKIP_DIRS.contains(&name))
        })
}

#[cfg(test)]
mod tests {
    use std::{
        collections::BTreeMap,
        path::PathBuf,
        sync::mpsc,
        time::{Duration, Instant},
    };

    use kinein_protocol::FsChangeKind;

    use super::{WorkspaceWatcher, is_ignored, record_change};

    #[test]
    fn ignores_build_state_and_atomic_save_temporaries() {
        let root = PathBuf::from("/ws");
        assert!(is_ignored(
            &root,
            PathBuf::from("/ws/target/debug/app").as_path()
        ));
        assert!(is_ignored(
            &root,
            PathBuf::from("/ws/.main.rs.kinein-tmp-10-2").as_path()
        ));
        assert!(!is_ignored(
            &root,
            PathBuf::from("/ws/src/main.rs").as_path()
        ));
        assert!(!is_ignored(
            PathBuf::from("/home/build/workspace").as_path(),
            PathBuf::from("/home/build/workspace/src/main.rs").as_path()
        ));
    }

    #[test]
    fn delete_then_create_collapses_to_one_change() {
        let path = std::env::temp_dir().join(format!("kinein-watch-merge-{}", std::process::id()));
        std::fs::write(&path, "replacement").unwrap();
        let mut observed = BTreeMap::new();

        record_change(&mut observed, &path, FsChangeKind::Deleted);
        record_change(&mut observed, &path, FsChangeKind::Created);

        assert_eq!(observed.get(&path), Some(&FsChangeKind::Modified));
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn native_or_polling_backend_emits_debounced_change() {
        let root =
            std::env::temp_dir().join(format!("kinein-watch-integration-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let root = root.canonicalize().unwrap();
        let target = root.join("external.txt");
        let (sender, receiver) = mpsc::channel();
        let watcher = WorkspaceWatcher::new(&root, sender).unwrap();

        std::fs::write(&target, "external\n").unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        let mut observed = false;
        while Instant::now() < deadline {
            let Ok(event) = receiver.recv_timeout(Duration::from_millis(500)) else {
                continue;
            };
            if event.method == "event.fs.changed"
                && event.params.as_ref().is_some_and(|params| {
                    params["changes"].as_array().is_some_and(|changes| {
                        changes
                            .iter()
                            .any(|change| change["path"] == target.display().to_string())
                    })
                })
            {
                observed = true;
                break;
            }
        }

        assert!(observed, "watcher nao publicou a alteracao externa");
        drop(watcher);
        std::fs::remove_dir_all(root).unwrap();
    }
}

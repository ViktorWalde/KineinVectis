//! Global recent-workspace history stored in the Kinein XDG directory.
//!
//! The UI receives a complete, already sorted snapshot. It never reads the
//! filesystem to determine availability and always opens an entry through
//! `workspace.open`.

use std::{
    collections::BTreeMap,
    error::Error,
    fmt, fs, io,
    io::Write,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use kinein_protocol::{RecentWorkspaceInfo, WorkspaceInfo};
use serde::{Deserialize, Serialize};

/// Current on-disk schema for `recent-workspaces.json`.
const RECENT_WORKSPACES_SCHEMA_VERSION: u32 = 1;
/// Maximum number of recent workspace entries retained globally.
const MAX_RECENT_WORKSPACES: usize = 12;

const RECENT_WORKSPACES_FILE: &str = "recent-workspaces.json";

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PersistedRecentWorkspace {
    name: String,
    root: String,
    last_opened_at: u64,
    #[serde(default)]
    pinned: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RecentWorkspacesFile {
    schema_version: u32,
    #[serde(default)]
    workspaces: Vec<PersistedRecentWorkspace>,
}

impl RecentWorkspacesFile {
    const fn empty() -> Self {
        Self {
            schema_version: RECENT_WORKSPACES_SCHEMA_VERSION,
            workspaces: Vec::new(),
        }
    }
}

/// Error produced while validating or persisting recent workspaces.
#[derive(Debug)]
pub enum RecentWorkspaceError {
    /// A mutation did not receive a non-empty absolute root.
    InvalidRoot(String),
    /// A mutation referenced an entry that is not in the persisted list.
    MissingRoot(String),
    /// Filesystem operation failed.
    Io {
        /// Path involved in the failed operation.
        path: String,
        /// Underlying I/O error.
        source: io::Error,
    },
    /// Serialization of the versioned file failed.
    Serialize(serde_json::Error),
}

impl RecentWorkspaceError {
    /// Whether this error represents invalid user parameters.
    #[must_use]
    pub const fn is_invalid_params(&self) -> bool {
        matches!(self, Self::InvalidRoot(_) | Self::MissingRoot(_))
    }
}

impl fmt::Display for RecentWorkspaceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRoot(root) => {
                write!(formatter, "root de workspace recente invalido: {root}")
            }
            Self::MissingRoot(root) => {
                write!(formatter, "workspace recente nao encontrado: {root}")
            }
            Self::Io { path, source } => write!(formatter, "falha acessando {path}: {source}"),
            Self::Serialize(source) => {
                write!(
                    formatter,
                    "falha serializando workspaces recentes: {source}"
                )
            }
        }
    }
}

impl Error for RecentWorkspaceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::Serialize(source) => Some(source),
            Self::InvalidRoot(_) | Self::MissingRoot(_) => None,
        }
    }
}

/// Global path used by the full core process.
#[must_use]
pub fn recent_workspaces_path() -> PathBuf {
    crate::settings::global_dir().join(RECENT_WORKSPACES_FILE)
}

/// Loads the complete recent-workspace snapshot from global storage.
#[must_use]
pub fn load_recent_workspaces() -> Vec<RecentWorkspaceInfo> {
    load_at(&recent_workspaces_path())
}

/// Records a successfully opened canonical workspace.
pub fn record_recent_workspace(
    workspace: &WorkspaceInfo,
) -> Result<Vec<RecentWorkspaceInfo>, RecentWorkspaceError> {
    touch_at(&recent_workspaces_path(), workspace, current_unix_millis())
}

/// Changes the pinned state of a recent workspace.
pub fn set_recent_workspace_pinned(
    root: &str,
    pinned: bool,
) -> Result<Vec<RecentWorkspaceInfo>, RecentWorkspaceError> {
    pin_at(&recent_workspaces_path(), root, pinned)
}

/// Removes one recent workspace without requiring the root to still exist.
pub fn remove_recent_workspace(
    root: &str,
) -> Result<Vec<RecentWorkspaceInfo>, RecentWorkspaceError> {
    remove_at(&recent_workspaces_path(), root)
}

/// Clears the complete global recent-workspace history.
pub fn clear_recent_workspaces() -> Result<Vec<RecentWorkspaceInfo>, RecentWorkspaceError> {
    clear_at(&recent_workspaces_path())
}

fn current_unix_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| {
            u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
        })
}

fn validate_root(root: &str) -> Result<(), RecentWorkspaceError> {
    if root.is_empty() || !Path::new(root).is_absolute() {
        return Err(RecentWorkspaceError::InvalidRoot(root.to_owned()));
    }
    Ok(())
}

fn read_file(path: &Path) -> Result<RecentWorkspacesFile, RecentWorkspaceError> {
    let body = match fs::read_to_string(path) {
        Ok(body) => body,
        Err(source) if source.kind() == io::ErrorKind::NotFound => {
            return Ok(RecentWorkspacesFile::empty());
        }
        Err(source) => {
            return Err(RecentWorkspaceError::Io {
                path: path.display().to_string(),
                source,
            });
        }
    };
    let Ok(mut file) = serde_json::from_str::<RecentWorkspacesFile>(&body) else {
        return Ok(RecentWorkspacesFile::empty());
    };
    if file.schema_version > RECENT_WORKSPACES_SCHEMA_VERSION {
        return Ok(RecentWorkspacesFile::empty());
    }
    file.schema_version = RECENT_WORKSPACES_SCHEMA_VERSION;
    normalize(&mut file.workspaces);
    Ok(file)
}

fn normalize(workspaces: &mut Vec<PersistedRecentWorkspace>) {
    let mut merged = BTreeMap::<String, PersistedRecentWorkspace>::new();
    for entry in workspaces.drain(..) {
        if entry.root.is_empty() || !Path::new(&entry.root).is_absolute() {
            continue;
        }
        if let Some(current) = merged.get_mut(&entry.root) {
            current.pinned |= entry.pinned;
            if entry.last_opened_at > current.last_opened_at {
                current.last_opened_at = entry.last_opened_at;
                current.name = entry.name;
            }
        } else {
            merged.insert(entry.root.clone(), entry);
        }
    }
    *workspaces = merged.into_values().collect();
    workspaces.sort_by(|left, right| {
        right
            .pinned
            .cmp(&left.pinned)
            .then_with(|| right.last_opened_at.cmp(&left.last_opened_at))
            .then_with(|| left.root.cmp(&right.root))
    });
    workspaces.truncate(MAX_RECENT_WORKSPACES);
}

fn snapshot(file: &RecentWorkspacesFile) -> Vec<RecentWorkspaceInfo> {
    file.workspaces
        .iter()
        .map(|workspace| RecentWorkspaceInfo {
            name: workspace.name.clone(),
            root: workspace.root.clone(),
            last_opened_at: workspace.last_opened_at,
            pinned: workspace.pinned,
            available: Path::new(&workspace.root).is_dir(),
        })
        .collect()
}

fn load_at(path: &Path) -> Vec<RecentWorkspaceInfo> {
    read_file(path).map_or_else(|_| Vec::new(), |file| snapshot(&file))
}

fn touch_at(
    path: &Path,
    workspace: &WorkspaceInfo,
    opened_at: u64,
) -> Result<Vec<RecentWorkspaceInfo>, RecentWorkspaceError> {
    validate_root(&workspace.root)?;
    let mut file = read_file(path)?;
    let previous_pinned = file
        .workspaces
        .iter()
        .find(|entry| entry.root == workspace.root)
        .is_some_and(|entry| entry.pinned);
    let next_sequence = file
        .workspaces
        .iter()
        .map(|entry| entry.last_opened_at)
        .max()
        .unwrap_or(0)
        .saturating_add(1);
    file.workspaces.retain(|entry| entry.root != workspace.root);
    file.workspaces.push(PersistedRecentWorkspace {
        name: workspace.name.clone(),
        root: workspace.root.clone(),
        last_opened_at: opened_at.max(next_sequence),
        pinned: previous_pinned,
    });
    normalize(&mut file.workspaces);
    write_file(path, &file)?;
    Ok(snapshot(&file))
}

fn pin_at(
    path: &Path,
    root: &str,
    pinned: bool,
) -> Result<Vec<RecentWorkspaceInfo>, RecentWorkspaceError> {
    validate_root(root)?;
    let mut file = read_file(path)?;
    let Some(entry) = file.workspaces.iter_mut().find(|entry| entry.root == root) else {
        return Err(RecentWorkspaceError::MissingRoot(root.to_owned()));
    };
    entry.pinned = pinned;
    normalize(&mut file.workspaces);
    write_file(path, &file)?;
    Ok(snapshot(&file))
}

fn remove_at(path: &Path, root: &str) -> Result<Vec<RecentWorkspaceInfo>, RecentWorkspaceError> {
    validate_root(root)?;
    let mut file = read_file(path)?;
    let before = file.workspaces.len();
    file.workspaces.retain(|entry| entry.root != root);
    if file.workspaces.len() == before {
        return Err(RecentWorkspaceError::MissingRoot(root.to_owned()));
    }
    write_file(path, &file)?;
    Ok(snapshot(&file))
}

fn clear_at(path: &Path) -> Result<Vec<RecentWorkspaceInfo>, RecentWorkspaceError> {
    let file = RecentWorkspacesFile::empty();
    write_file(path, &file)?;
    Ok(Vec::new())
}

fn write_file(path: &Path, file: &RecentWorkspacesFile) -> Result<(), RecentWorkspaceError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| RecentWorkspaceError::Io {
            path: parent.display().to_string(),
            source,
        })?;
    }
    let mut body = serde_json::to_string_pretty(file).map_err(RecentWorkspaceError::Serialize)?;
    body.push('\n');
    atomic_write(path, body.as_bytes())
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), RecentWorkspaceError> {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let sequence = COUNTER.fetch_add(1, Ordering::Relaxed);
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(RECENT_WORKSPACES_FILE);
    let temporary = path.with_file_name(format!(
        ".{file_name}.kinein-tmp-{}-{sequence}",
        std::process::id()
    ));
    let io_error = |target: &Path, source| RecentWorkspaceError::Io {
        path: target.display().to_string(),
        source,
    };
    let mut output = fs::File::create(&temporary).map_err(|source| io_error(&temporary, source))?;
    if let Err(source) = output.write_all(bytes) {
        drop(fs::remove_file(&temporary));
        return Err(io_error(&temporary, source));
    }
    if let Err(source) = output.sync_all() {
        drop(fs::remove_file(&temporary));
        return Err(io_error(&temporary, source));
    }
    drop(output);
    fs::rename(&temporary, path).map_err(|source| {
        drop(fs::remove_file(&temporary));
        io_error(path, source)
    })
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use kinein_protocol::{ProjectKind, WorkspaceInfo};

    use super::{
        MAX_RECENT_WORKSPACES, RECENT_WORKSPACES_SCHEMA_VERSION, RecentWorkspaceError, clear_at,
        load_at, pin_at, remove_at, touch_at,
    };

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("kinein-recent-workspaces-tests")
            .join(format!("{}-{name}", std::process::id()));
        if dir.exists() {
            std::fs::remove_dir_all(&dir).unwrap();
        }
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn workspace(root: &Path) -> WorkspaceInfo {
        std::fs::create_dir_all(root).unwrap();
        WorkspaceInfo {
            name: root
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap()
                .to_owned(),
            root: root.display().to_string(),
            kind: ProjectKind::Unknown,
            markers: Vec::new(),
        }
    }

    #[test]
    fn touch_orders_deduplicates_and_preserves_pin() {
        let dir = temp_dir("touch");
        let storage = dir.join("recent.json");
        let first = workspace(&dir.join("first"));
        let second = workspace(&dir.join("second"));

        touch_at(&storage, &first, 10).unwrap();
        touch_at(&storage, &second, 20).unwrap();
        pin_at(&storage, &first.root, true).unwrap();
        let state = touch_at(&storage, &first, 30).unwrap();

        assert_eq!(state.len(), 2);
        assert_eq!(state[0].root, first.root);
        assert!(state[0].pinned);
        assert_eq!(state[0].last_opened_at, 30);
        assert_eq!(state[1].root, second.root);
    }

    #[test]
    fn limit_keeps_pinned_first_then_latest_access() {
        let dir = temp_dir("limit");
        let storage = dir.join("recent.json");
        let mut roots = Vec::new();
        for index in 0..(MAX_RECENT_WORKSPACES + 2) {
            let item = workspace(&dir.join(format!("workspace-{index}")));
            touch_at(&storage, &item, u64::try_from(index).unwrap() + 1).unwrap();
            roots.push(item.root);
        }
        let newest = roots.last().unwrap();
        pin_at(&storage, newest, true).unwrap();
        let state = load_at(&storage);

        assert_eq!(state.len(), MAX_RECENT_WORKSPACES);
        assert_eq!(&state[0].root, newest);
        assert!(state[0].pinned);
        assert!(!state.iter().any(|entry| entry.root == roots[0]));
    }

    #[test]
    fn version_zero_migrates_and_unknown_schema_is_ignored() {
        let dir = temp_dir("migration");
        let storage = dir.join("recent.json");
        let legacy_root = dir.join("legacy");
        std::fs::create_dir_all(&legacy_root).unwrap();
        std::fs::write(
            &storage,
            format!(
                "{{\"schemaVersion\":0,\"workspaces\":[{{\"name\":\"legacy\",\"root\":\"{}\",\"lastOpenedAt\":7}}]}}",
                legacy_root.display()
            ),
        )
        .unwrap();

        let migrated = load_at(&storage);
        assert_eq!(migrated.len(), 1);
        assert!(!migrated[0].pinned);
        pin_at(&storage, &migrated[0].root, true).unwrap();
        let body = std::fs::read_to_string(&storage).unwrap();
        assert!(body.contains(&format!(
            "\"schemaVersion\": {RECENT_WORKSPACES_SCHEMA_VERSION}"
        )));

        std::fs::write(&storage, "{\"schemaVersion\":999,\"workspaces\":[]}").unwrap();
        assert!(load_at(&storage).is_empty());
    }

    #[test]
    fn mutations_propagate_io_failures_while_listing_stays_resilient() {
        let dir = temp_dir("io-error");
        let storage = dir.join("recent.json");
        std::fs::create_dir(&storage).unwrap();
        let item = workspace(&dir.join("workspace"));

        let error = touch_at(&storage, &item, 1).unwrap_err();
        assert!(matches!(error, RecentWorkspaceError::Io { .. }));
        assert!(load_at(&storage).is_empty());
    }

    #[test]
    fn missing_path_is_marked_and_can_be_removed_or_cleared() {
        let dir = temp_dir("missing");
        let storage = dir.join("recent.json");
        let item = workspace(&dir.join("deleted"));
        touch_at(&storage, &item, 1).unwrap();
        std::fs::remove_dir_all(&item.root).unwrap();

        let missing = load_at(&storage);
        assert_eq!(missing.len(), 1);
        assert!(!missing[0].available);
        assert!(remove_at(&storage, &item.root).unwrap().is_empty());

        let another = workspace(&dir.join("another"));
        touch_at(&storage, &another, 2).unwrap();
        assert!(clear_at(&storage).unwrap().is_empty());
        assert!(load_at(&storage).is_empty());
    }
}

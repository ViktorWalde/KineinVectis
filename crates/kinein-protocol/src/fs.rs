//! Workspace-confined filesystem payloads (`fs.*`).

use serde::{Deserialize, Serialize};

/// Kind of a file system entry returned by `fs.list`.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FsEntryKind {
    /// Regular file.
    File,
    /// Directory.
    Directory,
    /// Symlink, socket, device, or anything else.
    Other,
}

/// One entry of a directory listing.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FsEntry {
    /// Entry name without the parent path.
    pub name: String,
    /// Entry kind.
    pub kind: FsEntryKind,
    /// File size in bytes. Absent for directories.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
}

/// Parameters for `fs.list` and `fs.read`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FsPathParams {
    /// Absolute path inside the open workspace root.
    pub path: String,
}

/// Parameters for `fs.createFile`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FsCreateFileParams {
    /// Absolute path of the new file inside the workspace root.
    pub path: String,
    /// Initial UTF-8 content of the file.
    #[serde(default)]
    pub content: String,
}

/// Parameters for `fs.createDirectory`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FsCreateDirectoryParams {
    /// Absolute path of the new directory inside the workspace root.
    pub path: String,
}

/// Parameters for `fs.write`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FsWriteParams {
    /// Absolute path of an existing file inside the workspace root.
    pub path: String,
    /// New UTF-8 content of the file.
    pub content: String,
}

/// Parameters for the conflict-safe `fs.write` operation.
///
/// The core only replaces the file when its current disk content still
/// matches `expected_content`. This prevents a stale editor buffer from
/// silently overwriting changes made by another process.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FsSaveParams {
    /// Absolute path of an existing file inside the workspace root.
    pub path: String,
    /// New UTF-8 content of the file.
    pub content: String,
    /// Last disk content observed by the editor.
    pub expected_content: String,
}

/// Kind of change emitted by `event.fs.changed`.
#[derive(Debug, Clone, Copy, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FsChangeKind {
    /// A new path appeared on disk.
    Created,
    /// An existing path changed on disk.
    Modified,
    /// A path disappeared from disk.
    Deleted,
}

/// One debounced external file-system change.
#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FsChange {
    /// Absolute path inside the open workspace.
    pub path: String,
    /// Observed change kind.
    pub kind: FsChangeKind,
}

/// Payload of `event.fs.changed`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FsChangedEvent {
    /// Deduplicated changes collected during the debounce window.
    pub changes: Vec<FsChange>,
}

/// Payload of `event.fs.watchError`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FsWatchErrorEvent {
    /// Human-readable watcher failure.
    pub message: String,
}

/// Parameters for `fs.rename` (also used to move within the workspace).
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FsRenameParams {
    /// Absolute path of the existing file or directory inside the workspace root.
    pub from: String,
    /// Absolute destination path inside the workspace root; must not exist yet.
    pub to: String,
}

/// Result payload for `fs.list`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FsListResult {
    /// Canonical path of the listed directory.
    pub path: String,
    /// Entries sorted directories-first, then case-insensitive by name.
    pub entries: Vec<FsEntry>,
}

/// Result payload for `fs.read`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FsReadResult {
    /// Canonical path of the file.
    pub path: String,
    /// UTF-8 file content.
    pub content: String,
}

/// Result payload for `fs.createFile`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FsCreateFileResult {
    /// Canonical path of the created file.
    pub path: String,
    /// Number of bytes written as initial content.
    pub bytes_written: u64,
}

/// Result payload for `fs.createDirectory`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FsCreateDirectoryResult {
    /// Canonical path of the created directory.
    pub path: String,
}

/// Result payload for `fs.write`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FsWriteResult {
    /// Canonical path of the file.
    pub path: String,
    /// Number of bytes written.
    pub bytes_written: u64,
}

/// Result payload for `fs.rename`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FsRenameResult {
    /// Canonical path of the source before the rename.
    pub from: String,
    /// Canonical path of the destination after the rename.
    pub to: String,
}

/// Result payload for `fs.delete`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FsDeleteResult {
    /// Canonical path that was deleted.
    pub path: String,
}

/// Parameters for `fs.search`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FsSearchParams {
    /// Literal text to look for. Not a regex.
    pub query: String,
    /// Match case exactly. Defaults to `false` (ASCII case-insensitive).
    #[serde(default)]
    pub case_sensitive: bool,
}

/// One match returned by `fs.search`. At most one match per line.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FsSearchMatch {
    /// Path relative to the workspace root.
    pub path: String,
    /// One-based line number of the match.
    pub line: u64,
    /// One-based character column of the first occurrence in the line.
    pub column: u64,
    /// Trimmed line content for preview, capped by the core.
    pub preview: String,
}

/// Result payload for `fs.search`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FsSearchResult {
    /// Matches in deterministic workspace order, capped by the core.
    pub matches: Vec<FsSearchMatch>,
    /// `true` when the match cap was reached and results were dropped.
    pub truncated: bool,
}

/// Parameters for the confirmed project-wide literal replacement.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FsReplaceParams {
    /// Non-empty literal text to replace.
    pub query: String,
    /// Replacement text; may be empty to delete matches.
    pub replacement: String,
    /// Match case exactly. Defaults to ASCII case-insensitive.
    #[serde(default)]
    pub case_sensitive: bool,
}

/// Result payload for `fs.replace`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FsReplaceResult {
    /// Canonical absolute paths rewritten by the transaction.
    pub files: Vec<String>,
    /// Total literal occurrences replaced.
    pub replacements: u64,
}

/// Parameters for `fs.findFiles`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FsFindFilesParams {
    /// Literal file-name query passed to `fd`.
    pub query: String,
}

/// One file result returned by `fs.findFiles`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FsFileMatch {
    /// Path relative to the workspace root.
    pub path: String,
    /// File name without parent directories.
    pub name: String,
}

/// Result payload for `fs.findFiles`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FsFindFilesResult {
    /// File matches, capped by the core.
    pub matches: Vec<FsFileMatch>,
    /// `true` when the match cap was reached and results were dropped.
    pub truncated: bool,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::{
        FsChange, FsChangeKind, FsChangedEvent, FsCreateDirectoryParams, FsCreateDirectoryResult,
        FsCreateFileParams, FsCreateFileResult, FsDeleteResult, FsFileMatch, FsFindFilesParams,
        FsFindFilesResult, FsRenameParams, FsRenameResult, FsReplaceParams, FsReplaceResult,
        FsSaveParams, FsSearchMatch, FsSearchParams, FsSearchResult,
    };

    #[test]
    fn fs_save_requires_expected_content() {
        let parsed = serde_json::from_value::<FsSaveParams>(json!({
            "path": "/tmp/main.rs",
            "content": "novo",
            "expectedContent": "antigo"
        }))
        .unwrap();
        let missing_expected = serde_json::from_value::<FsSaveParams>(json!({
            "path": "/tmp/main.rs",
            "content": "novo"
        }));

        assert_eq!(parsed.expected_content, "antigo");
        assert!(missing_expected.is_err());
    }

    #[test]
    fn fs_changed_event_serializes_camel_case() {
        let value = serde_json::to_value(FsChangedEvent {
            changes: vec![FsChange {
                path: "/tmp/main.rs".to_owned(),
                kind: FsChangeKind::Modified,
            }],
        })
        .unwrap();

        assert_eq!(value["changes"][0]["path"], "/tmp/main.rs");
        assert_eq!(value["changes"][0]["kind"], "modified");
    }

    #[test]
    fn fs_replace_payloads_use_camel_case_and_reject_unknown_fields() {
        let params: FsReplaceParams = serde_json::from_value(json!({
            "query": "old",
            "replacement": "new",
            "caseSensitive": true
        }))
        .unwrap();
        assert!(params.case_sensitive);

        let value = serde_json::to_value(FsReplaceResult {
            files: vec!["/tmp/ws/a.rs".to_owned()],
            replacements: 2,
        })
        .unwrap();
        assert_eq!(value["replacements"], 2);
    }

    #[test]
    fn fs_search_params_default_case_and_reject_unknown_fields() {
        let valid = serde_json::from_value::<FsSearchParams>(json!({ "query": "todo" })).unwrap();
        let explicit = serde_json::from_value::<FsSearchParams>(
            json!({ "query": "todo", "caseSensitive": true }),
        )
        .unwrap();
        let invalid = serde_json::from_value::<FsSearchParams>(json!({ "query": "x", "y": 1 }));

        assert_eq!(valid.query, "todo");
        assert!(!valid.case_sensitive);
        assert!(explicit.case_sensitive);
        assert!(invalid.is_err());
    }

    #[test]
    fn fs_create_file_params_default_content_and_reject_unknown_fields() {
        let valid =
            serde_json::from_value::<FsCreateFileParams>(json!({ "path": "/tmp/new.rs" })).unwrap();
        let invalid =
            serde_json::from_value::<FsCreateFileParams>(json!({ "path": "/tmp/new.rs", "x": 1 }));
        let result = FsCreateFileResult {
            path: "/tmp/new.rs".to_owned(),
            bytes_written: 3,
        };
        let value = serde_json::to_value(result).unwrap();

        assert_eq!(valid.path, "/tmp/new.rs");
        assert_eq!(valid.content, "");
        assert!(invalid.is_err());
        assert_eq!(value["bytesWritten"], 3);
    }

    #[test]
    fn fs_create_directory_params_and_result_use_camel_case() {
        let valid =
            serde_json::from_value::<FsCreateDirectoryParams>(json!({ "path": "/tmp/module" }))
                .unwrap();
        let invalid = serde_json::from_value::<FsCreateDirectoryParams>(
            json!({ "path": "/tmp/module", "recursive": true }),
        );
        let result = FsCreateDirectoryResult {
            path: "/tmp/module".to_owned(),
        };
        let value = serde_json::to_value(result).unwrap();

        assert_eq!(valid.path, "/tmp/module");
        assert!(invalid.is_err());
        assert_eq!(value["path"], "/tmp/module");
    }

    #[test]
    fn fs_rename_params_and_result_use_camel_case() {
        let valid = serde_json::from_value::<FsRenameParams>(
            json!({ "from": "/tmp/a.rs", "to": "/tmp/b.rs" }),
        )
        .unwrap();
        let invalid = serde_json::from_value::<FsRenameParams>(
            json!({ "from": "/tmp/a.rs", "to": "/tmp/b.rs", "x": 1 }),
        );
        let result = FsRenameResult {
            from: "/tmp/a.rs".to_owned(),
            to: "/tmp/b.rs".to_owned(),
        };
        let value = serde_json::to_value(result).unwrap();

        assert_eq!(valid.from, "/tmp/a.rs");
        assert_eq!(valid.to, "/tmp/b.rs");
        assert!(invalid.is_err());
        assert_eq!(value["from"], "/tmp/a.rs");
        assert_eq!(value["to"], "/tmp/b.rs");
    }

    #[test]
    fn fs_delete_result_serializes_camel_case() {
        let result = FsDeleteResult {
            path: "/tmp/gone.rs".to_owned(),
        };
        let value = serde_json::to_value(result).unwrap();

        assert_eq!(value["path"], "/tmp/gone.rs");
    }

    #[test]
    fn fs_search_result_serializes_camel_case() {
        let result = FsSearchResult {
            matches: vec![FsSearchMatch {
                path: "src/main.rs".to_owned(),
                line: 3,
                column: 5,
                preview: "let total = somar(2, 3);".to_owned(),
            }],
            truncated: false,
        };
        let value = serde_json::to_value(result).unwrap();

        assert_eq!(value["matches"][0]["path"], "src/main.rs");
        assert_eq!(value["matches"][0]["line"], 3);
        assert_eq!(value["matches"][0]["column"], 5);
        assert_eq!(value["truncated"], false);
    }

    #[test]
    fn fs_find_files_params_and_result_use_camel_case() {
        let valid =
            serde_json::from_value::<FsFindFilesParams>(json!({ "query": "main" })).unwrap();
        let invalid = serde_json::from_value::<FsFindFilesParams>(
            json!({ "query": "main", "caseSensitive": true }),
        );
        let result = FsFindFilesResult {
            matches: vec![FsFileMatch {
                path: "src/main.rs".to_owned(),
                name: "main.rs".to_owned(),
            }],
            truncated: false,
        };
        let value = serde_json::to_value(result).unwrap();

        assert_eq!(valid.query, "main");
        assert!(invalid.is_err());
        assert_eq!(value["matches"][0]["path"], "src/main.rs");
        assert_eq!(value["matches"][0]["name"], "main.rs");
        assert_eq!(value["truncated"], false);
    }
}

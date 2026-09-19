//! Git status payloads (`git.*`).

use serde::{Deserialize, Serialize};

/// Consolidated state of one changed file (the core decodes porcelain).
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GitEntryKind {
    /// Tracked file with content changes.
    Modified,
    /// File added to the index (new tracked file).
    Added,
    /// Tracked file deleted.
    Deleted,
    /// File renamed (path is the new name).
    Renamed,
    /// File unknown to git.
    Untracked,
    /// Merge conflict.
    Conflicted,
}

/// One changed file of the repository, workspace-relative.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GitEntryInfo {
    /// Path relative to the workspace root.
    pub path: String,
    /// Consolidated state for coloring/listing.
    pub kind: GitEntryKind,
    /// `true` when the change (or part of it) is in the index.
    pub staged: bool,
}

/// Result payload for `git.status`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GitStatusResult {
    /// `false` when the workspace is not inside a git repository.
    pub repo: bool,
    /// Current branch name; absent on detached HEAD or when not a repo.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    /// `true` when HEAD is detached (`shortSha` identifies it).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub detached: bool,
    /// Abbreviated commit of a detached HEAD.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub short_sha: Option<String>,
    /// Upstream ref name, when the branch tracks one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub upstream: Option<String>,
    /// Commits ahead of upstream.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ahead: Option<i64>,
    /// Commits behind upstream.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub behind: Option<i64>,
    /// Changed files, workspace-relative, porcelain order.
    pub entries: Vec<GitEntryInfo>,
}

/// One local branch returned by `git.branches`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GitBranchInfo {
    /// Short local branch name.
    pub name: String,
    /// Whether HEAD currently points at this branch.
    pub current: bool,
}

/// Result payload for `git.branches`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GitBranchesResult {
    /// `false` when the workspace is outside a Git repository.
    pub repo: bool,
    /// Local branches in ref-name order.
    pub branches: Vec<GitBranchInfo>,
}

/// Parameters for `git.checkout`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GitCheckoutParams {
    /// Existing local branch name.
    pub branch: String,
}

/// Parameters for `git.branchCreate`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GitBranchCreateParams {
    /// New branch name.
    pub name: String,
    /// Switch to the new branch immediately.
    #[serde(default = "default_true")]
    pub checkout: bool,
}

/// Stash operation requested by `git.stash`.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GitStashAction {
    /// Store worktree/index changes, including untracked files.
    Push,
    /// Apply and drop the most recent stash.
    Pop,
}

/// Parameters for `git.stash`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GitStashParams {
    /// Push or pop the most recent stash.
    pub action: GitStashAction,
    /// Optional message for `push`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

const fn default_true() -> bool {
    true
}

/// Parameters for `git.fileDiff`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GitFileDiffParams {
    /// Absolute file path inside the workspace.
    pub path: String,
}

/// Kind of one diff hunk, consolidated for the editor gutter.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GitHunkKind {
    /// Lines added (range on the new side).
    Added,
    /// Lines changed (range on the new side).
    Modified,
    /// Lines removed between `startLine - 1` and `startLine`.
    Removed,
}

/// One gutter-ready hunk of `git diff --unified=0`.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GitDiffHunkInfo {
    /// Consolidated hunk kind.
    pub kind: GitHunkKind,
    /// First 1-based line of the hunk on the new side (anchor line for
    /// removals).
    pub start_line: u32,
    /// Lines covered on the new side (1 for removals).
    pub line_count: u32,
}

/// Result payload for `git.fileDiff`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GitFileDiffResult {
    /// Canonical path echoed back for correlation/stale-drop.
    pub path: String,
    /// `false` when the workspace is not a git repository.
    pub repo: bool,
    /// `false` when git does not track the file (untracked = whole file
    /// reported as one added hunk).
    pub tracked: bool,
    /// Gutter hunks (from `--unified=0`), top to bottom.
    pub hunks: Vec<GitDiffHunkInfo>,
    /// Unified diff text (`--unified=3`) for the diff view; empty for
    /// untracked files.
    pub text: String,
}

/// Parameters for `git.stage` / `git.unstage` / `git.discard`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GitPathsParams {
    /// Absolute paths inside the workspace (deleted files allowed).
    pub paths: Vec<String>,
}

/// Parameters for `git.commit`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GitCommitParams {
    /// Non-empty commit message (single line in v1).
    pub message: String,
    /// `true` rewrites the last commit with the staged changes and this
    /// message (`git commit --amend`, `0.126.0`). The UI asks before, because
    /// it rewrites history; the core refuses when HEAD was already pushed
    /// is NOT checked here — that is the author's call.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub amend: bool,
}

/// Parameters for `git.blame`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GitBlameParams {
    /// Absolute file path inside the workspace (must exist on disk).
    pub path: String,
}

/// One consecutive run of lines attributed to the same commit.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GitBlameGroupInfo {
    /// First 1-based line of the group in the current file.
    pub start_line: u32,
    /// Lines covered by the group.
    pub line_count: u32,
    /// Full commit sha; all zeros while the change is uncommitted.
    pub sha: String,
    /// Author name as recorded by git.
    pub author: String,
    /// Author time (epoch seconds); the UI formats relative age.
    pub author_time: i64,
    /// First line of the commit message; empty for uncommitted lines.
    pub summary: String,
    /// `false` when the lines are not committed yet (zero sha).
    pub committed: bool,
}

/// Result payload for `git.blame`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GitBlameResult {
    /// Canonical path echoed back for correlation/stale-drop.
    pub path: String,
    /// `false` when the workspace is not a git repository.
    pub repo: bool,
    /// `false` when git does not track the file (groups stay empty).
    pub tracked: bool,
    /// Blame groups, top to bottom.
    pub groups: Vec<GitBlameGroupInfo>,
}

/// Parameters for `git.log`.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GitLogParams {
    /// Maximum commits to list (default 100; the handler rejects values
    /// outside 1..=500).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_count: Option<u32>,
}

/// One commit of `git.log`, newest first.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GitLogEntryInfo {
    /// Full commit sha.
    pub sha: String,
    /// Abbreviated sha for display.
    pub short_sha: String,
    /// Author name.
    pub author: String,
    /// Author time (epoch seconds).
    pub author_time: i64,
    /// First line of the commit message.
    pub summary: String,
    /// Parent shas, oldest first (`%P`; two on a merge) — what the graph
    /// of the Log view is drawn from (`0.126.0`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parents: Vec<String>,
    /// Ref names pointing at this commit (`%D`: `HEAD -> main`, `origin/main`,
    /// `tag: v1`), already split (`0.126.0`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub refs: Vec<String>,
}

/// Result payload for `git.log`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GitLogResult {
    /// `false` when the workspace is not a git repository.
    pub repo: bool,
    /// Commits, newest first; empty on a repo without commits.
    pub entries: Vec<GitLogEntryInfo>,
}

/// Parameters for `git.commitDiff`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GitCommitDiffParams {
    /// Commit sha (4..=64 hex chars, validated before reaching git).
    pub sha: String,
}

/// Result payload for `git.commitDiff`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GitCommitDiffResult {
    /// Sha echoed back for correlation/stale-drop.
    pub sha: String,
    /// Unified patch of the commit (`git show --pretty=format:`); may
    /// be empty on trivial merge commits.
    pub text: String,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{
        GitBranchCreateParams, GitBranchesResult, GitCheckoutParams, GitEntryInfo, GitEntryKind,
        GitStashAction, GitStashParams, GitStatusResult,
    };

    #[test]
    fn status_serializes_camel_case_and_omits_empty_fields() {
        let value = serde_json::to_value(GitStatusResult {
            repo: true,
            branch: Some("main".to_owned()),
            detached: false,
            short_sha: None,
            upstream: None,
            ahead: None,
            behind: None,
            entries: vec![GitEntryInfo {
                path: "src/main.rs".to_owned(),
                kind: GitEntryKind::Modified,
                staged: false,
            }],
        })
        .unwrap();

        assert_eq!(value["repo"], true);
        assert_eq!(value["branch"], "main");
        assert!(value.get("detached").is_none());
        assert!(value.get("upstream").is_none());
        assert_eq!(value["entries"][0]["kind"], "modified");
        assert_eq!(value["entries"][0]["staged"], false);
    }

    #[test]
    fn file_diff_result_serializes_hunks_camel_case() {
        let value = serde_json::to_value(super::GitFileDiffResult {
            path: "/w/a.rs".to_owned(),
            repo: true,
            tracked: true,
            hunks: vec![super::GitDiffHunkInfo {
                kind: super::GitHunkKind::Removed,
                start_line: 4,
                line_count: 1,
            }],
            text: "@@ ...".to_owned(),
        })
        .unwrap();
        assert_eq!(value["hunks"][0]["kind"], "removed");
        assert_eq!(value["hunks"][0]["startLine"], 4);
        assert_eq!(value["tracked"], true);
    }

    #[test]
    fn non_repo_payload_is_minimal() {
        let value = serde_json::to_value(GitStatusResult {
            repo: false,
            branch: None,
            detached: false,
            short_sha: None,
            upstream: None,
            ahead: None,
            behind: None,
            entries: Vec::new(),
        })
        .unwrap();
        assert_eq!(value, json!({ "repo": false, "entries": [] }));
    }

    #[test]
    fn branch_and_stash_payloads_are_strict_and_use_camel_case() {
        let create: GitBranchCreateParams =
            serde_json::from_value(json!({ "name": "feature/semantic" })).unwrap();
        assert!(create.checkout);
        assert!(
            serde_json::from_value::<GitCheckoutParams>(json!({
                "branch": "main",
                "unexpected": true
            }))
            .is_err()
        );

        let stash: GitStashParams = serde_json::from_value(json!({
            "action": "push",
            "message": "checkpoint"
        }))
        .unwrap();
        assert_eq!(stash.action, GitStashAction::Push);

        let branches = serde_json::to_value(GitBranchesResult {
            repo: true,
            branches: vec![super::GitBranchInfo {
                name: "main".to_owned(),
                current: true,
            }],
        })
        .unwrap();
        assert_eq!(branches["branches"][0]["name"], "main");
        assert_eq!(branches["branches"][0]["current"], true);
    }
}

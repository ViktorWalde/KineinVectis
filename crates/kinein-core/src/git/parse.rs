//! Parsers puros das saidas estaveis usadas pelo dominio Git.

use std::collections::HashMap;

use kinein_protocol::{
    GitBlameGroupInfo, GitDiffHunkInfo, GitEntryInfo, GitEntryKind, GitHunkKind, GitLogEntryInfo,
    GitStatusResult,
};

/// The `{ repo: false }` payload of a workspace without git.
pub(super) const fn non_repo_status() -> GitStatusResult {
    GitStatusResult {
        repo: false,
        branch: None,
        detached: false,
        short_sha: None,
        upstream: None,
        ahead: None,
        behind: None,
        entries: Vec::new(),
    }
}

/// Parses `git status --porcelain=v2 --branch -z` output.
///
/// `prefix` is the workspace position inside the repo
/// (`rev-parse --show-prefix`): porcelain paths are toplevel-relative, the
/// protocol wants workspace-relative, and entries outside the workspace
/// are dropped.
#[must_use]
pub fn parse_status(body: &str, prefix: &str) -> GitStatusResult {
    let mut result = non_repo_status();
    result.repo = true;
    let mut oid_short: Option<String> = None;

    let mut tokens = body.split('\0');
    while let Some(token) = tokens.next() {
        if token.is_empty() {
            continue;
        }
        if let Some(header) = token.strip_prefix("# ") {
            parse_branch_header(header, &mut result, &mut oid_short);
            continue;
        }
        let (entry, consumes_extra) = parse_entry_token(token);
        if consumes_extra {
            // Tipo 2 (rename): o token seguinte e o caminho antigo.
            let _original = tokens.next();
        }
        if let Some(entry) = entry {
            if let Some(path) = workspace_relative(&entry.path, prefix) {
                result.entries.push(GitEntryInfo { path, ..entry });
            }
        }
    }

    if result.detached {
        result.short_sha = oid_short;
    }
    result
}

/// Applies one `# branch.*` porcelain header.
fn parse_branch_header(header: &str, result: &mut GitStatusResult, oid_short: &mut Option<String>) {
    if let Some(oid) = header.strip_prefix("branch.oid ") {
        if oid != "(initial)" {
            *oid_short = Some(oid.chars().take(8).collect());
        }
    } else if let Some(head) = header.strip_prefix("branch.head ") {
        if head == "(detached)" {
            result.detached = true;
        } else {
            result.branch = Some(head.to_owned());
        }
    } else if let Some(upstream) = header.strip_prefix("branch.upstream ") {
        result.upstream = Some(upstream.to_owned());
    } else if let Some(ab) = header.strip_prefix("branch.ab ") {
        for part in ab.split_whitespace() {
            if let Some(ahead) = part.strip_prefix('+') {
                result.ahead = ahead.parse().ok();
            } else if let Some(behind) = part.strip_prefix('-') {
                result.behind = behind.parse().ok();
            }
        }
    }
}

/// Parses one porcelain entry token; `true` means a rename consumed the
/// following token (original path).
fn parse_entry_token(token: &str) -> (Option<GitEntryInfo>, bool) {
    if let Some(path) = token.strip_prefix("? ") {
        return (
            Some(GitEntryInfo {
                path: path.to_owned(),
                kind: GitEntryKind::Untracked,
                staged: false,
            }),
            false,
        );
    }
    if token.starts_with("u ") {
        // u XY sub m1 m2 m3 mW h1 h2 h3 <path>: 10 campos antes do path.
        let path = nth_field_rest(token, 10);
        return (
            path.map(|path| GitEntryInfo {
                path: path.to_owned(),
                kind: GitEntryKind::Conflicted,
                staged: false,
            }),
            false,
        );
    }
    let renamed = token.starts_with("2 ");
    if token.starts_with("1 ") || renamed {
        // 1 XY sub mH mI mW hH hI <path> (8 campos antes do path);
        // 2 XY sub mH mI mW hH hI R<score> <path> (9 campos antes).
        let xy = token.split_ascii_whitespace().nth(1).unwrap_or("..");
        let path = nth_field_rest(token, if renamed { 9 } else { 8 });
        let Some(path) = path else {
            return (None, renamed);
        };
        let mut chars = xy.chars();
        let index = chars.next().unwrap_or('.');
        let worktree = chars.next().unwrap_or('.');
        let kind = if renamed {
            GitEntryKind::Renamed
        } else {
            consolidated_kind(index, worktree)
        };
        return (
            Some(GitEntryInfo {
                path: path.to_owned(),
                kind,
                staged: index != '.',
            }),
            renamed,
        );
    }
    (None, false)
}

/// Consolidates the porcelain XY pair into one display kind (worktree
/// state wins over index state; the pair never reaches the UI).
const fn consolidated_kind(index: char, worktree: char) -> GitEntryKind {
    match worktree {
        'D' => GitEntryKind::Deleted,
        'M' | 'T' => GitEntryKind::Modified,
        'A' => GitEntryKind::Added,
        _ => match index {
            'A' => GitEntryKind::Added,
            'D' => GitEntryKind::Deleted,
            'R' => GitEntryKind::Renamed,
            _ => GitEntryKind::Modified,
        },
    }
}

/// Returns the rest of `token` after `fields` whitespace-separated fields
/// (porcelain paths may contain spaces; only the prefix is field-split).
fn nth_field_rest(token: &str, fields: usize) -> Option<&str> {
    let mut rest = token;
    for _ in 0..fields {
        let space = rest.find(' ')?;
        rest = &rest[space + 1..];
    }
    if rest.is_empty() { None } else { Some(rest) }
}

/// Converts a toplevel-relative path into a workspace-relative one.
///
/// `None` drops entries fora do workspace, entradas vazias e os metadados
/// da propria IDE (`.kinein/` mudando a cada open so poluiria o status;
/// gatilho registrado: quem versionar .kinein de proposito nao ve essas
/// entries no v1).
pub(super) fn workspace_relative(path: &str, prefix: &str) -> Option<String> {
    let relative = if prefix.is_empty() {
        path
    } else {
        path.strip_prefix(prefix)?
    };
    if relative.is_empty() || relative == ".kinein" || relative.starts_with(".kinein/") {
        return None;
    }
    Some(relative.to_owned())
}

/// Extracts gutter hunks from `git diff --unified=0` output.
///
/// Only `@@ -a,b +c,d @@` headers matter: `b == 0` → added run,
/// `d == 0` → removed (anchored below the boundary, on new-side line
/// `c + 1`), otherwise → modified.
#[must_use]
pub fn parse_hunks(diff: &str) -> Vec<GitDiffHunkInfo> {
    diff.lines()
        .filter_map(|line| {
            let header = line.strip_prefix("@@ ")?;
            let header = &header[..header.find(" @@")?];
            let mut sides = header.split_whitespace();
            let old = parse_range(sides.next()?.strip_prefix('-')?)?;
            let new = parse_range(sides.next()?.strip_prefix('+')?)?;
            let (kind, start_line, line_count) = if old.1 == 0 {
                (GitHunkKind::Added, new.0, new.1)
            } else if new.1 == 0 {
                (GitHunkKind::Removed, new.0 + 1, 1)
            } else {
                (GitHunkKind::Modified, new.0, new.1)
            };
            Some(GitDiffHunkInfo {
                kind,
                start_line: start_line.max(1),
                line_count: line_count.max(1),
            })
        })
        .collect()
}

/// Parses a `start[,count]` diff range; count defaults to 1.
fn parse_range(range: &str) -> Option<(u32, u32)> {
    match range.split_once(',') {
        Some((start, count)) => Some((start.parse().ok()?, count.parse().ok()?)),
        None => Some((range.parse().ok()?, 1)),
    }
}

/// Metadata of one blame commit, memoized across the porcelain stream.
#[derive(Default, Clone)]
struct BlameCommitMeta {
    author: String,
    author_time: i64,
    summary: String,
}

/// Parses `git blame --porcelain` output into consecutive line groups.
///
/// Group headers are `<sha> <orig> <final> <count>`; headers without the
/// 4th field continue the current group line by line and are skipped.
/// Metadata tags (`author`, `author-time`, `summary`) follow only the
/// FIRST header of each sha — the parser memoizes them by sha. Content
/// lines start with a tab.
#[must_use]
pub fn parse_blame(body: &str) -> Vec<GitBlameGroupInfo> {
    let mut metadata: HashMap<String, BlameCommitMeta> = HashMap::new();
    let mut groups: Vec<(String, u32, u32)> = Vec::new();
    let mut current_sha = String::new();

    for line in body.lines() {
        if line.starts_with('\t') {
            continue;
        }
        if let Some((sha, start_line, line_count)) = parse_blame_header(line) {
            sha.clone_into(&mut current_sha);
            metadata.entry(current_sha.clone()).or_default();
            if let Some((start, count)) = start_line.zip(line_count) {
                groups.push((current_sha.clone(), start, count));
            }
            continue;
        }
        let Some(meta) = metadata.get_mut(&current_sha) else {
            continue;
        };
        if let Some(author) = line.strip_prefix("author ") {
            author.clone_into(&mut meta.author);
        } else if let Some(time) = line.strip_prefix("author-time ") {
            meta.author_time = time.parse().unwrap_or_default();
        } else if let Some(summary) = line.strip_prefix("summary ") {
            summary.clone_into(&mut meta.summary);
        }
    }

    groups
        .into_iter()
        .map(|(sha, start_line, line_count)| {
            let meta = metadata.get(&sha).cloned().unwrap_or_default();
            let committed = !sha.chars().all(|character| character == '0');
            GitBlameGroupInfo {
                start_line,
                line_count,
                sha,
                author: meta.author,
                author_time: meta.author_time,
                summary: meta.summary,
                committed,
            }
        })
        .collect()
}

/// Splits a porcelain blame header; `None` when the line is a metadata
/// tag. Headers carry the group line count only on the group's first
/// line — continuation headers answer `(sha, None, None)`.
fn parse_blame_header(line: &str) -> Option<(&str, Option<u32>, Option<u32>)> {
    let mut fields = line.split_ascii_whitespace();
    let sha = fields.next()?;
    if sha.len() < 40 || !sha.chars().all(|character| character.is_ascii_hexdigit()) {
        return None;
    }
    let _original_line = fields.next()?;
    let final_line = fields.next()?.parse().ok();
    let line_count = fields.next().and_then(|count| count.parse().ok());
    Some((sha, final_line, line_count))
}

/// Parses the `git log -z` records (NUL between records, US 0x1f between fields).
///
/// Format: `%H%x1f%h%x1f%an%x1f%at%x1f%s%x1f%P%x1f%D`. `%P` and `%D`
/// (0.126.0) may be absent in older captures: both default to empty.
#[must_use]
pub fn parse_log(body: &str) -> Vec<GitLogEntryInfo> {
    body.split('\0')
        .filter_map(|record| {
            let mut fields = record.split('\u{1f}');
            let sha = fields.next()?.trim();
            if sha.is_empty() {
                return None;
            }
            Some(GitLogEntryInfo {
                sha: sha.to_owned(),
                short_sha: fields.next()?.to_owned(),
                author: fields.next()?.to_owned(),
                author_time: fields.next()?.parse().unwrap_or_default(),
                summary: fields.next().unwrap_or_default().to_owned(),
                parents: fields
                    .next()
                    .unwrap_or_default()
                    .split_whitespace()
                    .map(str::to_owned)
                    .collect(),
                refs: fields
                    .next()
                    .unwrap_or_default()
                    .split(", ")
                    .map(str::trim)
                    .filter(|r| !r.is_empty())
                    .map(str::to_owned)
                    .collect(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use kinein_protocol::GitEntryKind;

    use super::{parse_blame, parse_hunks, parse_log, parse_status};

    #[test]
    fn parses_blame_groups_with_memoized_metadata_and_zero_sha() {
        let committed = "a".repeat(40);
        let zero = "0".repeat(40);
        let body = format!(
            concat!(
                "{sha} 1 1 2\n",
                "author Alice\n",
                "author-time 1700000000\n",
                "summary primeiro commit\n",
                "filename a.txt\n",
                "\tlinha um\n",
                "{sha} 2 2\n",
                "\tlinha dois\n",
                "{zero} 3 3 1\n",
                "author Not Committed Yet\n",
                "author-time 1700000100\n",
                "summary Version of a.txt from a.txt\n",
                "\tlinha tres\n",
                "{sha} 4 4 1\n",
                "\tlinha quatro\n",
            ),
            sha = committed,
            zero = zero,
        );
        let groups = parse_blame(&body);

        assert_eq!(groups.len(), 3);
        assert_eq!(groups[0].start_line, 1);
        assert_eq!(groups[0].line_count, 2);
        assert_eq!(groups[0].author, "Alice");
        assert_eq!(groups[0].author_time, 1_700_000_000);
        assert_eq!(groups[0].summary, "primeiro commit");
        assert!(groups[0].committed);
        assert_eq!(groups[1].sha, zero);
        assert!(!groups[1].committed);
        // O sha reaparece SEM metadado no stream: precisa vir memoizado.
        assert_eq!(groups[2].start_line, 4);
        assert_eq!(groups[2].author, "Alice");
        assert!(groups[2].committed);
    }

    #[test]
    fn parses_log_records_with_stable_separators() {
        let body = format!(
            "{sha1}\u{1f}abc1234\u{1f}Alice\u{1f}1700000000\u{1f}feat: algo\u{1f}{sha2} {sha3}\u{1f}HEAD -> main, origin/main, tag: v1\0\
             {sha2}\u{1f}def5678\u{1f}Bob B.\u{1f}1690000000\u{1f}fix: virgula, aspas \"x\"",
            sha1 = "1".repeat(40),
            sha2 = "2".repeat(40),
            sha3 = "3".repeat(40),
        );
        let entries = parse_log(&body);

        assert_eq!(entries.len(), 2);
        // 0.126.0: pais (um merge tem dois) e refs, ja' separados.
        assert_eq!(entries[0].parents, ["2".repeat(40), "3".repeat(40)]);
        assert_eq!(entries[0].refs, ["HEAD -> main", "origin/main", "tag: v1"]);
        assert!(entries[1].parents.is_empty() && entries[1].refs.is_empty());
        assert_eq!(entries[0].short_sha, "abc1234");
        assert_eq!(entries[0].author, "Alice");
        assert_eq!(entries[0].author_time, 1_700_000_000);
        assert_eq!(entries[0].summary, "feat: algo");
        assert_eq!(entries[1].author, "Bob B.");
        assert_eq!(entries[1].summary, "fix: virgula, aspas \"x\"");
        assert!(parse_log("").is_empty());
    }

    #[test]
    fn parses_branch_headers_and_common_entries() {
        let body = concat!(
            "# branch.oid 1234567890abcdef\0",
            "# branch.head main\0",
            "# branch.upstream origin/main\0",
            "# branch.ab +2 -1\0",
            "1 .M N... 100644 100644 100644 h h src/main.rs\0",
            "1 M. N... 100644 100644 100644 h h staged.rs\0",
            "1 MM N... 100644 100644 100644 h h both.rs\0",
            "1 .D N... 100644 100644 100644 h h gone.rs\0",
            "? novo.txt\0",
            "u UU N... 100644 100644 100644 100644 h h h conflito.rs\0",
        );
        let status = parse_status(body, "");

        assert!(status.repo);
        assert_eq!(status.branch.as_deref(), Some("main"));
        assert!(!status.detached);
        assert_eq!(status.upstream.as_deref(), Some("origin/main"));
        assert_eq!(status.ahead, Some(2));
        assert_eq!(status.behind, Some(1));

        let kinds: Vec<(&str, GitEntryKind, bool)> = status
            .entries
            .iter()
            .map(|entry| (entry.path.as_str(), entry.kind, entry.staged))
            .collect();
        assert_eq!(
            kinds,
            vec![
                ("src/main.rs", GitEntryKind::Modified, false),
                ("staged.rs", GitEntryKind::Modified, true),
                ("both.rs", GitEntryKind::Modified, true),
                ("gone.rs", GitEntryKind::Deleted, false),
                ("novo.txt", GitEntryKind::Untracked, false),
                ("conflito.rs", GitEntryKind::Conflicted, false),
            ]
        );
    }

    #[test]
    fn rename_consumes_original_path_token_and_detached_head_uses_oid() {
        let body = concat!(
            "# branch.oid abcdef0123456789\0",
            "# branch.head (detached)\0",
            "2 R. N... 100644 100644 100644 h h R100 novo/nome.rs\0",
            "antigo/nome.rs\0",
            "? depois-do-rename.txt\0",
        );
        let status = parse_status(body, "");

        assert!(status.detached);
        assert_eq!(status.branch, None);
        assert_eq!(status.short_sha.as_deref(), Some("abcdef01"));
        assert_eq!(status.entries.len(), 2);
        assert_eq!(status.entries[0].path, "novo/nome.rs");
        assert_eq!(status.entries[0].kind, GitEntryKind::Renamed);
        assert!(status.entries[0].staged);
        assert_eq!(status.entries[1].path, "depois-do-rename.txt");
    }

    #[test]
    fn workspace_prefix_strips_and_filters_outside_entries() {
        let body = concat!(
            "# branch.head main\0",
            "1 .M N... 100644 100644 100644 h h ui/src/app.cpp\0",
            "1 .M N... 100644 100644 100644 h h core/lib.rs\0",
        );
        let status = parse_status(body, "ui/");

        assert_eq!(status.entries.len(), 1);
        assert_eq!(status.entries[0].path, "src/app.cpp");
    }

    #[test]
    fn ide_metadata_and_empty_paths_are_dropped() {
        let body = concat!(
            "# branch.head main\0",
            "? .kinein/workspace.json\0",
            "? modulo/\0",
            "? modulo/x.txt\0",
        );
        let status = parse_status(body, "modulo/");
        let paths: Vec<&str> = status
            .entries
            .iter()
            .map(|entry| entry.path.as_str())
            .collect();
        assert_eq!(paths, vec!["x.txt"]);
    }

    #[test]
    fn hunk_headers_become_gutter_marks() {
        let diff = concat!(
            "diff --git a/x b/x\n",
            "index 111..222 100644\n",
            "--- a/x\n",
            "+++ b/x\n",
            "@@ -3,0 +4,2 @@ contexto\n",
            "+nova a\n",
            "+nova b\n",
            "@@ -10 +11 @@\n",
            "-antiga\n",
            "+trocada\n",
            "@@ -20,3 +20,0 @@\n",
            "-some a\n",
            "-some b\n",
            "-some c\n",
        );
        let hunks = parse_hunks(diff);
        assert_eq!(hunks.len(), 3);
        assert_eq!(
            (hunks[0].kind, hunks[0].start_line, hunks[0].line_count),
            (super::GitHunkKind::Added, 4, 2)
        );
        assert_eq!(
            (hunks[1].kind, hunks[1].start_line, hunks[1].line_count),
            (super::GitHunkKind::Modified, 11, 1)
        );
        assert_eq!(
            (hunks[2].kind, hunks[2].start_line, hunks[2].line_count),
            (super::GitHunkKind::Removed, 21, 1)
        );
    }

    #[test]
    fn added_and_intent_states_consolidate() {
        let body = concat!(
            "# branch.head main\0",
            "1 A. N... 000000 100644 100644 h h novo-staged.rs\0",
            "1 .A N... 000000 100644 100644 h h intent.rs\0",
        );
        let status = parse_status(body, "");
        assert_eq!(status.entries[0].kind, GitEntryKind::Added);
        assert!(status.entries[0].staged);
        assert_eq!(status.entries[1].kind, GitEntryKind::Added);
        assert!(!status.entries[1].staged);
    }
}

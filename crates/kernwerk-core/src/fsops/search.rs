//! Literal content search with a deterministic depth-first walk.

use std::{fs, path::Path};

use kernwerk_protocol::FsSearchMatch;

use super::{FsError, MAX_READ_BYTES, MAX_SEARCH_MATCHES, SEARCH_SKIP_DIRS};

/// Maximum preview length of a search match, in characters.
const MAX_PREVIEW_CHARS: usize = 200;

/// Searches every UTF-8 text file of the workspace for a literal query.
///
/// The walk is depth-first in case-insensitive name order, so results are
/// deterministic. Directories in [`SEARCH_SKIP_DIRS`], files larger than
/// [`MAX_READ_BYTES`], non-UTF-8 files, and symlinks are skipped silently;
/// individual IO failures skip the entry instead of aborting the search.
/// At most one match per line is reported, capped at [`MAX_SEARCH_MATCHES`].
pub fn search(
    root: &Path,
    query: &str,
    case_sensitive: bool,
) -> Result<(Vec<FsSearchMatch>, bool), FsError> {
    let mut matches = Vec::new();
    let mut pending_dirs = vec![root.to_path_buf()];

    while let Some(directory) = pending_dirs.pop() {
        let Ok(read_dir) = fs::read_dir(&directory) else {
            if directory == root {
                return Err(FsError::NotADirectory {
                    path: directory.display().to_string(),
                });
            }
            continue;
        };

        let mut files = Vec::new();
        let mut subdirs = Vec::new();
        for dir_entry in read_dir.flatten() {
            let Ok(file_type) = dir_entry.file_type() else {
                continue;
            };
            if file_type.is_symlink() {
                continue;
            }
            let name = dir_entry.file_name().to_string_lossy().into_owned();
            if file_type.is_dir() {
                if !SEARCH_SKIP_DIRS.contains(&name.as_str()) {
                    subdirs.push((name.to_lowercase(), dir_entry.path()));
                }
            } else if file_type.is_file() {
                files.push((name.to_lowercase(), dir_entry.path()));
            }
        }

        files.sort_by(|left, right| left.0.cmp(&right.0));
        subdirs.sort_by(|left, right| right.0.cmp(&left.0));
        pending_dirs.extend(subdirs.into_iter().map(|(_name, path)| path));

        for (_name, file) in files {
            if search_file(root, &file, query, case_sensitive, &mut matches) {
                return Ok((matches, true));
            }
        }
    }

    Ok((matches, false))
}

/// Searches one file, appending matches. Returns `true` when the cap is hit.
fn search_file(
    root: &Path,
    file: &Path,
    query: &str,
    case_sensitive: bool,
    matches: &mut Vec<FsSearchMatch>,
) -> bool {
    let Ok(metadata) = fs::metadata(file) else {
        return false;
    };
    if metadata.len() > MAX_READ_BYTES {
        return false;
    }
    let Ok(bytes) = fs::read(file) else {
        return false;
    };
    let Ok(content) = String::from_utf8(bytes) else {
        return false;
    };

    let relative = file
        .strip_prefix(root)
        .unwrap_or(file)
        .display()
        .to_string();
    for (index, line) in content.lines().enumerate() {
        let Some(offset) = find_literal(line, query, case_sensitive) else {
            continue;
        };
        let column = line[..offset].chars().count() as u64 + 1;
        matches.push(FsSearchMatch {
            path: relative.clone(),
            line: index as u64 + 1,
            column,
            preview: line.trim().chars().take(MAX_PREVIEW_CHARS).collect(),
        });
        if matches.len() >= MAX_SEARCH_MATCHES {
            return true;
        }
    }
    false
}

/// Finds the byte offset of the first literal occurrence of `query` in `line`.
///
/// Case-insensitive comparison is ASCII-only, which keeps byte offsets exact.
fn find_literal(line: &str, query: &str, case_sensitive: bool) -> Option<usize> {
    if case_sensitive {
        return line.find(query);
    }
    let line_bytes = line.as_bytes();
    let query_bytes = query.as_bytes();
    if query_bytes.is_empty() || query_bytes.len() > line_bytes.len() {
        return None;
    }
    line_bytes
        .windows(query_bytes.len())
        .position(|window| window.eq_ignore_ascii_case(query_bytes))
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use super::super::MAX_SEARCH_MATCHES;
    use super::search;

    fn temp_root(test_name: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("kernwerk-fsops-search-tests")
            .join(format!("{}-{test_name}", std::process::id()));
        if dir.exists() {
            fs::remove_dir_all(&dir).unwrap();
        }
        fs::create_dir_all(&dir).unwrap();
        dir.canonicalize().unwrap()
    }

    #[test]
    fn search_finds_matches_case_insensitive_by_default() {
        let root = temp_root("search-basic");
        fs::create_dir(root.join("src")).unwrap();
        fs::write(
            root.join("src/main.rs"),
            "fn main() {\n    Somar(2, 3);\n}\n",
        )
        .unwrap();
        fs::write(root.join("notas.txt"), "sem ocorrencia\n").unwrap();

        let (matches, truncated) = search(&root, "somar", false).unwrap();

        assert!(!truncated);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].path, "src/main.rs");
        assert_eq!(matches[0].line, 2);
        assert_eq!(matches[0].column, 5);
        assert_eq!(matches[0].preview, "Somar(2, 3);");
    }

    #[test]
    fn search_respects_case_sensitive_flag() {
        let root = temp_root("search-case");
        fs::write(root.join("a.txt"), "Alpha\nalpha\n").unwrap();

        let (insensitive, _) = search(&root, "alpha", false).unwrap();
        let (sensitive, _) = search(&root, "alpha", true).unwrap();

        assert_eq!(insensitive.len(), 2);
        assert_eq!(sensitive.len(), 1);
        assert_eq!(sensitive[0].line, 2);
    }

    #[test]
    fn search_skips_build_dirs_binaries_and_symlinks() {
        let root = temp_root("search-skip");
        fs::create_dir(root.join("target")).unwrap();
        fs::write(root.join("target/gerado.rs"), "alvo aqui\n").unwrap();
        fs::write(root.join("blob.bin"), [0xFF, 0x00, b'a', b'l', b'v', b'o']).unwrap();
        fs::write(root.join("fonte.rs"), "alvo aqui\n").unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(root.join("fonte.rs"), root.join("link.rs")).unwrap();

        let (matches, _) = search(&root, "alvo", false).unwrap();

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].path, "fonte.rs");
    }

    #[test]
    fn search_truncates_at_the_match_cap() {
        let root = temp_root("search-cap");
        let line = "ocorrencia\n".repeat(MAX_SEARCH_MATCHES + 10);
        fs::write(root.join("muitos.txt"), line).unwrap();

        let (matches, truncated) = search(&root, "ocorrencia", false).unwrap();

        assert!(truncated);
        assert_eq!(matches.len(), MAX_SEARCH_MATCHES);
    }

    #[test]
    fn search_walks_in_deterministic_order() {
        let root = temp_root("search-order");
        fs::create_dir(root.join("zeta")).unwrap();
        fs::create_dir(root.join("alfa")).unwrap();
        fs::write(root.join("zeta/z.txt"), "x\n").unwrap();
        fs::write(root.join("alfa/a.txt"), "x\n").unwrap();
        fs::write(root.join("raiz.txt"), "x\n").unwrap();

        let (matches, _) = search(&root, "x", false).unwrap();

        let paths = matches
            .iter()
            .map(|entry| entry.path.as_str())
            .collect::<Vec<_>>();
        assert_eq!(paths, ["raiz.txt", "alfa/a.txt", "zeta/z.txt"]);
    }
}

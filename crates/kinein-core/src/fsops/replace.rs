//! Substituicao literal multi-arquivo, confinada e transacional.

use std::{fs, path::Path};

use super::{FsError, MAX_READ_BYTES, SEARCH_SKIP_DIRS, TextFileUpdate, write_text_transaction};
use crate::fsops::search::find_literal;

const MAX_REPLACE_FILES: usize = 256;
const MAX_REPLACEMENTS: u64 = 20_000;
const MAX_REPLACE_BYTES: usize = 16 * 1_048_576;

/// Substitui todas as ocorrencias literais em arquivos de texto elegiveis.
///
/// Todos os resultados sao calculados antes da primeira escrita. A gravacao
/// compartilhada com `WorkspaceEdit` valida snapshots e faz rollback inverso.
pub fn replace(
    root: &Path,
    query: &str,
    replacement: &str,
    case_sensitive: bool,
) -> Result<(Vec<String>, u64), FsError> {
    let mut updates = Vec::new();
    let mut replacements = 0_u64;
    let mut bytes = 0_usize;
    let mut pending_dirs = vec![root.to_path_buf()];

    while let Some(directory) = pending_dirs.pop() {
        let read_dir = fs::read_dir(&directory).map_err(|source| FsError::Io {
            path: directory.display().to_string(),
            source,
        })?;
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

        for (_name, path) in files {
            let Ok(metadata) = fs::metadata(&path) else {
                continue;
            };
            if metadata.len() > MAX_READ_BYTES {
                continue;
            }
            let Ok(raw) = fs::read(&path) else {
                continue;
            };
            let Ok(content) = String::from_utf8(raw) else {
                continue;
            };
            let (rewritten, count) = replace_literal(&content, query, replacement, case_sensitive);
            if count == 0 {
                continue;
            }
            replacements = replacements.saturating_add(count);
            bytes = bytes
                .saturating_add(content.len())
                .saturating_add(rewritten.len());
            if updates.len() >= MAX_REPLACE_FILES
                || replacements > MAX_REPLACEMENTS
                || bytes > MAX_REPLACE_BYTES
            {
                return Err(FsError::ReplaceLimit {
                    message: format!(
                        "substituicao excede os limites de {MAX_REPLACE_FILES} arquivos, \
                         {MAX_REPLACEMENTS} ocorrencias ou {MAX_REPLACE_BYTES} bytes"
                    ),
                });
            }
            updates.push(TextFileUpdate {
                path,
                expected_content: content,
                new_content: rewritten,
            });
        }
    }

    write_text_transaction(root, &updates)?;
    let files = updates
        .into_iter()
        .map(|update| update.path.display().to_string())
        .collect();
    Ok((files, replacements))
}

fn replace_literal(
    content: &str,
    query: &str,
    replacement: &str,
    case_sensitive: bool,
) -> (String, u64) {
    if query.is_empty() {
        return (content.to_owned(), 0);
    }
    let mut rewritten = String::with_capacity(content.len());
    let mut cursor = 0_usize;
    let mut count = 0_u64;
    while let Some(relative) = find_literal(&content[cursor..], query, case_sensitive) {
        let start = cursor + relative;
        let end = start + query.len();
        rewritten.push_str(&content[cursor..start]);
        rewritten.push_str(replacement);
        cursor = end;
        count = count.saturating_add(1);
    }
    rewritten.push_str(&content[cursor..]);
    (rewritten, count)
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use super::{replace, replace_literal};

    fn root(name: &str) -> PathBuf {
        let root = std::env::temp_dir()
            .join("kinein-replace-tests")
            .join(format!("{}-{name}", std::process::id()));
        if root.exists() {
            fs::remove_dir_all(&root).unwrap();
        }
        fs::create_dir_all(&root).unwrap();
        root.canonicalize().unwrap()
    }

    #[test]
    fn literal_replace_handles_all_occurrences_and_growth() {
        let (text, count) = replace_literal("old old OLD", "old", "new-value", false);
        assert_eq!(text, "new-value new-value new-value");
        assert_eq!(count, 3);
    }

    #[test]
    fn project_replace_skips_binary_and_build_directories() {
        let root = root("walk");
        fs::create_dir(root.join("target")).unwrap();
        fs::write(root.join("a.txt"), "Alpha alpha\n").unwrap();
        fs::write(root.join("target/generated.txt"), "alpha\n").unwrap();
        fs::write(root.join("binary.bin"), [0xFF, b'a']).unwrap();

        let (files, count) = replace(&root, "alpha", "beta", false).unwrap();

        assert_eq!(files, [root.join("a.txt").display().to_string()]);
        assert_eq!(count, 2);
        assert_eq!(
            fs::read_to_string(root.join("a.txt")).unwrap(),
            "beta beta\n"
        );
        assert_eq!(
            fs::read_to_string(root.join("target/generated.txt")).unwrap(),
            "alpha\n"
        );
    }
}

//! Substituicao literal multi-arquivo, confinada e transacional.

use std::{fs, path::Path};

use super::walk::walk_text_files;
use super::{FsError, MAX_READ_BYTES, TextFileUpdate, write_text_transaction};
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
    let mut estourou_limite = false;

    walk_text_files(root, |path| {
        let Ok(metadata) = fs::metadata(path) else {
            return false;
        };
        if metadata.len() > MAX_READ_BYTES {
            return false;
        }
        let Ok(raw) = fs::read(path) else {
            return false;
        };
        let Ok(content) = String::from_utf8(raw) else {
            return false;
        };
        let (rewritten, count) = replace_literal(&content, query, replacement, case_sensitive);
        if count == 0 {
            return false;
        }
        replacements = replacements.saturating_add(count);
        bytes = bytes
            .saturating_add(content.len())
            .saturating_add(rewritten.len());
        if updates.len() >= MAX_REPLACE_FILES
            || replacements > MAX_REPLACEMENTS
            || bytes > MAX_REPLACE_BYTES
        {
            estourou_limite = true;
            return true;
        }
        updates.push(TextFileUpdate {
            path: path.to_path_buf(),
            expected_content: content,
            new_content: rewritten,
        });
        false
    })?;

    if estourou_limite {
        return Err(FsError::ReplaceLimit {
            message: format!(
                "substituicao excede os limites de {MAX_REPLACE_FILES} arquivos, \
                 {MAX_REPLACEMENTS} ocorrencias ou {MAX_REPLACE_BYTES} bytes"
            ),
        });
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

    /// `fs.replace` nao pode tocar arquivo que `fs.search` nao mostrou.
    ///
    /// O usuario le o resultado da busca e manda substituir: se os dois
    /// caminharem por conjuntos diferentes, a IDE edita as costas do usuario.
    /// Enquanto cada um tinha a sua copia do walk isso era coincidencia — e as
    /// copias JA divergiam (diretorio ilegivel abortava o replace inteiro e era
    /// apenas pulado na busca). Hoje os dois usam `fsops::walk`.
    #[test]
    fn replace_touches_exactly_the_files_search_reports() {
        let root = root("paridade-com-a-busca");
        fs::create_dir(root.join("src")).unwrap();
        fs::create_dir(root.join("target")).unwrap();
        fs::create_dir(root.join(".git")).unwrap();
        fs::write(root.join("src/main.rs"), "let alvo = 1;\n").unwrap();
        fs::write(root.join("raiz.txt"), "alvo na raiz\n").unwrap();
        fs::write(root.join("target/gerado.rs"), "alvo gerado\n").unwrap();
        fs::write(root.join(".git/config"), "alvo versionado\n").unwrap();
        fs::write(root.join("blob.bin"), [0xFF, b'a', b'l', b'v', b'o']).unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(root.join("raiz.txt"), root.join("link.txt")).unwrap();

        let (encontrados, _) = super::super::search::search(&root, "alvo", false).unwrap();
        let mut vistos_pela_busca = encontrados
            .iter()
            .map(|item| item.path.clone())
            .collect::<Vec<_>>();
        vistos_pela_busca.sort();
        vistos_pela_busca.dedup();

        let (arquivos, _) = replace(&root, "alvo", "novo", false).unwrap();
        let mut tocados = arquivos
            .iter()
            .map(|absoluto| {
                std::path::Path::new(absoluto)
                    .strip_prefix(&root)
                    .unwrap()
                    .display()
                    .to_string()
            })
            .collect::<Vec<_>>();
        tocados.sort();

        assert_eq!(tocados, vistos_pela_busca);
        assert_eq!(tocados, ["raiz.txt", "src/main.rs"]);
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

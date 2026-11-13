//! Leitura limitada e extracao estrutural com a identidade documental do contexto.

use super::{IndexedFile, MAX_FILE_BYTES, context::CompileContext, language_of, relative_path};
use crate::lang::extract::SymbolExtractor;
use kinein_protocol::{IndexSymbol, SyntaxOutlineItem};
use std::{collections::BTreeMap, path::Path};

/// Classifica, le e parseia UM arquivo. Todo arquivo entra (contado); so' os
/// de fonte sao lidos; so' os com gramatica ganham simbolos.
pub(super) fn index_file(
    root: &Path,
    absolute_path: &Path,
    extractor: &mut SymbolExtractor,
    skipped: &mut Vec<String>,
    context: Option<&CompileContext>,
) -> Option<IndexedFile> {
    let path = relative_path(root, absolute_path);
    let meta = std::fs::metadata(absolute_path).ok()?;
    let language = context.map_or_else(
        || language_of(absolute_path),
        |ctx| ctx.document_language(absolute_path),
    );
    if language == "other" {
        return Some(IndexedFile {
            path,
            language: language.to_owned(),
            lines: 0,
            bytes: meta.len(),
            symbols: Vec::new(),
        });
    }
    if meta.len() > MAX_FILE_BYTES {
        skipped.push(format!("{path} ({} bytes: acima do teto)", meta.len()));
        return Some(IndexedFile {
            path,
            language: language.to_owned(),
            lines: 0,
            bytes: meta.len(),
            symbols: Vec::new(),
        });
    }
    // Sem esperar num FIFO; o teto e' o do indice (INF-18, 63 §A.9).
    let Ok(buffer) = crate::fsops::read_project_text(absolute_path) else {
        skipped.push(format!("{path} (não é UTF-8 legível)"));
        return Some(IndexedFile {
            path,
            language: language.to_owned(),
            lines: 0,
            bytes: meta.len(),
            symbols: Vec::new(),
        });
    };
    let lines = buffer.lines().count() as u64;
    let mut symbols = Vec::new();
    if let Some(outline) = extractor.symbols_with_language(language, &buffer) {
        flatten_outline(&outline, None, &path, language, &mut symbols);
        deduplicate(&mut symbols);
    }
    Some(IndexedFile {
        path,
        language: language.to_owned(),
        lines,
        bytes: meta.len(),
        symbols,
    })
}

/// O outline aninhado do editor vira lista plana com `container`.
fn flatten_outline(
    items: &[SyntaxOutlineItem],
    container: Option<&str>,
    path: &str,
    language: &str,
    output: &mut Vec<IndexSymbol>,
) {
    for item in items {
        output.push(IndexSymbol {
            name: item.name.clone(),
            kind: item.kind.clone(),
            path: path.to_owned(),
            language: language.to_owned(),
            line: item.line,
            end_line: item.end_line,
            container: container.map(str::to_owned),
        });
        flatten_outline(&item.children, Some(&item.name), path, language, output);
    }
}

/// A query `tags` do Rust captura um `fn` dentro de `impl` DUAS vezes —
/// `definition.function` e `definition.method` (medido em 2026-09-12 no
/// proprio repositorio: `handle_request` aparecia em dobro). Uma declaracao
/// e' uma: fica a mais especifica (method) por (linha, nome).
fn deduplicate(symbols: &mut Vec<IndexSymbol>) {
    let mut seen: BTreeMap<(u64, String), usize> = BTreeMap::new();
    let mut keep: Vec<bool> = vec![true; symbols.len()];
    for (i, s) in symbols.iter().enumerate() {
        let key = (s.line, s.name.clone());
        match seen.get(&key) {
            None => {
                seen.insert(key, i);
            }
            Some(&previous) => {
                if specificity(&s.kind) > specificity(&symbols[previous].kind) {
                    keep[previous] = false;
                    seen.insert(key, i);
                } else {
                    keep[i] = false;
                }
            }
        }
    }
    let mut i = 0;
    symbols.retain(|_| {
        let retain = keep[i];
        i += 1;
        retain
    });
}

fn specificity(kind: &str) -> u8 {
    match kind {
        "method" | "constructor" => 2,
        "function" => 1,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::index_file;
    use crate::lang::extract::SymbolExtractor;

    /// As linhas e os símbolos do arquivo, quando entra, e quantos ficaram
    /// de fora com o motivo.
    fn indexed(root: &Path, path: &Path) -> (Option<(u64, usize)>, usize) {
        let mut skipped = Vec::new();
        let file = index_file(
            root,
            path,
            &mut SymbolExtractor::default(),
            &mut skipped,
            None,
        );
        (
            file.map(|file| (file.lines, file.symbols.len())),
            skipped.len(),
        )
    }

    /// INF-18: a fonte que não é arquivo regular entra sem ser lida e com o
    /// motivo, sem esperar num FIFO; a varredura do índice já a deixa de fora.
    #[test]
    fn a_source_that_is_not_a_regular_file_is_counted_without_reading() {
        let root: PathBuf = std::env::temp_dir()
            .join("kinein-index-file-fifo")
            .join(std::process::id().to_string());
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join("main.c");
        std::fs::write(&path, "int main(void) { return 0; }\n").unwrap();
        assert_eq!(indexed(&root, &path), (Some((1, 1)), 0));
        std::fs::remove_file(&path).unwrap();
        let unread = (Some((0, 0)), 1);
        std::fs::create_dir(&path).unwrap();
        assert_eq!(indexed(&root, &path), unread);
        std::fs::remove_dir(&path).unwrap();
        if crate::fsops::fifo(&path) {
            let (project, held) = (root.clone(), path.clone());
            assert_eq!(
                crate::fsops::unblocked(&path, move || indexed(&project, &held)),
                unread
            );
        }
        let _ = std::fs::remove_dir_all(&root);
    }
}

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
    let Ok(buffer) = std::fs::read_to_string(absolute_path) else {
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

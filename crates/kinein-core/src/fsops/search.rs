//! Literal content search with a deterministic depth-first walk.
//!
//! # A busca casa no CONTEUDO, e nao linha a linha (2026-09-02)
//!
//! Ate a etapa 9 do `roadmaps/30` ela iterava `content.lines()`, o que a tornava
//! incapaz de achar uma query com `\n` — e era por isso que o `fs.replace`
//! RECUSAVA quebra de linha: a substituicao casa no conteudo inteiro, entao
//! aceitar `\n` significaria reescrever arquivos a partir de um preview que
//! devolveu "0 resultados". Transacao, snapshot e rollback protegem contra
//! falha de ESCRITA, nao contra o usuario aprovar o que nao viu.
//!
//! Agora as duas casam do mesmo jeito, no mesmo texto, com o mesmo
//! `find_literal`. **Essa igualdade e a invariante desta fatia**, e ela ja tinha
//! teste antes de existir multi-linha: o numero de resultados da busca tem de
//! ser o numero de substituicoes do replace. Preview de operacao destrutiva que
//! conta errado e pior que preview nenhum.

use std::{fs, path::Path};

use kinein_protocol::FsSearchMatch;

use super::walk::walk_text_files;
use super::{FsError, MAX_READ_BYTES, MAX_SEARCH_MATCHES};

/// Maximum preview length of a search match, in characters.
const MAX_PREVIEW_CHARS: usize = 200;

/// Marca a quebra de linha dentro do preview de um casamento multi-linha.
///
/// Um `\n` cru viraria uma linha nova na lista de resultados e desalinharia a
/// tabela; o simbolo mantem o resultado em UMA linha e diz que ha quebra ali.
const LINE_BREAK_MARK: &str = " ⏎ ";

/// Searches every UTF-8 text file of the workspace for a literal query.
///
/// The walk is depth-first in case-insensitive name order, so results are
/// deterministic. Directories in [`SEARCH_SKIP_DIRS`], files larger than
/// [`MAX_READ_BYTES`], non-UTF-8 files, and symlinks are skipped silently;
/// individual IO failures skip the entry instead of aborting the search.
/// Every occurrence is reported — including matches that span lines — capped at
/// [`MAX_SEARCH_MATCHES`].
pub fn search(
    root: &Path,
    query: &str,
    case_sensitive: bool,
) -> Result<(Vec<FsSearchMatch>, bool), FsError> {
    let mut matches = Vec::new();
    let truncated = walk_text_files(root, |file| {
        search_file(root, file, query, case_sensitive, &mut matches)
    })?;

    Ok((matches, truncated))
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

    // TODAS as ocorrencias, nao so a primeira de cada linha.
    //
    // `fs.replace` substitui todas; enquanto a busca parava na primeira, a
    // linha "Alpha alpha" aparecia como 1 resultado e virava 2 substituicoes.
    // O preview de uma operacao destrutiva tem de contar o que ela vai fazer.
    let mut cursor = 0_usize;
    // Posicao humana do inicio da linha corrente, avancada INCREMENTALMENTE.
    // Recontar do inicio do arquivo a cada casamento seria quadratico num
    // arquivo com muitas ocorrencias.
    let mut line_number = 1_u64;
    let mut line_start = 0_usize;

    while let Some(offset) = find_literal(&content[cursor..], query, case_sensitive) {
        let start = cursor + offset;
        for (index, byte) in content.as_bytes().iter().enumerate().skip(line_start) {
            if index >= start {
                break;
            }
            if *byte == b'\n' {
                line_number += 1;
                line_start = index + 1;
            }
        }
        let column = content[line_start..start].chars().count() as u64 + 1;
        let end = start.saturating_add(query.len()).min(content.len());
        matches.push(FsSearchMatch {
            path: relative.clone(),
            line: line_number,
            column,
            preview: preview_for(content.as_str(), line_start, start, end),
        });
        if matches.len() >= MAX_SEARCH_MATCHES {
            return true;
        }
        // Avanca o comprimento da QUERY: `find_literal` casa byte a byte
        // (case-insensitive e' ASCII), entao o casamento tem o mesmo tamanho e
        // `start + len` cai em fronteira de caractere.
        cursor = end;
    }
    false
}

/// Texto de preview de um casamento.
///
/// Casamento de UMA linha mostra a linha inteira (contexto e o que ajuda a
/// reconhecer o lugar). Casamento MULTI-LINHA mostra o texto CASADO, com as
/// quebras visiveis: mostrar so a primeira linha faria um casamento de tres
/// linhas parecer um de uma — e o usuario aprovaria uma reescrita maior do que
/// a que viu.
fn preview_for(content: &str, line_start: usize, start: usize, end: usize) -> String {
    if content[start..end].contains('\n') {
        return content[start..end]
            .replace('\n', LINE_BREAK_MARK)
            .trim()
            .chars()
            .take(MAX_PREVIEW_CHARS)
            .collect();
    }
    let line_end = content[line_start..]
        .find('\n')
        .map_or(content.len(), |offset| line_start + offset);
    content[line_start..line_end]
        .trim()
        .chars()
        .take(MAX_PREVIEW_CHARS)
        .collect()
}

/// Finds the byte offset of the first literal occurrence of `query` in `line`.
///
/// Case-insensitive comparison is ASCII-only, which keeps byte offsets exact.
pub(super) fn find_literal(line: &str, query: &str, case_sensitive: bool) -> Option<usize> {
    if case_sensitive {
        return line.find(query);
    }
    let line_bytes = line.as_bytes();
    let query_bytes = query.as_bytes();
    if query_bytes.is_empty() || query_bytes.len() > line_bytes.len() {
        return None;
    }
    line.char_indices().find_map(|(offset, _character)| {
        let end = offset.saturating_add(query_bytes.len());
        (end <= line_bytes.len()
            && line.is_char_boundary(end)
            && line_bytes[offset..end].eq_ignore_ascii_case(query_bytes))
        .then_some(offset)
    })
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use super::super::MAX_SEARCH_MATCHES;
    use super::search;

    fn temp_root(test_name: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("kinein-fsops-search-tests")
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

    /// A busca tem de CONTAR o que a substituicao vai fazer.
    ///
    /// Enquanto ela parava na primeira ocorrencia da linha, "Alpha alpha"
    /// aparecia como 1 resultado e `fs.replace` devolvia 2 substituicoes. O
    /// usuario aprova uma operacao destrutiva olhando essa lista.
    #[test]
    fn search_reports_every_occurrence_of_a_line() {
        let root = temp_root("search-todas-da-linha");
        fs::write(
            root.join("a.txt"),
            "Alpha alpha ALPHA
sem nada
",
        )
        .unwrap();

        let (matches, truncated) = search(&root, "alpha", false).unwrap();

        assert!(!truncated);
        assert_eq!(matches.len(), 3);
        assert!(matches.iter().all(|item| item.line == 1));
        let colunas = matches.iter().map(|item| item.column).collect::<Vec<_>>();
        assert_eq!(colunas, [1, 7, 13]);

        // E o numero tem de bater com o do replace, que e' o ponto.
        let (_, substituicoes) =
            super::super::replace::replace(&root, "alpha", "beta", false).unwrap();
        assert_eq!(substituicoes, matches.len() as u64);
    }

    /// Ocorrencias sobrepostas nao podem gerar lista infinita nem casamento
    /// duplo: o cursor avanca o comprimento da query.
    #[test]
    fn search_does_not_rescan_inside_a_match() {
        let root = temp_root("search-sobreposta");
        fs::write(
            root.join("a.txt"),
            "aaaa
",
        )
        .unwrap();

        let (matches, _) = search(&root, "aa", true).unwrap();

        assert_eq!(matches.len(), 2);
        assert_eq!(
            matches.iter().map(|item| item.column).collect::<Vec<_>>(),
            [1, 3]
        );
    }

    /// Casamento que ATRAVESSA linhas: posicao do inicio e preview honesto.
    ///
    /// O preview de uma linha so mostraria "primeira" e faria um casamento de
    /// duas linhas parecer de uma — o usuario aprovaria uma reescrita maior do
    /// que a que viu.
    #[test]
    fn search_finds_a_match_that_spans_lines() {
        let root = temp_root("search-multilinha");
        // A ISCA e a segunda "primeira": ela casa o COMECO da query e nao a
        // query. Sem ela, uma busca que truncasse a query no primeiro `\n`
        // passaria neste teste por coincidencia — e foi exatamente o que a
        // primeira versao dele deixou passar na prova por mutacao.
        fs::write(
            root.join("a.txt"),
            "antes\n  primeira\nsegunda\ndepois\nprimeira\noutra\n",
        )
        .unwrap();

        let (matches, _) = search(&root, "primeira\nsegunda", true).unwrap();

        assert_eq!(matches.len(), 1, "a query inteira casa uma vez so");
        assert_eq!(matches[0].line, 2, "a linha e a do INICIO do casamento");
        assert_eq!(matches[0].column, 3, "e a coluna tambem, contando o recuo");
        assert_eq!(matches[0].preview, "primeira ⏎ segunda");
    }

    /// A contagem da busca continua sendo a do replace quando ha multi-linha.
    ///
    /// E a invariante que tornou legitimo remover a recusa de `\n` no
    /// `fs.replace`: preview que conta errado e pior que preview nenhum.
    #[test]
    fn multiline_search_and_replace_agree_on_the_count() {
        let root = temp_root("search-multilinha-contagem");
        fs::write(root.join("a.txt"), "ab\nab\nab\nab\n").unwrap();

        let (matches, _) = search(&root, "ab\nab", true).unwrap();
        let (_, substituicoes) =
            super::super::replace::replace(&root, "ab\nab", "X", true).unwrap();

        // Casamentos NAO se sobrepoem: "ab\nab\nab\nab" tem dois, nao tres.
        assert_eq!(matches.len(), 2);
        assert_eq!(substituicoes, matches.len() as u64);
        assert_eq!(fs::read_to_string(root.join("a.txt")).unwrap(), "X\nX\n");
    }

    /// Casamento de UMA linha continua mostrando a linha inteira como contexto.
    #[test]
    fn a_single_line_match_still_previews_the_whole_line() {
        let root = temp_root("search-preview-linha");
        fs::write(root.join("a.txt"), "  int total = somar(1, 2);  \n").unwrap();

        let (matches, _) = search(&root, "somar", true).unwrap();

        assert_eq!(matches[0].preview, "int total = somar(1, 2);");
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

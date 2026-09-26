//! Indentation derived from the grammar, for one cursor position.
//!
//! # Why this is a tree walk and not an indent query
//!
//! Editors that indent from Tree-sitter usually ship `indents.scm` queries with
//! captures such as `@indent`, `@outdent`, `@align` and `@branch`, plus the
//! logic that interprets them. That machinery is powerful and large, and it is
//! only worth its size once the set of supported languages is open.
//!
//! Here the set is closed and known: C, C++, Rust and Python. Walking the
//! ancestors of the node under the cursor and counting the ones that open a
//! block is smaller, explainable in one paragraph, and — the part that decides
//! it — testable without loading a query file.
//!
//! # The tree is almost always broken, and that is the normal case
//!
//! While someone types, the buffer is mid-edit: an unclosed brace, a statement
//! without its semicolon. Tree-sitter still returns a tree, with `ERROR` nodes
//! inside it. Every rule below has to hold on that tree, because that is the
//! tree it will see nearly every time.

use tree_sitter::{Node, Tree};

use super::registry::LanguageId;

/// What the editor should do with the line it is about to create or has just
/// closed.
#[derive(Debug, Clone, Eq, PartialEq)]
#[allow(clippy::redundant_pub_crate)]
pub(crate) struct IndentDecision {
    /// Indent levels for the line the editor is about to fix.
    ///
    /// Which line that is comes from the trigger the editor sent: `Newline`
    /// means the line being opened, `CloseDelimiter` means the one the author
    /// just typed into. An earlier version also carried a `dedent_to`, and it
    /// was removed once measured: for `CloseDelimiter` it held exactly the same
    /// number as `level`, and nothing ever read it.
    pub(crate) level: u32,
}

/// What made the editor ask.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[allow(clippy::redundant_pub_crate)]
pub(crate) enum IndentTrigger {
    /// Enter: decide the level of the line being opened.
    Newline,
    /// A closing delimiter was typed: decide the level of the current line.
    CloseDelimiter,
}

/// Node kinds that open one indentation level.
///
/// Deliberately a list and not a heuristic on the node name: `block` means
/// different things in different grammars, and guessing by substring would put
/// `parameter_list` and `block_comment` in the same bag.
const fn opens_block(language: LanguageId, kind: &str) -> bool {
    match language {
        LanguageId::C | LanguageId::Cpp => matches!(
            kind.as_bytes(),
            b"compound_statement"
                | b"field_declaration_list"
                | b"enumerator_list"
                | b"initializer_list"
                | b"argument_list"
                | b"parameter_list"
                | b"declaration_list"
                | b"init_declarator"
        ),
        LanguageId::Rust => matches!(
            kind.as_bytes(),
            b"block"
                | b"declaration_list"
                | b"field_declaration_list"
                | b"enum_variant_list"
                | b"match_block"
                | b"arguments"
                | b"parameters"
                | b"use_list"
                | b"field_initializer_list"
                // O grupo de delimitadores de uma macro: dentro de
                // `println!(` esta'-se um nivel mais fundo, e a gramatica nao
                // chama isso de `arguments`.
                | b"token_tree"
        ),
        LanguageId::Python => matches!(
            kind.as_bytes(),
            b"block" | b"argument_list" | b"parameters" | b"list" | b"dictionary" | b"set"
        ),
    }
}

/// The byte offset of the first character of the line containing `offset`.
fn line_start(content: &str, offset: usize) -> usize {
    content[..offset].rfind('\n').map_or(0, |index| index + 1)
}

/// The deepest node that contains the character *before* `offset`, on the tree
/// as it is — errors and all.
///
/// Probing at `offset` itself looks right and is wrong: a zero-width range at
/// the END of a node is not inside it, so a cursor sitting after the last typed
/// character resolves to some ancestor near the root and every block is missed.
/// Measured in 2026-09-25: four of seven cases answered level zero until this
/// stepped back one byte.
///
/// Stepping back is also what the question means. The cursor sits after what
/// was just typed, and what was just typed is what decides whether a block is
/// open.
fn node_at<'tree>(tree: &'tree Tree, content: &str, offset: usize) -> Node<'tree> {
    let root = tree.root_node();
    let probe = if offset == 0 {
        0
    } else {
        // Back to the previous character boundary, not the previous byte: a
        // cursor after an accented character would land mid-codepoint.
        content[..offset]
            .char_indices()
            .next_back()
            .map_or(0, |(index, _)| index)
    };
    root.descendant_for_byte_range(probe, probe).unwrap_or(root)
}

/// Counts the block ancestors that are already open at `offset`.
///
/// `None` means **the grammar cannot answer here**, and that is a legitimate
/// result rather than a failure — see the module note on broken trees.
///
/// A block only counts when it *starts on an earlier line*: a brace opened on
/// the current line is what the new line is being indented into, and counting
/// it twice is the classic double-indent bug.
fn open_levels(language: LanguageId, node: Node<'_>, cursor_line: usize) -> Option<u32> {
    let mut levels = 0_u32;
    let mut current = Some(node);
    while let Some(ancestor) = current {
        // AN ERROR ANCESTOR MEANS THE STRUCTURE AROUND THE CURSOR IS UNKNOWN.
        //
        // Measured in 2026-09-25: `fn main() {\n    if x {` — an unclosed brace
        // — parses to `{ < ERROR`, with no block node anywhere. Counting blocks
        // there answers ZERO, and zero is not "no opinion": the editor would
        // apply it and pull the line back to column one. Saying nothing keeps
        // the local fallback, which is right about this case.
        //
        // An error somewhere else in the file does not reach here, because only
        // ANCESTORS are walked.
        if ancestor.is_error() {
            return None;
        }
        if opens_block(language, ancestor.kind()) && ancestor.start_position().row < cursor_line {
            levels = levels.saturating_add(1);
        }
        current = ancestor.parent();
    }
    Some(levels)
}

/// Decides the indentation for one position.
///
/// `offset` is a byte offset into `content`, and points at the cursor.
/// `None` when the grammar has no usable structure at that position — the
/// caller keeps whatever it already applied.
#[allow(clippy::redundant_pub_crate)]
pub(crate) fn decide(
    language: LanguageId,
    tree: &Tree,
    content: &str,
    offset: usize,
    trigger: IndentTrigger,
) -> Option<IndentDecision> {
    let offset = offset.min(content.len());
    let start = line_start(content, offset);
    let cursor_line = content[..start].matches('\n').count();
    let node = node_at(tree, content, offset);

    match trigger {
        IndentTrigger::Newline => {
            // The line being opened lives inside whatever is open right now.
            // A delimiter opened ON this line counts, because the new line goes
            // inside it — which is why `cursor_line` is compared against the
            // node that starts before the *new* line, not before this one.
            let inside = open_levels(language, node, cursor_line.saturating_add(1))?;
            Some(IndentDecision { level: inside })
        }
        IndentTrigger::CloseDelimiter => {
            // The delimiter just typed closes the innermost block, so the line
            // it sits on belongs to the level OUTSIDE that block.
            let inside = open_levels(language, node, cursor_line)?;
            Some(IndentDecision {
                level: inside.saturating_sub(1),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{IndentTrigger, decide};
    use crate::lang::registry::{LanguageId, LanguageRegistry};

    use tree_sitter::{Parser, Tree};

    /// Parseia como o servico parseia, para o teste medir a MESMA arvore.
    fn parse(language: LanguageId, source: &str) -> Tree {
        let mut registry = LanguageRegistry::default();
        let runtime = registry.runtime(language).expect("gramatica");
        let mut parser = Parser::new();
        parser
            .set_language(runtime.language())
            .expect("linguagem valida");
        parser.parse(source, None).expect("arvore")
    }

    /// O cursor e' marcado com `|` no fonte do teste — ler o offset de um
    /// numero solto tornaria cada caso ilegivel.
    fn at_cursor(source: &str) -> (String, usize) {
        let offset = source.find('|').expect("o teste precisa marcar o cursor");
        (source.replace('|', ""), offset)
    }

    fn level(language: LanguageId, marked: &str, trigger: IndentTrigger) -> u32 {
        let (source, offset) = at_cursor(marked);
        let tree = parse(language, &source);
        decide(language, &tree, &source, offset, trigger)
            .expect("a gramatica deveria responder aqui")
            .level
    }

    /// O caso mais simples, e a regra inteira em uma frase: a linha nova mora
    /// dentro do que estiver aberto.
    #[test]
    fn dentro_de_um_bloco_a_linha_nova_indenta() {
        assert_eq!(
            level(
                LanguageId::C,
                "int main(void) {|\n}\n",
                IndentTrigger::Newline
            ),
            1
        );
        assert_eq!(
            level(
                LanguageId::Rust,
                "fn main() {|\n}\n",
                IndentTrigger::Newline
            ),
            1
        );
    }

    /// Blocos aninhados somam, e e' o que separa esta regra da heuristica
    /// local: ela conta a linha anterior, esta conta a ESTRUTURA.
    #[test]
    fn blocos_aninhados_somam() {
        let fonte = "fn main() {\n    if x {\n        loop {|\n        }\n    }\n}\n";
        assert_eq!(level(LanguageId::Rust, fonte, IndentTrigger::Newline), 3);
    }

    /// A ARVORE QUEBRADA NAO INVENTA RESPOSTA, e isto foi medido: com uma
    /// chave sem fechamento, a arvore do Rust vira `{ < ERROR` — sem bloco
    /// nenhum para contar. Responder ZERO ali nao seria "sem opiniao": a UI
    /// aplicaria e puxaria a linha para a coluna um.
    ///
    /// Na IDE este caso e' raro porque o auto-close ja' inseriu o fechamento;
    /// ele aparece ao colar e com o auto-close desligado. Nos dois, o fallback
    /// local do editor e' quem sabe responder.
    #[test]
    fn arvore_com_erro_nao_responde() {
        for (lang, fonte) in [
            (LanguageId::Rust, "fn main() {\n    if x {|\n"),
            (LanguageId::C, "int main(void) {\n    while (1) {|\n"),
        ] {
            let (source, offset) = at_cursor(fonte);
            let tree = parse(lang, &source);
            assert_eq!(
                decide(lang, &tree, &source, offset, IndentTrigger::Newline),
                None,
                "{fonte:?} deveria ficar sem resposta"
            );
        }
    }

    /// E o MESMO texto com o fechamento que o auto-close poe — que e' o que a
    /// IDE realmente tem no buffer — responde normalmente.
    #[test]
    fn com_o_fechamento_do_auto_close_a_gramatica_responde() {
        assert_eq!(
            level(
                LanguageId::Rust,
                "fn main() {\n    if x {|}\n}\n",
                IndentTrigger::Newline
            ),
            2
        );
        assert_eq!(
            level(
                LanguageId::C,
                "int main(void) {\n    while (1) {|}\n}\n",
                IndentTrigger::Newline
            ),
            2
        );
    }

    /// O `}` digitado pertence ao nivel de FORA do bloco que ele fecha — e a
    /// correcao cai na linha atual, nao na proxima.
    #[test]
    fn fechar_bloco_dedenta_a_linha_atual() {
        let (fonte, offset) = at_cursor("fn main() {\n    if x {\n    }|\n}\n");
        let arvore = parse(LanguageId::Rust, &fonte);
        let decisao = decide(
            LanguageId::Rust,
            &arvore,
            &fonte,
            offset,
            IndentTrigger::CloseDelimiter,
        )
        .expect("texto fechado, a gramatica responde");
        assert_eq!(decisao.level, 1, "a linha do `}}` interno");
    }

    /// Python nao tem chave: o bloco vem do `:`, e o `block` da gramatica e'
    /// quem carrega a estrutura.
    #[test]
    fn python_indenta_pelo_bloco_da_gramatica() {
        let fonte = "def f():\n    if x:\n        return 1|\n";
        assert_eq!(level(LanguageId::Python, fonte, IndentTrigger::Newline), 2);
    }

    /// Fora de qualquer bloco o nivel e' zero — e a regra nao pode inventar
    /// indentacao para o topo do arquivo.
    #[test]
    fn no_topo_do_arquivo_o_nivel_e_zero() {
        assert_eq!(
            level(
                LanguageId::Rust,
                "|\nfn main() {}\n",
                IndentTrigger::Newline
            ),
            0
        );
        assert_eq!(
            level(
                LanguageId::C,
                "#include <stdio.h>|\n",
                IndentTrigger::Newline
            ),
            0
        );
    }

    /// Lista de argumentos tambem abre nivel: quebrar uma chamada longa e' o
    /// caso que a heuristica local ja' acertava por acidente (termina em `(`) e
    /// que aqui e' estrutura.
    #[test]
    fn lista_de_argumentos_abre_nivel() {
        let fonte = "fn main() {\n    println!(|\n    );\n}\n";
        assert!(level(LanguageId::Rust, fonte, IndentTrigger::Newline) >= 2);
    }
}

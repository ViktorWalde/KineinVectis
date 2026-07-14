//! Structural folding range extraction from a concrete syntax tree.

use std::collections::BTreeSet;

use kinein_protocol::SyntaxFoldingRange;
use tree_sitter::{Node, Tree};

/// Returns deduplicated, document-ordered multi-line fold ranges.
pub(super) fn folding_ranges(tree: &Tree) -> Vec<SyntaxFoldingRange> {
    let mut ranges = BTreeSet::new();
    visit(tree.root_node(), &mut ranges);
    ranges
        .into_iter()
        .map(|(start_line, end_line)| SyntaxFoldingRange {
            start_line,
            end_line,
        })
        .collect()
}

fn visit(node: Node<'_>, ranges: &mut BTreeSet<(u64, u64)>) {
    if is_foldable(node.kind()) {
        let start_line = usize_to_line(node.start_position().row);
        let end_position = node.end_position();
        let end_line = if end_position.column == 0 {
            u64::try_from(end_position.row).unwrap_or(u64::MAX)
        } else {
            usize_to_line(end_position.row)
        };
        if end_line > start_line {
            ranges.insert((start_line, end_line));
        }
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        visit(child, ranges);
    }
}

fn usize_to_line(row: usize) -> u64 {
    u64::try_from(row.saturating_add(1)).unwrap_or(u64::MAX)
}

fn is_foldable(kind: &str) -> bool {
    matches!(
        kind,
        "comment"
            | "compound_statement"
            | "declaration_list"
            | "enumerator_list"
            | "field_declaration_list"
            | "initializer_list"
            | "namespace_definition"
            | "class_specifier"
            | "struct_specifier"
            | "union_specifier"
            | "enum_specifier"
            | "block"
            | "match_block"
            | "use_list"
            | "token_tree"
            | "impl_item"
            | "trait_item"
            | "mod_item"
    )
}

#[cfg(test)]
mod tests {
    use tree_sitter::Parser;

    use super::folding_ranges;

    #[test]
    fn rust_blocks_are_foldable_by_real_structure() {
        let mut parser = Parser::new();
        assert!(
            parser
                .set_language(&tree_sitter_rust::LANGUAGE.into())
                .is_ok()
        );
        let source = "fn main() {\n    if true {\n        work();\n    }\n}\n";
        let Some(tree) = parser.parse(source, None) else {
            return;
        };

        let ranges = folding_ranges(&tree);
        assert!(
            ranges
                .iter()
                .any(|range| range.start_line == 1 && range.end_line == 5)
        );
        assert!(
            ranges
                .iter()
                .any(|range| range.start_line == 2 && range.end_line == 4)
        );
    }
}

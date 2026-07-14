//! Conversion of upstream tags queries into a nested local outline.

use std::{cmp::Reverse, collections::BTreeSet};

use kinein_protocol::SyntaxOutlineItem;
use tree_sitter::{Query, QueryCursor, StreamingIterator, Tree};

use super::positions::utf16_position;

#[derive(Debug)]
struct RawOutline {
    item: SyntaxOutlineItem,
    start_byte: usize,
    end_byte: usize,
}

#[derive(Debug)]
struct ArenaOutline {
    item: SyntaxOutlineItem,
    children: Vec<usize>,
    start_byte: usize,
    end_byte: usize,
}

/// Executes a grammar's official tags query and nests declarations by range.
pub(super) fn outline(tree: &Tree, query: &Query, source: &str) -> Vec<SyntaxOutlineItem> {
    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(query, tree.root_node(), source.as_bytes());
    let capture_names = query.capture_names();
    let mut raw = Vec::new();
    let mut seen = BTreeSet::new();

    while let Some(query_match) = matches.next() {
        let mut definition = None;
        let mut name = None;
        for capture in query_match.captures {
            let Some(capture_name) = capture_names.get(capture.index as usize) else {
                continue;
            };
            if let Some(kind) = capture_name.strip_prefix("definition.") {
                definition = Some((capture.node, kind));
            } else if *capture_name == "name" {
                name = Some(capture.node);
            }
        }
        let (Some((definition_node, kind)), Some(name_node)) = (definition, name) else {
            continue;
        };
        let Some(display_name) = source.get(name_node.byte_range()).map(str::trim) else {
            continue;
        };
        if display_name.is_empty() {
            continue;
        }
        let key = (
            definition_node.start_byte(),
            definition_node.end_byte(),
            kind,
            display_name,
        );
        if !seen.insert(key) {
            continue;
        }
        let (line, column) = utf16_position(source, name_node.start_byte());
        raw.push(RawOutline {
            item: SyntaxOutlineItem {
                name: display_name.to_owned(),
                kind: kind.to_owned(),
                line,
                column: column.saturating_add(1),
                end_line: u64::try_from(definition_node.end_position().row.saturating_add(1))
                    .unwrap_or(u64::MAX),
                children: Vec::new(),
            },
            start_byte: definition_node.start_byte(),
            end_byte: definition_node.end_byte(),
        });
    }

    raw.sort_by_key(|entry| (entry.start_byte, Reverse(entry.end_byte)));
    nest(raw)
}

fn nest(raw: Vec<RawOutline>) -> Vec<SyntaxOutlineItem> {
    let mut arena: Vec<ArenaOutline> = Vec::with_capacity(raw.len());
    let mut roots = Vec::new();
    let mut stack: Vec<usize> = Vec::new();

    for entry in raw {
        while stack.last().is_some_and(|index| {
            let parent = &arena[*index];
            parent.start_byte == entry.start_byte
                || parent.end_byte < entry.end_byte
                || parent.start_byte > entry.start_byte
        }) {
            stack.pop();
        }
        let parent = stack.last().copied();
        let index = arena.len();
        arena.push(ArenaOutline {
            item: entry.item,
            children: Vec::new(),
            start_byte: entry.start_byte,
            end_byte: entry.end_byte,
        });
        if let Some(parent) = parent {
            arena[parent].children.push(index);
        } else {
            roots.push(index);
        }
        stack.push(index);
    }

    roots
        .into_iter()
        .map(|index| materialize(index, &arena))
        .collect()
}

fn materialize(index: usize, arena: &[ArenaOutline]) -> SyntaxOutlineItem {
    let node = &arena[index];
    let mut item = node.item.clone();
    item.children = node
        .children
        .iter()
        .map(|child| materialize(*child, arena))
        .collect();
    item
}

#[cfg(test)]
mod tests {
    use tree_sitter::{Parser, Query};

    use super::outline;

    #[test]
    fn rust_tags_produce_local_symbols_without_lsp() {
        let language = tree_sitter_rust::LANGUAGE.into();
        let mut parser = Parser::new();
        assert!(parser.set_language(&language).is_ok());
        let source = "struct Device;\nfn run() {}\n";
        let Some(tree) = parser.parse(source, None) else {
            return;
        };
        let query = Query::new(&language, tree_sitter_rust::TAGS_QUERY);
        assert!(query.is_ok());
        let Ok(query) = query else {
            return;
        };

        let items = outline(&tree, &query, source);
        assert!(
            items
                .iter()
                .any(|item| item.name == "Device" && item.kind == "class")
        );
        assert!(
            items
                .iter()
                .any(|item| item.name == "run" && item.kind == "function")
        );
    }
}

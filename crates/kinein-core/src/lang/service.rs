//! Bounded incremental syntax-tree cache and snapshot production.

use std::{
    collections::HashMap,
    error::Error,
    fmt,
    path::{Path, PathBuf},
};

use kinein_protocol::{SyntaxHighlight, SyntaxLocal, SyntaxLocalKind, SyntaxTreeSnapshotResult};
use tree_sitter::{InputEdit, Parser, Query, QueryCursor, StreamingIterator, Tree};

use super::{
    folding::folding_ranges,
    outline::outline,
    positions::{LineIndex, byte_point},
    registry::{LanguageId, LanguageRegistry, RegistryError},
};

const MAX_DOCUMENTS: usize = 32;
const MAX_CONTENT_BYTES: usize = 4 * 1024 * 1024;
const MAX_HIGHLIGHTS: usize = 100_000;
const MAX_LOCALS: usize = 50_000;

#[derive(Debug)]
struct ParsedDocument {
    language: LanguageId,
    content: String,
    tree: Tree,
    touched: u64,
}

/// Failure while producing a local syntax snapshot.
#[derive(Debug)]
#[allow(clippy::redundant_pub_crate)]
pub(crate) enum SyntaxTreeError {
    /// The buffer exceeds the bounded interactive parsing budget.
    TooLarge {
        /// Actual UTF-8 byte length.
        bytes: usize,
        /// Accepted byte limit.
        maximum: usize,
    },
    /// A grammar query failed to compile.
    Registry(RegistryError),
    /// The parser rejected a grammar ABI.
    Language(tree_sitter::LanguageError),
    /// Tree-sitter cancelled or failed to produce a tree.
    Parse,
}

impl fmt::Display for SyntaxTreeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooLarge { bytes, maximum } => write!(
                formatter,
                "buffer de {bytes} bytes excede o limite sintatico de {maximum} bytes"
            ),
            Self::Registry(error) => error.fmt(formatter),
            Self::Language(error) => {
                write!(formatter, "gramatica Tree-sitter incompativel: {error}")
            }
            Self::Parse => formatter.write_str("Tree-sitter nao produziu uma arvore sintatica"),
        }
    }
}

impl Error for SyntaxTreeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Registry(error) => Some(error),
            Self::Language(error) => Some(error),
            Self::TooLarge { .. } | Self::Parse => None,
        }
    }
}

impl From<RegistryError> for SyntaxTreeError {
    fn from(value: RegistryError) -> Self {
        Self::Registry(value)
    }
}

/// Lazy, LRU-bounded incremental parser for editor buffers.
#[derive(Debug, Default)]
#[allow(clippy::redundant_pub_crate)]
pub(crate) struct SyntaxTreeService {
    registry: LanguageRegistry,
    documents: HashMap<PathBuf, ParsedDocument>,
    clock: u64,
}

impl SyntaxTreeService {
    /// Produces highlights, folds, outline and locals for one document version.
    pub(crate) fn update(
        &mut self,
        path: &Path,
        content: &str,
        version: u64,
    ) -> Result<SyntaxTreeSnapshotResult, SyntaxTreeError> {
        if content.len() > MAX_CONTENT_BYTES {
            return Err(SyntaxTreeError::TooLarge {
                bytes: content.len(),
                maximum: MAX_CONTENT_BYTES,
            });
        }
        let Some(language_id) = LanguageId::for_path(path) else {
            return Ok(empty_snapshot(path, "plain", version));
        };

        let language = self.registry.runtime(language_id)?.language().clone();
        let previous = self.documents.remove(path);
        let tree = parse_incremental(&language, language_id, content, previous.as_ref())?;

        let runtime = self.registry.runtime(language_id)?;
        let lines = LineIndex::new(content);
        let highlights = highlights(&tree, runtime.highlights(), content, &lines);
        let folding_ranges = folding_ranges(&tree);
        let outline = outline(&tree, runtime.tags(), content, &lines);
        let locals = locals(&tree, runtime.locals(), content, &lines);
        let has_errors = tree.root_node().has_error();

        self.clock = self.clock.saturating_add(1);
        self.documents.insert(
            path.to_path_buf(),
            ParsedDocument {
                language: language_id,
                content: content.to_owned(),
                tree,
                touched: self.clock,
            },
        );
        self.evict_if_needed();

        Ok(SyntaxTreeSnapshotResult {
            path: path.display().to_string(),
            language: language_id.as_str().to_owned(),
            version,
            has_errors,
            highlights,
            folding_ranges,
            outline,
            locals,
        })
    }

    /// Drops all trees when the owning workspace changes.
    pub(crate) fn clear(&mut self) {
        self.documents.clear();
    }

    fn evict_if_needed(&mut self) {
        while self.documents.len() > MAX_DOCUMENTS {
            let oldest = self
                .documents
                .iter()
                .min_by_key(|(_, document)| document.touched)
                .map(|(path, _)| path.clone());
            let Some(oldest) = oldest else {
                break;
            };
            self.documents.remove(&oldest);
        }
    }
}

fn parse_incremental(
    language: &tree_sitter::Language,
    language_id: LanguageId,
    content: &str,
    previous: Option<&ParsedDocument>,
) -> Result<Tree, SyntaxTreeError> {
    let mut parser = Parser::new();
    parser
        .set_language(language)
        .map_err(SyntaxTreeError::Language)?;

    let mut edited_tree = previous
        .filter(|document| document.language == language_id)
        .map(|document| document.tree.clone());
    if let (Some(document), Some(tree)) = (previous, edited_tree.as_mut()) {
        if document.content != content {
            tree.edit(&contiguous_edit(&document.content, content));
        }
    }
    parser
        .parse(content.as_bytes(), edited_tree.as_ref())
        .ok_or(SyntaxTreeError::Parse)
}

fn contiguous_edit(old: &str, new: &str) -> InputEdit {
    let old_bytes = old.as_bytes();
    let new_bytes = new.as_bytes();
    let mut prefix = old_bytes
        .iter()
        .zip(new_bytes)
        .take_while(|(left, right)| left == right)
        .count();
    while prefix > 0 && (!old.is_char_boundary(prefix) || !new.is_char_boundary(prefix)) {
        prefix -= 1;
    }

    let maximum_suffix = old.len().min(new.len()).saturating_sub(prefix);
    let mut suffix = old_bytes[old.len().saturating_sub(maximum_suffix)..]
        .iter()
        .rev()
        .zip(
            new_bytes[new.len().saturating_sub(maximum_suffix)..]
                .iter()
                .rev(),
        )
        .take_while(|(left, right)| left == right)
        .count();
    while suffix > 0
        && (!old.is_char_boundary(old.len() - suffix) || !new.is_char_boundary(new.len() - suffix))
    {
        suffix -= 1;
    }

    let old_end = old.len() - suffix;
    let new_end = new.len() - suffix;
    InputEdit {
        start_byte: prefix,
        old_end_byte: old_end,
        new_end_byte: new_end,
        start_position: byte_point(old, prefix),
        old_end_position: byte_point(old, old_end),
        new_end_position: byte_point(new, new_end),
    }
}

fn highlights(tree: &Tree, query: &Query, source: &str, lines: &LineIndex) -> Vec<SyntaxHighlight> {
    let mut result = Vec::new();
    let mut cursor = QueryCursor::new();
    let mut captures = cursor.captures(query, tree.root_node(), source.as_bytes());
    let names = query.capture_names();

    while let Some((query_match, capture_index)) = captures.next() {
        if result.len() >= MAX_HIGHLIGHTS {
            break;
        }
        let Some(capture) = query_match.captures.get(*capture_index) else {
            continue;
        };
        let Some(scope) = names.get(capture.index as usize) else {
            continue;
        };
        push_highlight_segments(&mut result, source, lines, capture.node.byte_range(), scope);
    }
    result
}

fn push_highlight_segments(
    target: &mut Vec<SyntaxHighlight>,
    source: &str,
    lines: &LineIndex,
    range: std::ops::Range<usize>,
    scope: &str,
) {
    let mut start = range.start;
    while start < range.end && target.len() < MAX_HIGHLIGHTS {
        let segment_end = source[start..range.end]
            .find('\n')
            .map_or(range.end, |offset| start + offset);
        let (line, column) = lines.utf16_position(source, start);
        let (_, end_column) = lines.utf16_position(source, segment_end);
        if end_column > column {
            target.push(SyntaxHighlight {
                line,
                start: column,
                length: end_column - column,
                scope: scope.to_owned(),
            });
        }
        start = segment_end.saturating_add(1);
    }
}

fn locals(tree: &Tree, query: &Query, source: &str, lines: &LineIndex) -> Vec<SyntaxLocal> {
    let mut result = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    let mut cursor = QueryCursor::new();
    let mut captures = cursor.captures(query, tree.root_node(), source.as_bytes());
    let names = query.capture_names();
    while let Some((query_match, capture_index)) = captures.next() {
        if result.len() >= MAX_LOCALS {
            break;
        }
        let Some(capture) = query_match.captures.get(*capture_index) else {
            continue;
        };
        let Some(capture_name) = names.get(capture.index as usize) else {
            continue;
        };
        let kind = match *capture_name {
            "local.scope" => SyntaxLocalKind::Scope,
            "local.definition" => SyntaxLocalKind::Definition,
            "local.reference" => SyntaxLocalKind::Reference,
            _ => continue,
        };
        let range = capture.node.byte_range();
        if !seen.insert((capture_name.to_string(), range.start, range.end)) {
            continue;
        }
        let (line, column) = lines.utf16_position(source, range.start);
        let (end_line, end_column) = lines.utf16_position(source, range.end);
        let name = if kind == SyntaxLocalKind::Scope {
            None
        } else {
            source.get(range).map(str::to_owned)
        };
        result.push(SyntaxLocal {
            kind,
            name,
            line,
            column: column.saturating_add(1),
            end_line,
            end_column: end_column.saturating_add(1),
        });
    }
    result
}

fn empty_snapshot(path: &Path, language: &str, version: u64) -> SyntaxTreeSnapshotResult {
    SyntaxTreeSnapshotResult {
        path: path.display().to_string(),
        language: language.to_owned(),
        version,
        has_errors: false,
        highlights: Vec::new(),
        folding_ranges: Vec::new(),
        outline: Vec::new(),
        locals: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{SyntaxTreeService, contiguous_edit};

    #[test]
    fn contiguous_edit_preserves_utf8_boundaries() {
        let edit = contiguous_edit("let café = 1;", "let café = 20;");
        assert_eq!(edit.start_byte, 12);
        assert_eq!(edit.old_end_byte, 13);
        assert_eq!(edit.new_end_byte, 14);
    }

    #[test]
    fn service_reuses_one_document_and_returns_all_structural_layers() {
        let mut service = SyntaxTreeService::default();
        let path = Path::new("/tmp/main.rs");
        let first = service.update(path, "fn main() {\n    let café = 1;\n}\n", 1);
        assert!(first.is_ok());
        let second = service.update(path, "fn main() {\n    let café = 2;\n}\n", 2);
        assert!(second.is_ok());
        let Ok(second) = second else {
            return;
        };
        assert_eq!(service.documents.len(), 1);
        assert_eq!(second.language, "rust");
        assert!(!second.highlights.is_empty());
        assert!(!second.folding_ranges.is_empty());
        assert!(second.outline.iter().any(|item| item.name == "main"));
        assert!(
            second
                .locals
                .iter()
                .any(|item| item.name.as_deref() == Some("café"))
        );
    }

    #[test]
    fn unsupported_files_have_a_typed_empty_snapshot() {
        let result = SyntaxTreeService::default().update(Path::new("/tmp/readme.txt"), "text", 3);
        assert!(result.is_ok());
        let Ok(result) = result else {
            return;
        };
        assert_eq!(result.language, "plain");
        assert!(result.highlights.is_empty());
    }
}

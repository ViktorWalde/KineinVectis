//! Incremental syntax-tree payloads (`syntaxTree.*`).

use serde::{Deserialize, Serialize};

/// Parameters for `syntaxTree.update`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SyntaxTreeUpdateParams {
    /// Absolute path of an existing file inside the workspace root.
    pub path: String,
    /// Current UTF-8 editor buffer.
    pub content: String,
    /// Monotonic editor-side document version.
    pub version: u64,
}

/// One Tree-sitter highlight capture, adapted to Qt UTF-16 positions.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyntaxHighlight {
    /// One-based line number.
    pub line: u64,
    /// Zero-based UTF-16 column.
    pub start: u64,
    /// Capture length in UTF-16 code units.
    pub length: u64,
    /// Original capture scope, such as `function.call` or `type.builtin`.
    pub scope: String,
}

/// One structural multi-line range that can be folded by the editor.
#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyntaxFoldingRange {
    /// One-based line containing the fold header.
    pub start_line: u64,
    /// One-based final line hidden when the fold is active.
    pub end_line: u64,
}

/// One syntactic declaration in the local document outline.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyntaxOutlineItem {
    /// Display name captured by the grammar tags query.
    pub name: String,
    /// Syntactic declaration kind (`function`, `class`, `module`, ...).
    pub kind: String,
    /// One-based selection line.
    pub line: u64,
    /// One-based UTF-16 selection column.
    pub column: u64,
    /// One-based final line of the declaration.
    pub end_line: u64,
    /// Declarations structurally contained by this declaration.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<Self>,
}

/// Role of one capture in the lightweight local syntax index.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SyntaxLocalKind {
    /// Lexical scope boundary.
    Scope,
    /// Syntactic local declaration.
    Definition,
    /// Syntactic identifier use; not a semantically resolved reference.
    Reference,
}

/// One scope/definition/reference capture from the local syntax tree.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyntaxLocal {
    /// Capture role.
    pub kind: SyntaxLocalKind,
    /// Captured source text when the capture names an identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// One-based start line.
    pub line: u64,
    /// One-based UTF-16 start column.
    pub column: u64,
    /// One-based end line.
    pub end_line: u64,
    /// One-based UTF-16 end column.
    pub end_column: u64,
}

/// Combined incremental syntax snapshot returned by `syntaxTree.update`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyntaxTreeSnapshotResult {
    /// Canonical absolute document path.
    pub path: String,
    /// Registry language (`c`, `cpp`, `rust` or `plain`).
    pub language: String,
    /// Echo of the document version that produced this snapshot.
    pub version: u64,
    /// Whether Tree-sitter found at least one error node.
    pub has_errors: bool,
    /// Structural syntax highlight captures.
    pub highlights: Vec<SyntaxHighlight>,
    /// Foldable structural ranges.
    pub folding_ranges: Vec<SyntaxFoldingRange>,
    /// Nested local outline.
    pub outline: Vec<SyntaxOutlineItem>,
    /// Lightweight syntactic locals index.
    pub locals: Vec<SyntaxLocal>,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{SyntaxLocalKind, SyntaxTreeSnapshotResult, SyntaxTreeUpdateParams};

    #[test]
    fn update_params_require_a_version_and_reject_unknown_fields() {
        let valid = serde_json::from_value::<SyntaxTreeUpdateParams>(json!({
            "path": "/tmp/main.rs",
            "content": "fn main() {}\n",
            "version": 7
        }));
        let missing = serde_json::from_value::<SyntaxTreeUpdateParams>(json!({
            "path": "/tmp/main.rs",
            "content": "fn main() {}\n"
        }));
        let unknown = serde_json::from_value::<SyntaxTreeUpdateParams>(json!({
            "path": "/tmp/main.rs",
            "content": "fn main() {}\n",
            "version": 7,
            "language": "rust"
        }));

        assert!(valid.is_ok());
        assert_eq!(valid.unwrap().version, 7);
        assert!(missing.is_err());
        assert!(unknown.is_err());
    }

    #[test]
    fn snapshot_uses_camel_case_and_omits_empty_children() {
        let value = serde_json::to_value(SyntaxTreeSnapshotResult {
            path: "/tmp/main.rs".to_owned(),
            language: "rust".to_owned(),
            version: 1,
            has_errors: false,
            highlights: Vec::new(),
            folding_ranges: Vec::new(),
            outline: Vec::new(),
            locals: vec![super::SyntaxLocal {
                kind: SyntaxLocalKind::Definition,
                name: Some("main".to_owned()),
                line: 1,
                column: 4,
                end_line: 1,
                end_column: 8,
            }],
        });

        assert!(value.is_ok());
        assert_eq!(value.unwrap()["locals"][0]["kind"], json!("definition"));
    }
}

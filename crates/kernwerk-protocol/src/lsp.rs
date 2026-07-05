//! Language-server payloads (`lsp.*`).

use serde::{Deserialize, Serialize};

/// Parameters for LSP requests tied to the current editor cursor.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LspTextDocumentPositionParams {
    /// Absolute path of an existing file inside the workspace root.
    pub path: String,
    /// Current UTF-8 editor buffer. The core syncs it before sending the LSP request.
    pub content: String,
    /// One-based line number at the editor cursor.
    pub line: u64,
    /// One-based column number at the editor cursor.
    pub column: u64,
}

/// Result payload for `lsp.definition`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LspDefinitionResult {
    /// Target file, when the server found a definition.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// One-based target line number.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<u64>,
    /// One-based target column number.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub column: Option<u64>,
}

/// Result payload for `lsp.hover`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LspHoverResult {
    /// Human-readable hover content, already flattened from LSP markup.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

/// One completion entry returned by `lsp.completion`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LspCompletionItem {
    /// Text shown in the completion popup.
    pub label: String,
    /// Text inserted when the item is accepted.
    pub insert_text: String,
    /// Extra detail (type signature, module), when the server provides it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    /// Item kind, flattened from LSP numbers (`function`, `variable`, ...).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
}

/// Result payload for `lsp.completion`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LspCompletionResult {
    /// Completion entries, already sorted and capped by the core.
    pub items: Vec<LspCompletionItem>,
}

/// Parameters for `lsp.rename`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LspRenameParams {
    /// Absolute path of an existing file inside the workspace root.
    pub path: String,
    /// Current UTF-8 editor buffer. The core syncs it before sending the LSP request.
    pub content: String,
    /// One-based line number at the editor cursor.
    pub line: u64,
    /// One-based column number at the editor cursor.
    pub column: u64,
    /// New symbol name.
    pub new_name: String,
}

/// Result payload for `lsp.rename`, after the core applied every edit.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LspRenameResult {
    /// Canonical absolute paths of the files rewritten by the rename.
    pub files: Vec<String>,
    /// Total number of text edits applied.
    pub edits: u64,
}

/// One code location returned by `lsp.references`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LspReferenceItem {
    /// Canonical absolute path of the file.
    pub path: String,
    /// One-based line number.
    pub line: u64,
    /// One-based column number.
    pub column: u64,
}

/// Result payload for `lsp.references`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LspReferencesResult {
    /// Every usage found by the server, capped by the core.
    pub references: Vec<LspReferenceItem>,
}

/// One semantic token resolved by `lsp.semanticTokens`.
///
/// Positions are tailored for the Qt text renderer: `line` is 1-based and
/// `start`/`length` are 0-based UTF-16 code units within the line, matching
/// `QString` indices, so the highlighter applies them without conversion.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LspSemanticToken {
    /// One-based line number.
    pub line: u64,
    /// Zero-based UTF-16 start column within the line.
    pub start: u64,
    /// Token length in UTF-16 code units.
    pub length: u64,
    /// Token kind name from the server legend (e.g. `variable`, `function`).
    pub kind: String,
}

/// Result payload for `lsp.semanticTokens`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LspSemanticTokensResult {
    /// Tokens in document order.
    pub tokens: Vec<LspSemanticToken>,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::{
        LspCompletionItem, LspCompletionResult, LspDefinitionResult, LspRenameParams,
        LspRenameResult, LspTextDocumentPositionParams,
    };

    #[test]
    fn lsp_position_params_reject_unknown_fields() {
        let valid = serde_json::from_value::<LspTextDocumentPositionParams>(json!({
            "path": "/tmp/main.rs",
            "content": "fn main() {}\n",
            "line": 1,
            "column": 4,
        }));
        let invalid = serde_json::from_value::<LspTextDocumentPositionParams>(json!({
            "path": "/tmp/main.rs",
            "content": "fn main() {}\n",
            "line": 1,
            "column": 4,
            "offset": 3,
        }));

        assert_eq!(valid.unwrap().column, 4);
        assert!(invalid.is_err());
    }

    #[test]
    fn lsp_rename_params_reject_unknown_fields_and_use_camel_case() {
        let valid = serde_json::from_value::<LspRenameParams>(json!({
            "path": "/tmp/main.rs",
            "content": "fn main() {}\n",
            "line": 1,
            "column": 4,
            "newName": "start",
        }));
        let invalid = serde_json::from_value::<LspRenameParams>(json!({
            "path": "/tmp/main.rs",
            "content": "fn main() {}\n",
            "line": 1,
            "column": 4,
            "newName": "start",
            "force": true,
        }));

        assert_eq!(valid.unwrap().new_name, "start");
        assert!(invalid.is_err());
    }

    #[test]
    fn lsp_completion_item_omits_empty_optional_fields() {
        let result = LspCompletionResult {
            items: vec![LspCompletionItem {
                label: "main()".to_owned(),
                insert_text: "main".to_owned(),
                detail: None,
                kind: Some("function".to_owned()),
            }],
        };
        let value = serde_json::to_value(result).unwrap();

        assert_eq!(value["items"][0]["label"], "main()");
        assert_eq!(value["items"][0]["insertText"], "main");
        assert_eq!(value["items"][0]["kind"], "function");
        assert!(value["items"][0].get("detail").is_none());
    }

    #[test]
    fn lsp_rename_result_serializes_camel_case() {
        let result = LspRenameResult {
            files: vec!["/tmp/demo/src/main.rs".to_owned()],
            edits: 3,
        };
        let value = serde_json::to_value(result).unwrap();

        assert_eq!(value["files"][0], "/tmp/demo/src/main.rs");
        assert_eq!(value["edits"], 3);
    }

    #[test]
    fn lsp_definition_result_omits_missing_target() {
        let result = LspDefinitionResult {
            path: None,
            line: None,
            column: None,
        };
        let value = serde_json::to_value(result).unwrap();

        assert!(value.as_object().unwrap().is_empty());
    }
}

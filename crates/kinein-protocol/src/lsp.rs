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
    /// `true` when the server marked the list incomplete (the UI must
    /// re-request as the user types more, instead of filtering the cache).
    #[serde(default)]
    pub is_incomplete: bool,
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

/// One file in a pending workspace-edit preview.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LspWorkspaceEditFilePreview {
    /// Canonical absolute path of the file that would be rewritten.
    pub path: String,
    /// Current editor/disk content used as the edit base.
    pub before_content: String,
    /// Resulting content if the transaction is confirmed.
    pub after_content: String,
    /// Number of text edits targeting this file.
    pub edits: u64,
}

/// Pending workspace edit returned by `lsp.rename` and
/// `lsp.applyCodeAction`. No file has been changed yet.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LspWorkspaceEditPreviewResult {
    /// Opaque transaction identifier accepted by apply/cancel.
    pub transaction_id: String,
    /// Human-readable operation title.
    pub title: String,
    /// Per-file before/after preview.
    pub files: Vec<LspWorkspaceEditFilePreview>,
    /// Total number of edits in every file.
    pub edits: u64,
}

/// Parameters for `lsp.workspaceEdit.apply` and
/// `lsp.workspaceEdit.cancel`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LspWorkspaceEditTransactionParams {
    /// Opaque identifier returned by the preview-producing request.
    pub transaction_id: String,
}

/// Result returned after an atomic workspace-edit transaction succeeds.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LspWorkspaceEditApplyResult {
    /// Identifier of the applied transaction.
    pub transaction_id: String,
    /// Human-readable operation title.
    pub title: String,
    /// Canonical absolute paths rewritten by the transaction.
    pub files: Vec<String>,
    /// Total number of applied text edits.
    pub edits: u64,
}

/// Result returned after discarding a pending workspace edit.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LspWorkspaceEditCancelResult {
    /// Identifier of the cancelled transaction.
    pub transaction_id: String,
    /// Always `true` when the pending transaction existed and was removed.
    pub cancelled: bool,
}

/// One action offered by `lsp.codeActions`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LspCodeActionInfo {
    /// Human-readable action title, as sent by the server.
    pub title: String,
    /// LSP code-action kind (`quickfix`, `refactor.rewrite`, ...), when the
    /// server provides one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
}

/// Result payload for `lsp.codeActions`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LspCodeActionsResult {
    /// Applicable actions, in server order. Only actions carrying an inline
    /// `edit` are listed; the core keeps them cached for `lsp.applyCodeAction`.
    pub actions: Vec<LspCodeActionInfo>,
}

/// Parameters for `lsp.applyCodeAction`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LspApplyCodeActionParams {
    /// Absolute path of the file the actions were requested for.
    pub path: String,
    /// Current UTF-8 editor buffer, used as the base for edits on this file.
    pub content: String,
    /// Zero-based index into the last `lsp.codeActions` response for `path`.
    pub action_index: u64,
}

/// Parameters for `lsp.workspaceSymbols`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LspWorkspaceSymbolsParams {
    /// Absolute path of the active file; picks the language server queried.
    pub path: String,
    /// Current UTF-8 editor buffer, synced before the request.
    pub content: String,
    /// Non-empty symbol name query, matched server-side.
    pub query: String,
}

/// One symbol returned by `lsp.documentSymbols` or `lsp.workspaceSymbols`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LspSymbolInfo {
    /// Symbol name as reported by the server.
    pub name: String,
    /// Flattened LSP symbol kind (`function`, `struct`, `enumMember`, ...).
    pub kind: String,
    /// Canonical absolute path of the file that declares the symbol.
    pub path: String,
    /// One-based line of the symbol's selection position.
    pub line: u64,
    /// One-based column of the symbol's selection position.
    pub column: u64,
    /// Enclosing container (parent symbol or module), when reported.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub container: Option<String>,
}

/// Result payload for `lsp.documentSymbols` and `lsp.workspaceSymbols`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LspSymbolsResult {
    /// Symbols in document order (file) or server order (workspace), capped.
    pub symbols: Vec<LspSymbolInfo>,
    /// The file the request was anchored to.
    ///
    /// Carried since `0.135.0` because the answer has to say what it is about.
    /// Two symbol requests differ only by method and argument, and a client
    /// that has to remember what it asked paints a late `documentSymbol`
    /// answer into a `workspace/symbol` list.
    pub path: String,
    /// The query, for `workspace/symbol` only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
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

/// Parameters for `lsp.semanticTokens`.
///
/// `version` is owned by the editor UI and echoed unchanged by the core. It
/// prevents a slow language-server response from being applied to a newer
/// buffer or to another tab.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LspSemanticTokensParams {
    /// Absolute path of an existing file inside the workspace root.
    pub path: String,
    /// Current UTF-8 editor buffer synchronized before the LSP request.
    pub content: String,
    /// Monotonic editor-side version for stale-response rejection.
    pub version: u64,
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
    /// Canonical path whose buffer produced these tokens.
    pub path: String,
    /// Editor-side version received in the request.
    pub version: u64,
    /// Tokens in document order.
    pub tokens: Vec<LspSemanticToken>,
}

/// Result payload for `lsp.switchSourceHeader` (clangd extension).
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LspSwitchSourceHeaderResult {
    /// Counterpart file (header ↔ source); absent when clangd found none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

/// Parameters for `lsp.restart` (M4.3b): restart one language server or all.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LspRestartParams {
    /// Language to restart (e.g. `rust`, `cpp`). Absent = restart every
    /// running server.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
}

/// Result of `lsp.restart`: which servers were killed for restart.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LspRestartResult {
    /// Languages whose server was stopped (respawns lazily on next request).
    pub restarted: Vec<String>,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::{
        LspApplyCodeActionParams, LspCodeActionInfo, LspCodeActionsResult, LspCompletionItem,
        LspCompletionResult, LspDefinitionResult, LspRenameParams, LspSemanticToken,
        LspSemanticTokensParams, LspSemanticTokensResult, LspTextDocumentPositionParams,
        LspWorkspaceEditApplyResult, LspWorkspaceEditCancelResult, LspWorkspaceEditFilePreview,
        LspWorkspaceEditPreviewResult, LspWorkspaceEditTransactionParams,
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
            is_incomplete: true,
        };
        let value = serde_json::to_value(result).unwrap();

        assert_eq!(value["items"][0]["label"], "main()");
        assert_eq!(value["items"][0]["insertText"], "main");
        assert_eq!(value["items"][0]["kind"], "function");
        assert!(value["items"][0].get("detail").is_none());
        assert_eq!(value["isIncomplete"], true);
    }

    #[test]
    fn lsp_semantic_tokens_echo_path_and_version() {
        let params: LspSemanticTokensParams = serde_json::from_value(json!({
            "path": "/tmp/main.rs",
            "content": "fn main() {}\n",
            "version": 7,
        }))
        .unwrap();
        assert_eq!(params.version, 7);

        let result = LspSemanticTokensResult {
            path: params.path,
            version: params.version,
            tokens: vec![LspSemanticToken {
                line: 1,
                start: 3,
                length: 4,
                kind: "function".to_owned(),
            }],
        };
        let value = serde_json::to_value(result).unwrap();

        assert_eq!(value["path"], "/tmp/main.rs");
        assert_eq!(value["version"], 7);
        assert_eq!(value["tokens"][0]["kind"], "function");

        let rejected = serde_json::from_value::<LspSemanticTokensParams>(json!({
            "path": "/tmp/main.rs",
            "content": "",
            "version": 8,
            "extra": true,
        }));
        assert!(rejected.is_err());
    }

    #[test]
    fn lsp_workspace_edit_preview_serializes_camel_case() {
        let result = LspWorkspaceEditPreviewResult {
            transaction_id: "workspace-edit-7".to_owned(),
            title: "Rename to start".to_owned(),
            files: vec![LspWorkspaceEditFilePreview {
                path: "/tmp/demo/src/main.rs".to_owned(),
                before_content: "fn main() {}\n".to_owned(),
                after_content: "fn start() {}\n".to_owned(),
                edits: 1,
            }],
            edits: 3,
        };
        let value = serde_json::to_value(result).unwrap();

        assert_eq!(value["transactionId"], "workspace-edit-7");
        assert_eq!(value["files"][0]["path"], "/tmp/demo/src/main.rs");
        assert_eq!(value["files"][0]["afterContent"], "fn start() {}\n");
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

    #[test]
    fn lsp_code_actions_result_serializes_camel_case_and_omits_empty_kind() {
        let result = LspCodeActionsResult {
            actions: vec![
                LspCodeActionInfo {
                    title: "Fill match arms".to_owned(),
                    kind: Some("quickfix".to_owned()),
                },
                LspCodeActionInfo {
                    title: "Inline variable".to_owned(),
                    kind: None,
                },
            ],
        };
        let value = serde_json::to_value(result).unwrap();

        assert_eq!(value["actions"][0]["title"], "Fill match arms");
        assert_eq!(value["actions"][0]["kind"], "quickfix");
        assert_eq!(value["actions"][1]["title"], "Inline variable");
        assert!(value["actions"][1].get("kind").is_none());
    }

    #[test]
    fn lsp_apply_code_action_params_reject_unknown_fields_and_use_camel_case() {
        let parsed: LspApplyCodeActionParams = serde_json::from_value(serde_json::json!({
            "path": "/w/src/main.rs",
            "content": "fn main() {}",
            "actionIndex": 2,
        }))
        .unwrap();
        assert_eq!(parsed.action_index, 2);

        let rejected = serde_json::from_value::<LspApplyCodeActionParams>(serde_json::json!({
            "path": "/w/src/main.rs",
            "content": "",
            "actionIndex": 0,
            "extra": true,
        }));
        assert!(rejected.is_err());
    }

    #[test]
    fn lsp_symbol_info_serializes_camel_case_and_omits_empty_container() {
        let value = serde_json::to_value(crate::LspSymbolsResult {
            symbols: vec![
                crate::LspSymbolInfo {
                    name: "tamanho".to_owned(),
                    kind: "method".to_owned(),
                    path: "/w/src/main.rs".to_owned(),
                    line: 16,
                    column: 8,
                    container: Some("Ponto".to_owned()),
                },
                crate::LspSymbolInfo {
                    name: "main".to_owned(),
                    kind: "function".to_owned(),
                    path: "/w/src/main.rs".to_owned(),
                    line: 21,
                    column: 4,
                    container: None,
                },
            ],
            path: "/w/src/main.rs".to_owned(),
            query: None,
        })
        .unwrap();

        assert_eq!(value["symbols"][0]["container"], "Ponto");
        assert!(value["symbols"][1].get("container").is_none());
        assert_eq!(value["symbols"][1]["kind"], "function");
        // A RESPOSTA DIZ SOBRE O QUE ELA E' (0.135.0). Sem isto, duas
        // perguntas de simbolo diferem so' pelo metodo, e uma resposta atrasada
        // de `documentSymbol` pinta a lista de `workspace/symbol`.
        assert_eq!(value["path"], "/w/src/main.rs");
        assert!(value.get("query").is_none(), "documentSymbol nao tem query");
    }

    #[test]
    fn lsp_workspace_symbols_params_reject_unknown_fields() {
        let parsed: crate::LspWorkspaceSymbolsParams = serde_json::from_value(json!({
            "path": "/w/src/main.rs",
            "content": "fn main() {}",
            "query": "Ponto",
        }))
        .unwrap();
        assert_eq!(parsed.query, "Ponto");

        let rejected = serde_json::from_value::<crate::LspWorkspaceSymbolsParams>(json!({
            "path": "/w/src/main.rs",
            "content": "",
            "query": "x",
            "extra": 1,
        }));
        assert!(rejected.is_err());
    }

    #[test]
    fn lsp_workspace_edit_apply_and_cancel_payloads_use_camel_case() {
        let result = LspWorkspaceEditApplyResult {
            transaction_id: "workspace-edit-9".to_owned(),
            title: "Fill match arms".to_owned(),
            files: vec!["/w/src/main.rs".to_owned()],
            edits: 1,
        };
        let value = serde_json::to_value(result).unwrap();

        assert_eq!(value["title"], "Fill match arms");
        assert_eq!(value["transactionId"], "workspace-edit-9");
        assert_eq!(value["files"][0], "/w/src/main.rs");
        assert_eq!(value["edits"], 1);

        let params: LspWorkspaceEditTransactionParams = serde_json::from_value(json!({
            "transactionId": "workspace-edit-9"
        }))
        .unwrap();
        assert_eq!(params.transaction_id, "workspace-edit-9");

        let cancelled = serde_json::to_value(LspWorkspaceEditCancelResult {
            transaction_id: params.transaction_id,
            cancelled: true,
        })
        .unwrap();
        assert_eq!(cancelled["transactionId"], "workspace-edit-9");
        assert_eq!(cancelled["cancelled"], true);
    }
}

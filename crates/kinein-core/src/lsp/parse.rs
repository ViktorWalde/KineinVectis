//! Conversao das respostas cruas do LSP nos tipos do protocolo Kinein Vectis.
//!
//! Cada funcao recebe o `result`/`params` JSON de um metodo LSP e devolve o
//! valor de dominio correspondente (localizacoes, hover, completion, semantic
//! tokens, plano de rename ou o evento de diagnosticos). Nenhuma delas fala com
//! o servidor; sao puras sobre `serde_json::Value`.

use kinein_protocol::{
    Diagnostic, DiagnosticSeverity, DiagnosticSource, JsonRpcRequest, LspCodeActionInfo,
    LspCompletionItem, LspSemanticToken,
};
use serde_json::{Value, json};

use super::types::{FileEdits, LspError, LspLocation, TextSpanEdit, WorkspaceEditPlan};
use super::uri::path_for_uri;

/// Maximo de itens de completion repassados a UI por request.
const MAX_COMPLETION_ITEMS: usize = 100;

/// Maximo de referencias (find usages) repassadas a UI por request.
const MAX_REFERENCE_ITEMS: usize = 200;

/// Maximo de simbolos de arquivo (`documentSymbol`) por request.
pub(super) const MAX_DOCUMENT_SYMBOLS: usize = 500;

/// Maximo de simbolos de workspace (`workspace/symbol`) por request.
pub(super) const MAX_WORKSPACE_SYMBOLS: usize = 100;

/// Extrai `result`/`error` de uma resposta LSP em `Result<Value, LspError>`.
pub(super) fn response_result(method: &'static str, response: &Value) -> Result<Value, LspError> {
    if let Some(error) = response.get("error") {
        let message = error
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("erro LSP sem mensagem")
            .to_owned();
        return Err(LspError::RequestFailed { method, message });
    }
    Ok(response.get("result").cloned().unwrap_or(Value::Null))
}

/// Converte `textDocument/publishDiagnostics` no evento Kinein Vectis.
pub(super) fn diagnostics_event(params: &Value) -> Option<JsonRpcRequest> {
    let uri = params.get("uri")?.as_str()?;
    let path = path_for_uri(uri)?;
    let empty = Vec::new();
    let raw_diagnostics = params
        .get("diagnostics")
        .and_then(Value::as_array)
        .unwrap_or(&empty);

    let diagnostics = raw_diagnostics
        .iter()
        .filter_map(|diagnostic| {
            let message = diagnostic.get("message")?.as_str()?.to_owned();
            let severity = match diagnostic.get("severity").and_then(Value::as_i64) {
                Some(1) | None => DiagnosticSeverity::Error,
                Some(2) => DiagnosticSeverity::Warning,
                _ => DiagnosticSeverity::Note,
            };
            let range = diagnostic.get("range")?;
            let start = range.get("start")?;
            let line = start.get("line").and_then(Value::as_u64).unwrap_or(0) + 1;
            let column = start.get("character").and_then(Value::as_u64).unwrap_or(0) + 1;
            // Fim do range (para o sublinhado no editor); ausente ou
            // invertido cai de volta ao inicio (marca ao menos 1 char).
            let end = range.get("end");
            let end_line = end
                .and_then(|end| end.get("line"))
                .and_then(Value::as_u64)
                .map_or(line, |value| value + 1);
            let end_column = end
                .and_then(|end| end.get("character"))
                .and_then(Value::as_u64)
                .map_or(column, |value| value + 1);
            Some(Diagnostic {
                id: None,
                source: DiagnosticSource::Lsp,
                severity,
                category: Some("lsp".to_owned()),
                message,
                file: None,
                line: Some(line),
                column: Some(column),
                end_line: Some(end_line),
                end_column: Some(end_column),
                code: diagnostic_code(diagnostic.get("code")),
                job_id: None,
                command: None,
                target: None,
                log_ref: None,
            })
        })
        .collect::<Vec<_>>();

    Some(JsonRpcRequest::notification(
        "event.lsp.diagnostics",
        Some(json!({ "path": path, "diagnostics": diagnostics })),
    ))
}

/// Normaliza o `code` de um diagnostico LSP (string OU numero) em string.
fn diagnostic_code(code: Option<&Value>) -> Option<String> {
    match code? {
        Value::String(text) if !text.is_empty() => Some(text.clone()),
        Value::Number(number) => Some(number.to_string()),
        _ => None,
    }
}

/// Resolve `textDocument/definition` (`Location`, `Location[]` ou `LocationLink[]`).
pub(super) fn definition_location(result: &Value) -> Option<LspLocation> {
    if result.is_null() {
        return None;
    }
    if let Some(locations) = result.as_array() {
        return locations.iter().find_map(location_from_value);
    }
    location_from_value(result)
}

pub(super) fn location_from_value(value: &Value) -> Option<LspLocation> {
    let (uri, range) = if let Some(uri) = value.get("uri").and_then(Value::as_str) {
        (uri, value.get("range")?)
    } else {
        (
            value.get("targetUri").and_then(Value::as_str)?,
            value
                .get("targetSelectionRange")
                .or_else(|| value.get("targetRange"))?,
        )
    };
    let path = path_for_uri(uri)?;
    let start = range.get("start")?;
    let line = start.get("line").and_then(Value::as_u64).unwrap_or(0) + 1;
    let column = start.get("character").and_then(Value::as_u64).unwrap_or(0) + 1;
    Some(LspLocation { path, line, column })
}

/// Achata o `contents` de `textDocument/hover` em texto simples.
pub(super) fn hover_content(result: &Value) -> Option<String> {
    if result.is_null() {
        return None;
    }
    let contents = result.get("contents")?;
    flatten_markup(contents).filter(|text| !text.trim().is_empty())
}

/// Nome plano do `CompletionItemKind` numerico do LSP.
const fn completion_kind_name(kind: i64) -> Option<&'static str> {
    Some(match kind {
        1 => "text",
        2 => "method",
        3 => "function",
        4 => "constructor",
        5 => "field",
        6 => "variable",
        7 => "class",
        8 => "interface",
        9 => "module",
        10 => "property",
        11 => "unit",
        12 => "value",
        13 => "enum",
        14 => "keyword",
        15 => "snippet",
        16 => "color",
        17 => "file",
        18 => "reference",
        19 => "folder",
        20 => "enumMember",
        21 => "constant",
        22 => "struct",
        23 => "event",
        24 => "operator",
        25 => "typeParameter",
        _ => return None,
    })
}

/// Achata `CompletionItem[] | CompletionList | null` nos itens do protocolo.
pub(super) fn completion_items(result: &Value) -> (Vec<LspCompletionItem>, bool) {
    let empty = Vec::new();
    let raw = result
        .as_array()
        .or_else(|| result.get("items").and_then(Value::as_array))
        .unwrap_or(&empty);
    // O servidor marca `isIncomplete` quando a lista foi truncada por
    // ele (ex.: std::c no clangd): a UI precisa REPEDIR ao digitar mais,
    // nao filtrar o cache. Se nos truncamos, tambem vira incompleto.
    let server_incomplete = result
        .get("isIncomplete")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    let mut entries = raw
        .iter()
        .filter_map(|item| {
            let label = item.get("label")?.as_str()?.trim().to_owned();
            if label.is_empty() {
                return None;
            }
            let insert_text = item
                .get("insertText")
                .and_then(Value::as_str)
                .or_else(|| {
                    item.get("textEdit")
                        .and_then(|edit| edit.get("newText"))
                        .and_then(Value::as_str)
                })
                .map_or_else(|| label.clone(), str::to_owned);
            let sort_key = item
                .get("sortText")
                .and_then(Value::as_str)
                .map_or_else(|| label.clone(), str::to_owned);
            let detail = item
                .get("detail")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|text| !text.is_empty())
                .map(str::to_owned);
            let kind = item
                .get("kind")
                .and_then(Value::as_i64)
                .and_then(completion_kind_name)
                .map(str::to_owned);
            Some((
                sort_key,
                LspCompletionItem {
                    label,
                    insert_text,
                    detail,
                    kind,
                },
            ))
        })
        .collect::<Vec<_>>();

    entries.sort_by(|left, right| {
        left.0
            .cmp(&right.0)
            .then_with(|| left.1.label.cmp(&right.1.label))
    });
    let truncated = entries.len() > MAX_COMPLETION_ITEMS;
    entries.truncate(MAX_COMPLETION_ITEMS);
    let items = entries.into_iter().map(|(_sort, item)| item).collect();
    (items, server_incomplete || truncated)
}

/// Decodifica o array `data` de semantic tokens (grupos de 5 inteiros).
///
/// Cada grupo e `[deltaLine, deltaStart, length, tokenType, modifiers]`;
/// `deltaStart` e relativo ao token anterior apenas quando `deltaLine == 0`.
/// Posicoes ficam em UTF-16 (como o LSP entrega), que coincide com indices
/// de `QString` no highlighter da UI.
pub(super) fn decode_semantic_tokens(result: &Value, legend: &[String]) -> Vec<LspSemanticToken> {
    let Some(data) = result.get("data").and_then(Value::as_array) else {
        return Vec::new();
    };
    let mut tokens = Vec::new();
    let mut line = 0_u64;
    let mut start = 0_u64;
    for group in data.chunks_exact(5) {
        let (Some(delta_line), Some(delta_start), Some(length), Some(kind_index)) = (
            group[0].as_u64(),
            group[1].as_u64(),
            group[2].as_u64(),
            group[3].as_u64(),
        ) else {
            continue;
        };
        line += delta_line;
        start = if delta_line == 0 {
            start + delta_start
        } else {
            delta_start
        };
        let Some(kind) = legend.get(usize::try_from(kind_index).unwrap_or(usize::MAX)) else {
            continue;
        };
        tokens.push(LspSemanticToken {
            line: line + 1,
            start,
            length,
            kind: kind.clone(),
        });
    }
    tokens
}

/// Converte `Location[]` de references nas localizacoes do Kinein Vectis.
pub(super) fn reference_locations(result: &Value) -> Vec<LspLocation> {
    result.as_array().map_or_else(Vec::new, |locations| {
        locations
            .iter()
            .filter_map(location_from_value)
            .take(MAX_REFERENCE_ITEMS)
            .collect()
    })
}

/// Converte um `WorkspaceEdit` de rename no plano de edits por arquivo.
///
/// Operacoes de recurso (criar/renomear/apagar arquivo) ainda nao sao
/// suportadas e geram erro estruturado, sem tocar em nada.
pub(super) fn workspace_edit_plan(result: &Value) -> Result<WorkspaceEditPlan, LspError> {
    let mut files = Vec::new();
    if result.is_null() {
        return Ok(WorkspaceEditPlan { files });
    }

    if let Some(changes) = result.get("changes").and_then(Value::as_object) {
        for (uri, edits) in changes {
            files.push(file_edits_from_value(uri, None, edits)?);
        }
    } else if let Some(changes) = result.get("documentChanges").and_then(Value::as_array) {
        for change in changes {
            if change.get("kind").and_then(Value::as_str).is_some() {
                return Err(LspError::RequestFailed {
                    method: "textDocument/rename",
                    message: "rename que cria/renomeia/apaga arquivos ainda nao e suportado"
                        .to_owned(),
                });
            }
            let uri = change
                .get("textDocument")
                .and_then(|document| document.get("uri"))
                .and_then(Value::as_str)
                .ok_or_else(|| LspError::Transport {
                    message: "documentChanges de rename sem textDocument.uri".to_owned(),
                })?;
            let version = change
                .get("textDocument")
                .and_then(|document| document.get("version"))
                .and_then(Value::as_i64);
            let edits = change.get("edits").unwrap_or(&Value::Null);
            files.push(file_edits_from_value(uri, version, edits)?);
        }
    }

    files.retain(|file| !file.edits.is_empty());
    Ok(WorkspaceEditPlan { files })
}

/// Nome plano do `SymbolKind` numerico do LSP (1..=26).
pub(super) const fn symbol_kind_name(kind: i64) -> Option<&'static str> {
    Some(match kind {
        1 => "file",
        2 => "module",
        3 => "namespace",
        4 => "package",
        5 => "class",
        6 => "method",
        7 => "property",
        8 => "field",
        9 => "constructor",
        10 => "enum",
        11 => "interface",
        12 => "function",
        13 => "variable",
        14 => "constant",
        15 => "string",
        16 => "number",
        17 => "boolean",
        18 => "array",
        19 => "object",
        20 => "key",
        21 => "null",
        22 => "enumMember",
        23 => "struct",
        24 => "event",
        25 => "operator",
        26 => "typeParameter",
        _ => return None,
    })
}

/// Converte a resposta de `textDocument/codeAction` em pares (info, ação crua).
///
/// So ações aplicáveis localmente entram: `CodeAction` literal com `edit`
/// inline e sem `disabled`. Comandos puros e ações que dependem de
/// `workspace/executeCommand` são filtrados (decisão registrada em
/// docs-privada/diario/18, fatia M1.3). A ordem do servidor é preservada.
pub(super) fn code_action_infos(result: &Value) -> (Vec<LspCodeActionInfo>, Vec<Value>) {
    let Some(items) = result.as_array() else {
        return (Vec::new(), Vec::new());
    };
    let mut infos = Vec::new();
    let mut raw = Vec::new();
    for item in items {
        if item.get("disabled").is_some() {
            continue;
        }
        if !item.get("edit").is_some_and(Value::is_object) {
            continue;
        }
        let Some(title) = item.get("title").and_then(Value::as_str) else {
            continue;
        };
        infos.push(LspCodeActionInfo {
            title: title.to_owned(),
            kind: item.get("kind").and_then(Value::as_str).map(str::to_owned),
        });
        raw.push(item.clone());
    }
    (infos, raw)
}

fn file_edits_from_value(
    uri: &str,
    version: Option<i64>,
    edits: &Value,
) -> Result<FileEdits, LspError> {
    let path = path_for_uri(uri).ok_or_else(|| LspError::Transport {
        message: format!("uri de rename invalida: {uri}"),
    })?;
    let raw = edits.as_array().ok_or_else(|| LspError::Transport {
        message: format!("edits de rename invalidos para {path}"),
    })?;

    let mut spans = Vec::with_capacity(raw.len());
    for edit in raw {
        let new_text = edit
            .get("newText")
            .and_then(Value::as_str)
            .ok_or_else(|| LspError::Transport {
                message: format!("edit de rename sem newText em {path}"),
            })?
            .to_owned();
        let range = edit.get("range").ok_or_else(|| LspError::Transport {
            message: format!("edit de rename sem range em {path}"),
        })?;
        let start = range.get("start");
        let end = range.get("end");
        spans.push(TextSpanEdit {
            start_line: position_component(start, "line"),
            start_character: position_component(start, "character"),
            end_line: position_component(end, "line"),
            end_character: position_component(end, "character"),
            new_text,
        });
    }

    Ok(FileEdits {
        path,
        version,
        edits: spans,
    })
}

pub(super) fn position_component(position: Option<&Value>, key: &str) -> u64 {
    position
        .and_then(|value| value.get(key))
        .and_then(Value::as_u64)
        .unwrap_or(0)
}

fn flatten_markup(value: &Value) -> Option<String> {
    if let Some(text) = value.as_str() {
        return Some(text.to_owned());
    }
    if let Some(text) = value.get("value").and_then(Value::as_str) {
        return Some(text.to_owned());
    }
    if let Some(items) = value.as_array() {
        let parts = items.iter().filter_map(flatten_markup).collect::<Vec<_>>();
        if parts.is_empty() {
            return None;
        }
        return Some(parts.join("\n\n"));
    }
    None
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{
        MAX_COMPLETION_ITEMS, completion_items, decode_semantic_tokens, definition_location,
        diagnostics_event, hover_content, reference_locations, workspace_edit_plan,
    };

    #[test]
    fn decode_semantic_tokens_applies_line_and_start_deltas() {
        let legend = vec!["variable".to_owned(), "function".to_owned()];
        // Linha 0: token em 4..7 (function); mesmo-linha em 10..13 (variable);
        // linha 2 (delta 2): token em 0..5 (variable).
        let result = json!({
            "data": [
                0, 4, 3, 1, 0,
                0, 6, 3, 0, 0,
                2, 0, 5, 0, 0,
            ]
        });

        let tokens = decode_semantic_tokens(&result, &legend);

        assert_eq!(tokens.len(), 3);
        assert_eq!(
            (tokens[0].line, tokens[0].start, tokens[0].length),
            (1, 4, 3)
        );
        assert_eq!(tokens[0].kind, "function");
        assert_eq!((tokens[1].line, tokens[1].start), (1, 10));
        assert_eq!(tokens[1].kind, "variable");
        assert_eq!((tokens[2].line, tokens[2].start), (3, 0));
    }

    #[test]
    fn decode_semantic_tokens_skips_kinds_outside_the_legend() {
        let legend = vec!["variable".to_owned()];
        let result = json!({ "data": [0, 0, 2, 9, 0, 0, 4, 2, 0, 0] });

        let tokens = decode_semantic_tokens(&result, &legend);

        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].start, 4);
        assert_eq!(tokens[0].kind, "variable");
    }

    #[test]
    fn publish_diagnostics_becomes_kinein_event() {
        let params = json!({
            "uri": "file:///tmp/demo/src/main.rs",
            "diagnostics": [{
                "range": { "start": { "line": 4, "character": 8 }, "end": {} },
                "severity": 1,
                "message": "mismatched types",
            }],
        });

        let event = diagnostics_event(&params).unwrap();

        assert_eq!(event.method, "event.lsp.diagnostics");
        let event_params = event.params.unwrap();
        assert_eq!(event_params["path"], "/tmp/demo/src/main.rs");
        assert_eq!(event_params["diagnostics"][0]["source"], "lsp");
        assert_eq!(event_params["diagnostics"][0]["line"], 5);
        assert_eq!(event_params["diagnostics"][0]["column"], 9);
        assert_eq!(event_params["diagnostics"][0]["severity"], "error");
        // End vazio cai de volta ao inicio (marca ao menos o ponto).
        assert_eq!(event_params["diagnostics"][0]["endLine"], 5);
        assert_eq!(event_params["diagnostics"][0]["endColumn"], 9);
    }

    #[test]
    fn publish_diagnostics_keeps_full_range_and_code() {
        let params = json!({
            "uri": "file:///tmp/demo/src/main.rs",
            "diagnostics": [
                {
                    "range": {
                        "start": { "line": 2, "character": 4 },
                        "end": { "line": 3, "character": 10 },
                    },
                    "severity": 2,
                    "code": "unused_variables",
                    "message": "unused variable: `x`",
                },
                {
                    "range": {
                        "start": { "line": 0, "character": 0 },
                        "end": { "line": 0, "character": 6 },
                    },
                    "severity": 1,
                    "code": 425,
                    "message": "cannot find value",
                },
            ],
        });

        let event = diagnostics_event(&params).unwrap();
        let diagnostics = &event.params.unwrap()["diagnostics"];

        // Range multi-linha preservado (1-based) + code string.
        assert_eq!(diagnostics[0]["line"], 3);
        assert_eq!(diagnostics[0]["column"], 5);
        assert_eq!(diagnostics[0]["endLine"], 4);
        assert_eq!(diagnostics[0]["endColumn"], 11);
        assert_eq!(diagnostics[0]["severity"], "warning");
        assert_eq!(diagnostics[0]["code"], "unused_variables");
        // code numerico do LSP vira string.
        assert_eq!(diagnostics[1]["code"], "425");
    }

    #[test]
    fn definition_location_accepts_location_array() {
        let result = json!([{
            "uri": "file:///tmp/demo/src/lib.rs",
            "range": { "start": { "line": 9, "character": 4 }, "end": {} },
        }]);

        let location = definition_location(&result).unwrap();

        assert_eq!(location.path, "/tmp/demo/src/lib.rs");
        assert_eq!(location.line, 10);
        assert_eq!(location.column, 5);
    }

    #[test]
    fn definition_location_accepts_location_link() {
        let result = json!([{
            "targetUri": "file:///tmp/demo/src/main.cpp",
            "targetSelectionRange": {
                "start": { "line": 2, "character": 11 },
                "end": {},
            },
        }]);

        let location = definition_location(&result).unwrap();

        assert_eq!(location.path, "/tmp/demo/src/main.cpp");
        assert_eq!(location.line, 3);
        assert_eq!(location.column, 12);
    }

    #[test]
    fn hover_content_flattens_markup_content() {
        let result = json!({
            "contents": {
                "kind": "markdown",
                "value": "```rust\nfn main()\n```",
            },
        });

        assert_eq!(hover_content(&result).unwrap(), "```rust\nfn main()\n```");
    }

    #[test]
    fn completion_items_accepts_list_and_array_and_sorts_by_sort_text() {
        let as_list = json!({
            "isIncomplete": false,
            "items": [
                {
                    "label": " zebra()",
                    "kind": 3,
                    "sortText": "b",
                    "detail": "fn zebra()",
                    "insertText": "zebra",
                },
                { "label": "alpha", "kind": 6, "sortText": "a" },
            ],
        });
        let (items, incomplete) = completion_items(&as_list);

        assert_eq!(items.len(), 2);
        assert!(!incomplete);
        assert_eq!(items[0].label, "alpha");
        assert_eq!(items[0].insert_text, "alpha");
        assert_eq!(items[0].kind.as_deref(), Some("variable"));
        assert_eq!(items[1].label, "zebra()");
        assert_eq!(items[1].insert_text, "zebra");
        assert_eq!(items[1].detail.as_deref(), Some("fn zebra()"));
        assert_eq!(items[1].kind.as_deref(), Some("function"));

        let as_array = json!([{ "label": "solo" }]);
        assert_eq!(completion_items(&as_array).0.len(), 1);
        assert!(completion_items(&json!(null)).0.is_empty());
        // isIncomplete do servidor propaga mesmo com poucos itens.
        let flagged = json!({ "isIncomplete": true, "items": [{ "label": "x" }] });
        assert!(completion_items(&flagged).1);
    }

    #[test]
    fn completion_items_uses_text_edit_and_caps_results() {
        let mut raw_items = Vec::new();
        for index in 0..150 {
            raw_items.push(json!({
                "label": format!("item{index:03}"),
                "textEdit": {
                    "range": { "start": { "line": 0, "character": 0 }, "end": {} },
                    "newText": format!("item{index:03}"),
                },
            }));
        }
        let (items, incomplete) = completion_items(&json!(raw_items));

        assert_eq!(items.len(), MAX_COMPLETION_ITEMS);
        // Truncou -> a lista vira incompleta (a UI deve repedir ao digitar).
        assert!(incomplete);
        assert_eq!(items[0].insert_text, "item000");
    }

    #[test]
    fn reference_locations_parses_location_array() {
        let result = json!([
            {
                "uri": "file:///tmp/demo/src/main.rs",
                "range": { "start": { "line": 0, "character": 3 }, "end": {} },
            },
            {
                "uri": "file:///tmp/demo/src/lib.rs",
                "range": { "start": { "line": 7, "character": 0 }, "end": {} },
            },
        ]);
        let references = reference_locations(&result);

        assert_eq!(references.len(), 2);
        assert_eq!(references[0].path, "/tmp/demo/src/main.rs");
        assert_eq!(references[0].line, 1);
        assert_eq!(references[0].column, 4);
        assert!(reference_locations(&json!(null)).is_empty());
    }

    #[test]
    fn workspace_edit_plan_parses_changes_map() {
        let result = json!({
            "changes": {
                "file:///tmp/demo/src/main.rs": [
                    {
                        "range": {
                            "start": { "line": 0, "character": 3 },
                            "end": { "line": 0, "character": 7 },
                        },
                        "newText": "start",
                    },
                ],
            },
        });
        let plan = workspace_edit_plan(&result).unwrap();

        assert_eq!(plan.files.len(), 1);
        assert_eq!(plan.files[0].path, "/tmp/demo/src/main.rs");
        assert_eq!(plan.files[0].edits[0].new_text, "start");
        assert_eq!(plan.files[0].edits[0].end_character, 7);
        assert_eq!(plan.edit_count(), 1);
    }

    #[test]
    fn workspace_edit_plan_parses_document_changes() {
        let result = json!({
            "documentChanges": [
                {
                    "textDocument": { "uri": "file:///tmp/demo/src/lib.rs", "version": 4 },
                    "edits": [
                        {
                            "range": {
                                "start": { "line": 2, "character": 0 },
                                "end": { "line": 2, "character": 3 },
                            },
                            "newText": "novo",
                        },
                    ],
                },
            ],
        });
        let plan = workspace_edit_plan(&result).unwrap();

        assert_eq!(plan.files.len(), 1);
        assert_eq!(plan.files[0].path, "/tmp/demo/src/lib.rs");
        assert_eq!(plan.edit_count(), 1);
    }

    #[test]
    fn workspace_edit_plan_rejects_resource_operations_and_accepts_null() {
        let with_resource_op = json!({
            "documentChanges": [
                { "kind": "rename", "oldUri": "file:///a.rs", "newUri": "file:///b.rs" },
            ],
        });
        assert!(workspace_edit_plan(&with_resource_op).is_err());

        let plan = workspace_edit_plan(&json!(null)).unwrap();
        assert!(plan.files.is_empty());
        assert_eq!(plan.edit_count(), 0);
    }

    #[test]
    fn code_action_infos_keeps_order_and_filters_unusable_actions() {
        let result = json!([
            { "title": "Fix a", "kind": "quickfix", "edit": { "changes": {} } },
            {
                "title": "Disabled",
                "edit": { "changes": {} },
                "disabled": { "reason": "n/a" },
            },
            { "title": "Command only", "command": { "title": "x", "command": "x" } },
            { "title": "Bare command", "command": "x" },
            { "title": "Refactor b", "edit": { "documentChanges": [] } },
        ]);

        let (infos, raw) = super::code_action_infos(&result);

        assert_eq!(infos.len(), 2);
        assert_eq!(infos[0].title, "Fix a");
        assert_eq!(infos[0].kind.as_deref(), Some("quickfix"));
        assert_eq!(infos[1].title, "Refactor b");
        assert_eq!(infos[1].kind, None);
        assert_eq!(raw.len(), 2);
        assert_eq!(raw[0]["title"], "Fix a");
    }

    #[test]
    fn code_action_infos_accepts_null_and_non_arrays() {
        assert!(super::code_action_infos(&json!(null)).0.is_empty());
        assert!(super::code_action_infos(&json!({})).0.is_empty());
    }

    #[test]
    fn document_symbols_flattens_hierarchical_shape_with_containers() {
        let result = json!([
            {
                "name": "Ponto",
                "kind": 23,
                "range": { "start": { "line": 3, "character": 0 },
                           "end": { "line": 8, "character": 1 } },
                "selectionRange": { "start": { "line": 3, "character": 11 },
                                    "end": { "line": 3, "character": 16 } },
                "children": [
                    {
                        "name": "tamanho",
                        "kind": 6,
                        "selectionRange": { "start": { "line": 5, "character": 7 },
                                            "end": { "line": 5, "character": 14 } },
                        "range": { "start": { "line": 5, "character": 4 },
                                   "end": { "line": 7, "character": 5 } },
                    },
                ],
            },
            {
                "name": "main",
                "kind": 12,
                "selectionRange": { "start": { "line": 10, "character": 3 },
                                    "end": { "line": 10, "character": 7 } },
                "range": { "start": { "line": 10, "character": 0 },
                           "end": { "line": 12, "character": 1 } },
            },
        ]);

        let symbols = crate::lsp::parse_symbols::document_symbols(&result, "/w/src/main.rs");

        assert_eq!(symbols.len(), 3);
        assert_eq!(symbols[0].name, "Ponto");
        assert_eq!(symbols[0].kind, "struct");
        assert_eq!(symbols[0].line, 4);
        assert_eq!(symbols[0].column, 12);
        assert_eq!(symbols[0].container, None);
        assert_eq!(symbols[1].name, "tamanho");
        assert_eq!(symbols[1].kind, "method");
        assert_eq!(symbols[1].container.as_deref(), Some("Ponto"));
        assert_eq!(symbols[1].path, "/w/src/main.rs");
        assert_eq!(symbols[2].name, "main");
        assert_eq!(symbols[2].kind, "function");
    }

    #[test]
    fn workspace_symbols_parses_symbol_information_and_skips_bad_uris() {
        let result = json!([
            {
                "name": "Ponto",
                "kind": 23,
                "containerName": "geometria",
                "location": {
                    "uri": "file:///w/src/lib.rs",
                    "range": { "start": { "line": 2, "character": 11 },
                               "end": { "line": 2, "character": 16 } },
                },
            },
            {
                "name": "quebrado",
                "kind": 12,
                "location": { "uri": "untitled:sem-arquivo" },
            },
        ]);

        let symbols = crate::lsp::parse_symbols::workspace_symbols(&result);

        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "Ponto");
        assert_eq!(symbols[0].kind, "struct");
        assert_eq!(symbols[0].path, "/w/src/lib.rs");
        assert_eq!(symbols[0].line, 3);
        assert_eq!(symbols[0].column, 12);
        assert_eq!(symbols[0].container.as_deref(), Some("geometria"));
        assert!(crate::lsp::parse_symbols::workspace_symbols(&json!(null)).is_empty());
    }
}

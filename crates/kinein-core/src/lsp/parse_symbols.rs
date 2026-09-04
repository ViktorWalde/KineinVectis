//! Simbolos do LSP: `documentSymbol` e `workspace/symbol`.
//!
//! Separado do `parse.rs` em 2026-09-03. Os dois metodos devolvem formas
//! DIFERENTES para a mesma ideia — `DocumentSymbol` e' uma ARVORE aninhada,
//! `SymbolInformation` e' uma lista plana com localizacao —, e achatar as duas
//! na mesma estrutura e' o trabalho deste arquivo. E' a unica familia do parse
//! que precisa de recursao, e por isso e' a que se le pior misturada.

use kinein_protocol::LspSymbolInfo;
use serde_json::Value;

use super::parse::{MAX_DOCUMENT_SYMBOLS, MAX_WORKSPACE_SYMBOLS, symbol_kind_name};
use super::uri::path_for_uri;

/// Achata `textDocument/documentSymbol` nos simbolos do protocolo.
///
/// Aceita os dois shapes do LSP: `DocumentSymbol[]` hierarquico (achatado em
/// pre-ordem, com o pai como `container`) e `SymbolInformation[]` plano.
/// `fallback_path` e o proprio arquivo consultado (o shape hierarquico nao
/// carrega URI).
pub(super) fn document_symbols(result: &Value, fallback_path: &str) -> Vec<LspSymbolInfo> {
    let Some(items) = result.as_array() else {
        return Vec::new();
    };
    let mut symbols = Vec::new();
    for item in items {
        if symbols.len() >= MAX_DOCUMENT_SYMBOLS {
            break;
        }
        if item.get("location").is_some() {
            if let Some(info) = symbol_information(item) {
                symbols.push(info);
            }
        } else {
            flatten_document_symbol(item, None, fallback_path, &mut symbols);
        }
    }
    symbols.truncate(MAX_DOCUMENT_SYMBOLS);
    symbols
}

fn flatten_document_symbol(
    item: &Value,
    container: Option<&str>,
    path: &str,
    out: &mut Vec<LspSymbolInfo>,
) {
    if out.len() >= MAX_DOCUMENT_SYMBOLS {
        return;
    }
    let Some(name) = item.get("name").and_then(Value::as_str) else {
        return;
    };
    let kind = item
        .get("kind")
        .and_then(Value::as_i64)
        .and_then(symbol_kind_name)
        .unwrap_or("symbol")
        .to_owned();
    let position = item
        .pointer("/selectionRange/start")
        .or_else(|| item.pointer("/range/start"));
    let line = position
        .and_then(|start| start.get("line"))
        .and_then(Value::as_u64)
        .unwrap_or(0)
        + 1;
    let column = position
        .and_then(|start| start.get("character"))
        .and_then(Value::as_u64)
        .unwrap_or(0)
        + 1;
    out.push(LspSymbolInfo {
        name: name.to_owned(),
        kind,
        path: path.to_owned(),
        line,
        column,
        container: container.map(str::to_owned),
    });
    if let Some(children) = item.get("children").and_then(Value::as_array) {
        for child in children {
            flatten_document_symbol(child, Some(name), path, out);
        }
    }
}

/// Converte um `SymbolInformation` (com `location.uri`) num simbolo do
/// protocolo; entradas com URI fora do esquema `file://` sao ignoradas.
fn symbol_information(item: &Value) -> Option<LspSymbolInfo> {
    let name = item.get("name")?.as_str()?;
    let kind = item
        .get("kind")
        .and_then(Value::as_i64)
        .and_then(symbol_kind_name)
        .unwrap_or("symbol")
        .to_owned();
    let uri = item.pointer("/location/uri")?.as_str()?;
    let path = path_for_uri(uri)?;
    let start = item.pointer("/location/range/start");
    let line = start
        .and_then(|position| position.get("line"))
        .and_then(Value::as_u64)
        .unwrap_or(0)
        + 1;
    let column = start
        .and_then(|position| position.get("character"))
        .and_then(Value::as_u64)
        .unwrap_or(0)
        + 1;
    let container = item
        .get("containerName")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_owned);
    Some(LspSymbolInfo {
        name: name.to_owned(),
        kind,
        path,
        line,
        column,
        container,
    })
}

/// Converte `workspace/symbol` nos simbolos do protocolo (ordem do servidor).
pub(super) fn workspace_symbols(result: &Value) -> Vec<LspSymbolInfo> {
    let Some(items) = result.as_array() else {
        return Vec::new();
    };
    let mut symbols = items
        .iter()
        .filter_map(symbol_information)
        .collect::<Vec<_>>();
    symbols.truncate(MAX_WORKSPACE_SYMBOLS);
    symbols
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{document_symbols, workspace_symbols};

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

        let symbols = document_symbols(&result, "/w/src/main.rs");

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

        let symbols = workspace_symbols(&result);

        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "Ponto");
        assert_eq!(symbols[0].kind, "struct");
        assert_eq!(symbols[0].path, "/w/src/lib.rs");
        assert_eq!(symbols[0].line, 3);
        assert_eq!(symbols[0].column, 12);
        assert_eq!(symbols[0].container.as_deref(), Some("geometria"));
        assert!(workspace_symbols(&json!(null)).is_empty());
    }
}

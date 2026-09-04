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

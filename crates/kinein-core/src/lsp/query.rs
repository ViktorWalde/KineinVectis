//! As CONSULTAS interativas como pedidos ADIADOS (Etapa 2, F6, 2026-09-18).
//!
//! Medido na F3: o laco do core parava ate' 4 s em cada `lsp.hover`/
//! `semanticTokens` enquanto o rust-analyzer subia — e com ele parava o
//! `fs.list`, o salvar, o git. Aqui o manager faz a parte que precisa dele
//! (sincronizar o documento, escrever o request) e devolve a ESPERA mais o
//! `finish` que transforma a resposta crua no resultado do protocolo; quem
//! espera e' uma thread do `Core::defer_lsp`, e o laco segue.

use std::path::Path;

use kinein_protocol::{
    LspCompletionResult, LspDefinitionResult, LspHoverResult, LspReferenceItem,
    LspReferencesResult, LspSemanticTokensResult, LspSymbolsResult,
};
use serde_json::{Value, json};

use super::manager::LspManager;
use super::parse::{
    completion_items, decode_semantic_tokens, definition_location, hover_content,
    reference_locations,
};
use super::parse_symbols::{document_symbols, workspace_symbols};
use super::session::LspReply;
use super::types::LspError;
use super::uri::uri_for_path;

/// Qual consulta; os campos sao o que cada uma precisa alem de path/content.
#[derive(Debug, Clone)]
pub enum LspQuery {
    /// `textDocument/hover`.
    Hover {
        /// Linha 1-based.
        line: u64,
        /// Coluna 1-based.
        column: u64,
    },
    /// `textDocument/definition`.
    Definition {
        /// Linha 1-based.
        line: u64,
        /// Coluna 1-based.
        column: u64,
    },
    /// `textDocument/completion`.
    Completion {
        /// Linha 1-based.
        line: u64,
        /// Coluna 1-based.
        column: u64,
    },
    /// `textDocument/references`.
    References {
        /// Linha 1-based.
        line: u64,
        /// Coluna 1-based.
        column: u64,
    },
    /// `textDocument/documentSymbol`.
    DocumentSymbols,
    /// `workspace/symbol`.
    WorkspaceSymbols {
        /// O texto procurado.
        query: String,
    },
    /// `textDocument/semanticTokens/full`; `version` volta no resultado.
    SemanticTokens {
        /// A versao do buffer que a UI mandou.
        version: u64,
    },
}

/// Resposta crua do servidor -> `result` do protocolo Kinein.
pub(super) type Finish = Box<dyn FnOnce(&Value) -> Value + Send>;

/// A consulta em voo: a espera e o que fazer com a resposta.
pub struct LspPending {
    /// A resposta que ainda nao chegou.
    pub reply: LspReply,
    /// Resposta crua do servidor -> `result` do protocolo Kinein.
    pub(crate) finish: Finish,
}

impl std::fmt::Debug for LspPending {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LspPending")
            .field("reply", &self.reply)
            .finish_non_exhaustive()
    }
}

/// O que uma consulta devolve ANTES de esperar: ou a espera, ou um
/// resultado pronto (semantic tokens sem legend = lista vazia, sem viagem).
#[derive(Debug)]
pub enum LspBegun {
    /// Esperar pela resposta do servidor.
    Pending(LspPending),
    /// Ja' esta' respondido.
    Ready(Value),
}

fn position_params(path: &Path, line: u64, column: u64, extra: &Value) -> Value {
    let mut params = json!({
        "textDocument": { "uri": uri_for_path(path) },
        "position": {
            "line": line.saturating_sub(1),
            "character": column.saturating_sub(1),
        },
    });
    if let (Some(target), Some(fields)) = (params.as_object_mut(), extra.as_object()) {
        for (key, value) in fields {
            target.insert(key.clone(), value.clone());
        }
    }
    params
}

impl LspManager {
    /// Sincroniza o documento e ESCREVE a consulta; nao espera.
    ///
    /// # Errors
    /// Arquivo sem servidor, servidor que nao sobe, falha ao escrever.
    pub fn begin_query(
        &mut self,
        query: LspQuery,
        path: &Path,
        content: &str,
    ) -> Result<LspBegun, LspError> {
        let language = self.sync_document(path, content)?;
        let path_text = path.display().to_string();
        let (method, params, finish) = match query {
            LspQuery::SemanticTokens { version } => {
                let legend = self
                    .servers
                    .get(language)
                    .map(super::server::ServerHandle::legend)
                    .unwrap_or_default();
                if legend.is_empty() {
                    return Ok(LspBegun::Ready(json!(LspSemanticTokensResult {
                        path: path_text,
                        version,
                        tokens: Vec::new(),
                    })));
                }
                semantic_tokens_plan(path, path_text, version, legend)
            }
            other => query_plan(other, path, path_text),
        };
        let reply = self.begin_request(language, method, &params)?;
        Ok(LspBegun::Pending(LspPending { reply, finish }))
    }
}

type Plan = (&'static str, Value, Finish);

fn semantic_tokens_plan(path: &Path, path_text: String, version: u64, legend: Vec<String>) -> Plan {
    (
        "textDocument/semanticTokens/full",
        json!({ "textDocument": { "uri": uri_for_path(path) } }),
        Box::new(move |v: &Value| {
            json!(LspSemanticTokensResult {
                path: path_text,
                version,
                tokens: decode_semantic_tokens(v, &legend),
            })
        }),
    )
}

/// Metodo, params e o `finish` de cada consulta (menos semantic tokens,
/// que precisa da legend do servidor).
fn query_plan(query: LspQuery, path: &Path, path_text: String) -> Plan {
    match query {
        LspQuery::Hover { line, column } => (
            "textDocument/hover",
            position_params(path, line, column, &Value::Null),
            Box::new(|v: &Value| {
                json!(LspHoverResult {
                    content: hover_content(v)
                })
            }),
        ),
        LspQuery::Definition { line, column } => (
            "textDocument/definition",
            position_params(path, line, column, &Value::Null),
            Box::new(|v: &Value| {
                let location = definition_location(v);
                json!(LspDefinitionResult {
                    path: location.as_ref().map(|t| t.path.clone()),
                    line: location.as_ref().map(|t| t.line),
                    column: location.as_ref().map(|t| t.column),
                })
            }),
        ),
        LspQuery::Completion { line, column } => (
            "textDocument/completion",
            position_params(path, line, column, &Value::Null),
            Box::new(|v: &Value| {
                let (items, is_incomplete) = completion_items(v);
                json!(LspCompletionResult {
                    items,
                    is_incomplete
                })
            }),
        ),
        LspQuery::References { line, column } => (
            "textDocument/references",
            position_params(
                path,
                line,
                column,
                &json!({ "context": { "includeDeclaration": true } }),
            ),
            Box::new(|v: &Value| {
                let references = reference_locations(v)
                    .into_iter()
                    .map(|l| LspReferenceItem {
                        path: l.path,
                        line: l.line,
                        column: l.column,
                    })
                    .collect();
                json!(LspReferencesResult { references })
            }),
        ),
        LspQuery::DocumentSymbols => (
            "textDocument/documentSymbol",
            json!({ "textDocument": { "uri": uri_for_path(path) } }),
            Box::new(move |v: &Value| {
                json!(LspSymbolsResult {
                    symbols: document_symbols(v, &path_text),
                    path: path_text.clone(),
                    query: None,
                })
            }),
        ),
        LspQuery::WorkspaceSymbols { query } => {
            let asked = query.clone();
            let anchor = path.display().to_string();
            (
                "workspace/symbol",
                json!({ "query": query }),
                // `FnOnce`: o fecho roda uma vez so', entao os valores saem
                // dele por movimento e nao ha' clone a pagar.
                Box::new(move |v: &Value| {
                    json!(LspSymbolsResult {
                        symbols: workspace_symbols(v),
                        path: anchor,
                        query: Some(asked),
                    })
                }),
            )
        }
        // Tratado antes, com a legend do servidor.
        LspQuery::SemanticTokens { .. } => (
            "textDocument/semanticTokens/full",
            json!({ "textDocument": { "uri": uri_for_path(path) } }),
            Box::new(|_: &Value| Value::Null),
        ),
    }
}

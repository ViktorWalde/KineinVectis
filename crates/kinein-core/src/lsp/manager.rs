//! `LspManager`: o estado da sessao e as OPERACOES INTERATIVAS.
//!
//! Aqui moram o struct e o que o roteador `lsp.*` pergunta ao servidor —
//! definition, hover, completion, references, rename, code actions, symbols e
//! semantic tokens. O manager nunca escreve arquivos: rename devolve um plano
//! que o chamador aplica confinado ao workspace.
//!
//! As outras duas metades vivem nos irmaos, e o "e" que havia nesta frase ate
//! 2026-09-02 era o sintoma de que ainda eram tres:
//!
//! - o que o servidor sabe sobre o TEXTO: [`super::sync`];
//! - qual executavel roda cada linguagem, subir/reiniciar/encerrar e o
//!   transporte de request: [`super::session`].

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use kinein_protocol::{LspCodeActionInfo, LspCompletionItem, LspSemanticToken, LspSymbolInfo};
use serde_json::{Value, json};

use super::parse::{
    code_action_infos, completion_items, decode_semantic_tokens, definition_location,
    document_symbols, hover_content, reference_locations, workspace_edit_plan, workspace_symbols,
};
use super::server::{ServerHandle, ServerRegistry};
use super::types::{LspError, LspLocation, WorkspaceEditPlan};
use super::uri::{path_for_uri, uri_for_path};
use super::{EventSender, PendingResponses};

/// Ultima consulta de code actions respondida, com as ações cruas do servidor.
///
/// `lsp.applyCodeAction` só aplica índices desta consulta; qualquer
/// sincronizacao de texto ou troca de raiz a invalida — quem zera o campo e o
/// [`super::sync`], porque as posições dos edits valem para o conteúdo que o
/// servidor viu na consulta.
#[derive(Debug)]
pub(super) struct ActiveCodeActions {
    path: PathBuf,
    actions: Vec<Value>,
}

/// Gerencia os language servers do workspace aberto.
#[derive(Debug)]
pub struct LspManager {
    pub(super) events: EventSender,
    pub(super) servers: HashMap<&'static str, ServerHandle>,
    pub(super) pending: PendingResponses,
    pub(super) root: Option<PathBuf>,
    pub(super) next_request_id: i64,
    pub(super) active_code_actions: Option<ActiveCodeActions>,
    /// Timeouts consecutivos por linguagem; zera em qualquer resposta. A
    /// politica que o consome vive em [`super::session`].
    pub(super) timeout_streak: HashMap<&'static str, u32>,
    /// Qual executavel roda cada linguagem. Ver [`ServerRegistry`].
    pub(super) registry: ServerRegistry,
}

impl LspManager {
    /// Cria um manager que envia notificacoes pelo canal do loop principal.
    #[must_use]
    pub fn new(events: EventSender) -> Self {
        Self {
            events,
            servers: HashMap::new(),
            pending: Arc::new(Mutex::new(HashMap::new())),
            root: None,
            next_request_id: 2,
            active_code_actions: None,
            timeout_streak: HashMap::new(),
            registry: ServerRegistry::default(),
        }
    }

    /// Resolve `textDocument/definition` para a posicao atual do editor.
    pub fn definition(
        &mut self,
        path: &Path,
        content: &str,
        line: u64,
        column: u64,
    ) -> Result<Option<LspLocation>, LspError> {
        let result = self.text_document_position_request(
            "textDocument/definition",
            path,
            content,
            line,
            column,
        )?;
        Ok(definition_location(&result))
    }

    /// Resolve `textDocument/hover` para a posicao atual do editor.
    pub fn hover(
        &mut self,
        path: &Path,
        content: &str,
        line: u64,
        column: u64,
    ) -> Result<Option<String>, LspError> {
        let result =
            self.text_document_position_request("textDocument/hover", path, content, line, column)?;
        Ok(hover_content(&result))
    }

    /// Resolve `textDocument/completion` para a posicao atual do editor.
    ///
    /// Os itens voltam ordenados por `sortText` e limitados ao teto interno.
    pub fn completion(
        &mut self,
        path: &Path,
        content: &str,
        line: u64,
        column: u64,
    ) -> Result<(Vec<LspCompletionItem>, bool), LspError> {
        let result = self.text_document_position_request(
            "textDocument/completion",
            path,
            content,
            line,
            column,
        )?;
        Ok(completion_items(&result))
    }

    /// Resolve `textDocument/references` (find usages) para a posicao atual.
    pub fn references(
        &mut self,
        path: &Path,
        content: &str,
        line: u64,
        column: u64,
    ) -> Result<Vec<LspLocation>, LspError> {
        let result = self.position_request_with(
            "textDocument/references",
            path,
            content,
            line,
            column,
            &json!({ "context": { "includeDeclaration": true } }),
        )?;
        Ok(reference_locations(&result))
    }

    /// Alterna entre header e source via a extensao do clangd
    /// `textDocument/switchSourceHeader`.
    ///
    /// So o servidor C/C++ tem essa extensao: arquivo de outra linguagem
    /// vira `UnsupportedFile` (nunca deixamos rust-analyzer responder
    /// "method not found" cru). Sem contraparte, o clangd responde `null`
    /// e devolvemos `None` — nao e erro.
    pub fn switch_source_header(
        &mut self,
        path: &Path,
        content: &str,
    ) -> Result<Option<String>, LspError> {
        let language = self.sync_document(path, content)?;
        if language != "cpp" {
            return Err(LspError::UnsupportedFile {
                path: path.display().to_string(),
            });
        }
        // A extensao usa TextDocumentIdentifier PLANO ({ uri }), sem o
        // envelope { textDocument } dos requests de posicao.
        let params = json!({ "uri": uri_for_path(path) });
        let result = self.send_request(language, "textDocument/switchSourceHeader", &params)?;
        Ok(result.as_str().and_then(path_for_uri))
    }

    /// Solicita `textDocument/rename` e devolve o plano de edits por arquivo.
    ///
    /// O manager NAO escreve arquivos; quem aplica o plano (confinado ao
    /// workspace) e o chamador.
    pub fn rename(
        &mut self,
        path: &Path,
        content: &str,
        line: u64,
        column: u64,
        new_name: &str,
    ) -> Result<WorkspaceEditPlan, LspError> {
        let result = self.position_request_with(
            "textDocument/rename",
            path,
            content,
            line,
            column,
            &json!({ "newName": new_name }),
        )?;
        workspace_edit_plan(&result)
    }

    /// Resolve `textDocument/codeAction` no ponto do cursor.
    ///
    /// O `context` leva os diagnostics que o proprio servidor publicou para a
    /// linha (cache da thread leitora); a UI nao devolve diagnostico nenhum.
    /// As acoes cruas aplicaveis ficam guardadas como consulta ativa para o
    /// `lsp.applyCodeAction` seguinte.
    pub fn code_actions(
        &mut self,
        path: &Path,
        content: &str,
        line: u64,
        column: u64,
    ) -> Result<Vec<LspCodeActionInfo>, LspError> {
        let language = self.sync_document(path, content)?;
        let uri = uri_for_path(path);
        let context_diagnostics = self
            .servers
            .get(language)
            .map(|handle| diagnostics_for_line(&handle.diagnostics_by_uri, &uri, line))
            .unwrap_or_default();
        let position = json!({
            "line": line.saturating_sub(1),
            "character": column.saturating_sub(1),
        });
        let params = json!({
            "textDocument": { "uri": uri },
            "range": { "start": position, "end": position },
            "context": { "diagnostics": context_diagnostics },
        });
        let result = self.send_request(language, "textDocument/codeAction", &params)?;
        let (infos, raw) = code_action_infos(&result);
        self.active_code_actions = Some(ActiveCodeActions {
            path: path.to_path_buf(),
            actions: raw,
        });
        Ok(infos)
    }

    /// Consome a acao `index` da consulta ativa de `path`, se ela existir.
    ///
    /// Retorna o titulo e o plano de edits ja convertido; `None` quando nao
    /// ha consulta ativa para o arquivo ou o indice nao existe. A consulta
    /// inteira e invalidada na chamada: depois de aplicar um edit o conteudo
    /// muda e os indices antigos deixam de valer.
    pub fn take_code_action(
        &mut self,
        path: &Path,
        index: usize,
    ) -> Option<Result<(String, WorkspaceEditPlan), LspError>> {
        let active = self.active_code_actions.take()?;
        if active.path != path {
            return None;
        }
        let action = active.actions.get(index)?;
        let title = action
            .get("title")
            .and_then(Value::as_str)
            .unwrap_or("code action")
            .to_owned();
        let edit = action.get("edit")?;
        Some(workspace_edit_plan(edit).map(|plan| (title, plan)))
    }

    /// Resolve `textDocument/documentSymbol`: a estrutura achatada do arquivo.
    pub fn document_symbols(
        &mut self,
        path: &Path,
        content: &str,
    ) -> Result<Vec<LspSymbolInfo>, LspError> {
        let language = self.sync_document(path, content)?;
        let params = json!({ "textDocument": { "uri": uri_for_path(path) } });
        let result = self.send_request(language, "textDocument/documentSymbol", &params)?;
        Ok(document_symbols(&result, &path.display().to_string()))
    }

    /// Resolve `workspace/symbol` no servidor da linguagem do arquivo ativo.
    pub fn workspace_symbols(
        &mut self,
        path: &Path,
        content: &str,
        query: &str,
    ) -> Result<Vec<LspSymbolInfo>, LspError> {
        let language = self.sync_document(path, content)?;
        let params = json!({ "query": query });
        let result = self.send_request(language, "workspace/symbol", &params)?;
        Ok(workspace_symbols(&result))
    }

    /// Resolve `textDocument/semanticTokens/full` para o documento inteiro.
    ///
    /// Retorna tokens absolutos ja decodificados com a legend do servidor;
    /// vazio quando o servidor nao suporta semantic tokens.
    pub fn semantic_tokens(
        &mut self,
        path: &Path,
        content: &str,
    ) -> Result<Vec<LspSemanticToken>, LspError> {
        let language = self.sync_document(path, content)?;
        let legend = self
            .servers
            .get(language)
            .map(|handle| handle.semantic_token_types.clone())
            .unwrap_or_default();
        if legend.is_empty() {
            return Ok(Vec::new());
        }
        let params = json!({ "textDocument": { "uri": uri_for_path(path) } });
        let result = self.send_request(language, "textDocument/semanticTokens/full", &params)?;
        Ok(decode_semantic_tokens(&result, &legend))
    }

    fn text_document_position_request(
        &mut self,
        method: &'static str,
        path: &Path,
        content: &str,
        line: u64,
        column: u64,
    ) -> Result<Value, LspError> {
        self.position_request_with(method, path, content, line, column, &Value::Null)
    }

    fn position_request_with(
        &mut self,
        method: &'static str,
        path: &Path,
        content: &str,
        line: u64,
        column: u64,
        extra: &Value,
    ) -> Result<Value, LspError> {
        let language = self.sync_document(path, content)?;
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
        self.send_request(language, method, &params)
    }
}

/// Filtra os diagnostics cacheados de `uri` que intersectam a linha do cursor.
///
/// `line` e 1-based (protocolo Kinein); ranges LSP sao 0-based.
fn diagnostics_for_line(
    cache: &Arc<Mutex<HashMap<String, Value>>>,
    uri: &str,
    line: u64,
) -> Vec<Value> {
    let zero_based = line.saturating_sub(1);
    let Ok(entries) = cache.lock() else {
        return Vec::new();
    };
    let Some(diagnostics) = entries.get(uri).and_then(Value::as_array) else {
        return Vec::new();
    };
    diagnostics
        .iter()
        .filter(|diagnostic| {
            let start = diagnostic
                .pointer("/range/start/line")
                .and_then(Value::as_u64)
                .unwrap_or(0);
            let end = diagnostic
                .pointer("/range/end/line")
                .and_then(Value::as_u64)
                .unwrap_or(start);
            start <= zero_based && zero_based <= end
        })
        .cloned()
        .collect()
}

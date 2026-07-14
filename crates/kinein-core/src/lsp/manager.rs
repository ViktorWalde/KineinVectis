//! `LspManager`: estado da sessao e orquestracao dos language servers.
//!
//! Mantem os servidores por linguagem, a versao de cada documento aberto e o
//! mapa de requests pendentes. Sincroniza documentos com texto completo
//! (didOpen/didChange/didSave) e expoe as operacoes interativas
//! (definition/hover/completion/references/rename/semanticTokens) que o
//! roteador `lsp.*` consome. O manager nunca escreve arquivos: rename devolve
//! um plano que o chamador aplica confinado ao workspace.

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        mpsc::{self},
    },
    time::Duration,
};

use kinein_protocol::{
    JsonRpcRequest, LspCodeActionInfo, LspCompletionItem, LspSemanticToken, LspSymbolInfo,
};
use serde_json::{Value, json};

use super::framing::{full_change_params, send_notification, write_locked_message};
use super::parse::{
    code_action_infos, completion_items, decode_semantic_tokens, definition_location,
    document_symbols, hover_content, reference_locations, response_result, workspace_edit_plan,
    workspace_symbols,
};
use super::server::{ServerHandle, ServerSpec, spawn_server, spec_for_path};
use super::types::{LspError, LspLocation, WorkspaceEditPlan};
use super::uri::{path_for_uri, uri_for_path};
use super::{EventSender, PendingResponses};

/// Tempo maximo aguardando respostas interativas do LSP.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(4);

/// Timeouts CONSECUTIVOS de um servidor antes do auto-restart (M4.3b). Com
/// `REQUEST_TIMEOUT` de 4s, são ~12s preso antes de ressuscitar o servidor.
const MAX_TIMEOUT_STREAK: u32 = 3;

/// Ultima consulta de code actions respondida, com as ações cruas do servidor.
///
/// `lsp.applyCodeAction` só aplica índices desta consulta; qualquer didChange,
/// didOpen ou troca de raiz a invalida (as posições dos edits valem para o
/// conteúdo que o servidor viu na consulta).
#[derive(Debug)]
struct ActiveCodeActions {
    path: PathBuf,
    actions: Vec<Value>,
}

/// Gerencia os language servers do workspace aberto.
#[derive(Debug)]
pub struct LspManager {
    events: EventSender,
    servers: HashMap<&'static str, ServerHandle>,
    pending: PendingResponses,
    root: Option<PathBuf>,
    next_request_id: i64,
    active_code_actions: Option<ActiveCodeActions>,
    /// Timeouts consecutivos por linguagem; zera em qualquer resposta.
    timeout_streak: HashMap<&'static str, u32>,
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
        }
    }

    /// Reinicia o servidor de UMA linguagem: mata o processo e o remove; sobe
    /// de novo (lazy) no próximo request. `true` se havia servidor. A UI
    /// re-sincroniza o arquivo ativo ao ver `event.lsp.restarted` (M4.3b).
    pub fn restart_language(&mut self, language: &str) -> bool {
        let Some(key) = self.servers.keys().copied().find(|k| *k == language) else {
            return false;
        };
        if let Some(mut handle) = self.servers.remove(key) {
            drop(handle.child.kill());
            drop(handle.child.wait());
        }
        self.timeout_streak.remove(key);
        self.emit_status(key, "restarting");
        self.emit_restarted(key);
        true
    }

    /// Reinicia TODOS os servidores vivos; devolve as linguagens reiniciadas.
    pub fn restart_all(&mut self) -> Vec<String> {
        let languages: Vec<&'static str> = self.servers.keys().copied().collect();
        let mut restarted = Vec::new();
        for language in languages {
            if self.restart_language(language) {
                restarted.push(language.to_owned());
            }
        }
        restarted
    }

    /// Registra um timeout do servidor; ao acumular `MAX_TIMEOUT_STREAK`
    /// seguidos, auto-reinicia aquela linguagem (M4.3b).
    fn note_timeout(&mut self, language: &'static str) {
        let streak = {
            let entry = self.timeout_streak.entry(language).or_insert(0);
            *entry += 1;
            *entry
        };
        if streak >= MAX_TIMEOUT_STREAK {
            self.restart_language(language);
        }
    }

    /// Define a raiz do workspace, derrubando servidores da raiz anterior.
    pub fn set_root(&mut self, root: Option<PathBuf>) {
        if self.root == root {
            return;
        }
        self.shutdown_all();
        self.root = root;
    }

    /// Encerra todos os servidores gerenciados.
    pub fn shutdown_all(&mut self) {
        self.active_code_actions = None;
        let drained: Vec<(&'static str, ServerHandle)> = self.servers.drain().collect();
        for (language, mut handle) in drained {
            // Melhor esforco: mata o processo; shutdown educado fica para
            // quando houver cancelamento generico no core.
            drop(handle.child.kill());
            drop(handle.child.wait());
            self.emit_status(language, "stopped");
        }
    }

    /// Abre (ou re-sincroniza) um documento no servidor da linguagem dele.
    ///
    /// Conteudo identico ao ultimo sincronizado nao gera notificacao nova:
    /// alem de evitar reparse no servidor, preserva a consulta ativa de code
    /// actions e os fix-its que o clangd amarra a versao do documento.
    pub fn did_open(&mut self, path: &Path, content: &str) {
        let Some(spec) = spec_for_path(path) else {
            return;
        };
        if self.ensure_server(spec).is_err() {
            return;
        }
        let uri = uri_for_path(path);
        let hash = content_hash(content);
        let Some(handle) = self.servers.get_mut(spec.language) else {
            return;
        };

        if let Some(version) = handle.versions.get_mut(&uri) {
            if handle.content_hashes.get(&uri) == Some(&hash) {
                return;
            }
            self.active_code_actions = None;
            *version += 1;
            let params = full_change_params(&uri, *version, content);
            handle.content_hashes.insert(uri.clone(), hash);
            send_notification(&handle.stdin, "textDocument/didChange", &params);
            return;
        }

        self.active_code_actions = None;
        handle.versions.insert(uri.clone(), 1);
        handle.content_hashes.insert(uri.clone(), hash);
        let params = json!({
            "textDocument": {
                "uri": uri,
                "languageId": spec.language_id,
                "version": 1,
                "text": content,
            }
        });
        send_notification(&handle.stdin, "textDocument/didOpen", &params);
    }

    /// Sincroniza o conteudo atual de um documento (texto completo).
    ///
    /// Sem mudanca real de conteudo, nada e enviado (ver `did_open`).
    pub fn did_change(&mut self, path: &Path, content: &str) {
        let Some(spec) = spec_for_path(path) else {
            return;
        };
        let uri = uri_for_path(path);
        let has_document = self
            .servers
            .get(spec.language)
            .is_some_and(|handle| handle.versions.contains_key(&uri));
        if !has_document {
            self.did_open(path, content);
            return;
        }
        let hash = content_hash(content);
        let Some(handle) = self.servers.get_mut(spec.language) else {
            return;
        };
        if handle.content_hashes.get(&uri) == Some(&hash) {
            return;
        }
        let Some(version) = handle.versions.get_mut(&uri) else {
            return;
        };
        self.active_code_actions = None;
        *version += 1;
        let params = full_change_params(&uri, *version, content);
        handle.content_hashes.insert(uri.clone(), hash);
        send_notification(&handle.stdin, "textDocument/didChange", &params);
    }

    /// Notifica salvamento de um documento.
    pub fn did_save(&mut self, path: &Path, content: &str) {
        self.did_change(path, content);
        let Some(spec) = spec_for_path(path) else {
            return;
        };
        let Some(handle) = self.servers.get(spec.language) else {
            return;
        };
        let params = json!({
            "textDocument": { "uri": uri_for_path(path) },
            "text": content,
        });
        send_notification(&handle.stdin, "textDocument/didSave", &params);
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
        let spec = self.sync_document(path, content)?;
        if spec.language != "cpp" {
            return Err(LspError::UnsupportedFile {
                path: path.display().to_string(),
            });
        }
        // A extensao usa TextDocumentIdentifier PLANO ({ uri }), sem o
        // envelope { textDocument } dos requests de posicao.
        let params = json!({ "uri": uri_for_path(path) });
        let result =
            self.send_request(spec.language, "textDocument/switchSourceHeader", &params)?;
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
        let spec = self.sync_document(path, content)?;
        let uri = uri_for_path(path);
        let context_diagnostics = self
            .servers
            .get(spec.language)
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
        let result = self.send_request(spec.language, "textDocument/codeAction", &params)?;
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
        let spec = self.sync_document(path, content)?;
        let params = json!({ "textDocument": { "uri": uri_for_path(path) } });
        let result = self.send_request(spec.language, "textDocument/documentSymbol", &params)?;
        Ok(document_symbols(&result, &path.display().to_string()))
    }

    /// Resolve `workspace/symbol` no servidor da linguagem do arquivo ativo.
    pub fn workspace_symbols(
        &mut self,
        path: &Path,
        content: &str,
        query: &str,
    ) -> Result<Vec<LspSymbolInfo>, LspError> {
        let spec = self.sync_document(path, content)?;
        let params = json!({ "query": query });
        let result = self.send_request(spec.language, "workspace/symbol", &params)?;
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
        let spec = self.sync_document(path, content)?;
        let legend = self
            .servers
            .get(spec.language)
            .map(|handle| handle.semantic_token_types.clone())
            .unwrap_or_default();
        if legend.is_empty() {
            return Ok(Vec::new());
        }
        let params = json!({ "textDocument": { "uri": uri_for_path(path) } });
        let result =
            self.send_request(spec.language, "textDocument/semanticTokens/full", &params)?;
        Ok(decode_semantic_tokens(&result, &legend))
    }

    /// Re-sincroniza um documento que ja esta aberto no servidor.
    ///
    /// Usado depois de um rename reescrever arquivos no disco; documentos que
    /// o servidor nao conhece ficam intactos (nenhum `didOpen` novo).
    pub fn sync_if_open(&mut self, path: &Path, content: &str) {
        let Some(spec) = spec_for_path(path) else {
            return;
        };
        let uri = uri_for_path(path);
        let hash = content_hash(content);
        let Some(handle) = self.servers.get_mut(spec.language) else {
            return;
        };
        let Some(version) = handle.versions.get_mut(&uri) else {
            return;
        };
        if handle.content_hashes.get(&uri) == Some(&hash) {
            return;
        }
        *version += 1;
        let params = full_change_params(&uri, *version, content);
        handle.content_hashes.insert(uri.clone(), hash);
        send_notification(&handle.stdin, "textDocument/didChange", &params);
    }

    /// Versao do documento que o servidor da linguagem conhece, se aberto.
    ///
    /// Usada para rejeitar `WorkspaceEdit.documentChanges` obsoleto antes de
    /// criar uma transacao de escrita.
    #[must_use]
    pub fn document_version(&self, path: &Path) -> Option<i64> {
        let spec = spec_for_path(path)?;
        let uri = uri_for_path(path);
        self.servers
            .get(spec.language)
            .and_then(|handle| handle.versions.get(&uri))
            .copied()
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
        let spec = self.sync_document(path, content)?;
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
        self.send_request(spec.language, method, &params)
    }

    fn sync_document(
        &mut self,
        path: &Path,
        content: &str,
    ) -> Result<&'static ServerSpec, LspError> {
        let Some(spec) = spec_for_path(path) else {
            return Err(LspError::UnsupportedFile {
                path: path.display().to_string(),
            });
        };
        self.ensure_server(spec)?;
        let uri = uri_for_path(path);
        let Some(handle) = self.servers.get_mut(spec.language) else {
            return Err(LspError::Transport {
                message: format!("servidor {} nao esta registrado", spec.language),
            });
        };

        if let Some(version) = handle.versions.get_mut(&uri) {
            *version += 1;
            let params = full_change_params(&uri, *version, content);
            send_notification(&handle.stdin, "textDocument/didChange", &params);
        } else {
            handle.versions.insert(uri.clone(), 1);
            let params = json!({
                "textDocument": {
                    "uri": uri,
                    "languageId": spec.language_id,
                    "version": 1,
                    "text": content,
                }
            });
            send_notification(&handle.stdin, "textDocument/didOpen", &params);
        }

        Ok(spec)
    }

    fn ensure_server(&mut self, spec: &'static ServerSpec) -> Result<(), LspError> {
        if self.servers.contains_key(spec.language) {
            return Ok(());
        }
        let Some(root) = self.root.clone() else {
            return Err(LspError::Transport {
                message: "nenhum workspace LSP configurado".to_owned(),
            });
        };

        match spawn_server(spec, &root, self.events.clone(), Arc::clone(&self.pending)) {
            Ok(handle) => {
                self.servers.insert(spec.language, handle);
                self.emit_status(spec.language, "running");
                Ok(())
            }
            Err(error) => {
                let message = error.to_string();
                self.emit_status_message(spec.language, "failed", Some(&message));
                Err(error)
            }
        }
    }

    fn send_request(
        &mut self,
        language: &'static str,
        method: &'static str,
        params: &Value,
    ) -> Result<Value, LspError> {
        let id = self.next_request_id;
        self.next_request_id += 1;
        let Some(handle) = self.servers.get(language) else {
            return Err(LspError::Transport {
                message: format!("servidor {language} nao esta em execucao"),
            });
        };
        let (response_tx, response_rx) = mpsc::channel::<Value>();
        {
            let Ok(mut pending) = self.pending.lock() else {
                return Err(LspError::Transport {
                    message: "mapa de requests LSP envenenado".to_owned(),
                });
            };
            pending.insert(id, response_tx);
        }

        let message = json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params });
        if let Err(error) = write_locked_message(&handle.stdin, &message) {
            self.remove_pending(id);
            return Err(LspError::Transport {
                message: format!("falha ao enviar {method}: {error}"),
            });
        }

        match response_rx.recv_timeout(REQUEST_TIMEOUT) {
            Ok(response) => {
                // Qualquer resposta (mesmo erro do LSP) prova que o servidor
                // está vivo: zera a contagem de timeouts (M4.3b).
                self.timeout_streak.remove(language);
                response_result(method, &response)
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                self.remove_pending(id);
                self.note_timeout(language);
                Err(LspError::Timeout { method })
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => Err(LspError::Transport {
                message: format!("canal de resposta de {method} foi fechado"),
            }),
        }
    }

    fn remove_pending(&self, id: i64) {
        if let Ok(mut pending) = self.pending.lock() {
            pending.remove(&id);
        }
    }

    fn emit_status(&self, language: &str, status: &str) {
        self.emit_status_message(language, status, None);
    }

    /// Avisa a UI que um servidor reiniciou, para ela re-sincronizar o
    /// arquivo ativo (mesmo caminho do `recovered()` do crash — M4.3b).
    fn emit_restarted(&self, language: &str) {
        drop(self.events.send(JsonRpcRequest::notification(
            "event.lsp.restarted",
            Some(json!({ "language": language })),
        )));
    }

    fn emit_status_message(&self, language: &str, status: &str, message: Option<&str>) {
        let mut params = json!({ "language": language, "status": status });
        if let (Some(map), Some(text)) = (params.as_object_mut(), message) {
            map.insert("message".to_owned(), Value::String(text.to_owned()));
        }
        drop(self.events.send(JsonRpcRequest::notification(
            "event.lsp.status",
            Some(params),
        )));
    }
}

impl Drop for LspManager {
    fn drop(&mut self) {
        self.shutdown_all();
    }
}

/// Hash estavel do conteudo de um documento para detectar sync redundante.
fn content_hash(content: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    content.hash(&mut hasher);
    hasher.finish()
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

#[cfg(test)]
mod tests {
    use std::sync::mpsc;

    use super::LspManager;

    #[test]
    fn restart_without_server_is_noop() {
        let (sender, _receiver) = mpsc::channel();
        let mut manager = LspManager::new(sender);
        // Sem servidor vivo, reiniciar uma linguagem é no-op; todos = vazio.
        assert!(!manager.restart_language("rust"));
        assert!(manager.restart_all().is_empty());
    }
}

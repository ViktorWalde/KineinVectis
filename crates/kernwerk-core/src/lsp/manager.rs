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

use kernwerk_protocol::{JsonRpcRequest, LspCompletionItem, LspSemanticToken};
use serde_json::{Value, json};

use super::framing::{full_change_params, send_notification, write_locked_message};
use super::parse::{
    completion_items, decode_semantic_tokens, definition_location, hover_content,
    reference_locations, response_result, workspace_edit_plan,
};
use super::server::{ServerHandle, ServerSpec, spawn_server, spec_for_path};
use super::types::{LspError, LspLocation, WorkspaceEditPlan};
use super::uri::uri_for_path;
use super::{EventSender, PendingResponses};

/// Tempo maximo aguardando respostas interativas do LSP.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(4);

/// Gerencia os language servers do workspace aberto.
#[derive(Debug)]
pub struct LspManager {
    events: EventSender,
    servers: HashMap<&'static str, ServerHandle>,
    pending: PendingResponses,
    root: Option<PathBuf>,
    next_request_id: i64,
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
    pub fn did_open(&mut self, path: &Path, content: &str) {
        let Some(spec) = spec_for_path(path) else {
            return;
        };
        if self.ensure_server(spec).is_err() {
            return;
        }
        let uri = uri_for_path(path);
        let Some(handle) = self.servers.get_mut(spec.language) else {
            return;
        };

        if let Some(version) = handle.versions.get_mut(&uri) {
            *version += 1;
            let params = full_change_params(&uri, *version, content);
            send_notification(&handle.stdin, "textDocument/didChange", &params);
            return;
        }

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

    /// Sincroniza o conteudo atual de um documento (texto completo).
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
        let Some(handle) = self.servers.get_mut(spec.language) else {
            return;
        };
        let Some(version) = handle.versions.get_mut(&uri) else {
            return;
        };
        *version += 1;
        let params = full_change_params(&uri, *version, content);
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
    ) -> Result<Vec<LspCompletionItem>, LspError> {
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
        let Some(handle) = self.servers.get_mut(spec.language) else {
            return;
        };
        let Some(version) = handle.versions.get_mut(&uri) else {
            return;
        };
        *version += 1;
        let params = full_change_params(&uri, *version, content);
        send_notification(&handle.stdin, "textDocument/didChange", &params);
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
            Ok(response) => response_result(method, &response),
            Err(mpsc::RecvTimeoutError::Timeout) => {
                self.remove_pending(id);
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

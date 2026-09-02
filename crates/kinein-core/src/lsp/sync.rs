//! Sincronizacao de documentos com o language server.
//!
//! Responsabilidade unica: manter o servidor sabendo o que o editor tem aberto
//! — `didOpen`, `didChange`, `didSave`, `didClose` e a versao conhecida de cada
//! documento.
//! Quem gerencia PROCESSO e request interativo e o [`super::manager`]; aqui so
//! se decide o que o servidor precisa saber sobre o TEXTO.
//!
//! A regra que governa este modulo e o curto-circuito por hash: conteudo
//! identico ao ultimo sincronizado nao gera notificacao nenhuma. Alem de evitar
//! reparse, ele preserva os fix-its que o clangd amarra a versao do documento —
//! e e por isso que cada request posicional pode re-sincronizar sem custo.

use std::path::Path;

use serde_json::json;

use super::framing::{full_change_params, send_notification};
use super::manager::LspManager;
use super::server::language_for_path;
use super::types::LspError;
use super::uri::uri_for_path;

impl LspManager {
    /// Abre (ou re-sincroniza) um documento no servidor da linguagem dele.
    ///
    /// Conteudo identico ao ultimo sincronizado nao gera notificacao nova:
    /// alem de evitar reparse no servidor, preserva a consulta ativa de code
    /// actions e os fix-its que o clangd amarra a versao do documento.
    pub fn did_open(&mut self, path: &Path, content: &str) {
        let Some(spec) = self.registry.spec_for_path(path) else {
            return;
        };
        if self.ensure_server(&spec).is_err() {
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
        let Some(language) = language_for_path(path) else {
            return;
        };
        let uri = uri_for_path(path);
        let has_document = self
            .servers
            .get(language)
            .is_some_and(|handle| handle.versions.contains_key(&uri));
        if !has_document {
            self.did_open(path, content);
            return;
        }
        let hash = content_hash(content);
        let Some(handle) = self.servers.get_mut(language) else {
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
        let Some(language) = language_for_path(path) else {
            return;
        };
        let Some(handle) = self.servers.get(language) else {
            return;
        };
        let params = json!({
            "textDocument": { "uri": uri_for_path(path) },
            "text": content,
        });
        send_notification(&handle.stdin, "textDocument/didSave", &params);
    }

    /// Fecha um documento no servidor da linguagem dele.
    ///
    /// Devolve `true` quando havia mesmo um documento aberto para fechar. O
    /// servidor descarta o texto e os diagnosticos daquele URI; a proxima
    /// abertura recomeca na versao 1.
    ///
    /// Existe por dois motivos concretos, nao por simetria com o `did_open`:
    /// um arquivo APAGADO que continua aberto no servidor deixa diagnosticos
    /// de um arquivo que nao existe mais (o chamador de hoje, `fs.delete`), e a
    /// reabertura apos `cmake.configure` precisa de `didClose` + `didOpen` para
    /// o clangd recompilar com as flags novas — a etapa 4 do roadmap 30, que
    /// estava barrada por nao existir nem a operacao nem prova dela.
    pub fn did_close(&mut self, path: &Path) -> bool {
        let Some(language) = language_for_path(path) else {
            return false;
        };
        let uri = uri_for_path(path);
        let Some(handle) = self.servers.get_mut(language) else {
            return false;
        };
        if handle.versions.remove(&uri).is_none() {
            return false;
        }
        handle.content_hashes.remove(&uri);
        // As posicoes das code actions valem para o conteudo que o servidor
        // viu; fechar o documento invalida a consulta como qualquer sync.
        self.active_code_actions = None;
        let Some(handle) = self.servers.get(language) else {
            return false;
        };
        let params = json!({ "textDocument": { "uri": uri } });
        send_notification(&handle.stdin, "textDocument/didClose", &params);
        true
    }

    /// Re-sincroniza um documento que ja esta aberto no servidor.
    ///
    /// Usado depois de um rename reescrever arquivos no disco; documentos que
    /// o servidor nao conhece ficam intactos (nenhum `didOpen` novo).
    pub fn sync_if_open(&mut self, path: &Path, content: &str) {
        let Some(language) = language_for_path(path) else {
            return;
        };
        let uri = uri_for_path(path);
        let hash = content_hash(content);
        let Some(handle) = self.servers.get_mut(language) else {
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
        let language = language_for_path(path)?;
        let uri = uri_for_path(path);
        self.servers
            .get(language)
            .and_then(|handle| handle.versions.get(&uri))
            .copied()
    }

    /// Garante o servidor da linguagem e sincroniza o texto; devolve a
    /// LINGUAGEM, que e o que todo chamador precisa daqui.
    pub(super) fn sync_document(
        &mut self,
        path: &Path,
        content: &str,
    ) -> Result<&'static str, LspError> {
        let Some(spec) = self.registry.spec_for_path(path) else {
            return Err(LspError::UnsupportedFile {
                path: path.display().to_string(),
            });
        };
        self.ensure_server(&spec)?;
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

        Ok(spec.language)
    }
}

/// Hash estavel do conteudo de um documento para detectar sync redundante.
fn content_hash(content: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    content.hash(&mut hasher);
    hasher.finish()
}

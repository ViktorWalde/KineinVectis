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
//!
//! Desde 2026-09-13 uma linguagem pode ter mais de um servidor (o principal e
//! os companheiros, ver [`super::server::ServerSpec`]): o TEXTO vai para
//! todos, cada um com a propria versao do documento; o que e' pergunta
//! interativa continua indo so' ao principal.

use std::path::Path;

use serde_json::json;

use super::framing::{full_change_params, send_notification};
use super::manager::LspManager;
use super::server::{ServerHandle, language_for_path};
use super::types::LspError;
use super::uri::uri_for_path;

impl LspManager {
    /// Abre (ou re-sincroniza) um documento nos servidores da linguagem dele.
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
        let mut enviou = false;
        let mut keys = vec![spec.key];
        keys.extend(self.ensure_companions(path).iter().map(|c| c.key));
        for key in keys {
            if let Some(handle) = self.servers.get_mut(key) {
                enviou |= open_or_change(handle, &uri, spec.language_id, content, hash, true);
            }
        }
        if enviou {
            self.active_code_actions = None;
        }
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
        let language_id = self.registry.language_id_of(language);
        let mut enviou = false;
        for key in self.registry.keys_of(language) {
            if let Some(handle) = self.servers.get_mut(key) {
                if handle.versions.contains_key(&uri) {
                    enviou |= open_or_change(handle, &uri, language_id, content, hash, true);
                }
            }
        }
        if enviou {
            self.active_code_actions = None;
        }
    }

    /// Notifica salvamento de um documento.
    pub fn did_save(&mut self, path: &Path, content: &str) {
        self.did_change(path, content);
        let Some(language) = language_for_path(path) else {
            return;
        };
        let uri = uri_for_path(path);
        let params = json!({
            "textDocument": { "uri": uri },
            "text": content,
        });
        for key in self.registry.keys_of(language) {
            if let Some(handle) = self.servers.get(key) {
                if handle.versions.contains_key(&uri) {
                    send_notification(&handle.stdin, "textDocument/didSave", &params);
                }
            }
        }
    }

    /// Fecha um documento nos servidores da linguagem dele.
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
        let mut fechou = false;
        for key in self.registry.keys_of(language) {
            if let Some(handle) = self.servers.get_mut(key) {
                fechou |= close_in(handle, &uri);
            }
        }
        if fechou {
            // As posicoes das code actions valem para o conteudo que o
            // servidor viu; fechar o documento invalida a consulta como
            // qualquer sync.
            self.active_code_actions = None;
        }
        fechou
    }

    /// Fecha TODOS os documentos abertos de uma linguagem; devolve quantos
    /// (contados no principal — os companheiros seguem o mesmo conjunto).
    ///
    /// Existe para o caso medido no `roadmaps/29` §5b: o clangd tem hot-reload
    /// da `compile_commands.json` desde a v12 (reconfere a cada ~5 s,
    /// <https://reviews.llvm.org/D92663>), mas **o documento ja aberto fica com
    /// a compilacao em cache**. Reiniciar o servidor resolveria e jogaria o
    /// indice fora; a acao certa e reabrir os documentos.
    ///
    /// E por que FECHAR e nao "reenviar didOpen": o curto-circuito por hash
    /// deste modulo torna um `didOpen` repetido INERTE — depois do configure o
    /// texto nao mudou. Fechado, o documento sai de `versions`, e o proximo
    /// `didOpen` volta a ser real, **com o buffer do editor**, nao com o que o
    /// core teria lido do disco.
    pub fn close_documents(&mut self, language: &str) -> usize {
        let mut fechados = 0;
        for key in self.registry.keys_of(language) {
            let Some(handle) = self.servers.get_mut(key) else {
                continue;
            };
            let uris: Vec<String> = handle.versions.keys().cloned().collect();
            for uri in &uris {
                close_in(handle, uri);
            }
            if key == language {
                fechados = uris.len();
            }
        }
        if fechados > 0 {
            self.active_code_actions = None;
        }
        fechados
    }

    /// Re-sincroniza um documento que ja esta aberto nos servidores.
    ///
    /// Usado depois de um rename reescrever arquivos no disco; documentos que
    /// o servidor nao conhece ficam intactos (nenhum `didOpen` novo).
    pub fn sync_if_open(&mut self, path: &Path, content: &str) {
        let Some(language) = language_for_path(path) else {
            return;
        };
        let uri = uri_for_path(path);
        let hash = content_hash(content);
        let language_id = self.registry.language_id_of(language);
        for key in self.registry.keys_of(language) {
            if let Some(handle) = self.servers.get_mut(key) {
                if handle.versions.contains_key(&uri) {
                    open_or_change(handle, &uri, language_id, content, hash, true);
                }
            }
        }
    }

    /// Versao do documento que o servidor PRINCIPAL da linguagem conhece, se
    /// aberto.
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

    /// Garante o servidor principal da linguagem e sincroniza o texto nele e
    /// nos companheiros vivos; devolve a LINGUAGEM (= a chave do principal),
    /// que e o que todo chamador precisa daqui.
    ///
    /// No principal o `didChange` vai SEMPRE (e' o que o roadmap 30 §3 mediu
    /// e os testes do fio esperam); nos companheiros so' quando o texto
    /// mudou — um linter re-analisa a cada didChange, e a pergunta
    /// interativa nao e' para ele.
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
        let hash = content_hash(content);
        let Some(handle) = self.servers.get_mut(spec.key) else {
            return Err(LspError::Transport {
                message: format!("servidor {} nao esta registrado", spec.key),
            });
        };
        open_or_change(handle, &uri, spec.language_id, content, hash, false);
        for companion in self.ensure_companions(path) {
            if let Some(handle) = self.servers.get_mut(companion.key) {
                open_or_change(handle, &uri, spec.language_id, content, hash, true);
            }
        }
        Ok(spec.language)
    }

}

/// `didOpen` na primeira vez; depois `didChange` com a versao seguinte — ou
/// nada, quando `skip_if_same` e o texto e' o ultimo que este servidor viu.
/// Devolve se alguma notificacao saiu.
fn open_or_change(
    handle: &mut ServerHandle,
    uri: &str,
    language_id: &str,
    content: &str,
    hash: u64,
    skip_if_same: bool,
) -> bool {
    if let Some(version) = handle.versions.get_mut(uri) {
        if skip_if_same && handle.content_hashes.get(uri) == Some(&hash) {
            return false;
        }
        *version += 1;
        let params = full_change_params(uri, *version, content);
        if skip_if_same {
            handle.content_hashes.insert(uri.to_owned(), hash);
        }
        send_notification(&handle.stdin, "textDocument/didChange", &params);
        return true;
    }
    handle.versions.insert(uri.to_owned(), 1);
    handle.content_hashes.insert(uri.to_owned(), hash);
    let params = json!({
        "textDocument": {
            "uri": uri,
            "languageId": language_id,
            "version": 1,
            "text": content,
        }
    });
    send_notification(&handle.stdin, "textDocument/didOpen", &params);
    true
}

/// `didClose` num servidor, se ele tinha o documento; devolve se tinha.
fn close_in(handle: &mut ServerHandle, uri: &str) -> bool {
    if handle.versions.remove(uri).is_none() {
        return false;
    }
    handle.content_hashes.remove(uri);
    let params = json!({ "textDocument": { "uri": uri } });
    send_notification(&handle.stdin, "textDocument/didClose", &params);
    true
}

/// Hash estavel do conteudo de um documento para detectar sync redundante.
fn content_hash(content: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    content.hash(&mut hasher);
    hasher.finish()
}

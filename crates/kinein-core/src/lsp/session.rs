//! Ciclo de vida da sessao LSP: qual executavel, subir, reiniciar, encerrar.
//!
//! # Por que isto saiu do `manager.rs` em 2026-09-02
//!
//! O doc do manager dizia, com um "e" no meio: "mantem os servidores por
//! linguagem e o mapa de requests pendentes, **e** expoe as operacoes
//! interativas". Sao duas responsabilidades, e a catraca cobrou quando a etapa
//! 3 do roadmap 30 precisou acrescentar a troca de executavel. As duas saidas
//! faceis — encolher o comentario ate caber, ou subir o baseline — sao as
//! trapacas que a `ARCHITECTURE.md` §4 regra 9 nomeia; a terceira e esta, e e a
//! mesma de 2026-08-30, quando o `sync.rs` levou embora o que o servidor sabe
//! sobre o TEXTO.
//!
//! O que mora aqui: a TABELA de executaveis, subir sob demanda, reiniciar
//! (inclusive o auto-restart por timeout), encerrar, e o transporte de request
//! que precisa desses tres. O que ficou la: as operacoes interativas que a UI
//! pede (definition, hover, completion, ...).

use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex, mpsc},
    time::Duration,
};

use kinein_protocol::JsonRpcRequest;
use serde_json::{Value, json};

use super::PendingResponses;
use super::framing::write_locked_message;
use super::manager::LspManager;
use super::parse::response_result;
use super::registry::ServerSpec;
use super::server::{ServerHandle, spawn_server};
use super::types::LspError;

/// Tempo maximo aguardando respostas interativas do LSP.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(4);

/// Timeouts CONSECUTIVOS de um servidor antes do auto-restart (M4.3b). Com
/// `REQUEST_TIMEOUT` de 4s, sao ~12s preso antes de ressuscitar o servidor.
const MAX_TIMEOUT_STREAK: u32 = 3;

/// Quanto o laco espera pelo handshake de um servidor RECEM-subido (Etapa 2
/// F6): o bastante para um servidor rapido ficar pronto na mesma chamada, e
/// pouco o bastante para nao ser sentido — o resto do handshake e' da thread.
const HANDSHAKE_GRACE: Duration = Duration::from_millis(300);

impl LspManager {
    /// Reinicia os servidores de UMA linguagem — o principal e os
    /// companheiros vivos: mata cada processo e o remove; sobem de novo
    /// (lazy) no proximo request. `true` se havia algum. A UI re-sincroniza o
    /// arquivo ativo ao ver `event.lsp.restarted` (M4.3b) — um evento por
    /// linguagem, nao por processo.
    pub fn restart_language(&mut self, language: &str) -> bool {
        let keys: Vec<&'static str> = self
            .servers
            .keys()
            .copied()
            .filter(|key| self.registry.keys_of(language).contains(key) || *key == language)
            .collect();
        if keys.is_empty() {
            return false;
        }
        for key in &keys {
            self.kill_server(key);
            self.emit_status(key, "restarting");
        }
        if let Some(key) = keys
            .iter()
            .find(|key| **key == language)
            .or_else(|| keys.first())
        {
            self.emit_restarted(key);
        }
        true
    }

    /// Mata UM servidor pela chave e esquece o que ele publicou (a tela
    /// perde os diagnosticos dele na hora, nao no proximo evento).
    fn kill_server(&mut self, key: &'static str) {
        if let Some(handle) = self.servers.remove(key) {
            kill_child(&handle);
        }
        self.reset_timeout_streak(key);
        for event in super::diagnostics_merge::forget(&self.merged_diagnostics, key) {
            drop(self.events.send(event));
        }
    }

    /// Reinicia TODOS os servidores vivos; devolve as linguagens reiniciadas.
    pub fn restart_all(&mut self) -> Vec<String> {
        let mut languages: Vec<&'static str> = self
            .servers
            .keys()
            .filter_map(|key| self.registry.language_of(key))
            .collect();
        languages.sort_unstable();
        languages.dedup();
        let mut restarted = Vec::new();
        for language in languages {
            if self.restart_language(language) {
                restarted.push(language.to_owned());
            }
        }
        restarted
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
        super::diagnostics_merge::clear(&self.merged_diagnostics);
        let drained: Vec<(&'static str, ServerHandle)> = self.servers.drain().collect();
        for (key, handle) in drained {
            // Melhor esforco: mata o processo; shutdown educado fica para
            // quando houver cancelamento generico no core.
            kill_child(&handle);
            self.emit_status(key, "stopped");
        }
    }

    /// Aponta uma linguagem para outro executavel de language server.
    ///
    /// Devolve `false` quando a linguagem nao existe na tabela. Serve hoje ao
    /// GATE: ate 2026-09-02 nenhum teste deste repositorio conseguia observar o
    /// que o core FALA com um servidor, porque o executavel estava fixado no
    /// binario — e 15 metodos `lsp.*` viviam sem um unico teste de
    /// comportamento (roadmap 30, etapa 3). Um servidor falso entra por aqui.
    ///
    /// Nao e mecanismo sem usuario: e a mesma pergunta que a etapa 5 (toolchain
    /// como entidade) tera de responder para o produto — "qual executavel roda
    /// esta linguagem?" —, e o servidor ja em execucao NAO e trocado, para a
    /// troca nunca derrubar uma sessao viva sem pedido explicito.
    pub fn use_server_command(&mut self, language: &str, command: &str, args: &[&str]) -> bool {
        self.registry.set_command(language, command, args)
    }

    /// Troca a configuracao (`settings`) que uma linguagem recebe ao subir.
    pub fn use_server_settings(&mut self, language: &str, settings: Value) -> bool {
        self.registry.set_settings(language, settings)
    }

    /// Poe um COMPANHEIRO ao lado do principal de `language` (o `ruff server`
    /// ao lado do basedpyright): mesmo texto, diagnosticos fundidos, code
    /// actions na mesma lista. Registrar de novo troca o executavel sem
    /// derrubar o que esta' vivo (como `use_server_command`). `false` quando a
    /// linguagem nao existe ou a chave e' de um principal.
    pub fn use_companion(
        &mut self,
        language: &'static str,
        key: &'static str,
        command: &str,
        args: &[&str],
    ) -> bool {
        self.registry.add_companion(language, key, command, args)
    }

    /// Tira um companheiro: mata o processo se vivo, e a tela perde os
    /// diagnosticos dele. `false` se nao estava registrado.
    pub fn disable_companion(&mut self, key: &str) -> bool {
        if let Some(vivo) = self.servers.keys().copied().find(|k| *k == key) {
            self.kill_server(vivo);
            self.emit_status(vivo, "stopped");
        }
        self.registry.remove(key)
    }

    /// Ha' um servidor vivo com esta chave (`python`, `python-ruff`)?
    #[must_use]
    pub fn is_running(&self, key: &str) -> bool {
        self.servers.contains_key(key)
    }

    pub(super) fn ensure_server(&mut self, spec: &ServerSpec) -> Result<(), LspError> {
        // Um handshake que falhou na thread: o servidor sai da tabela agora,
        // com o motivo, e o proximo pedido sobe outro.
        let falhou = self
            .servers
            .get(spec.key)
            .and_then(|h| h.failed.lock().ok().and_then(|f| f.clone()));
        if let Some(message) = falhou {
            self.servers.remove(spec.key);
            return Err(LspError::ServerFailed {
                command: spec.command.clone(),
                message,
            });
        }
        if self.servers.contains_key(spec.key) {
            return Ok(());
        }
        let Some(root) = self.root.clone() else {
            return Err(LspError::Transport {
                message: "nenhum workspace LSP configurado".to_owned(),
            });
        };

        match spawn_server(
            spec,
            &root,
            self.events.clone(),
            Arc::clone(&self.pending),
            Arc::clone(&self.merged_diagnostics),
        ) {
            Ok(handle) => {
                // Um servidor rapido (clangd num projeto pequeno, o falso
                // dos testes) responde ao initialize em milissegundos: o
                // laco espera ate' HANDSHAKE_GRACE por ele, e nao mais — o
                // rust-analyzer que leva segundos sobe por conta da thread,
                // e a UI re-sincroniza ao ver `running`.
                let deadline = std::time::Instant::now() + HANDSHAKE_GRACE;
                while !handle.is_ready() && std::time::Instant::now() < deadline {
                    let falhou = handle.failed.lock().map_or(true, |f| f.is_some());
                    if falhou {
                        break;
                    }
                    std::thread::sleep(Duration::from_millis(5));
                }
                let pronto = handle.is_ready();
                self.servers.insert(spec.key, handle);
                if !pronto {
                    // `running` vem da thread do handshake (Etapa 2 F6).
                    self.emit_status(spec.key, "starting");
                }
                Ok(())
            }
            Err(error) => {
                let message = error.to_string();
                self.emit_status_message(spec.key, "failed", Some(&message));
                Err(error)
            }
        }
    }

    /// Sobe os companheiros de um arquivo, se ainda nao estao vivos. Um que
    /// nao sobe SAI da tabela: o motivo vai uma vez no `event.lsp.status`
    /// (`failed`), e o principal nao paga por ele a cada sincronizacao.
    pub(super) fn ensure_companions(&mut self, path: &std::path::Path) -> Vec<ServerSpec> {
        let mut vivos = Vec::new();
        for spec in self.registry.companions_for_path(path) {
            if self.ensure_server(&spec).is_ok() {
                vivos.push(spec);
            } else {
                self.registry.remove(spec.key);
            }
        }
        vivos
    }

    /// Escreve o request e devolve a ESPERA — quem chama decide se bloqueia
    /// ([`LspReply::wait`]) ou espera noutra thread (F6 da Etapa 2,
    /// 2026-09-18: o laco do core nao pode parar 4 s por um hover).
    ///
    /// # Errors
    /// Servidor ausente ou falha ao escrever no stdin dele.
    pub(super) fn begin_request(
        &mut self,
        key: &'static str,
        method: &'static str,
        params: &Value,
    ) -> Result<LspReply, LspError> {
        // Tres timeouts seguidos derrubam o servidor (M4.3b). Com a espera
        // fora do laco, a contagem e' compartilhada e a decisao e' tomada
        // AQUI, no proximo pedido — nunca na thread que esperou.
        if self.timeout_streak_of(key) >= MAX_TIMEOUT_STREAK {
            self.reset_timeout_streak(key);
            self.kill_server(key);
            self.emit_status(key, "restarting");
            self.emit_restarted(key);
            return Err(LspError::Transport {
                message: format!("servidor {key} reiniciado apos {MAX_TIMEOUT_STREAK} timeouts"),
            });
        }
        let Some(handle) = self.servers.get(key) else {
            return Err(LspError::Transport {
                message: format!("servidor {key} nao esta em execucao"),
            });
        };
        if !handle.is_ready() {
            return Err(LspError::Starting { key });
        }
        let id = self.next_request_id;
        self.next_request_id += 1;
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
        Ok(LspReply {
            key,
            method,
            id,
            rx: response_rx,
            pending: Arc::clone(&self.pending),
            streak: Arc::clone(&self.timeout_streak),
        })
    }

    /// [`Self::begin_request`] + espera SINCRONA: o caminho dos pedidos que
    /// precisam do resultado no laco (rename, workspaceEdit) e dos testes.
    pub(super) fn send_request(
        &mut self,
        key: &'static str,
        method: &'static str,
        params: &Value,
    ) -> Result<Value, LspError> {
        self.begin_request(key, method, params)?.wait()
    }

    fn timeout_streak_of(&self, key: &str) -> u32 {
        self.timeout_streak
            .lock()
            .map_or(0, |m| m.get(key).copied().unwrap_or(0))
    }

    fn reset_timeout_streak(&self, key: &str) {
        if let Ok(mut m) = self.timeout_streak.lock() {
            m.remove(key);
        }
    }

    pub(super) fn remove_pending(&self, id: i64) {
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

/// Mata o processo de um handle (o `Child` vive num `Mutex` porque a thread
/// do handshake tambem pode mata-lo).
fn kill_child(handle: &super::server::ServerHandle) {
    if let Ok(mut child) = handle.child.lock() {
        drop(child.kill());
        drop(child.wait());
    }
}

/// A resposta de um request LSP que ainda nao chegou.
///
/// `wait` bloqueia ate' `REQUEST_TIMEOUT`; e' o que o laco fazia inline ate'
/// 2026-09-18 e o que agora acontece numa thread (`Core::defer_lsp`). O
/// timeout entra na contagem compartilhada; a resposta zera a contagem.
#[derive(Debug)]
pub struct LspReply {
    key: &'static str,
    method: &'static str,
    id: i64,
    rx: mpsc::Receiver<Value>,
    pending: PendingResponses,
    streak: Arc<Mutex<HashMap<&'static str, u32>>>,
}

impl LspReply {
    /// O metodo LSP pedido (para a mensagem de erro).
    #[must_use]
    pub const fn method(&self) -> &'static str {
        self.method
    }

    /// Espera a resposta (ate' `REQUEST_TIMEOUT`).
    ///
    /// # Errors
    /// `Timeout` (contado para o auto-restart), transporte fechado, ou o
    /// erro que o servidor devolveu.
    pub fn wait(self) -> Result<Value, LspError> {
        match self.rx.recv_timeout(REQUEST_TIMEOUT) {
            Ok(response) => {
                if let Ok(mut m) = self.streak.lock() {
                    m.remove(self.key);
                }
                response_result(self.method, &response)
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if let Ok(mut pending) = self.pending.lock() {
                    pending.remove(&self.id);
                }
                if let Ok(mut m) = self.streak.lock() {
                    *m.entry(self.key).or_insert(0) += 1;
                }
                Err(LspError::Timeout {
                    method: self.method,
                })
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => Err(LspError::Transport {
                message: format!("canal de resposta de {} foi fechado", self.method),
            }),
        }
    }
}

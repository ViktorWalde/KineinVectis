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

use std::{path::PathBuf, sync::Arc, sync::mpsc, time::Duration};

use kinein_protocol::JsonRpcRequest;
use serde_json::{Value, json};

use super::framing::write_locked_message;
use super::manager::LspManager;
use super::parse::response_result;
use super::server::{ServerHandle, ServerSpec, spawn_server};
use super::types::LspError;

/// Tempo maximo aguardando respostas interativas do LSP.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(4);

/// Timeouts CONSECUTIVOS de um servidor antes do auto-restart (M4.3b). Com
/// `REQUEST_TIMEOUT` de 4s, sao ~12s preso antes de ressuscitar o servidor.
const MAX_TIMEOUT_STREAK: u32 = 3;

impl LspManager {
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

    /// Ha' um servidor vivo para a linguagem?
    #[must_use]
    pub fn is_running(&self, language: &str) -> bool {
        self.servers.contains_key(language)
    }

    pub(super) fn ensure_server(&mut self, spec: &ServerSpec) -> Result<(), LspError> {
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

    pub(super) fn send_request(
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

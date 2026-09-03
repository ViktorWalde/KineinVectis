//! O TRANSPORTE do DAP: uma requisicao sai, uma resposta volta.
//!
//! Pasta desde 2026-09-03 (etapa 15 do `roadmaps/34`). O corpo DAP nao e
//! JSON-RPC (`{ seq, type, command/event }`), mas o envelope `Content-Length`
//! e o mesmo do LSP — o framing e reutilizado de `lsp::framing`. Respostas
//! voltam pelo mapa `pending` (seq -> canal), preenchido pela thread leitora.
//!
//! Este arquivo nao sabe o que e um breakpoint nem um frame: ele so' leva e
//! traz. Quem interpreta e' o `parse`, quem orquestra e' a `session`.

use std::{
    collections::HashMap,
    process::ChildStdin,
    sync::{
        Arc, Mutex,
        atomic::{AtomicI64, Ordering},
        mpsc,
    },
    time::Duration,
};

use serde_json::{Value, json};

use super::DebugError;
use crate::lsp::framing::write_message;

/// Requests DAP aguardando resposta, compartilhados com a thread leitora.
pub(super) type Pending = Arc<Mutex<HashMap<i64, mpsc::Sender<Value>>>>;

/// qualquer thread (handler, leitora, enriquecimento).
#[derive(Debug, Clone)]
pub(super) struct Wire {
    stdin: Arc<Mutex<ChildStdin>>,
    pub(super) pending: Pending,
    seq: Arc<AtomicI64>,
}

impl Wire {
    /// Monta o transporte sobre o stdin do adapter recem-nascido.
    pub(super) fn new(stdin: ChildStdin) -> Self {
        Self {
            stdin: Arc::new(Mutex::new(stdin)),
            pending: Arc::new(Mutex::new(HashMap::new())),
            seq: Arc::new(AtomicI64::new(1)),
        }
    }

    /// Envia um request e devolve `(seq, receiver)` sem esperar a resposta.
    pub(super) fn send_request(
        &self,
        command: &str,
        arguments: &Value,
    ) -> Result<(i64, mpsc::Receiver<Value>), DebugError> {
        let seq = self.seq.fetch_add(1, Ordering::SeqCst);
        let (sender, receiver) = mpsc::channel();
        if let Ok(mut guard) = self.pending.lock() {
            guard.insert(seq, sender);
        }
        let message = json!({
            "seq": seq,
            "type": "request",
            "command": command,
            "arguments": arguments,
        });
        let written = match self.stdin.lock() {
            Ok(mut guard) => write_message(&mut *guard, &message).is_ok(),
            Err(_poisoned) => false,
        };
        if !written {
            self.forget(seq);
            return Err(DebugError::Adapter {
                message: format!("falha ao enviar `{command}` ao adapter"),
            });
        }
        Ok((seq, receiver))
    }

    /// Aguarda a resposta de `seq`; sucesso devolve o `body` do adapter.
    pub(super) fn wait_response(
        &self,
        command: &str,
        seq: i64,
        receiver: &mpsc::Receiver<Value>,
        timeout: Duration,
    ) -> Result<Value, DebugError> {
        let Ok(response) = receiver.recv_timeout(timeout) else {
            self.forget(seq);
            return Err(DebugError::Adapter {
                message: format!("o adapter nao respondeu a `{command}`"),
            });
        };
        if response.get("success").and_then(Value::as_bool) == Some(true) {
            return Ok(response.get("body").cloned().unwrap_or(Value::Null));
        }
        let detail = response
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("falha no adapter");
        Err(DebugError::Adapter {
            message: format!("`{command}` falhou: {detail}"),
        })
    }

    /// Request sincrono completo (envia e espera com timeout).
    pub(super) fn request(
        &self,
        command: &str,
        arguments: &Value,
        timeout: Duration,
    ) -> Result<Value, DebugError> {
        let (seq, receiver) = self.send_request(command, arguments)?;
        self.wait_response(command, seq, &receiver, timeout)
    }

    /// Descarta um request pendente (timeout/falha de escrita).
    fn forget(&self, seq: i64) {
        if let Ok(mut guard) = self.pending.lock() {
            guard.remove(&seq);
        }
    }
}

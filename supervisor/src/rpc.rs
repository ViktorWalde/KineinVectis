//! O canal msgpack-RPC do core com o `nvim`, pelo stdio do processo filho.
//!
//! O leitor roda numa thread própria e entrega cada mensagem ao `Handler`. Um
//! pedido do supervisor espera a resposta numa fila própria; quem espera nunca
//! é a thread leitora, senão ela esperaria por si mesma.

use rmpv::Value;
use std::collections::HashMap;
use std::io::{BufReader, BufWriter, Read, Write};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub enum Incoming {
    Notification { method: String, params: Vec<Value> },
    Request { id: u64, method: String, params: Vec<Value> },
}

type Reply = Result<Value, Value>;

pub struct Rpc {
    writer: Mutex<Box<dyn Write + Send>>,
    pending: Mutex<HashMap<u64, mpsc::Sender<Reply>>>,
    next_id: AtomicU64,
    closed: AtomicBool,
    pub protocol_errors: AtomicU64,
}

impl Rpc {
    pub fn new(writer: Box<dyn Write + Send>) -> Arc<Self> {
        Arc::new(Self {
            writer: Mutex::new(Box::new(BufWriter::new(writer))),
            pending: Mutex::new(HashMap::new()),
            next_id: AtomicU64::new(1),
            closed: AtomicBool::new(false),
            protocol_errors: AtomicU64::new(0),
        })
    }

    fn send(&self, message: Value) -> Result<(), String> {
        let mut w = self.writer.lock().unwrap();
        rmpv::encode::write_value(&mut *w, &message).map_err(|e| e.to_string())?;
        w.flush().map_err(|e| e.to_string())
    }

    pub fn request(&self, method: &str, params: Vec<Value>, timeout: Duration) -> Result<Value, String> {
        if self.closed.load(Ordering::Acquire) {
            return Err("canal fechado".into());
        }
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let (tx, rx) = mpsc::channel();
        self.pending.lock().unwrap().insert(id, tx);
        self.send(Value::Array(vec![0.into(), id.into(), method.into(), Value::Array(params)]))?;
        match rx.recv_timeout(timeout) {
            Ok(Ok(v)) => Ok(v),
            Ok(Err(e)) => Err(format!("{method}: {e}")),
            Err(RecvTimeoutError::Timeout) => {
                self.pending.lock().unwrap().remove(&id);
                Err(format!("{method}: sem resposta em {timeout:?}"))
            }
            Err(RecvTimeoutError::Disconnected) => Err(format!("{method}: canal fechado")),
        }
    }

    pub fn respond(&self, id: u64, result: Result<Value, String>) {
        let (err, res) = match result {
            Ok(v) => (Value::Nil, v),
            Err(e) => (Value::from(e), Value::Nil),
        };
        let _ = self.send(Value::Array(vec![1.into(), id.into(), err, res]));
    }

    /// Lê até o fim do stream; cada mensagem vai ao `handler` na ordem em que
    /// chegou. Quando o canal fecha, quem esperava resposta recebe erro.
    pub fn read_loop(self: &Arc<Self>, reader: impl Read, mut handler: impl FnMut(Incoming)) {
        let mut reader = BufReader::with_capacity(1 << 16, reader);
        loop {
            let message = match rmpv::decode::read_value(&mut reader) {
                Ok(v) => v,
                Err(rmpv::decode::Error::InvalidMarkerRead(e)) | Err(rmpv::decode::Error::InvalidDataRead(e))
                    if e.kind() == std::io::ErrorKind::UnexpectedEof =>
                {
                    break;
                }
                Err(_) => {
                    self.protocol_errors.fetch_add(1, Ordering::Relaxed);
                    break;
                }
            };
            let Value::Array(items) = message else {
                self.protocol_errors.fetch_add(1, Ordering::Relaxed);
                continue;
            };
            match items.first().and_then(Value::as_u64) {
                Some(0) if items.len() == 4 => {
                    let id = items[1].as_u64().unwrap_or(0);
                    let method = items[2].as_str().unwrap_or_default().to_owned();
                    let params = items[3].as_array().cloned().unwrap_or_default();
                    handler(Incoming::Request { id, method, params });
                }
                Some(1) if items.len() == 4 => {
                    let id = items[1].as_u64().unwrap_or(0);
                    let reply = if items[2].is_nil() { Ok(items[3].clone()) } else { Err(items[2].clone()) };
                    if let Some(tx) = self.pending.lock().unwrap().remove(&id) {
                        let _ = tx.send(reply);
                    }
                }
                Some(2) if items.len() == 3 => {
                    let method = items[1].as_str().unwrap_or_default().to_owned();
                    let params = items[2].as_array().cloned().unwrap_or_default();
                    handler(Incoming::Notification { method, params });
                }
                _ => {
                    self.protocol_errors.fetch_add(1, Ordering::Relaxed);
                }
            }
        }
        self.closed.store(true, Ordering::Release);
        self.pending.lock().unwrap().clear();
    }
}

/// O número de um handle do Neovim (Buffer, Window): ext com um inteiro
/// msgpack dentro, ou o inteiro direto.
pub fn handle(v: &Value) -> Option<i64> {
    match v {
        Value::Ext(_, data) => rmpv::decode::read_value(&mut &data[..]).ok()?.as_i64(),
        other => other.as_i64(),
    }
}

pub fn to_json(v: &Value) -> serde_json::Value {
    use serde_json::Value as J;
    match v {
        Value::Nil => J::Null,
        Value::Boolean(b) => J::Bool(*b),
        Value::Integer(i) => {
            if let Some(n) = i.as_i64() {
                J::from(n)
            } else {
                J::from(i.as_u64().unwrap_or(0))
            }
        }
        Value::F32(f) => J::from(*f as f64),
        Value::F64(f) => J::from(*f),
        Value::String(s) => J::String(s.as_str().map(str::to_owned).unwrap_or_else(|| String::from_utf8_lossy(s.as_bytes()).into_owned())),
        Value::Binary(b) => J::String(String::from_utf8_lossy(b).into_owned()),
        Value::Array(a) => J::Array(a.iter().map(to_json).collect()),
        Value::Map(m) => J::Object(
            m.iter()
                .map(|(k, v)| (k.as_str().map(str::to_owned).unwrap_or_else(|| k.to_string()), to_json(v)))
                .collect(),
        ),
        Value::Ext(_, _) => handle(v).map(J::from).unwrap_or(J::Null),
    }
}

pub fn from_json(v: &serde_json::Value) -> Value {
    use serde_json::Value as J;
    match v {
        J::Null => Value::Nil,
        J::Bool(b) => Value::Boolean(*b),
        J::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::from(i)
            } else if let Some(u) = n.as_u64() {
                Value::from(u)
            } else {
                Value::F64(n.as_f64().unwrap_or(0.0))
            }
        }
        J::String(s) => Value::from(s.as_str()),
        J::Array(a) => Value::Array(a.iter().map(from_json).collect()),
        J::Object(o) => Value::Map(o.iter().map(|(k, v)| (Value::from(k.as_str()), from_json(v))).collect()),
    }
}

/// Uma string de um Value, também quando o Neovim manda bytes que não são UTF-8.
pub fn text(v: &Value) -> String {
    match v {
        Value::String(s) => s.as_str().map(str::to_owned).unwrap_or_else(|| String::from_utf8_lossy(s.as_bytes()).into_owned()),
        Value::Binary(b) => String::from_utf8_lossy(b).into_owned(),
        other => other.to_string(),
    }
}

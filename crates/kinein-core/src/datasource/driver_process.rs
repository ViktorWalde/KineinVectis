//! Ponte com o processo de um adaptador de banco (D1b.2, `DocsPublic/arquitetura/39` §6.1).
//!
//! O contrato puro (handshake, erros, `StreamGuard`) ja' existe; aqui ele ganha
//! um processo de verdade, sobre a base [`crate::owned_child`]. A ponte so'
//! TRANSPORTA: limita cada linha antes do parse, faz o handshake com prazo,
//! respeita fila e slots, roteia resposta por id e chunk por `operationId`, e,
//! quando o transporte morre, termina cada pedido com o desfecho que se sabe.
//! Validar o conteudo de chunk e terminal continua com o `StreamGuard` de quem
//! pediu; nada aqui repete uma operacao.

use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Read, Write},
    process::{ChildStdout, Command},
    sync::{
        Arc, Condvar, Mutex, MutexGuard,
        atomic::{AtomicUsize, Ordering},
        mpsc::{self, Receiver, RecvTimeoutError, Sender},
    },
    time::Duration,
};

use kinein_protocol::JsonRpcRequest;
use kinein_protocol::driver::{Error, OperationOutcome, deserialize_object};
use serde::Deserialize;
use serde_json::{Value, value::RawValue};

use super::driver_contract::{
    self, HandshakeFailure, INITIALIZE_TIMEOUT_MS, LIMITS, Negotiated, QUEUE_CAPACITY,
    SHUTDOWN_TIMEOUT_MS,
};
use crate::owned_child::{Closed, Env, OwnedChild};

/// Espera pelo EOF de um adaptador que falhou no handshake, antes do TERM.
const FAILED_START_GRACE: Duration = Duration::from_millis(200);

/// O que chega para um pedido, na ordem do fio.
#[derive(Debug)]
pub enum Event {
    /// `driver.chunk` desta operacao: `{ context, sequence, payload }` cru.
    Chunk(Box<RawValue>),
    /// O `result` do terminal, cru, para o `StreamGuard` de quem pediu.
    Terminal(Box<RawValue>),
    /// Erro numerico do adaptador; o mapeamento publico e' do `driver_contract`.
    Failed(Error),
    /// O transporte morreu antes do terminal, com o desfecho que se sabe.
    Lost(OperationOutcome),
}

/// Um pedido enviado; os eventos dele chegam por `events`.
#[derive(Debug)]
pub struct Pending {
    /// O id JSON-RPC do pedido.
    pub id: u64,
    /// Chunks e o fim do pedido.
    pub events: Receiver<Event>,
}

/// Por que o adaptador nao ficou pronto.
#[derive(Debug)]
pub enum StartFailure {
    /// O executavel nao subiu.
    Spawn(std::io::Error),
    /// O handshake recusou, com o motivo fixo do contrato.
    Refused(HandshakeFailure),
    /// O `initialize` nao respondeu no prazo.
    Timeout,
    /// O processo morreu antes de responder.
    Exited,
}

/// Por que um pedido nao foi enviado. Nada dele chegou ao adaptador.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SendRefusal {
    /// A fila local esta' cheia.
    Busy,
    /// O transporte ja' morreu ou a instancia esta' encerrando.
    Closed,
}

/// Tipo do pedido para fila e desfecho.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Kind {
    /// Pedido comum que pode escrever (`query`, `preview`, `decide`).
    Write,
    /// Pedido comum que so' le (`test`, `introspect`, `impact`, `open`).
    Read,
    /// Decisao, cancelamento, fechamento e encerramento: usam o slot reservado.
    Control,
}

/// Um adaptador vivo, com o contrato negociado.
#[derive(Debug)]
pub struct DriverProcess {
    child: Mutex<OwnedChild>,
    shared: Arc<Shared>,
    negotiated: Negotiated,
}

#[derive(Debug)]
struct Entry {
    target: Target,
    operation_id: Option<String>,
    kind: Kind,
    written: bool,
}

#[derive(Debug)]
enum Target {
    Handshake(Sender<Vec<u8>>),
    Operation(Sender<Event>),
}

#[derive(Debug, Default)]
struct State {
    pending: HashMap<u64, Entry>,
    next_id: u64,
    ordinary: usize,
    control: usize,
    waiting: usize,
    in_flight: usize,
    dead: bool,
}

#[derive(Debug)]
struct Shared {
    state: Mutex<State>,
    slots: Condvar,
    message_bytes: AtomicUsize,
}

impl Shared {
    fn lock(&self) -> MutexGuard<'_, State> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// O transporte morreu: cada pedido termina com o desfecho que se sabe.
    fn lose_all(&self) {
        let mut state = self.lock();
        state.dead = true;
        for (_, entry) in state.pending.drain() {
            let outcome = match (entry.written, entry.kind) {
                (false, _) => OperationOutcome::NotStarted,
                (true, Kind::Write) => OperationOutcome::Unknown,
                (true, Kind::Read | Kind::Control) => OperationOutcome::Failed,
            };
            if let Target::Operation(events) = entry.target {
                drop(events.send(Event::Lost(outcome)));
            }
        }
        state.ordinary = 0;
        state.control = 0;
        drop(state);
        self.slots.notify_all();
    }

    fn finish(&self, id: u64) -> Option<Entry> {
        let mut state = self.lock();
        let entry = state.pending.remove(&id)?;
        match entry.kind {
            Kind::Control => state.control = state.control.saturating_sub(1),
            Kind::Read | Kind::Write => state.ordinary = state.ordinary.saturating_sub(1),
        }
        drop(state);
        self.slots.notify_all();
        Some(entry)
    }
}

impl DriverProcess {
    /// Sobe o adaptador e faz o handshake com prazo, sem segredo.
    ///
    /// # Errors
    /// [`StartFailure`]; o processo e' encerrado e colhido em todos os casos.
    pub fn start(
        command: Command,
        env: &[(&str, &str)],
        adapter_id: &str,
        engine: &str,
    ) -> Result<Self, StartFailure> {
        let shared = Arc::new(Shared {
            state: Mutex::new(State::default()),
            slots: Condvar::new(),
            message_bytes: AtomicUsize::new(LIMITS.message_bytes as usize),
        });
        let reader_shared = Arc::clone(&shared);
        let child = OwnedChild::spawn(command, Env::Allowlist(env), None, move |stdout| {
            read_loop(stdout, &reader_shared);
        })
        .map_err(StartFailure::Spawn)?;
        let child = Mutex::new(child);
        let (sender, receiver) = mpsc::channel();
        {
            let mut state = shared.lock();
            state.pending.insert(
                0,
                Entry {
                    target: Target::Handshake(sender),
                    operation_id: None,
                    kind: Kind::Control,
                    written: true,
                },
            );
            state.next_id = 1;
        }
        let request = driver_contract::initialize_request(0);
        let written = write_line(&child, &request);
        let reply = if written {
            receiver.recv_timeout(Duration::from_millis(INITIALIZE_TIMEOUT_MS))
        } else {
            Err(RecvTimeoutError::Disconnected)
        };
        // Um adaptador que falhou no handshake nao merece a cortesia do prazo
        // inteiro: EOF, uma espera curta e o TERM.
        let close =
            |child: Mutex<OwnedChild>| into_inner(child).close(FAILED_START_GRACE).collected();
        let line = match reply {
            Ok(line) => line,
            Err(RecvTimeoutError::Timeout) => {
                close(child);
                return Err(StartFailure::Timeout);
            }
            Err(RecvTimeoutError::Disconnected) => {
                close(child);
                return Err(StartFailure::Exited);
            }
        };
        match driver_contract::accept_initialize(&line, 0, adapter_id, engine) {
            Ok(negotiated) => {
                shared
                    .message_bytes
                    .store(negotiated.limits.message_bytes as usize, Ordering::Relaxed);
                shared.lock().in_flight = usize::from(negotiated.limits.in_flight);
                Ok(Self {
                    child,
                    shared,
                    negotiated,
                })
            }
            Err(failure) => {
                close(child);
                Err(StartFailure::Refused(failure))
            }
        }
    }

    /// O contrato negociado com este adaptador.
    #[must_use]
    pub const fn negotiated(&self) -> &Negotiated {
        &self.negotiated
    }

    /// Envia um pedido. Espera um slot quando ha' fila; `Control` usa o
    /// reservado, que nenhum pedido comum ocupa.
    ///
    /// # Errors
    /// [`SendRefusal`] antes de qualquer byte chegar ao adaptador.
    pub fn send(
        &self,
        method: &str,
        params: Value,
        operation_id: &str,
        kind: Kind,
    ) -> Result<Pending, SendRefusal> {
        let mut state = self.shared.lock();
        if state.dead {
            return Err(SendRefusal::Closed);
        }
        if kind != Kind::Control && state.waiting >= QUEUE_CAPACITY {
            return Err(SendRefusal::Busy);
        }
        state.waiting += 1;
        while !state.dead && !has_slot(&state, kind) {
            state = self
                .shared
                .slots
                .wait(state)
                .unwrap_or_else(std::sync::PoisonError::into_inner);
        }
        state.waiting -= 1;
        if state.dead {
            return Err(SendRefusal::Closed);
        }
        let id = state.next_id;
        state.next_id += 1;
        match kind {
            Kind::Control => state.control += 1,
            Kind::Read | Kind::Write => state.ordinary += 1,
        }
        let (sender, events) = mpsc::channel();
        // Marcado como escrito ANTES de escrever: uma linha pela metade pode
        // ter chegado, e uma escrita nesse estado tem desfecho desconhecido.
        state.pending.insert(
            id,
            Entry {
                target: Target::Operation(sender),
                operation_id: Some(operation_id.to_owned()),
                kind,
                written: true,
            },
        );
        drop(state);
        if !write_line(&self.child, &JsonRpcRequest::new(id, method, Some(params))) {
            self.shared.lose_all();
        }
        Ok(Pending { id, events })
    }

    /// Encerra: `driver.shutdown` pelo slot de controle, depois o encerramento
    /// da base, tudo com prazo. `confirmed` diz se o adaptador confirmou.
    #[must_use]
    pub fn shutdown(self, context: &Value) -> (bool, Closed) {
        let confirmed = self
            .send(
                "driver.shutdown",
                serde_json::json!({ "context": context }),
                "shutdown",
                Kind::Control,
            )
            .is_ok_and(|pending| {
                matches!(
                    pending
                        .events
                        .recv_timeout(Duration::from_millis(SHUTDOWN_TIMEOUT_MS)),
                    Ok(Event::Terminal(_))
                )
            });
        let closed = into_inner(self.child).close(Duration::from_millis(SHUTDOWN_TIMEOUT_MS));
        (confirmed, closed)
    }
}

fn into_inner(child: Mutex<OwnedChild>) -> OwnedChild {
    child
        .into_inner()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

const fn has_slot(state: &State, kind: Kind) -> bool {
    let total = state.ordinary + state.control;
    match kind {
        Kind::Control => total < state.in_flight,
        Kind::Read | Kind::Write => state.ordinary + 1 < state.in_flight && total < state.in_flight,
    }
}

fn write_line(child: &Mutex<OwnedChild>, request: &JsonRpcRequest) -> bool {
    let Ok(mut line) = serde_json::to_vec(request) else {
        return false;
    };
    line.push(b'\n');
    let mut child = child
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    child
        .stdin()
        .is_some_and(|stdin| stdin.write_all(&line).and_then(|()| stdin.flush()).is_ok())
}

/// Envelope minimo para rotear; o conteudo e' do dono de cada mensagem.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    jsonrpc: String,
    #[serde(default)]
    id: Option<u64>,
    #[serde(default)]
    method: Option<String>,
    #[serde(default)]
    params: Option<Box<RawValue>>,
    #[serde(default)]
    result: Option<Box<RawValue>>,
    #[serde(default)]
    error: Option<Box<RawValue>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ChunkRoute {
    context: RouteContext,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RouteContext {
    operation_id: String,
}

/// Le linhas com teto antes do parse; violacao ou EOF matam o transporte.
fn read_loop(stdout: ChildStdout, shared: &Shared) {
    let mut reader = BufReader::new(stdout);
    let ceiling = LIMITS.message_bytes as usize;
    loop {
        let mut line = Vec::new();
        // Le ate' o teto LOCAL (+2: terminador e um byte que denuncia o
        // excesso) e confere o NEGOCIADO depois: carregar o limite antes da
        // leitura deixava a primeira linha pos-handshake passar com o teto
        // antigo (achado pelo teste do modo big_line, 2026-10-08).
        let read = (&mut reader)
            .take(ceiling as u64 + 2)
            .read_until(b'\n', &mut line);
        match read {
            Ok(0) | Err(_) => break,
            Ok(_) => {}
        }
        let limit = shared.message_bytes.load(Ordering::Relaxed);
        let body = line.strip_suffix(b"\n").unwrap_or(&line);
        let body = body.strip_suffix(b"\r").unwrap_or(body);
        if body.len() > limit || !line.ends_with(b"\n") || !route(body, &line, shared) {
            break;
        }
    }
    shared.lose_all();
}

fn route(body: &[u8], line: &[u8], shared: &Shared) -> bool {
    let mut decoder = serde_json::Deserializer::from_slice(body);
    let Ok(envelope) = deserialize_object::<Envelope, _>(&mut decoder) else {
        return false;
    };
    if envelope.jsonrpc != "2.0" || decoder.end().is_err() {
        return false;
    }
    match envelope {
        Envelope {
            method: Some(method),
            id: None,
            params: Some(params),
            result: None,
            error: None,
            ..
        } if method == "driver.chunk" => {
            let Ok(route) = serde_json::from_str::<ChunkRoute>(params.get()) else {
                return false;
            };
            let state = shared.lock();
            let target = state
                .pending
                .values()
                .find_map(|entry| match &entry.target {
                    Target::Operation(events)
                        if entry.operation_id.as_deref() == Some(&route.context.operation_id) =>
                    {
                        Some(events.clone())
                    }
                    _ => None,
                });
            drop(state);
            target.is_some_and(|events| {
                drop(events.send(Event::Chunk(params)));
                true
            })
        }
        Envelope {
            method: None,
            id: Some(id),
            params: None,
            result,
            error,
            ..
        } if result.is_some() != error.is_some() => {
            let Some(entry) = shared.finish(id) else {
                return false;
            };
            match (entry.target, result, error) {
                (Target::Handshake(sender), _, _) => {
                    drop(sender.send(line.to_vec()));
                    true
                }
                (Target::Operation(events), Some(result), None) => {
                    drop(events.send(Event::Terminal(result)));
                    true
                }
                (Target::Operation(events), None, Some(error)) => {
                    serde_json::from_str::<Error>(error.get()).is_ok_and(|error| {
                        drop(events.send(Event::Failed(error)));
                        true
                    })
                }
                _ => false,
            }
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests;

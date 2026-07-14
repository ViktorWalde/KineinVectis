//! Sessao DAP viva: spawn do lldb-dap, handshake, thread leitora e requests.
//!
//! O corpo DAP nao e JSON-RPC (`{ seq, type, command/event }`), mas o
//! envelope `Content-Length` e o mesmo do LSP — o framing e reutilizado de
//! `lsp::framing`. Respostas voltam pelo mapa `pending` (seq -> canal);
//! eventos do adapter viram notificacoes `event.debug.*` ja mastigadas para
//! a UI. A thread leitora nunca espera resposta de request (e ela quem as
//! entrega): o enriquecimento do `stopped` (stackTrace -> file/line) roda em
//! thread propria.

use std::{
    collections::{BTreeMap, HashMap},
    io::{self, BufReader},
    path::Path,
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicI64, Ordering},
        mpsc,
    },
    thread,
    time::Duration,
};

use kinein_protocol::{BreakpointInfo, JsonRpcRequest, StackFrameInfo, VariableInfo};
use serde_json::{Value, json};

use super::DebugError;
use crate::lsp::EventSender;
use crate::lsp::framing::{read_message, write_message};

/// Binario do debug adapter (pacote `lldb` no Arch).
pub(super) const ADAPTER_BINARY: &str = "lldb-dap";

/// Tempo maximo aguardando respostas comuns do adapter.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

/// Tempo maximo aguardando a resposta de `launch` (carregar o alvo demora
/// mais que um request comum em binarios grandes).
const LAUNCH_TIMEOUT: Duration = Duration::from_secs(20);

/// Frames maximos devolvidos por `debug.stackTrace` (JetBrains-like; a
/// paginacao so entra se o uso real pedir).
const STACK_TRACE_LEVELS: u32 = 20;

/// Tempo maximo do `disconnect` no encerramento; depois o processo e morto.
const DISCONNECT_TIMEOUT: Duration = Duration::from_secs(5);

/// Requests DAP aguardando resposta, compartilhados com a thread leitora.
type Pending = Arc<Mutex<HashMap<i64, mpsc::Sender<Value>>>>;

/// Lado de escrita do adapter: tudo que um request precisa, clonavel para
/// qualquer thread (handler, leitora, enriquecimento).
#[derive(Debug, Clone)]
struct Wire {
    stdin: Arc<Mutex<ChildStdin>>,
    pending: Pending,
    seq: Arc<AtomicI64>,
}

impl Wire {
    /// Envia um request e devolve `(seq, receiver)` sem esperar a resposta.
    fn send_request(
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
    fn wait_response(
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
    fn request(
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

/// Uma sessao de debug viva (um adapter, um processo alvo).
#[derive(Debug)]
pub(super) struct DapSession {
    child: Child,
    wire: Wire,
    events: EventSender,
    alive: Arc<AtomicBool>,
    stopped_thread: Arc<Mutex<Option<i64>>>,
}

impl Drop for DapSession {
    fn drop(&mut self) {
        drop(self.child.kill());
        drop(self.child.wait());
    }
}

impl DapSession {
    /// Sobe o adapter, faz a danca DAP (initialize -> launch -> initialized
    /// -> replay de breakpoints -> configurationDone) e emite
    /// `event.debug.started`.
    pub(super) fn launch(
        root: &Path,
        program: &Path,
        breakpoints: &BTreeMap<String, Vec<u32>>,
        events: EventSender,
    ) -> Result<Self, DebugError> {
        let mut child = Command::new(ADAPTER_BINARY)
            .current_dir(root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|error| {
                if error.kind() == io::ErrorKind::NotFound {
                    DebugError::MissingAdapter
                } else {
                    DebugError::Adapter {
                        message: format!("falha ao iniciar {ADAPTER_BINARY}: {error}"),
                    }
                }
            })?;
        let (Some(stdin), Some(stdout)) = (child.stdin.take(), child.stdout.take()) else {
            drop(child.kill());
            return Err(DebugError::Adapter {
                message: format!("{ADAPTER_BINARY} subiu sem stdin/stdout utilizaveis"),
            });
        };

        let wire = Wire {
            stdin: Arc::new(Mutex::new(stdin)),
            pending: Arc::new(Mutex::new(HashMap::new())),
            seq: Arc::new(AtomicI64::new(1)),
        };
        let alive = Arc::new(AtomicBool::new(true));
        let stopped_thread = Arc::new(Mutex::new(None));
        let (initialized_sender, initialized_receiver) = mpsc::channel();

        spawn_reader(
            stdout,
            &wire,
            &events,
            &alive,
            &stopped_thread,
            initialized_sender,
        );

        let session = Self {
            child,
            wire,
            events,
            alive,
            stopped_thread,
        };
        // Drop mata o adapter se qualquer passo do handshake falhar.
        session.handshake(root, program, breakpoints, &initialized_receiver)?;
        send_event(
            &session.events,
            "event.debug.started",
            json!({ "program": program.display().to_string() }),
        );
        Ok(session)
    }

    /// `true` enquanto o adapter esta vivo (nem `terminated` nem EOF).
    pub(super) fn is_alive(&self) -> bool {
        self.alive.load(Ordering::SeqCst)
    }

    /// Substitui os breakpoints de `file` no adapter e devolve o resultado.
    pub(super) fn set_breakpoints(
        &self,
        file: &str,
        lines: &[u32],
    ) -> Result<Vec<BreakpointInfo>, DebugError> {
        let body = self.wire.request(
            "setBreakpoints",
            &breakpoints_arguments(file, lines),
            REQUEST_TIMEOUT,
        )?;
        Ok(parse_breakpoints(&body, lines))
    }

    /// Retoma a execucao a partir da thread pausada.
    pub(super) fn continue_run(&self) -> Result<(), DebugError> {
        let thread = self.stopped_thread_id()?;
        self.wire
            .request("continue", &json!({ "threadId": thread }), REQUEST_TIMEOUT)?;
        note_continued(&self.stopped_thread, &self.events);
        Ok(())
    }

    /// Executa um passo (`next`, `stepIn` ou `stepOut`) na thread pausada.
    pub(super) fn step(&self, command: &str) -> Result<(), DebugError> {
        let thread = self.stopped_thread_id()?;
        self.wire
            .request(command, &json!({ "threadId": thread }), REQUEST_TIMEOUT)?;
        note_continued(&self.stopped_thread, &self.events);
        Ok(())
    }

    /// Pausa o processo em execucao (primeira thread listada pelo adapter).
    pub(super) fn pause(&self) -> Result<(), DebugError> {
        let body = self.wire.request("threads", &json!({}), REQUEST_TIMEOUT)?;
        let thread = body
            .get("threads")
            .and_then(Value::as_array)
            .and_then(|threads| threads.first())
            .and_then(|thread| thread.get("id"))
            .and_then(Value::as_i64)
            .ok_or_else(|| DebugError::Adapter {
                message: "o adapter nao listou threads para pausar".to_owned(),
            })?;
        self.wire
            .request("pause", &json!({ "threadId": thread }), REQUEST_TIMEOUT)?;
        Ok(())
    }

    /// Frames da thread pausada, do topo para baixo.
    pub(super) fn stack_trace(&self) -> Result<Vec<StackFrameInfo>, DebugError> {
        let thread = self.stopped_thread_id()?;
        let body = self.wire.request(
            "stackTrace",
            &json!({
                "threadId": thread,
                "startFrame": 0,
                "levels": STACK_TRACE_LEVELS,
            }),
            REQUEST_TIMEOUT,
        )?;
        Ok(parse_stack_frames(&body))
    }

    /// Variaveis do primeiro escopo nao-caro do frame (Locals no lldb-dap).
    ///
    /// A UI nao conhece "scopes": o core resolve `scopes(frameId)` aqui e
    /// devolve direto as variaveis. Globals/Registers ficam pos-M2.
    pub(super) fn frame_variables(&self, frame_id: i64) -> Result<Vec<VariableInfo>, DebugError> {
        let _ = self.stopped_thread_id()?;
        let scopes =
            self.wire
                .request("scopes", &json!({ "frameId": frame_id }), REQUEST_TIMEOUT)?;
        let Some(reference) = first_cheap_scope_reference(&scopes) else {
            return Ok(Vec::new());
        };
        self.reference_variables(reference)
    }

    /// Expande uma variavel estruturada (handle `ref` de resposta anterior).
    pub(super) fn reference_variables(
        &self,
        reference: i64,
    ) -> Result<Vec<VariableInfo>, DebugError> {
        let _ = self.stopped_thread_id()?;
        let body = self.wire.request(
            "variables",
            &json!({ "variablesReference": reference }),
            REQUEST_TIMEOUT,
        )?;
        Ok(parse_variables(&body))
    }

    /// Pede o encerramento educado; `terminated`/EOF emitem o `finished`.
    pub(super) fn shutdown(&self) {
        drop(self.wire.request(
            "disconnect",
            &json!({ "terminateDebuggee": true }),
            DISCONNECT_TIMEOUT,
        ));
    }

    /// Thread pausada atual, ou o erro que explica que nada esta pausado.
    fn stopped_thread_id(&self) -> Result<i64, DebugError> {
        self.stopped_thread
            .lock()
            .ok()
            .and_then(|guard| *guard)
            .ok_or(DebugError::NotStopped)
    }

    /// A danca de inicializacao do DAP, na ordem que os adapters esperam.
    fn handshake(
        &self,
        root: &Path,
        program: &Path,
        breakpoints: &BTreeMap<String, Vec<u32>>,
        initialized: &mpsc::Receiver<()>,
    ) -> Result<(), DebugError> {
        self.wire.request(
            "initialize",
            &json!({
                "clientID": "kinein-vectis",
                "adapterID": "lldb",
                "linesStartAt1": true,
                "columnsStartAt1": true,
                "pathFormat": "path",
            }),
            REQUEST_TIMEOUT,
        )?;
        // A resposta de launch so chega depois do configurationDone; o
        // request vai agora e a espera fica para o final.
        let (launch_seq, launch_receiver) = self.wire.send_request(
            "launch",
            &json!({
                "program": program.display().to_string(),
                "cwd": root.display().to_string(),
                "stopOnEntry": false,
            }),
        )?;
        if initialized.recv_timeout(REQUEST_TIMEOUT).is_err() {
            return Err(DebugError::Adapter {
                message: "o adapter nao publicou o evento initialized".to_owned(),
            });
        }
        for (file, lines) in breakpoints {
            self.wire.request(
                "setBreakpoints",
                &breakpoints_arguments(file, lines),
                REQUEST_TIMEOUT,
            )?;
        }
        self.wire
            .request("configurationDone", &json!({}), REQUEST_TIMEOUT)?;
        self.wire
            .wait_response("launch", launch_seq, &launch_receiver, LAUNCH_TIMEOUT)?;
        Ok(())
    }
}

/// Monta os `arguments` de `setBreakpoints` para um arquivo.
fn breakpoints_arguments(file: &str, lines: &[u32]) -> Value {
    json!({
        "source": { "path": file },
        "breakpoints": lines
            .iter()
            .map(|line| json!({ "line": line }))
            .collect::<Vec<_>>(),
    })
}

/// Converte o `body` de `setBreakpoints` no resultado do protocolo.
fn parse_breakpoints(body: &Value, requested: &[u32]) -> Vec<BreakpointInfo> {
    let Some(items) = body.get("breakpoints").and_then(Value::as_array) else {
        return requested
            .iter()
            .map(|&line| BreakpointInfo {
                line,
                verified: false,
            })
            .collect();
    };
    items
        .iter()
        .enumerate()
        .map(|(index, item)| BreakpointInfo {
            line: item
                .get("line")
                .and_then(Value::as_u64)
                .and_then(|line| u32::try_from(line).ok())
                .or_else(|| requested.get(index).copied())
                .unwrap_or(0),
            verified: item
                .get("verified")
                .and_then(Value::as_bool)
                .unwrap_or(false),
        })
        .collect()
}

/// Converte o `body` de `stackTrace` nos frames do protocolo.
fn parse_stack_frames(body: &Value) -> Vec<StackFrameInfo> {
    let Some(items) = body.get("stackFrames").and_then(Value::as_array) else {
        return Vec::new();
    };
    items
        .iter()
        .map(|frame| StackFrameInfo {
            id: frame.get("id").and_then(Value::as_i64).unwrap_or(0),
            name: frame
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("?")
                .to_owned(),
            file: frame
                .get("source")
                .and_then(|source| source.get("path"))
                .and_then(Value::as_str)
                .map(str::to_owned),
            line: frame
                .get("line")
                .and_then(Value::as_u64)
                .and_then(|line| u32::try_from(line).ok())
                .filter(|&line| line > 0),
        })
        .collect()
}

/// Acha o `variablesReference` do primeiro escopo nao-caro (Locals).
fn first_cheap_scope_reference(body: &Value) -> Option<i64> {
    body.get("scopes")
        .and_then(Value::as_array)?
        .iter()
        .find(|scope| scope.get("expensive").and_then(Value::as_bool) != Some(true))
        .and_then(|scope| scope.get("variablesReference"))
        .and_then(Value::as_i64)
        .filter(|&reference| reference > 0)
}

/// Converte o `body` de `variables` nas variaveis do protocolo.
fn parse_variables(body: &Value) -> Vec<VariableInfo> {
    let Some(items) = body.get("variables").and_then(Value::as_array) else {
        return Vec::new();
    };
    items
        .iter()
        .map(|variable| VariableInfo {
            name: variable
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("?")
                .to_owned(),
            value: variable
                .get("value")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_owned(),
            type_name: variable
                .get("type")
                .and_then(Value::as_str)
                .map(str::to_owned),
            reference: variable
                .get("variablesReference")
                .and_then(Value::as_i64)
                .unwrap_or(0),
        })
        .collect()
}

/// Sobe a thread leitora do stdout do adapter.
fn spawn_reader(
    stdout: ChildStdout,
    wire: &Wire,
    events: &EventSender,
    alive: &Arc<AtomicBool>,
    stopped_thread: &Arc<Mutex<Option<i64>>>,
    initialized: mpsc::Sender<()>,
) {
    let wire = wire.clone();
    let events = events.clone();
    let alive = Arc::clone(alive);
    let stopped_thread = Arc::clone(stopped_thread);
    thread::spawn(move || {
        let exit_code = Arc::new(Mutex::new(None));
        let mut reader = BufReader::new(stdout);
        while let Ok(Some(message)) = read_message(&mut reader) {
            match message.get("type").and_then(Value::as_str) {
                Some("response") => deliver_response(&wire.pending, &message),
                Some("event") => handle_adapter_event(
                    &message,
                    &wire,
                    &events,
                    &alive,
                    &stopped_thread,
                    &exit_code,
                    &initialized,
                ),
                _ => {}
            }
        }
        // EOF/erro: destrava quem espera resposta e fecha a sessao uma vez.
        if let Ok(mut guard) = wire.pending.lock() {
            guard.clear();
        }
        finish_once(&alive, &events, &exit_code);
    });
}

/// Entrega uma resposta ao request que a aguarda.
fn deliver_response(pending: &Pending, message: &Value) {
    let Some(seq) = message.get("request_seq").and_then(Value::as_i64) else {
        return;
    };
    let sender = match pending.lock() {
        Ok(mut guard) => guard.remove(&seq),
        Err(_poisoned) => None,
    };
    if let Some(sender) = sender {
        drop(sender.send(message.clone()));
    }
}

/// Traduz um evento do adapter em `event.debug.*` da UI.
fn handle_adapter_event(
    message: &Value,
    wire: &Wire,
    events: &EventSender,
    alive: &Arc<AtomicBool>,
    stopped_thread: &Arc<Mutex<Option<i64>>>,
    exit_code: &Arc<Mutex<Option<i64>>>,
    initialized: &mpsc::Sender<()>,
) {
    let name = message
        .get("event")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let body = message.get("body").cloned().unwrap_or(Value::Null);
    match name {
        "initialized" => drop(initialized.send(())),
        "output" => emit_output(events, &body),
        "stopped" => {
            let thread_id = body.get("threadId").and_then(Value::as_i64).unwrap_or(0);
            let reason = body
                .get("reason")
                .and_then(Value::as_str)
                .unwrap_or("stopped")
                .to_owned();
            if let Ok(mut guard) = stopped_thread.lock() {
                *guard = Some(thread_id);
            }
            let wire = wire.clone();
            let events = events.clone();
            // A leitora nao pode esperar o stackTrace que ela mesma entrega.
            thread::spawn(move || emit_enriched_stopped(&wire, &events, thread_id, &reason));
        }
        "continued" => note_continued(stopped_thread, events),
        "exited" => {
            if let Ok(mut guard) = exit_code.lock() {
                *guard = body.get("exitCode").and_then(Value::as_i64);
            }
        }
        "terminated" => finish_once(alive, events, exit_code),
        _ => {}
    }
}

/// Emite `event.debug.output` linha a linha (telemetry e ignorada).
fn emit_output(events: &EventSender, body: &Value) {
    let category = body
        .get("category")
        .and_then(Value::as_str)
        .unwrap_or("console");
    if category == "telemetry" {
        return;
    }
    let Some(output) = body.get("output").and_then(Value::as_str) else {
        return;
    };
    for line in output.lines() {
        send_event(
            events,
            "event.debug.output",
            json!({ "category": category, "line": line }),
        );
    }
}

/// Consulta o frame do topo e emite `event.debug.stopped` com file/line.
fn emit_enriched_stopped(wire: &Wire, events: &EventSender, thread_id: i64, reason: &str) {
    let mut file = Value::Null;
    let mut line = Value::Null;
    if let Ok(body) = wire.request(
        "stackTrace",
        &json!({ "threadId": thread_id, "startFrame": 0, "levels": 1 }),
        REQUEST_TIMEOUT,
    ) {
        if let Some(frame) = body
            .get("stackFrames")
            .and_then(Value::as_array)
            .and_then(|frames| frames.first())
        {
            file = frame
                .get("source")
                .and_then(|source| source.get("path"))
                .cloned()
                .unwrap_or(Value::Null);
            line = frame.get("line").cloned().unwrap_or(Value::Null);
        }
    }
    send_event(
        events,
        "event.debug.stopped",
        json!({
            "reason": reason,
            "file": file,
            "line": line,
            "threadId": thread_id,
        }),
    );
}

/// Emite `event.debug.continued` uma unica vez por retomada.
///
/// Chamada tanto apos um continue/step bem-sucedido quanto no evento
/// `continued` do adapter: o `take()` do estado pausado deduplica.
fn note_continued(stopped_thread: &Arc<Mutex<Option<i64>>>, events: &EventSender) {
    let was_stopped = stopped_thread
        .lock()
        .is_ok_and(|mut guard| guard.take().is_some());
    if was_stopped {
        send_event(events, "event.debug.continued", json!({}));
    }
}

/// Emite `event.debug.finished` exatamente uma vez por sessao.
fn finish_once(alive: &Arc<AtomicBool>, events: &EventSender, exit_code: &Arc<Mutex<Option<i64>>>) {
    if alive.swap(false, Ordering::SeqCst) {
        let code = exit_code.lock().ok().and_then(|guard| *guard);
        send_event(events, "event.debug.finished", json!({ "exitCode": code }));
    }
}

/// Serializa uma notificacao `event.debug.*` no canal assincrono.
fn send_event(events: &EventSender, method: &str, params: Value) {
    drop(events.send(JsonRpcRequest::notification(method, Some(params))));
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{first_cheap_scope_reference, parse_stack_frames, parse_variables};

    #[test]
    fn stack_frames_map_source_and_drop_zero_lines() {
        let frames = parse_stack_frames(&json!({
            "stackFrames": [
                { "id": 4, "name": "soma", "line": 4,
                  "source": { "path": "/w/main.c" } },
                { "id": 5, "name": "??", "line": 0 },
            ]
        }));
        assert_eq!(frames.len(), 2);
        assert_eq!(frames[0].id, 4);
        assert_eq!(frames[0].file.as_deref(), Some("/w/main.c"));
        assert_eq!(frames[0].line, Some(4));
        assert_eq!(frames[1].file, None);
        assert_eq!(frames[1].line, None);
    }

    #[test]
    fn scope_selection_skips_expensive_and_requires_reference() {
        let reference = first_cheap_scope_reference(&json!({
            "scopes": [
                { "name": "Registers", "expensive": true,
                  "variablesReference": 9 },
                { "name": "Locals", "variablesReference": 3 },
            ]
        }));
        assert_eq!(reference, Some(3));
        assert_eq!(first_cheap_scope_reference(&json!({ "scopes": [] })), None);
    }

    #[test]
    fn variables_carry_type_and_expansion_reference() {
        let variables = parse_variables(&json!({
            "variables": [
                { "name": "a", "value": "2", "type": "int",
                  "variablesReference": 0 },
                { "name": "p", "value": "{...}", "variablesReference": 12 },
            ]
        }));
        assert_eq!(variables[0].type_name.as_deref(), Some("int"));
        assert_eq!(variables[0].reference, 0);
        assert_eq!(variables[1].reference, 12);
        assert_eq!(variables[1].type_name, None);
    }
}

//! A SESSAO DAP viva: sobe o adapter, faz o handshake e opera o alvo.
//!
//! Cortado em 2026-09-03 (etapa 15 do `roadmaps/34`), que estava em 672/500.
//! O corte separou o que o arquivo misturava, e cada parte tem dono:
//!
//! ```text
//! dap/wire.rs     o TRANSPORTE — leva e traz, nao interpreta nada
//! dap/parse.rs    INTERPRETAR a resposta (e o unico testavel sem processo)
//! dap/reader.rs   a thread LEITORA e os eventos do adapter
//! dap/adapter.rs  QUAL adaptador sobe, com que argumentos e que pedido
//! dap/server.rs   o processo SERVIDOR do alvo remoto (QEMU, OpenOCD)
//! dap/session.rs  ESTE: ciclo de vida do alvo e as operacoes de debug
//! ```
//!
//! `adapter.rs` e `server.rs` sairam daqui em 2026-09-11 (fatia 2 da frente
//! F), quando a sessao ganhou o segundo caminho — `attach` a um servidor que
//! ela mesma sobe — e este arquivo chegaria ao limite misturando tres coisas.

use std::{
    collections::BTreeMap,
    io,
    path::Path,
    process::{Child, Command, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    time::Duration,
};

use kinein_protocol::{
    BreakpointInfo, DebugEvaluateResult, SourceBreakpointParams, StackFrameInfo, VariableInfo,
};
use serde_json::{Value, json};

use super::DebugError;
use super::adapter::Adapter;
use super::parse::{
    breakpoints_arguments, parse_breakpoints, parse_evaluate, parse_stack_frames, parse_variables,
    preferred_scope_reference,
};
use super::reader::{note_continued, send_event, spawn_reader};
use super::server::DebugServer;
use super::target::DebugTarget;
use super::wire::Wire;
use crate::lsp::EventSender;

/// Tempo maximo aguardando respostas comuns do adapter.
pub(super) const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

/// Tempo maximo aguardando a resposta de `launch` (carregar o alvo demora
/// mais que um request comum em binarios grandes).
const LAUNCH_TIMEOUT: Duration = Duration::from_secs(20);

/// Frames maximos devolvidos por `debug.stackTrace` (JetBrains-like; a
/// paginacao so entra se o uso real pedir).
const STACK_TRACE_LEVELS: u32 = 20;

/// Tempo maximo do `disconnect` no encerramento; depois o processo e morto.
const DISCONNECT_TIMEOUT: Duration = Duration::from_secs(5);

/// Uma sessao de debug viva (um adapter, um processo alvo).
#[derive(Debug)]
pub(super) struct DapSession {
    child: Child,
    wire: Wire,
    events: EventSender,
    alive: Arc<AtomicBool>,
    stopped_thread: Arc<Mutex<Option<i64>>>,
    /// O servidor do alvo remoto, quando o kit declarou um. Guarda de RAII:
    /// ninguem o le, ele existe para CAIR junto com a sessao. Fica DEPOIS do
    /// `child` de proposito: o `Drop` mata o adaptador primeiro (desconecta),
    /// e so' entao o campo cai e mata o servidor.
    _server: Option<DebugServer>,
}

impl Drop for DapSession {
    fn drop(&mut self) {
        drop(self.child.kill());
        drop(self.child.wait());
    }
}

impl DapSession {
    /// Sobe o servidor (se o kit declarou um), sobe o adapter, faz a danca
    /// DAP (initialize -> launch|attach -> initialized -> replay de
    /// breakpoints -> configurationDone) e emite `event.debug.started`.
    pub(super) fn launch(
        root: &Path,
        target: &DebugTarget,
        breakpoints: &BTreeMap<String, Vec<SourceBreakpointParams>>,
        events: EventSender,
        adapter: &Adapter,
    ) -> Result<Self, DebugError> {
        // O servidor vem ANTES do adaptador, e so' faz sentido com um alvo
        // remoto para esperar: um `debugServer` sem `remoteTarget` e' um
        // processo que subiria e ninguem conectaria — recusado com o motivo.
        let server = match (&adapter.debug_server, &adapter.remote_target) {
            (Some(command), Some(remote)) => {
                // O servidor recebe o ELF no `{program}`: um modulo Python nao
                // tem arquivo para dar — e nao ha' servidor de debug para Python.
                let Some(program) = target.program_path() else {
                    return Err(DebugError::Adapter {
                        message: "o servidor de debug do kit precisa de um executavel; o alvo e' \
                                  um modulo Python"
                            .to_owned(),
                    });
                };
                Some(DebugServer::spawn(root, command, program, remote)?)
            }
            (Some(_), None) => {
                return Err(DebugError::Adapter {
                    message: "o kit declara debugServer sem remoteTarget: a IDE nao saberia \
                              a que porta conectar"
                        .to_owned(),
                });
            }
            (None, _) => None,
        };
        let mut child = Command::new(&adapter.program)
            .args(adapter.arguments)
            .current_dir(root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|error| {
                if error.kind() == io::ErrorKind::NotFound {
                    DebugError::MissingAdapter {
                        program: adapter.program.display().to_string(),
                    }
                } else {
                    DebugError::Adapter {
                        message: format!("falha ao iniciar {}: {error}", adapter.program.display()),
                    }
                }
            })?;
        let (Some(stdin), Some(stdout)) = (child.stdin.take(), child.stdout.take()) else {
            drop(child.kill());
            return Err(DebugError::Adapter {
                message: format!(
                    "{} subiu sem stdin/stdout utilizaveis",
                    adapter.program.display()
                ),
            });
        };

        let wire = Wire::new(stdin);
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
            _server: server,
        };
        // Drop mata o adapter (e o servidor) se qualquer passo do handshake
        // falhar.
        session.handshake(root, target, breakpoints, &initialized_receiver, adapter)?;
        send_event(
            &session.events,
            "event.debug.started",
            json!({ "program": target.display() }),
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
        breakpoints: &[SourceBreakpointParams],
    ) -> Result<Vec<BreakpointInfo>, DebugError> {
        let body = self.wire.request(
            "setBreakpoints",
            &breakpoints_arguments(file, breakpoints),
            REQUEST_TIMEOUT,
        )?;
        let lines: Vec<u32> = breakpoints.iter().map(|bp| bp.line).collect();
        Ok(parse_breakpoints(&body, &lines))
    }

    /// Avalia uma expressao no frame pedido (watch).
    ///
    /// Sem `frame_id`, usa o frame do TOPO da thread parada — e' o que a UI
    /// quer quando o usuario digita um watch sem ter clicado num frame. Se
    /// nao ha' thread parada, `stack_trace` devolve `NotStopped`, que e' a
    /// resposta honesta: nao existe contexto para avaliar.
    pub(super) fn evaluate(
        &self,
        expression: &str,
        frame_id: Option<i64>,
    ) -> Result<DebugEvaluateResult, DebugError> {
        let frame = match frame_id {
            Some(id) => id,
            None => self
                .stack_trace()?
                .first()
                .map(|frame| frame.id)
                .ok_or(DebugError::NotStopped)?,
        };
        let body = self.wire.request(
            "evaluate",
            &json!({
                "expression": expression,
                "frameId": frame,
                "context": "watch",
            }),
            REQUEST_TIMEOUT,
        )?;
        Ok(parse_evaluate(expression, &body))
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
        let Some(reference) = preferred_scope_reference(&scopes) else {
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
        target: &DebugTarget,
        breakpoints: &BTreeMap<String, Vec<SourceBreakpointParams>>,
        initialized: &mpsc::Receiver<()>,
        adapter: &Adapter,
    ) -> Result<(), DebugError> {
        self.wire.request(
            "initialize",
            &json!({
                "clientID": "kinein-vectis",
                "adapterID": adapter.id,
                "linesStartAt1": true,
                "columnsStartAt1": true,
                "pathFormat": "path",
            }),
            REQUEST_TIMEOUT,
        )?;
        // A resposta de launch/attach so chega depois do configurationDone
        // (o GDB o diz na fonte: `_LaunchOrAttachDeferredRequest`); o request
        // vai agora e a espera fica para o final.
        let (start_command, start_args) = adapter.start_request(root, target);
        let (launch_seq, launch_receiver) = self.wire.send_request(start_command, &start_args)?;
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
            .wait_response(start_command, launch_seq, &launch_receiver, LAUNCH_TIMEOUT)?;
        Ok(())
    }
}

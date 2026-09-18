//! Subsistema de debug: orquestracao do lldb-dap via DAP.
//!
//! O core sobe o debug adapter como processo filho, traduz o wire DAP em
//! eventos `event.debug.*` ja mastigados (a UI nunca fala DAP) e guarda os
//! breakpoints por arquivo — quem seta breakpoint antes da sessao existir
//! tem o conjunto replayado no proximo launch. Filosofia da IDE: orquestrar
//! o debugger maduro, nunca reimplementar um.
//!
//! Organizacao interna:
//! - [`session`]: sessao viva (spawn, handshake, thread leitora, requests);
//! - [`target`]: resolucao do binario "Automatico" (espelho do run).

mod adapter;
pub mod gdb_pick;
mod parse;
mod reader;
mod server;
mod session;
mod target;
mod transport;
mod wire;

use std::{collections::BTreeMap, error::Error, fmt, path::Path};

use kinein_protocol::{
    BreakpointInfo, DebugDisassembleParams, DebugEvaluateResult, DebugInstruction,
    DebugReadMemoryResult, DebugScopeInfo, SourceBreakpointParams, StackFrameInfo, VariableInfo,
};

use crate::lsp::EventSender;

pub use adapter::{AdapterChoice, DEBUGPY};
pub use target::{DebugTarget, resolve_program};

/// O adaptador padrao quando o kit nao escolheu nenhum.
#[must_use]
pub(crate) const fn default_adapter() -> &'static str {
    adapter::DEFAULT_ADAPTER
}

/// Error produced by the debug manager.
#[derive(Debug)]
pub enum DebugError {
    /// The chosen adapter binary is not on `PATH` (nor at the pinned path).
    MissingAdapter {
        /// What was looked for — `lldb-dap`, `probe-rs`, `gdb`, or a path.
        program: String,
    },
    /// O adaptador e' um MODULO do interpretador do projeto (debugpy) e o
    /// interpretador nao o tem: a mensagem ja' traz o passo para instalar la'.
    MissingAdapterModule {
        /// O que a sonda do interpretador respondeu.
        message: String,
    },
    /// A debug session is already running in this workspace.
    AlreadyRunning,
    /// No debug session is currently running.
    NotRunning,
    /// The debuggee is running (not paused), so stepping is impossible.
    NotStopped,
    /// The workspace has no automatic debug target.
    NoTarget {
        /// Human explanation of what to do instead.
        message: String,
    },
    /// The adapter failed, answered an error or timed out.
    Adapter {
        /// Underlying failure description.
        message: String,
    },
}

impl fmt::Display for DebugError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // Diz QUAL adaptador faltou: desde 2026-09-03 ele e' escolha do
            // kit, e "lldb-dap nao foi encontrado" para um kit com probe-rs
            // mandaria o usuario instalar a ferramenta errada.
            Self::MissingAdapter { program } => write!(
                formatter,
                "o adaptador de debug `{program}` nao foi encontrado (lldb-dap vem no \
                 pacote lldb; probe-rs e gdb tem passo a passo em Instalar ferramentas)"
            ),
            Self::AlreadyRunning => write!(
                formatter,
                "ja existe uma sessao de debug; pare-a antes (debug.stop)"
            ),
            Self::NotRunning => write!(formatter, "nenhuma sessao de debug em execucao"),
            Self::NotStopped => write!(
                formatter,
                "o processo nao esta pausado; pause ou aguarde um breakpoint"
            ),
            Self::NoTarget { message }
            | Self::Adapter { message }
            | Self::MissingAdapterModule { message } => {
                write!(formatter, "{message}")
            }
        }
    }
}

impl Error for DebugError {}

/// Owns the breakpoint store and the single debug session of a workspace.
#[derive(Debug)]
pub struct DebugManager {
    events: EventSender,
    breakpoints: BTreeMap<String, Vec<SourceBreakpointParams>>,
    session: Option<session::DapSession>,
}

impl DebugManager {
    /// Creates a manager that pushes `event.debug.*` through `events`.
    #[must_use]
    pub const fn new(events: EventSender) -> Self {
        Self {
            events,
            breakpoints: BTreeMap::new(),
            session: None,
        }
    }

    /// Returns `true` while a debug session is alive.
    #[must_use]
    pub fn is_running(&self) -> bool {
        self.session
            .as_ref()
            .is_some_and(session::DapSession::is_alive)
    }

    /// Avalia uma expressao (watch) no frame pedido, ou no do topo.
    pub fn evaluate(
        &mut self,
        expression: &str,
        frame_id: Option<i64>,
    ) -> Result<DebugEvaluateResult, DebugError> {
        self.live_session()
            .ok_or(DebugError::NotRunning)?
            .evaluate(expression, frame_id)
    }

    /// Replaces the breakpoint set of `file` (empty clears it).
    ///
    /// Without a live session the set is stored (`verified: false`) and
    /// replayed on the next launch; with one, the adapter answers what it
    /// actually bound.
    pub fn set_breakpoints(
        &mut self,
        file: &str,
        breakpoints: &[SourceBreakpointParams],
    ) -> Result<Vec<BreakpointInfo>, DebugError> {
        let normalized = normalize_breakpoints(breakpoints);
        if normalized.is_empty() {
            self.breakpoints.remove(file);
        } else {
            self.breakpoints.insert(file.to_owned(), normalized.clone());
        }
        if let Some(session) = self.live_session() {
            return session.set_breakpoints(file, &normalized);
        }
        Ok(normalized
            .iter()
            .map(|bp| BreakpointInfo {
                line: bp.line,
                verified: false,
            })
            .collect())
    }

    /// Launches a debug session for `program`, replaying stored breakpoints.
    ///
    /// `adapter_id`/`adapter_path` vem do KIT (papel `debugAdapter`). Sem
    /// escolha, cai no `lldb-dap` e o comportamento e o de sempre — foi assim
    /// que o desktop continuou funcionando quando o embarcado entrou.
    pub fn start(
        &mut self,
        root: &Path,
        target: &DebugTarget,
        choice: &AdapterChoice<'_>,
    ) -> Result<(), DebugError> {
        if self.is_running() {
            return Err(DebugError::AlreadyRunning);
        }
        // Sessao morta (terminated/EOF) ainda ocupa o slot: descarta antes.
        self.session = None;
        let adapter = adapter::Adapter::from_choice(choice);
        let session = session::DapSession::launch(
            root,
            target,
            &self.breakpoints,
            self.events.clone(),
            &adapter,
        )?;
        self.session = Some(session);
        Ok(())
    }

    /// Resumes execution from the paused thread.
    pub fn continue_run(&self) -> Result<(), DebugError> {
        self.live_session()
            .ok_or(DebugError::NotRunning)?
            .continue_run()
    }

    /// Steps over the current line (`next`).
    pub fn step_over(&self) -> Result<(), DebugError> {
        self.live_session()
            .ok_or(DebugError::NotRunning)?
            .step("next")
    }

    /// Steps into the call under the cursor (`stepIn`).
    pub fn step_in(&self) -> Result<(), DebugError> {
        self.live_session()
            .ok_or(DebugError::NotRunning)?
            .step("stepIn")
    }

    /// Steps out of the current function (`stepOut`).
    pub fn step_out(&self) -> Result<(), DebugError> {
        self.live_session()
            .ok_or(DebugError::NotRunning)?
            .step("stepOut")
    }

    /// Stack frames of the paused thread, top first.
    pub fn stack_trace(&self) -> Result<Vec<StackFrameInfo>, DebugError> {
        self.live_session()
            .ok_or(DebugError::NotRunning)?
            .stack_trace()
    }

    /// Variables of a frame's locals scope.
    pub fn frame_variables(&self, frame_id: i64) -> Result<Vec<VariableInfo>, DebugError> {
        self.live_session()
            .ok_or(DebugError::NotRunning)?
            .frame_variables(frame_id)
    }

    /// Children of a structured variable (`ref` handle).
    pub fn reference_variables(&self, reference: i64) -> Result<Vec<VariableInfo>, DebugError> {
        self.live_session()
            .ok_or(DebugError::NotRunning)?
            .reference_variables(reference)
    }

    /// Every scope of a frame (P3).
    pub fn scopes(&self, frame_id: i64) -> Result<Vec<DebugScopeInfo>, DebugError> {
        self.live_session()
            .ok_or(DebugError::NotRunning)?
            .scopes(frame_id)
    }

    /// `readMemory`, verbatim (P3).
    pub fn read_memory(
        &self,
        memory_reference: &str,
        offset: Option<i64>,
        count: u64,
    ) -> Result<DebugReadMemoryResult, DebugError> {
        self.live_session()
            .ok_or(DebugError::NotRunning)?
            .read_memory(memory_reference, offset, count)
    }

    /// `disassemble`, verbatim (P3).
    pub fn disassemble(
        &self,
        params: &DebugDisassembleParams,
    ) -> Result<Vec<DebugInstruction>, DebugError> {
        self.live_session()
            .ok_or(DebugError::NotRunning)?
            .disassemble(params)
    }

    /// Pauses the running debuggee.
    pub fn pause(&self) -> Result<(), DebugError> {
        self.live_session().ok_or(DebugError::NotRunning)?.pause()
    }

    /// Disconnects the session; only an adapter spawned by us is killed.
    pub fn stop(&mut self) -> Result<(), DebugError> {
        let Some(session) = self.session.take() else {
            return Err(DebugError::NotRunning);
        };
        // Drop mata o adapter; terminated/EOF ja emitiram (ou emitem agora)
        // o event.debug.finished unico da sessao.
        drop(session);
        Ok(())
    }

    /// Release the session and breakpoints when changing/closing a workspace.
    pub fn clear(&mut self) {
        drop(self.stop());
        self.breakpoints.clear();
    }

    /// The session, only while the adapter is alive.
    fn live_session(&self) -> Option<&session::DapSession> {
        self.session.as_ref().filter(|session| session.is_alive())
    }
}

/// Sorts, dedupes by LINE and drops invalid (zero) breakpoints.
///
/// A dedupe por linha, e nao pelo par (linha, condicao): duas condicoes na
/// mesma linha nao existem no modelo da UI — um breakpoint por linha, com uma
/// condicao opcional. Se um dia existirem, isto aqui e' o lugar que muda.
fn normalize_breakpoints(breakpoints: &[SourceBreakpointParams]) -> Vec<SourceBreakpointParams> {
    let mut normalized: Vec<SourceBreakpointParams> = breakpoints
        .iter()
        .filter(|bp| bp.line > 0)
        .cloned()
        .collect();
    normalized.sort_unstable_by_key(|bp| bp.line);
    normalized.dedup_by_key(|bp| bp.line);
    normalized
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;

    use kinein_protocol::SourceBreakpointParams;

    use super::{DebugError, DebugManager};

    /// Atalho: linhas viram breakpoints sem condicao.
    fn bps(lines: &[u32]) -> Vec<SourceBreakpointParams> {
        lines
            .iter()
            .map(|&line| SourceBreakpointParams {
                line,
                condition: None,
                hit_condition: None,
            })
            .collect()
    }

    #[test]
    fn breakpoint_store_keeps_condition_through_normalization() {
        let (sender, _receiver) = mpsc::channel();
        let mut manager = DebugManager::new(sender);

        // Desordenado, com zero invalido e uma linha repetida: a condicao da
        // PRIMEIRA ocorrencia da linha e' a que fica.
        let pedido = vec![
            SourceBreakpointParams {
                line: 9,
                condition: Some("i == 42".into()),
                hit_condition: None,
            },
            SourceBreakpointParams {
                line: 0,
                condition: Some("descartado".into()),
                hit_condition: None,
            },
            SourceBreakpointParams {
                line: 4,
                condition: None,
                hit_condition: Some("5".into()),
            },
        ];
        let stored = manager.set_breakpoints("/w/main.c", &pedido).unwrap();
        assert_eq!(
            stored.iter().map(|bp| bp.line).collect::<Vec<_>>(),
            vec![4, 9],
            "linha zero tinha que sumir e a ordem tinha que ser por linha"
        );

        let guardado = &manager.breakpoints["/w/main.c"];
        assert_eq!(guardado[0].hit_condition.as_deref(), Some("5"));
        assert_eq!(guardado[1].condition.as_deref(), Some("i == 42"));
    }

    #[test]
    fn breakpoint_store_normalizes_and_clears_without_a_session() {
        let (sender, _receiver) = mpsc::channel();
        let mut manager = DebugManager::new(sender);

        let stored = manager
            .set_breakpoints("/w/main.c", &bps(&[7, 3, 7, 0]))
            .unwrap();
        assert_eq!(
            stored.iter().map(|bp| bp.line).collect::<Vec<_>>(),
            vec![3, 7]
        );
        assert!(stored.iter().all(|bp| !bp.verified));

        let cleared = manager.set_breakpoints("/w/main.c", &[]).unwrap();
        assert!(cleared.is_empty());
        assert!(manager.breakpoints.is_empty());
    }

    #[test]
    fn session_operations_without_a_session_report_not_running() {
        let (sender, _receiver) = mpsc::channel();
        let mut manager = DebugManager::new(sender);

        assert!(!manager.is_running());
        assert!(matches!(
            manager.continue_run(),
            Err(DebugError::NotRunning)
        ));
        assert!(matches!(manager.step_over(), Err(DebugError::NotRunning)));
        assert!(matches!(manager.stop(), Err(DebugError::NotRunning)));
    }
}

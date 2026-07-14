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

mod session;
mod target;

use std::{collections::BTreeMap, error::Error, fmt, path::Path};

use kinein_protocol::{BreakpointInfo, StackFrameInfo, VariableInfo};

use crate::lsp::EventSender;

pub use target::resolve_program;

/// Error produced by the debug manager.
#[derive(Debug)]
pub enum DebugError {
    /// The `lldb-dap` binary is not on `PATH`.
    MissingAdapter,
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
            Self::MissingAdapter => write!(
                formatter,
                "lldb-dap nao foi encontrado no PATH (vem no pacote lldb; \
                 ex.: sudo pacman -S lldb)"
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
            Self::NoTarget { message } | Self::Adapter { message } => {
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
    breakpoints: BTreeMap<String, Vec<u32>>,
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

    /// Replaces the breakpoint set of `file` (empty clears it).
    ///
    /// Without a live session the set is stored (`verified: false`) and
    /// replayed on the next launch; with one, the adapter answers what it
    /// actually bound.
    pub fn set_breakpoints(
        &mut self,
        file: &str,
        lines: &[u32],
    ) -> Result<Vec<BreakpointInfo>, DebugError> {
        let normalized = normalize_lines(lines);
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
            .map(|&line| BreakpointInfo {
                line,
                verified: false,
            })
            .collect())
    }

    /// Launches a debug session for `program`, replaying stored breakpoints.
    pub fn start(&mut self, root: &Path, program: &Path) -> Result<(), DebugError> {
        if self.is_running() {
            return Err(DebugError::AlreadyRunning);
        }
        // Sessao morta (terminated/EOF) ainda ocupa o slot: descarta antes.
        self.session = None;
        let session =
            session::DapSession::launch(root, program, &self.breakpoints, self.events.clone())?;
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

    /// Pauses the running debuggee.
    pub fn pause(&self) -> Result<(), DebugError> {
        self.live_session().ok_or(DebugError::NotRunning)?.pause()
    }

    /// Stops the session (polite disconnect, then the adapter is killed).
    pub fn stop(&mut self) -> Result<(), DebugError> {
        let Some(session) = self.session.take() else {
            return Err(DebugError::NotRunning);
        };
        if session.is_alive() {
            session.shutdown();
        }
        // Drop mata o adapter; terminated/EOF ja emitiram (ou emitem agora)
        // o event.debug.finished unico da sessao.
        drop(session);
        Ok(())
    }

    /// The session, only while the adapter is alive.
    fn live_session(&self) -> Option<&session::DapSession> {
        self.session.as_ref().filter(|session| session.is_alive())
    }
}

/// Sorts, dedupes and drops invalid (zero) breakpoint lines.
fn normalize_lines(lines: &[u32]) -> Vec<u32> {
    let mut normalized: Vec<u32> = lines.iter().copied().filter(|&line| line > 0).collect();
    normalized.sort_unstable();
    normalized.dedup();
    normalized
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;

    use super::{DebugError, DebugManager};

    #[test]
    fn breakpoint_store_normalizes_and_clears_without_a_session() {
        let (sender, _receiver) = mpsc::channel();
        let mut manager = DebugManager::new(sender);

        let stored = manager.set_breakpoints("/w/main.c", &[7, 3, 7, 0]).unwrap();
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

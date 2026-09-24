//! Selecao/copia sob demanda sobre o emulador da sessao existente.
//! Nao mantem outro historico nem escreve no PTY.

use super::super::render::emit_locked_render;
use super::{TerminalError, TerminalManager};

impl TerminalManager {
    /// Selects the complete retained active buffer and publishes its identity.
    pub fn select_all(&self, id: &str, selection_id: &str) -> Result<(), TerminalError> {
        if selection_id.is_empty() || selection_id.len() > 128 {
            return Err(TerminalError::InvalidSelection);
        }
        let session = self.live(id)?;
        let mut state = session.state.lock().map_err(|_| TerminalError::Process {
            message: "estado do terminal indisponivel para selecionar".to_owned(),
        })?;
        state.select_all(selection_id);
        emit_locked_render(&self.events, id, &state);
        drop(state);
        Ok(())
    }

    /// Reads native selection text only when its identity still matches.
    pub fn copy_selection(
        &self,
        id: &str,
        selection_id: &str,
    ) -> Result<Option<String>, TerminalError> {
        let session = self.live(id)?;
        let state = session.state.lock().map_err(|_| TerminalError::Process {
            message: "estado do terminal indisponivel para copiar".to_owned(),
        })?;
        Ok(state.copy_selection(selection_id))
    }
}

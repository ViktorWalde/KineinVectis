//! Vocabulário de erro do domínio terminal.
//!
//! Mora em módulo próprio porque atravessa a fronteira: o `rpc.rs` mapeia cada
//! variante para um código JSON-RPC, então a lista é contrato, não detalhe do
//! gerenciador de sessão.

use std::error::Error;
use std::fmt;

use super::MAX_SESSIONS;

/// Error produced by the terminal session manager.
#[derive(Debug)]
pub enum TerminalError {
    /// Too many sessions open at once.
    TooMany,
    /// No session with the given id (or it already died).
    NotOpen,
    /// Gesto de mouse cujo contrato existe mas ainda não tem comportamento
    /// (clique/arrasto/movimento, fatia R5 de `docs/roadmaps/26`).
    MouseUnimplemented,
    /// The session process could not be spawned or reached.
    Process {
        /// Underlying failure description.
        message: String,
    },
}

impl fmt::Display for TerminalError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooMany => write!(
                formatter,
                "limite de {MAX_SESSIONS} terminais abertos atingido"
            ),
            Self::NotOpen => write!(formatter, "sessao de terminal inexistente ou encerrada"),
            Self::MouseUnimplemented => write!(
                formatter,
                "gesto de mouse ainda nao implementado: so a roda esta ativa"
            ),
            Self::Process { message } => write!(formatter, "{message}"),
        }
    }
}

impl Error for TerminalError {}

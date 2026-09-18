//! O desfecho de um pedido (`RequestOutcome`) e o erro dos lacos de IO
//! (`CoreError`). Saíram do `lib.rs` em 2026-09-18, quando ele bateu em 500
//! ao ganhar o `Deferred` (Etapa 2 F6).

use std::{error::Error, fmt, io};

use kinein_protocol::JsonRpcResponse;

/// Result of handling a request.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RequestOutcome {
    /// The core should continue reading requests.
    Continue(JsonRpcResponse),
    /// The core should return this response and stop the loop.
    Shutdown(JsonRpcResponse),
    /// A resposta vem DEPOIS, pelo canal de respostas adiadas (Etapa 2 F6):
    /// o laco nao escreve nada agora. O payload e' o marcador, nunca vai
    /// para o stdout.
    Deferred(JsonRpcResponse),
}

impl RequestOutcome {
    /// Returns the JSON-RPC response (o marcador, quando adiada).
    #[must_use]
    pub const fn response(&self) -> &JsonRpcResponse {
        match self {
            Self::Continue(response) | Self::Shutdown(response) | Self::Deferred(response) => {
                response
            }
        }
    }

    /// `true` quando a resposta real chega depois, pelo canal adiado.
    #[must_use]
    pub const fn is_deferred(&self) -> bool {
        matches!(self, Self::Deferred(_))
    }

    /// Returns `true` when the core should stop after writing the response.
    #[must_use]
    pub const fn should_shutdown(&self) -> bool {
        matches!(self, Self::Shutdown(_))
    }
}

/// Error returned by core IO loops.
#[derive(Debug)]
pub enum CoreError {
    /// Failed while reading a request.
    Read(io::Error),
    /// Failed while writing a response.
    Write(io::Error),
    /// Failed while serializing a response.
    Serialize(serde_json::Error),
}

impl fmt::Display for CoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read(error) => write!(formatter, "failed to read IPC request: {error}"),
            Self::Write(error) => write!(formatter, "failed to write IPC response: {error}"),
            Self::Serialize(error) => {
                write!(formatter, "failed to serialize IPC response: {error}")
            }
        }
    }
}

impl Error for CoreError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Read(error) | Self::Write(error) => Some(error),
            Self::Serialize(error) => Some(error),
        }
    }
}

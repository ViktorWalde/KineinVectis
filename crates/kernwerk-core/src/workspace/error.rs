//! Structured error produced while opening or persisting a workspace.

use std::{error::Error, fmt, io};

/// Error produced while opening or persisting a workspace.
#[derive(Debug)]
pub enum WorkspaceError {
    /// The requested root does not exist or cannot be resolved.
    InvalidRoot {
        /// Path as requested by the client.
        path: String,
        /// Underlying IO error.
        source: io::Error,
    },
    /// The requested root exists but is not a directory.
    NotADirectory {
        /// Canonical path that was rejected.
        path: String,
    },
    /// Workspace metadata could not be written.
    Persist {
        /// Path that failed.
        path: String,
        /// Underlying IO error.
        source: io::Error,
    },
    /// A directory could not be listed.
    ListDirectory {
        /// Path that failed.
        path: String,
        /// Underlying IO error.
        source: io::Error,
    },
    /// A new directory/project name is not a safe single path segment.
    InvalidName {
        /// Name as requested by the client.
        name: String,
        /// Human-readable reason.
        reason: String,
    },
    /// The target path already exists.
    AlreadyExists {
        /// Existing path.
        path: String,
    },
    /// A directory could not be created.
    CreateDirectory {
        /// Path that failed.
        path: String,
        /// Underlying IO error.
        source: io::Error,
    },
    /// A template file could not be written.
    WriteTemplate {
        /// Path that failed.
        path: String,
        /// Underlying IO error.
        source: io::Error,
    },
    /// A required external tool was missing.
    MissingTool {
        /// Tool executable.
        tool: &'static str,
    },
    /// A required external tool returned a failure.
    ToolFailed {
        /// Human-readable command line.
        command: String,
        /// Process exit code.
        exit_code: Option<i32>,
        /// Captured stderr/stdout detail.
        message: String,
    },
    /// Workspace metadata could not be serialized.
    Serialize(serde_json::Error),
}

impl WorkspaceError {
    /// Returns `true` when the error was caused by an invalid client path.
    #[must_use]
    pub const fn is_invalid_path(&self) -> bool {
        matches!(
            self,
            Self::InvalidRoot { .. }
                | Self::NotADirectory { .. }
                | Self::InvalidName { .. }
                | Self::AlreadyExists { .. }
        )
    }

    /// Returns `true` when the operation failed because a tool is missing.
    #[must_use]
    pub const fn is_missing_tool(&self) -> bool {
        matches!(self, Self::MissingTool { .. })
    }
}

impl fmt::Display for WorkspaceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRoot { path, source } => {
                write!(
                    formatter,
                    "nao foi possivel abrir o workspace em {path}: {source}"
                )
            }
            Self::NotADirectory { path } => {
                write!(formatter, "o caminho {path} nao e um diretorio")
            }
            Self::Persist { path, source } => {
                write!(
                    formatter,
                    "falha ao gravar metadados do workspace em {path}: {source}"
                )
            }
            Self::ListDirectory { path, source } => {
                write!(formatter, "falha ao listar diretorio {path}: {source}")
            }
            Self::InvalidName { name, reason } => {
                write!(formatter, "nome invalido {name}: {reason}")
            }
            Self::AlreadyExists { path } => {
                write!(formatter, "o caminho {path} ja existe")
            }
            Self::CreateDirectory { path, source } => {
                write!(formatter, "falha ao criar diretorio {path}: {source}")
            }
            Self::WriteTemplate { path, source } => {
                write!(formatter, "falha ao gravar template em {path}: {source}")
            }
            Self::MissingTool { tool } => {
                write!(formatter, "{tool} nao foi encontrado no PATH")
            }
            Self::ToolFailed {
                command,
                exit_code,
                message,
            } => {
                write!(
                    formatter,
                    "{command} falhou com codigo {exit_code:?}: {message}"
                )
            }
            Self::Serialize(error) => {
                write!(
                    formatter,
                    "falha ao serializar metadados do workspace: {error}"
                )
            }
        }
    }
}

impl Error for WorkspaceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidRoot { source, .. }
            | Self::Persist { source, .. }
            | Self::ListDirectory { source, .. }
            | Self::CreateDirectory { source, .. }
            | Self::WriteTemplate { source, .. } => Some(source),
            Self::Serialize(error) => Some(error),
            Self::NotADirectory { .. }
            | Self::InvalidName { .. }
            | Self::AlreadyExists { .. }
            | Self::MissingTool { .. }
            | Self::ToolFailed { .. } => None,
        }
    }
}

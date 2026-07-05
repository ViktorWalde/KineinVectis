//! Structured error produced by confined file system operations.

use std::{error::Error, fmt, io};

use super::MAX_READ_BYTES;

/// Error produced by confined file system operations.
#[derive(Debug)]
pub enum FsError {
    /// The path does not exist or cannot be resolved.
    InvalidPath {
        /// Path as requested by the client.
        path: String,
        /// Underlying IO error.
        source: io::Error,
    },
    /// The path resolves outside the workspace root.
    OutsideRoot {
        /// Canonical path that was rejected.
        path: String,
    },
    /// Expected a directory but found something else.
    NotADirectory {
        /// Canonical path that was rejected.
        path: String,
    },
    /// The requested new file path has no valid file name.
    InvalidFileName {
        /// Path as requested by the client.
        path: String,
    },
    /// The operation would overwrite an existing file.
    AlreadyExists {
        /// Path that was rejected.
        path: String,
    },
    /// Expected a regular file but found something else.
    NotAFile {
        /// Canonical path that was rejected.
        path: String,
    },
    /// The operation targets the workspace root itself, which is not allowed.
    WorkspaceRoot {
        /// Canonical path that was rejected.
        path: String,
    },
    /// The file exceeds [`MAX_READ_BYTES`].
    TooLarge {
        /// Canonical path that was rejected.
        path: String,
        /// Actual file size in bytes.
        size: u64,
    },
    /// The file is not valid UTF-8 text.
    NotText {
        /// Canonical path that was rejected.
        path: String,
    },
    /// Underlying IO failure while reading or writing.
    Io {
        /// Canonical path that failed.
        path: String,
        /// Underlying IO error.
        source: io::Error,
    },
    /// A required external search tool was missing.
    MissingTool {
        /// Tool executable.
        tool: &'static str,
    },
    /// A required external search tool returned a failure.
    ToolFailed {
        /// Tool executable.
        tool: &'static str,
        /// Captured stderr/stdout detail.
        message: String,
    },
}

impl FsError {
    /// Returns `true` when the error was caused by an invalid client path.
    #[must_use]
    pub const fn is_invalid_path(&self) -> bool {
        !matches!(
            self,
            Self::Io { .. } | Self::MissingTool { .. } | Self::ToolFailed { .. }
        )
    }

    /// Returns `true` when the operation failed because a tool is missing.
    #[must_use]
    pub const fn is_missing_tool(&self) -> bool {
        matches!(self, Self::MissingTool { .. })
    }
}

impl fmt::Display for FsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPath { path, source } => {
                write!(formatter, "caminho invalido {path}: {source}")
            }
            Self::OutsideRoot { path } => {
                write!(formatter, "o caminho {path} esta fora do workspace aberto")
            }
            Self::NotADirectory { path } => {
                write!(formatter, "o caminho {path} nao e um diretorio")
            }
            Self::InvalidFileName { path } => {
                write!(formatter, "o caminho {path} nao tem nome de arquivo valido")
            }
            Self::AlreadyExists { path } => {
                write!(formatter, "o arquivo {path} ja existe")
            }
            Self::NotAFile { path } => {
                write!(formatter, "o caminho {path} nao e um arquivo regular")
            }
            Self::WorkspaceRoot { path } => {
                write!(
                    formatter,
                    "o caminho {path} e a raiz do workspace e nao pode ser renomeado ou removido"
                )
            }
            Self::TooLarge { path, size } => {
                write!(
                    formatter,
                    "o arquivo {path} tem {size} bytes e excede o limite de {MAX_READ_BYTES}"
                )
            }
            Self::NotText { path } => {
                write!(formatter, "o arquivo {path} nao e texto UTF-8 valido")
            }
            Self::Io { path, source } => {
                write!(formatter, "falha de IO em {path}: {source}")
            }
            Self::MissingTool { tool } => {
                write!(formatter, "{tool} nao foi encontrado no PATH")
            }
            Self::ToolFailed { tool, message } => {
                write!(formatter, "{tool} falhou: {message}")
            }
        }
    }
}

impl Error for FsError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidPath { source, .. } | Self::Io { source, .. } => Some(source),
            Self::OutsideRoot { .. }
            | Self::NotADirectory { .. }
            | Self::InvalidFileName { .. }
            | Self::AlreadyExists { .. }
            | Self::NotAFile { .. }
            | Self::WorkspaceRoot { .. }
            | Self::TooLarge { .. }
            | Self::NotText { .. }
            | Self::MissingTool { .. }
            | Self::ToolFailed { .. } => None,
        }
    }
}

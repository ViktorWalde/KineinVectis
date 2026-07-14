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
    /// The file no longer matches the content observed by the editor.
    ChangedOnDisk {
        /// Canonical path whose external version must be reviewed.
        path: String,
    },
    /// A multi-file transaction failed and could not restore every file.
    RollbackFailed {
        /// Original write failure plus rollback failures.
        message: String,
    },
    /// A bulk replacement exceeded a defensive resource limit.
    ReplaceLimit {
        /// Limit that rejected the operation.
        message: String,
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
            Self::Io { .. }
                | Self::RollbackFailed { .. }
                | Self::MissingTool { .. }
                | Self::ToolFailed { .. }
        )
    }

    /// Returns `true` when the operation failed because a tool is missing.
    #[must_use]
    pub const fn is_missing_tool(&self) -> bool {
        matches!(self, Self::MissingTool { .. })
    }

    /// Returns the conflicting path when the disk changed since the last read.
    #[must_use]
    pub fn changed_path(&self) -> Option<&str> {
        match self {
            Self::ChangedOnDisk { path } => Some(path),
            _ => None,
        }
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
            Self::ChangedOnDisk { path } => {
                write!(
                    formatter,
                    "o arquivo {path} foi alterado fora da IDE; revise antes de salvar"
                )
            }
            Self::RollbackFailed { message } | Self::ReplaceLimit { message } => {
                formatter.write_str(message)
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
            | Self::ChangedOnDisk { .. }
            | Self::RollbackFailed { .. }
            | Self::ReplaceLimit { .. }
            | Self::MissingTool { .. }
            | Self::ToolFailed { .. } => None,
        }
    }
}

//! Tipos de dominio e erros do subsistema LSP.
//!
//! Sao os valores que cruzam a fronteira do modulo: localizacoes resolvidas,
//! planos de edicao de rename e o erro estruturado que o roteador `lsp.*`
//! traduz em codigos JSON-RPC.

use std::{error::Error, fmt};

/// Localizacao de codigo resolvida por um servidor LSP.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LspLocation {
    /// Caminho absoluto do alvo.
    pub path: String,
    /// Linha 1-based.
    pub line: u64,
    /// Coluna 1-based.
    pub column: u64,
}

/// Um edit de texto LSP com posicoes 0-based em unidades UTF-16.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TextSpanEdit {
    /// Linha inicial (0-based).
    pub start_line: u64,
    /// Coluna inicial (0-based, UTF-16).
    pub start_character: u64,
    /// Linha final (0-based).
    pub end_line: u64,
    /// Coluna final (0-based, UTF-16).
    pub end_character: u64,
    /// Texto que substitui o range.
    pub new_text: String,
}

/// Edits de rename agrupados por arquivo.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FileEdits {
    /// Caminho absoluto do arquivo alvo.
    pub path: String,
    /// Versao LSP esperada pelo servidor, quando o `WorkspaceEdit` usa
    /// `OptionalVersionedTextDocumentIdentifier`.
    pub version: Option<i64>,
    /// Edits a aplicar, na ordem enviada pelo servidor.
    pub edits: Vec<TextSpanEdit>,
}

/// Plano de aplicacao de um `WorkspaceEdit` de rename.
#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct WorkspaceEditPlan {
    /// Chave do servidor que produziu o plano. `None` usa o principal da
    /// linguagem; origem interna, nunca fornecida pela UI ou pelo wire LSP.
    pub server: Option<&'static str>,
    /// Arquivos afetados; vazio quando o servidor nao encontrou o simbolo.
    pub files: Vec<FileEdits>,
}

impl WorkspaceEditPlan {
    /// Total de edits em todos os arquivos do plano.
    #[must_use]
    pub fn edit_count(&self) -> u64 {
        let total: usize = self.files.iter().map(|file| file.edits.len()).sum();
        u64::try_from(total).unwrap_or(u64::MAX)
    }
}

/// Erro produzido pelo cliente LSP gerenciado.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum LspError {
    /// A extensao do arquivo ainda nao tem servidor configurado.
    UnsupportedFile {
        /// Caminho solicitado.
        path: String,
    },
    /// O executavel do language server nao foi encontrado.
    MissingServer {
        /// Comando esperado.
        command: String,
    },
    /// Falha ao iniciar ou inicializar o servidor.
    ServerFailed {
        /// Comando do servidor.
        command: String,
        /// Mensagem tecnica.
        message: String,
    },
    /// O servidor nao respondeu a tempo.
    Timeout {
        /// Metodo LSP solicitado.
        method: &'static str,
    },
    /// O servidor respondeu com erro LSP.
    RequestFailed {
        /// Metodo LSP solicitado.
        method: &'static str,
        /// Mensagem do servidor.
        message: String,
    },
    /// Resposta inesperada ou canal interno quebrado.
    Transport {
        /// Mensagem tecnica.
        message: String,
    },
}

impl LspError {
    /// Retorna `true` quando o erro e causado por arquivo sem servidor.
    #[must_use]
    pub const fn is_invalid_params(&self) -> bool {
        matches!(self, Self::UnsupportedFile { .. })
    }

    /// Retorna `true` quando o binario do language server esta ausente.
    #[must_use]
    pub const fn is_missing_tool(&self) -> bool {
        matches!(self, Self::MissingServer { .. })
    }
}

impl fmt::Display for LspError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedFile { path } => {
                write!(formatter, "nenhum servidor LSP suporta o arquivo {path}")
            }
            Self::MissingServer { command } => {
                write!(formatter, "{command} nao foi encontrado no PATH")
            }
            Self::ServerFailed { command, message } => {
                write!(formatter, "falha ao iniciar {command}: {message}")
            }
            Self::Timeout { method } => {
                write!(formatter, "{method} nao respondeu a tempo")
            }
            Self::RequestFailed { method, message } => {
                write!(formatter, "{method} falhou: {message}")
            }
            Self::Transport { message } => formatter.write_str(message),
        }
    }
}

impl Error for LspError {}

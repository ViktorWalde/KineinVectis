//! Erro estruturado do dominio das Configuration Actions.

use std::{error::Error, fmt};

use crate::fsops::FsError;

/// Falha ao listar, planejar ou aplicar uma Configuration Action.
#[derive(Debug)]
pub enum ConfigActionError {
    /// O id nao existe no catalogo.
    UnknownAction {
        /// Id pedido pelo cliente.
        id: String,
    },
    /// O build system da acao nao esta ativo neste workspace.
    OutOfScope {
        /// Id pedido pelo cliente.
        id: String,
    },
    /// Falta um parametro declarado como obrigatorio.
    MissingParam {
        /// Nome do parametro.
        name: &'static str,
    },
    /// Um parametro chegou com valor que o gerador nao aceita.
    InvalidParam {
        /// Nome do parametro.
        name: &'static str,
        /// Por que o valor foi recusado.
        reason: String,
    },
    /// O projeto nao esta no estado que a acao exige.
    NotApplicable {
        /// Explicacao para a UI, na lingua do usuario.
        reason: String,
    },
    /// O arquivo de configuracao tem uma forma que o editor nao entende.
    ///
    /// Recusar e o comportamento CERTO: um editor textual que adivinha corrompe
    /// o manifest do usuario, e nao ha desfazer para isso.
    UnsupportedShape {
        /// Arquivo relativo a raiz.
        path: String,
        /// O que impediu a edicao segura.
        reason: String,
    },
    /// Falha de arquivo (confinamento, `compare-before-save`, IO).
    Fs(FsError),
}

impl ConfigActionError {
    /// `true` quando o cliente mandou algo invalido (vira `INVALID_PARAMS`).
    #[must_use]
    pub const fn is_invalid_params(&self) -> bool {
        match self {
            Self::UnknownAction { .. }
            | Self::OutOfScope { .. }
            | Self::MissingParam { .. }
            | Self::InvalidParam { .. }
            | Self::NotApplicable { .. }
            | Self::UnsupportedShape { .. } => true,
            Self::Fs(error) => error.is_invalid_path(),
        }
    }

    /// Caminho em conflito quando o disco mudou desde o preview.
    #[must_use]
    pub fn changed_path(&self) -> Option<&str> {
        match self {
            Self::Fs(error) => error.changed_path(),
            _ => None,
        }
    }
}

impl From<FsError> for ConfigActionError {
    fn from(error: FsError) -> Self {
        Self::Fs(error)
    }
}

impl fmt::Display for ConfigActionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownAction { id } => {
                write!(formatter, "configuration action desconhecida: {id}")
            }
            Self::OutOfScope { id } => write!(
                formatter,
                "a acao {id} nao pertence a nenhum build system ativo deste workspace"
            ),
            Self::MissingParam { name } => {
                write!(
                    formatter,
                    "o parametro obrigatorio '{name}' nao foi enviado"
                )
            }
            Self::InvalidParam { name, reason } => {
                write!(formatter, "parametro '{name}' invalido: {reason}")
            }
            Self::NotApplicable { reason } => formatter.write_str(reason),
            Self::UnsupportedShape { path, reason } => write!(
                formatter,
                "{path} tem uma forma que a IDE nao edita com seguranca: {reason}"
            ),
            Self::Fs(error) => write!(formatter, "{error}"),
        }
    }
}

impl Error for ConfigActionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Fs(error) => Some(error),
            _ => None,
        }
    }
}

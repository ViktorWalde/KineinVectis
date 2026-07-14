//! Git orquestrado sobre o binario `git`.
//!
//! O dominio permanece stateless: a UI decide quando consultar e toda acao
//! usa formatos estaveis, NUL-delimited quando ha paths. Nenhuma mensagem
//! localizada do Git e usada como controle de fluxo.

mod operations;
mod parse;

use std::{error::Error, fmt};

pub use operations::{
    blame, branch_name_valid, branches, checkout, commit, commit_diff, create_branch, discard,
    file_diff, log, pull, push, stage, stash_pop, stash_push, status, unstage,
};
pub use parse::{parse_blame, parse_hunks, parse_log, parse_status};

/// Failure while orchestrating the `git` binary.
#[derive(Debug)]
pub enum GitError {
    /// `git` is not on `PATH`.
    MissingGit,
    /// A mutation was asked outside a git repository.
    NotARepo,
    /// `git.commit` with an empty index.
    NothingStaged,
    /// The index includes paths hidden from the opened workspace.
    InvisibleStagedPaths {
        /// Repository-relative paths that the Git UI does not expose.
        paths: Vec<String>,
    },
    /// `git` ran but failed unexpectedly (a non-repo read is NOT this).
    Failed {
        /// Underlying failure description.
        message: String,
    },
}

impl fmt::Display for GitError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingGit => write!(
                formatter,
                "git nao foi encontrado no PATH (ex.: sudo pacman -S git)"
            ),
            Self::NotARepo => write!(formatter, "o workspace nao e um repositorio git"),
            Self::NothingStaged => write!(
                formatter,
                "nada staged para commitar; marque arquivos na aba Git antes"
            ),
            Self::InvisibleStagedPaths { paths } => write!(
                formatter,
                "o indice contem alteracoes staged invisiveis neste workspace: {}",
                paths.join(", ")
            ),
            Self::Failed { message } => write!(formatter, "{message}"),
        }
    }
}

impl Error for GitError {}

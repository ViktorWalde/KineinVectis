//! Transacoes confirmaveis para `WorkspaceEdit` do LSP.
//!
//! O servidor produz um plano, mas nenhuma escrita acontece antes da previa.
//! Na confirmacao, todos os snapshots de disco sao validados e cada arquivo e
//! salvo de forma atomica. Uma falha intermediaria dispara rollback inverso.

use std::{
    collections::{HashMap, HashSet, VecDeque},
    error::Error,
    fmt,
    path::{Path, PathBuf},
};

use kinein_protocol::{LspWorkspaceEditFilePreview, LspWorkspaceEditPreviewResult};

use crate::fsops;

use super::{LspError, WorkspaceEditPlan, apply_text_edits};

const MAX_PENDING_TRANSACTIONS: usize = 8;
const MAX_TRANSACTION_FILES: usize = 256;
const MAX_TRANSACTION_EDITS: u64 = 20_000;
const MAX_PREVIEW_BYTES: usize = 16 * 1_048_576;

#[derive(Debug, Clone)]
struct PendingFile {
    path: PathBuf,
    expected_disk: String,
    before_content: String,
    after_content: String,
    edits: u64,
}

#[derive(Debug, Clone)]
struct PendingTransaction {
    id: String,
    title: String,
    files: Vec<PendingFile>,
    edits: u64,
}

impl PendingTransaction {
    fn preview(&self) -> LspWorkspaceEditPreviewResult {
        LspWorkspaceEditPreviewResult {
            transaction_id: self.id.clone(),
            title: self.title.clone(),
            files: self
                .files
                .iter()
                .map(|file| LspWorkspaceEditFilePreview {
                    path: file.path.display().to_string(),
                    before_content: file.before_content.clone(),
                    after_content: file.after_content.clone(),
                    edits: file.edits,
                })
                .collect(),
            edits: self.edits,
        }
    }
}

/// Resultado interno da confirmacao; inclui o conteudo final para ressincronia
/// dos documentos que ja estavam abertos no LSP.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WorkspaceEditApplied {
    /// Identificador consumido.
    pub transaction_id: String,
    /// Titulo apresentado na previa.
    pub title: String,
    /// Caminho e conteudo final de cada arquivo gravado.
    pub files: Vec<(PathBuf, String)>,
    /// Total de edits aplicados.
    pub edits: u64,
}

/// Falha de preparacao, validacao ou confirmacao da transacao.
#[derive(Debug)]
pub enum WorkspaceEditTransactionError {
    /// Erro confinado de leitura/escrita.
    FileSystem(fsops::FsError),
    /// Range LSP invalido, sobreposto ou fora do texto.
    Edit(LspError),
    /// O servidor amarrou o edit a uma versao que ja nao esta aberta.
    StaleVersion {
        /// Arquivo afetado.
        path: String,
        /// Versao presente no plano.
        expected: i64,
        /// Versao atual no manager, se o documento ainda estiver aberto.
        actual: Option<i64>,
    },
    /// Identificador ausente/expirado.
    UnknownTransaction {
        /// Identificador fornecido pelo cliente.
        id: String,
    },
    /// Plano ambiguo ou alem dos limites de seguranca.
    InvalidPlan {
        /// Motivo da rejeicao antes de qualquer escrita.
        message: String,
    },
}

impl fmt::Display for WorkspaceEditTransactionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FileSystem(error) => error.fmt(formatter),
            Self::Edit(error) => error.fmt(formatter),
            Self::StaleVersion {
                path,
                expected,
                actual,
            } => write!(
                formatter,
                "workspace edit obsoleto para {path}: versao esperada {expected}, atual {actual:?}"
            ),
            Self::UnknownTransaction { id } => {
                write!(
                    formatter,
                    "transacao de workspace edit inexistente ou expirada: {id}"
                )
            }
            Self::InvalidPlan { message } => formatter.write_str(message),
        }
    }
}

impl Error for WorkspaceEditTransactionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::FileSystem(error) => Some(error),
            Self::Edit(error) => Some(error),
            Self::StaleVersion { .. }
            | Self::UnknownTransaction { .. }
            | Self::InvalidPlan { .. } => None,
        }
    }
}

/// Cache limitado de transacoes LSP aguardando confirmacao explicita.
#[derive(Debug, Default)]
pub struct WorkspaceEditTransactions {
    pending: HashMap<String, PendingTransaction>,
    order: VecDeque<String>,
    next_id: u64,
}

impl WorkspaceEditTransactions {
    /// Descarta todas as previas, por exemplo ao trocar de workspace.
    pub fn clear(&mut self) {
        self.pending.clear();
        self.order.clear();
    }

    /// Valida versoes opcionais do `documentChanges` contra o manager LSP.
    pub fn validate_versions(
        plan: &WorkspaceEditPlan,
        mut current_version: impl FnMut(&Path) -> Option<i64>,
    ) -> Result<(), WorkspaceEditTransactionError> {
        for file in &plan.files {
            let Some(expected) = file.version else {
                continue;
            };
            let actual = current_version(Path::new(&file.path));
            if actual != Some(expected) {
                return Err(WorkspaceEditTransactionError::StaleVersion {
                    path: file.path.clone(),
                    expected,
                    actual,
                });
            }
        }
        Ok(())
    }

    /// Prepara uma previa sem alterar o disco.
    pub fn prepare(
        &mut self,
        root: &Path,
        active_path: &Path,
        active_content: &str,
        plan: &WorkspaceEditPlan,
        title: impl Into<String>,
    ) -> Result<LspWorkspaceEditPreviewResult, WorkspaceEditTransactionError> {
        if plan.files.len() > MAX_TRANSACTION_FILES {
            return Err(WorkspaceEditTransactionError::InvalidPlan {
                message: format!(
                    "workspace edit excede o limite de {MAX_TRANSACTION_FILES} arquivos"
                ),
            });
        }
        let edits = plan.edit_count();
        if edits > MAX_TRANSACTION_EDITS {
            return Err(WorkspaceEditTransactionError::InvalidPlan {
                message: format!("workspace edit excede o limite de {MAX_TRANSACTION_EDITS} edits"),
            });
        }

        let mut files = Vec::with_capacity(plan.files.len());
        let mut seen = HashSet::with_capacity(plan.files.len());
        let mut preview_bytes = 0_usize;
        for file in &plan.files {
            let target = fsops::confine_file(root, Path::new(&file.path))
                .map_err(WorkspaceEditTransactionError::FileSystem)?;
            if !seen.insert(target.clone()) {
                return Err(WorkspaceEditTransactionError::InvalidPlan {
                    message: format!(
                        "workspace edit contem o arquivo {} mais de uma vez",
                        target.display()
                    ),
                });
            }
            let (_, expected_disk) = fsops::read_file(root, &target)
                .map_err(WorkspaceEditTransactionError::FileSystem)?;
            let before_content = if target == active_path {
                active_content.to_owned()
            } else {
                expected_disk.clone()
            };
            let after_content = apply_text_edits(&before_content, &file.edits)
                .map_err(WorkspaceEditTransactionError::Edit)?;
            preview_bytes = preview_bytes
                .saturating_add(before_content.len())
                .saturating_add(after_content.len());
            if preview_bytes > MAX_PREVIEW_BYTES {
                return Err(WorkspaceEditTransactionError::InvalidPlan {
                    message: format!(
                        "workspace edit excede o limite de {MAX_PREVIEW_BYTES} bytes de previa"
                    ),
                });
            }
            files.push(PendingFile {
                path: target,
                expected_disk,
                before_content,
                after_content,
                edits: u64::try_from(file.edits.len()).unwrap_or(u64::MAX),
            });
        }

        self.next_id = self.next_id.saturating_add(1);
        let id = format!("workspace-edit-{}", self.next_id);
        let transaction = PendingTransaction {
            id: id.clone(),
            title: title.into(),
            files,
            edits,
        };
        let preview = transaction.preview();
        self.pending.insert(id.clone(), transaction);
        self.order.push_back(id);
        while self.order.len() > MAX_PENDING_TRANSACTIONS {
            if let Some(expired) = self.order.pop_front() {
                self.pending.remove(&expired);
            }
        }
        Ok(preview)
    }

    /// Confirma uma previa, com compare-before-save e rollback inverso.
    pub fn apply(
        &mut self,
        root: &Path,
        transaction_id: &str,
    ) -> Result<WorkspaceEditApplied, WorkspaceEditTransactionError> {
        self.apply_with(root, transaction_id, fsops::write_file_if_unchanged)
    }

    fn apply_with(
        &mut self,
        root: &Path,
        transaction_id: &str,
        mut write: impl FnMut(&Path, &Path, &str, &str) -> Result<(PathBuf, u64), fsops::FsError>,
    ) -> Result<WorkspaceEditApplied, WorkspaceEditTransactionError> {
        let transaction = self.pending.get(transaction_id).cloned().ok_or_else(|| {
            WorkspaceEditTransactionError::UnknownTransaction {
                id: transaction_id.to_owned(),
            }
        })?;

        let updates = transaction
            .files
            .iter()
            .map(|file| fsops::TextFileUpdate {
                path: file.path.clone(),
                expected_content: file.expected_disk.clone(),
                new_content: file.after_content.clone(),
            })
            .collect::<Vec<_>>();
        fsops::write_text_transaction_with(root, &updates, &mut write)
            .map_err(WorkspaceEditTransactionError::FileSystem)?;

        self.remove(transaction_id);
        Ok(WorkspaceEditApplied {
            transaction_id: transaction.id,
            title: transaction.title,
            files: transaction
                .files
                .into_iter()
                .map(|file| (file.path, file.after_content))
                .collect(),
            edits: transaction.edits,
        })
    }

    /// Cancela uma previa ainda pendente.
    pub fn cancel(&mut self, transaction_id: &str) -> Result<(), WorkspaceEditTransactionError> {
        if self.remove(transaction_id).is_none() {
            return Err(WorkspaceEditTransactionError::UnknownTransaction {
                id: transaction_id.to_owned(),
            });
        }
        Ok(())
    }

    fn remove(&mut self, transaction_id: &str) -> Option<PendingTransaction> {
        self.order.retain(|id| id != transaction_id);
        self.pending.remove(transaction_id)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs, io,
        path::{Path, PathBuf},
    };

    use super::{WorkspaceEditTransactionError, WorkspaceEditTransactions};
    use crate::{
        fsops,
        lsp::{FileEdits, TextSpanEdit, WorkspaceEditPlan},
    };

    fn workspace(name: &str) -> PathBuf {
        let root = std::env::temp_dir()
            .join("kinein-workspace-edit-tests")
            .join(format!("{}-{name}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        root
    }

    fn replace(path: &Path, old_len: u64, new_text: &str) -> WorkspaceEditPlan {
        WorkspaceEditPlan {
            server: None,
            files: vec![FileEdits {
                path: path.display().to_string(),
                version: None,
                edits: vec![TextSpanEdit {
                    start_line: 0,
                    start_character: 0,
                    end_line: 0,
                    end_character: old_len,
                    new_text: new_text.to_owned(),
                }],
            }],
        }
    }

    #[test]
    fn prepare_does_not_write_and_cancel_discards_transaction() {
        let root = workspace("cancel");
        let path = root.join("main.rs");
        fs::write(&path, "main\n").unwrap();
        let mut transactions = WorkspaceEditTransactions::default();

        let preview = transactions
            .prepare(
                &root,
                &path,
                "main\n",
                &replace(&path, 4, "start"),
                "Rename",
            )
            .unwrap();

        assert_eq!(fs::read_to_string(&path).unwrap(), "main\n");
        assert_eq!(preview.files[0].after_content, "start\n");
        transactions.cancel(&preview.transaction_id).unwrap();
        assert!(matches!(
            transactions.apply(&root, &preview.transaction_id),
            Err(WorkspaceEditTransactionError::UnknownTransaction { .. })
        ));
    }

    #[test]
    fn apply_rejects_changed_disk_without_overwriting_it() {
        let root = workspace("stale");
        let path = root.join("main.rs");
        fs::write(&path, "main\n").unwrap();
        let mut transactions = WorkspaceEditTransactions::default();
        let preview = transactions
            .prepare(
                &root,
                &path,
                "main\n",
                &replace(&path, 4, "start"),
                "Rename",
            )
            .unwrap();
        fs::write(&path, "external\n").unwrap();

        assert!(matches!(
            transactions.apply(&root, &preview.transaction_id),
            Err(WorkspaceEditTransactionError::FileSystem(
                fsops::FsError::ChangedOnDisk { .. }
            ))
        ));
        assert_eq!(fs::read_to_string(&path).unwrap(), "external\n");
    }

    #[test]
    fn failure_after_first_write_rolls_back_the_first_file() {
        let root = workspace("rollback");
        let first = root.join("a.rs");
        let second = root.join("b.rs");
        fs::write(&first, "alpha\n").unwrap();
        fs::write(&second, "beta\n").unwrap();
        let plan = WorkspaceEditPlan {
            server: None,
            files: vec![
                replace(&first, 5, "new-alpha").files.remove(0),
                replace(&second, 4, "new-beta").files.remove(0),
            ],
        };
        let mut transactions = WorkspaceEditTransactions::default();
        let preview = transactions
            .prepare(&root, &first, "alpha\n", &plan, "Two files")
            .unwrap();
        let mut writes = 0_u8;

        let result = transactions.apply_with(
            &root,
            &preview.transaction_id,
            |root, path, content, expected| {
                writes = writes.saturating_add(1);
                if writes == 2 {
                    return Err(fsops::FsError::Io {
                        path: path.display().to_string(),
                        source: io::Error::other("injected second write failure"),
                    });
                }
                fsops::write_file_if_unchanged(root, path, content, expected)
            },
        );

        assert!(matches!(
            result,
            Err(WorkspaceEditTransactionError::FileSystem(
                fsops::FsError::Io { .. }
            ))
        ));
        assert_eq!(fs::read_to_string(&first).unwrap(), "alpha\n");
        assert_eq!(fs::read_to_string(&second).unwrap(), "beta\n");
    }

    #[test]
    fn successful_apply_consumes_transaction() {
        let root = workspace("success");
        let path = root.join("main.rs");
        fs::write(&path, "main\n").unwrap();
        let mut transactions = WorkspaceEditTransactions::default();
        let preview = transactions
            .prepare(
                &root,
                &path,
                "main\n",
                &replace(&path, 4, "start"),
                "Rename",
            )
            .unwrap();

        let applied = transactions.apply(&root, &preview.transaction_id).unwrap();

        assert_eq!(applied.files[0].1, "start\n");
        assert_eq!(fs::read_to_string(&path).unwrap(), "start\n");
        assert!(transactions.cancel(&preview.transaction_id).is_err());
    }
}

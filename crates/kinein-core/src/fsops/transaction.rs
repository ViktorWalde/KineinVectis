//! Compare-before-save e rollback para mutacoes atomicas multi-arquivo.

use std::path::{Path, PathBuf};

use super::{FsError, read_file, write_file_if_unchanged};

/// Um arquivo dentro de uma transacao textual.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TextFileUpdate {
    /// Caminho absoluto confinado ao workspace.
    pub path: PathBuf,
    /// Snapshot que ainda precisa estar no disco ao aplicar.
    pub expected_content: String,
    /// Conteudo que substitui o snapshot quando a transacao confirma.
    pub new_content: String,
}

/// Valida todos os snapshots, grava em ordem e restaura em ordem inversa se
/// uma escrita falhar.
pub fn write_text_transaction(root: &Path, updates: &[TextFileUpdate]) -> Result<(), FsError> {
    write_text_transaction_with(root, updates, write_file_if_unchanged)
}

/// Variante injetavel usada para provar rollback em testes de falha parcial.
pub fn write_text_transaction_with(
    root: &Path,
    updates: &[TextFileUpdate],
    mut write: impl FnMut(&Path, &Path, &str, &str) -> Result<(PathBuf, u64), FsError>,
) -> Result<(), FsError> {
    for update in updates {
        let (_, current) = read_file(root, &update.path)?;
        if current != update.expected_content {
            return Err(FsError::ChangedOnDisk {
                path: update.path.display().to_string(),
            });
        }
    }

    let mut written: Vec<&TextFileUpdate> = Vec::with_capacity(updates.len());
    for update in updates {
        if let Err(error) = write(
            root,
            &update.path,
            &update.new_content,
            &update.expected_content,
        ) {
            let mut rollback_errors = Vec::new();
            for completed in written.iter().rev() {
                if let Err(rollback_error) = write(
                    root,
                    &completed.path,
                    &completed.expected_content,
                    &completed.new_content,
                ) {
                    rollback_errors.push(rollback_error.to_string());
                }
            }
            if rollback_errors.is_empty() {
                return Err(error);
            }
            return Err(FsError::RollbackFailed {
                message: format!(
                    "falha ao gravar ({error}) e ao restaurar: {}",
                    rollback_errors.join("; ")
                ),
            });
        }
        written.push(update);
    }
    Ok(())
}

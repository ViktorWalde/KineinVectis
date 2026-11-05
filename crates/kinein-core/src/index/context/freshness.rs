//! Historical flags must never be presented as a current effective environment.

use std::{io, path::Path};

use kinein_protocol::FileContext;

use super::CompileContext;

impl CompileContext {
    pub(super) fn apply_freshness(&self, root: &Path, context: &mut FileContext) {
        if !matches!(context.language.as_str(), "c" | "cpp") {
            return;
        }
        let reason = self.snapshot_reason(root).or_else(|| {
            let unit = context.unit.as_ref()?;
            missing_compiler(&unit.compiler, &unit.directory)
        });
        if let Some(reason) = reason {
            let note = format!(
                "Contexto desatualizado: {reason}; reconfigure/recarregue para atualizar. Comando anterior apenas para inspeção."
            );
            context.hint = Some(
                context
                    .hint
                    .take()
                    .map_or_else(|| note.clone(), |hint| format!("{hint}; {note}")),
            );
            context.stale_because = Some(reason);
            context.effective = None;
            context.drift.clear();
        }
    }

    fn snapshot_reason(&self, root: &Path) -> Option<String> {
        if let Some(path) = &self.cdb_path {
            if let Some(reason) = crate::cdb::status(root).stale_because {
                return Some(reason);
            }
            if !self.has_current_cdb(root) {
                return Some(
                    "a origem ou o conteúdo da CDB mudou; recarregue/reconfigure".to_owned(),
                );
            }
            return super::cdb::newer_cmake_input(root, Path::new(path), &self.units);
        }
        crate::cmake::incomplete_configure(root)
    }
}

pub(super) fn missing_compiler(compiler: &str, directory: &str) -> Option<String> {
    let path = Path::new(compiler);
    // Bare names depend on the producer's environment; do not guess its PATH.
    if compiler.starts_with('(') || (!path.is_absolute() && path.components().count() < 2) {
        return None;
    }
    let path = Path::new(directory).join(path);
    let missing = match std::fs::metadata(&path) {
        Ok(metadata) => !metadata.is_file(),
        Err(error) => error.kind() == io::ErrorKind::NotFound,
    };
    missing.then(|| {
        format!(
            "compilador ausente ou inválido no caminho registrado: {}",
            path.display()
        )
    })
}

//! Identidade por documento, independente da chave do processo clangd compartilhado.

use super::{manager::LspManager, registry::ServerSpec};
use crate::{index::ProjectIndex, lang::extract::SymbolExtractor};
use std::{
    path::Path,
    sync::{Arc, Mutex},
};

impl LspManager {
    pub(crate) fn use_document_context(&mut self, index: Arc<Mutex<ProjectIndex>>) {
        self.document_context = Some(index);
    }

    pub(super) fn document_language(&self, path: &Path) -> &'static str {
        if let Some(language) = self
            .document_context
            .as_ref()
            .and_then(|index| {
                let index = index.lock().ok()?;
                if self.root.as_deref() != Some(index.root.as_path()) {
                    return None;
                }
                index
                    .context
                    .as_ref()
                    .map(|context| context.document_language(path))
            })
            .filter(|language| matches!(*language, "c" | "cpp"))
        {
            return language;
        }
        SymbolExtractor::language_for(path).unwrap_or_else(|| {
            self.registry
                .spec_for_path(path)
                .map_or("plaintext", |spec| spec.language_id)
        })
    }

    pub(super) fn spec_for_document(&self, path: &Path) -> Option<ServerSpec> {
        let language = self.document_language(path);
        let mut spec = if matches!(language, "c" | "cpp") {
            self.registry.spec_for_language("cpp")?
        } else {
            self.registry.spec_for_path(path)?
        };
        spec.language_id = language;
        Some(spec)
    }
}

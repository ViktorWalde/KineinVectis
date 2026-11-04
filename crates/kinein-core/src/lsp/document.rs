//! Identidade por documento, independente da chave do processo clangd compartilhado.

use super::{manager::LspManager, registry::ServerSpec};
use crate::{
    index::{ProjectIndex, context::HeaderCommand},
    lang::extract::SymbolExtractor,
};
use std::{
    path::Path,
    sync::{Arc, Mutex},
};

impl LspManager {
    pub(super) fn header_command(&self, path: &Path) -> Option<HeaderCommand> {
        let index = self.document_context.as_ref()?.lock().ok()?;
        let root = self.root.as_deref()?;
        if index.root != root {
            return None;
        }
        index.context.as_ref()?.header_command(root, path)
    }

    /// A extensão clangd não remove overrides: retirar um exige novo processo.
    pub(super) fn prepare_document_server(
        &mut self,
        path: &Path,
        spec: &ServerSpec,
    ) -> Result<(), super::types::LspError> {
        let command = (spec.key == "cpp")
            .then(|| self.header_command(path))
            .flatten();
        if spec.key == "cpp"
            && command.is_none()
            && self
                .servers
                .get("cpp")
                .is_some_and(|handle| handle.header_commands.contains_key(path))
        {
            self.restart_language("cpp");
        }
        self.ensure_server(spec)?;
        let Some(command) = command else {
            return Ok(());
        };
        let Some(handle) = self
            .servers
            .get_mut("cpp")
            .filter(|handle| handle.is_ready())
        else {
            return Ok(());
        };
        if handle.header_commands.get(path) == Some(&command) {
            return Ok(());
        }
        super::sync::close_in(handle, &super::uri::uri_for_path(path));
        super::framing::send_notification(
            &handle.stdin,
            "workspace/didChangeConfiguration",
            &serde_json::json!({
                "settings": { "compilationDatabaseChanges": {
                    (path.display().to_string()): { "workingDirectory": command.directory, "compilationCommand": command.arguments }
                }}
            }),
        );
        handle.header_commands.insert(path.to_path_buf(), command);
        self.invalidate_code_actions();
        Ok(())
    }

    /// Revalida também headers inativos quando chega um contexto novo.
    pub(crate) fn refresh_header_context(&mut self) {
        let paths = self
            .servers
            .get("cpp")
            .map(|handle| handle.header_commands.keys().cloned().collect::<Vec<_>>())
            .unwrap_or_default();
        if paths.iter().any(|path| {
            self.servers
                .get("cpp")
                .and_then(|handle| handle.header_commands.get(path))
                != self.header_command(path).as_ref()
        }) {
            self.restart_language("cpp");
        }
    }

    pub(super) fn header_matches(&self, path: &Path) -> bool {
        self.servers
            .get("cpp")
            .and_then(|handle| handle.header_commands.get(path))
            == self.header_command(path).as_ref()
    }

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

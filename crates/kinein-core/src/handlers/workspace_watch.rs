//! O vigia de arquivos do workspace (`fswatch`).
//!
//! Troca o vigia quando um workspace abre e observa, sob demanda, as pastas
//! que o explorer, o editor e o indice alcancam. Mora junto dos handlers que o
//! usam (`workspace`, `fs_read`, `index`); o `Core` so' guarda o vigia.

use std::path::Path;

use kinein_protocol::JsonRpcRequest;
use serde_json::json;

use crate::{Core, fswatch};

impl Core {
    /// Replaces the watcher when a workspace opens. Failure is reported as an
    /// async warning, while compare-before-save remains active as the final
    /// protection against silent overwrites.
    pub(super) fn reset_workspace_watcher(&mut self, root: &Path) {
        self.fswatch = None;
        let Some(events) = self.events.as_ref() else {
            return;
        };
        match fswatch::WorkspaceWatcher::new(root, events.clone()) {
            Ok(watcher) => self.fswatch = Some(watcher),
            Err(error) => drop(events.send(JsonRpcRequest::notification(
                "event.fs.watchError",
                Some(json!({ "message": error.to_string() })),
            ))),
        }
    }

    /// Lazily observes a directory reached by the explorer, the editor — or
    /// the index, that registers every folder it walked. Returns `false` when
    /// the watcher refused (the error is reported once, here).
    pub(super) fn watch_workspace_directory(&mut self, directory: &Path) -> bool {
        let error = self
            .fswatch
            .as_mut()
            .and_then(|watcher| watcher.watch_directory(directory).err());
        let Some(error) = error else {
            return true;
        };
        if let Some(events) = self.events.as_ref() {
            drop(events.send(JsonRpcRequest::notification(
                "event.fs.watchError",
                Some(json!({ "message": error.to_string() })),
            )));
        }
        false
    }
}

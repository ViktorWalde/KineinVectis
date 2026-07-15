//! Handlers for `workspace.*` requests (`impl Core`). `workspace_root` stays in
//! `lib.rs` because every domain relies on it.

use std::path::{Path, PathBuf};

use kinein_protocol::{
    DraftInfo, JsonRpcError, JsonRpcErrorCode, JsonRpcResponse, RecentWorkspacePinParams,
    RecentWorkspaceRemoveParams, RecentWorkspacesParams, RecentWorkspacesResult,
    WorkspaceBrowseParams, WorkspaceCreateFolderParams, WorkspaceCreateFolderResult,
    WorkspaceCreateProjectParams, WorkspaceOpenParams, WorkspaceSaveSessionParams,
    WorkspaceSaveSessionResult,
};
use serde_json::{Value, json};

use crate::rpc::{
    fs_error_response, no_workspace_response, parse_params, workspace_error_response,
};
use crate::{Core, db, workspace};

fn recent_workspace_error_response(
    request_id: Option<Value>,
    error: &workspace::RecentWorkspaceError,
) -> JsonRpcResponse {
    let code = if error.is_invalid_params() {
        JsonRpcErrorCode::InvalidParams
    } else {
        JsonRpcErrorCode::InternalError
    };
    JsonRpcResponse::failure(request_id, JsonRpcError::new(code, error.to_string(), None))
}

impl Core {
    /// Routes the compact `workspace.recent.*` family outside the central
    /// composition root.
    pub(crate) fn recent_workspace_response(
        &self,
        method: &str,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> Option<JsonRpcResponse> {
        match method {
            "workspace.recent.list" => {
                Some(self.list_recent_workspaces_response(request_id, params))
            }
            "workspace.recent.pin" => Some(self.pin_recent_workspace_response(request_id, params)),
            "workspace.recent.remove" => {
                Some(self.remove_recent_workspace_response(request_id, params))
            }
            "workspace.recent.clear" => {
                Some(self.clear_recent_workspaces_response(request_id, params))
            }
            _ => None,
        }
    }

    fn recent_storage_unavailable_response(
        request_id: Option<Value>,
        method: &str,
    ) -> JsonRpcResponse {
        JsonRpcResponse::failure(
            request_id,
            JsonRpcError::new(
                JsonRpcErrorCode::InternalError,
                "storage global nao esta habilitado neste loop do core",
                Some(json!({ "method": method })),
            ),
        )
    }

    /// Lists the global recent-workspace snapshot. Lightweight in-process
    /// loops keep persistence disabled and therefore expose an empty list.
    pub(crate) fn list_recent_workspaces_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        if let Err(response) = parse_params::<RecentWorkspacesParams>(
            request_id.as_ref(),
            params,
            "workspace.recent.list nao aceita campos",
        ) {
            return *response;
        }
        let workspaces = if self.persistence_enabled {
            workspace::load_recent_workspaces()
        } else {
            Vec::new()
        };
        JsonRpcResponse::success(request_id, json!(RecentWorkspacesResult { workspaces }))
    }

    /// Changes one recent workspace's pinned state.
    pub(crate) fn pin_recent_workspace_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let parsed = match parse_params::<RecentWorkspacePinParams>(
            request_id.as_ref(),
            params,
            "workspace.recent.pin requer root e pinned",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        if !self.persistence_enabled {
            return Self::recent_storage_unavailable_response(request_id, "workspace.recent.pin");
        }
        match workspace::set_recent_workspace_pinned(&parsed.root, parsed.pinned) {
            Ok(workspaces) => {
                JsonRpcResponse::success(request_id, json!(RecentWorkspacesResult { workspaces }))
            }
            Err(error) => recent_workspace_error_response(request_id, &error),
        }
    }

    /// Removes one recent workspace, including a root missing from disk.
    pub(crate) fn remove_recent_workspace_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let parsed = match parse_params::<RecentWorkspaceRemoveParams>(
            request_id.as_ref(),
            params,
            "workspace.recent.remove requer root",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        if !self.persistence_enabled {
            return Self::recent_storage_unavailable_response(
                request_id,
                "workspace.recent.remove",
            );
        }
        match workspace::remove_recent_workspace(&parsed.root) {
            Ok(workspaces) => {
                JsonRpcResponse::success(request_id, json!(RecentWorkspacesResult { workspaces }))
            }
            Err(error) => recent_workspace_error_response(request_id, &error),
        }
    }

    /// Clears the complete global recent-workspace history.
    pub(crate) fn clear_recent_workspaces_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        if let Err(response) = parse_params::<RecentWorkspacesParams>(
            request_id.as_ref(),
            params,
            "workspace.recent.clear nao aceita campos",
        ) {
            return *response;
        }
        if !self.persistence_enabled {
            return Self::recent_storage_unavailable_response(request_id, "workspace.recent.clear");
        }
        match workspace::clear_recent_workspaces() {
            Ok(workspaces) => {
                JsonRpcResponse::success(request_id, json!(RecentWorkspacesResult { workspaces }))
            }
            Err(error) => recent_workspace_error_response(request_id, &error),
        }
    }

    /// Rascunhos que DIFEREM do disco (recuperáveis após um crash) e apaga os
    /// obsoletos (já salvos). Chamado no `workspace.open` (docs/23, M-S1).
    fn recover_drafts(&self) -> Vec<DraftInfo> {
        let Some(store) = self.drafts.as_ref() else {
            return Vec::new();
        };
        let Ok(drafts) = store.list() else {
            return Vec::new();
        };
        let mut recovered = Vec::new();
        for draft in drafts {
            let on_disk = std::fs::read_to_string(&draft.path).ok();
            if on_disk.as_deref() == Some(draft.content.as_str()) {
                // Buffer já salvo em disco → rascunho obsoleto, limpa.
                drop(store.clear(&draft.path));
            } else {
                recovered.push(DraftInfo {
                    path: draft.path,
                    content: draft.content,
                    saved_at: draft.saved_at,
                });
            }
        }
        recovered
    }

    /// Persiste as abas abertas/aba ativa em `.kinein/session.json`.
    pub(crate) fn save_session_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "workspace.saveSession");
        };
        let parsed = match parse_params::<WorkspaceSaveSessionParams>(
            request_id.as_ref(),
            params,
            "workspace.saveSession requer o campo openFiles",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        match workspace::save_session(&root, &parsed.open_files, parsed.active_file.as_deref()) {
            Ok(files) => {
                JsonRpcResponse::success(request_id, json!(WorkspaceSaveSessionResult { files }))
            }
            Err(error) => fs_error_response(request_id, &error),
        }
    }

    pub(crate) fn close_workspace_response(
        &mut self,
        request_id: Option<Value>,
    ) -> JsonRpcResponse {
        let closed = self.workspace.take();
        self.fswatch = None;
        self.syntax.clear();
        self.workspace_edits.clear();
        if let Some(lsp) = self.lsp.as_mut() {
            lsp.set_root(None);
        }
        if let Some(runner) = self.run.as_mut() {
            drop(runner.stop());
        }
        if let Some(session) = self.terminal.as_mut() {
            session.close_all();
        }
        JsonRpcResponse::success(
            request_id,
            json!({
                "status": "ok",
                "closed": closed.map(|workspace| workspace.root),
            }),
        )
    }

    pub(crate) fn browse_workspace_response(
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let params = params.cloned().unwrap_or_else(|| json!({}));
        match serde_json::from_value::<WorkspaceBrowseParams>(params) {
            Ok(params) => match workspace::browse_directories(Path::new(&params.path)) {
                Ok(result) => JsonRpcResponse::success(request_id, json!(result)),
                Err(error) => {
                    let code = if error.is_invalid_path() {
                        JsonRpcErrorCode::InvalidParams
                    } else {
                        JsonRpcErrorCode::InternalError
                    };
                    JsonRpcResponse::failure(
                        request_id,
                        JsonRpcError::new(
                            code,
                            error.to_string(),
                            Some(json!({ "path": params.path })),
                        ),
                    )
                }
            },
            Err(error) => JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(
                    JsonRpcErrorCode::InvalidParams,
                    "workspace.browse requer params com o campo path",
                    Some(json!({ "error": error.to_string() })),
                ),
            ),
        }
    }

    pub(crate) fn create_workspace_folder_response(
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let params = params.cloned().unwrap_or_else(|| json!({}));
        match serde_json::from_value::<WorkspaceCreateFolderParams>(params) {
            Ok(params) => {
                match workspace::create_directory(Path::new(&params.parent), &params.name) {
                    Ok(path) => JsonRpcResponse::success(
                        request_id,
                        json!(WorkspaceCreateFolderResult {
                            path: path.display().to_string(),
                        }),
                    ),
                    Err(error) => workspace_error_response(request_id, &error),
                }
            }
            Err(error) => JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(
                    JsonRpcErrorCode::InvalidParams,
                    "workspace.createFolder requer parent e name",
                    Some(json!({ "error": error.to_string() })),
                ),
            ),
        }
    }

    pub(crate) fn create_workspace_project_response(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let params = params.cloned().unwrap_or_else(|| json!({}));
        match serde_json::from_value::<WorkspaceCreateProjectParams>(params) {
            Ok(params) => match workspace::create_project(
                Path::new(&params.parent),
                &params.name,
                params.template,
            ) {
                Ok(opened) => {
                    self.workspace = Some(opened.clone());
                    self.syntax.clear();
                    self.workspace_edits.clear();
                    self.reset_workspace_watcher(Path::new(&opened.root));
                    if let Some(lsp) = self.lsp.as_mut() {
                        lsp.set_root(Some(PathBuf::from(&opened.root)));
                    }
                    if self.persistence_enabled {
                        drop(workspace::record_recent_workspace(&opened));
                    }
                    JsonRpcResponse::success(request_id, json!(opened))
                }
                Err(error) => workspace_error_response(request_id, &error),
            },
            Err(error) => JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(
                    JsonRpcErrorCode::InvalidParams,
                    "workspace.createProject requer parent, name e template",
                    Some(json!({ "error": error.to_string() })),
                ),
            ),
        }
    }

    pub(crate) fn open_workspace_response(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let params = params.cloned().unwrap_or_else(|| json!({}));
        match serde_json::from_value::<WorkspaceOpenParams>(params) {
            Ok(params) => match workspace::open_workspace(Path::new(&params.path)) {
                Ok(opened) => {
                    self.workspace = Some(opened.clone());
                    self.syntax.clear();
                    self.workspace_edits.clear();
                    self.reset_workspace_watcher(Path::new(&opened.root));
                    if let Some(lsp) = self.lsp.as_mut() {
                        lsp.set_root(Some(PathBuf::from(&opened.root)));
                    }
                    if self.persistence_enabled {
                        drop(workspace::record_recent_workspace(&opened));
                    }
                    // M-S1: abre a store local e recupera rascunhos não salvos
                    // (buffers que sobreviveram a um crash da UI — docs/23).
                    self.drafts = if self.persistence_enabled {
                        db::DraftStore::open(Path::new(&opened.root))
                    } else {
                        None
                    };
                    let recovered = self.recover_drafts();
                    let session = workspace::load_session(Path::new(&opened.root));
                    let mut result = json!(opened);
                    if let Some(map) = result.as_object_mut() {
                        if let Some(session) = session {
                            map.insert("session".to_owned(), json!(session));
                        }
                        if !recovered.is_empty() {
                            map.insert("drafts".to_owned(), json!(recovered));
                        }
                    }
                    JsonRpcResponse::success(request_id, result)
                }
                Err(error) => {
                    let code = if error.is_invalid_path() {
                        JsonRpcErrorCode::InvalidParams
                    } else {
                        JsonRpcErrorCode::InternalError
                    };
                    JsonRpcResponse::failure(
                        request_id,
                        JsonRpcError::new(
                            code,
                            error.to_string(),
                            Some(json!({ "path": params.path })),
                        ),
                    )
                }
            },
            Err(error) => JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(
                    JsonRpcErrorCode::InvalidParams,
                    "workspace.open requer params com o campo path",
                    Some(json!({ "error": error.to_string() })),
                ),
            ),
        }
    }
}

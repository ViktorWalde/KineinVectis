//! Handlers for `workspace.*` requests (`impl Core`). `workspace_root` stays in
//! `lib.rs` because every domain relies on it.

use std::path::{Path, PathBuf};

use kinein_protocol::{
    JsonRpcError, JsonRpcErrorCode, JsonRpcResponse, WorkspaceBrowseParams,
    WorkspaceCreateFolderParams, WorkspaceCreateFolderResult, WorkspaceCreateProjectParams,
    WorkspaceOpenParams,
};
use serde_json::{Value, json};

use crate::rpc::workspace_error_response;
use crate::{Core, workspace};

impl Core {
    pub(crate) fn close_workspace_response(
        &mut self,
        request_id: Option<Value>,
    ) -> JsonRpcResponse {
        let closed = self.workspace.take();
        if let Some(lsp) = self.lsp.as_mut() {
            lsp.set_root(None);
        }
        if let Some(runner) = self.run.as_mut() {
            drop(runner.stop());
        }
        if let Some(session) = self.terminal.as_mut() {
            drop(session.close());
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
                    if let Some(lsp) = self.lsp.as_mut() {
                        lsp.set_root(Some(PathBuf::from(&opened.root)));
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
                    if let Some(lsp) = self.lsp.as_mut() {
                        lsp.set_root(Some(PathBuf::from(&opened.root)));
                    }
                    JsonRpcResponse::success(request_id, json!(opened))
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

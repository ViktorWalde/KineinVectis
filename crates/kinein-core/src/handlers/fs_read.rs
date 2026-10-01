//! Directory listing and bounded text reads for workspace and external preview.

use std::path::Path;

use kinein_protocol::{FsListResult, FsPathParams, FsReadResult, JsonRpcResponse};
use serde_json::{Value, json};

use crate::rpc::{fs_error_response, no_workspace_response, parse_params};
use crate::{Core, fsops};

impl Core {
    pub(super) fn fs_list_response(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "fs.list");
        };
        match parse_params::<FsPathParams>(
            request_id.as_ref(),
            params,
            "fs.list requer o campo path",
        ) {
            Ok(parsed) => match fsops::list_dir(&root, Path::new(&parsed.path)) {
                Ok((path, entries)) => {
                    self.watch_workspace_directory(&path);
                    JsonRpcResponse::success(
                        request_id,
                        json!(FsListResult {
                            path: path.display().to_string(),
                            entries,
                        }),
                    )
                }
                Err(error) => fs_error_response(request_id, &error),
            },
            Err(response) => *response,
        }
    }

    pub(super) fn fs_read_response(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "fs.read");
        };
        match parse_params::<FsPathParams>(
            request_id.as_ref(),
            params,
            "fs.read requer o campo path",
        ) {
            Ok(parsed) => match fsops::read_file(&root, Path::new(&parsed.path)) {
                Ok((path, content)) => {
                    if let Some(parent) = path.parent() {
                        self.watch_workspace_directory(parent);
                    }
                    if let Some(lsp) = self.lsp.as_mut() {
                        lsp.did_open(&path, &content);
                    }
                    JsonRpcResponse::success(
                        request_id,
                        json!(FsReadResult {
                            path: path.display().to_string(),
                            content,
                        }),
                    )
                }
                Err(error) => fs_error_response(request_id, &error),
            },
            Err(response) => *response,
        }
    }

    pub(super) fn fs_read_external_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        if self.workspace_root().is_none() {
            return no_workspace_response(request_id, "fs.readExternal");
        }
        match parse_params::<FsPathParams>(
            request_id.as_ref(),
            params,
            "fs.readExternal requer o campo path",
        ) {
            Ok(parsed) => match fsops::read_external_file(Path::new(&parsed.path)) {
                Ok((path, content)) => JsonRpcResponse::success(
                    request_id,
                    json!(FsReadResult {
                        path: path.display().to_string(),
                        content,
                    }),
                ),
                Err(error) => fs_error_response(request_id, &error),
            },
            Err(response) => *response,
        }
    }
}

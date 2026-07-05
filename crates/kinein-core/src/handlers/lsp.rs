//! Handlers for `lsp.*` requests (`impl Core`).
//!
//! The `lsp.*` router plus the didChange/semanticTokens/definition/hover/
//! completion/references/rename leaf handlers, delegating to the language
//! servers managed by `crate::lsp`.

use std::path::{Path, PathBuf};

use kinein_protocol::{
    FsWriteParams, JsonRpcError, JsonRpcErrorCode, JsonRpcResponse, LspCompletionResult,
    LspDefinitionResult, LspHoverResult, LspReferenceItem, LspReferencesResult, LspRenameParams,
    LspRenameResult, LspSemanticTokensResult,
};
use serde_json::{Value, json};

use crate::rpc::{
    fs_error_response, lsp_error_response, lsp_unavailable_response, no_workspace_response,
    parse_lsp_position_params, parse_params,
};
use crate::{Core, fsops, lsp};

impl Core {
    /// Roteia os metodos `lsp.*`; `None` quando o metodo nao e LSP.
    pub(crate) fn lsp_request_response(
        &mut self,
        method: &str,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> Option<JsonRpcResponse> {
        match method {
            "lsp.didChange" => Some(self.lsp_did_change_response(request_id, params)),
            "lsp.semanticTokens" => Some(self.lsp_semantic_tokens_response(request_id, params)),
            "lsp.definition" => Some(self.lsp_definition_response(request_id, params)),
            "lsp.hover" => Some(self.lsp_hover_response(request_id, params)),
            "lsp.completion" => Some(self.lsp_completion_response(request_id, params)),
            "lsp.references" => Some(self.lsp_references_response(request_id, params)),
            "lsp.rename" => Some(self.lsp_rename_response(request_id, params)),
            _ => None,
        }
    }

    fn lsp_did_change_response(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "lsp.didChange");
        };
        match parse_params::<FsWriteParams>(
            request_id.as_ref(),
            params,
            "lsp.didChange requer os campos path e content",
        ) {
            Ok(parsed) => {
                let path = Path::new(&parsed.path);
                let canonical = match fsops::confine_file(&root, path) {
                    Ok(canonical) => canonical,
                    Err(error) => return fs_error_response(request_id, &error),
                };
                if let Some(lsp) = self.lsp.as_mut() {
                    lsp.did_change(&canonical, &parsed.content);
                }
                JsonRpcResponse::success(request_id, json!({ "status": "ok" }))
            }
            Err(response) => *response,
        }
    }

    fn lsp_semantic_tokens_response(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "lsp.semanticTokens");
        };
        match parse_params::<FsWriteParams>(
            request_id.as_ref(),
            params,
            "lsp.semanticTokens requer os campos path e content",
        ) {
            Ok(parsed) => {
                let path = match fsops::confine_file(&root, Path::new(&parsed.path)) {
                    Ok(canonical) => canonical,
                    Err(error) => return fs_error_response(request_id, &error),
                };
                let Some(lsp) = self.lsp.as_mut() else {
                    return lsp_unavailable_response(request_id, "lsp.semanticTokens");
                };
                match lsp.semantic_tokens(&path, &parsed.content) {
                    Ok(tokens) => JsonRpcResponse::success(
                        request_id,
                        json!(LspSemanticTokensResult { tokens }),
                    ),
                    Err(error) => lsp_error_response(request_id, &error),
                }
            }
            Err(response) => *response,
        }
    }

    fn lsp_definition_response(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "lsp.definition");
        };
        let (path, parsed) = match parse_lsp_position_params(
            &root,
            request_id.as_ref(),
            params,
            "lsp.definition requer path, content, line e column",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let Some(lsp) = self.lsp.as_mut() else {
            return lsp_unavailable_response(request_id, "lsp.definition");
        };
        match lsp.definition(&path, &parsed.content, parsed.line, parsed.column) {
            Ok(location) => JsonRpcResponse::success(
                request_id,
                json!(LspDefinitionResult {
                    path: location.as_ref().map(|target| target.path.clone()),
                    line: location.as_ref().map(|target| target.line),
                    column: location.as_ref().map(|target| target.column),
                }),
            ),
            Err(error) => lsp_error_response(request_id, &error),
        }
    }

    fn lsp_hover_response(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "lsp.hover");
        };
        let (path, parsed) = match parse_lsp_position_params(
            &root,
            request_id.as_ref(),
            params,
            "lsp.hover requer path, content, line e column",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let Some(lsp) = self.lsp.as_mut() else {
            return lsp_unavailable_response(request_id, "lsp.hover");
        };
        match lsp.hover(&path, &parsed.content, parsed.line, parsed.column) {
            Ok(content) => JsonRpcResponse::success(request_id, json!(LspHoverResult { content })),
            Err(error) => lsp_error_response(request_id, &error),
        }
    }

    fn lsp_completion_response(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "lsp.completion");
        };
        let (path, parsed) = match parse_lsp_position_params(
            &root,
            request_id.as_ref(),
            params,
            "lsp.completion requer path, content, line e column",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let Some(lsp) = self.lsp.as_mut() else {
            return lsp_unavailable_response(request_id, "lsp.completion");
        };
        match lsp.completion(&path, &parsed.content, parsed.line, parsed.column) {
            Ok(items) => JsonRpcResponse::success(request_id, json!(LspCompletionResult { items })),
            Err(error) => lsp_error_response(request_id, &error),
        }
    }

    fn lsp_references_response(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "lsp.references");
        };
        let (path, parsed) = match parse_lsp_position_params(
            &root,
            request_id.as_ref(),
            params,
            "lsp.references requer path, content, line e column",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let Some(lsp) = self.lsp.as_mut() else {
            return lsp_unavailable_response(request_id, "lsp.references");
        };
        match lsp.references(&path, &parsed.content, parsed.line, parsed.column) {
            Ok(locations) => {
                let references = locations
                    .into_iter()
                    .map(|location| LspReferenceItem {
                        path: location.path,
                        line: location.line,
                        column: location.column,
                    })
                    .collect();
                JsonRpcResponse::success(request_id, json!(LspReferencesResult { references }))
            }
            Err(error) => lsp_error_response(request_id, &error),
        }
    }

    /// Executa `lsp.rename` fim a fim: consulta o servidor, valida que todos
    /// os arquivos afetados estao dentro do workspace, aplica os edits em
    /// memoria e so entao reescreve os arquivos no disco.
    fn lsp_rename_response(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "lsp.rename");
        };
        let parsed = match parse_params::<LspRenameParams>(
            request_id.as_ref(),
            params,
            "lsp.rename requer path, content, line, column e newName",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        if parsed.new_name.trim().is_empty() {
            return JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(
                    JsonRpcErrorCode::InvalidParams,
                    "lsp.rename requer um novo nome nao vazio",
                    None,
                ),
            );
        }
        let active_path = match fsops::confine_file(&root, Path::new(&parsed.path)) {
            Ok(path) => path,
            Err(error) => return fs_error_response(request_id, &error),
        };
        let Some(lsp) = self.lsp.as_mut() else {
            return lsp_unavailable_response(request_id, "lsp.rename");
        };
        let plan = match lsp.rename(
            &active_path,
            &parsed.content,
            parsed.line,
            parsed.column,
            parsed.new_name.trim(),
        ) {
            Ok(plan) => plan,
            Err(error) => return lsp_error_response(request_id, &error),
        };

        let edits = plan.edit_count();
        let mut updates: Vec<(PathBuf, String)> = Vec::with_capacity(plan.files.len());
        for file in &plan.files {
            let target = match fsops::confine_file(&root, Path::new(&file.path)) {
                Ok(target) => target,
                Err(error) => return fs_error_response(request_id, &error),
            };
            let base = if target == active_path {
                parsed.content.clone()
            } else {
                match fsops::read_file(&root, &target) {
                    Ok((_, content)) => content,
                    Err(error) => return fs_error_response(request_id, &error),
                }
            };
            let rewritten = match lsp::apply_text_edits(&base, &file.edits) {
                Ok(rewritten) => rewritten,
                Err(error) => return lsp_error_response(request_id, &error),
            };
            updates.push((target, rewritten));
        }

        let mut files = Vec::with_capacity(updates.len());
        for (target, content) in &updates {
            if let Err(error) = fsops::write_file(&root, target, content) {
                return fs_error_response(request_id, &error);
            }
            files.push(target.display().to_string());
        }
        for (target, content) in &updates {
            lsp.sync_if_open(target, content);
        }

        JsonRpcResponse::success(request_id, json!(LspRenameResult { files, edits }))
    }
}

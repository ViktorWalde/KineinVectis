//! Operacoes interativas de consulta: definicao, hover, completion,
//! referencias, simbolos e switch source/header.
//! Par de `crate::lsp::manager`.

use std::path::Path;

use kinein_protocol::{
    FsWriteParams, JsonRpcError, JsonRpcErrorCode, JsonRpcResponse, LspCompletionResult,
    LspDefinitionResult, LspHoverResult, LspReferenceItem, LspReferencesResult,
    LspSwitchSourceHeaderResult, LspSymbolsResult, LspWorkspaceSymbolsParams,
};
use serde_json::{Value, json};

use crate::Core;
use crate::fsops;
use crate::rpc::{
    fs_error_response, lsp_error_response, lsp_unavailable_response, no_workspace_response,
    parse_lsp_position_params, parse_params,
};

impl Core {
    pub(super) fn lsp_definition_response(
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

    pub(super) fn lsp_hover_response(
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

    pub(super) fn lsp_completion_response(
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
            Ok((items, is_incomplete)) => JsonRpcResponse::success(
                request_id,
                json!(LspCompletionResult {
                    items,
                    is_incomplete,
                }),
            ),
            Err(error) => lsp_error_response(request_id, &error),
        }
    }

    pub(super) fn lsp_references_response(
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

    /// Estrutura achatada de simbolos do arquivo ativo.
    pub(super) fn lsp_document_symbols_response(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "lsp.documentSymbols");
        };
        match parse_params::<FsWriteParams>(
            request_id.as_ref(),
            params,
            "lsp.documentSymbols requer os campos path e content",
        ) {
            Ok(parsed) => {
                let path = match fsops::confine_file(&root, Path::new(&parsed.path)) {
                    Ok(canonical) => canonical,
                    Err(error) => return fs_error_response(request_id, &error),
                };
                let Some(lsp) = self.lsp.as_mut() else {
                    return lsp_unavailable_response(request_id, "lsp.documentSymbols");
                };
                match lsp.document_symbols(&path, &parsed.content) {
                    Ok(symbols) => {
                        JsonRpcResponse::success(request_id, json!(LspSymbolsResult { symbols }))
                    }
                    Err(error) => lsp_error_response(request_id, &error),
                }
            }
            Err(response) => *response,
        }
    }

    /// Busca de simbolos no workspace pelo servidor do arquivo ativo.
    pub(super) fn lsp_workspace_symbols_response(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "lsp.workspaceSymbols");
        };
        let parsed = match parse_params::<LspWorkspaceSymbolsParams>(
            request_id.as_ref(),
            params,
            "lsp.workspaceSymbols requer path, content e query",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        if parsed.query.trim().is_empty() {
            return JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(
                    JsonRpcErrorCode::InvalidParams,
                    "lsp.workspaceSymbols requer uma query nao vazia",
                    None,
                ),
            );
        }
        let path = match fsops::confine_file(&root, Path::new(&parsed.path)) {
            Ok(canonical) => canonical,
            Err(error) => return fs_error_response(request_id, &error),
        };
        let Some(lsp) = self.lsp.as_mut() else {
            return lsp_unavailable_response(request_id, "lsp.workspaceSymbols");
        };
        match lsp.workspace_symbols(&path, &parsed.content, parsed.query.trim()) {
            Ok(symbols) => {
                JsonRpcResponse::success(request_id, json!(LspSymbolsResult { symbols }))
            }
            Err(error) => lsp_error_response(request_id, &error),
        }
    }

    /// Alterna header/source (clangd) para o arquivo ativo.
    pub(super) fn lsp_switch_source_header_response(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "lsp.switchSourceHeader");
        };
        let parsed = match parse_params::<FsWriteParams>(
            request_id.as_ref(),
            params,
            "lsp.switchSourceHeader requer os campos path e content",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let path = match fsops::confine_file(&root, Path::new(&parsed.path)) {
            Ok(canonical) => canonical,
            Err(error) => return fs_error_response(request_id, &error),
        };
        let Some(lsp) = self.lsp.as_mut() else {
            return lsp_unavailable_response(request_id, "lsp.switchSourceHeader");
        };
        match lsp.switch_source_header(&path, &parsed.content) {
            Ok(path) => {
                JsonRpcResponse::success(request_id, json!(LspSwitchSourceHeaderResult { path }))
            }
            Err(error) => lsp_error_response(request_id, &error),
        }
    }
}

//! Operacoes interativas de consulta: definicao, hover, completion,
//! referencias, simbolos e switch source/header.
//! Par de `crate::lsp::manager`.

use std::path::Path;

use kinein_protocol::{
    FsWriteParams, JsonRpcError, JsonRpcErrorCode, JsonRpcResponse, LspSwitchSourceHeaderResult,
    LspWorkspaceSymbolsParams,
};
use serde_json::{Value, json};

use crate::Core;
use crate::fsops;
use crate::lsp::LspQuery;
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
        // Adiada (Etapa 2 F6): o laco nao espera o servidor.
        match lsp.begin_query(
            LspQuery::Definition {
                line: parsed.line,
                column: parsed.column,
            },
            &path,
            &parsed.content,
        ) {
            Ok(begun) => self.defer_lsp(request_id, begun),
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
        // Adiada (Etapa 2 F6): o laco nao espera o servidor.
        match lsp.begin_query(
            LspQuery::Hover {
                line: parsed.line,
                column: parsed.column,
            },
            &path,
            &parsed.content,
        ) {
            Ok(begun) => self.defer_lsp(request_id, begun),
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
        // Adiada (Etapa 2 F6): o laco nao espera o servidor.
        match lsp.begin_query(
            LspQuery::Completion {
                line: parsed.line,
                column: parsed.column,
            },
            &path,
            &parsed.content,
        ) {
            Ok(begun) => self.defer_lsp(request_id, begun),
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
        // Adiada (Etapa 2 F6): o laco nao espera o servidor.
        match lsp.begin_query(
            LspQuery::References {
                line: parsed.line,
                column: parsed.column,
            },
            &path,
            &parsed.content,
        ) {
            Ok(begun) => self.defer_lsp(request_id, begun),
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
                match lsp.begin_query(LspQuery::DocumentSymbols, &path, &parsed.content) {
                    Ok(begun) => self.defer_lsp(request_id, begun),
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
        let query = LspQuery::WorkspaceSymbols {
            query: parsed.query.trim().to_owned(),
        };
        match lsp.begin_query(query, &path, &parsed.content) {
            Ok(begun) => self.defer_lsp(request_id, begun),
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

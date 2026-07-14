//! Handlers for `lsp.*` requests (`impl Core`).
//!
//! The `lsp.*` router plus the didChange/semanticTokens/definition/hover/
//! completion/references/rename leaf handlers, delegating to the language
//! servers managed by `crate::lsp`.

use std::path::Path;

use kinein_protocol::{
    FsWriteParams, JsonRpcError, JsonRpcErrorCode, JsonRpcResponse, LspCodeActionsResult,
    LspCompletionResult, LspDefinitionResult, LspHoverResult, LspReferenceItem,
    LspReferencesResult, LspRenameParams, LspRestartParams, LspRestartResult,
    LspSemanticTokensResult, LspSwitchSourceHeaderResult, LspSymbolsResult,
    LspWorkspaceEditApplyResult, LspWorkspaceEditCancelResult, LspWorkspaceEditTransactionParams,
    LspWorkspaceSymbolsParams,
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
            "lsp.codeActions" => Some(self.lsp_code_actions_response(request_id, params)),
            "lsp.applyCodeAction" => Some(self.lsp_apply_code_action_response(request_id, params)),
            "lsp.workspaceEdit.apply" => {
                Some(self.lsp_workspace_edit_apply_response(request_id, params))
            }
            "lsp.workspaceEdit.cancel" => {
                Some(self.lsp_workspace_edit_cancel_response(request_id, params))
            }
            "lsp.documentSymbols" => Some(self.lsp_document_symbols_response(request_id, params)),
            "lsp.workspaceSymbols" => Some(self.lsp_workspace_symbols_response(request_id, params)),
            "lsp.switchSourceHeader" => {
                Some(self.lsp_switch_source_header_response(request_id, params))
            }
            "lsp.restart" => Some(self.lsp_restart_response(request_id, params)),
            _ => None,
        }
    }

    /// Reinicia servidor(es) LSP (M4.3b): `{ language? }` — sem language,
    /// reinicia todos os vivos. Cada um sobe de novo (lazy) no próximo
    /// request; a UI re-sincroniza o arquivo ativo ao ver `event.lsp.restarted`.
    fn lsp_restart_response(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        // `params` ausente = reiniciar todos (language None).
        let language = match params {
            Some(value) => match parse_params::<LspRestartParams>(
                request_id.as_ref(),
                Some(value),
                "lsp.restart aceita o campo opcional language",
            ) {
                Ok(parsed) => parsed.language,
                Err(response) => return *response,
            },
            None => None,
        };
        let Some(lsp) = self.lsp.as_mut() else {
            return lsp_unavailable_response(request_id, "lsp.restart");
        };
        let restarted = match language {
            Some(language) => {
                if lsp.restart_language(&language) {
                    vec![language]
                } else {
                    Vec::new()
                }
            }
            None => lsp.restart_all(),
        };
        JsonRpcResponse::success(request_id, json!(LspRestartResult { restarted }))
    }

    /// Alterna header/source (clangd) para o arquivo ativo.
    fn lsp_switch_source_header_response(
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

    /// Estrutura achatada de simbolos do arquivo ativo.
    fn lsp_document_symbols_response(
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
    fn lsp_workspace_symbols_response(
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

    /// Consulta `textDocument/rename` e prepara uma previa confirmavel. Nenhum
    /// arquivo e alterado nesta etapa.
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

        if let Err(error) = lsp::WorkspaceEditTransactions::validate_versions(&plan, |path| {
            lsp.document_version(path)
        }) {
            return workspace_edit_error_response(request_id, &error);
        }
        let title = format!("Renomear para {}", parsed.new_name.trim());
        let preview =
            match self
                .workspace_edits
                .prepare(&root, &active_path, &parsed.content, &plan, title)
            {
                Ok(preview) => preview,
                Err(error) => return workspace_edit_error_response(request_id, &error),
            };

        JsonRpcResponse::success(request_id, json!(preview))
    }

    /// Lista as code actions aplicaveis no ponto do cursor.
    fn lsp_code_actions_response(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "lsp.codeActions");
        };
        let (path, parsed) = match parse_lsp_position_params(
            &root,
            request_id.as_ref(),
            params,
            "lsp.codeActions requer path, content, line e column",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let Some(lsp) = self.lsp.as_mut() else {
            return lsp_unavailable_response(request_id, "lsp.codeActions");
        };
        match lsp.code_actions(&path, &parsed.content, parsed.line, parsed.column) {
            Ok(actions) => {
                JsonRpcResponse::success(request_id, json!(LspCodeActionsResult { actions }))
            }
            Err(error) => lsp_error_response(request_id, &error),
        }
    }

    /// Converte uma acao da ultima consulta em previa confirmavel. Nenhuma
    /// escrita acontece antes de `lsp.workspaceEdit.apply`.
    fn lsp_apply_code_action_response(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "lsp.applyCodeAction");
        };
        let parsed = match parse_params::<kinein_protocol::LspApplyCodeActionParams>(
            request_id.as_ref(),
            params,
            "lsp.applyCodeAction requer path, content e actionIndex",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let active_path = match fsops::confine_file(&root, Path::new(&parsed.path)) {
            Ok(path) => path,
            Err(error) => return fs_error_response(request_id, &error),
        };
        let Some(lsp) = self.lsp.as_mut() else {
            return lsp_unavailable_response(request_id, "lsp.applyCodeAction");
        };
        let index = usize::try_from(parsed.action_index).unwrap_or(usize::MAX);
        let Some(taken) = lsp.take_code_action(&active_path, index) else {
            return JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(
                    JsonRpcErrorCode::InvalidParams,
                    "a consulta de code actions expirou ou o indice e invalido; \
                     peca as acoes novamente",
                    Some(json!({ "path": parsed.path, "actionIndex": parsed.action_index })),
                ),
            );
        };
        let (title, plan) = match taken {
            Ok(pair) => pair,
            Err(error) => return lsp_error_response(request_id, &error),
        };

        if let Err(error) = lsp::WorkspaceEditTransactions::validate_versions(&plan, |path| {
            lsp.document_version(path)
        }) {
            return workspace_edit_error_response(request_id, &error);
        }
        let preview =
            match self
                .workspace_edits
                .prepare(&root, &active_path, &parsed.content, &plan, title)
            {
                Ok(preview) => preview,
                Err(error) => return workspace_edit_error_response(request_id, &error),
            };

        JsonRpcResponse::success(request_id, json!(preview))
    }

    /// Confirma e consome uma transacao previamente apresentada pela UI.
    fn lsp_workspace_edit_apply_response(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "lsp.workspaceEdit.apply");
        };
        let parsed = match parse_params::<LspWorkspaceEditTransactionParams>(
            request_id.as_ref(),
            params,
            "lsp.workspaceEdit.apply requer transactionId",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let applied = match self.workspace_edits.apply(&root, &parsed.transaction_id) {
            Ok(applied) => applied,
            Err(error) => return workspace_edit_error_response(request_id, &error),
        };
        if let Some(lsp) = self.lsp.as_mut() {
            for (path, content) in &applied.files {
                lsp.sync_if_open(path, content);
            }
        }
        self.syntax.clear();
        let files = applied
            .files
            .iter()
            .map(|(path, _)| path.display().to_string())
            .collect();
        JsonRpcResponse::success(
            request_id,
            json!(LspWorkspaceEditApplyResult {
                transaction_id: applied.transaction_id,
                title: applied.title,
                files,
                edits: applied.edits,
            }),
        )
    }

    /// Descarta uma transacao pendente sem tocar no disco.
    fn lsp_workspace_edit_cancel_response(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        if self.workspace_root().is_none() {
            return no_workspace_response(request_id, "lsp.workspaceEdit.cancel");
        }
        let parsed = match parse_params::<LspWorkspaceEditTransactionParams>(
            request_id.as_ref(),
            params,
            "lsp.workspaceEdit.cancel requer transactionId",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        if let Err(error) = self.workspace_edits.cancel(&parsed.transaction_id) {
            return workspace_edit_error_response(request_id, &error);
        }
        JsonRpcResponse::success(
            request_id,
            json!(LspWorkspaceEditCancelResult {
                transaction_id: parsed.transaction_id,
                cancelled: true,
            }),
        )
    }
}

fn workspace_edit_error_response(
    request_id: Option<Value>,
    error: &lsp::WorkspaceEditTransactionError,
) -> JsonRpcResponse {
    match error {
        lsp::WorkspaceEditTransactionError::FileSystem(error) => {
            fs_error_response(request_id, error)
        }
        lsp::WorkspaceEditTransactionError::Edit(error) => lsp_error_response(request_id, error),
        lsp::WorkspaceEditTransactionError::StaleVersion {
            path,
            expected,
            actual,
        } => JsonRpcResponse::failure(
            request_id,
            JsonRpcError::new(
                JsonRpcErrorCode::FileChanged,
                error.to_string(),
                Some(json!({ "path": path, "expectedVersion": expected, "actualVersion": actual })),
            ),
        ),
        lsp::WorkspaceEditTransactionError::UnknownTransaction { id } => JsonRpcResponse::failure(
            request_id,
            JsonRpcError::new(
                JsonRpcErrorCode::InvalidParams,
                error.to_string(),
                Some(json!({ "transactionId": id })),
            ),
        ),
        lsp::WorkspaceEditTransactionError::InvalidPlan { .. } => JsonRpcResponse::failure(
            request_id,
            JsonRpcError::new(JsonRpcErrorCode::InvalidParams, error.to_string(), None),
        ),
    }
}

//! Transacao de `WorkspaceEdit`: rename, code actions e o apply/cancel do
//! preview. Par de `crate::lsp::transaction`.

use std::path::Path;

use kinein_protocol::{
    JsonRpcError, JsonRpcErrorCode, JsonRpcResponse, LspCodeActionsResult, LspRenameParams,
    LspWorkspaceEditApplyResult, LspWorkspaceEditCancelResult, LspWorkspaceEditTransactionParams,
};
use serde_json::{Value, json};

use crate::rpc::{
    fs_error_response, lsp_error_response, lsp_unavailable_response, no_workspace_response,
    parse_lsp_position_params, parse_params,
};
use crate::{Core, fsops, lsp};

impl Core {
    /// A cauda que `rename` e `applyCodeAction` compartilham: valida as versoes
    /// do plano contra os documentos abertos e prepara a transacao para a UI
    /// confirmar. Estava escrita DUAS vezes; um plano que nasce de origens
    /// diferentes tem que virar preview pelo MESMO caminho, senao uma das duas
    /// pode divergir na validacao — que e' a barreira de versao.
    fn workspace_edit_preview_response(
        &mut self,
        request_id: Option<Value>,
        root: &Path,
        active_path: &Path,
        active_content: &str,
        plan: &lsp::WorkspaceEditPlan,
        title: String,
    ) -> JsonRpcResponse {
        let Some(lsp) = self.lsp.as_ref() else {
            return lsp_unavailable_response(request_id, "lsp.workspaceEdit");
        };
        if let Err(error) = lsp::WorkspaceEditTransactions::validate_versions(plan, |path| {
            lsp.document_version(path)
        }) {
            return workspace_edit_error_response(request_id, &error);
        }
        match self
            .workspace_edits
            .prepare(root, active_path, active_content, plan, title)
        {
            Ok(preview) => JsonRpcResponse::success(request_id, json!(preview)),
            Err(error) => workspace_edit_error_response(request_id, &error),
        }
    }
    /// Consulta `textDocument/rename` e prepara uma previa confirmavel. Nenhum
    /// arquivo e alterado nesta etapa.
    pub(super) fn lsp_rename_response(
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

        let title = format!("Renomear para {}", parsed.new_name.trim());
        self.workspace_edit_preview_response(
            request_id,
            &root,
            &active_path,
            &parsed.content,
            &plan,
            title,
        )
    }

    /// Lista as code actions aplicaveis no ponto do cursor.
    pub(super) fn lsp_code_actions_response(
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
    pub(super) fn lsp_apply_code_action_response(
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

        self.workspace_edit_preview_response(
            request_id,
            &root,
            &active_path,
            &parsed.content,
            &plan,
            title,
        )
    }

    /// Confirma e consome uma transacao previamente apresentada pela UI.
    pub(super) fn lsp_workspace_edit_apply_response(
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
    pub(super) fn lsp_workspace_edit_cancel_response(
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

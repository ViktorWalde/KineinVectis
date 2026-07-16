//! Handlers de `draft.*` (autosave/rascunhos) — rede de segurança (docs/seguranca/23).
//!
//! `draft.save { path, content }` persiste o buffer não salvo de um arquivo
//! na store `SQLite`; `draft.clear { path }` remove o rascunho (save/close
//! limpo). A recuperação após crash sai na resposta de `workspace.open`. O
//! path é confinado à raiz (como `fs.*`); a chave é o caminho absoluto.

use std::path::Path;

use kinein_protocol::{
    DraftSaveResult, FsPathParams, FsWriteParams, JsonRpcError, JsonRpcErrorCode, JsonRpcResponse,
};
use serde_json::{Value, json};

use crate::rpc::{fs_error_response, no_workspace_response, parse_params};
use crate::{Core, fsops};

impl Core {
    /// Roteia `draft.*`; `None` quando não é de draft.
    pub(crate) fn draft_request_response(
        &self,
        method: &str,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> Option<JsonRpcResponse> {
        match method {
            "draft.save" => Some(self.draft_save_response(request_id, params)),
            "draft.clear" => Some(self.draft_clear_response(request_id, params)),
            _ => None,
        }
    }

    fn draft_save_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "draft.save");
        };
        let parsed = match parse_params::<FsWriteParams>(
            request_id.as_ref(),
            params,
            "draft.save requer os campos path e content",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let path = match fsops::confine_file(&root, Path::new(&parsed.path)) {
            Ok(canonical) => canonical,
            Err(error) => return fs_error_response(request_id, &error),
        };
        let Some(store) = self.drafts.as_ref() else {
            return drafts_unavailable_response(request_id);
        };
        match store.save(&path.display().to_string(), &parsed.content) {
            Ok(saved_at) => {
                JsonRpcResponse::success(request_id, json!(DraftSaveResult { saved_at }))
            }
            Err(error) => drafts_error_response(request_id, &error.to_string()),
        }
    }

    fn draft_clear_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "draft.clear");
        };
        let parsed = match parse_params::<FsPathParams>(
            request_id.as_ref(),
            params,
            "draft.clear requer o campo path",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let path = match fsops::confine_file(&root, Path::new(&parsed.path)) {
            Ok(canonical) => canonical,
            Err(error) => return fs_error_response(request_id, &error),
        };
        let Some(store) = self.drafts.as_ref() else {
            return drafts_unavailable_response(request_id);
        };
        match store.clear(&path.display().to_string()) {
            Ok(()) => JsonRpcResponse::success(request_id, json!({ "ok": true })),
            Err(error) => drafts_error_response(request_id, &error.to_string()),
        }
    }
}

/// Limpa o rascunho de um arquivo recém-salvo (chamado pelo `fs.write`). É
/// best-effort: se a store estiver ausente ou falhar, não afeta o save.
pub(super) fn clear_draft_after_save(core: &Core, path: &Path) {
    if let Some(store) = core.drafts.as_ref() {
        drop(store.clear(&path.display().to_string()));
    }
}

fn drafts_unavailable_response(request_id: Option<Value>) -> JsonRpcResponse {
    JsonRpcResponse::failure(
        request_id,
        JsonRpcError::new(
            JsonRpcErrorCode::InternalError,
            "persistência local de rascunhos indisponível",
            None,
        ),
    )
}

fn drafts_error_response(request_id: Option<Value>, message: &str) -> JsonRpcResponse {
    JsonRpcResponse::failure(
        request_id,
        JsonRpcError::new(
            JsonRpcErrorCode::InternalError,
            format!("falha na store de rascunhos: {message}"),
            None,
        ),
    )
}

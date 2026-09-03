//! O TEXTO do documento: `lsp.didChange` e `lsp.semanticTokens`.
//! Par de `crate::lsp::sync`.

use std::path::Path;

use kinein_protocol::{
    FsWriteParams, JsonRpcResponse, LspSemanticTokensParams, LspSemanticTokensResult,
};
use serde_json::{Value, json};

use crate::Core;
use crate::fsops;
use crate::rpc::{
    fs_error_response, lsp_error_response, lsp_unavailable_response, no_workspace_response,
    parse_params,
};

impl Core {
    pub(super) fn lsp_did_change_response(
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

    pub(super) fn lsp_semantic_tokens_response(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "lsp.semanticTokens");
        };
        match parse_params::<LspSemanticTokensParams>(
            request_id.as_ref(),
            params,
            "lsp.semanticTokens requer path, content e version",
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
                        json!(LspSemanticTokensResult {
                            path: path.to_string_lossy().into_owned(),
                            version: parsed.version,
                            tokens,
                        }),
                    ),
                    Err(error) => lsp_error_response(request_id, &error),
                }
            }
            Err(response) => *response,
        }
    }
}

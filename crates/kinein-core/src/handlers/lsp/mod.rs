//! Handlers for `lsp.*` requests (`impl Core`).
//!
//! Pasta desde 2026-09-03 (etapa 13 do `roadmaps/34`). Era um arquivo de 653
//! linhas, e a `ARCHITECTURE.md` §4 diz que `handlers/<dominio>.rs` e' FINO:
//! roteia, valida params, delega, formata a resposta. Um handler daquele
//! tamanho nao e' um handler grande, e' um handler que virou dominio.
//!
//! O corte segue as MESMAS costuras que o dominio `crate::lsp` ja' tem, para
//! que handler e dominio se leiam em paralelo:
//!
//! ```text
//! handlers/lsp/sessao.rs    <-> lsp/session.rs      ciclo de vida do servidor
//! handlers/lsp/documento.rs <-> lsp/sync.rs         o TEXTO do documento
//! handlers/lsp/consulta.rs  <-> lsp/manager.rs      operacoes interativas
//! handlers/lsp/edicao.rs    <-> lsp/transaction.rs  transacao de WorkspaceEdit
//! ```
//!
//! Este `mod.rs` fica com o roteador e mais nada.

mod consulta;
mod documento;
mod edicao;
mod sessao;
mod toolchain;

use kinein_protocol::JsonRpcResponse;
use serde_json::Value;

use crate::Core;

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
}

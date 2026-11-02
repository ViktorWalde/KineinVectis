//! Compiler evidence for the code-action preview, off the IPC loop.

use kinein_protocol::{EditVerification, JsonRpcResponse, LspWorkspaceEditPreviewResult};
use serde_json::{Value, json};
use std::{path::Path, sync::atomic::Ordering, time::Duration};

use crate::{Core, build::edit_verification};

impl Core {
    pub(super) fn verify_code_action_preview(
        &mut self,
        request_id: Option<Value>,
        root: &Path,
        response: JsonRpcResponse,
        plan: &crate::lsp::WorkspaceEditPlan,
    ) -> JsonRpcResponse {
        let Some(value) = response.result.as_ref() else {
            return response;
        };
        let Ok(mut preview) =
            serde_json::from_value::<LspWorkspaceEditPreviewResult>(value.clone())
        else {
            return response;
        };
        let trusted = self.workspace.as_ref().is_some_and(|w| w.trusted);
        self.workspace_edits
            .track_verification(&preview.transaction_id);
        let mut spans = match preview
            .files
            .iter()
            .zip(&plan.files)
            .map(|(file, change)| {
                crate::lsp::text_edit_byte_spans(&file.before_content, &change.edits)
            })
            .collect::<Result<Vec<_>, _>>()
        {
            Ok(spans) => spans,
            Err(error) => {
                let mut evidence = EditVerification::not_verified(error.to_string());
                evidence.obsolete = true;
                preview.verification = Some(evidence);
                let _ = self.workspace_edits.cancel(&preview.transaction_id);
                return JsonRpcResponse::success(request_id, json!(preview));
            }
        };
        let reason = if !trusted {
            Some("Workspace não confiável: compilador não executado.")
        } else if preview.files.len() != 1 {
            Some("Escopo ainda não suportado: a correção altera múltiplos arquivos.")
        } else {
            None
        };
        if let Some(reason) = reason {
            let mut evidence = EditVerification::not_verified(reason);
            if !trusted {
                evidence.reason_code = Some("untrustedWorkspace".into());
            }
            preview.verification = Some(evidence);
            return JsonRpcResponse::success(request_id, json!(preview));
        }
        let file = preview.files[0].clone();
        let edits = spans.pop().unwrap_or_default();
        let path = std::path::PathBuf::from(&file.path);
        let version = self
            .lsp
            .as_ref()
            .and_then(|lsp| lsp.document_version(&path, None));
        let root = root.to_path_buf();
        let cancel = self
            .workspace_edits
            .begin_verification(&preview.transaction_id);
        self.defer_then(
            request_id,
            {
                let cancel = cancel.clone();
                move || {
                    edit_verification::verify(
                        edit_verification::Comparison {
                            root: &root,
                            path: &path,
                            before: &file.before_content,
                            after: &file.after_content,
                            version,
                            edits: &edits,
                        },
                        &cancel,
                        Duration::from_secs(5),
                        || crate::index::context::verification_context(&root, &path),
                    )
                }
            },
            move |core, request_id, result| {
                core.workspace_edits
                    .finish_verification(&preview.transaction_id);
                if cancel.load(Ordering::SeqCst)
                    || !core.workspace_edits.contains(&preview.transaction_id)
                    || result.guard.as_ref().is_some_and(|g| !g.is_current())
                    || result.evidence.obsolete
                {
                    let mut evidence = EditVerification::not_verified(
                        "Edição ou contexto obsoleto; consulte as ações novamente.",
                    );
                    evidence.obsolete = true;
                    preview.verification = Some(evidence);
                    let _ = core.workspace_edits.cancel(&preview.transaction_id);
                } else {
                    preview.verification = Some(result.evidence);
                    if let Some(guard) = result.guard {
                        core.workspace_edits
                            .set_verification_guard(&preview.transaction_id, guard);
                    }
                }
                JsonRpcResponse::success(request_id, json!(preview))
            },
        )
    }
}

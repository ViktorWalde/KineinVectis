//! Async orchestration for workspace file transfers in a batch.

use kinein_protocol::{
    FsTransferBatchParams, FsTransferBatchResult, FsTransferOperation, FsTransferStatus, JobRisk,
    JsonRpcResponse,
};
use serde_json::{Value, json};

use crate::jobs::JobOutcome;
use crate::rpc::{fs_error_response, no_workspace_response, parse_params};
use crate::{Core, fsops};

impl Core {
    pub(super) fn fs_transfer_batch_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "fs.transferBatch");
        };
        let parsed = match parse_params::<FsTransferBatchParams>(
            request_id.as_ref(),
            params,
            "fs.transferBatch requer operation e items",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        if let (Some(jobs), Some(responses)) = (self.jobs.as_ref(), self.deferred.as_ref()) {
            let responses = responses.clone();
            let title = format!(
                "{} {} itens",
                match parsed.operation {
                    FsTransferOperation::Copy => "Copiar",
                    FsTransferOperation::Move => "Mover",
                    FsTransferOperation::Import => "Importar",
                },
                parsed.items.len()
            );
            jobs.spawn(
                "fs.transferBatch",
                title,
                JobRisk::Medium,
                true,
                move |ctx| {
                    ctx.report_progress(0.0, Some("Conferindo lote"));
                    let outcome = fsops::transfer_batch_with_progress(
                        &root,
                        &parsed,
                        || ctx.is_cancelled(),
                        |index, count, done, total| {
                            let item_permille = if total == 0 {
                                0
                            } else {
                                (u128::from(done) * 1000 / u128::from(total)).min(990)
                            };
                            let permille = (index as u128 * 1000 + item_permille) / count as u128;
                            let fraction =
                                f64::from(u16::try_from(permille).unwrap_or(1000)) / 1000.0;
                            ctx.report_progress(fraction, Some("Transferindo arquivos"));
                        },
                    );
                    let status = match &outcome {
                        Ok(result)
                            if result
                                .items
                                .iter()
                                .any(|item| matches!(item.status, FsTransferStatus::Failed)) =>
                        {
                            JobOutcome::Warning
                        }
                        Ok(_) => JobOutcome::Success,
                        Err(_) => JobOutcome::Failed,
                    };
                    drop(responses.send(transfer_batch_result_response(request_id, outcome)));
                    status
                },
            );
            return crate::rpc::deferred_marker();
        }
        self.defer_work(request_id, move |request_id| {
            transfer_batch_result_response(
                request_id,
                fsops::transfer_batch_with_progress(&root, &parsed, || false, |_, _, _, _| {}),
            )
        })
    }
}

fn transfer_batch_result_response(
    request_id: Option<Value>,
    outcome: Result<FsTransferBatchResult, fsops::FsError>,
) -> JsonRpcResponse {
    match outcome {
        Ok(result) => JsonRpcResponse::success(request_id, json!(result)),
        Err(error) => fs_error_response(request_id, &error),
    }
}

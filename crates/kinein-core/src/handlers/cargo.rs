//! Handlers for `cargo.*` requests (`impl Core`): metadata sincrono e
//! check como job, ambos exigindo workspace Rust/Cargo.

use kinein_protocol::{
    BuildSystem, JobAcceptedResult, JobRisk, JsonRpcError, JsonRpcErrorCode, JsonRpcResponse,
};
use serde_json::{Value, json};

use super::build::{emit_build_event, emit_run_error, job_outcome};
use crate::rpc::no_workspace_response;
use crate::{Core, build, cargo, jobs};

impl Core {
    /// Roteia os metodos `cargo.*`; `None` quando o metodo nao e Cargo.
    pub(crate) fn cargo_request_response(
        &self,
        method: &str,
        request_id: Option<Value>,
        _params: Option<&Value>,
    ) -> Option<JsonRpcResponse> {
        match method {
            "cargo.metadata" => Some(self.cargo_metadata_response(request_id)),
            "cargo.check" => Some(self.cargo_check_response(request_id)),
            _ => None,
        }
    }

    /// Valida a capacidade Cargo do workspace; devolve o root.
    fn cargo_workspace_root(
        &self,
        request_id: Option<&Value>,
        method: &str,
    ) -> Result<std::path::PathBuf, Box<JsonRpcResponse>> {
        let Some(workspace) = self.workspace.as_ref() else {
            return Err(Box::new(no_workspace_response(request_id.cloned(), method)));
        };
        if !workspace.capabilities.supports(BuildSystem::Cargo) {
            return Err(Box::new(JsonRpcResponse::failure(
                request_id.cloned(),
                JsonRpcError::new(
                    JsonRpcErrorCode::InvalidParams,
                    format!("{method} requer um workspace Rust/Cargo"),
                    Some(json!({
                        "kind": workspace.kind,
                        "buildSystems": workspace.capabilities.build_systems,
                    })),
                ),
            )));
        }
        Ok(std::path::PathBuf::from(&workspace.root))
    }

    fn cargo_metadata_response(&self, request_id: Option<Value>) -> JsonRpcResponse {
        let root = match self.cargo_workspace_root(request_id.as_ref(), "cargo.metadata") {
            Ok(root) => root,
            Err(response) => return *response,
        };
        match cargo::run_metadata(&root) {
            Ok(summary) => JsonRpcResponse::success(request_id, json!(summary)),
            Err(message) => JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(JsonRpcErrorCode::InternalError, message, None),
            ),
        }
    }

    fn cargo_check_response(&self, request_id: Option<Value>) -> JsonRpcResponse {
        let root = match self.cargo_workspace_root(request_id.as_ref(), "cargo.check") {
            Ok(root) => root,
            Err(response) => return *response,
        };
        let Some(jobs) = self.jobs.as_ref() else {
            return JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(
                    JsonRpcErrorCode::InternalError,
                    "jobs nao estao habilitados neste loop do core",
                    Some(json!({ "method": "cargo.check" })),
                ),
            );
        };

        // Reusa o pipeline de eventos do quality: mesmo JSON de diagnostics,
        // mesma aba Problems. O titulo do job distingue check de clippy;
        // facetas por origem entram com o Problems 2.0 (roadmap P1).
        let job_id = jobs.spawn(
            "cargo.check",
            "Cargo Check",
            JobRisk::Medium,
            true,
            move |ctx| {
                let cancel = ctx.cancellation();
                let mut sink = |event: build::BuildEvent| emit_build_event(ctx, "quality", &event);
                match build::run_cargo_check(&root, &cancel, &mut sink) {
                    Ok(outcome) => {
                        ctx.emit_event(
                            "event.quality.finished",
                            json!({
                                "jobId": ctx.id(),
                                "success": outcome.success,
                                "exitCode": outcome.exit_code,
                                "diagnostics": outcome.diagnostics,
                            }),
                        );
                        job_outcome(outcome.success)
                    }
                    Err(error) => {
                        emit_run_error(ctx, "quality", &error.to_string());
                        jobs::JobOutcome::Failed
                    }
                }
            },
        );

        JsonRpcResponse::success(request_id, json!(JobAcceptedResult { job_id }))
    }
}

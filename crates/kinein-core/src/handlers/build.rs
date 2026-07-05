//! Handlers for `build.run` (async job) and the streaming `test.run` /
//! `quality.run` requests.
//!
//! `build.run` validates synchronously, then spawns a cancelable job: it returns
//! `{ jobId }` immediately and the build runs on the [`JobManager`](crate::jobs)
//! thread, emitting `event.build.*` (tagged with `jobId`) for the build tool
//! window and Problems panel, plus `event.job.*` for the status bar. `test.run`
//! and `quality.run` still stream synchronously through an `emit` callback.

use std::path::PathBuf;
use std::sync::{Arc, atomic::AtomicBool};

use kinein_protocol::{
    JobAcceptedResult, JobRisk, JsonRpcError, JsonRpcErrorCode, JsonRpcRequest, JsonRpcResponse,
    ProjectKind, QualityRunResult, TestRunResult,
};
use serde_json::{Value, json};

use crate::jobs::JobOutcome;
use crate::rpc::no_workspace_response;
use crate::{Core, build, test};

impl Core {
    pub(crate) fn build_run_response(&self, request_id: Option<Value>) -> JsonRpcResponse {
        let Some(workspace) = self.workspace.as_ref() else {
            return no_workspace_response(request_id, "build.run");
        };
        let root = PathBuf::from(&workspace.root);
        let kind = workspace.kind;

        // Pre-flight, synchronous: reject kinds we cannot build before starting
        // a job, so the caller still gets INVALID_REQUEST inline.
        if !matches!(kind, ProjectKind::RustCargo | ProjectKind::Cmake) {
            return JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(
                    JsonRpcErrorCode::InvalidRequest,
                    format!(
                        "build ainda nao e suportado para projetos do tipo {}",
                        build::project_kind_name(kind)
                    ),
                    None,
                ),
            );
        }

        let Some(jobs) = self.jobs.as_ref() else {
            return JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(
                    JsonRpcErrorCode::InternalError,
                    "jobs nao estao habilitados neste loop do core",
                    Some(json!({ "method": "build.run" })),
                ),
            );
        };

        let job_id = jobs.spawn("build", "Build", JobRisk::Medium, true, move |ctx| {
            let cancel = ctx.cancellation();
            let mut sink = |event: build::BuildEvent| {
                let id = ctx.id();
                let (method, params) = match event {
                    build::BuildEvent::Started { command } => (
                        "event.build.started",
                        json!({ "jobId": id, "command": command }),
                    ),
                    build::BuildEvent::Output { stream, line } => (
                        "event.build.output",
                        json!({ "jobId": id, "stream": stream, "line": line }),
                    ),
                    build::BuildEvent::Diagnostic(diagnostic) => {
                        let mut params =
                            serde_json::to_value(&diagnostic).unwrap_or_else(|_error| json!({}));
                        if let Some(map) = params.as_object_mut() {
                            map.insert("jobId".to_owned(), json!(id));
                        }
                        ("event.build.diagnostic", params)
                    }
                };
                ctx.emit_event(method, params);
            };

            match build::run_build(&root, kind, &cancel, &mut sink) {
                Ok(outcome) => {
                    ctx.emit_event(
                        "event.build.finished",
                        json!({
                            "jobId": ctx.id(),
                            "success": outcome.success,
                            "exitCode": outcome.exit_code,
                            "diagnostics": outcome.diagnostics,
                        }),
                    );
                    if outcome.success {
                        JobOutcome::Success
                    } else {
                        JobOutcome::Failed
                    }
                }
                Err(error) => {
                    ctx.emit_event(
                        "event.build.finished",
                        json!({
                            "jobId": ctx.id(),
                            "success": false,
                            "error": error.to_string(),
                        }),
                    );
                    JobOutcome::Failed
                }
            }
        });

        JsonRpcResponse::success(request_id, json!(JobAcceptedResult { job_id }))
    }

    pub(crate) fn quality_run_response(
        &self,
        request_id: Option<Value>,
        emit: &mut dyn FnMut(&JsonRpcRequest),
    ) -> JsonRpcResponse {
        let Some(workspace) = self.workspace.as_ref() else {
            return no_workspace_response(request_id, "quality.run");
        };
        let root = PathBuf::from(&workspace.root);
        let kind = workspace.kind;

        let mut sink = |event: build::BuildEvent| {
            let (method, params) = match event {
                build::BuildEvent::Started { command } => {
                    ("event.quality.started", json!({ "command": command }))
                }
                build::BuildEvent::Output { stream, line } => (
                    "event.quality.output",
                    json!({ "stream": stream, "line": line }),
                ),
                build::BuildEvent::Diagnostic(diagnostic) => {
                    ("event.quality.diagnostic", json!(diagnostic))
                }
            };
            emit(&JsonRpcRequest::notification(method, Some(params)));
        };

        // quality.run is still synchronous; it does not expose cancellation yet.
        let never_cancel = Arc::new(AtomicBool::new(false));
        match build::run_quality(&root, kind, &never_cancel, &mut sink) {
            Ok(outcome) => {
                emit(&JsonRpcRequest::notification(
                    "event.quality.finished",
                    Some(json!({
                        "success": outcome.success,
                        "exitCode": outcome.exit_code,
                        "diagnostics": outcome.diagnostics,
                    })),
                ));
                JsonRpcResponse::success(
                    request_id,
                    json!(QualityRunResult {
                        success: outcome.success,
                        exit_code: outcome.exit_code,
                        diagnostics: outcome.diagnostics,
                    }),
                )
            }
            Err(error) => {
                let code = if error.is_missing_tool() {
                    JsonRpcErrorCode::ToolNotFound
                } else if matches!(error, build::BuildError::Unsupported { .. }) {
                    JsonRpcErrorCode::InvalidRequest
                } else {
                    JsonRpcErrorCode::InternalError
                };
                JsonRpcResponse::failure(
                    request_id,
                    JsonRpcError::new(code, error.to_string(), None),
                )
            }
        }
    }

    pub(crate) fn test_run_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
        emit: &mut dyn FnMut(&JsonRpcRequest),
    ) -> JsonRpcResponse {
        let Some(workspace) = self.workspace.as_ref() else {
            return no_workspace_response(request_id, "test.run");
        };
        let filter = params
            .and_then(|value| value.get("filter"))
            .and_then(Value::as_str)
            .map(ToOwned::to_owned);
        let root = PathBuf::from(&workspace.root);
        let kind = workspace.kind;

        let mut sink = |event: test::TestEvent| {
            let (method, params) = match event {
                test::TestEvent::Started { command } => {
                    ("event.test.started", json!({ "command": command }))
                }
                test::TestEvent::Output { stream, line } => (
                    "event.test.output",
                    json!({ "stream": stream, "line": line }),
                ),
                test::TestEvent::Case { name, status } => (
                    "event.test.case",
                    json!({ "name": name, "status": status.as_str() }),
                ),
            };
            emit(&JsonRpcRequest::notification(method, Some(params)));
        };

        match test::run_tests(&root, kind, filter.as_deref(), &mut sink) {
            Ok(outcome) => {
                emit(&JsonRpcRequest::notification(
                    "event.test.finished",
                    Some(json!({
                        "success": outcome.success,
                        "exitCode": outcome.exit_code,
                        "passed": outcome.passed,
                        "failed": outcome.failed,
                        "ignored": outcome.ignored,
                    })),
                ));
                JsonRpcResponse::success(
                    request_id,
                    json!(TestRunResult {
                        success: outcome.success,
                        exit_code: outcome.exit_code,
                        passed: outcome.passed,
                        failed: outcome.failed,
                        ignored: outcome.ignored,
                    }),
                )
            }
            Err(error) => {
                let code = if error.is_missing_tool() {
                    JsonRpcErrorCode::ToolNotFound
                } else if matches!(error, test::TestError::Unsupported { .. }) {
                    JsonRpcErrorCode::InvalidRequest
                } else {
                    JsonRpcErrorCode::InternalError
                };
                JsonRpcResponse::failure(
                    request_id,
                    JsonRpcError::new(code, error.to_string(), None),
                )
            }
        }
    }
}

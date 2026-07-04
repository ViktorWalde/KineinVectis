//! Handlers for the streaming `build.run` / `test.run` / `quality.run` requests.
//!
//! Each is an `impl Core` method that streams `event.*` notifications through an
//! `emit` callback, delegating to `crate::build` and `crate::test`.

use std::path::PathBuf;

use kernwerk_protocol::{
    BuildRunResult, JsonRpcError, JsonRpcErrorCode, JsonRpcRequest, JsonRpcResponse,
    QualityRunResult, TestRunResult,
};
use serde_json::{Value, json};

use crate::rpc::no_workspace_response;
use crate::{Core, build, test};

impl Core {
    pub(crate) fn build_run_response(
        &self,
        request_id: Option<Value>,
        emit: &mut dyn FnMut(&JsonRpcRequest),
    ) -> JsonRpcResponse {
        let Some(workspace) = self.workspace.as_ref() else {
            return no_workspace_response(request_id, "build.run");
        };
        let root = PathBuf::from(&workspace.root);
        let kind = workspace.kind;

        let mut sink = |event: build::BuildEvent| {
            let (method, params) = match event {
                build::BuildEvent::Started { command } => {
                    ("event.build.started", json!({ "command": command }))
                }
                build::BuildEvent::Output { stream, line } => (
                    "event.build.output",
                    json!({ "stream": stream, "line": line }),
                ),
                build::BuildEvent::Diagnostic(diagnostic) => {
                    ("event.build.diagnostic", json!(diagnostic))
                }
            };
            emit(&JsonRpcRequest::notification(method, Some(params)));
        };

        match build::run_build(&root, kind, &mut sink) {
            Ok(outcome) => {
                emit(&JsonRpcRequest::notification(
                    "event.build.finished",
                    Some(json!({
                        "success": outcome.success,
                        "exitCode": outcome.exit_code,
                        "diagnostics": outcome.diagnostics,
                    })),
                ));
                JsonRpcResponse::success(
                    request_id,
                    json!(BuildRunResult {
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

        match build::run_quality(&root, kind, &mut sink) {
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

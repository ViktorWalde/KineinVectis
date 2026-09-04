//! Handlers for the build / quality / test runners, all async cancelable jobs.
//!
//! `build.run`, `quality.run` and `test.run` validate synchronously, then spawn
//! a job on the [`JobManager`](crate::jobs): each returns `{ jobId }` immediately
//! and runs on a background thread, emitting its rich domain events tagged with
//! `jobId` (`event.build.*` / `event.quality.*` / `event.test.*`, for the build
//! tool window and Problems) plus `event.job.*` (for the status bar). Cancelling
//! the job kills the underlying process.

use std::path::PathBuf;

use kinein_protocol::{
    BuildRunParams, BuildSystem, DiagnosticSource, JobAcceptedResult, JobRisk, JsonRpcError,
    JsonRpcErrorCode, JsonRpcResponse, ProjectKind, QualityRunParams, TestRunParams,
};
use serde_json::{Value, json};

use crate::jobs::{JobContext, JobOutcome};
use crate::rpc::{jobs_unavailable_response, no_workspace_response, parse_params};
use crate::{Core, build, test};

impl Core {
    pub(crate) fn build_run_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let parsed = match parse_params::<BuildRunParams>(
            request_id.as_ref(),
            params,
            "build.run aceita apenas o campo opcional buildSystem",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let (root, kind) =
            match self.runner_workspace(request_id.as_ref(), "build.run", parsed.build_system) {
                Ok(context) => context,
                Err(response) => return *response,
            };
        if !matches!(kind, ProjectKind::RustCargo | ProjectKind::Cmake) {
            return unsupported_kind_response(request_id, "build", kind);
        }
        let Some(jobs) = self.jobs.as_ref() else {
            return jobs_unavailable_response(request_id, "build.run");
        };

        // M4.5: o perfil de rigor efetivo regula o build do USUARIO.
        let profile = crate::settings::effective_rigor_profile(&root);
        // Resolvida na thread do loop: o job nao alcanca o `Core`
        // (arquitetura/04 §3) e a toolchain precisa do detector de ferramentas.
        let toolchain = crate::toolchain::Toolchain::resolve(&root, &self.detected_tools());
        let title = format!("{} Build", project_system_name(kind));
        let job_id = jobs.spawn("build", title, JobRisk::Medium, true, move |ctx| {
            let cancel = ctx.cancellation();
            let mut sink = |event: build::BuildEvent| emit_build_event(ctx, "build", &event);
            match build::run_build(&root, kind, profile, &toolchain, &cancel, &mut sink) {
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
                    job_outcome(outcome.success)
                }
                Err(error) => {
                    emit_run_error(ctx, "build", &error.to_string());
                    JobOutcome::Failed
                }
            }
        });

        JsonRpcResponse::success(request_id, json!(JobAcceptedResult { job_id }))
    }

    pub(crate) fn quality_run_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let parsed = match parse_params::<QualityRunParams>(
            request_id.as_ref(),
            params,
            "quality.run aceita apenas o campo opcional buildSystem",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let (root, kind) =
            match self.runner_workspace(request_id.as_ref(), "quality.run", parsed.build_system) {
                Ok(context) => context,
                Err(response) => return *response,
            };
        // Only Rust/Cargo has a linter integration (cargo clippy) so far.
        if kind != ProjectKind::RustCargo {
            return unsupported_kind_response(request_id, "quality", kind);
        }
        let Some(jobs) = self.jobs.as_ref() else {
            return jobs_unavailable_response(request_id, "quality.run");
        };

        let profile = crate::settings::effective_rigor_profile(&root);
        let toolchain = crate::toolchain::Toolchain::resolve(&root, &self.detected_tools());
        let job_id = jobs.spawn("quality", "Quality", JobRisk::Medium, true, move |ctx| {
            let cancel = ctx.cancellation();
            let mut sink = |event: build::BuildEvent| emit_build_event(ctx, "quality", &event);
            match build::run_quality(&root, kind, profile, &toolchain, &cancel, &mut sink) {
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
                    JobOutcome::Failed
                }
            }
        });

        JsonRpcResponse::success(request_id, json!(JobAcceptedResult { job_id }))
    }

    pub(crate) fn test_run_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let parsed = match parse_params::<TestRunParams>(
            request_id.as_ref(),
            params,
            "test.run aceita apenas os campos opcionais filter e buildSystem",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let (root, kind) =
            match self.runner_workspace(request_id.as_ref(), "test.run", parsed.build_system) {
                Ok(context) => context,
                Err(response) => return *response,
            };
        if !matches!(kind, ProjectKind::RustCargo | ProjectKind::Cmake) {
            return unsupported_kind_response(request_id, "test", kind);
        }
        let filter = parsed.filter;
        let Some(jobs) = self.jobs.as_ref() else {
            return jobs_unavailable_response(request_id, "test.run");
        };

        let title = format!("{} Tests", project_system_name(kind));
        let job_id = jobs.spawn("test", title, JobRisk::Medium, true, move |ctx| {
            let cancel = ctx.cancellation();
            let mut sink = |event: test::TestEvent| {
                let id = ctx.id();
                let (method, params) = match event {
                    test::TestEvent::Started { command } => (
                        "event.test.started",
                        json!({ "jobId": id, "command": command }),
                    ),
                    test::TestEvent::Output { stream, line } => ("event.test.output", {
                        ctx.emit_output(&line);
                        json!({ "jobId": id, "stream": stream, "line": line })
                    }),
                    test::TestEvent::Case { name, status } => (
                        "event.test.case",
                        json!({ "jobId": id, "name": name, "status": status.as_str() }),
                    ),
                };
                ctx.emit_event(method, params);
            };
            match test::run_tests(&root, kind, filter.as_deref(), &cancel, &mut sink) {
                Ok(outcome) => {
                    ctx.emit_event(
                        "event.test.finished",
                        json!({
                            "jobId": ctx.id(),
                            "success": outcome.success,
                            "exitCode": outcome.exit_code,
                            "passed": outcome.passed,
                            "failed": outcome.failed,
                            "ignored": outcome.ignored,
                        }),
                    );
                    job_outcome(outcome.success)
                }
                Err(error) => {
                    emit_run_error(ctx, "test", &error.to_string());
                    JobOutcome::Failed
                }
            }
        });

        JsonRpcResponse::success(request_id, json!(JobAcceptedResult { job_id }))
    }

    /// Resolves an explicit hybrid build-system selection against the one
    /// workspace snapshot. With no selection, preserves the primary `kind`.
    fn runner_workspace(
        &self,
        request_id: Option<&Value>,
        method: &str,
        requested: Option<BuildSystem>,
    ) -> Result<(PathBuf, ProjectKind), Box<JsonRpcResponse>> {
        let Some(workspace) = self.workspace.as_ref() else {
            return Err(Box::new(no_workspace_response(request_id.cloned(), method)));
        };
        let kind = if let Some(build_system) = requested {
            if !workspace.capabilities.supports(build_system) {
                return Err(Box::new(JsonRpcResponse::failure(
                    request_id.cloned(),
                    JsonRpcError::new(
                        JsonRpcErrorCode::InvalidParams,
                        format!(
                            "{method} requer que o sistema {} exista no workspace",
                            project_system_name(build_system.project_kind())
                        ),
                        Some(json!({
                            "buildSystem": build_system,
                            "available": workspace.capabilities.build_systems,
                        })),
                    ),
                )));
            }
            build_system.project_kind()
        } else {
            workspace.kind
        };
        Ok((PathBuf::from(&workspace.root), kind))
    }
}

const fn project_system_name(kind: ProjectKind) -> &'static str {
    match kind {
        ProjectKind::RustCargo => "Cargo",
        ProjectKind::Cmake => "CMake",
        ProjectKind::Maven => "Maven",
        ProjectKind::Gradle => "Gradle",
        ProjectKind::Python => "Python",
        ProjectKind::Unknown => "Unknown",
    }
}

/// Maps a [`build::BuildEvent`] onto an `event.<domain>.*` notification tagged
/// with the job id, so `build` and `quality` share one mapping.
pub(super) fn emit_build_event(ctx: &JobContext, domain: &str, event: &build::BuildEvent) {
    let id = ctx.id();
    let (method, params) = match event {
        build::BuildEvent::Started { command } => (
            format!("event.{domain}.started"),
            json!({ "jobId": id, "command": command }),
        ),
        build::BuildEvent::Output { stream, line } => (format!("event.{domain}.output"), {
            ctx.emit_output(line);
            json!({ "jobId": id, "stream": stream, "line": line })
        }),
        build::BuildEvent::Diagnostic(diagnostic) => {
            let params = serde_json::to_value(
                diagnostic.to_diagnostic(diagnostic_source(domain), Some(id.to_owned())),
            )
            .unwrap_or_else(|_error| json!({}));
            (format!("event.{domain}.diagnostic"), params)
        }
    };
    ctx.emit_event(&method, params);
}

const fn diagnostic_source(domain: &str) -> DiagnosticSource {
    match domain.as_bytes() {
        b"quality" => DiagnosticSource::Quality,
        _ => DiagnosticSource::Build,
    }
}

/// Emits `event.<domain>.finished` with a failure and a message.
pub(super) fn emit_run_error(ctx: &JobContext, domain: &str, message: &str) {
    ctx.emit_event(
        &format!("event.{domain}.finished"),
        json!({ "jobId": ctx.id(), "success": false, "error": message }),
    );
}

pub(super) const fn job_outcome(success: bool) -> JobOutcome {
    if success {
        JobOutcome::Success
    } else {
        JobOutcome::Failed
    }
}

fn unsupported_kind_response(
    request_id: Option<Value>,
    action: &str,
    kind: ProjectKind,
) -> JsonRpcResponse {
    JsonRpcResponse::failure(
        request_id,
        JsonRpcError::new(
            JsonRpcErrorCode::InvalidRequest,
            format!(
                "{action} ainda nao e suportado para projetos do tipo {}",
                build::project_kind_name(kind)
            ),
            None,
        ),
    )
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use kinein_protocol::{BuildDiagnostic, BuildDiagnosticSeverity, JobRisk};

    use super::emit_build_event;
    use crate::{
        build::BuildEvent,
        jobs::{JobManager, JobOutcome},
    };

    #[test]
    fn diagnostic_events_include_source_and_job_id() {
        let (sender, receiver) = std::sync::mpsc::channel();
        let manager = JobManager::new(sender);

        let job_id = manager.spawn("quality", "Quality", JobRisk::Medium, false, |ctx| {
            emit_build_event(
                ctx,
                "quality",
                &BuildEvent::Diagnostic(BuildDiagnostic {
                    severity: BuildDiagnosticSeverity::Error,
                    message: "falhou".to_owned(),
                    file: Some("src/lib.rs".to_owned()),
                    line: Some(10),
                    column: Some(4),
                }),
            );
            JobOutcome::Success
        });

        let mut saw_diagnostic = false;
        loop {
            let event = receiver
                .recv_timeout(Duration::from_secs(5))
                .expect("evento do job dentro do timeout");
            match event.method.as_str() {
                "event.quality.diagnostic" => {
                    let params = event.params.as_ref().unwrap();
                    assert_eq!(params["jobId"], job_id.as_str());
                    assert_eq!(params["source"], "quality");
                    assert_eq!(params["severity"], "error");
                    assert_eq!(params["message"], "falhou");
                    saw_diagnostic = true;
                }
                "event.job.finished" => break,
                _ => {}
            }
        }

        assert!(saw_diagnostic, "faltou diagnostico com source");
    }
}

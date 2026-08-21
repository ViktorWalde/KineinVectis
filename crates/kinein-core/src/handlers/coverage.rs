//! Handler de `coverage.run` (`impl Core`).
//!
//! Fino: valida, spawna o job e delega ao dominio `crate::coverage`. O
//! resultado chega por `event.coverage.finished` — sucesso com totais e
//! arquivos, falha com `error`.

use kinein_protocol::{
    CoverageRunParams, JobAcceptedResult, JobRisk, JsonRpcResponse, ProjectKind,
};
use serde_json::{Value, json};

use super::build::{jobs_unavailable_response, unsupported_kind_response};
use crate::rpc::parse_params;
use crate::{Core, coverage, jobs::JobOutcome};

impl Core {
    pub(crate) fn coverage_run_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let parsed = match parse_params::<CoverageRunParams>(
            request_id.as_ref(),
            params,
            "coverage.run aceita apenas o campo opcional buildSystem",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let (root, kind) =
            match self.runner_workspace(request_id.as_ref(), "coverage.run", parsed.build_system) {
                Ok(context) => context,
                Err(response) => return *response,
            };
        if !matches!(kind, ProjectKind::RustCargo | ProjectKind::Cmake) {
            return unsupported_kind_response(request_id, "coverage", kind);
        }
        let Some(jobs) = self.jobs.as_ref() else {
            return jobs_unavailable_response(request_id, "coverage.run");
        };

        let job_id = jobs.spawn("coverage", "Coverage", JobRisk::Medium, true, move |ctx| {
            let cancel = ctx.cancellation();
            let mut on_output = |line: &str| ctx.emit_output(line);
            match coverage::run_coverage(&root, kind, &cancel, &mut on_output) {
                Ok(report) => {
                    ctx.emit_event(
                        "event.coverage.finished",
                        json!({
                            "jobId": ctx.id(),
                            "success": true,
                            "linesCovered": report.totals.lines_covered,
                            "linesTotal": report.totals.lines_total,
                            "percent": report.totals.percent,
                            "files": report.files,
                        }),
                    );
                    JobOutcome::Success
                }
                Err(error) => {
                    ctx.emit_event(
                        "event.coverage.finished",
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
}

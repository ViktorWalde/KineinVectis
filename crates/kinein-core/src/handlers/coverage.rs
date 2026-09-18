//! Handlers de `coverage.*` (`impl Core`).
//!
//! A cobertura dos testes como JOB (`coverage.run`) e as linhas de um
//! arquivo do ultimo relatorio (`coverage.lines`, sincrono — le o LCOV do
//! disco). D8 do `roadmaps/41`, 2026-09-17.

use kinein_protocol::{
    CoverageFinishedEvent, CoverageLinesParams, CoverageRunParams, JobAcceptedResult, JobRisk,
    JsonRpcError, JsonRpcErrorCode, JsonRpcResponse, ProjectKind,
};
use serde_json::{Value, json};

use crate::Core;
use crate::coverage::{self, CoverageEvent, CoverageTools, Report};
use crate::jobs::JobOutcome;
use crate::rpc::{jobs_unavailable_response, no_workspace_response, parse_params};

impl Core {
    /// Roteia `coverage.run` e `coverage.lines`.
    pub(crate) fn coverage_request_response(
        &self,
        method: &str,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> Option<JsonRpcResponse> {
        match method {
            "coverage.run" => Some(self.coverage_run_response(request_id, params)),
            "coverage.lines" => Some(self.coverage_lines_response(request_id, params)),
            _ => None,
        }
    }

    /// `coverage.run {}` -> `{ jobId }`; `event.coverage.finished` no fim.
    fn coverage_run_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        if let Err(response) = parse_params::<CoverageRunParams>(
            request_id.as_ref(),
            params,
            "coverage.run nao aceita parametros",
        ) {
            return *response;
        }
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "coverage.run");
        };
        let kind = self
            .workspace
            .as_ref()
            .map_or(ProjectKind::Unknown, |w| w.kind);
        if !matches!(kind, ProjectKind::RustCargo | ProjectKind::Python) {
            return JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(
                    JsonRpcErrorCode::InvalidRequest,
                    "cobertura so' para Rust (cargo-llvm-cov) e Python (coverage.py) por \
                     enquanto; C/C++ exige compilar com --coverage, o que e' decisao sua no build",
                    Some(json!({ "kind": kind })),
                ),
            );
        }
        let Some(jobs) = self.jobs.as_ref() else {
            return jobs_unavailable_response(request_id, "coverage.run");
        };
        let toolchain = crate::toolchain::Toolchain::resolve(&root, &self.detected_tools());
        let tools = CoverageTools {
            cargo: toolchain.program_for(kinein_protocol::ToolchainRole::Cargo),
            cargo_llvm_cov: self.detector.find_in_path("cargo-llvm-cov"),
            python: (kind == ProjectKind::Python)
                .then(|| self.python_host_launcher(&root))
                .flatten(),
        };
        let job_id = jobs.spawn("coverage", "Cobertura", JobRisk::Medium, true, move |ctx| {
            let cancel = ctx.cancellation();
            let mut sink = |event: CoverageEvent| match event {
                CoverageEvent::Started { command } => ctx.emit_output(&format!("$ {command}")),
                CoverageEvent::Output { line, .. } => ctx.emit_output(&line),
            };
            let resultado = coverage::run(&root, kind, &tools, &cancel, &mut sink);
            let (success, tool, files, error) = match resultado {
                Ok((tool, report)) => {
                    let files = report.summary();
                    let (hit, found) = files
                        .iter()
                        .fold((0, 0), |(h, f), x| (h + x.lines_hit, f + x.lines_found));
                    ctx.emit_output(&format!(
                        "{} arquivo(s): {hit}/{found} linhas cobertas",
                        files.len()
                    ));
                    (true, tool, files, None)
                }
                Err(erro) => {
                    ctx.emit_output(&erro.to_string());
                    (false, String::new(), Vec::new(), Some(erro.to_string()))
                }
            };
            ctx.emit_event(
                "event.coverage.finished",
                json!(CoverageFinishedEvent {
                    job_id: ctx.id().to_owned(),
                    success,
                    tool,
                    path: success.then(|| root.join(coverage::LCOV_PATH).display().to_string()),
                    files,
                    error,
                }),
            );
            if success {
                JobOutcome::Success
            } else {
                JobOutcome::Failed
            }
        });
        JsonRpcResponse::success(request_id, json!(JobAcceptedResult { job_id }))
    }

    /// `coverage.lines { file }` -> as linhas cobertas/perdidas do ultimo
    /// relatorio; `known: false` quando ele nao tem o arquivo (ou nao ha'
    /// relatorio).
    fn coverage_lines_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let parsed = match parse_params::<CoverageLinesParams>(
            request_id.as_ref(),
            params,
            "coverage.lines requer o campo file",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "coverage.lines");
        };
        JsonRpcResponse::success(request_id, json!(Report::load(&root).lines(&parsed.file)))
    }
}

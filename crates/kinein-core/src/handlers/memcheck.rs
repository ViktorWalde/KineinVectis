//! Handler de `memcheck.run` (`impl Core`).
//!
//! Fino: valida, le os extras da config, spawna o job e delega ao dominio
//! `crate::valgrind`. Os achados saem por `event.memcheck.diagnostic` e o
//! resultado por `event.memcheck.finished`.

use kinein_protocol::{
    JobAcceptedResult, JobRisk, JsonRpcResponse, MemcheckRunParams, ProjectKind,
};
use serde_json::{Value, json};

use super::build::{emit_build_event, jobs_unavailable_response, unsupported_kind_response};
use crate::rpc::parse_params;
use crate::{Core, build::BuildEvent, jobs::JobOutcome, valgrind};

impl Core {
    pub(crate) fn memcheck_run_response(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        if let Err(response) = parse_params::<MemcheckRunParams>(
            request_id.as_ref(),
            params,
            "memcheck.run nao aceita parametros",
        ) {
            return *response;
        }
        let (root, kind) = match self.runner_workspace(request_id.as_ref(), "memcheck.run", None) {
            Ok(context) => context,
            Err(response) => return *response,
        };
        // Rust tem seguranca de memoria no compilador e nao roda sob Memcheck
        // no fluxo normal. Erro sincrono e mais honesto que um job que nasce
        // para nao encontrar nada.
        if kind != ProjectKind::Cmake {
            return unsupported_kind_response(request_id, "memcheck", kind);
        }

        let extra_args = self.integration_config_args("valgrind");

        let Some(jobs) = self.jobs.as_ref() else {
            return jobs_unavailable_response(request_id, "memcheck.run");
        };

        // Risk alto: a analise dinamica EXECUTA os testes do projeto, com os
        // efeitos colaterais que eles tiverem. Isso nao e um leitor de codigo.
        let job_id = jobs.spawn("memcheck", "Memcheck", JobRisk::High, true, move |ctx| {
            let cancel = ctx.cancellation();
            let mut on_output = |line: &str| ctx.emit_output(line);
            emit_build_event(
                ctx,
                "memcheck",
                &BuildEvent::Started {
                    command: "valgrind --tool=memcheck".to_owned(),
                },
            );
            match valgrind::run_memcheck(&root, &extra_args, &cancel, &mut on_output) {
                Ok(report) => {
                    let achados = report.findings.len() as u64;
                    for achado in report.findings {
                        emit_build_event(ctx, "memcheck", &BuildEvent::Diagnostic(achado));
                    }
                    ctx.emit_event(
                        "event.memcheck.finished",
                        json!({
                            "jobId": ctx.id(),
                            "success": true,
                            "tests": report.testes,
                            "findings": achados,
                        }),
                    );
                    JobOutcome::Success
                }
                Err(error) => {
                    ctx.emit_event(
                        "event.memcheck.finished",
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

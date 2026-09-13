//! Handler de `test.discover` (`impl Core`).
//!
//! A ARVORE de testes antes do primeiro run (41 B6, o que faltou;
//! 2026-09-13). Um JOB — o cargo compila os testes para lista-los — que
//! termina em `event.test.discovered` com um `TestCaseInfo` por caso, no id
//! exato que `test.run { testId }` aceita.
//!
//! Fino: valida params, resolve o workspace e o lancador Python como o
//! `test.run`, chama `test::discover_tests`, formata.

use kinein_protocol::{
    JobAcceptedResult, JobRisk, JsonRpcResponse, ProjectKind, TestDiscoverParams,
    TestDiscoveredEvent,
};
use serde_json::{Value, json};

use crate::jobs::JobOutcome;
use crate::rpc::{jobs_unavailable_response, parse_params};
use crate::{Core, test};

impl Core {
    /// `test.discover { buildSystem? }` -> `{ jobId }`.
    pub(crate) fn test_discover_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let parsed = match parse_params::<TestDiscoverParams>(
            request_id.as_ref(),
            params,
            "test.discover aceita apenas o campo opcional buildSystem",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let (root, kind) = match self.runner_workspace(
            request_id.as_ref(),
            "test.discover",
            parsed.build_system,
        ) {
            Ok(context) => context,
            Err(response) => return *response,
        };
        let Some(runner) = test::discover::runner_name(kind) else {
            return super::build::unsupported_kind_response(request_id, "test", kind);
        };
        let Some(jobs) = self.jobs.as_ref() else {
            return jobs_unavailable_response(request_id, "test.discover");
        };
        let python = (kind == ProjectKind::Python)
            .then(|| self.python_host_launcher(&root))
            .flatten();
        let job_id = jobs.spawn(
            "test",
            format!("Listar testes ({runner})"),
            JobRisk::Low,
            true,
            move |ctx| {
                let cancel = ctx.cancellation();
                let mut command = String::new();
                let mut sink = |event: test::TestEvent| match event {
                    test::TestEvent::Started { command: c } => {
                        ctx.emit_output(&format!("$ {c}"));
                        command = c;
                    }
                    test::TestEvent::Output { line, .. } => ctx.emit_output(&line),
                    test::TestEvent::Case { .. } => {}
                };
                let resultado =
                    test::discover_tests(&root, kind, python.as_ref(), &cancel, &mut sink);
                let (tests, success, error) = match resultado {
                    Ok(tests) => {
                        ctx.emit_output(&format!("{} teste(s) encontrado(s)", tests.len()));
                        (tests, true, None)
                    }
                    Err(erro) => {
                        ctx.emit_output(&erro.to_string());
                        (Vec::new(), false, Some(erro.to_string()))
                    }
                };
                ctx.emit_event(
                    "event.test.discovered",
                    json!(TestDiscoveredEvent {
                        job_id: ctx.id().to_owned(),
                        runner: runner.to_owned(),
                        command,
                        tests,
                        success,
                        error,
                    }),
                );
                if success {
                    JobOutcome::Success
                } else {
                    JobOutcome::Failed
                }
            },
        );
        JsonRpcResponse::success(request_id, json!(JobAcceptedResult { job_id }))
    }
}

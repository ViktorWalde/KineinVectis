//! O `compiledb` do `PlatformIO` como job (0.4.0e, `roadmaps/52` §14.5): a
//! Configuration Action `platformio.compiledb` delega aqui.
//!
//! So' em projeto confiavel (o `pio` roda os scripts do projeto, como o
//! configure do `CMake`). No sucesso, o mesmo caminho do configure que
//! termina: o contexto do indice e' relido (a CDB nova) e os documentos C/C++
//! sao fechados no clangd, para a UI reabri-los com as flags da CDB — quem
//! descobre o fim e' o `observe_notification`, pelo `event.job.finished` do job
//! anotado aqui; nenhum evento novo.

use kinein_protocol::{
    JobAcceptedResult, JobRisk, JsonRpcError, JsonRpcErrorCode, JsonRpcResponse,
};
use serde_json::{Value, json};

use crate::Core;
use crate::build::{self, Engine};
use crate::rpc::{jobs_unavailable_response, no_workspace_response};

const METHOD: &str = "platformio.compiledb";

impl Core {
    /// `platformio.compiledb` como job: o `pio run -t compiledb` na raiz.
    pub(crate) fn platformio_compiledb_response(
        &self,
        request_id: Option<Value>,
    ) -> JsonRpcResponse {
        let Some(open) = self.workspace.as_ref() else {
            return no_workspace_response(request_id, METHOD);
        };
        let root = std::path::PathBuf::from(&open.root);
        if !open.trusted {
            return refuse(
                request_id,
                "o projeto ainda não é confiável: o pio roda os scripts do projeto. Confie \
                 nele na faixa de saúde e aplique de novo.",
            );
        }
        let pio = match self.framework_engine(&root) {
            Ok(Some(Engine::PlatformIo { pio })) => pio,
            Ok(_) => {
                return refuse(
                    request_id,
                    "este projeto não é PlatformIO (sem platformio.ini)",
                );
            }
            // A mesma recusa do `build.run`: a ferramenta, com o passo do modelo.
            Err(error) => {
                return JsonRpcResponse::failure(
                    request_id,
                    JsonRpcError::new(
                        JsonRpcErrorCode::ToolNotFound,
                        format!("{METHOD}: {error}"),
                        Some(json!({ "framework": error.framework })),
                    ),
                );
            }
        };
        let Some(jobs) = self.jobs.as_ref() else {
            return jobs_unavailable_response(request_id, METHOD);
        };
        let job_id = jobs.spawn(
            METHOD,
            "PlatformIO compiledb",
            JobRisk::Low,
            true,
            move |ctx| {
                let cancel = ctx.cancellation();
                let mut sink = |event: build::BuildEvent| {
                    if let build::BuildEvent::Output { line, .. } = event {
                        ctx.emit_output(&line);
                    }
                };
                let outcome = build::run_platformio_compiledb(&pio, &root, &cancel, &mut sink);
                super::build::job_outcome(outcome.is_ok_and(|done| done.success))
            },
        );
        if let Ok(mut slot) = self.compiledb_job.lock() {
            *slot = Some(job_id.clone());
        }
        JsonRpcResponse::success(request_id, json!(JobAcceptedResult { job_id }))
    }

    /// O fim de um job: o do `compiledb` que deu certo troca a CDB. O pedido
    /// anota o job (`&self`); o fim, visto no `observe_notification`, consome.
    pub(crate) fn on_job_finished(&mut self, params: Option<&Value>) {
        let Some(params) = params else {
            return;
        };
        let ours = self.compiledb_job.lock().ok().is_some_and(|mut slot| {
            let matches = slot.as_deref() == params.get("jobId").and_then(Value::as_str);
            if matches {
                *slot = None;
            }
            matches
        });
        if ours && params.get("status").and_then(Value::as_str) == Some("success") {
            self.reload_index_context();
            self.on_cmake_configure_finished(true);
        }
    }
}

fn refuse(request_id: Option<Value>, message: &str) -> JsonRpcResponse {
    JsonRpcResponse::failure(
        request_id,
        JsonRpcError::new(JsonRpcErrorCode::InvalidRequest, message.to_owned(), None),
    )
}

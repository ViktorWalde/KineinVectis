//! Handlers for `cmake.*` requests (`impl Core`): configure como job,
//! presets/targets/status sincronos, todos exigindo workspace `CMake`.

use kinein_protocol::{
    BuildSystem, CmakeConfigureParams, CmakePresetsResult, CmakeStatusResult, CmakeTargetsResult,
    JobAcceptedResult, JobRisk, JsonRpcError, JsonRpcErrorCode, JsonRpcRequest, JsonRpcResponse,
};
use serde_json::{Value, json};

use crate::rpc::{no_workspace_response, parse_params};
use crate::{Core, cmake, jobs, process};

/// Linguagem cujos documentos o `cmake.configure` invalida.
const CPP_LANGUAGE: &str = "cpp";

impl Core {
    /// Reage ao fim de um `cmake.configure` bem-sucedido fechando os documentos
    /// C/C++ que o language server conhece.
    ///
    /// **Por que isto mora no CORE e nao na UI** (medido em 2026-08-30,
    /// `roadmaps/29` §5b): a UI nao consegue fazer sozinha. Um "reenviar
    /// didOpen" e inerte, porque o texto nao mudou e o curto-circuito por hash
    /// do `lsp/sync` engole a notificacao. Fechar e' operacao de core.
    ///
    /// **Por que aqui e nao num objeto compartilhado com o job** (a saida que o
    /// `arquitetura/04` §3 previa): nao e preciso atravessar a fronteira de
    /// thread. O evento do job JA volta ao dono do estado — o loop principal o
    /// recebe como `LoopEvent::Notification` antes de escrever no stdout, e e
    /// ele que possui o `Core`. Um `Arc<Atomic…>` seria um segundo caminho para
    /// o mesmo fato, com dois donos.
    ///
    /// O `didOpen` seguinte vem da UI, com o BUFFER do editor: o core nao le o
    /// disco aqui, entao arquivo com edicao nao salva nao regride para a versao
    /// gravada.
    pub(crate) fn on_cmake_configure_finished(&mut self, success: bool) {
        if !success {
            return;
        }
        let Some(lsp) = self.lsp.as_mut() else {
            return;
        };
        let closed = lsp.close_documents(CPP_LANGUAGE);
        if closed == 0 {
            return;
        }
        if let Some(events) = self.events.as_ref() {
            drop(events.send(JsonRpcRequest::notification(
                "event.lsp.documentsClosed",
                Some(json!({ "language": CPP_LANGUAGE, "count": closed })),
            )));
        }
    }

    /// Roteia os metodos `cmake.*`; `None` quando o metodo nao e `CMake`.
    pub(crate) fn cmake_request_response(
        &self,
        method: &str,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> Option<JsonRpcResponse> {
        match method {
            "cmake.configure" => Some(self.cmake_configure_response(request_id, params)),
            "cmake.presets.list" => Some(self.cmake_presets_response(request_id)),
            "cmake.targets.list" => Some(self.cmake_targets_response(request_id)),
            "cmake.status" => Some(self.cmake_status_response(request_id)),
            _ => None,
        }
    }

    /// Valida a capacidade `CMake` do workspace; devolve o root.
    fn cmake_workspace_root(
        &self,
        request_id: Option<&Value>,
        method: &str,
    ) -> Result<std::path::PathBuf, Box<JsonRpcResponse>> {
        let Some(workspace) = self.workspace.as_ref() else {
            return Err(Box::new(no_workspace_response(request_id.cloned(), method)));
        };
        if !workspace.capabilities.supports(BuildSystem::Cmake) {
            return Err(Box::new(JsonRpcResponse::failure(
                request_id.cloned(),
                JsonRpcError::new(
                    JsonRpcErrorCode::InvalidParams,
                    format!("{method} requer um workspace CMake"),
                    Some(json!({
                        "kind": workspace.kind,
                        "buildSystems": workspace.capabilities.build_systems,
                    })),
                ),
            )));
        }
        Ok(std::path::PathBuf::from(&workspace.root))
    }

    fn cmake_configure_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let root = match self.cmake_workspace_root(request_id.as_ref(), "cmake.configure") {
            Ok(root) => root,
            Err(response) => return *response,
        };
        let parsed = match parse_params::<CmakeConfigureParams>(
            request_id.as_ref(),
            params,
            "cmake.configure aceita apenas o campo opcional preset",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let Some(jobs) = self.jobs.as_ref() else {
            return JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(
                    JsonRpcErrorCode::InternalError,
                    "jobs nao estao habilitados neste loop do core",
                    Some(json!({ "method": "cmake.configure" })),
                ),
            );
        };

        let preset = parsed.preset;
        // A toolchain e resolvida AQUI, na thread do loop, e vai pronta para o
        // job: um job nao alcanca o `Core` (arquitetura/04 §3), e resolver la
        // dentro exigiria o `ToolDetector`, que e estado do core.
        let toolchain = crate::toolchain::Toolchain::resolve(&root, &self.detected_tools());
        let job_id = jobs.spawn(
            "cmake.configure",
            "CMake Configure",
            JobRisk::Medium,
            true,
            move |ctx| {
                let cancel = ctx.cancellation();
                if let Err(error) = cmake::write_file_api_query(&root) {
                    ctx.emit_output(&format!("aviso: query do file-api falhou: {error}"));
                }
                let command = cmake::configure_command(&root, preset.as_deref(), &toolchain);
                let label = preset.as_deref().map_or_else(
                    || "cmake configure".to_owned(),
                    |name| format!("cmake configure --preset {name}"),
                );
                ctx.emit_event(
                    "event.cmake.started",
                    json!({ "jobId": ctx.id(), "command": label }),
                );

                let outcome = process::stream_command_lines_cancelable(
                    command,
                    &cancel,
                    &mut |_stream, line| ctx.emit_output(&line),
                );
                let (success, exit_code) = match outcome {
                    Ok(exit_status) => (exit_status.success(), exit_status.code().unwrap_or(-1)),
                    Err(error) => {
                        ctx.emit_output(&match &error {
                            process::ProcessError::Spawn(source) => {
                                format!("cmake nao pode ser iniciado: {source}")
                            }
                            process::ProcessError::Wait(source) => {
                                format!("falha aguardando o cmake: {source}")
                            }
                        });
                        (false, -1)
                    }
                };
                let status = cmake::status(&root);
                ctx.emit_event(
                    "event.cmake.finished",
                    json!({
                        "jobId": ctx.id(),
                        "success": success,
                        "exitCode": exit_code,
                        "hasCompileCommands": status.has_compile_commands,
                    }),
                );
                if success {
                    jobs::JobOutcome::Success
                } else {
                    jobs::JobOutcome::Failed
                }
            },
        );

        JsonRpcResponse::success(request_id, json!(JobAcceptedResult { job_id }))
    }

    fn cmake_presets_response(&self, request_id: Option<Value>) -> JsonRpcResponse {
        let root = match self.cmake_workspace_root(request_id.as_ref(), "cmake.presets.list") {
            Ok(root) => root,
            Err(response) => return *response,
        };
        match cmake::list_presets(&root) {
            Ok(presets) => {
                JsonRpcResponse::success(request_id, json!(CmakePresetsResult { presets }))
            }
            Err(message) => JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(JsonRpcErrorCode::InternalError, message, None),
            ),
        }
    }

    fn cmake_targets_response(&self, request_id: Option<Value>) -> JsonRpcResponse {
        let root = match self.cmake_workspace_root(request_id.as_ref(), "cmake.targets.list") {
            Ok(root) => root,
            Err(response) => return *response,
        };
        JsonRpcResponse::success(
            request_id,
            json!(CmakeTargetsResult {
                targets: cmake::list_targets(&root),
            }),
        )
    }

    fn cmake_status_response(&self, request_id: Option<Value>) -> JsonRpcResponse {
        let root = match self.cmake_workspace_root(request_id.as_ref(), "cmake.status") {
            Ok(root) => root,
            Err(response) => return *response,
        };
        let status = cmake::status(&root);
        // O diagnostico da CDB e' mais amplo que o `has_compile_commands`: este
        // olha so o build dir da IDE, e o clangd acha base em `build/` e nos
        // diretorios pai sozinho. Um projeto Meson funciona com
        // `hasCompileCommands: false` — quem sabe disso e' o `cdb`.
        let cdb = crate::cdb::status(&root);
        JsonRpcResponse::success(
            request_id,
            json!(CmakeStatusResult {
                configured: status.configured,
                has_compile_commands: status.has_compile_commands,
                build_dir: status.build_dir.display().to_string(),
                cdb_directory: cdb.directory,
                cdb_stale: cdb.stale,
                cdb_stale_because: cdb.stale_because,
            }),
        )
    }
}

//! Handlers for `cmake.*` requests (`impl Core`): configure como job,
//! presets/targets/status sincronos, todos exigindo workspace `CMake`.

use kinein_protocol::{
    CmakeConfigureParams, CmakePresetsResult, CmakeStatusResult, CmakeTargetsResult,
    JobAcceptedResult, JobRisk, JsonRpcError, JsonRpcErrorCode, JsonRpcResponse, ProjectKind,
};
use serde_json::{Value, json};

use crate::rpc::{no_workspace_response, parse_params};
use crate::{Core, cmake, jobs, process};

impl Core {
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

    /// Valida workspace aberto com kind `CMake`; devolve o root.
    fn cmake_workspace_root(
        &self,
        request_id: Option<&Value>,
        method: &str,
    ) -> Result<std::path::PathBuf, Box<JsonRpcResponse>> {
        let Some(workspace) = self.workspace.as_ref() else {
            return Err(Box::new(no_workspace_response(request_id.cloned(), method)));
        };
        if workspace.kind != ProjectKind::Cmake {
            return Err(Box::new(JsonRpcResponse::failure(
                request_id.cloned(),
                JsonRpcError::new(
                    JsonRpcErrorCode::InvalidParams,
                    format!("{method} requer um workspace CMake"),
                    Some(json!({ "kind": workspace.kind })),
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
                let command = cmake::configure_command(&root, preset.as_deref());
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
        JsonRpcResponse::success(
            request_id,
            json!(CmakeStatusResult {
                configured: status.configured,
                has_compile_commands: status.has_compile_commands,
                build_dir: status.build_dir.display().to_string(),
            }),
        )
    }
}

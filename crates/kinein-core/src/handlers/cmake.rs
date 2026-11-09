//! Handlers for `cmake.*` requests (`impl Core`): configure como job,
//! presets/targets/status sincronos, todos exigindo workspace `CMake`.

use kinein_protocol::{
    BuildSystem, CmakeConfigureParams, CmakePresetsResult, CmakeSelectConfigurationParams,
    CmakeStatusResult, CmakeTargetsResult, JobAcceptedResult, JobRisk, JsonRpcError,
    JsonRpcErrorCode, JsonRpcRequest, JsonRpcResponse,
};
use serde_json::{Value, json};

use crate::rpc::{jobs_unavailable_response, no_workspace_response, parse_params};
use crate::{Core, cmake, jobs, process};

/// Linguagem cujos documentos o `cmake.configure` invalida.
const CPP_LANGUAGE: &str = "cpp";

impl Core {
    /// O build pode configurar/regenerar sem um pedido explicito de configure.
    pub(crate) fn on_build_context_finished(&mut self) {
        let Some(root) = self.workspace_root() else {
            return;
        };
        let changed = self.index.lock().is_ok_and(|index| {
            index.context.as_ref().map_or_else(
                || crate::cdb::status(&root).directory.is_some(),
                |context| !context.has_current_cdb(&root),
            )
        });
        let server_needs_context = self.lsp.as_ref().is_some_and(|lsp| {
            lsp.needs_compile_commands(CPP_LANGUAGE)
                && crate::cdb::status(&root).directory.is_some()
        });
        if changed || server_needs_context {
            self.workspace_edits.invalidate_verifications();
            self.reload_index_context();
            self.on_cmake_configure_finished(true);
            self.on_cmake_finished_for_qml();
        }
    }

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
        // O clangd que subiu sem CDB e' reiniciado, e o `restarted` ja' leva
        // a UI a re-sincronizar (0.4.0e).
        let Some(closed) = lsp.refresh_compile_commands(CPP_LANGUAGE) else {
            return;
        };
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
        &mut self,
        method: &str,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> Option<JsonRpcResponse> {
        match method {
            "cmake.configure" => Some(self.cmake_configure_response(request_id, params)),
            "cmake.presets.list" => Some(self.cmake_presets_response(request_id)),
            "cmake.targets.list" => Some(self.cmake_targets_response(request_id)),
            "cmake.status" => Some(self.cmake_status_response(request_id)),
            "cmake.selectConfiguration" => {
                Some(self.cmake_select_configuration_response(request_id, params))
            }
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
                    format!("{method} requer um projeto CMake"),
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
            return jobs_unavailable_response(request_id, "cmake.configure");
        };

        // O preset: o pedido, o anotado, o unico, ou a recusa (52 §14.2).
        let (preset, preset_origem) = match cmake::choose_preset(&root, parsed.preset) {
            Ok(choice) => choice,
            Err(refusal) => return invalid_params(request_id, refusal),
        };
        let (kit, toolchain) =
            match self.configure_toolchain(&root, preset.as_deref(), preset_origem) {
                Ok(resolved) => resolved,
                Err(reason) => return invalid_params(request_id, reason),
            };
        let extra = self.framework_cmake_args(&root);
        // O perfil de rigor do usuario (M4.5) entra no configure, como no
        // configure que o build faz sozinho.
        let profile = crate::settings::effective_rigor_profile(&root);
        let job_id = jobs.spawn(
            "cmake.configure",
            "CMake Configure",
            JobRisk::Medium,
            true,
            move |ctx| {
                let attempt = match cmake::begin_configure(&root, preset.as_deref(), &kit) {
                    Ok(attempt) => attempt,
                    Err(error) => {
                        ctx.emit_output(&format!(
                            "não foi possível registrar o configure: {error}"
                        ));
                        ctx.emit_event(
                            "event.cmake.finished",
                            json!({
                                "jobId":ctx.id(), "success":false, "exitCode":-1,
                                "hasCompileCommands":cmake::status(&root).has_compile_commands
                            }),
                        );
                        return jobs::JobOutcome::Failed;
                    }
                };
                if let Err(error) = cmake::write_file_api_query(&root) {
                    ctx.emit_output(&format!("aviso: query do file-api falhou: {error}"));
                }
                let command = || {
                    cmake::configure_with_rigor(
                        &root,
                        preset.as_deref(),
                        &toolchain,
                        &extra,
                        profile,
                    )
                };
                let label = preset.as_deref().map_or_else(
                    || "cmake configure".to_owned(),
                    |name| format!("cmake configure --preset {name}"),
                );
                ctx.emit_event(
                    "event.cmake.started",
                    json!({
                        "jobId": ctx.id(),
                        "command": label,
                        "preset": preset,
                        "presetSource": preset_origem,
                    }),
                );

                let (mut success, exit_code) = configure_keeping_definitions(&root, command, ctx);
                if success {
                    cmake::record_preset(&root, preset.as_deref());
                    cmake::record_rigor(&root, profile);
                    if let Err(error) = cmake::complete_configure(attempt) {
                        ctx.emit_output(&format!("não foi possível concluir o registro: {error}"));
                        success = false;
                    }
                }
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

    /// O kit e a toolchain do configure, resolvidos AQUI, na thread do loop:
    /// um job nao alcanca o `Core` (arquitetura/04 §3), e resolver la dentro
    /// exigiria o `ToolDetector`, que e estado do core. As ESCOLHAS sao as do
    /// kit com o nome do preset pedido; um preset escolhido pelo projeto nao
    /// tem kit proprio e usa o padrao (""). O preset anotado no build dir e a
    /// tentativa pendente conservam o kit que o configure usou (C1-13), e um
    /// compilador fixado indisponivel recusa antes do `CMake` (C1-12).
    fn configure_toolchain(
        &self,
        root: &std::path::Path,
        preset: Option<&str>,
        origin: Option<&str>,
    ) -> Result<(String, crate::toolchain::Toolchain), String> {
        let kit = match origin {
            Some("kit" | "configuration") => preset.unwrap_or_default().to_owned(),
            Some("incompleteConfigure" | "buildDir") => cmake::active_kit(root),
            _ => String::new(),
        };
        let toolchain =
            crate::toolchain::Toolchain::resolve_kit(root, &self.detected_tools(), &kit);
        toolchain
            .unavailable_compiler()
            .map_or(Ok((kit, toolchain)), Err)
    }

    /// O que o framework acrescenta ao configure (bloco E do 41): o
    /// `-DPICO_SDK_PATH` do pico-sdk. Nada para os demais; um framework com
    /// ferramenta ausente nao impede o configure (o build.run e' quem recusa).
    fn framework_cmake_args(&self, root: &std::path::Path) -> Vec<String> {
        self.framework_engine(root)
            .ok()
            .flatten()
            .map(|motor| motor.cmake_extra_args())
            .unwrap_or_default()
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
        // O file-api e' a verdade, mas so' existe DEPOIS de um configure. Num
        // projeto recem-aberto ele esta vazio, e e' exatamente ai' que o autor
        // quer escolher bibliotecas — por isso o CMakeLists e' a segunda
        // fonte, e a resposta diz qual das duas respondeu.
        let (targets, origin) = match cmake::list_targets(&root) {
            confirmados if !confirmados.is_empty() => (confirmados, "fileApi"),
            _ => match cmake::targets_from_source(&root) {
                lidos if !lidos.is_empty() => (lidos, "source"),
                vazio => (vazio, "none"),
            },
        };
        JsonRpcResponse::success(
            request_id,
            json!(CmakeTargetsResult {
                origin: origin.to_owned(),
                targets,
            }),
        )
    }

    fn cmake_status_response(&self, request_id: Option<Value>) -> JsonRpcResponse {
        match self.cmake_workspace_root(request_id.as_ref(), "cmake.status") {
            Ok(root) => JsonRpcResponse::success(request_id, json!(status_result(&root))),
            Err(response) => *response,
        }
    }

    /// Grava ou apaga a configuracao escolhida (C-05, consolidacao C §16).
    /// Quando a CDB em uso muda, o contexto recarrega e o clangd recebe a
    /// pasta nova; a resposta e' o `cmake.status` ja' com a escolha.
    fn cmake_select_configuration_response(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let root = match self.cmake_workspace_root(request_id.as_ref(), "cmake.selectConfiguration")
        {
            Ok(root) => root,
            Err(response) => return *response,
        };
        let parsed = match parse_params::<CmakeSelectConfigurationParams>(
            request_id.as_ref(),
            params,
            "cmake.selectConfiguration exige o campo preset: um nome ou null",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        if let Some(name) = parsed.preset.as_deref() {
            match cmake::list_presets(&root) {
                Ok(presets) if presets.iter().any(|preset| preset.name == name) => {}
                Ok(_) => {
                    return invalid_params(
                        request_id,
                        format!("preset de configure inexistente ou oculto: {name}"),
                    );
                }
                Err(reason) => return invalid_params(request_id, reason),
            }
        }
        let before = crate::cdb::status(&root).file(&root);
        if let Err(error) = cmake::select_configuration(&root, parsed.preset.as_deref()) {
            return JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(
                    JsonRpcErrorCode::InternalError,
                    format!("não foi possível gravar a configuração escolhida: {error}"),
                    None,
                ),
            );
        }
        let result = status_result(&root);
        if crate::cdb::status(&root).file(&root) != before {
            self.workspace_edits.invalidate_verifications();
            self.reload_index_context();
            self.on_cmake_configure_finished(true);
        }
        JsonRpcResponse::success(request_id, json!(result))
    }
}

/// O `cmake.status`: o configure da IDE e a CDB em uso, com origem.
fn status_result(root: &std::path::Path) -> CmakeStatusResult {
    let status = cmake::status(root);
    // Os presets entre os quais a pessoa escolhe, quando o configure sem
    // preset seria recusado (52 §14.2).
    let (preset_choices, preset_choices_file) = match cmake::choose_default(root) {
        Err((file, names)) => (names, Some(file.to_owned())),
        Ok(_) => (Vec::new(), None),
    };
    // O diagnostico da CDB e' mais amplo que o `has_compile_commands`: este
    // olha so o build dir da IDE, e a CDB em uso pode estar no `binaryDir` de
    // um preset, em `build/` ou onde o `.clangd` aponta — quem sabe e' o `cdb`.
    let cdb = crate::cdb::status(root);
    CmakeStatusResult {
        configured: status.configured,
        has_compile_commands: status.has_compile_commands,
        build_dir: status.build_dir.display().to_string(),
        preset: status.preset,
        cdb_directory: cdb.directory,
        cdb_stale: cdb.stale,
        cdb_stale_because: cdb.stale_because,
        preset_choices,
        preset_choices_file,
        cdb_origin: cdb.origin,
        cdb_preset: cdb.preset,
        cdb_candidates: cdb.candidates,
        cdb_absence: cdb.absence,
        selected_configuration: cdb.selected,
    }
}

fn configure_outcome(command: std::process::Command, ctx: &jobs::JobContext) -> (bool, i32) {
    let outcome = process::stream_command_lines_cancelable(
        command,
        &ctx.cancellation(),
        &mut |_stream, line| ctx.emit_output(&line),
    );
    match outcome {
        Ok(status) => (
            status.success() && !ctx.is_cancelled(),
            status.code().unwrap_or(-1),
        ),
        Err(error) => {
            ctx.emit_output(&match error {
                process::ProcessError::Spawn(source) => {
                    format!("cmake não pode ser iniciado: {source}")
                }
                process::ProcessError::Wait(source) => {
                    format!("falha aguardando o cmake: {source}")
                }
            });
            (false, -1)
        }
    }
}

/// O configure com a conferencia de C1-14: se o `CMake` recriou o cache sem
/// as definicoes da IDE, a mesma linha roda uma vez sobre o cache novo.
fn configure_keeping_definitions(
    root: &std::path::Path,
    command: impl Fn() -> std::process::Command,
    ctx: &jobs::JobContext,
) -> (bool, i32) {
    let (success, exit_code) = configure_outcome(command(), ctx);
    if !success || !cmake::attempt::lost_ide_definitions(root) {
        return (success, exit_code);
    }
    ctx.emit_output(cmake::attempt::DEFINITIONS_LOST);
    let (success, exit_code) = configure_outcome(command(), ctx);
    if success && cmake::attempt::lost_ide_definitions(root) {
        ctx.emit_output("o cache continua sem as definições da IDE; configure não concluído");
        return (false, exit_code);
    }
    (success, exit_code)
}

/// A recusa do configure sem preset (vários, nada anotado), sem job.
fn invalid_params(request_id: Option<Value>, message: String) -> JsonRpcResponse {
    JsonRpcResponse::failure(
        request_id,
        JsonRpcError::new(JsonRpcErrorCode::InvalidParams, message, None),
    )
}

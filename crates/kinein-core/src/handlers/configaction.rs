//! Handlers for `configAction.*` requests (`impl Core`): lista contextual,
//! preview do plano e aplicacao com o snapshot do preview.
//!
//! Fino de proposito (`ARCHITECTURE.md` §4 regra 2): parse dos params, escopo
//! do workspace, delega ao [`crate::configaction`] e formata a resposta. A
//! unica decisao que mora aqui e a que o dominio nao pode tomar — disparar o
//! job do `cargo.check`, porque o `JobManager` e do `Core` (arquitetura/04 §3).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use kinein_protocol::{
    ConfigActionApplyParams, ConfigActionApplyResult, ConfigActionListParams,
    ConfigActionPreviewParams, JsonRpcError, JsonRpcErrorCode, JsonRpcResponse,
};
use serde_json::{Value, json};

use crate::Core;
use crate::configaction::{self, ConfigActionError, ConfigActionOutcome};
use crate::rpc::{no_workspace_response, parse_params};

/// A receita cuja previa/aplicacao confere uma instalacao fora do laco.
const COMPILER_ACTION: &str = "cmake.selectCCompiler";

impl Core {
    /// Roteia os metodos `configAction.*`; `None` quando o metodo nao e deles.
    pub(crate) fn configaction_request_response(
        &mut self,
        method: &str,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> Option<JsonRpcResponse> {
        match method {
            "configAction.list" => Some(self.configaction_list_response(request_id, params)),
            "configAction.preview" => Some(self.configaction_preview_response(request_id, params)),
            "configAction.apply" => Some(self.configaction_apply_response(request_id, params)),
            _ => None,
        }
    }

    fn configaction_list_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(workspace) = self.workspace.as_ref() else {
            return no_workspace_response(request_id, "configAction.list");
        };
        let parsed = match parse_params::<ConfigActionListParams>(
            request_id.as_ref(),
            params,
            "configAction.list aceita o campo opcional includeHiddenByScope",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let root = std::path::PathBuf::from(&workspace.root);
        let result = configaction::list(
            &root,
            &workspace.capabilities.build_systems,
            parsed.include_hidden_by_scope,
            &self.configaction_inputs(&root),
        );
        JsonRpcResponse::success(request_id, json!(result))
    }

    fn configaction_preview_response(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(workspace) = self.workspace.as_ref() else {
            return no_workspace_response(request_id, "configAction.preview");
        };
        let parsed = match parse_params::<ConfigActionPreviewParams>(
            request_id.as_ref(),
            params,
            "configAction.preview requer o campo id (params e opcional)",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let root = PathBuf::from(&workspace.root);
        let build_systems = workspace.capabilities.build_systems.clone();
        let compiler = (parsed.id == COMPILER_ACTION).then(|| parsed.params.clone());
        let preview = move |_: &mut Self, request_id, root: &Path, inputs: configaction::Inputs| {
            match configaction::preview(root, &build_systems, &parsed, &inputs) {
                Ok(result) => JsonRpcResponse::success(request_id, json!(result)),
                Err(error) => configaction_error_response(request_id, &error),
            }
        };
        if let Some(params) = compiler {
            return self.defer_compiler_choice(request_id, root, &params, preview);
        }
        let inputs = self.configaction_inputs(&root);
        preview(self, request_id, &root, inputs)
    }

    fn configaction_apply_response(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(workspace) = self.workspace.as_ref() else {
            return no_workspace_response(request_id, "configAction.apply");
        };
        let parsed = match parse_params::<ConfigActionApplyParams>(
            request_id.as_ref(),
            params,
            "configAction.apply requer o campo id (params e expected são opcionais)",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let root = PathBuf::from(&workspace.root);
        let build_systems = workspace.capabilities.build_systems.clone();
        let compiler = (parsed.id == COMPILER_ACTION).then(|| parsed.params.clone());
        let apply =
            move |core: &mut Self, request_id, root: &Path, inputs: configaction::Inputs| {
                let outcome = match configaction::apply(root, &build_systems, &parsed, &inputs) {
                    Ok(outcome) => outcome,
                    Err(error) => return configaction_error_response(request_id, &error),
                };
                match outcome {
                    ConfigActionOutcome::Done { message, files } => {
                        if parsed.id == COMPILER_ACTION {
                            // O mesmo que `toolchain.set` faz depois de gravar o kit.
                            core.configure_clangd_from_toolchain(root);
                            core.emit_project_changed();
                        }
                        JsonRpcResponse::success(
                            request_id,
                            json!(ConfigActionApplyResult {
                                id: parsed.id,
                                message,
                                files,
                                job_id: None,
                            }),
                        )
                    }
                    ConfigActionOutcome::Job { method } => {
                        core.configaction_job_response(request_id, &parsed.id, method)
                    }
                }
            };
        if let Some(params) = compiler {
            return self.defer_compiler_choice(request_id, root, &params, apply);
        }
        let inputs = self.configaction_inputs(&root);
        apply(self, request_id, &root, inputs)
    }

    /// `cmake.selectCCompiler` confere o executavel com processos (versao e
    /// sonda C): fora do laco, como a criacao (C1-03). A continuacao recusa
    /// se o workspace mudou e segue o caminho comum de preview/apply com o
    /// plano do `toolchain` (consolidacao C §15).
    fn defer_compiler_choice<F>(
        &mut self,
        request_id: Option<Value>,
        root: PathBuf,
        params: &BTreeMap<String, String>,
        finish: F,
    ) -> JsonRpcResponse
    where
        F: FnOnce(&mut Self, Option<Value>, &Path, configaction::Inputs) -> JsonRpcResponse
            + Send
            + 'static,
    {
        let compiler =
            configaction::optional_param(params, "compiler").map(|value| value.trim().to_owned());
        let kit = configaction::optional_param(params, "kit").map_or_else(
            || crate::cmake::active_kit(&root),
            |value| value.trim().to_owned(),
        );
        let detector = self.detector.clone();
        self.defer_then(
            request_id,
            move || {
                let checked = compiler
                    .as_deref()
                    .map(crate::workspace::validate_c_driver)
                    .transpose();
                (compiler, checked, detector.c_environments())
            },
            move |core, request_id, (compiler, checked, detected)| {
                if core.workspace_root().as_deref() != Some(root.as_path()) {
                    return JsonRpcResponse::failure(
                        request_id,
                        JsonRpcError::new(
                            JsonRpcErrorCode::InvalidParams,
                            "o workspace mudou durante a conferência da instalação; repita a \
                             ação no projeto atual",
                            None,
                        ),
                    );
                }
                let mut inputs = core.configaction_inputs(&root);
                // O kit padrao configura com o preset da configuracao ativa.
                let preset = if kit.is_empty() {
                    crate::cmake::active_configuration(&root)
                        .ok()
                        .and_then(|configuration| configuration.preset)
                } else {
                    Some(kit.clone())
                };
                inputs.compiler_choice = Some(checked.and_then(|version| {
                    let measured = crate::toolchain::Measured {
                        version: version.as_deref(),
                        detected: &detected,
                        preset: preset.as_deref(),
                    };
                    crate::toolchain::plan_c_compiler(
                        &root,
                        &core.detected_tools(),
                        &kit,
                        compiler.as_deref(),
                        measured,
                    )
                }));
                finish(core, request_id, &root, inputs)
            },
        )
    }

    /// O que as acoes precisam de outros donos (0.4.1a): o kit do configure
    /// exportado como toolchain file, pelo `toolchain`, e os drivers C que o
    /// detector encontra (consolidacao C §15).
    fn configaction_inputs(&self, root: &Path) -> configaction::Inputs {
        let kit = crate::cmake::active_kit(root);
        let toolchain =
            crate::toolchain::Toolchain::resolve_kit(root, &self.detected_tools(), &kit);
        configaction::Inputs {
            kit_export: Some(toolchain.export(&kit)),
            c_installations: self.detector.c_installations(),
            compiler_choice: None,
        }
    }

    /// Acoes de efeito `job` reusam o metodo que ja existe, sem executor novo.
    fn configaction_job_response(
        &self,
        request_id: Option<Value>,
        id: &str,
        method: &str,
    ) -> JsonRpcResponse {
        let delegated = if method == "platformio.compiledb" {
            Some(self.platformio_compiledb_response(request_id.clone()))
        } else {
            self.cargo_request_response(method, request_id.clone(), None)
        };
        let Some(response) = delegated else {
            return JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(
                    JsonRpcErrorCode::InternalError,
                    format!("a ação {id} aponta para o método desconhecido {method}"),
                    None,
                ),
            );
        };
        if response.error.is_some() {
            return response;
        }
        let job_id = response
            .result
            .as_ref()
            .and_then(|result| result.get("jobId"))
            .and_then(Value::as_str)
            .map(str::to_owned);
        JsonRpcResponse::success(
            request_id,
            json!(ConfigActionApplyResult {
                id: id.to_owned(),
                message: format!("{method} disparado como job"),
                files: Vec::new(),
                job_id,
            }),
        )
    }
}

/// Erro de dominio vira erro estruturado do protocolo.
fn configaction_error_response(
    request_id: Option<Value>,
    error: &ConfigActionError,
) -> JsonRpcResponse {
    let code = if error.changed_path().is_some() {
        JsonRpcErrorCode::FileChanged
    } else if error.is_invalid_params() {
        JsonRpcErrorCode::InvalidParams
    } else {
        JsonRpcErrorCode::InternalError
    };
    let details = error.changed_path().map(|path| json!({ "path": path }));
    JsonRpcResponse::failure(
        request_id,
        JsonRpcError::new(code, error.to_string(), details),
    )
}

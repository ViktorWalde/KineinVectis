//! O `cmake.status` e o `cmake.selectConfiguration` (`impl Core`): a
//! configuracao ativa como a UI a ve (C1-20, consolidacao C §17) e a CDB em
//! uso (C-05, §16).

use kinein_protocol::{
    CmakeSelectConfigurationParams, CmakeStatusResult, JsonRpcError, JsonRpcErrorCode,
    JsonRpcResponse,
};
use serde_json::{Value, json};

use super::cmake::invalid_params;
use crate::rpc::parse_params;
use crate::{Core, cmake};

impl Core {
    pub(super) fn cmake_status_response(&self, request_id: Option<Value>) -> JsonRpcResponse {
        match self.cmake_workspace_root(request_id.as_ref(), "cmake.status") {
            Ok(root) => JsonRpcResponse::success(request_id, json!(status_result(&root))),
            Err(response) => *response,
        }
    }

    /// Grava ou apaga a configuracao escolhida (C-05, consolidacao C §16).
    /// Quando a CDB em uso muda, o contexto recarrega e o clangd recebe a
    /// pasta nova; a resposta e' o `cmake.status` ja' com a escolha.
    pub(super) fn cmake_select_configuration_response(
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
        let before = (
            crate::cdb::status(&root).file(&root),
            cmake::active_dir(&root).ok(),
        );
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
        // A CDB em uso ou a pasta ativa mudou: contexto, clangd e `qmlls`
        // seguem a configuracao (C1-21).
        let after = (
            crate::cdb::status(&root).file(&root),
            cmake::active_dir(&root).ok(),
        );
        if after != before {
            self.workspace_edits.invalidate_verifications();
            self.reload_index_context();
            self.on_cmake_configure_finished(true);
            self.on_cmake_finished_for_qml();
        }
        JsonRpcResponse::success(request_id, json!(result))
    }
}

/// O `cmake.status`: o configure da IDE e a CDB em uso, com origem.
fn status_result(root: &std::path::Path) -> CmakeStatusResult {
    let status = cmake::status(root);
    // Os presets entre os quais a pessoa escolhe, quando o configure sem
    // preset seria recusado (52 §14.2). Com o registro da configuracao ativa
    // recusado (preset que saiu, registro ilegivel), as escolhas do
    // automatico: configurar uma delas grava o registro de novo (C1-20).
    let (preset_choices, preset_choices_file) = match &status.configuration {
        Err(cmake::Refusal::Several(file, names)) => (names.clone(), Some((*file).to_owned())),
        Err(cmake::Refusal::Refused(_)) if cmake::selected_configuration(root) != Ok(None) => {
            match cmake::default_preset(root) {
                cmake::PresetDefault::Several(file, names) => (names, Some(file.to_owned())),
                cmake::PresetDefault::One(name, file) => (vec![name], Some(file.to_owned())),
                cmake::PresetDefault::Absent | cmake::PresetDefault::Refused(_) => {
                    (Vec::new(), None)
                }
            }
        }
        Ok(_) | Err(cmake::Refusal::Refused(_)) => (Vec::new(), None),
    };
    let (configuration_preset, configuration_source, configuration_refused) =
        match &status.configuration {
            Ok(configuration) => (
                configuration.preset.clone(),
                Some(configuration.source.to_owned()),
                None,
            ),
            Err(refusal) => (None, None, Some(refusal.reason())),
        };
    // O diagnostico da CDB e' mais amplo que o `has_compile_commands`: este
    // olha so a pasta da configuracao ativa, e a CDB em uso pode estar em
    // outra pasta de preset, em `build/` ou onde o `.clangd` aponta — quem
    // sabe e' o `cdb`.
    let cdb = crate::cdb::status(root);
    // C1-22: with a multi-config generator, what the IDE builds and what the
    // terminal does.
    let build = status
        .build_dir
        .as_deref()
        .and_then(cmake::build_configuration::of);
    CmakeStatusResult {
        configured: status.configured,
        has_compile_commands: status.has_compile_commands,
        build_dir: status
            .build_dir
            .as_ref()
            .map(|directory| directory.display().to_string()),
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
        configuration_preset,
        configuration_source,
        configuration_refused,
        configurations: cmake::configurations(root),
        build_configuration: build.as_ref().map(|build| build.name.clone()),
        default_build_configuration: build.and_then(|build| build.default),
        project_problem: super::workspace::project_problem(root),
    }
}

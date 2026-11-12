//! Handlers for `settings.*` requests (`impl Core`).
//!
//! `settings.get` resolve `default ← global ← workspace` (o global existe
//! sem workspace aberto); `settings.set` faz merge parcial no escopo e
//! responde o estado completo novo (padrão de mutação do repo).

use kinein_protocol::{
    JsonRpcError, JsonRpcErrorCode, JsonRpcResponse, SettingsResult, SettingsScope,
    SettingsSetParams, SettingsValues,
};
use serde_json::{Value, json};

use crate::rpc::{no_workspace_response, parse_params};
use crate::{Core, settings};

impl Core {
    /// Roteia os metodos `settings.*`; `None` quando nao e de settings.
    pub(crate) fn settings_request_response(
        &self,
        method: &str,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> Option<JsonRpcResponse> {
        match method {
            "settings.get" => Some(self.settings_get_response(request_id)),
            "settings.set" => Some(self.settings_set_response(request_id, params)),
            _ => None,
        }
    }

    fn settings_get_response(&self, request_id: Option<Value>) -> JsonRpcResponse {
        let global = settings::read_global();
        let workspace = self
            .workspace_root()
            .map(|root| settings::read_workspace(&root))
            .unwrap_or_default();
        JsonRpcResponse::success(request_id, json!(settings_result(global, workspace)))
    }

    fn settings_set_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let parsed = match parse_params::<SettingsSetParams>(
            request_id.as_ref(),
            params,
            "settings.set requer scope (global|workspace) e values",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        if let Some(message) = validate_values(&parsed.values) {
            return invalid_settings_response(request_id, message);
        }

        let written = match parsed.scope {
            SettingsScope::Global => settings::set_global(&parsed.values),
            SettingsScope::Workspace => {
                let Some(root) = self.workspace_root() else {
                    return no_workspace_response(request_id, "settings.set");
                };
                settings::set_workspace(&root, &parsed.values)
            }
        };
        // INF-07: o arquivo que a IDE nao entende fica como estava, e o
        // motivo e' o mesmo do `settings.get`.
        match written {
            Ok(_) => {}
            Err(settings::SetError::Refused(message)) => {
                return settings_failure(request_id, JsonRpcErrorCode::InvalidRequest, message);
            }
            Err(settings::SetError::Write(message)) => {
                return settings_failure(request_id, JsonRpcErrorCode::InternalError, message);
            }
        }

        // Reconstroi o estado completo a partir do disco.
        self.settings_get_response(request_id)
    }
}

/// Monta o `SettingsResult` (efetivo + os values de cada escopo + o motivo
/// do arquivo que existe e nao foi lido, `0.220.0`).
fn settings_result(global: settings::Stored, workspace: settings::Stored) -> SettingsResult {
    SettingsResult {
        settings: settings::resolve(&global.values, &workspace.values),
        global: global.values,
        workspace: workspace.values,
        global_problem: global.problem,
        workspace_problem: workspace.problem,
    }
}

/// `None` quando os values sao aceitaveis; senao a mensagem do erro.
fn validate_values(values: &SettingsValues) -> Option<String> {
    if let Some(size) = values.editor_font_size
        && !(settings::MIN_EDITOR_FONT_SIZE..=settings::MAX_EDITOR_FONT_SIZE).contains(&size)
    {
        return Some(format!(
            "editorFontSize deve estar entre {} e {}",
            settings::MIN_EDITOR_FONT_SIZE,
            settings::MAX_EDITOR_FONT_SIZE
        ));
    }
    for (name, width) in [
        ("explorerWidth", values.explorer_width),
        ("contextWidth", values.context_width),
        ("outlineWidth", values.outline_width),
    ] {
        if width.is_some_and(|value| {
            !(settings::MIN_PANEL_WIDTH..=settings::MAX_PANEL_WIDTH).contains(&value)
        }) {
            return Some(format!(
                "{name} deve estar entre {} e {}",
                settings::MIN_PANEL_WIDTH,
                settings::MAX_PANEL_WIDTH
            ));
        }
    }
    if values.assistant_terminal_width.is_some_and(|width| {
        !(settings::MIN_ASSISTANT_TERMINAL_WIDTH..=settings::MAX_ASSISTANT_TERMINAL_WIDTH)
            .contains(&width)
    }) {
        return Some(format!(
            "assistantTerminalWidth deve estar entre {} e {}",
            settings::MIN_ASSISTANT_TERMINAL_WIDTH,
            settings::MAX_ASSISTANT_TERMINAL_WIDTH
        ));
    }
    if values.bottom_panel_height.is_some_and(|height| {
        !(settings::MIN_BOTTOM_PANEL_HEIGHT..=settings::MAX_BOTTOM_PANEL_HEIGHT).contains(&height)
    }) {
        return Some(format!(
            "bottomPanelHeight deve estar entre {} e {}",
            settings::MIN_BOTTOM_PANEL_HEIGHT,
            settings::MAX_BOTTOM_PANEL_HEIGHT
        ));
    }
    // O layout (0.146.0) e' um retrato que a UI monta: tem de ser objeto com
    // `schemaVersion`. O conteudo, a UI confere ao aplicar (cada tamanho passa
    // pelos limites do ShellController); aqui so' se recusa lixo.
    if let Some(layout) = &values.layout
        && layout
            .get("schemaVersion")
            .and_then(serde_json::Value::as_u64)
            .is_none_or(|version| version == 0)
    {
        return Some("layout deve ser um objeto com schemaVersion >= 1".to_owned());
    }
    None
}

fn invalid_settings_response(request_id: Option<Value>, message: String) -> JsonRpcResponse {
    settings_failure(request_id, JsonRpcErrorCode::InvalidParams, message)
}

fn settings_failure(
    request_id: Option<Value>,
    code: JsonRpcErrorCode,
    message: String,
) -> JsonRpcResponse {
    JsonRpcResponse::failure(request_id, JsonRpcError::new(code, message, None))
}

#[cfg(test)]
mod tests {
    use kinein_protocol::SettingsValues;

    use crate::settings::Stored;

    /// O motivo de cada escopo vai no campo dele, e os valores vazios dele
    /// deixam o efetivo cair para o escopo de baixo (INF-07).
    #[test]
    fn each_scope_problem_goes_in_its_own_field() {
        let global = Stored {
            values: SettingsValues {
                editor_font_size: Some(18),
                ..SettingsValues::default()
            },
            problem: None,
        };
        let workspace = Stored {
            values: SettingsValues::default(),
            problem: Some(".kinein/settings.json nao tem schemaVersion".to_owned()),
        };
        let result = super::settings_result(global, workspace);
        assert_eq!(result.settings.editor_font_size, 18);
        assert_eq!(result.global_problem, None);
        assert_eq!(
            result.workspace_problem.as_deref(),
            Some(".kinein/settings.json nao tem schemaVersion")
        );
        let swapped = super::settings_result(
            Stored {
                values: SettingsValues::default(),
                problem: Some("/x/settings.json nao e' um objeto JSON".to_owned()),
            },
            Stored::default(),
        );
        assert_eq!(
            swapped.global_problem.as_deref(),
            Some("/x/settings.json nao e' um objeto JSON")
        );
        assert_eq!(swapped.workspace_problem, None);
        assert_eq!(swapped.settings.editor_font_size, 14);
    }
}

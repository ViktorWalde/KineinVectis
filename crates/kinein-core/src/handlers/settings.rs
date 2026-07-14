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
        let global = settings::load_global();
        let workspace = self
            .workspace_root()
            .map(|root| settings::load_workspace(&root))
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

        match parsed.scope {
            SettingsScope::Global => {
                if let Err(message) = settings::set_global(&parsed.values) {
                    return internal_settings_response(request_id, message);
                }
            }
            SettingsScope::Workspace => {
                let Some(root) = self.workspace_root() else {
                    return no_workspace_response(request_id, "settings.set");
                };
                if let Err(message) = settings::set_workspace(&root, &parsed.values) {
                    return internal_settings_response(request_id, message);
                }
            }
        }

        // Reconstroi o estado completo a partir do disco.
        self.settings_get_response(request_id)
    }
}

/// Monta o `SettingsResult` (efetivo + os values de cada escopo).
fn settings_result(global: SettingsValues, workspace: SettingsValues) -> SettingsResult {
    SettingsResult {
        settings: settings::resolve(&global, &workspace),
        global,
        workspace,
    }
}

/// `None` quando os values sao aceitaveis; senao a mensagem do erro.
fn validate_values(values: &SettingsValues) -> Option<String> {
    if let Some(size) = values.editor_font_size {
        if !(settings::MIN_EDITOR_FONT_SIZE..=settings::MAX_EDITOR_FONT_SIZE).contains(&size) {
            return Some(format!(
                "editorFontSize deve estar entre {} e {}",
                settings::MIN_EDITOR_FONT_SIZE,
                settings::MAX_EDITOR_FONT_SIZE
            ));
        }
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
    if values.bottom_panel_height.is_some_and(|height| {
        !(settings::MIN_BOTTOM_PANEL_HEIGHT..=settings::MAX_BOTTOM_PANEL_HEIGHT).contains(&height)
    }) {
        return Some(format!(
            "bottomPanelHeight deve estar entre {} e {}",
            settings::MIN_BOTTOM_PANEL_HEIGHT,
            settings::MAX_BOTTOM_PANEL_HEIGHT
        ));
    }
    None
}

fn invalid_settings_response(request_id: Option<Value>, message: String) -> JsonRpcResponse {
    JsonRpcResponse::failure(
        request_id,
        JsonRpcError::new(JsonRpcErrorCode::InvalidParams, message, None),
    )
}

fn internal_settings_response(request_id: Option<Value>, message: String) -> JsonRpcResponse {
    JsonRpcResponse::failure(
        request_id,
        JsonRpcError::new(JsonRpcErrorCode::InternalError, message, None),
    )
}

//! Settings payloads (`settings.*`).
//!
//! Two scopes: global (XDG) and per-workspace. `SettingsValues` carries the
//! fields explicitly set at one scope (absent = not set there);
//! `EffectiveSettings` is the resolved view after `default ← global ←
//! workspace`.

use serde::{Deserialize, Serialize};

/// Rigor profile that regulates what the IDE runs for the USER's project
/// (clippy lints, build warnings) — never the Kinein repo's own gate.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RigorProfile {
    /// Pedantic + nursery clippy, warnings as errors (product identity).
    #[default]
    Strict,
    /// Default clippy lints, warnings shown but not fatal.
    Balanced,
    /// Only correctness lints (real bugs); everything else allowed.
    Relaxed,
}

/// Scope a settings mutation targets.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SettingsScope {
    /// Global settings (XDG config, all workspaces).
    Global,
    /// Per-workspace settings (`.kinein/settings.json`), override global.
    Workspace,
}

/// Values explicitly set at one scope.
///
/// Absent fields fall through to the lower-priority scope (workspace →
/// global → default). No `deny_unknown_fields`: forward compatibility (a
/// newer field written by a future core is ignored, not rejected).
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsValues {
    /// Format the buffer on save (Ctrl+S).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format_on_save: Option<bool>,
    /// Editor font size in points.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub editor_font_size: Option<u32>,
    /// Auto-close bracket/quote pairs while typing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auto_close_pairs: Option<bool>,
    /// Save the buffer by itself (`0.123.0`, Etapa 2 F3): after a typing
    /// pause, on tab switch and when the editor loses focus. Absent = on.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auto_save: Option<bool>,
    /// Rigor profile for the user's build/quality runs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rigor_profile: Option<RigorProfile>,
    /// Preferred Project explorer width in logical pixels.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub explorer_width: Option<u32>,
    /// Preferred KV Context width in logical pixels.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_width: Option<u32>,
    /// Preferred width of an active external AI terminal in logical pixels.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assistant_terminal_width: Option<u32>,
    /// Preferred bottom tool-window height in logical pixels.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bottom_panel_height: Option<u32>,
    /// Preferred editor Structure width in logical pixels.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outline_width: Option<u32>,
    /// Whether the editor Structure tool window is explicitly collapsed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outline_collapsed: Option<bool>,
}

/// Settings after merging defaults, global and workspace scopes.
// Sao FLAGS de preferencia, nao um estado com combinacoes proibidas: o
// `autoSave` (0.123.0) foi o quarto, e um enum para "quatro botoes" seria
// cerimonia.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EffectiveSettings {
    /// Effective format-on-save flag.
    pub format_on_save: bool,
    /// Effective editor font size.
    pub editor_font_size: u32,
    /// Effective auto-close flag.
    pub auto_close_pairs: bool,
    /// Effective autosave flag.
    pub auto_save: bool,
    /// Effective rigor profile.
    pub rigor_profile: RigorProfile,
    /// Effective Project explorer width.
    pub explorer_width: u32,
    /// Effective KV Context width.
    pub context_width: u32,
    /// Effective width of an active external AI terminal.
    pub assistant_terminal_width: u32,
    /// Effective bottom tool-window height.
    pub bottom_panel_height: u32,
    /// Effective editor Structure width.
    pub outline_width: u32,
    /// Effective explicit Structure collapsed state.
    pub outline_collapsed: bool,
}

/// Result payload for `settings.get` / `settings.set`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsResult {
    /// Resolved effective settings the UI applies.
    pub settings: EffectiveSettings,
    /// Values explicitly set at the global scope.
    pub global: SettingsValues,
    /// Values explicitly set at the workspace scope (empty when no
    /// workspace is open).
    pub workspace: SettingsValues,
}

/// Parameters for `settings.set`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SettingsSetParams {
    /// Scope to merge `values` into.
    pub scope: SettingsScope,
    /// Partial values to merge (absent fields keep their stored value).
    pub values: SettingsValues,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{SettingsScope, SettingsSetParams, SettingsValues};

    #[test]
    fn set_params_parse_scope_and_partial_values() {
        let parsed = serde_json::from_value::<SettingsSetParams>(json!({
            "scope": "workspace",
            "values": { "editorFontSize": 16 },
        }))
        .unwrap();
        assert_eq!(parsed.scope, SettingsScope::Workspace);
        assert_eq!(parsed.values.editor_font_size, Some(16));
        assert_eq!(parsed.values.format_on_save, None);
    }

    #[test]
    fn values_serialize_camel_case_and_omit_absent() {
        let value = serde_json::to_value(SettingsValues {
            format_on_save: Some(true),
            editor_font_size: None,
            auto_close_pairs: Some(false),
            auto_save: None,
            rigor_profile: Some(super::RigorProfile::Relaxed),
            ..SettingsValues::default()
        })
        .unwrap();
        assert_eq!(value["formatOnSave"], true);
        assert_eq!(value["autoClosePairs"], false);
        assert_eq!(value["rigorProfile"], "relaxed");
        assert!(value.get("editorFontSize").is_none());
    }

    #[test]
    fn unknown_field_is_ignored_for_forward_compat() {
        let parsed = serde_json::from_value::<SettingsValues>(json!({
            "editorFontSize": 18,
            "somethingNewFromTheFuture": 7,
        }))
        .unwrap();
        assert_eq!(parsed.editor_font_size, Some(18));
    }
}

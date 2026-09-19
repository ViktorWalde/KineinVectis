//! Settings persistidos em dois níveis (`settings.*`).
//!
//! GLOBAL: XDG (`$XDG_CONFIG_HOME` ou `~/.config`) →
//! `kinein-vectis/settings.json`, vale para todos os workspaces.
//! WORKSPACE: `.kinein/settings.json`, sobrepõe o global campo a campo.
//! Ambos com `schemaVersion` + arquivo inválido/desconhecido tratado como
//! vazio (nunca quebra o fluxo), no mesmo padrão de `runconfig.rs`.

use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use kinein_protocol::{EffectiveSettings, SettingsValues};

/// Versão do schema escrita por este core.
const SETTINGS_SCHEMA_VERSION: u32 = 1;

/// Default do tamanho de fonte do editor (casa com `Theme.fontSizeEditor`).
const DEFAULT_EDITOR_FONT_SIZE: u32 = 14;

/// Limites aceitos para o tamanho de fonte do editor.
pub const MIN_EDITOR_FONT_SIZE: u32 = 8;
/// Limite superior aceito para o tamanho de fonte do editor.
pub const MAX_EDITOR_FONT_SIZE: u32 = 40;
/// Limits for persisted horizontal tool-window widths.
pub const MIN_PANEL_WIDTH: u32 = 160;
/// Upper limit for persisted horizontal tool-window widths.
pub const MAX_PANEL_WIDTH: u32 = 600;
/// Minimum persisted width of an active external AI terminal.
pub const MIN_ASSISTANT_TERMINAL_WIDTH: u32 = 300;
/// Maximum persisted width of an active external AI terminal.
pub const MAX_ASSISTANT_TERMINAL_WIDTH: u32 = 720;
/// Limits for persisted bottom tool-window height.
pub const MIN_BOTTOM_PANEL_HEIGHT: u32 = 120;
/// Upper limit for persisted bottom tool-window height.
pub const MAX_BOTTOM_PANEL_HEIGHT: u32 = 700;

/// Representação em disco de um `settings.json` (global ou workspace).
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SettingsFile {
    schema_version: u32,
    #[serde(flatten)]
    values: SettingsValues,
}

/// Caminho do `settings.json` global (XDG).
#[must_use]
pub fn global_path() -> PathBuf {
    global_dir().join("settings.json")
}

/// Shared XDG directory for global Kinein state files.
#[must_use]
pub(crate) fn global_dir() -> PathBuf {
    config_home().join("kinein-vectis")
}

/// Caminho do `settings.json` por-workspace.
#[must_use]
pub fn workspace_path(root: &Path) -> PathBuf {
    root.join(".kinein").join("settings.json")
}

/// Base de config do XDG: `$XDG_CONFIG_HOME`, senão `$HOME/.config`.
fn config_home() -> PathBuf {
    if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME") {
        if !xdg.is_empty() {
            return PathBuf::from(xdg);
        }
    }
    let home = std::env::var_os("HOME").unwrap_or_default();
    PathBuf::from(home).join(".config")
}

fn read_values(path: &Path) -> SettingsValues {
    let Ok(body) = fs::read_to_string(path) else {
        return SettingsValues::default();
    };
    match serde_json::from_str::<SettingsFile>(&body) {
        Ok(file) if file.schema_version == SETTINGS_SCHEMA_VERSION => file.values,
        _ => SettingsValues::default(),
    }
}

fn write_values(path: &Path, values: &SettingsValues) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("falha criando {}: {error}", parent.display()))?;
    }
    let file = SettingsFile {
        schema_version: SETTINGS_SCHEMA_VERSION,
        values: values.clone(),
    };
    let body = serde_json::to_string_pretty(&file)
        .map_err(|error| format!("falha serializando settings: {error}"))?;
    fs::write(path, body).map_err(|error| format!("falha escrevendo settings: {error}"))
}

/// Values setados no escopo global (arquivo ausente/invalido → vazio).
#[must_use]
pub fn load_global() -> SettingsValues {
    read_values(&global_path())
}

/// Values setados no escopo do workspace.
#[must_use]
pub fn load_workspace(root: &Path) -> SettingsValues {
    read_values(&workspace_path(root))
}

/// Resolve `default ← global ← workspace` campo a campo.
#[must_use]
pub fn resolve(global: &SettingsValues, workspace: &SettingsValues) -> EffectiveSettings {
    EffectiveSettings {
        format_on_save: workspace
            .format_on_save
            .or(global.format_on_save)
            .unwrap_or(false),
        editor_font_size: workspace
            .editor_font_size
            .or(global.editor_font_size)
            .unwrap_or(DEFAULT_EDITOR_FONT_SIZE),
        auto_close_pairs: workspace
            .auto_close_pairs
            .or(global.auto_close_pairs)
            .unwrap_or(true),
        // Ligado por padrao: decisao do autor (Etapa 2, 2026-09-18); o
        // rascunho do seguranca/23 continua sendo a rede entre um salvar e
        // outro.
        auto_save: workspace.auto_save.or(global.auto_save).unwrap_or(true),
        rigor_profile: workspace
            .rigor_profile
            .or(global.rigor_profile)
            .unwrap_or_default(),
        explorer_width: workspace
            .explorer_width
            .or(global.explorer_width)
            .unwrap_or(280),
        context_width: workspace
            .context_width
            .or(global.context_width)
            .unwrap_or(360),
        assistant_terminal_width: workspace
            .assistant_terminal_width
            .or(global.assistant_terminal_width)
            .unwrap_or(640),
        bottom_panel_height: workspace
            .bottom_panel_height
            .or(global.bottom_panel_height)
            .unwrap_or(260),
        outline_width: workspace
            .outline_width
            .or(global.outline_width)
            .unwrap_or(220),
        outline_collapsed: workspace
            .outline_collapsed
            .or(global.outline_collapsed)
            .unwrap_or(false),
        rail_expanded: workspace
            .rail_expanded
            .or(global.rail_expanded)
            .unwrap_or(false),
    }
}

/// Perfil de rigor efetivo (global ← workspace) lido do disco, para regular o
/// build/quality do usuario (fatia M4.5).
#[must_use]
pub fn effective_rigor_profile(root: &Path) -> kinein_protocol::RigorProfile {
    resolve(&load_global(), &load_workspace(root)).rigor_profile
}

/// Merge de `incoming` sobre `base` (campo `Some` sobrescreve; `None` mantém).
fn merge(base: &SettingsValues, incoming: &SettingsValues) -> SettingsValues {
    SettingsValues {
        format_on_save: incoming.format_on_save.or(base.format_on_save),
        editor_font_size: incoming.editor_font_size.or(base.editor_font_size),
        auto_close_pairs: incoming.auto_close_pairs.or(base.auto_close_pairs),
        auto_save: incoming.auto_save.or(base.auto_save),
        rigor_profile: incoming.rigor_profile.or(base.rigor_profile),
        explorer_width: incoming.explorer_width.or(base.explorer_width),
        context_width: incoming.context_width.or(base.context_width),
        assistant_terminal_width: incoming
            .assistant_terminal_width
            .or(base.assistant_terminal_width),
        bottom_panel_height: incoming.bottom_panel_height.or(base.bottom_panel_height),
        outline_width: incoming.outline_width.or(base.outline_width),
        outline_collapsed: incoming.outline_collapsed.or(base.outline_collapsed),
        rail_expanded: incoming.rail_expanded.or(base.rail_expanded),
    }
}

/// Aplica `values` (parcial) ao escopo global e devolve o novo estado global.
pub fn set_global(values: &SettingsValues) -> Result<SettingsValues, String> {
    let merged = merge(&load_global(), values);
    write_values(&global_path(), &merged)?;
    Ok(merged)
}

/// Aplica `values` (parcial) ao escopo do workspace.
pub fn set_workspace(root: &Path, values: &SettingsValues) -> Result<SettingsValues, String> {
    let merged = merge(&load_workspace(root), values);
    write_values(&workspace_path(root), &merged)?;
    Ok(merged)
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use kinein_protocol::SettingsValues;

    use super::{merge, read_values, resolve, write_values};

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("kinein-settings-tests")
            .join(format!("{}-{name}", std::process::id()));
        if dir.exists() {
            std::fs::remove_dir_all(&dir).unwrap();
        }
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn resolve_workspace_overrides_global_and_defaults_fill() {
        let global = SettingsValues {
            editor_font_size: Some(16),
            format_on_save: Some(true),
            rigor_profile: Some(kinein_protocol::RigorProfile::Relaxed),
            ..SettingsValues::default()
        };
        let workspace = SettingsValues {
            editor_font_size: Some(18),
            ..SettingsValues::default()
        };
        let effective = resolve(&global, &workspace);
        // workspace vence no font size; global no format/rigor; default no pairs.
        assert_eq!(effective.editor_font_size, 18);
        assert!(effective.format_on_save);
        assert!(effective.auto_close_pairs);
        assert_eq!(
            effective.rigor_profile,
            kinein_protocol::RigorProfile::Relaxed
        );
    }

    #[test]
    fn merge_partial_keeps_other_fields() {
        let base = SettingsValues {
            editor_font_size: Some(16),
            format_on_save: Some(true),
            auto_close_pairs: Some(true),
            auto_save: None,
            ..SettingsValues::default()
        };
        let incoming = SettingsValues {
            editor_font_size: Some(20),
            ..SettingsValues::default()
        };
        let merged = merge(&base, &incoming);
        assert_eq!(merged.editor_font_size, Some(20));
        assert_eq!(merged.format_on_save, Some(true));
        assert_eq!(merged.auto_close_pairs, Some(true));
    }

    #[test]
    fn roundtrip_and_unknown_schema_is_empty() {
        let dir = temp_dir("roundtrip");
        let path = dir.join("settings.json");
        let values = SettingsValues {
            editor_font_size: Some(22),
            ..SettingsValues::default()
        };
        write_values(&path, &values).unwrap();
        assert_eq!(read_values(&path).editor_font_size, Some(22));

        std::fs::write(&path, "{\"schemaVersion\":99,\"editorFontSize\":30}").unwrap();
        assert_eq!(read_values(&path), SettingsValues::default());
    }

    #[test]
    fn missing_file_reads_empty() {
        let dir = temp_dir("missing");
        assert_eq!(
            read_values(&dir.join("absent.json")),
            SettingsValues::default()
        );
    }

    #[test]
    fn resolve_defaults_when_both_empty() {
        let effective = resolve(&SettingsValues::default(), &SettingsValues::default());
        assert_eq!(effective.editor_font_size, 14);
        assert!(!effective.format_on_save);
        assert!(effective.auto_close_pairs);
        // Default do perfil de rigor e Strict (identidade do produto).
        assert_eq!(
            effective.rigor_profile,
            kinein_protocol::RigorProfile::Strict
        );
    }

    #[test]
    fn write_creates_parent_dir() {
        let dir = temp_dir("nested");
        let path = dir.join(".kinein").join("settings.json");
        assert!(!path.exists());
        write_values(&path, &SettingsValues::default()).unwrap();
        assert!(path.exists());
    }

    // Garante que a assinatura de workspace_path monta `.kinein`.
    #[test]
    fn workspace_path_lives_under_kinein() {
        let path = super::workspace_path(Path::new("/tmp/ws"));
        assert!(path.ends_with(".kinein/settings.json"));
    }
}

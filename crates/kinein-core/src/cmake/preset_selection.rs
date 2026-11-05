//! Selecao unica de preset para configure explicito e build automatico.
#![allow(
    clippy::redundant_pub_crate,
    reason = "Keep shared preset selection internal to the core"
)]

use std::path::Path;

use super::{PresetDefault, default_preset, list_presets, status};

pub(crate) type PresetChoice = (Option<String>, Option<&'static str>);

pub(crate) fn choose_preset(root: &Path, request: Option<String>) -> Result<PresetChoice, String> {
    if let Some(preset) = request.filter(|p| !p.trim().is_empty()) {
        return Ok((Some(preset), Some("kit")));
    }
    if let Some(attempt) = super::attempt::pending_preset(root)? {
        let preset = attempt.preset;
        if let Some(name) = preset.as_ref()
            && !list_presets(root)?.iter().any(|entry| entry.name == *name)
        {
            return Err(format!(
                "preset do configure incompleto removido: {name}; escolha outro"
            ));
        }
        return Ok((preset, Some("incompleteConfigure")));
    }
    choose_default(root).map_err(|(file, names)| {
        format!(
            "o projeto tem {} presets de configure em {file} ({}): escolha um",
            names.len(),
            names.join(", ")
        )
    })
}

pub(crate) fn choose_default(root: &Path) -> Result<PresetChoice, (&'static str, Vec<String>)> {
    if let Some(recorded) = status(root).preset
        && list_presets(root).is_ok_and(|presets| presets.iter().any(|p| p.name == recorded))
    {
        return Ok((Some(recorded), Some("buildDir")));
    }
    match default_preset(root) {
        PresetDefault::Absent => Ok((None, None)),
        PresetDefault::One(preset, file) => Ok((Some(preset), Some(file))),
        PresetDefault::Several(file, names) => Err((file, names)),
    }
}

//! The configurations of a `CMake` project (C1-19 and C1-20 of C-04,
//! consolidacao C §17): the folder each configure runs in, the active
//! configuration, and the records the IDE keeps in each folder.
//!
//! Until 2026-11-09 every configure ran in `.kinein/build` with `-B`, which
//! overrides the `binaryDir` of a preset (measured with `CMake` 4.4.5): the IDE
//! and the terminal configured different folders for the same preset, and
//! switching presets reconfigured the same folder.
//!
//! A leaf beside `cmake_presets`: `cmake` and `toolchain` read the active
//! configuration here, and `toolchain` cannot read it in `cmake`, which
//! depends on `toolchain`.

use std::{
    fs, io,
    path::{Path, PathBuf},
};

use kinein_protocol::CmakeConfigurationInfo;
use serde::{Deserialize, Serialize};

use crate::cmake_presets::{PresetDefault, PresetDirectory};

/// The preset the IDE configured a folder with: the cache does not record it.
pub(crate) const PRESET_MARKER: &str = ".kinein-preset";
/// A configure that began in a folder and has not succeeded yet.
pub(crate) const PENDING_MARKER: &str = ".kinein-configure-pending";
/// The kit the last successful configure of a folder used (C1-13).
pub(crate) const KIT_MARKER: &str = ".kinein-kit";

/// The folder of a configure without preset, and of every configure before
/// C-04.
#[must_use]
pub fn ide_build_dir(root: &Path) -> PathBuf {
    root.join(".kinein").join("build")
}

/// O texto que o fio usa para uma pasta: relativa ao root, `"."` para ele,
/// absoluta fora dele.
#[must_use]
pub fn label(root: &Path, directory: &Path) -> String {
    let relative = directory
        .strip_prefix(root)
        .map(Path::to_path_buf)
        .ok()
        .or_else(|| {
            let root = crate::platform::canonicalize(root).ok()?;
            crate::platform::canonicalize(directory)
                .ok()?
                .strip_prefix(&root)
                .map(Path::to_path_buf)
                .ok()
        });
    match relative {
        Some(path) if path.as_os_str().is_empty() => ".".to_owned(),
        Some(path) => crate::platform::portable_relative(&path),
        None => directory.display().to_string(),
    }
}

/// Where a preset without `binaryDir` configures.
fn ide_presets_dir(root: &Path) -> PathBuf {
    root.join(".kinein").join("presets")
}

/// The folder a configure of `preset` runs in: `.kinein/build` without a
/// preset, the resolved `binaryDir`, else `.kinein/presets/<preset>`.
///
/// # Errors
/// Presets `CMake` would refuse, an unknown preset, a `binaryDir` that does not
/// resolve, and a name that cannot be a folder.
pub fn configuration_dir(root: &Path, preset: Option<&str>) -> Result<PathBuf, String> {
    let Some(preset) = preset else {
        return Ok(ide_build_dir(root));
    };
    match crate::cmake_presets::binary_dir(root, preset) {
        Ok(Some(directory)) => Ok(directory),
        Ok(None) if is_folder_name(preset) => Ok(ide_presets_dir(root).join(preset)),
        Ok(None) => Err(format!(
            "o preset \"{preset}\" não declara binaryDir, e o nome não serve de pasta em \
             .kinein/presets: declare binaryDir no preset ou mude o nome"
        )),
        Err(reason) => Err(format!(
            "a pasta do preset {preset} não é conhecida: {reason}"
        )),
    }
}

/// One path component: not empty, `.` or `..`, and without a separator.
fn is_folder_name(name: &str) -> bool {
    let separators: &[char] = if cfg!(windows) {
        &['/', '\\', ':', '\0']
    } else {
        &['/', '\\', '\0']
    };
    !matches!(name, "" | "." | "..") && !name.contains(separators)
}

/// Whether the IDE made `directory` (`.kinein/build` or a folder of
/// `.kinein/presets`); any other configuration folder belongs to the project.
#[must_use]
pub fn owned_by_ide(root: &Path, directory: &Path) -> bool {
    directory == ide_build_dir(root) || directory.parent() == Some(ide_presets_dir(root).as_path())
}

/// The folder of every listed preset: its `binaryDir`, or the IDE's folder
/// for one without it, in the order of the list.
pub(crate) fn preset_dirs(root: &Path) -> Vec<PresetDirectory> {
    let mut directories = crate::cmake_presets::binary_dirs(root);
    for preset in crate::cmake_presets::list_presets(root).unwrap_or_default() {
        if crate::cmake_presets::binary_dir(root, &preset.name) == Ok(None)
            && is_folder_name(&preset.name)
        {
            directories.push(PresetDirectory {
                directory: ide_presets_dir(root).join(&preset.name),
                name: preset.name,
            });
        }
    }
    directories
}

/// Every listed configure preset with the folder of its configuration: what
/// the configuration menu offers.
///
/// In the order of the list (C1-23, C1-25); empty without presets or with
/// presets `CMake` would refuse.
#[must_use]
pub fn configurations(root: &Path) -> Vec<CmakeConfigurationInfo> {
    crate::cmake_presets::list_presets(root)
        .unwrap_or_default()
        .into_iter()
        .map(|preset| {
            let directory = configuration_dir(root, Some(&preset.name));
            CmakeConfigurationInfo {
                configured: directory
                    .as_ref()
                    .is_ok_and(|directory| directory.join("CMakeCache.txt").is_file()),
                build_dir: directory
                    .as_ref()
                    .ok()
                    .map(|directory| label(root, directory)),
                build_dir_refused: directory.err(),
                display_name: preset.display_name,
                condition_note: preset.condition_note,
                preset: preset.name,
            }
        })
        .collect()
}

/// The configuration that build, run, debug, ctest, flash, `qmlls`, the
/// context and the project's compiler use.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Configuration {
    /// Its configure preset; `None` for the configure without preset.
    pub preset: Option<String>,
    /// The folder it is configured in.
    pub directory: PathBuf,
    /// Why it is the active one, as `cmake.status.configurationSource`:
    /// `"configuration"` (the record), `"buildDir"` (`.kinein/build` with the
    /// IDE's evidence), the presets file of the default, or `"noPresets"`.
    pub source: &'static str,
}

/// Why the project has no active configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    /// Several presets in the same file and no choice: the person chooses
    /// (52 §14.2).
    Several(&'static str, Vec<String>),
    /// Presets `CMake` would refuse, an unreadable record, a recorded preset
    /// that left the presets, or a folder that is not known.
    Refused(String),
}

impl Refusal {
    /// The sentence for the person.
    #[must_use]
    pub fn reason(&self) -> String {
        match self {
            Self::Several(file, names) => format!(
                "o projeto tem {} presets de configure em {file} ({}): escolha um",
                names.len(),
                names.join(", ")
            ),
            Self::Refused(reason) => reason.clone(),
        }
    }
}

/// The active configuration, in this order.
///
/// The record; else `.kinein/build` with the IDE's evidence, a pending
/// attempt or an annotated preset, also from before C-04; else the single
/// default preset; else `.kinein/build` without preset.
///
/// # Errors
/// No active configuration: never a silent return to `.kinein/build`.
pub fn active_configuration(root: &Path) -> Result<Configuration, Refusal> {
    let refused = Refusal::Refused;
    if let Some(preset) = selected_configuration(root).map_err(refused)? {
        if !listed(root, &preset).map_err(refused)? {
            return Err(refused(format!(
                "configuração escolhida removida dos presets: {preset}; escolha outra"
            )));
        }
        let directory = configuration_dir(root, Some(&preset)).map_err(refused)?;
        return Ok(Configuration {
            preset: Some(preset),
            directory,
            source: "configuration",
        });
    }
    let ide = ide_build_dir(root);
    if let Some(attempt) = pending(&ide).map_err(refused)? {
        if let Some(name) = attempt.preset.as_deref()
            && !listed(root, name).map_err(refused)?
        {
            return Err(refused(format!(
                "preset do configure incompleto removido: {name}; escolha outro"
            )));
        }
        return Ok(Configuration {
            preset: attempt.preset,
            directory: ide,
            source: "buildDir",
        });
    }
    if let Some(preset) = annotated_preset(&ide)
        && listed(root, &preset).unwrap_or(false)
    {
        return Ok(Configuration {
            preset: Some(preset),
            directory: ide,
            source: "buildDir",
        });
    }
    match crate::cmake_presets::default_preset(root) {
        PresetDefault::Absent => Ok(Configuration {
            preset: None,
            directory: ide,
            source: "noPresets",
        }),
        PresetDefault::One(preset, file) => {
            let directory = configuration_dir(root, Some(&preset)).map_err(refused)?;
            Ok(Configuration {
                preset: Some(preset),
                directory,
                source: file,
            })
        }
        PresetDefault::Several(file, names) => Err(Refusal::Several(file, names)),
        PresetDefault::Refused(reason) => Err(refused(reason)),
    }
}

/// The folder of the active configuration, or why there is none.
///
/// # Errors
/// As [`active_configuration`], with the sentence for the person.
pub fn active_dir(root: &Path) -> Result<PathBuf, String> {
    active_configuration(root)
        .map(|configuration| configuration.directory)
        .map_err(|refusal| refusal.reason())
}

fn listed(root: &Path, preset: &str) -> Result<bool, String> {
    Ok(crate::cmake_presets::list_presets(root)?
        .iter()
        .any(|entry| entry.name == preset))
}

/// A configure that began in a folder: its preset and kit.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Attempt {
    pub(crate) preset: Option<String>,
    pub(crate) kit: String,
}

/// The bytes of the marker `name` of the configure folder `directory`;
/// `None` when absent. Inside the `.kinein`, by its guard (INF-15); a
/// folder of the project outside it is read as it is (INF-18).
fn marker(directory: &Path, name: &str) -> Result<Option<Vec<u8>>, String> {
    if let Some((root, relative)) = crate::kinein_dir::inside(directory) {
        return crate::kinein_dir::read(root, &format!("{relative}/{name}"));
    }
    match fs::read(directory.join(name)) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.to_string()),
    }
}

/// The attempt pending in `directory`, if any.
pub(crate) fn pending(directory: &Path) -> Result<Option<Attempt>, String> {
    let text = match marker(directory, PENDING_MARKER) {
        Ok(Some(text)) => text,
        Ok(None) => return Ok(None),
        Err(reason) => {
            return Err(format!(
                "registro do configure incompleto ilegível: {reason}"
            ));
        }
    };
    serde_json::from_slice::<Attempt>(&text)
        .map(Some)
        .map_err(|error| format!("registro do configure incompleto inválido: {error}"))
}

/// The text of the marker `name`, when it can be read as UTF-8.
fn marker_text(directory: &Path, name: &str) -> Option<String> {
    String::from_utf8(marker(directory, name).ok()??).ok()
}

/// The preset the IDE configured `directory` with, for a configured folder.
pub(crate) fn annotated_preset(directory: &Path) -> Option<String> {
    if !directory.join("CMakeCache.txt").is_file() {
        return None;
    }
    marker_text(directory, PRESET_MARKER)
        .map(|preset| preset.trim().to_owned())
        .filter(|preset| !preset.is_empty())
}

/// The kit the last successful configure of `directory` recorded.
pub(crate) fn last_kit(directory: &Path) -> Option<String> {
    marker_text(directory, KIT_MARKER).map(|kit| kit.trim_end_matches('\n').to_owned())
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Record {
    preset: String,
}

fn record_path(root: &Path) -> PathBuf {
    root.join(".kinein").join("configuration.json")
}

/// The preset of `.kinein/configuration.json` (C-05).
///
/// Since C1-20 it is the active configuration, recorded by
/// `cmake.selectConfiguration` and by the configure. An unreadable or invalid
/// record is an error to correct, never a silent return to the automatic
/// choice.
///
/// # Errors
/// The record exists and cannot be read or parsed.
pub fn selected_configuration(root: &Path) -> Result<Option<String>, String> {
    // INF-15: pelo guarda do `.kinein`, que nao segue o link nem prende num
    // FIFO.
    let bytes = match crate::kinein_dir::read(root, "configuration.json") {
        Ok(Some(bytes)) => bytes,
        Ok(None) => return Ok(None),
        Err(reason) => {
            return Err(format!(
                "registro da configuração escolhida ilegível: {reason}"
            ));
        }
    };
    serde_json::from_slice::<Record>(&bytes)
        .map(|record| Some(record.preset))
        .map_err(|error| format!("registro da configuração escolhida inválido: {error}"))
}

/// Records `preset`, or removes the record for `None`.
///
/// # Errors
/// The record cannot be written or removed.
pub fn select_configuration(root: &Path, preset: Option<&str>) -> io::Result<()> {
    let path = record_path(root);
    let Some(preset) = preset else {
        return match fs::remove_file(&path) {
            Err(error) if error.kind() != io::ErrorKind::NotFound => Err(error),
            _ => Ok(()),
        };
    };
    let body = serde_json::to_vec_pretty(&Record {
        preset: preset.to_owned(),
    })?;
    // INF-13: pelo guarda do `.kinein`, que nao segue o link.
    crate::kinein_dir::write(root, "configuration.json", &body).map_err(io::Error::other)
}

#[cfg(test)]
mod tests;

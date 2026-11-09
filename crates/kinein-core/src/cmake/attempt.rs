//! Persist the requested configure until the producer completes successfully.

use std::{
    fs, io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

const MARKER: &str = ".kinein-configure-pending";

/// The kit the last successful configure used. The preset alone does not say
/// it: a preset requested by name uses its own kit, one chosen from the
/// project files uses the default kit (C1-13).
const LAST_KIT: &str = ".kinein-kit";

/// One producer of configure artifacts in the IDE build directory.
#[derive(Debug)]
#[must_use = "hold the configure lock until the producer finishes"]
pub struct ConfigureAttempt {
    root: PathBuf,
    kit: String,
    /// A configuracao escolhida quando o configure comecou: o sucesso so'
    /// apaga um registro que ninguem trocou durante o produtor (C-05).
    selection: Result<Option<String>, String>,
    _lock: fs::File,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Attempt {
    pub(super) preset: Option<String>,
    pub(super) kit: String,
}

/// No marker, an unfinished configure without preset, or its requested preset.
pub(super) fn pending_preset(root: &Path) -> Result<Option<Attempt>, String> {
    let text = match fs::read(super::build_dir(root).join(MARKER)) {
        Ok(text) => text,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(format!(
                "registro do configure incompleto ilegível: {error}"
            ));
        }
    };
    serde_json::from_slice::<Attempt>(&text)
        .map(Some)
        .map_err(|error| format!("registro do configure incompleto inválido: {error}"))
}

/// Must succeed before running either configure producer. Never edit project inputs.
pub fn begin_configure(
    root: &Path,
    preset: Option<&str>,
    kit: &str,
) -> io::Result<ConfigureAttempt> {
    let directory = super::build_dir(root);
    fs::create_dir_all(&directory)?;
    let lock = crate::platform::configuration_lock(&directory.join(".kinein-configure-lock"))?;
    let bytes = serde_json::to_vec(&Attempt {
        preset: preset.map(str::to_owned),
        kit: kit.to_owned(),
    })?;
    fs::write(directory.join(MARKER), bytes)?;
    Ok(ConfigureAttempt {
        root: root.to_owned(),
        kit: kit.to_owned(),
        selection: super::selected_configuration(root),
        _lock: lock,
    })
}

/// Only a successful configure may restore current evidence.
///
/// It also returns the compilation database to `.kinein/build`: the chosen
/// configuration recorded before this configure began is removed (C-05). A
/// choice made while the producer ran is newer and stays.
pub fn complete_configure(attempt: ConfigureAttempt) -> io::Result<()> {
    let directory = super::build_dir(&attempt.root);
    fs::write(directory.join(LAST_KIT), format!("{}\n", attempt.kit))?;
    if let Ok(Some(_)) = &attempt.selection
        && super::selected_configuration(&attempt.root) == attempt.selection
    {
        super::select_configuration(&attempt.root, None)?;
    }
    let result = fs::remove_file(directory.join(MARKER));
    drop(attempt);
    result
}

/// The pending attempt's kit, else the one the last success recorded; `None`
/// for a build dir configured before the record existed.
pub(super) fn active_kit(root: &Path) -> Option<String> {
    if let Ok(Some(attempt)) = pending_preset(root) {
        return Some(attempt.kit);
    }
    if !super::status(root).configured {
        return Some(String::new());
    }
    fs::read_to_string(super::build_dir(root).join(LAST_KIT))
        .ok()
        .map(|kit| kit.trim_end_matches('\n').to_owned())
}

/// Explain why the IDE's configure artifacts cannot establish current context.
#[must_use]
pub fn incomplete_configure(root: &Path) -> Option<String> {
    match pending_preset(root) {
        Ok(None) => None,
        Ok(Some(attempt)) => Some(format!(
            "configure não concluído ({}); repita ou escolha outra configuração",
            attempt
                .preset
                .map_or_else(|| "sem preset".to_owned(), |name| format!("preset {name}"))
        )),
        Err(reason) => Some(reason),
    }
}

/// What the producer says before repeating a configure whose cache lost the
/// IDE's definitions (C1-14).
pub const DEFINITIONS_LOST: &str = "o CMake recriou o cache sem as definições da IDE \
     (compilador alterado); repetindo o configure uma vez";

/// Whether a successful configure lost the IDE's command-line definitions.
///
/// Changing the compiler of a configured build dir makes `CMake` delete the
/// cache and configure again without the `-D` from the command line (measured
/// with `CMake` 4.4.5, C1-14): the cache keeps `CMAKE_EXPORT_COMPILE_COMMANDS`
/// declared but empty, and the previous `compile_commands.json` stays,
/// describing the old compiler. Repeating the same configure on the recreated
/// cache applies them again. A cache without the entry is not this signal.
#[must_use]
pub fn lost_ide_definitions(root: &Path) -> bool {
    fs::read_to_string(super::build_dir(root).join("CMakeCache.txt")).is_ok_and(|cache| {
        cache.lines().any(|line| {
            line.split_once('=').is_some_and(|(declaration, value)| {
                declaration.split(':').next() == Some("CMAKE_EXPORT_COMPILE_COMMANDS")
                    && !matches!(
                        value.trim().to_ascii_uppercase().as_str(),
                        "ON" | "1" | "TRUE" | "YES" | "Y"
                    )
            })
        })
    })
}

#[cfg(test)]
mod tests {
    use super::{begin_configure, complete_configure};
    use crate::cmake::{select_configuration, selected_configuration};

    #[test]
    fn success_clears_only_the_choice_recorded_when_the_configure_began() {
        let root = std::env::temp_dir()
            .join("kinein-cmake-attempt")
            .join(format!("{} escolha", std::process::id()));
        drop(std::fs::remove_dir_all(&root));
        std::fs::create_dir_all(&root).unwrap();
        let chosen = |name: &str| Ok(Some(name.to_owned()));

        // A escolha anterior volta a' pasta da IDE com o sucesso.
        select_configuration(&root, Some("debug")).unwrap();
        complete_configure(begin_configure(&root, Some("debug"), "").unwrap()).unwrap();
        assert_eq!(selected_configuration(&root), Ok(None));

        // Trocada, criada ou apagada durante o produtor: a nova vence.
        for (before, during) in [(Some("debug"), Some("release")), (None, Some("release"))] {
            select_configuration(&root, before).unwrap();
            let attempt = begin_configure(&root, before, "").unwrap();
            select_configuration(&root, during).unwrap();
            complete_configure(attempt).unwrap();
            assert_eq!(
                selected_configuration(&root),
                chosen("release"),
                "{before:?}"
            );
        }
        select_configuration(&root, Some("debug")).unwrap();
        let attempt = begin_configure(&root, Some("debug"), "").unwrap();
        select_configuration(&root, None).unwrap();
        complete_configure(attempt).unwrap();
        assert_eq!(selected_configuration(&root), Ok(None));

        // Um registro ilegível no início não é apagado por um sucesso.
        let record = root.join(".kinein/configuration.json");
        std::fs::write(&record, "{").unwrap();
        complete_configure(begin_configure(&root, None, "").unwrap()).unwrap();
        assert_eq!(std::fs::read_to_string(&record).unwrap(), "{");
    }
}

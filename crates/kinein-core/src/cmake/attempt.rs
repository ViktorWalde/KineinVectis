//! Persist the requested configure until the producer completes successfully.
//!
//! Since C1-20 the records live in the folder of the configuration (consolidacao
//! C §17): the configure of one configuration never touches another's.

use std::{
    fs, io,
    path::{Path, PathBuf},
};

use crate::cmake_configuration::{Attempt, KIT_MARKER, PENDING_MARKER, pending};

/// One producer of configure artifacts in the folder of a configuration.
#[derive(Debug)]
#[must_use = "hold the configure lock until the producer finishes"]
pub struct ConfigureAttempt {
    directory: PathBuf,
    kit: String,
    _lock: fs::File,
}

/// Must succeed before running either configure producer. Never edit project
/// inputs.
///
/// After the folder's lock, the configuration becomes the active one (C1-20,
/// decision of 2026-11-09): the record names `preset`, or is removed for a
/// configure without preset. A configure that fails or is cancelled keeps it.
pub fn begin_configure(
    root: &Path,
    directory: &Path,
    preset: Option<&str>,
    kit: &str,
) -> io::Result<ConfigureAttempt> {
    fs::create_dir_all(directory)?;
    let lock = crate::platform::configuration_lock(&directory.join(".kinein-configure-lock"))?;
    let bytes = serde_json::to_vec(&Attempt {
        preset: preset.map(str::to_owned),
        kit: kit.to_owned(),
    })?;
    fs::write(directory.join(PENDING_MARKER), bytes)?;
    super::select_configuration(root, preset)?;
    Ok(ConfigureAttempt {
        directory: directory.to_owned(),
        kit: kit.to_owned(),
        _lock: lock,
    })
}

/// Only a successful configure may restore current evidence. The record of
/// the active configuration stays: until C1-20 it was removed here.
pub fn complete_configure(attempt: ConfigureAttempt) -> io::Result<()> {
    fs::write(
        attempt.directory.join(KIT_MARKER),
        format!("{}\n", attempt.kit),
    )?;
    let result = fs::remove_file(attempt.directory.join(PENDING_MARKER));
    drop(attempt);
    result
}

/// Explain why the configure artifacts of `directory` cannot establish
/// current context.
#[must_use]
pub fn incomplete_in(directory: &Path) -> Option<String> {
    match pending(directory) {
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

/// [`incomplete_in`] the folder of the active configuration; nothing without
/// one, whose refusal the consumers show.
#[must_use]
pub fn incomplete_configure(root: &Path) -> Option<String> {
    super::active_dir(root)
        .ok()
        .and_then(|directory| incomplete_in(&directory))
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
pub fn lost_ide_definitions(directory: &Path) -> bool {
    fs::read_to_string(directory.join("CMakeCache.txt")).is_ok_and(|cache| {
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
    use super::{begin_configure, complete_configure, incomplete_in};
    use crate::cmake::selected_configuration;

    #[test]
    fn the_configuration_is_active_from_the_beginning_and_each_folder_keeps_its_records() {
        let root = std::env::temp_dir()
            .join("kinein-cmake-attempt")
            .join(format!("{} escolha", std::process::id()));
        drop(std::fs::remove_dir_all(&root));
        std::fs::create_dir_all(&root).unwrap();
        let debug = root.join("build/debug");
        let release = root.join(".kinein/presets/release");
        let chosen = |name: &str| Ok(Some(name.to_owned()));

        let attempt = begin_configure(&root, &debug, Some("debug"), "gcc").unwrap();
        assert_eq!(selected_configuration(&root), chosen("debug"));
        assert!(incomplete_in(&debug).unwrap().contains("preset debug"));
        // A mesma pasta está travada; outra pasta, não.
        assert!(begin_configure(&root, &debug, Some("debug"), "").is_err());
        assert_eq!(
            selected_configuration(&root),
            chosen("debug"),
            "a recusa da trava não muda a configuração ativa"
        );
        let other = begin_configure(&root, &release, Some("release"), "").unwrap();
        assert_eq!(selected_configuration(&root), chosen("release"));
        drop(other);
        // O sucesso de debug não volta a ativa para ele nem apaga o registro.
        complete_configure(attempt).unwrap();
        assert_eq!(selected_configuration(&root), chosen("release"));
        assert_eq!(incomplete_in(&debug), None);
        assert_eq!(
            std::fs::read_to_string(debug.join(".kinein-kit")).unwrap(),
            "gcc\n"
        );
        assert!(
            incomplete_in(&release).unwrap().contains("preset release"),
            "a tentativa de release continua pendente na pasta dela"
        );
        assert!(!release.join(".kinein-kit").exists());

        // Sem preset: o registro sai, e a pasta é .kinein/build.
        let ide = root.join(".kinein/build");
        complete_configure(begin_configure(&root, &ide, None, "").unwrap()).unwrap();
        assert_eq!(selected_configuration(&root), Ok(None));
        assert_eq!(incomplete_in(&ide), None);
    }
}

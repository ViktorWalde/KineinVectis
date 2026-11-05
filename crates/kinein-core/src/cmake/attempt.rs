//! Persist the requested configure until the producer completes successfully.

use std::{
    fs, io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

const MARKER: &str = ".kinein-configure-pending";

/// One producer of configure artifacts in the IDE build directory.
#[derive(Debug)]
#[must_use = "hold the configure lock until the producer finishes"]
pub struct ConfigureAttempt {
    root: PathBuf,
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
        _lock: lock,
    })
}

/// Only a successful configure may restore current evidence.
pub fn complete_configure(attempt: ConfigureAttempt) -> io::Result<()> {
    let result = fs::remove_file(super::build_dir(&attempt.root).join(MARKER));
    drop(attempt);
    result
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

//! The configuration the person chose (`.kinein/configuration.json`, C-05).
//!
//! `cmake.selectConfiguration` records a configure preset; the compilation
//! database in use and the configure without a requested preset follow it
//! (consolidacao C §16). A successful IDE configure removes the record, so
//! `.kinein/build` is current again. In C-04 the record becomes the active
//! configuration of the configure itself.

use std::{
    fs, io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Record {
    preset: String,
}

fn record_path(root: &Path) -> PathBuf {
    root.join(".kinein").join("configuration.json")
}

/// The recorded preset; an unreadable or invalid record is an error to
/// correct, never a silent return to the automatic choice.
pub fn selected_configuration(root: &Path) -> Result<Option<String>, String> {
    let bytes = match fs::read(record_path(root)) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(format!(
                "registro da configuração escolhida ilegível: {error}"
            ));
        }
    };
    serde_json::from_slice::<Record>(&bytes)
        .map(|record| Some(record.preset))
        .map_err(|error| format!("registro da configuração escolhida inválido: {error}"))
}

/// Records `preset`, or removes the record for `None`.
pub fn select_configuration(root: &Path, preset: Option<&str>) -> io::Result<()> {
    let path = record_path(root);
    let Some(preset) = preset else {
        return match fs::remove_file(&path) {
            Err(error) if error.kind() != io::ErrorKind::NotFound => Err(error),
            _ => Ok(()),
        };
    };
    let directory = root.join(".kinein");
    fs::create_dir_all(&directory)?;
    let body = serde_json::to_vec_pretty(&Record {
        preset: preset.to_owned(),
    })?;
    crate::fsops::atomic_write(&path, &body).map_err(io::Error::other)
}

#[cfg(test)]
mod tests {
    use super::{record_path, select_configuration, selected_configuration};

    #[test]
    fn the_record_round_trips_and_an_invalid_one_is_reported() {
        let root = std::env::temp_dir()
            .join("kinein-cmake-configuration")
            .join(format!("{} ação", std::process::id()));
        drop(std::fs::remove_dir_all(&root));
        std::fs::create_dir_all(&root).unwrap();
        assert_eq!(selected_configuration(&root), Ok(None));
        select_configuration(&root, None).unwrap();
        select_configuration(&root, Some("debug")).unwrap();
        assert_eq!(selected_configuration(&root), Ok(Some("debug".to_owned())));
        std::fs::write(record_path(&root), r#"{"preset":"debug","extra":1}"#).unwrap();
        assert!(
            selected_configuration(&root)
                .unwrap_err()
                .starts_with("registro da configuração escolhida inválido")
        );
        select_configuration(&root, None).unwrap();
        assert_eq!(selected_configuration(&root), Ok(None));
    }
}

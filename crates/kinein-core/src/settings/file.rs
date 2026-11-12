//! O `settings.json` no disco (`0.220.0`, INF-07, INF-11 e INF-12).
//!
//! A leitura distingue o arquivo ausente, que vale como vazio, do que existe
//! e a IDE não entende, que volta com o motivo e não é gravado por cima. A
//! escrita parte do objeto lido e guarda as chaves que esta versão não
//! conhece. No projeto, o `.kinein` e o arquivo que são link simbólico são
//! recusados, e nada se grava fora dele; o global segue o link, porque é da
//! pessoa e pode vir de dotfiles.

use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
};

use serde::Serialize;
use serde_json::{Map, Value};

use kinein_protocol::SettingsValues;

/// Versão do schema que este core lê e escreve.
const SCHEMA_VERSION: u64 = 1;

/// Um `settings.json` e o jeito de chegar a ele.
pub(super) struct Place {
    path: PathBuf,
    /// Como o motivo nomeia o arquivo: o caminho inteiro no global, o
    /// relativo ao projeto no workspace.
    shown: String,
    /// O `.kinein` do projeto. Presente, o link é recusado nele e no arquivo.
    folder: Option<PathBuf>,
}

impl Place {
    /// O global, em `path`, que segue o link.
    pub(super) fn global(path: PathBuf) -> Self {
        Self {
            shown: path.display().to_string(),
            path,
            folder: None,
        }
    }

    /// O do projeto em `root`, que recusa o link.
    pub(super) fn workspace(root: &Path) -> Self {
        let folder = root.join(".kinein");
        Self {
            path: folder.join("settings.json"),
            shown: ".kinein/settings.json".to_owned(),
            folder: Some(folder),
        }
    }

    fn problem(&self, reason: &str) -> String {
        format!("{} {reason}", self.shown)
    }
}

/// Um arquivo entendido: os valores que esta versão conhece e o objeto
/// inteiro, que a escrita guarda (INF-11). O ausente é o padrão.
#[derive(Debug, Default)]
pub(super) struct Contents {
    pub(super) values: SettingsValues,
    object: Map<String, Value>,
}

/// O que se grava: o `schemaVersion`, os valores conhecidos e o resto.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct OnDisk<'a> {
    schema_version: u64,
    #[serde(flatten)]
    values: &'a SettingsValues,
    #[serde(flatten)]
    rest: &'a Map<String, Value>,
}

/// O arquivo de `place`. `Err` traz o motivo, com o nome do arquivo, quando
/// ele existe e a IDE não o entende.
pub(super) fn read(place: &Place) -> Result<Contents, String> {
    if let Some(folder) = &place.folder {
        check_folder(folder)?;
    }
    let metadata = if place.folder.is_some() {
        fs::symlink_metadata(&place.path)
    } else {
        fs::metadata(&place.path)
    };
    match metadata {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(Contents::default());
        }
        Err(error) => return Err(place.problem(&format!("não pôde ser lido ({error})"))),
        Ok(metadata) if metadata.file_type().is_symlink() => {
            return Err(place.problem("é um link simbólico, que a IDE não segue no projeto"));
        }
        Ok(metadata) if !metadata.is_file() => {
            return Err(place.problem("não é um arquivo regular"));
        }
        Ok(_) => {}
    }
    let mut bytes = Vec::new();
    fs::File::open(&place.path)
        .and_then(|file| {
            file.take(crate::fsops::MAX_READ_BYTES + 1)
                .read_to_end(&mut bytes)
        })
        .map_err(|error| place.problem(&format!("não pôde ser lido ({error})")))?;
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > crate::fsops::MAX_READ_BYTES {
        return Err(place.problem("passa de 1 MiB, o limite de leitura da IDE"));
    }
    let body = String::from_utf8(bytes).map_err(|_| place.problem("não está em UTF-8"))?;
    let value: Value = serde_json::from_str(&body).map_err(|error| {
        place.problem(&format!(
            "não é JSON válido (linha {}, coluna {})",
            error.line(),
            error.column()
        ))
    })?;
    let Value::Object(object) = value else {
        return Err(place.problem("não é um objeto JSON"));
    };
    match object.get("schemaVersion") {
        None => return Err(place.problem("não tem schemaVersion")),
        Some(version) if version.as_u64() != Some(SCHEMA_VERSION) => {
            return Err(place.problem(&format!(
                "tem schemaVersion {version}, e esta versão da IDE só conhece a {SCHEMA_VERSION}"
            )));
        }
        Some(_) => {}
    }
    let values = known_values(&object).map_err(|reason| place.problem(&reason))?;
    Ok(Contents { values, object })
}

/// Os valores que esta versão conhece; o de tipo errado dá o motivo, com o
/// nome da chave.
fn known_values(object: &Map<String, Value>) -> Result<SettingsValues, String> {
    serde_json::from_value(Value::Object(object.clone())).map_err(|error| {
        let wrong = object.iter().find(|(key, value)| {
            let single = Map::from_iter([((*key).clone(), (*value).clone())]);
            serde_json::from_value::<SettingsValues>(Value::Object(single)).is_err()
        });
        match wrong {
            Some((key, _)) => {
                format!("tem em {key} um valor que esta versão da IDE não entende ({error})")
            }
            None => format!("tem um valor que esta versão da IDE não entende ({error})"),
        }
    })
}

/// O `.kinein` do projeto: ausente, ou uma pasta de verdade.
fn check_folder(folder: &Path) -> Result<(), String> {
    match fs::symlink_metadata(folder) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!(".kinein não pôde ser lido ({error})")),
        Ok(metadata) if metadata.file_type().is_symlink() => {
            Err(".kinein é um link simbólico, que a IDE não segue no projeto".to_owned())
        }
        Ok(metadata) if !metadata.is_dir() => Err(".kinein não é uma pasta".to_owned()),
        Ok(_) => Ok(()),
    }
}

/// Grava `values` em `place`, sobre `read`, o que [`read`] devolveu dele: as
/// chaves que esta versão não conhece ficam, com os valores delas. No
/// projeto, a troca é atômica e não segue um link que tenha aparecido depois
/// da leitura; no global, a escrita segue o link, como a leitura.
pub(super) fn write(place: &Place, read: &Contents, values: &SettingsValues) -> Result<(), String> {
    let failure =
        |error: &dyn std::fmt::Display| format!("falha escrevendo {}: {error}", place.shown);
    let mut rest = read.object.clone();
    rest.remove("schemaVersion");
    if let Value::Object(known) = serde_json::to_value(values).map_err(|error| failure(&error))? {
        for key in known.keys() {
            rest.remove(key);
        }
    }
    let body = serde_json::to_string_pretty(&OnDisk {
        schema_version: SCHEMA_VERSION,
        values,
        rest: &rest,
    })
    .map_err(|error| failure(&error))?;
    let Some(folder) = &place.folder else {
        if let Some(parent) = place.path.parent() {
            fs::create_dir_all(parent).map_err(|error| failure(&error))?;
        }
        return fs::write(&place.path, body).map_err(|error| failure(&error));
    };
    fs::create_dir_all(folder).map_err(|error| failure(&error))?;
    check_folder(folder)?;
    crate::fsops::atomic_write(&place.path, body.as_bytes()).map_err(|error| failure(&error))
}

#[cfg(test)]
mod tests;

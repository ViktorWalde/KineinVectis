//! Sessao de editor por workspace (`.kinein/session.json`).
//!
//! Guarda as abas abertas e a aba ativa com caminhos RELATIVOS ao root (a
//! pasta do projeto pode ser movida sem perder a sessao); o contrato IPC so
//! fala em caminhos absolutos canonicos — `save_session` relativiza e
//! `load_session` resolve, confina e filtra arquivos que deixaram de
//! existir. Sessao invalida (schema desconhecido, JSON corrompido) nunca
//! quebra o `workspace.open`: e simplesmente ignorada.

use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use kinein_protocol::WorkspaceSession;

use crate::fsops::{self, FsError};

/// Versao do schema de `.kinein/session.json` escrita por este core.
const SESSION_SCHEMA_VERSION: u32 = 1;

/// Representacao em disco de `.kinein/session.json`.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SessionFile {
    schema_version: u32,
    open_files: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    active_file: Option<String>,
}

/// Caminho de `.kinein/session.json` para um root de workspace.
#[must_use]
pub fn session_path(root: &Path) -> PathBuf {
    root.join(".kinein").join("session.json")
}

/// Persiste a sessao, relativizando os caminhos ao root.
///
/// Entradas invalidas (fora do root ou que ja nao existem) sao puladas — a
/// sessao nao pode falhar por causa de uma aba orfa. Retorna quantas
/// entradas foram efetivamente salvas.
pub fn save_session(
    root: &Path,
    open_files: &[String],
    active_file: Option<&str>,
) -> Result<u64, FsError> {
    let mut relative_files = Vec::with_capacity(open_files.len());
    for file in open_files {
        if let Some(relative) = relative_inside_root(root, Path::new(file)) {
            relative_files.push(relative);
        }
    }
    let relative_active = active_file
        .and_then(|active| relative_inside_root(root, Path::new(active)))
        .filter(|active| relative_files.contains(active));

    let saved = u64::try_from(relative_files.len()).unwrap_or(u64::MAX);
    let session = SessionFile {
        schema_version: SESSION_SCHEMA_VERSION,
        open_files: relative_files,
        active_file: relative_active,
    };
    let target = session_path(root);
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(|source| FsError::Io {
            path: parent.display().to_string(),
            source,
        })?;
    }
    let body = serde_json::to_string_pretty(&session).map_err(|source| FsError::Io {
        path: target.display().to_string(),
        source: source.into(),
    })?;
    fs::write(&target, body).map_err(|source| FsError::Io {
        path: target.display().to_string(),
        source,
    })?;
    Ok(saved)
}

/// Carrega a sessao persistida, com caminhos absolutos ja validados.
///
/// Retorna `None` quando nao ha sessao, o schema e desconhecido, o JSON e
/// invalido ou nenhum arquivo da sessao existe mais.
#[must_use]
pub fn load_session(root: &Path) -> Option<WorkspaceSession> {
    let body = fs::read_to_string(session_path(root)).ok()?;
    let session: SessionFile = serde_json::from_str(&body).ok()?;
    if session.schema_version != SESSION_SCHEMA_VERSION {
        return None;
    }

    let mut open_files = Vec::with_capacity(session.open_files.len());
    for relative in &session.open_files {
        if let Ok(absolute) = fsops::confine_file(root, &root.join(relative)) {
            open_files.push(absolute.display().to_string());
        }
    }
    if open_files.is_empty() {
        return None;
    }
    let active_file = session
        .active_file
        .and_then(|relative| fsops::confine_file(root, &root.join(relative)).ok())
        .map(|absolute| absolute.display().to_string())
        .filter(|active| open_files.contains(active));

    Some(WorkspaceSession {
        open_files,
        active_file,
    })
}

/// Relativiza `path` ao root quando ele e um arquivo existente confinado.
fn relative_inside_root(root: &Path, path: &Path) -> Option<String> {
    let absolute = fsops::confine_file(root, path).ok()?;
    let relative = absolute.strip_prefix(root).ok()?;
    Some(relative.display().to_string())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{load_session, save_session, session_path};

    fn workspace_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("kinein-core-tests")
            .join(format!("{}-session-{name}", std::process::id()));
        std::fs::create_dir_all(dir.join("src")).unwrap();
        std::fs::write(dir.join("Cargo.toml"), "[package]\n").unwrap();
        std::fs::write(dir.join("src/main.rs"), "fn main() {}\n").unwrap();
        std::fs::write(dir.join("src/lib.rs"), "pub fn x() {}\n").unwrap();
        dir.canonicalize().unwrap()
    }

    #[test]
    fn session_roundtrip_stores_relative_and_returns_absolute() {
        let root = workspace_dir("roundtrip");
        let main = root.join("src/main.rs").display().to_string();
        let lib = root.join("src/lib.rs").display().to_string();

        let saved = save_session(&root, &[main.clone(), lib.clone()], Some(&lib)).unwrap();
        assert_eq!(saved, 2);

        let raw = std::fs::read_to_string(session_path(&root)).unwrap();
        assert!(raw.contains("\"src/main.rs\""), "json cru: {raw}");
        assert!(!raw.contains(root.to_str().unwrap()), "json cru: {raw}");

        let session = load_session(&root).unwrap();
        assert_eq!(session.open_files, vec![main, lib.clone()]);
        assert_eq!(session.active_file.as_deref(), Some(lib.as_str()));
    }

    #[test]
    fn load_filters_files_that_no_longer_exist() {
        let root = workspace_dir("filter");
        let main = root.join("src/main.rs").display().to_string();
        let lib = root.join("src/lib.rs").display().to_string();
        save_session(&root, &[main.clone(), lib.clone()], Some(&lib)).unwrap();

        std::fs::remove_file(root.join("src/lib.rs")).unwrap();
        let session = load_session(&root).unwrap();
        assert_eq!(session.open_files, vec![main]);
        assert_eq!(session.active_file, None);
    }

    #[test]
    fn save_skips_paths_outside_the_root() {
        let root = workspace_dir("outside");
        let main = root.join("src/main.rs").display().to_string();
        let saved = save_session(&root, &[main.clone(), "/etc/hostname".to_owned()], None).unwrap();
        assert_eq!(saved, 1);
        assert_eq!(load_session(&root).unwrap().open_files, vec![main]);
    }

    #[test]
    fn unknown_schema_version_is_ignored() {
        let root = workspace_dir("schema");
        std::fs::create_dir_all(root.join(".kinein")).unwrap();
        std::fs::write(
            session_path(&root),
            "{\"schemaVersion\":999,\"openFiles\":[\"src/main.rs\"]}",
        )
        .unwrap();
        assert!(load_session(&root).is_none());
    }

    #[test]
    fn missing_or_empty_session_is_none() {
        let root = workspace_dir("missing");
        assert!(load_session(&root).is_none());
        save_session(&root, &[], None).unwrap();
        assert!(load_session(&root).is_none());
    }
}

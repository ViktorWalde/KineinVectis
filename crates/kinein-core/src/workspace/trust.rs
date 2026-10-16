//! A confianca no projeto (16.4b, `roadmaps/59` §7.1.4).
//!
//! Medido em 2026-10-16 num projeto hostil: so' ABRIR configurava o `CMake`
//! (o `execute_process` do `CMakeLists.txt` rodou), e abrir um `.rs` fez o
//! rust-analyzer rodar o `build.rs`. O requisito do autor (59 §7) e' que
//! abrir um repositorio de terceiros nao execute nada sem gesto explicito.
//!
//! A confianca e' um gesto da pessoa, guardado FORA do projeto, no estado
//! global da IDE, pelo caminho canonico da raiz: nada que o repositorio traga
//! (um `.kinein/` clonado, por exemplo) a declara.

use std::{fs, path::Path, path::PathBuf};

use serde::{Deserialize, Serialize};

const TRUST_SCHEMA_VERSION: u32 = 1;
const TRUST_FILE: &str = "trusted-projects.json";

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TrustFile {
    schema_version: u32,
    #[serde(default)]
    roots: Vec<String>,
}

/// O arquivo da confianca no estado global.
#[must_use]
pub fn trust_path_in(global_storage: &Path) -> PathBuf {
    global_storage.join(TRUST_FILE)
}

fn load(global_storage: &Path) -> Vec<String> {
    let Ok(body) = fs::read_to_string(trust_path_in(global_storage)) else {
        return Vec::new();
    };
    match serde_json::from_str::<TrustFile>(&body) {
        Ok(file) if file.schema_version == TRUST_SCHEMA_VERSION => file.roots,
        _ => Vec::new(),
    }
}

/// A pessoa ja' confiou em `root`?
#[must_use]
pub fn is_trusted_in(global_storage: &Path, root: &str) -> bool {
    load(global_storage).iter().any(|trusted| trusted == root)
}

/// Grava a confianca em `root` (idempotente).
///
/// # Errors
///
/// Disco: criar a pasta ou gravar o arquivo.
pub fn trust_in(global_storage: &Path, root: &str) -> Result<(), String> {
    let mut roots = load(global_storage);
    if roots.iter().any(|trusted| trusted == root) {
        return Ok(());
    }
    roots.push(root.to_owned());
    roots.sort();
    fs::create_dir_all(global_storage)
        .map_err(|error| format!("falha criando {}: {error}", global_storage.display()))?;
    let body = serde_json::to_string_pretty(&TrustFile {
        schema_version: TRUST_SCHEMA_VERSION,
        roots,
    })
    .map_err(|error| format!("falha serializando a confiança: {error}"))?;
    let path = trust_path_in(global_storage);
    fs::write(&path, body).map_err(|error| format!("falha gravando {}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Gravar lembra; outra raiz continua sem confianca; arquivo de outro
    /// schema vale como vazio.
    #[test]
    fn trusting_is_remembered_per_root() {
        let dir = std::env::temp_dir()
            .join("kinein-trust-store")
            .join(std::process::id().to_string());
        let _ = fs::remove_dir_all(&dir);
        assert!(!is_trusted_in(&dir, "/projetos/a"));
        trust_in(&dir, "/projetos/a").unwrap();
        trust_in(&dir, "/projetos/a").unwrap();
        assert!(is_trusted_in(&dir, "/projetos/a"));
        assert!(!is_trusted_in(&dir, "/projetos/b"));
        assert_eq!(load(&dir), vec!["/projetos/a".to_owned()]);
        fs::write(
            trust_path_in(&dir),
            r#"{"schemaVersion":9,"roots":["/projetos/b"]}"#,
        )
        .unwrap();
        assert!(!is_trusted_in(&dir, "/projetos/b"));
        let _ = fs::remove_dir_all(&dir);
    }
}

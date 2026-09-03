//! Persistencia da escolha, em `.kinein/toolchain.json`.
//!
//! Mesmo molde do `runconfig.rs` e do `settings.rs`, e pelo mesmo motivo:
//! `schemaVersion` explicito, e arquivo invalido ou de schema desconhecido
//! tratado como VAZIO. Toolchain quebrada nao pode impedir a IDE de abrir o
//! projeto — sem escolha, tudo volta a ser "o que estiver no `PATH`", que e o
//! comportamento historico.

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

/// Versao do schema escrita por este core.
const SCHEMA_VERSION: u32 = 1;

/// Representacao em disco.
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ToolchainFile {
    schema_version: u32,
    /// Papel (chave estavel) -> id do candidato escolhido.
    #[serde(default)]
    selections: BTreeMap<String, String>,
}

/// Caminho do arquivo para um root de workspace.
#[must_use]
pub(super) fn path_for(root: &Path) -> PathBuf {
    root.join(".kinein").join("toolchain.json")
}

/// Le as escolhas persistidas; arquivo ausente/invalido = nenhuma escolha.
#[must_use]
pub(super) fn load(root: &Path) -> BTreeMap<String, String> {
    let Ok(body) = fs::read_to_string(path_for(root)) else {
        return BTreeMap::new();
    };
    match serde_json::from_str::<ToolchainFile>(&body) {
        Ok(file) if file.schema_version == SCHEMA_VERSION => file.selections,
        _ => BTreeMap::new(),
    }
}

/// Grava as escolhas, criando `.kinein/` se preciso.
pub(super) fn save(root: &Path, selections: &BTreeMap<String, String>) -> Result<(), String> {
    let target = path_for(root);
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("falha criando {}: {error}", parent.display()))?;
    }
    let file = ToolchainFile {
        schema_version: SCHEMA_VERSION,
        selections: selections.clone(),
    };
    let body = serde_json::to_string_pretty(&file)
        .map_err(|error| format!("falha serializando a toolchain: {error}"))?;
    fs::write(&target, body).map_err(|error| format!("falha escrevendo a toolchain: {error}"))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{load, path_for, save};

    fn temp_root(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir()
            .join("kinein-toolchain-store")
            .join(format!("{}-{name}", std::process::id()));
        if dir.exists() {
            std::fs::remove_dir_all(&dir).unwrap();
        }
        std::fs::create_dir_all(&dir).unwrap();
        dir.canonicalize().unwrap()
    }

    #[test]
    fn roundtrip_persists_the_choice() {
        let root = temp_root("roundtrip");
        assert!(load(&root).is_empty());

        let mut escolhas = BTreeMap::new();
        escolhas.insert("cxxCompiler".to_owned(), "gxx".to_owned());
        save(&root, &escolhas).unwrap();

        assert_eq!(
            load(&root).get("cxxCompiler").map(String::as_str),
            Some("gxx")
        );
    }

    #[test]
    fn unknown_schema_is_treated_as_empty() {
        let root = temp_root("schema");
        std::fs::create_dir_all(root.join(".kinein")).unwrap();
        std::fs::write(
            path_for(&root),
            "{\"schemaVersion\":99,\"selections\":{\"cxxCompiler\":\"gxx\"}}",
        )
        .unwrap();

        assert!(
            load(&root).is_empty(),
            "schema desconhecido nao pode ditar qual compilador roda"
        );
    }

    #[test]
    fn broken_file_does_not_break_the_workspace() {
        let root = temp_root("quebrado");
        std::fs::create_dir_all(root.join(".kinein")).unwrap();
        std::fs::write(path_for(&root), "nao e json").unwrap();

        assert!(load(&root).is_empty());
    }
}

//! Persistencia da escolha, em `.kinein/toolchain.json`.
//!
//! Mesmo molde do `runconfig.rs` e do `settings.rs`, e pelo mesmo motivo:
//! `schemaVersion` explicito, e arquivo invalido ou de schema desconhecido
//! tratado como VAZIO. Toolchain quebrada nao pode impedir a IDE de abrir o
//! projeto — sem escolha, tudo volta a ser "o que estiver no `PATH`", que e o
//! comportamento historico.
//!
//! SCHEMA 2 (2026-09-03, etapa 14 do `roadmaps/34`): a escolha deixou de ser
//! do workspace e passou a ser do KIT. Um kit e' um preset do `CMake` mais o que
//! ele precisa para compilar: os executaveis por papel, o `sysroot` e o triple
//! do alvo. O kit de chave vazia e' o padrao do workspace — que e' exatamente
//! o que o schema 1 guardava, e por isso a migracao e' direta e sem perda.

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

/// Versao do schema escrita por este core.
const SCHEMA_VERSION: u32 = 2;

/// Um kit: os executaveis escolhidos mais para onde eles compilam.
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(super) struct Kit {
    /// Papel (chave estavel) -> id do candidato escolhido.
    #[serde(default)]
    pub selections: BTreeMap<String, String>,
    /// Raiz do sistema alvo (`CMAKE_SYSROOT`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sysroot: Option<String>,
    /// Triple do alvo (`--target` do cargo).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_triple: Option<String>,
    /// Chip do alvo, para o adaptador de debug de embarcado.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chip: Option<String>,
}

impl Kit {
    /// `true` quando o kit nao guarda nada — nao vale a pena persistir.
    pub(super) fn is_empty(&self) -> bool {
        self.selections.is_empty()
            && self.sysroot.is_none()
            && self.target_triple.is_none()
            && self.chip.is_none()
    }
}

/// Representacao em disco (schema 2).
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ToolchainFile {
    schema_version: u32,
    /// Nome do preset -> kit. Chave vazia = o kit padrao do workspace.
    #[serde(default)]
    kits: BTreeMap<String, Kit>,
}

/// Representacao do schema 1, lida so' para migrar.
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ToolchainFileV1 {
    schema_version: u32,
    #[serde(default)]
    selections: BTreeMap<String, String>,
}

/// Caminho do arquivo para um root de workspace.
#[must_use]
pub(super) fn path_for(root: &Path) -> PathBuf {
    root.join(".kinein").join("toolchain.json")
}

/// Le os kits persistidos; arquivo ausente/invalido = nenhum kit.
///
/// Um arquivo do schema 1 e' MIGRADO em memoria: as selecoes dele viram o kit
/// padrao. Migrar na leitura, e nao na escrita, e' o que faz a IDE nao perder
/// a escolha de quem abriu o projeto e nao mexeu na toolchain.
#[must_use]
pub(super) fn load(root: &Path) -> BTreeMap<String, Kit> {
    let Ok(body) = fs::read_to_string(path_for(root)) else {
        return BTreeMap::new();
    };
    if let Ok(file) = serde_json::from_str::<ToolchainFile>(&body) {
        if file.schema_version == SCHEMA_VERSION {
            return file.kits;
        }
    }
    match serde_json::from_str::<ToolchainFileV1>(&body) {
        Ok(antigo) if antigo.schema_version == 1 && !antigo.selections.is_empty() => {
            let mut kits = BTreeMap::new();
            kits.insert(
                String::new(),
                Kit {
                    selections: antigo.selections,
                    sysroot: None,
                    target_triple: None,
                    chip: None,
                },
            );
            kits
        }
        _ => BTreeMap::new(),
    }
}

/// Grava os kits, criando `.kinein/` se preciso. Kit vazio nao e' persistido.
pub(super) fn save(root: &Path, kits: &BTreeMap<String, Kit>) -> Result<(), String> {
    let target = path_for(root);
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("falha criando {}: {error}", parent.display()))?;
    }
    let file = ToolchainFile {
        schema_version: SCHEMA_VERSION,
        kits: kits
            .iter()
            .filter(|(_, kit)| !kit.is_empty())
            .map(|(nome, kit)| (nome.clone(), kit.clone()))
            .collect(),
    };
    let body = serde_json::to_string_pretty(&file)
        .map_err(|error| format!("falha serializando a toolchain: {error}"))?;
    fs::write(&target, body).map_err(|error| format!("falha escrevendo a toolchain: {error}"))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{Kit, load, path_for, save};

    fn temp_root(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir()
            .join("kinein-toolchain-store")
            .join(format!("{}-{name}", std::process::id()));
        if dir.exists() {
            std::fs::remove_dir_all(&dir).unwrap();
        }
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn kits_roundtrip_and_empty_kit_is_not_written() {
        let root = temp_root("kits");
        assert!(load(&root).is_empty());

        let mut kits = BTreeMap::new();
        kits.insert(
            String::new(),
            Kit {
                selections: BTreeMap::from([("cxxCompiler".to_owned(), "gxx".to_owned())]),
                sysroot: None,
                target_triple: None,
                chip: None,
            },
        );
        kits.insert(
            "cross-arm".to_owned(),
            Kit {
                selections: BTreeMap::new(),
                sysroot: Some("/opt/sysroots/arm".to_owned()),
                target_triple: Some("aarch64-unknown-linux-gnu".to_owned()),
                chip: None,
            },
        );
        // Kit sem nada nao merece linha no arquivo.
        kits.insert("vazio".to_owned(), Kit::default());
        save(&root, &kits).unwrap();

        let lidos = load(&root);
        assert_eq!(
            lidos[""].selections.get("cxxCompiler").map(String::as_str),
            Some("gxx")
        );
        assert_eq!(
            lidos["cross-arm"].sysroot.as_deref(),
            Some("/opt/sysroots/arm")
        );
        assert!(!lidos.contains_key("vazio"), "kit vazio foi persistido");
    }

    /// Quem ja tinha `.kinein/toolchain.json` do schema 1 nao pode perder a
    /// escolha so' porque a IDE atualizou: as selecoes viram o kit padrao.
    #[test]
    fn schema_1_migrates_into_the_default_kit() {
        let root = temp_root("migracao");
        std::fs::create_dir_all(root.join(".kinein")).unwrap();
        std::fs::write(
            path_for(&root),
            r#"{ "schemaVersion": 1, "selections": { "cCompiler": "gcc" } }"#,
        )
        .unwrap();

        let lidos = load(&root);
        assert_eq!(
            lidos[""].selections.get("cCompiler").map(String::as_str),
            Some("gcc"),
            "a escolha do schema 1 tinha que virar o kit padrao"
        );
    }

    #[test]
    fn unknown_schema_and_broken_file_read_as_empty() {
        let root = temp_root("quebrado");
        std::fs::create_dir_all(root.join(".kinein")).unwrap();

        std::fs::write(path_for(&root), r#"{ "schemaVersion": 99, "kits": {} }"#).unwrap();
        assert!(load(&root).is_empty(), "schema desconhecido virou escolha");

        std::fs::write(path_for(&root), "{ nao e json").unwrap();
        assert!(load(&root).is_empty(), "arquivo quebrado virou escolha");
    }
}

//! Run configurations por workspace (`.kinein/runconfigs.json`).
//!
//! Uma run config v1 e um comando nomeado executado na raiz pelo `run.start`.
//! A configuracao ATIVA vive no arquivo (`activeId`): o botao de executar
//! resolve comando explicito > config ativa > heuristica. Arquivo invalido
//! ou com schema desconhecido e tratado como vazio — nunca quebra o fluxo.

use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use kinein_protocol::RunConfigInfo;

/// Versao do schema de `.kinein/runconfigs.json` escrita por este core.
const RUNCONFIG_SCHEMA_VERSION: u32 = 1;

/// Representacao em disco de `.kinein/runconfigs.json`.
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RunConfigsFile {
    schema_version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    active_id: Option<String>,
    #[serde(default)]
    configs: Vec<RunConfigInfo>,
}

/// Estado atual das run configs (lista + ativa).
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RunConfigsState {
    /// Configuracoes em ordem de criacao.
    pub configs: Vec<RunConfigInfo>,
    /// Config ativa, quando alguma.
    pub active_id: Option<String>,
}

/// Caminho de `.kinein/runconfigs.json` para um root de workspace.
#[must_use]
pub fn configs_path(root: &Path) -> PathBuf {
    root.join(".kinein").join("runconfigs.json")
}

fn read_file(root: &Path) -> RunConfigsFile {
    let Ok(body) = fs::read_to_string(configs_path(root)) else {
        return RunConfigsFile {
            schema_version: RUNCONFIG_SCHEMA_VERSION,
            ..RunConfigsFile::default()
        };
    };
    match serde_json::from_str::<RunConfigsFile>(&body) {
        Ok(file) if file.schema_version == RUNCONFIG_SCHEMA_VERSION => file,
        _ => RunConfigsFile {
            schema_version: RUNCONFIG_SCHEMA_VERSION,
            ..RunConfigsFile::default()
        },
    }
}

fn write_file(root: &Path, file: &RunConfigsFile) -> Result<(), String> {
    let target = configs_path(root);
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("falha criando {}: {error}", parent.display()))?;
    }
    let body = serde_json::to_string_pretty(file)
        .map_err(|error| format!("falha serializando run configs: {error}"))?;
    fs::write(&target, body).map_err(|error| format!("falha escrevendo run configs: {error}"))
}

fn state_of(file: RunConfigsFile) -> RunConfigsState {
    RunConfigsState {
        configs: file.configs,
        active_id: file.active_id,
    }
}

/// Lista as configs persistidas (arquivo ausente/invalido → vazio).
#[must_use]
pub fn load(root: &Path) -> RunConfigsState {
    state_of(read_file(root))
}

/// Comando da config ativa, quando ha uma.
#[must_use]
pub fn active_command(root: &Path) -> Option<String> {
    let file = read_file(root);
    let active = file.active_id.as_deref()?;
    file.configs
        .iter()
        .find(|config| config.id == active)
        .map(|config| config.command.clone())
}

/// Cria (`id` ausente) ou edita uma config; a config salva vira a ativa.
pub fn save(
    root: &Path,
    id: Option<&str>,
    name: &str,
    command: &str,
) -> Result<RunConfigsState, String> {
    let name = name.trim();
    let command = command.trim();
    if name.is_empty() || command.is_empty() {
        return Err("nome e comando nao podem ser vazios".to_owned());
    }

    let mut file = read_file(root);
    let saved_id = if let Some(id) = id {
        let Some(config) = file.configs.iter_mut().find(|config| config.id == id) else {
            return Err(format!("run config inexistente: {id}"));
        };
        name.clone_into(&mut config.name);
        command.clone_into(&mut config.command);
        id.to_owned()
    } else {
        let next = file
            .configs
            .iter()
            .filter_map(|config| {
                config
                    .id
                    .strip_prefix("cfg-")
                    .and_then(|suffix| suffix.parse::<u64>().ok())
            })
            .max()
            .unwrap_or(0)
            + 1;
        let new_id = format!("cfg-{next}");
        file.configs.push(RunConfigInfo {
            id: new_id.clone(),
            name: name.to_owned(),
            command: command.to_owned(),
        });
        new_id
    };
    file.active_id = Some(saved_id);
    write_file(root, &file)?;
    Ok(state_of(file))
}

/// Remove uma config; se era a ativa, volta para "automatico".
pub fn delete(root: &Path, id: &str) -> Result<RunConfigsState, String> {
    let mut file = read_file(root);
    let before = file.configs.len();
    file.configs.retain(|config| config.id != id);
    if file.configs.len() == before {
        return Err(format!("run config inexistente: {id}"));
    }
    if file.active_id.as_deref() == Some(id) {
        file.active_id = None;
    }
    write_file(root, &file)?;
    Ok(state_of(file))
}

/// Define a config ativa (`None` = automatico/heuristica).
pub fn set_active(root: &Path, id: Option<&str>) -> Result<RunConfigsState, String> {
    let mut file = read_file(root);
    if let Some(id) = id {
        if !file.configs.iter().any(|config| config.id == id) {
            return Err(format!("run config inexistente: {id}"));
        }
        file.active_id = Some(id.to_owned());
    } else {
        file.active_id = None;
    }
    write_file(root, &file)?;
    Ok(state_of(file))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{active_command, configs_path, delete, load, save, set_active};

    fn temp_root(name: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("kinein-runconfig-tests")
            .join(format!("{}-{name}", std::process::id()));
        if dir.exists() {
            std::fs::remove_dir_all(&dir).unwrap();
        }
        std::fs::create_dir_all(&dir).unwrap();
        dir.canonicalize().unwrap()
    }

    #[test]
    fn crud_roundtrip_tracks_active_config() {
        let root = temp_root("crud");
        assert!(load(&root).configs.is_empty());

        let created = save(&root, None, "Servidor", "cargo run --bin server").unwrap();
        assert_eq!(created.configs.len(), 1);
        assert_eq!(created.active_id.as_deref(), Some("cfg-1"));
        assert_eq!(
            active_command(&root).as_deref(),
            Some("cargo run --bin server")
        );

        let second = save(&root, None, "Cliente", "cargo run --bin client").unwrap();
        assert_eq!(second.configs.len(), 2);
        assert_eq!(second.active_id.as_deref(), Some("cfg-2"));

        let edited = save(&root, Some("cfg-1"), "Servidor Web", "cargo run").unwrap();
        assert_eq!(edited.configs[0].name, "Servidor Web");
        assert_eq!(edited.active_id.as_deref(), Some("cfg-1"));

        let back = set_active(&root, Some("cfg-2")).unwrap();
        assert_eq!(back.active_id.as_deref(), Some("cfg-2"));
        let automatic = set_active(&root, None).unwrap();
        assert_eq!(automatic.active_id, None);
        assert_eq!(active_command(&root), None);

        set_active(&root, Some("cfg-2")).unwrap();
        let removed = delete(&root, "cfg-2").unwrap();
        assert_eq!(removed.configs.len(), 1);
        assert_eq!(removed.active_id, None);
    }

    #[test]
    fn invalid_inputs_are_rejected() {
        let root = temp_root("invalid");
        assert!(save(&root, None, "  ", "echo").is_err());
        assert!(save(&root, None, "x", "").is_err());
        assert!(save(&root, Some("cfg-9"), "x", "echo").is_err());
        assert!(delete(&root, "cfg-9").is_err());
        assert!(set_active(&root, Some("cfg-9")).is_err());
    }

    #[test]
    fn unknown_schema_is_treated_as_empty() {
        let root = temp_root("schema");
        std::fs::create_dir_all(root.join(".kinein")).unwrap();
        std::fs::write(
            configs_path(&root),
            "{\"schemaVersion\":99,\"configs\":[{\"id\":\"cfg-1\",\"name\":\"x\",\"command\":\"y\"}]}",
        )
        .unwrap();
        assert!(load(&root).configs.is_empty());
    }
}

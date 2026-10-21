//! Run configurations por workspace (`.kinein/runconfigs.json`).
//!
//! Uma run config v1 e um comando nomeado executado na raiz pelo `run.start`.
//! A configuracao ATIVA vive no arquivo (`activeId`): o botao de executar
//! resolve comando explicito > config ativa > heuristica. Arquivo invalido
//! ou com schema desconhecido e recusado, preservando dados e intencao.

use std::{
    collections::BTreeSet,
    fs,
    io::Read,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use kinein_protocol::RunConfigInfo;

/// Versao do schema de `.kinein/runconfigs.json` escrita por este core.
const RUNCONFIG_SCHEMA_VERSION: u32 = 1;

/// Representacao em disco de `.kinein/runconfigs.json`.
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
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

/// Invalid input differs from unreadable persisted intent.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RunConfigError {
    /// The caller supplied an invalid name, command or identifier.
    Invalid(String),
    /// Persisted data could not be safely read or published.
    Storage(String),
}

impl std::fmt::Display for RunConfigError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid(message) | Self::Storage(message) => formatter.write_str(message),
        }
    }
}

fn storage_error(root: &Path, reason: impl std::fmt::Display) -> RunConfigError {
    RunConfigError::Storage(format!(
        "Não foi possível usar {}: {reason}. O arquivo não foi substituído. Confira o conteúdo e o acesso; preserve uma cópia antes de corrigir e tente novamente.",
        configs_path(root).display()
    ))
}

fn read_file(root: &Path) -> Result<RunConfigsFile, RunConfigError> {
    let target = configs_path(root);
    match fs::symlink_metadata(&target) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(RunConfigsFile {
                schema_version: RUNCONFIG_SCHEMA_VERSION,
                ..RunConfigsFile::default()
            });
        }
        Err(error) => return Err(storage_error(root, error)),
        Ok(metadata) if !metadata.is_file() => {
            return Err(storage_error(root, "o caminho não é um arquivo regular"));
        }
        Ok(_) => {}
    }
    let mut bytes = Vec::new();
    fs::File::open(target)
        .map_err(|error| storage_error(root, error))?
        .take(crate::fsops::MAX_READ_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| storage_error(root, error))?;
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > crate::fsops::MAX_READ_BYTES {
        return Err(storage_error(
            root,
            "o arquivo excede o limite de leitura da IDE",
        ));
    }
    let body = String::from_utf8(bytes).map_err(|error| storage_error(root, error))?;
    let file: RunConfigsFile =
        serde_json::from_str(&body).map_err(|error| storage_error(root, error))?;
    if file.schema_version != RUNCONFIG_SCHEMA_VERSION {
        return Err(storage_error(
            root,
            format!("schema {} não suportado", file.schema_version),
        ));
    }
    let mut ids = BTreeSet::new();
    for config in &file.configs {
        if config.id.trim().is_empty()
            || config.name.trim().is_empty()
            || config.command.trim().is_empty()
            || !ids.insert(&config.id)
        {
            return Err(storage_error(
                root,
                "há entradas vazias ou identidades duplicadas",
            ));
        }
    }
    if file.active_id.as_ref().is_some_and(|id| !ids.contains(id)) {
        return Err(storage_error(
            root,
            "a configuração ativa não existe na lista",
        ));
    }
    Ok(file)
}

fn mutation_guard(root: &Path) -> Result<fs::File, RunConfigError> {
    let directory = root.join(".kinein");
    fs::create_dir_all(&directory).map_err(|error| storage_error(root, error))?;
    if !fs::symlink_metadata(&directory)
        .map_err(|error| storage_error(root, error))?
        .is_dir()
    {
        return Err(storage_error(
            root,
            "a pasta .kinein não é um diretório regular",
        ));
    }
    crate::platform::configuration_lock(&directory.join("runconfigs.lock"))
        .map_err(|error| storage_error(root, error))
}

fn write_file(root: &Path, file: &RunConfigsFile) -> Result<(), RunConfigError> {
    let body = serde_json::to_string_pretty(file).map_err(|error| storage_error(root, error))?;
    if u64::try_from(body.len()).unwrap_or(u64::MAX) > crate::fsops::MAX_READ_BYTES {
        return Err(RunConfigError::Invalid(
            "As configurações excederiam o limite de leitura da IDE. Mantenha scripts grandes em arquivos e use comandos para executá-los.".to_owned()));
    }
    crate::fsops::atomic_write(&configs_path(root), body.as_bytes())
        .map_err(|error| storage_error(root, error))
}

fn state_of(file: RunConfigsFile) -> RunConfigsState {
    RunConfigsState {
        configs: file.configs,
        active_id: file.active_id,
    }
}

/// Lists persisted configurations; only an absent file means empty.
pub fn load(root: &Path) -> Result<RunConfigsState, RunConfigError> {
    read_file(root).map(state_of)
}

/// Comando da config ativa, quando ha uma.
pub fn active_command(root: &Path) -> Result<Option<String>, RunConfigError> {
    let file = read_file(root)?;
    let Some(active) = file.active_id.as_deref() else {
        return Ok(None);
    };
    Ok(file
        .configs
        .iter()
        .find(|config| config.id == active)
        .map(|config| config.command.clone()))
}

/// Cria (`id` ausente) ou edita uma config; a config salva vira a ativa.
pub fn save(
    root: &Path,
    id: Option<&str>,
    name: &str,
    command: &str,
) -> Result<RunConfigsState, RunConfigError> {
    let name = name.trim();
    let command = command.trim();
    if name.is_empty() || command.is_empty() {
        return Err(RunConfigError::Invalid(
            "nome e comando não podem ser vazios".to_owned(),
        ));
    }

    let _guard = mutation_guard(root)?;
    let mut file = read_file(root)?;
    let saved_id = if let Some(id) = id {
        let Some(config) = file.configs.iter_mut().find(|config| config.id == id) else {
            return Err(RunConfigError::Invalid(format!(
                "run config inexistente: {id}"
            )));
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
            .checked_add(1)
            .ok_or_else(|| {
                RunConfigError::Invalid("limite de identificadores atingido".to_owned())
            })?;
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
pub fn delete(root: &Path, id: &str) -> Result<RunConfigsState, RunConfigError> {
    let _guard = mutation_guard(root)?;
    let mut file = read_file(root)?;
    let before = file.configs.len();
    file.configs.retain(|config| config.id != id);
    if file.configs.len() == before {
        return Err(RunConfigError::Invalid(format!(
            "run config inexistente: {id}"
        )));
    }
    if file.active_id.as_deref() == Some(id) {
        file.active_id = None;
    }
    write_file(root, &file)?;
    Ok(state_of(file))
}

/// Define a config ativa (`None` = automatico/heuristica).
pub fn set_active(root: &Path, id: Option<&str>) -> Result<RunConfigsState, RunConfigError> {
    let _guard = mutation_guard(root)?;
    let mut file = read_file(root)?;
    if let Some(id) = id {
        if !file.configs.iter().any(|config| config.id == id) {
            return Err(RunConfigError::Invalid(format!(
                "run config inexistente: {id}"
            )));
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
        crate::platform::canonicalize(&dir).unwrap()
    }

    #[test]
    fn crud_roundtrip_tracks_active_config() {
        let root = temp_root("crud");
        assert!(load(&root).unwrap().configs.is_empty());

        let created = save(&root, None, "Servidor", "cargo run --bin server").unwrap();
        assert_eq!(created.configs.len(), 1);
        assert_eq!(created.active_id.as_deref(), Some("cfg-1"));
        assert_eq!(
            active_command(&root).unwrap().as_deref(),
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
        assert_eq!(active_command(&root).unwrap(), None);

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
    fn unknown_schema_is_refused_without_assuming_empty() {
        let root = temp_root("schema");
        std::fs::create_dir_all(root.join(".kinein")).unwrap();
        std::fs::write(
            configs_path(&root),
            "{\"schemaVersion\":99,\"configs\":[{\"id\":\"cfg-1\",\"name\":\"x\",\"command\":\"y\"}]}",
        )
        .unwrap();
        assert!(load(&root).is_err());
    }
    #[test]
    fn mutation_conflict_preserves_bytes_and_releases_after_guard_drop() {
        let root = temp_root("guard-conflict");
        save(&root, None, "Old", "echo old").unwrap();
        let path = configs_path(&root);
        let bytes = std::fs::read(&path).unwrap();
        let guard = super::mutation_guard(&root).unwrap();
        assert!(matches!(
            save(&root, None, "New", "echo new"),
            Err(super::RunConfigError::Storage(_))
        ));
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        assert_eq!(load(&root).unwrap().configs.len(), 1);
        drop(guard);
        assert_eq!(
            save(&root, None, "New", "echo new").unwrap().configs.len(),
            2
        );
    }

    #[test]
    fn publishing_keeps_an_open_snapshot_intact_and_replaces_the_complete_file() {
        use std::io::Read;
        let root = temp_root("atomic-snapshot");
        save(&root, None, "Old", "echo old").unwrap();
        let mut previous = std::fs::File::open(configs_path(&root)).unwrap();
        save(&root, Some("cfg-1"), "New", "echo new").unwrap();
        let mut body = String::new();
        previous.read_to_string(&mut body).unwrap();
        assert!(body.contains("echo old") && !body.contains("echo new"));
        assert_eq!(load(&root).unwrap().configs[0].command, "echo new");
    }

    #[test]
    fn concurrent_configuration_writer() {
        use std::{
            thread,
            time::{Duration, Instant},
        };
        let Some(root) = std::env::var_os("KINEIN_RUNCONFIG_TEST_ROOT") else {
            return;
        };
        let root = PathBuf::from(root);
        let writer = std::env::var("KINEIN_RUNCONFIG_TEST_WRITER").unwrap();
        std::fs::write(root.join(format!("ready-{writer}")), "ready").unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        while !root.join("start").exists() {
            assert!(Instant::now() < deadline, "parent did not release writers");
            thread::sleep(Duration::from_millis(5));
        }
        for index in 0..8 {
            let name = format!("{writer}-{index}");
            loop {
                match save(&root, None, &name, "echo fixture") {
                    Ok(_) => break,
                    Err(super::RunConfigError::Storage(message))
                        if message.contains("outra instância") =>
                    {
                        assert!(Instant::now() < deadline, "writer never acquired the guard");
                        thread::sleep(Duration::from_millis(5));
                    }
                    Err(error) => panic!("unexpected write refusal: {error}"),
                }
            }
        }
    }

    #[test]
    fn two_processes_keep_every_successful_update() {
        use std::{
            process::{Command, Stdio},
            thread,
            time::{Duration, Instant},
        };
        let root = temp_root("two-writers");
        let mut children = Vec::new();
        for writer in ["one", "two"] {
            children.push(
                Command::new(std::env::current_exe().unwrap())
                    .args([
                        "--exact",
                        "runconfig::tests::concurrent_configuration_writer",
                        "--nocapture",
                    ])
                    .env("KINEIN_RUNCONFIG_TEST_ROOT", &root)
                    .env("KINEIN_RUNCONFIG_TEST_WRITER", writer)
                    .stdin(Stdio::null())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                    .unwrap(),
            );
        }
        let deadline = Instant::now() + Duration::from_secs(5);
        while !root.join("ready-one").exists() || !root.join("ready-two").exists() {
            assert!(Instant::now() < deadline, "writers did not become ready");
            thread::sleep(Duration::from_millis(5));
        }
        std::fs::write(root.join("start"), "start").unwrap();
        for child in children {
            let output = child.wait_with_output().unwrap();
            assert!(
                output.status.success(),
                "writer failed: {} {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        let state = load(&root).unwrap();
        assert_eq!(state.configs.len(), 16);
        for writer in ["one", "two"] {
            for index in 0..8 {
                assert!(
                    state
                        .configs
                        .iter()
                        .any(|config| config.name == format!("{writer}-{index}"))
                );
            }
        }
    }
    #[test]
    fn oversized_storage_and_updates_are_refused_without_replacing_files() {
        let root = temp_root("storage-size");
        save(&root, None, "Old", "echo old").unwrap();
        let path = configs_path(&root);
        let original = std::fs::read(&path).unwrap();
        let large = "x".repeat(usize::try_from(crate::fsops::MAX_READ_BYTES).unwrap());
        assert!(matches!(
            save(&root, None, "Large", &large),
            Err(super::RunConfigError::Invalid(_))
        ));
        assert_eq!(std::fs::read(&path).unwrap(), original);
        std::fs::File::create(&path)
            .unwrap()
            .set_len(crate::fsops::MAX_READ_BYTES + 1)
            .unwrap();
        assert!(matches!(
            load(&root),
            Err(super::RunConfigError::Storage(_))
        ));
        assert_eq!(
            std::fs::metadata(path).unwrap().len(),
            crate::fsops::MAX_READ_BYTES + 1
        );
    }
}

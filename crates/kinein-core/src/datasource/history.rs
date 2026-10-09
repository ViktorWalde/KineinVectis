//! Historico de consultas por conexao (passo 13b, `roadmaps/59` §5.2.1).
//!
//! Decisao do autor (2026-10-08): o historico fica FORA do projeto, no estado
//! do usuario, separado por projeto, nunca em `.kinein/` nem no Git. O SQL pode
//! trazer senha (`ALTER USER ... PASSWORD`), entao ele nao viaja com o
//! repositorio: pasta `0700`, arquivo `0600`.
//!
//! O dono e' o core porque e' ele quem executa: cada `datasource.query` que
//! RODOU vira uma entrada. Um arquivo por projeto, com nome pelo SHA-256 do
//! caminho canonico da raiz; o arquivo guarda o caminho para conferir. Arquivo
//! ilegivel, de outra versao ou de outro projeto fica INTACTO: o historico
//! aparece vazio e nada e' gravado por cima.

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

use kinein_protocol::{DataSourceHistoryEntry, DataSourceHistoryOutcome};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Entradas guardadas por conexao; as mais antigas saem.
pub const MAX_ENTRIES: usize = 100;

/// Texto maior que isto nao entra (um script colado inteiro nao e' historico).
pub const MAX_SQL_BYTES: usize = 64 * 1024;

const SCHEMA_VERSION: u32 = 1;

/// Uma gravacao por vez no processo: os jobs de consulta rodam em paralelo e
/// cada gravacao le, muda e reescreve o arquivo inteiro.
static WRITING: Mutex<()> = Mutex::new(());

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct HistoryFile {
    schema_version: u32,
    workspace: String,
    connections: BTreeMap<String, Vec<DataSourceHistoryEntry>>,
}

/// Onde o historico mora: a pasta (o core usa a do estado do usuario).
#[derive(Debug, Clone)]
pub struct History {
    dir: PathBuf,
}

impl History {
    /// O historico numa pasta escolhida (os testes usam uma temporaria).
    #[must_use]
    pub const fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    /// A pasta do estado do usuario: `$XDG_STATE_HOME/kinein-vectis/datasource-history`.
    #[must_use]
    pub fn in_user_state() -> Self {
        Self::new(
            crate::settings::state_home()
                .join("kinein-vectis")
                .join("datasource-history"),
        )
    }

    /// As entradas de `name` no projeto `root`, a mais recente primeiro;
    /// vazio quando nao ha arquivo ou ele nao pode ser lido.
    #[must_use]
    pub fn list(&self, root: &Path, name: &str) -> Vec<DataSourceHistoryEntry> {
        self.read(root)
            .ok()
            .flatten()
            .and_then(|mut file| file.connections.remove(name))
            .unwrap_or_default()
    }

    /// Anota uma instrucao que rodou. A repeticao imediata do mesmo texto (o
    /// "Carregar mais") atualiza a entrada do topo.
    ///
    /// # Errors
    /// Arquivo existente que nao pode ser lido (fica intacto) ou falha de disco.
    pub fn record(
        &self,
        root: &Path,
        name: &str,
        sql: &str,
        outcome: DataSourceHistoryOutcome,
        rows: Option<u64>,
    ) -> Result<(), String> {
        if sql.trim().is_empty() || sql.len() > MAX_SQL_BYTES {
            return Ok(());
        }
        let entry = DataSourceHistoryEntry {
            sql: sql.to_owned(),
            at: now(),
            outcome,
            rows,
        };
        self.update(root, |file| {
            let entries = file.connections.entry(name.to_owned()).or_default();
            if entries.first().is_some_and(|top| top.sql == entry.sql) {
                entries.remove(0);
            }
            entries.insert(0, entry);
            entries.truncate(MAX_ENTRIES);
        })
    }

    /// Apaga o historico de `name` (pedido da pessoa, ou o perfil removido).
    ///
    /// # Errors
    /// Arquivo existente que nao pode ser lido (fica intacto) ou falha de disco.
    pub fn clear(&self, root: &Path, name: &str) -> Result<(), String> {
        if self
            .read(root)?
            .is_none_or(|file| !file.connections.contains_key(name))
        {
            return Ok(());
        }
        self.update(root, |file| {
            file.connections.remove(name);
        })
    }

    fn path_for(&self, workspace: &str) -> PathBuf {
        let mut hash = Sha256::new();
        hash.update(workspace.as_bytes());
        self.dir.join(format!("{}.json", super::hex_digest(hash)))
    }

    /// `Ok(None)` sem arquivo; `Err` quando ele existe e nao serve.
    fn read(&self, root: &Path) -> Result<Option<HistoryFile>, String> {
        let workspace = workspace_key(root);
        let path = self.path_for(&workspace);
        let body = match fs::read_to_string(&path) {
            Ok(body) => body,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(format!("{}: {error}", path.display())),
        };
        let file: HistoryFile = serde_json::from_str(&body)
            .map_err(|error| format!("{}: historico ilegivel ({error})", path.display()))?;
        if file.schema_version != SCHEMA_VERSION || file.workspace != workspace {
            return Err(format!(
                "{}: historico de outra versao ou de outro projeto",
                path.display()
            ));
        }
        Ok(Some(file))
    }

    fn update(&self, root: &Path, change: impl FnOnce(&mut HistoryFile)) -> Result<(), String> {
        let _writing = WRITING
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let workspace = workspace_key(root);
        let mut file = self.read(root)?.unwrap_or_else(|| HistoryFile {
            schema_version: SCHEMA_VERSION,
            workspace: workspace.clone(),
            connections: BTreeMap::new(),
        });
        change(&mut file);
        file.connections.retain(|_, entries| !entries.is_empty());
        self.private_dir()?;
        let body = serde_json::to_vec_pretty(&file).map_err(|error| error.to_string())?;
        crate::fsops::atomic_write_private(&self.path_for(&workspace), &body)
            .map_err(|error| error.to_string())
    }

    /// A pasta existe e so' o dono entra, mesmo se ja' existia mais aberta.
    fn private_dir(&self) -> Result<(), String> {
        let describe = |error: std::io::Error| format!("{}: {error}", self.dir.display());
        crate::platform::owner_only_dir_builder()
            .recursive(true)
            .create(&self.dir)
            .map_err(describe)?;
        crate::platform::restrict_dir_to_owner(&self.dir).map_err(describe)
    }
}

/// O caminho canonico da raiz: o mesmo projeto aberto por outro caminho
/// (link simbolico) cai no mesmo historico.
fn workspace_key(root: &Path) -> String {
    crate::platform::canonicalize(root)
        .unwrap_or_else(|_| root.to_path_buf())
        .display()
        .to_string()
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs())
}

#[cfg(test)]
mod tests;

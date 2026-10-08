//! unixODBC: descobrir sem conectar e recusar carregamento sem o gesto.
//! API segura da dependencia; nenhum FFI ou `unsafe` neste repositorio.

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use kinein_protocol::{DataSourceEngine, DataSourceOdbcSource, DataSourceProfile};
use odbc_api::{Connection, ConnectionOptions, Environment};
use sha2::{Digest, Sha256};

use super::connection::ConnectionFailure;
use super::secret::Secret;

type ApprovalKey = (PathBuf, String);
type Approval = (DataSourceProfile, String);

/// Aprovacoes efemeras, separadas do catalogo persistido.
#[derive(Debug, Default)]
pub struct Session {
    approvals: Mutex<HashMap<ApprovalKey, Approval>>,
    #[cfg(test)]
    sources_override: Option<Vec<DataSourceOdbcSource>>,
}

impl Session {
    /// Um gerenciador falso para provar recusa antes de carregar codigo.
    #[cfg(test)]
    pub(crate) fn with_sources(sources: Vec<DataSourceOdbcSource>) -> Self {
        Self {
            sources_override: Some(sources),
            ..Self::default()
        }
    }

    /// A lista publica, sem chamar `SQLConnect`.
    pub fn sources(&self) -> Result<Vec<DataSourceOdbcSource>, String> {
        #[cfg(test)]
        if let Some(sources) = &self.sources_override {
            return Ok(sources.clone());
        }
        sources()
    }

    /// A fonte atual do perfil; nao aceita string de conexao no lugar de DSN.
    pub fn source(&self, profile: &DataSourceProfile) -> Result<DataSourceOdbcSource, String> {
        validate_dsn(&profile.database)?;
        self.sources()?
            .into_iter()
            .find(|s| s.dsn == profile.database)
            .ok_or_else(|| "o DSN não está registrado no unixODBC; atualize a lista".to_owned())
    }

    /// Confere o que foi mostrado e so' depois guarda a aprovacao em memoria.
    pub fn authorize(
        &self,
        root: &Path,
        profile: &DataSourceProfile,
        identity: &str,
    ) -> Result<(), String> {
        if profile.engine != DataSourceEngine::Odbc {
            return Err("este perfil não usa ODBC".to_owned());
        }
        let source = self.source(profile)?;
        if challenge(root, profile, &source.identity) != identity {
            return Err("o driver mudou; atualize a lista e confirme novamente".to_owned());
        }
        self.approvals
            .lock()
            .map_err(|_| "a sessão ODBC está indisponível")?
            .insert(
                (root.to_path_buf(), profile.name.clone()),
                (profile.clone(), source.identity),
            );
        Ok(())
    }

    /// `None` libera; `Some` exige o aviso com a fonte atual.
    pub fn required(
        &self,
        root: &Path,
        profile: &DataSourceProfile,
    ) -> Result<Option<DataSourceOdbcSource>, String> {
        if profile.engine != DataSourceEngine::Odbc {
            return Ok(None);
        }
        let mut source = self.source(profile)?;
        let approved = self
            .approvals
            .lock()
            .map_err(|_| "a sessão ODBC está indisponível")?
            .get(&(root.to_path_buf(), profile.name.clone()))
            .is_some_and(|(saved, identity)| saved == profile && identity == &source.identity);
        source.identity = challenge(root, profile, &source.identity);
        Ok(if approved { None } else { Some(source) })
    }

    /// Remover e recriar um perfil exige outro gesto, mesmo com o mesmo nome.
    pub fn revoke(&self, root: &Path, name: &str) {
        if let Ok(mut approvals) = self.approvals.lock() {
            approvals.remove(&(root.to_path_buf(), name.to_owned()));
        }
    }

    /// Troca de projeto encerra as aprovacoes anteriores.
    pub fn clear(&self) {
        if let Ok(mut approvals) = self.approvals.lock() {
            approvals.clear();
        }
    }
}

/// Valida o nome como um DSN, nunca como uma connection string.
pub fn validate_dsn(dsn: &str) -> Result<(), String> {
    if dsn.is_empty()
        || dsn.len() > 32
        || dsn != dsn.trim()
        || dsn.chars().any(|c| c.is_control() || "[]{};=".contains(c))
    {
        return Err(
            "escolha um DSN registrado (até 32 bytes); não informe uma string de conexão"
                .to_owned(),
        );
    }
    Ok(())
}

fn challenge(root: &Path, profile: &DataSourceProfile, driver: &str) -> String {
    let mut hash = Sha256::new();
    hash.update(root.as_os_str().as_encoded_bytes());
    hash.update([0]);
    hash.update(serde_json::to_vec(profile).unwrap_or_default());
    hash.update([0]);
    hash.update(driver.as_bytes());
    super::hex_digest(hash)
}

fn manager() -> Result<&'static Environment, String> {
    // Serializa somente a inicializacao: a dependencia recomenda um ambiente
    // por processo. Nenhuma conexao de rede espera sob este lock.
    static INIT: Mutex<()> = Mutex::new(());
    let _guard = INIT
        .lock()
        .map_err(|_| "o gerenciador ODBC está indisponível")?;
    odbc_api::environment().map_err(|_| "não foi possível iniciar o unixODBC".to_owned())
}

/// `SQLDataSources`/`SQLDrivers` leem o registro; nao carregam a biblioteca do banco.
pub fn sources() -> Result<Vec<DataSourceOdbcSource>, String> {
    // ODBC guarda o cursor de enumeracao no ambiente; nao intercalar chamadas.
    static DISCOVERY: Mutex<()> = Mutex::new(());
    let _guard = DISCOVERY
        .lock()
        .map_err(|_| "a descoberta ODBC está indisponível")?;
    let env = manager()?;
    let drivers = env
        .drivers()
        .map_err(|_| "não foi possível listar os drivers ODBC")?;
    let mut result = BTreeMap::new();
    for source in env
        .data_sources()
        .map_err(|_| "não foi possível listar os DSN ODBC")?
    {
        if validate_dsn(&source.server_name).is_err() {
            continue;
        }
        let library = drivers
            .iter()
            .find(|d| d.description == source.driver)
            .and_then(|d| {
                d.attributes
                    .iter()
                    .find(|(k, _)| {
                        cfg!(target_pointer_width = "64") && k.eq_ignore_ascii_case("driver64")
                    })
                    .or_else(|| {
                        d.attributes
                            .iter()
                            .find(|(k, _)| k.eq_ignore_ascii_case("driver"))
                    })
            })
            .map_or(source.driver.as_str(), |(_, path)| path.as_str());
        let identity = driver_identity(&source.server_name, &source.driver, library);
        // A ordem do gerenciador preserva a precedencia do DSN do usuario.
        result
            .entry(source.server_name.clone())
            .or_insert(DataSourceOdbcSource {
                dsn: source.server_name,
                driver: source.driver,
                identity,
            });
    }
    Ok(result.into_values().collect())
}

fn driver_identity(dsn: &str, driver: &str, library: &str) -> String {
    let path = Path::new(library);
    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let metadata = canonical.metadata().ok();
    let modified = metadata.as_ref().and_then(|m| m.modified().ok());
    let size = metadata.as_ref().map(std::fs::Metadata::len);
    let mut hash = Sha256::new();
    for value in [
        dsn.to_owned(),
        driver.to_owned(),
        canonical.display().to_string(),
        format!("{size:?}/{modified:?}"),
    ] {
        hash.update(value.as_bytes());
        hash.update([0]);
    }
    super::hex_digest(hash)
}

/// Nao propaga o texto arbitrario do driver: ele pode repetir a senha/DSN.
#[must_use]
pub fn failure(error: &odbc_api::Error, operation: &str) -> ConnectionFailure {
    let state = match error {
        odbc_api::Error::Diagnostics { record, .. } => Some(record.state.as_str().to_owned()),
        _ => None,
    };
    ConnectionFailure {
        message: format!(
            "ODBC: falha ao {operation}{}; confira o DSN e o driver instalado",
            state
                .as_ref()
                .map_or(String::new(), |s| format!(" (SQLSTATE {s})"))
        ),
        secret_required: state.as_deref() == Some("28000"),
        sql_state: state,
    }
}

/// `SQLConnect` recebe DSN/usuario/senha em campos separados; nada vai a shell.
pub fn connect(
    profile: &DataSourceProfile,
    secret: Option<&Secret>,
) -> Result<Connection<'static>, ConnectionFailure> {
    let env = manager().map_err(|message| ConnectionFailure {
        message,
        sql_state: None,
        secret_required: false,
    })?;
    env.connect(
        &profile.database,
        &profile.user,
        secret.map_or("", Secret::expose),
        ConnectionOptions {
            login_timeout_sec: Some(5),
            ..ConnectionOptions::default()
        },
    )
    .map_err(|error| failure(&error, "conectar"))
}

/// O teste nao inventa uma consulta num dialeto desconhecido.
pub fn probe_server(
    profile: &DataSourceProfile,
    secret: Option<&Secret>,
) -> Result<String, ConnectionFailure> {
    let connection = connect(profile, secret)?;
    connection
        .database_management_system_name()
        .map_err(|e| failure(&e, "identificar o servidor"))
}

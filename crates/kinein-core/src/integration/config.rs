//! Configuração genérica das integrações (fatia 2.2, escrita do contrato v1).
//!
//! Chave→valor por integração, em DOIS escopos: `workspace` (o `kinein.db`
//! do workspace aberto, o mesmo dos rascunhos) e `global` (um `kinein.db` no
//! diretório XDG de estado). O escopo é o ARQUIVO — não há coluna de escopo.
//!
//! Reversível por construção: `set` grava uma sobreposição, `reset` a remove,
//! e o estado "sem sobreposição" (o default da vertical) é sempre alcançável.
//! O domínio não emite evento — ele DEVOLVE se algo mudou, e o handler decide.

use kinein_protocol::{IntegrationConfigEntry, IntegrationConfigGetResult, IntegrationConfigScope};

use crate::db::DraftStore;

/// Erro de configuração: escopo pedido sem store disponível (ex.: escopo
/// `workspace` sem workspace aberto) ou falha do `SQLite`.
#[derive(Debug)]
pub enum ConfigError {
    /// O escopo pedido não tem store aberta.
    ScopeUnavailable(IntegrationConfigScope),
    /// Falha de leitura/escrita no `SQLite`.
    Storage(rusqlite::Error),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ScopeUnavailable(IntegrationConfigScope::Workspace) => {
                write!(f, "escopo workspace exige um workspace aberto")
            }
            Self::ScopeUnavailable(IntegrationConfigScope::Global) => {
                write!(f, "armazenamento global indisponivel")
            }
            Self::Storage(error) => write!(f, "falha de armazenamento: {error}"),
        }
    }
}

impl From<rusqlite::Error> for ConfigError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Storage(error)
    }
}

/// Os dois stores possíveis, já resolvidos pelo chamador (`Core`).
#[derive(Debug)]
pub struct ConfigStores<'a> {
    /// Store global (XDG); `None` quando o diretório não pôde ser criado.
    pub global: Option<&'a DraftStore>,
    /// Store do workspace aberto; `None` sem workspace.
    pub workspace: Option<&'a DraftStore>,
}

impl ConfigStores<'_> {
    fn for_scope(&self, scope: IntegrationConfigScope) -> Result<&DraftStore, ConfigError> {
        let store = match scope {
            IntegrationConfigScope::Global => self.global,
            IntegrationConfigScope::Workspace => self.workspace,
        };
        store.ok_or(ConfigError::ScopeUnavailable(scope))
    }
}

/// Lê a configuração armazenada de uma integração nos dois escopos.
///
/// Um escopo sem store simplesmente não contribui entradas — leitura nunca
/// falha por falta de workspace. `workspace` vem depois de `global` na lista:
/// para a mesma chave, a UI aplica o último (sobreposição).
pub fn get(stores: &ConfigStores<'_>, id: &str) -> Result<IntegrationConfigGetResult, ConfigError> {
    let mut entries = Vec::new();
    if let Some(store) = stores.global {
        for (key, value) in store.config_list(id)? {
            entries.push(IntegrationConfigEntry {
                key,
                value,
                scope: IntegrationConfigScope::Global,
            });
        }
    }
    if let Some(store) = stores.workspace {
        for (key, value) in store.config_list(id)? {
            entries.push(IntegrationConfigEntry {
                key,
                value,
                scope: IntegrationConfigScope::Workspace,
            });
        }
    }
    Ok(IntegrationConfigGetResult {
        id: id.to_owned(),
        entries,
    })
}

/// Grava um valor no escopo pedido. `Ok(true)` = o valor MUDOU (evento);
/// `Ok(false)` = set idempotente, sem evento.
pub fn set(
    stores: &ConfigStores<'_>,
    id: &str,
    key: &str,
    value: &str,
    scope: IntegrationConfigScope,
) -> Result<bool, ConfigError> {
    Ok(stores.for_scope(scope)?.config_set(id, key, value)?)
}

/// Remove a sobreposição de uma chave no escopo pedido. `Ok(true)` = havia e
/// saiu (evento); `Ok(false)` = já estava no default, sem evento.
pub fn reset(
    stores: &ConfigStores<'_>,
    id: &str,
    key: &str,
    scope: IntegrationConfigScope,
) -> Result<bool, ConfigError> {
    Ok(stores.for_scope(scope)?.config_reset(id, key)?)
}

#[cfg(test)]
mod tests {
    use kinein_protocol::IntegrationConfigScope;

    use super::{ConfigStores, get, reset, set};
    use crate::db::DraftStore;

    fn store(nome: &str) -> DraftStore {
        let dir = std::env::temp_dir()
            .join("kinein-core-tests")
            .join(format!("{}-intconfig-{nome}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        DraftStore::open_in(&dir).expect("store temporaria")
    }

    #[test]
    fn escopo_workspace_nao_vaza_para_global_e_reset_e_reversivel() {
        let global = store("global");
        let workspace = store("workspace");
        let stores = ConfigStores {
            global: Some(&global),
            workspace: Some(&workspace),
        };

        // set no workspace: aparece SO no escopo workspace.
        assert!(
            set(
                &stores,
                "clangd",
                "args",
                "--log=error",
                IntegrationConfigScope::Workspace
            )
            .unwrap()
        );
        let lido = get(&stores, "clangd").unwrap();
        assert_eq!(lido.entries.len(), 1);
        assert_eq!(lido.entries[0].scope, IntegrationConfigScope::Workspace);

        // o MESMO par no global convive; get lista global antes de workspace.
        assert!(
            set(
                &stores,
                "clangd",
                "args",
                "--log=info",
                IntegrationConfigScope::Global
            )
            .unwrap()
        );
        let ambos = get(&stores, "clangd").unwrap();
        assert_eq!(ambos.entries.len(), 2);
        assert_eq!(ambos.entries[0].scope, IntegrationConfigScope::Global);
        assert_eq!(ambos.entries[1].scope, IntegrationConfigScope::Workspace);

        // set idempotente NAO conta como mudanca (contrato do evento).
        assert!(
            !set(
                &stores,
                "clangd",
                "args",
                "--log=error",
                IntegrationConfigScope::Workspace
            )
            .unwrap()
        );

        // reset remove SO o escopo pedido, e o segundo reset e no-op.
        assert!(reset(&stores, "clangd", "args", IntegrationConfigScope::Workspace).unwrap());
        assert!(!reset(&stores, "clangd", "args", IntegrationConfigScope::Workspace).unwrap());
        let restante = get(&stores, "clangd").unwrap();
        assert_eq!(restante.entries.len(), 1);
        assert_eq!(restante.entries[0].scope, IntegrationConfigScope::Global);
        assert_eq!(restante.entries[0].value, "--log=info");
    }

    #[test]
    fn escopo_sem_store_recusa_escrita_mas_nao_quebra_leitura() {
        let global = store("so-global");
        let stores = ConfigStores {
            global: Some(&global),
            workspace: None,
        };

        // Escrever no workspace sem workspace aberto e erro explicito...
        assert!(
            set(
                &stores,
                "clangd",
                "k",
                "v",
                IntegrationConfigScope::Workspace
            )
            .is_err()
        );
        assert!(reset(&stores, "clangd", "k", IntegrationConfigScope::Workspace).is_err());
        // ...mas a leitura so deixa de listar o escopo ausente.
        assert!(get(&stores, "clangd").unwrap().entries.is_empty());
    }
}

//! `datasource.destroy` (`0.129.0`): remover um perfil e, se pedido, os dados.
//!
//! O PLANO e' decidido aqui, sem rede e sem disco, a partir do perfil e de
//! onde ele nasceu; quem executa e' o handler.
//!
//! - `SQLite`: o arquivo (so' dentro do workspace; fora dele, recusa — a
//!   IDE nao apaga arquivo que nao e' do projeto);
//! - servidor em container (`kinein-<name>` existe no motor): `rm -f` do
//!   container, como job;
//! - banco DENTRO de um `PostgreSQL` (o perfil aponta para um banco que nao
//!   e' o de manutencao): `DROP DATABASE "<banco>"` pelo banco `postgres`
//!   do mesmo servidor, como job — nao se dropa o banco em que se esta';
//! - `MongoDB`, ou um `PostgreSQL` apontando para `postgres`: so' o perfil,
//!   com a nota do porque.

use std::path::{Path, PathBuf};

use kinein_protocol::{DataSourceEngine, DataSourceProfile};

/// O que o destroy vai fazer com `data = true`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DestroyPlan {
    /// So' o perfil.
    ProfileOnly {
        /// O que ficou e por que.
        note: String,
    },
    /// Apagar o arquivo `SQLite` (ja' confinado ao workspace).
    SqliteFile {
        /// O caminho absoluto do arquivo.
        path: PathBuf,
    },
    /// Parar e remover o container do servidor.
    Container {
        /// O nome do container (`kinein-<perfil>`).
        name: String,
    },
    /// `DROP DATABASE` pelo banco de manutencao do mesmo servidor.
    PostgresDrop {
        /// O perfil que conecta ao `postgres` do mesmo servidor.
        maintenance: DataSourceProfile,
        /// O banco a apagar.
        database: String,
    },
}

/// Decide o plano. `container_exists` diz se o motor tem `kinein-<name>`.
#[must_use]
pub fn plan(root: &Path, profile: &DataSourceProfile, container_exists: bool) -> DestroyPlan {
    match profile.engine {
        DataSourceEngine::Sqlite => {
            let path = Path::new(&profile.database);
            let absolute = if path.is_absolute() { path.to_path_buf() } else { root.join(path) };
            let inside = absolute
                .canonicalize()
                .ok()
                .zip(root.canonicalize().ok())
                .is_some_and(|(file, root)| file.starts_with(&root));
            if inside {
                DestroyPlan::SqliteFile { path: absolute }
            } else {
                DestroyPlan::ProfileOnly {
                    note: format!(
                        "{} fica: esta' fora do workspace, e a IDE nao apaga arquivo que nao e' do projeto",
                        absolute.display()
                    ),
                }
            }
        }
        _ if container_exists => DestroyPlan::Container {
            name: format!("kinein-{}", profile.name),
        },
        DataSourceEngine::Mongo => DestroyPlan::ProfileOnly {
            note: "o banco MongoDB fica no servidor: a IDE nao apaga dados de um servidor que nao subiu"
                .to_owned(),
        },
        DataSourceEngine::Postgres => {
            if profile.database.trim().is_empty() || profile.database == "postgres" {
                DestroyPlan::ProfileOnly {
                    note: "o servidor PostgreSQL fica: este perfil aponta para o banco de manutencao (`postgres`), que nao se apaga"
                        .to_owned(),
                }
            } else {
                let mut maintenance = profile.clone();
                "postgres".clone_into(&mut maintenance.database);
                DestroyPlan::PostgresDrop {
                    maintenance,
                    database: profile.database.clone(),
                }
            }
        }
    }
}

/// A instrucao do `DROP DATABASE`, com o nome entre aspas duplas (escapadas).
#[must_use]
pub fn drop_statement(database: &str) -> String {
    format!("DROP DATABASE \"{}\"", database.replace('"', "\"\""))
}

/// Apaga o arquivo `SQLite` e os companheiros `-wal`/`-shm`, se houver.
///
/// # Errors
/// A falha de disco, com o caminho.
pub fn delete_sqlite_file(path: &Path) -> Result<(), String> {
    std::fs::remove_file(path)
        .map_err(|e| format!("nao foi possivel apagar {}: {e}", path.display()))?;
    for suffix in ["-wal", "-shm"] {
        let companion = PathBuf::from(format!("{}{suffix}", path.display()));
        if companion.exists() {
            drop(std::fs::remove_file(&companion));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use kinein_protocol::SecretSource;

    fn profile(engine: DataSourceEngine, name: &str, database: &str) -> DataSourceProfile {
        DataSourceProfile {
            name: name.to_owned(),
            engine,
            host: "127.0.0.1".to_owned(),
            port: 5432,
            database: database.to_owned(),
            user: "postgres".to_owned(),
            secret_source: SecretSource::Automatic,
            secret_variable: None,
            sample_size: None,
            tls: None,
            ca_file: None,
        }
    }

    #[test]
    fn sqlite_inside_the_workspace_is_deleted_and_outside_is_kept() {
        let dir = std::env::temp_dir().join(format!("kinein-destroy-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("data")).unwrap();
        std::fs::write(dir.join("data/a.sqlite"), b"x").unwrap();
        let inside = profile(
            DataSourceEngine::Sqlite,
            "a",
            &dir.join("data/a.sqlite").display().to_string(),
        );
        assert!(matches!(
            plan(&dir, &inside, false),
            DestroyPlan::SqliteFile { .. }
        ));
        let fora = std::env::temp_dir().join("fora.sqlite");
        std::fs::write(&fora, b"x").unwrap();
        let outside = profile(DataSourceEngine::Sqlite, "b", &fora.display().to_string());
        assert!(matches!(
            plan(&dir, &outside, false),
            DestroyPlan::ProfileOnly { .. }
        ));
        let _ = std::fs::remove_file(&fora);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn servers_container_drop_and_notes() {
        let root = std::env::temp_dir();
        let pg = profile(DataSourceEngine::Postgres, "dev", "postgres");
        assert!(
            matches!(plan(&root, &pg, true), DestroyPlan::Container { ref name } if name == "kinein-dev")
        );
        assert!(matches!(
            plan(&root, &pg, false),
            DestroyPlan::ProfileOnly { .. }
        ));
        let loja = profile(DataSourceEngine::Postgres, "dev-loja", "loja");
        match plan(&root, &loja, false) {
            DestroyPlan::PostgresDrop {
                maintenance,
                database,
            } => {
                assert_eq!(maintenance.database, "postgres");
                assert_eq!(database, "loja");
            }
            other => panic!("{other:?}"),
        }
        let mongo = profile(DataSourceEngine::Mongo, "docs", "test");
        assert!(matches!(
            plan(&root, &mongo, false),
            DestroyPlan::ProfileOnly { .. }
        ));
        assert_eq!(drop_statement("lo\"ja"), "DROP DATABASE \"lo\"\"ja\"");
    }
}

//! `datasource.create`: um banco onde nao havia nenhum (`0.124.0`).
//!
//! Duas formas, e so' duas, porque sao as que se PROVAM:
//!
//! - um arquivo `SQLite` vazio no workspace (imediato; recusa se o caminho
//!   ja' existe — criar por cima de um banco e' perder dados);
//! - um SERVIDOR em container no loopback, para a maquina que tem Podman/
//!   Docker e nao tem servidor. E' um job: baixa imagem, entao nunca roda
//!   sem o clique, e o comando exato volta para a tela mostrar. Autenticacao:
//!   `trust` SO' no loopback (`-p 127.0.0.1:<porta>`) — o perfil nao tem
//!   senha para guardar, e a IDE nao guarda senha.
//!
//! Criar um banco DENTRO de um `PostgreSQL` que responde e' `CREATE DATABASE`
//! pelo `datasource.query` com `confirmWrite` — ja' existe, e a tela compoe.
//! No `MongoDB` o banco nasce na primeira escrita; nao ha' o que criar.

use std::{
    path::{Path, PathBuf},
    process::Command,
};

use kinein_protocol::{DataSourceEngine, DataSourceProfile, SecretSource};

use crate::container::Engine;

/// Imagens pinadas: o que a IDE sobe e' dito, nao `latest`.
const POSTGRES_IMAGE: &str = "docker.io/library/postgres:16";
const MONGO_IMAGE: &str = "docker.io/library/mongo:7";

/// Um nome de perfil/container: letras, digitos, `-` e `_`, ate' 40.
///
/// # Errors
/// A mensagem que o autor le.
pub fn validate_name(name: &str) -> Result<(), String> {
    if name.is_empty() || name.len() > 40 {
        return Err("de um nome de 1 a 40 caracteres".to_owned());
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err("o nome so' pode ter letras, digitos, `-` e `_`".to_owned());
    }
    Ok(())
}

/// Onde o arquivo nasce: `path` como veio (relativo ao workspace) ou
/// `data/<name>.sqlite`.
#[must_use]
pub fn sqlite_path(root: &Path, name: &str, path: Option<&str>) -> PathBuf {
    match path {
        Some(p) if !p.trim().is_empty() => {
            let p = Path::new(p.trim());
            if p.is_absolute() {
                p.to_path_buf()
            } else {
                root.join(p)
            }
        }
        _ => root.join("data").join(format!("{name}.sqlite")),
    }
}

/// Cria o arquivo `SQLite` vazio e devolve o perfil que o abre.
///
/// # Errors
/// Nome invalido, caminho ja' existente, ou falha de disco/`SQLite`.
pub fn sqlite_file(
    root: &Path,
    name: &str,
    path: Option<&str>,
) -> Result<DataSourceProfile, String> {
    validate_name(name)?;
    let file = sqlite_path(root, name, path);
    if file.exists() {
        return Err(format!(
            "{} ja' existe — criar por cima seria perder o que ha' nele",
            file.display()
        ));
    }
    if let Some(parent) = file.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("nao foi possivel criar {}: {e}", parent.display()))?;
    }
    // Abrir com CREATE grava o cabecalho; sem uma escrita o arquivo ficaria
    // com zero bytes e o `discover` (que le o cabecalho) nao o veria.
    let connection = rusqlite::Connection::open(&file)
        .map_err(|e| format!("SQLite recusou criar {}: {e}", file.display()))?;
    connection
        .execute_batch("PRAGMA user_version = 1;")
        .map_err(|e| format!("SQLite recusou escrever em {}: {e}", file.display()))?;
    drop(connection);
    Ok(DataSourceProfile {
        name: name.to_owned(),
        engine: DataSourceEngine::Sqlite,
        host: String::new(),
        port: 0,
        database: file.display().to_string(),
        user: String::new(),
        secret_source: SecretSource::Automatic,
        secret_variable: None,
        sample_size: None,
        tls: None,
        ca_file: None,
    })
}

/// O comando que sobe o servidor, e o perfil que o alcanca depois.
///
/// # Errors
/// Nome invalido, porta zero, ou `sqlite` (nao tem servidor).
pub fn container_server(
    engine: &Engine,
    database: DataSourceEngine,
    name: &str,
    port: u16,
) -> Result<(Command, String, DataSourceProfile), String> {
    validate_name(name)?;
    if port == 0 {
        return Err(
            "informe a porta do loopback (5432 para PostgreSQL, 27017 para MongoDB)".to_owned(),
        );
    }
    let container_name = format!("kinein-{name}");
    let (args, profile): (Vec<String>, DataSourceProfile) = match database {
        DataSourceEngine::Postgres => (
            vec![
                "run".into(),
                "-d".into(),
                "--name".into(),
                container_name,
                "-p".into(),
                format!("127.0.0.1:{port}:5432"),
                "-e".into(),
                "POSTGRES_HOST_AUTH_METHOD=trust".into(),
                POSTGRES_IMAGE.into(),
            ],
            DataSourceProfile {
                name: name.to_owned(),
                engine: DataSourceEngine::Postgres,
                host: "127.0.0.1".to_owned(),
                port,
                database: "postgres".to_owned(),
                user: "postgres".to_owned(),
                secret_source: SecretSource::Automatic,
                secret_variable: None,
                sample_size: None,
                tls: None,
                ca_file: None,
            },
        ),
        DataSourceEngine::Mongo => (
            vec![
                "run".into(),
                "-d".into(),
                "--name".into(),
                container_name,
                "-p".into(),
                format!("127.0.0.1:{port}:27017"),
                MONGO_IMAGE.into(),
            ],
            DataSourceProfile {
                name: name.to_owned(),
                engine: DataSourceEngine::Mongo,
                host: "127.0.0.1".to_owned(),
                port,
                database: "test".to_owned(),
                user: String::new(),
                secret_source: SecretSource::Automatic,
                secret_variable: None,
                sample_size: None,
                tls: None,
                ca_file: None,
            },
        ),
        DataSourceEngine::Sqlite => {
            return Err("SQLite e' um arquivo: use `sqliteFile`".to_owned());
        }
    };
    let mut command = Command::new(&engine.binary);
    command.args(&args);
    let shown = format!(
        "{} {}",
        engine
            .binary
            .file_name()
            .map_or_else(String::new, |n| n.to_string_lossy().to_string()),
        args.join(" ")
    );
    Ok((command, shown, profile))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_are_plain() {
        assert!(validate_name("app-dev_1").is_ok());
        assert!(validate_name("").is_err());
        assert!(validate_name("a b").is_err());
        assert!(validate_name("../x").is_err());
    }

    #[test]
    fn sqlite_file_is_created_once_with_a_real_header() {
        let dir = std::env::temp_dir().join(format!("kinein-create-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let profile = sqlite_file(&dir, "app", None).unwrap();
        assert_eq!(profile.engine, DataSourceEngine::Sqlite);
        assert!(profile.database.ends_with("data/app.sqlite"));
        let bytes = std::fs::read(&profile.database).unwrap();
        assert!(bytes.starts_with(b"SQLite format 3\0"));
        assert!(
            sqlite_file(&dir, "app", None)
                .unwrap_err()
                .contains("ja' existe")
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_server_command_is_pinned_and_on_loopback() {
        let engine = Engine {
            kind: kinein_protocol::ContainerEngine::Podman,
            binary: PathBuf::from("/usr/bin/podman"),
            emulated: false,
        };
        let (_, shown, profile) =
            container_server(&engine, DataSourceEngine::Postgres, "dev", 5433).unwrap();
        assert_eq!(
            shown,
            "podman run -d --name kinein-dev -p 127.0.0.1:5433:5432 -e POSTGRES_HOST_AUTH_METHOD=trust docker.io/library/postgres:16"
        );
        assert_eq!(profile.port, 5433);
        assert_eq!(profile.user, "postgres");
        let (_, shown, profile) =
            container_server(&engine, DataSourceEngine::Mongo, "docs", 27017).unwrap();
        assert!(shown.ends_with("docker.io/library/mongo:7"));
        assert_eq!(profile.database, "test");
        assert!(container_server(&engine, DataSourceEngine::Sqlite, "x", 1).is_err());
        assert!(container_server(&engine, DataSourceEngine::Postgres, "x", 0).is_err());
    }
}

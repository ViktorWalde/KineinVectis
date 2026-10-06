//! O CONSOLE de uma conexao: um arquivo do projeto aberto no editor da IDE.
//!
//! `0.149.0`, decisao do autor em 2026-10-03: "editar/criar comandos SQL no
//! proprio campo que ja' e' usado para desenvolver codigo".
//!
//! Mora em `.kinein/consoles/<conexao>.sql` (`.mongo` para `MongoDB`): o que se
//! escreveu continua ali ao reabrir o projeto, e o `.kinein/` fica fora do
//! Git. Este modulo so' garante que o arquivo existe; quem executa a
//! instrucao sob o cursor e' a UI, pelo `datasource.query` de sempre.

use std::{
    fs,
    path::{Path, PathBuf},
};

use kinein_protocol::DataSourceEngine;

/// Pasta dos consoles dentro do projeto.
#[must_use]
pub fn consoles_dir(root: &Path) -> PathBuf {
    root.join(".kinein").join("consoles")
}

/// O nome de arquivo de um perfil: so' letras, numeros, `.`, `-` e `_`. O
/// nome do perfil vem do usuario e nao pode virar caminho (`../x`).
#[must_use]
pub fn file_stem(name: &str) -> String {
    let stem: String = name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || matches!(c, '-' | '_' | '.') {
                c
            } else {
                '_'
            }
        })
        .collect();
    let stem = stem.trim_matches('.').to_owned();
    if stem.is_empty() {
        "console".to_owned()
    } else {
        stem
    }
}

fn header(name: &str, engine: DataSourceEngine) -> String {
    match engine {
        DataSourceEngine::Mongo => format!(
            "// Console da conexao {name}. Um comando por linha; Ctrl+Enter executa a linha do cursor.\n\
             // Ler: colecao.find({{\"campo\": \"valor\"}})  Escrever: insertOne, insertMany, updateOne,\n\
             // updateMany (com $set, $inc…). Apagar pede confirmacao: deleteOne, deleteMany, drop().\n\n"
        ),
        DataSourceEngine::Postgres | DataSourceEngine::Sqlite | DataSourceEngine::Odbc => format!(
            "-- Console da conexao {name}.\n-- Ctrl+Enter executa a instrucao sob o cursor (ou a selecao). Apagar pede confirmacao.\n\n"
        ),
    }
}

/// Garante o console do perfil `name`; devolve o caminho e se foi criado agora.
///
/// # Errors
/// Perfil inexistente, ou falha de disco ao criar a pasta ou o arquivo.
pub fn ensure(root: &Path, name: &str) -> Result<(PathBuf, bool), String> {
    let profile = super::list(root)
        .into_iter()
        .find(|profile| profile.name == name)
        .ok_or_else(|| format!("nao ha' perfil salvo com o nome {name}"))?;
    let extension = if profile.engine == DataSourceEngine::Mongo {
        "mongo"
    } else {
        "sql"
    };
    let dir = consoles_dir(root);
    let path = dir.join(format!("{}.{extension}", file_stem(name)));
    if path.is_file() {
        return Ok((path, false));
    }
    fs::create_dir_all(&dir)
        .map_err(|error| format!("falha criando {}: {error}", dir.display()))?;
    fs::write(&path, header(name, profile.engine))
        .map_err(|error| format!("falha criando {}: {error}", path.display()))?;
    Ok((path, true))
}

#[cfg(test)]
mod tests {
    use super::{ensure, file_stem};
    use kinein_protocol::{DataSourceEngine, DataSourceProfile, SecretSource};

    fn project(name: &str) -> std::path::PathBuf {
        let root =
            std::env::temp_dir().join(format!("kinein-console-{name}-{}", std::process::id()));
        drop(std::fs::remove_dir_all(&root));
        std::fs::create_dir_all(&root).unwrap();
        root
    }

    fn save(root: &std::path::Path, name: &str, engine: DataSourceEngine) {
        let file = engine == DataSourceEngine::Sqlite;
        let profile = DataSourceProfile {
            production: false,
            read_only: false,
            engine,
            name: name.to_owned(),
            host: if file {
                String::new()
            } else {
                "localhost".to_owned()
            },
            port: if file { 0 } else { 27017 },
            database: if file {
                root.join("loja.sqlite").display().to_string()
            } else {
                "app".to_owned()
            },
            user: String::new(),
            secret_source: SecretSource::Automatic,
            secret_variable: None,
            sample_size: None,
            tls: None,
            ca_file: None,
        };
        crate::datasource::save(root, &profile).unwrap();
    }

    #[test]
    fn the_console_is_created_once_and_kept() {
        let root = project("criar");
        save(&root, "loja", DataSourceEngine::Sqlite);
        let (path, created) = ensure(&root, "loja").unwrap();
        assert!(created);
        assert!(path.ends_with(".kinein/consoles/loja.sql"));
        assert!(
            std::fs::read_to_string(&path)
                .unwrap()
                .starts_with("-- Console da conexao loja.")
        );
        std::fs::write(&path, "select 1;\n").unwrap();
        let (again, created) = ensure(&root, "loja").unwrap();
        assert!(!created);
        assert_eq!(again, path);
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            "select 1;\n",
            "o que se escreveu fica"
        );
        drop(std::fs::remove_dir_all(&root));
    }

    #[test]
    fn mongo_gets_its_own_extension_and_unknown_profiles_are_refused() {
        let root = project("mongo");
        save(&root, "sensores", DataSourceEngine::Mongo);
        let (path, _) = ensure(&root, "sensores").unwrap();
        assert!(path.ends_with(".kinein/consoles/sensores.mongo"));
        assert!(ensure(&root, "nao-existe").is_err());
        drop(std::fs::remove_dir_all(&root));
    }

    #[test]
    fn a_profile_name_never_becomes_a_path() {
        assert_eq!(file_stem("../../etc/passwd"), "_.._etc_passwd");
        assert_eq!(file_stem("meu pg"), "meu_pg");
        assert_eq!(file_stem("..."), "console");
        // O mesmo que a UI calcula (tst_datasource_console.qml): os dois lados
        // precisam achar o MESMO arquivo para o mesmo perfil.
        assert_eq!(file_stem("../x"), "_x");
    }
}

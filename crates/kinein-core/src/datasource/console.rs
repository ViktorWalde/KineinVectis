//! O CONSOLE de uma conexao: um arquivo do projeto aberto no editor da IDE.
//!
//! `0.149.0`, decisao do autor em 2026-10-03: "editar/criar comandos SQL no
//! proprio campo que ja' e' usado para desenvolver codigo".
//!
//! Consoles novos ficam em `.kinein/consoles/v1/`, com identidade SHA-256
//! calculada aqui. Consoles anteriores só mantêm vínculo se não há colisão.
//! A criação é exclusiva, por descritores sem seguir links; texto existente
//! nunca é substituído. A instrução do buffer é resolvida no core e passa
//! depois pelo caminho normal de query/impact, com todas as políticas.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use kinein_protocol::{DataSourceConsoleBinding, DataSourceEngine, DataSourceProfile};
use sha2::{Digest, Sha256};

#[path = "console_fs.rs"]
mod storage;

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

fn extension(engine: DataSourceEngine) -> &'static str {
    if engine == DataSourceEngine::Mongo {
        "mongo"
    } else {
        "sql"
    }
}

fn new_file(profile: &DataSourceProfile) -> String {
    let label: String = profile
        .name
        .chars()
        .take(32)
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '-' | '_') {
                c
            } else {
                '_'
            }
        })
        .collect();
    let mut digest = String::with_capacity(64);
    for byte in Sha256::digest(profile.name.as_bytes()) {
        // Gravar em String é infalível; sem alocar um format por byte.
        let _ = write!(&mut digest, "{byte:02x}");
    }
    format!("{label}--{digest}.{}", extension(profile.engine))
}

fn legacy_file(profile: &DataSourceProfile, profiles: &[DataSourceProfile]) -> Option<String> {
    let file = format!("{}.{}", file_stem(&profile.name), extension(profile.engine));
    (file.len() <= 255
        && profiles
            .iter()
            .filter(|item| {
                extension(item.engine) == extension(profile.engine)
                    && file_stem(&item.name) == file_stem(&profile.name)
            })
            .count()
            == 1)
        .then_some(file)
}

/// Core-owned paths; legacy identity exists only when it names one profile.
#[must_use]
pub fn bindings(root: &Path, profiles: &[DataSourceProfile]) -> Vec<DataSourceConsoleBinding> {
    profiles
        .iter()
        .map(|profile| {
            let mut paths = vec![
                consoles_dir(root)
                    .join("v1")
                    .join(new_file(profile))
                    .display()
                    .to_string(),
            ];
            if let Some(file) = legacy_file(profile, profiles) {
                paths.push(consoles_dir(root).join(file).display().to_string());
            }
            DataSourceConsoleBinding {
                name: profile.name.clone(),
                paths,
            }
        })
        .collect()
}

fn header(name: &str, engine: DataSourceEngine) -> String {
    // JSON é uma única linha: newline, controles e aspas não escapam do comentário.
    let name = serde_json::to_string(name).unwrap_or_default();
    if engine == DataSourceEngine::Mongo {
        format!(
            "// Console da conexão {name}. Um comando por linha; Ctrl+Enter executa a linha.\n\
        // Ler: coleção.find({{}}). Escrever: insertOne, insertMany, updateOne, updateMany.\n\
        // Apagar pede confirmação: deleteOne, deleteMany, drop().\n\n"
        )
    } else {
        format!(
            "-- Console da conexão {name}.\n-- Ctrl+Enter executa a instrução sob o cursor ou a seleção. Apagar pede confirmação.\n\n"
        )
    }
}

/// Ensure an existing regular console or publish a new one without replacement.
/// # Errors
/// Unknown profile, unsafe filesystem entry or failure to write the console.
pub fn ensure(root: &Path, name: &str) -> Result<(PathBuf, bool), String> {
    let profiles = super::list(root);
    let profile = profiles
        .iter()
        .find(|profile| profile.name == name)
        .ok_or("A conexão não está salva neste projeto.")?;
    let base = storage::directory(root, false, true)?;
    let directory = storage::directory(root, true, true)?;
    let file = new_file(profile);
    let path = consoles_dir(root).join("v1").join(&file);
    if storage::inspect(&directory, &file)? {
        return Ok((path, false));
    }
    if let Some(legacy) = legacy_file(profile, &profiles)
        && storage::inspect(&base, &legacy)?
    {
        return Ok((consoles_dir(root).join(legacy), false));
    }
    let created = storage::create(&directory, &file, &header(name, profile.engine))?;
    Ok((path, created))
}

/// Verify an exact binding and a regular file without following links.
/// # Errors
/// Wrong profile/path, missing file, symlink or unsupported filesystem entry.
pub fn inspect(root: &Path, name: &str, path: &Path) -> Result<(), String> {
    let profiles = super::list(root);
    let binding = bindings(root, &profiles)
        .into_iter()
        .find(|item| item.name == name)
        .ok_or("A conexão não está salva neste projeto.")?;
    let index = binding.paths.iter().position(|item| Path::new(item) == path)
        .ok_or("Este arquivo não identifica o console da conexão. Abra o console pela árvore do Banco.")?;
    let directory = storage::directory(root, index == 0, false)?;
    let file = path
        .file_name()
        .and_then(|file| file.to_str())
        .ok_or("Caminho de console inválido.")?;
    if !storage::inspect(&directory, file)? {
        return Err("O arquivo do console não existe mais.".into());
    }
    Ok(())
}

// Os consoles so' existem no Unix (console_fs.rs); no Windows eles respondem
// "nao suportado", e estes testes nao se aplicam. Dois atributos, e nao
// `cfg(all(test, unix))`: o clippy so' reconhece modulo de teste (e libera o
// `unwrap`, clippy.toml) pelo `#[cfg(test)]` exato.
#[cfg(test)]
#[cfg(unix)]
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
            installation: None,
        };
        crate::datasource::save(root, &profile).unwrap();
    }

    #[test]
    fn the_console_is_created_once_and_kept() {
        let root = project("criar");
        save(&root, "loja", DataSourceEngine::Sqlite);
        let (path, created) = ensure(&root, "loja").unwrap();
        assert!(created);
        assert!(path.parent().unwrap().ends_with(".kinein/consoles/v1"));
        assert!(
            std::fs::read_to_string(&path)
                .unwrap()
                .starts_with("-- Console da conexão \"loja\".")
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
        assert_eq!(path.extension().unwrap(), "mongo");
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
    #[test]
    fn colliding_labels_never_share_a_console_or_overwrite_legacy_text() {
        let root = project("collision");
        save(&root, "meu pg", DataSourceEngine::Sqlite);
        let (old_path, _) = ensure(&root, "meu pg").unwrap();
        std::fs::write(&old_path, "SELECT 'texto preservado';").unwrap();
        save(&root, "meu_pg", DataSourceEngine::Sqlite);
        let (first, _) = ensure(&root, "meu pg").unwrap();
        let (second, _) = ensure(&root, "meu_pg").unwrap();
        assert_ne!(
            first, second,
            "perfis distintos não podem executar pelo mesmo arquivo"
        );
        assert_eq!(
            std::fs::read_to_string(&old_path).unwrap(),
            "SELECT 'texto preservado';"
        );
        drop(std::fs::remove_dir_all(&root));
    }

    #[test]
    fn a_symlinked_console_cannot_escape_the_workspace() {
        let root = project("symlink-file");
        let outside = project("symlink-outside");
        save(&root, "loja", DataSourceEngine::Sqlite);
        let (path, _) = ensure(&root, "loja").unwrap();
        std::fs::remove_file(&path).unwrap();
        let target = outside.join("query.sql");
        std::fs::write(&target, "SELECT 'intacto';").unwrap();
        std::os::unix::fs::symlink(&target, &path).unwrap();
        assert!(
            ensure(&root, "loja").is_err(),
            "symlink de arquivo precisa ser recusado"
        );
        assert_eq!(
            std::fs::read_to_string(&target).unwrap(),
            "SELECT 'intacto';"
        );
        drop(std::fs::remove_dir_all(&root));
        drop(std::fs::remove_dir_all(&outside));
    }

    #[test]
    fn profile_label_cannot_inject_a_command_into_the_console_header() {
        let root = project("header-newline");
        save(
            &root,
            "loja\nDELETE FROM clientes;",
            DataSourceEngine::Sqlite,
        );
        let (path, _) = ensure(&root, "loja\nDELETE FROM clientes;").unwrap();
        let text = std::fs::read_to_string(path).unwrap();
        assert!(
            text.lines()
                .all(|line| line.is_empty() || line.starts_with("--")),
            "nome não pode sair do comentário"
        );
        drop(std::fs::remove_dir_all(&root));
    }
    #[test]
    fn legacy_console_is_kept_only_while_its_identity_is_unambiguous() {
        let root = project("legacy");
        save(&root, "meu pg", DataSourceEngine::Sqlite);
        let directory = super::consoles_dir(&root);
        std::fs::create_dir_all(&directory).unwrap();
        let legacy = directory.join("meu_pg.sql");
        std::fs::write(&legacy, "SELECT 'original';").unwrap();
        assert_eq!(ensure(&root, "meu pg").unwrap(), (legacy.clone(), false));
        assert!(super::inspect(&root, "meu pg", &legacy).is_ok());
        save(&root, "meu_pg", DataSourceEngine::Sqlite);
        assert!(super::inspect(&root, "meu pg", &legacy).is_err());
        assert!(super::inspect(&root, "meu_pg", &legacy).is_err());
        let (first, created) = ensure(&root, "meu pg").unwrap();
        assert!(created);
        let (second, _) = ensure(&root, "meu_pg").unwrap();
        assert_ne!(first, second);
        assert_eq!(
            std::fs::read_to_string(legacy).unwrap(),
            "SELECT 'original';"
        );
        drop(std::fs::remove_dir_all(root));
    }

    #[test]
    fn symlinked_ancestors_and_special_files_are_refused_without_writes() {
        for level in [".kinein", "consoles", "v1"] {
            let root = project(&format!("ancestor-{level}"));
            let outside = project(&format!("external-{level}"));
            save(&root, "loja", DataSourceEngine::Sqlite);
            let (path, _) = ensure(&root, "loja").unwrap();
            let ancestor = match level {
                ".kinein" => root.join(level),
                "consoles" => root.join(".kinein").join(level),
                _ => root.join(".kinein/consoles/v1"),
            };
            let moved = outside.join("moved");
            std::fs::rename(&ancestor, &moved).unwrap();
            std::os::unix::fs::symlink(&moved, &ancestor).unwrap();
            assert!(ensure(&root, "loja").is_err(), "{level}");
            assert!(super::inspect(&root, "loja", &path).is_err(), "{level}");
            drop(std::fs::remove_dir_all(root));
            drop(std::fs::remove_dir_all(outside));
        }
        let root = project("special-file");
        save(&root, "loja", DataSourceEngine::Sqlite);
        let (path, _) = ensure(&root, "loja").unwrap();
        std::fs::remove_file(&path).unwrap();
        rustix::fs::mkfifoat(
            rustix::fs::CWD,
            &path,
            rustix::fs::Mode::RUSR | rustix::fs::Mode::WUSR,
        )
        .unwrap();
        assert!(ensure(&root, "loja").is_err());
        assert!(super::inspect(&root, "loja", &path).is_err());
        std::fs::remove_file(&path).unwrap();
        std::fs::create_dir(&path).unwrap();
        assert!(ensure(&root, "loja").is_err());
        drop(std::fs::remove_dir_all(root));
    }

    #[test]
    fn concurrent_creation_publishes_one_complete_header_without_staging_leftovers() {
        let root = project("concurrent");
        save(&root, "loja", DataSourceEngine::Sqlite);
        let barrier = std::sync::Barrier::new(8);
        let outcomes = std::thread::scope(|scope| {
            let handles: Vec<_> = (0..8)
                .map(|_| {
                    scope.spawn(|| {
                        barrier.wait();
                        let result = ensure(&root, "loja").unwrap();
                        let text = std::fs::read_to_string(&result.0).unwrap();
                        assert_eq!(text, super::header("loja", DataSourceEngine::Sqlite));
                        result
                    })
                })
                .collect();
            handles
                .into_iter()
                .map(|handle| handle.join().unwrap())
                .collect::<Vec<_>>()
        });
        assert_eq!(outcomes.iter().filter(|(_, created)| *created).count(), 1);
        let files = std::fs::read_dir(outcomes[0].0.parent().unwrap())
            .unwrap()
            .count();
        assert_eq!(files, 1, "arquivo temporário não pode ficar");
        drop(std::fs::remove_dir_all(root));
    }
}

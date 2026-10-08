//! Persistencia dos perfis, em `.kinein/datasources.json`.
//!
//! `schemaVersion` explicito: 1 (legado) e 2 (D1a.4), cujo formato mora em
//! `profile_format`. Ler nunca grava; toda gravacao escreve o schema 2, e a
//! primeira sobre um schema 1 e' a migracao, com copia e releitura. Ausencia
//! permite um catalogo novo; falha de leitura, formato invalido ou schema
//! desconhecido impedem a gravacao.
//! Catalogo quebrado nao impede abrir o projeto nem pode ser sobrescrito
//! como se estivesse vazio. Erros publicos nunca incluem seu conteudo bruto.
//!
//! **O que este arquivo NUNCA grava: senha.** A decisao esta' registrada em
//! `DocsPublic/seguranca/40-cofre-de-credencial.md` (autor, 2026-09-04), e o tipo
//! `DataSourceProfile` nao tem campo para uma — a garantia e' estrutural, nao
//! uma lembranca de quem escreve. Ha' um teste que serializa um perfil e
//! reprova se qualquer chave do JSON parecer segredo, para que a garantia
//! sobreviva a alguem acrescentar um campo sem ler este comentario.

use std::{
    fs,
    io::{self, Read},
    path::{Path, PathBuf},
};

use serde::Deserialize;

use super::profile_format::{self, CURRENT, Catalogue, LEGACY};

/// Teto de leitura e gravacao; excesso recusa o arquivo inteiro.
const MAX_BYTES: usize = 1_048_576;

/// Le somente a versao antes de interpretar um formato reconhecido.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SchemaHeader {
    schema_version: u64,
}

/// O catalogo lido e de onde ele veio.
pub(super) struct Loaded {
    /// Os perfis, usaveis e preservados.
    pub(super) catalogue: Catalogue,
    /// O schema em disco; `None` quando o arquivo nao existe.
    schema: Option<u32>,
    /// Os bytes lidos, para a copia e a restauracao da migracao.
    original: Vec<u8>,
}

/// Caminho do arquivo para um root de workspace.
#[must_use]
pub(super) fn path_for(root: &Path) -> PathBuf {
    root.join(".kinein").join("datasources.json")
}

/// Onde a migracao guarda os bytes do schema 1 (D1a.4).
#[must_use]
pub(super) fn legacy_copy_for(root: &Path) -> PathBuf {
    root.join(".kinein").join("datasources.schema1.json")
}

/// Le o catalogo; so' arquivo ausente significa nenhum perfil. Ler nunca grava.
pub(super) fn load(root: &Path) -> Result<Loaded, String> {
    let empty = || Loaded {
        catalogue: Catalogue::default(),
        schema: None,
        original: Vec::new(),
    };
    match fs::symlink_metadata(path_for(root)) {
        Ok(metadata) if !metadata.is_file() => {
            return Err(
                "O catálogo de perfis não é um arquivo regular e foi preservado sem alterações."
                    .to_owned(),
            );
        }
        Ok(_) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(empty()),
        Err(_) => {
            return Err(
                "Não foi possível acessar o catálogo de perfis. Ele foi preservado sem alterações."
                    .to_owned(),
            );
        }
    }
    let file = match fs::File::open(path_for(root)) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(empty()),
        Err(_) => return Err("Não foi possível ler os perfis. O arquivo foi preservado; verifique as permissões de .kinein/datasources.json.".to_owned()),
    };
    let mut body = Vec::new();
    file.take((MAX_BYTES + 1) as u64)
        .read_to_end(&mut body)
        .map_err(|_| "Não foi possível ler os perfis. O arquivo foi preservado.".to_owned())?;
    if body.len() > MAX_BYTES {
        return Err("O arquivo de perfis excede 1 MiB e foi preservado sem alterações.".to_owned());
    }
    let text = std::str::from_utf8(&body).map_err(|_| invalid_message())?;
    let header: SchemaHeader = serde_json::from_str(text).map_err(|_| invalid_message())?;
    // Decodificar direto do texto conserva a recusa de campos duplicados.
    // Um Value intermediario perderia a primeira ocorrencia silenciosamente.
    let (schema, catalogue) = match header.schema_version {
        version if version == u64::from(LEGACY) => (LEGACY, profile_format::parse_legacy(text)),
        version if version == u64::from(CURRENT) => (CURRENT, profile_format::parse_current(text)),
        _ => return Err("Esta versão da IDE não reconhece o formato dos perfis. O arquivo foi preservado; abra-o com uma versão compatível.".to_owned()),
    };
    let catalogue = catalogue.ok_or_else(invalid_message)?;
    Ok(Loaded {
        catalogue,
        schema: Some(schema),
        original: body,
    })
}

fn invalid_message() -> String {
    "O arquivo .kinein/datasources.json contém perfis inválidos ou opções não reconhecidas. Ele foi preservado; corrija o arquivo antes de salvar ou remover conexões.".to_owned()
}

/// Grava o catalogo no formato 2, criando `.kinein/` se preciso.
///
/// Sobre um arquivo schema 1, esta gravacao E' a migracao (39 §5.1): copia os
/// bytes originais, grava, rele e, se a releitura nao reproduzir o catalogo,
/// restaura o original.
pub(super) fn save(root: &Path, catalogue: &Catalogue) -> Result<(), String> {
    save_with(root, catalogue, profile_format::same_catalogue)
}

/// [`save`] com a verificacao da migracao injetada, para o teste da restauracao.
pub(super) fn save_with(
    root: &Path,
    catalogue: &Catalogue,
    verify: impl Fn(&Catalogue, &Catalogue) -> bool,
) -> Result<(), String> {
    // Tambem protege chamadas diretas e uma troca para formato desconhecido
    // entre a leitura no dominio e esta publicacao. O mutex e' do dominio.
    let before = load(root)?;
    let target = path_for(root);
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("falha criando {}: {error}", parent.display()))?;
    }
    let body = profile_format::serialize(catalogue)?;
    if body.len() > MAX_BYTES {
        return Err("Os perfis excedem o limite de 1 MiB. O arquivo não foi alterado.".to_owned());
    }
    if before.schema == Some(LEGACY) {
        keep_legacy_copy(root, &before.original)?;
    }
    write(&target, body.as_bytes())?;
    if before.schema != Some(LEGACY) {
        return Ok(());
    }
    let confirmed = load(root).is_ok_and(|after| verify(catalogue, &after.catalogue));
    if confirmed {
        return Ok(());
    }
    write(&target, &before.original)?;
    Err(
        "A migração dos perfis não se confirmou na releitura; o arquivo anterior foi restaurado."
            .to_owned(),
    )
}

/// A copia do schema 1 existe uma vez: igual, segue; diferente, recusa sem
/// escrever, para nao apagar a copia de uma migracao anterior.
fn keep_legacy_copy(root: &Path, original: &[u8]) -> Result<(), String> {
    let copy = legacy_copy_for(root);
    match fs::read(&copy) {
        Ok(existing) if existing == original => Ok(()),
        Ok(_) => Err("Já existe uma cópia .kinein/datasources.schema1.json diferente do catálogo atual. Mova-a antes de salvar: a migração não a sobrescreve.".to_owned()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => write(&copy, original),
        Err(_) => Err("Não foi possível conferir a cópia do catálogo anterior. Nada foi alterado.".to_owned()),
    }
}

fn write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    crate::fsops::atomic_write(path, bytes)
        .map_err(|error| format!("falha gravando perfis: {error}"))
}

#[cfg(test)]
mod tests {
    use kinein_protocol::{DataSourceEngine, DataSourceProfile, SecretSource};

    use super::*;

    const LEGACY_FIXTURE: &str = include_str!("fixtures/profiles/v1-legacy.json");
    const MIXED_FIXTURE: &str = include_str!("fixtures/profiles/v2-mixed.json");
    const FUTURE_FIXTURE: &str = include_str!("fixtures/profiles/v3-future.json");

    fn catalogue(profiles: &[DataSourceProfile]) -> Catalogue {
        Catalogue {
            profiles: profiles.to_vec(),
            preserved: Vec::new(),
        }
    }

    fn profiles(root: &Path) -> Vec<DataSourceProfile> {
        load(root).unwrap().catalogue.profiles
    }

    fn write_file(root: &Path, body: &str) {
        fs::create_dir_all(root.join(".kinein")).unwrap();
        fs::write(path_for(root), body).unwrap();
    }

    /// Mesmo molde do `toolchain/store.rs`: diretorio por PID + nome do teste,
    /// para a suite poder rodar em paralelo sem um teste pisar no outro.
    fn temp_root(name: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("kinein-datasource-store")
            .join(format!("{}-{name}", std::process::id()));
        if dir.exists() {
            fs::remove_dir_all(&dir).unwrap();
        }
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn profile(name: &str) -> DataSourceProfile {
        DataSourceProfile {
            production: false,
            read_only: false,
            engine: DataSourceEngine::Postgres,
            name: name.to_owned(),
            host: "localhost".to_owned(),
            port: 5432,
            database: "app".to_owned(),
            user: "postgres".to_owned(),
            secret_source: SecretSource::Environment,
            secret_variable: Some("PGPASSWORD".to_owned()),
            sample_size: None,
            tls: None,
            ca_file: None,
            installation: None,
        }
    }

    #[test]
    fn missing_file_is_empty_and_invalid_file_is_protected() {
        let root = temp_root("ausente");
        assert!(profiles(&root).is_empty());

        fs::create_dir_all(root.join(".kinein")).unwrap();
        fs::write(path_for(&root), "isto nao e json").unwrap();
        assert!(load(&root).is_err());
    }

    #[test]
    fn unknown_schema_is_protected() {
        let root = temp_root("schema");
        fs::create_dir_all(root.join(".kinein")).unwrap();
        fs::write(
            path_for(&root),
            r#"{"schemaVersion":999,"profiles":[{"name":"x"}]}"#,
        )
        .unwrap();
        assert!(load(&root).is_err());
    }

    #[test]
    fn grava_e_le_de_volta() {
        let root = temp_root("roundtrip");
        let saved = vec![profile("local"), profile("staging")];
        save(&root, &catalogue(&saved)).unwrap();
        assert_eq!(profiles(&root), saved);
        assert_eq!(load(&root).unwrap().schema, Some(CURRENT));
    }

    #[test]
    fn write_preserves_invalid_or_future_catalogue() {
        for (case, original) in [
            ("invalid", "isto nao e json"),
            (
                "future",
                r#"{"schemaVersion":999,"profiles":[{"name":"x"}]}"#,
            ),
            (
                "unknown-field",
                r#"{"schemaVersion":1,"profiles":[],"futureOption":true}"#,
            ),
            (
                "unknown-engine",
                r#"{"schemaVersion":1,"profiles":[{"name":"x","engine":"future"}]}"#,
            ),
            (
                "duplicate-schema",
                r#"{"schemaVersion":999,"schemaVersion":1,"profiles":[]}"#,
            ),
            (
                "duplicate-profiles",
                r#"{"schemaVersion":1,"profiles":[{"name":"x"}],"profiles":[]}"#,
            ),
            ("schema-3", FUTURE_FIXTURE),
        ] {
            let root = temp_root(case);
            fs::create_dir_all(root.join(".kinein")).unwrap();
            fs::write(path_for(&root), original).unwrap();
            assert!(
                save(&root, &catalogue(&[profile("local")])).is_err(),
                "{case}"
            );
            assert_eq!(
                fs::read_to_string(path_for(&root)).unwrap(),
                original,
                "{case}"
            );
        }
    }

    #[test]
    fn oversized_catalogue_and_public_errors_preserve_private_content() {
        let root = temp_root("oversized");
        fs::create_dir_all(root.join(".kinein")).unwrap();
        let original = "PRIVATE_DATA_NOT_FOR_UI".repeat(MAX_BYTES / 20 + 1);
        fs::write(path_for(&root), &original).unwrap();
        let message = save(&root, &catalogue(&[profile("local")])).unwrap_err();
        assert!(message.contains("1 MiB"));
        assert!(!message.contains("PRIVATE_DATA"));
        assert_eq!(fs::read_to_string(path_for(&root)).unwrap(), original);

        let original = "PRIVATE_DATA_NOT_FOR_UI";
        fs::write(path_for(&root), original).unwrap();
        assert!(!load(&root).err().unwrap().contains(original));
        assert_eq!(fs::read_to_string(path_for(&root)).unwrap(), original);
    }

    #[test]
    fn oversized_write_keeps_previous_catalogue_and_open_readers() {
        let root = temp_root("atomic");
        save(&root, &catalogue(&[profile("old")])).unwrap();
        let original = fs::read_to_string(path_for(&root)).unwrap();
        let mut reader = fs::File::open(path_for(&root)).unwrap();
        let mut large = profile("large");
        large.database = "x".repeat(MAX_BYTES);
        assert!(save(&root, &catalogue(&[large])).is_err());
        assert_eq!(fs::read_to_string(path_for(&root)).unwrap(), original);

        save(&root, &catalogue(&[profile("new")])).unwrap();
        let mut previous = String::new();
        reader.read_to_string(&mut previous).unwrap();
        assert_eq!(previous, original);
        assert_eq!(profiles(&root), vec![profile("new")]);
        assert_eq!(fs::read_dir(root.join(".kinein")).unwrap().count(), 1);
    }

    #[test]
    fn non_regular_catalogue_is_not_replaced() {
        let root = temp_root("directory");
        fs::create_dir_all(path_for(&root)).unwrap();
        assert!(load(&root).is_err());
        assert!(save(&root, &catalogue(&[profile("local")])).is_err());
        assert!(path_for(&root).is_dir());
    }

    /// A garantia central desta frente, e ela e' MECANICA de proposito.
    ///
    /// Se alguem acrescentar um campo de senha ao perfil, este teste reprova
    /// antes de a primeira gravacao chegar ao disco de alguem. E' o mesmo
    /// espirito do gate de duplicacao QML: a regra que so' vive num comentario
    /// apodrece calada.
    #[test]
    fn nada_que_pareca_segredo_vai_para_o_disco() {
        let root = temp_root("segredo");
        save(&root, &catalogue(&[profile("local")])).unwrap();
        let body = fs::read_to_string(path_for(&root)).unwrap();
        let gravado: serde_json::Value = serde_json::from_str(&body).unwrap();

        let proibidas = [
            "password",
            "passwd",
            "senha",
            "secret",
            "token",
            "credential",
        ];
        let mut pendentes = vec![&gravado];
        while let Some(valor) = pendentes.pop() {
            match valor {
                serde_json::Value::Object(mapa) => {
                    for (chave, filho) in mapa {
                        let minuscula = chave.to_lowercase();
                        for proibida in proibidas {
                            assert!(
                                !minuscula.contains(proibida)
                                    // `secretSource`/`secretVariable` dizem ONDE
                                    // procurar; nunca o que foi achado.
                                    || minuscula == "secretsource"
                                    || minuscula == "secretvariable",
                                "campo `{chave}` no JSON gravado parece segredo — \
                                 ver DocsPublic/seguranca/40"
                            );
                        }
                        pendentes.push(filho);
                    }
                }
                serde_json::Value::Array(itens) => pendentes.extend(itens),
                _ => {}
            }
        }
    }

    #[test]
    fn reading_a_legacy_file_never_writes() {
        let root = temp_root("legacy-read");
        write_file(&root, LEGACY_FIXTURE);
        let loaded = load(&root).unwrap();
        assert_eq!(loaded.schema, Some(LEGACY));
        assert_eq!(loaded.catalogue.profiles.len(), 4);
        assert_eq!(fs::read_to_string(path_for(&root)).unwrap(), LEGACY_FIXTURE);
        assert!(!legacy_copy_for(&root).exists());
    }

    #[test]
    fn the_first_write_migrates_with_an_identical_legacy_copy() {
        let root = temp_root("legacy-migrate");
        write_file(&root, LEGACY_FIXTURE);
        let mut current = load(&root).unwrap().catalogue;
        current.profiles.retain(|p| p.name != "dsn-fabrica");
        save(&root, &current).unwrap();
        assert_eq!(
            fs::read_to_string(legacy_copy_for(&root)).unwrap(),
            LEGACY_FIXTURE
        );
        let after = load(&root).unwrap();
        assert_eq!(after.schema, Some(CURRENT));
        assert!(profile_format::same_catalogue(&current, &after.catalogue));
        // A segunda gravacao ja' e' schema 2: nao toca na copia.
        save(&root, &after.catalogue).unwrap();
        assert_eq!(
            fs::read_to_string(legacy_copy_for(&root)).unwrap(),
            LEGACY_FIXTURE
        );
    }

    #[test]
    fn a_migration_that_does_not_read_back_restores_the_legacy_file() {
        let root = temp_root("legacy-restore");
        write_file(&root, LEGACY_FIXTURE);
        let current = load(&root).unwrap().catalogue;
        let message = save_with(&root, &current, |_, _| false).unwrap_err();
        assert!(message.contains("restaurado"), "{message}");
        assert_eq!(fs::read_to_string(path_for(&root)).unwrap(), LEGACY_FIXTURE);
        assert_eq!(load(&root).unwrap().schema, Some(LEGACY));
    }

    #[test]
    fn a_different_legacy_copy_refuses_the_migration_without_writing() {
        let root = temp_root("legacy-copy");
        write_file(&root, LEGACY_FIXTURE);
        fs::write(legacy_copy_for(&root), "COPIA_ANTERIOR").unwrap();
        let current = load(&root).unwrap().catalogue;
        assert!(save(&root, &current).unwrap_err().contains("Mova-a"));
        assert_eq!(fs::read_to_string(path_for(&root)).unwrap(), LEGACY_FIXTURE);
        assert_eq!(
            fs::read_to_string(legacy_copy_for(&root)).unwrap(),
            "COPIA_ANTERIOR"
        );
    }

    #[test]
    fn preserved_profiles_survive_a_write_of_another_profile() {
        let root = temp_root("preserved");
        write_file(&root, MIXED_FIXTURE);
        let before = load(&root).unwrap().catalogue;
        let mut current = before.clone();
        current.profiles.push(profile("novo"));
        save(&root, &current).unwrap();
        let after = load(&root).unwrap().catalogue;
        assert_eq!(after.unavailable(), before.unavailable());
        let raw = |c: &Catalogue| -> Vec<String> {
            c.preserved.iter().map(|p| p.raw.get().to_owned()).collect()
        };
        assert_eq!(raw(&after), raw(&before));
        assert!(after.profiles.iter().any(|p| p.name == "novo"));
        assert!(!legacy_copy_for(&root).exists());
    }
}

//! Persistencia dos perfis, em `.kinein/datasources.json`.
//!
//! `schemaVersion` explicito. Ausencia permite um catalogo novo; falha de
//! leitura, formato invalido ou schema desconhecido impedem a gravacao.
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

use kinein_protocol::DataSourceProfile;
use serde::{Deserialize, Serialize};

/// Versao do schema escrita por este core.
const SCHEMA_VERSION: u32 = 1;

/// Teto de leitura e gravacao; excesso recusa o arquivo inteiro.
const MAX_BYTES: usize = 1_048_576;

/// Representacao em disco.
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct DataSourceFile {
    schema_version: u32,
    #[serde(default)]
    profiles: Vec<DataSourceProfile>,
}

/// Le somente a versao antes de interpretar um formato reconhecido.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SchemaHeader {
    schema_version: u64,
}

/// Caminho do arquivo para um root de workspace.
#[must_use]
pub(super) fn path_for(root: &Path) -> PathBuf {
    root.join(".kinein").join("datasources.json")
}

/// Le os perfis; so' arquivo ausente significa nenhum perfil.
pub(super) fn load(root: &Path) -> Result<Vec<DataSourceProfile>, String> {
    match fs::symlink_metadata(path_for(root)) {
        Ok(metadata) if !metadata.is_file() => {
            return Err(
                "O catálogo de perfis não é um arquivo regular e foi preservado sem alterações."
                    .to_owned(),
            );
        }
        Ok(_) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(_) => {
            return Err(
                "Não foi possível acessar o catálogo de perfis. Ele foi preservado sem alterações."
                    .to_owned(),
            );
        }
    }
    let file = match fs::File::open(path_for(root)) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(_) => return Err("Não foi possível ler os perfis. O arquivo foi preservado; verifique as permissões de .kinein/datasources.json.".to_owned()),
    };
    let mut body = Vec::new();
    file.take((MAX_BYTES + 1) as u64)
        .read_to_end(&mut body)
        .map_err(|_| "Não foi possível ler os perfis. O arquivo foi preservado.".to_owned())?;
    if body.len() > MAX_BYTES {
        return Err("O arquivo de perfis excede 1 MiB e foi preservado sem alterações.".to_owned());
    }
    let header: SchemaHeader = serde_json::from_slice(&body).map_err(|_| invalid_message())?;
    if header.schema_version != u64::from(SCHEMA_VERSION) {
        return Err("Esta versão da IDE não reconhece o formato dos perfis. O arquivo foi preservado; abra-o com uma versão compatível.".to_owned());
    }
    // Decodificar direto dos bytes conserva a recusa de campos duplicados.
    // Um Value intermediario perderia a primeira ocorrencia silenciosamente.
    serde_json::from_slice::<DataSourceFile>(&body)
        .map(|file| file.profiles)
        .map_err(|_| invalid_message())
}

fn invalid_message() -> String {
    "O arquivo .kinein/datasources.json contém perfis inválidos ou opções não reconhecidas. Ele foi preservado; corrija o arquivo antes de salvar ou remover conexões.".to_owned()
}

/// Grava os perfis, criando `.kinein/` se preciso.
pub(super) fn save(root: &Path, profiles: &[DataSourceProfile]) -> Result<(), String> {
    // Tambem protege chamadas diretas e uma troca para formato desconhecido
    // entre a leitura no dominio e esta publicacao. O mutex e' do dominio.
    load(root)?;
    let target = path_for(root);
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("falha criando {}: {error}", parent.display()))?;
    }
    let file = DataSourceFile {
        schema_version: SCHEMA_VERSION,
        profiles: profiles.to_vec(),
    };
    let body = serde_json::to_string_pretty(&file)
        .map_err(|error| format!("falha serializando perfis: {error}"))?;
    if body.len() > MAX_BYTES {
        return Err("Os perfis excedem o limite de 1 MiB. O arquivo não foi alterado.".to_owned());
    }
    crate::fsops::atomic_write(&target, body.as_bytes())
        .map_err(|error| format!("falha gravando perfis: {error}"))
}

#[cfg(test)]
mod tests {
    use kinein_protocol::DataSourceEngine;
    use kinein_protocol::SecretSource;

    use super::*;

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
        }
    }

    #[test]
    fn missing_file_is_empty_and_invalid_file_is_protected() {
        let root = temp_root("ausente");
        assert!(load(&root).unwrap().is_empty());

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
        let profiles = vec![profile("local"), profile("staging")];
        save(&root, &profiles).unwrap();
        assert_eq!(load(&root).unwrap(), profiles);
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
        ] {
            let root = temp_root(case);
            fs::create_dir_all(root.join(".kinein")).unwrap();
            fs::write(path_for(&root), original).unwrap();
            assert!(save(&root, &[profile("local")]).is_err(), "{case}");
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
        let message = save(&root, &[profile("local")]).unwrap_err();
        assert!(message.contains("1 MiB"));
        assert!(!message.contains("PRIVATE_DATA"));
        assert_eq!(fs::read_to_string(path_for(&root)).unwrap(), original);

        let original = "PRIVATE_DATA_NOT_FOR_UI";
        fs::write(path_for(&root), original).unwrap();
        assert!(!load(&root).unwrap_err().contains(original));
        assert_eq!(fs::read_to_string(path_for(&root)).unwrap(), original);
    }

    #[test]
    fn oversized_write_keeps_previous_catalogue_and_open_readers() {
        let root = temp_root("atomic");
        save(&root, &[profile("old")]).unwrap();
        let original = fs::read_to_string(path_for(&root)).unwrap();
        let mut reader = fs::File::open(path_for(&root)).unwrap();
        let mut large = profile("large");
        large.database = "x".repeat(MAX_BYTES);
        assert!(save(&root, &[large]).is_err());
        assert_eq!(fs::read_to_string(path_for(&root)).unwrap(), original);

        save(&root, &[profile("new")]).unwrap();
        let mut previous = String::new();
        reader.read_to_string(&mut previous).unwrap();
        assert_eq!(previous, original);
        assert_eq!(load(&root).unwrap(), vec![profile("new")]);
        assert_eq!(fs::read_dir(root.join(".kinein")).unwrap().count(), 1);
    }

    #[test]
    fn non_regular_catalogue_is_not_replaced() {
        let root = temp_root("directory");
        fs::create_dir_all(path_for(&root)).unwrap();
        assert!(load(&root).is_err());
        assert!(save(&root, &[profile("local")]).is_err());
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
        save(&root, &[profile("local")]).unwrap();
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
}

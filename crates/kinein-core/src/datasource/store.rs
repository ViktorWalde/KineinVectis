//! Persistencia dos perfis, em `.kinein/datasources.json`.
//!
//! Mesmo molde do `toolchain/store.rs`, do `runconfig.rs` e do `settings.rs`, e
//! pelo mesmo motivo: `schemaVersion` explicito, e arquivo invalido ou de
//! schema desconhecido tratado como VAZIO. Catalogo de banco quebrado nao pode
//! impedir a IDE de abrir o projeto.
//!
//! **O que este arquivo NUNCA grava: senha.** A decisao esta' registrada em
//! `docs/seguranca/40-cofre-de-credencial.md` (autor, 2026-09-04), e o tipo
//! `DataSourceProfile` nao tem campo para uma — a garantia e' estrutural, nao
//! uma lembranca de quem escreve. Ha' um teste que serializa um perfil e
//! reprova se qualquer chave do JSON parecer segredo, para que a garantia
//! sobreviva a alguem acrescentar um campo sem ler este comentario.

use std::{
    fs,
    path::{Path, PathBuf},
};

use kinein_protocol::DataSourceProfile;
use serde::{Deserialize, Serialize};

/// Versao do schema escrita por este core.
const SCHEMA_VERSION: u32 = 1;

/// Representacao em disco.
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DataSourceFile {
    schema_version: u32,
    #[serde(default)]
    profiles: Vec<DataSourceProfile>,
}

/// Caminho do arquivo para um root de workspace.
#[must_use]
pub(super) fn path_for(root: &Path) -> PathBuf {
    root.join(".kinein").join("datasources.json")
}

/// Le os perfis persistidos; arquivo ausente/invalido = nenhum perfil.
#[must_use]
pub(super) fn load(root: &Path) -> Vec<DataSourceProfile> {
    let Ok(body) = fs::read_to_string(path_for(root)) else {
        return Vec::new();
    };
    match serde_json::from_str::<DataSourceFile>(&body) {
        Ok(file) if file.schema_version == SCHEMA_VERSION => file.profiles,
        _ => Vec::new(),
    }
}

/// Grava os perfis, criando `.kinein/` se preciso.
pub(super) fn save(root: &Path, profiles: &[DataSourceProfile]) -> Result<(), String> {
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
    fs::write(&target, body)
        .map_err(|error| format!("falha gravando {}: {error}", target.display()))
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

    fn perfil(name: &str) -> DataSourceProfile {
        DataSourceProfile {
            engine: DataSourceEngine::Postgres,
            name: name.to_owned(),
            host: "localhost".to_owned(),
            port: 5432,
            database: "app".to_owned(),
            user: "postgres".to_owned(),
            secret_source: SecretSource::Environment,
            secret_variable: Some("PGPASSWORD".to_owned()),
            sample_size: None,
        }
    }

    #[test]
    fn arquivo_ausente_e_invalido_viram_catalogo_vazio() {
        let root = temp_root("ausente");
        assert!(load(&root).is_empty());

        fs::create_dir_all(root.join(".kinein")).unwrap();
        fs::write(path_for(&root), "isto nao e json").unwrap();
        assert!(load(&root).is_empty());
    }

    #[test]
    fn schema_desconhecido_vira_catalogo_vazio() {
        let root = temp_root("schema");
        fs::create_dir_all(root.join(".kinein")).unwrap();
        fs::write(
            path_for(&root),
            r#"{"schemaVersion":999,"profiles":[{"name":"x"}]}"#,
        )
        .unwrap();
        assert!(load(&root).is_empty());
    }

    #[test]
    fn grava_e_le_de_volta() {
        let root = temp_root("roundtrip");
        let perfis = vec![perfil("local"), perfil("staging")];
        save(&root, &perfis).unwrap();
        assert_eq!(load(&root), perfis);
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
        save(&root, &[perfil("local")]).unwrap();
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
                                 ver docs/seguranca/40"
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

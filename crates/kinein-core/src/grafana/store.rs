//! Persistencia da instancia, em `.kinein/grafana.json`.
//!
//! Mesmo molde do `datasource/store.rs`: `schemaVersion` explicito, e arquivo
//! invalido ou de schema desconhecido tratado como AUSENTE. Um JSON quebrado
//! nao pode impedir a IDE de abrir o projeto.
//!
//! **O que este arquivo NUNCA grava: o token.** O `GrafanaProfile` nao tem
//! campo para um — a garantia e' estrutural — e ha' um teste que serializa o
//! perfil e reprova se qualquer chave do JSON parecer credencial, para que a
//! garantia sobreviva a alguem acrescentar um campo sem ler este comentario.

use std::{
    fs,
    path::{Path, PathBuf},
};

use kinein_protocol::GrafanaProfile;
use serde::{Deserialize, Serialize};

/// Versao do schema escrita por este core.
const SCHEMA_VERSION: u32 = 1;

/// Representacao em disco.
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GrafanaFile {
    schema_version: u32,
    #[serde(default)]
    profile: Option<GrafanaProfile>,
}

/// Caminho do arquivo para um root de workspace.
#[must_use]
pub(super) fn path_for(root: &Path) -> PathBuf {
    root.join(".kinein").join("grafana.json")
}

/// Le a instancia persistida; arquivo ausente/invalido = nenhuma.
#[must_use]
pub(super) fn load(root: &Path) -> Option<GrafanaProfile> {
    let body = fs::read_to_string(path_for(root)).ok()?;
    match serde_json::from_str::<GrafanaFile>(&body) {
        Ok(file) if file.schema_version == SCHEMA_VERSION => file.profile,
        _ => None,
    }
}

/// Grava a instancia, criando `.kinein/` se preciso.
pub(super) fn save(root: &Path, profile: Option<&GrafanaProfile>) -> Result<(), String> {
    let target = path_for(root);
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("falha criando {}: {error}", parent.display()))?;
    }
    let file = GrafanaFile {
        schema_version: SCHEMA_VERSION,
        profile: profile.cloned(),
    };
    let body = serde_json::to_string_pretty(&file)
        .map_err(|error| format!("falha serializando a instância: {error}"))?;
    fs::write(&target, body)
        .map_err(|error| format!("falha gravando {}: {error}", target.display()))
}

#[cfg(test)]
mod tests {
    use kinein_protocol::GrafanaTokenSource;

    use super::*;

    /// Mesmo molde do `datasource/store.rs`: diretorio por PID + nome do teste,
    /// para a suite rodar em paralelo sem um teste pisar no outro.
    fn temp_root(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("kinein-grafana-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("cria o root temporario");
        dir
    }

    #[test]
    fn ida_e_volta_preserva_a_politica() {
        let root = temp_root("ida-e-volta");
        let perfil = GrafanaProfile {
            url: "http://localhost:3000".to_owned(),
            token_source: GrafanaTokenSource::Environment,
            token_variable: Some("GRAFANA_TOKEN".to_owned()),
        };
        save(&root, Some(&perfil)).expect("grava");
        assert_eq!(load(&root), Some(perfil));
    }

    #[test]
    fn arquivo_invalido_vira_ausencia() {
        let root = temp_root("invalido");
        let alvo = path_for(&root);
        fs::create_dir_all(alvo.parent().expect("tem pai")).expect("cria .kinein");
        fs::write(&alvo, "{ nao e' json").expect("escreve lixo");
        assert_eq!(load(&root), None);
    }

    #[test]
    fn schema_desconhecido_vira_ausencia() {
        let root = temp_root("schema");
        let alvo = path_for(&root);
        fs::create_dir_all(alvo.parent().expect("tem pai")).expect("cria .kinein");
        fs::write(
            &alvo,
            r#"{"schemaVersion":99,"profile":{"url":"http://x"}}"#,
        )
        .expect("escreve versao futura");
        assert_eq!(load(&root), None);
    }

    /// A GARANTIA ESTRUTURAL, testada. Se alguem acrescentar um campo de
    /// credencial ao perfil, este teste reprova antes de o token chegar ao
    /// disco de alguem.
    #[test]
    fn nada_no_disco_parece_credencial() {
        let root = temp_root("sem-credencial");
        save(
            &root,
            Some(&GrafanaProfile {
                url: "http://localhost:3000".to_owned(),
                token_source: GrafanaTokenSource::Prompt,
                token_variable: None,
            }),
        )
        .expect("grava");
        let gravado = fs::read_to_string(path_for(&root)).expect("le");
        let baixo = gravado.to_lowercase();
        for proibido in ["\"token\"", "\"apikey\"", "\"password\"", "\"secret\""] {
            assert!(
                !baixo.contains(proibido),
                "o disco ganhou {proibido}: {gravado}"
            );
        }
        // `tokenSource` e `tokenVariable` sao POLITICA e podem ficar: eles
        // dizem ONDE procurar, nunca O QUE foi encontrado.
        assert!(baixo.contains("\"tokensource\""));
    }
}

//! Persistencia dos alvos SSH, em `.kinein/remotes.json` — o molde do
//! `datasource/store.rs`: `schemaVersion` explicito, arquivo invalido ou de
//! schema desconhecido tratado como VAZIO.
//!
//! **O que este arquivo NUNCA grava: senha.** O `RemoteTarget` nao tem campo
//! para uma (SSH aqui e' por chave; o que o `ssh` do sistema precisar
//! perguntar, pergunta no terminal da IDE), e o teste abaixo reprova
//! qualquer chave do JSON que pareca segredo — a mesma garantia estrutural
//! do `seguranca/40`.

use std::{
    fs,
    path::{Path, PathBuf},
};

use kinein_protocol::RemoteTarget;
use serde::{Deserialize, Serialize};

const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RemoteFile {
    schema_version: u32,
    #[serde(default)]
    targets: Vec<RemoteTarget>,
}

/// `.kinein/remotes.json` de um root.
#[must_use]
pub(super) fn path_for(root: &Path) -> PathBuf {
    root.join(".kinein").join("remotes.json")
}

/// Le os alvos; ausente/invalido = nenhum.
#[must_use]
pub(super) fn load(root: &Path) -> Vec<RemoteTarget> {
    let Ok(body) = fs::read_to_string(path_for(root)) else {
        return Vec::new();
    };
    match serde_json::from_str::<RemoteFile>(&body) {
        Ok(file) if file.schema_version == SCHEMA_VERSION => file.targets,
        _ => Vec::new(),
    }
}

/// Grava os alvos, criando `.kinein/`.
pub(super) fn save(root: &Path, targets: &[RemoteTarget]) -> Result<(), String> {
    let target = path_for(root);
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("falha criando {}: {error}", parent.display()))?;
    }
    let file = RemoteFile {
        schema_version: SCHEMA_VERSION,
        targets: targets.to_vec(),
    };
    let body = serde_json::to_string_pretty(&file)
        .map_err(|error| format!("falha serializando alvos: {error}"))?;
    fs::write(&target, body)
        .map_err(|error| format!("falha gravando {}: {error}", target.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_root(name: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("kinein-remote-store")
            .join(format!("{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn alvo(name: &str) -> RemoteTarget {
        RemoteTarget {
            name: name.to_owned(),
            host: "pi.local".to_owned(),
            user: Some("pi".to_owned()),
            port: Some(22),
            identity_file: Some("~/.ssh/id_ed25519".to_owned()),
            deploy_dir: None,
        }
    }

    #[test]
    fn saves_reloads_and_ignores_garbage_and_unknown_schema() {
        let root = temp_root("ida-e-volta");
        assert!(load(&root).is_empty());
        save(&root, &[alvo("pi")]).unwrap();
        assert_eq!(load(&root), vec![alvo("pi")]);
        fs::write(path_for(&root), "{ nao e json").unwrap();
        assert!(load(&root).is_empty());
        fs::write(path_for(&root), r#"{"schemaVersion": 99, "targets": []}"#).unwrap();
        assert!(load(&root).is_empty());
    }

    /// Nenhuma chave do JSON gravado pode parecer segredo: a garantia e'
    /// estrutural e este teste a mantem quando alguem acrescentar um campo.
    #[test]
    fn the_file_never_carries_a_secret_looking_key() {
        let root = temp_root("sem-segredo");
        save(&root, &[alvo("pi")]).unwrap();
        let texto = fs::read_to_string(path_for(&root)).unwrap().to_lowercase();
        for proibido in ["password", "senha", "passphrase", "secret", "token"] {
            assert!(
                !texto.contains(&format!("\"{proibido}")),
                "campo `{proibido}` no JSON gravado parece segredo — o cofre e' o do sistema"
            );
        }
    }
}

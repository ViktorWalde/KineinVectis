//! O ULTIMO CONTATO com cada alvo (`0.153.0`, 2026-10-04): o que a ultima
//! sonda achou e quando, em `.kinein/remote-contacts.json`.
//!
//! Pergunta do autor: "a visao mostraria todos os ssh conectados
//! recentemente?". Ate' aqui a sonda vivia so' na memoria da UI e so' do alvo
//! escolhido. Agora cada alvo da lista diz "respondeu ha' 3 min · `x86_64`" ou
//! "falhou ontem", e isso sobrevive a fechar a IDE. Arquivo separado do
//! `remotes.json` de proposito: aquele e' o PERFIL (o que a pessoa
//! configurou); este e' o que a IDE OBSERVOU, e se perde sem prejuizo.

use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use kinein_protocol::{RemoteContact, RemoteFailure};
use serde::{Deserialize, Serialize};

const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ContactsFile {
    schema_version: u32,
    #[serde(default)]
    contacts: Vec<RemoteContact>,
}

fn path_for(root: &Path) -> PathBuf {
    root.join(".kinein").join("remote-contacts.json")
}

/// Os contatos; ausente ou invalido = nenhum.
#[must_use]
pub fn load(root: &Path) -> Vec<RemoteContact> {
    let Ok(body) = fs::read_to_string(path_for(root)) else {
        return Vec::new();
    };
    match serde_json::from_str::<ContactsFile>(&body) {
        Ok(file) if file.schema_version == SCHEMA_VERSION => file.contacts,
        _ => Vec::new(),
    }
}

fn store(root: &Path, contacts: Vec<RemoteContact>) {
    let path = path_for(root);
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let file = ContactsFile {
        schema_version: SCHEMA_VERSION,
        contacts,
    };
    // Observacao, nao perfil: falhar em gravar nao derruba a sonda.
    if let Ok(body) = serde_json::to_string_pretty(&file) {
        let _ = fs::write(path, body);
    }
}

/// Agora, em segundos Unix.
#[must_use]
pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs())
}

/// O contato que a sonda que acabou de terminar observou.
#[must_use]
pub fn observed(
    name: &str,
    ok: bool,
    arch: Option<&String>,
    kernel: Option<&String>,
    failure: Option<RemoteFailure>,
) -> RemoteContact {
    RemoteContact {
        name: name.to_owned(),
        ok,
        at: now(),
        arch: arch.cloned(),
        kernel: kernel.cloned(),
        failure,
    }
}

/// Guarda o contato, no lugar do anterior do mesmo alvo.
pub fn record(root: &Path, contact: RemoteContact) {
    let mut contacts = load(root);
    contacts.retain(|known| known.name != contact.name);
    contacts.push(contact);
    contacts.sort_by(|a, b| a.name.cmp(&b.name));
    store(root, contacts);
}

/// Esquece o alvo removido.
pub fn forget(root: &Path, name: &str) {
    let mut contacts = load(root);
    let before = contacts.len();
    contacts.retain(|known| known.name != name);
    if contacts.len() != before {
        store(root, contacts);
    }
}

#[cfg(test)]
mod tests {
    use kinein_protocol::{RemoteContact, RemoteFailure};

    use super::{forget, load, record};

    fn contact(name: &str, ok: bool) -> RemoteContact {
        RemoteContact {
            name: name.to_owned(),
            ok,
            at: 1_700_000_000,
            arch: ok.then(|| "aarch64".to_owned()),
            kernel: None,
            failure: (!ok).then_some(RemoteFailure::Network),
        }
    }

    #[test]
    fn the_last_contact_replaces_the_previous_and_is_forgotten_with_the_target() {
        let root = std::env::temp_dir()
            .join("kinein-remote-contacts")
            .join(std::process::id().to_string());
        let _ = std::fs::remove_dir_all(&root);
        assert!(load(&root).is_empty());
        record(&root, contact("pi", false));
        record(&root, contact("bancada", true));
        record(&root, contact("pi", true));
        let contacts = load(&root);
        assert_eq!(contacts.len(), 2);
        assert_eq!(contacts[1].name, "pi");
        assert!(contacts[1].ok);
        forget(&root, "pi");
        assert_eq!(load(&root).len(), 1);
        std::fs::remove_dir_all(&root).unwrap();
    }
}

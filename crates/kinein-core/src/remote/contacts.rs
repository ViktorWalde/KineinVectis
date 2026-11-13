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
    path::Path,
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

/// Os contatos; ausente ou invalido = nenhum.
#[must_use]
pub fn load(root: &Path) -> Vec<RemoteContact> {
    // INF-15: pelo guarda do `.kinein`, que nao segue o link nem prende num
    // FIFO; o que ele recusa vale como nenhum contato.
    let Ok(Some(body)) = crate::kinein_dir::read(root, "remote-contacts.json") else {
        return Vec::new();
    };
    match serde_json::from_slice::<ContactsFile>(&body) {
        Ok(file) if file.schema_version == SCHEMA_VERSION => file.contacts,
        _ => Vec::new(),
    }
}

fn store(root: &Path, contacts: Vec<RemoteContact>) {
    let file = ContactsFile {
        schema_version: SCHEMA_VERSION,
        contacts,
    };
    // Observacao, nao perfil: falhar em gravar nao derruba a sonda. INF-17:
    // pelo guarda do `.kinein`, que nao segue o link.
    if let Ok(body) = serde_json::to_string_pretty(&file) {
        drop(crate::kinein_dir::write(
            root,
            "remote-contacts.json",
            body.as_bytes(),
        ));
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

    /// INF-17: o contato e' gravado pelo guarda do `.kinein`. Com o `.kinein`
    /// ligado a uma pasta de fora, ou com o arquivo ligado ao que nao existe,
    /// a sonda segue sem gravar, e nada e' criado fora.
    #[test]
    fn a_contact_is_not_recorded_through_a_link() {
        use crate::kinein_dir::link;

        let base = std::env::temp_dir()
            .join("kinein-remote-contacts")
            .join(format!("{}-ligados", std::process::id()));
        drop(std::fs::remove_dir_all(&base));
        let outside = base.join("fora");
        std::fs::create_dir_all(&outside).unwrap();

        let folder = base.join("pasta-ligada");
        std::fs::create_dir_all(&folder).unwrap();
        link(&outside, &folder.join(".kinein"));
        record(&folder, contact("pi", true));
        assert_eq!(std::fs::read_dir(&outside).unwrap().count(), 0);

        let file = base.join("arquivo-ligado");
        std::fs::create_dir_all(file.join(".kinein")).unwrap();
        let placed = file.join(".kinein").join("remote-contacts.json");
        link(&outside.join("contatos"), &placed);
        record(&file, contact("pi", true));
        assert_eq!(std::fs::read_dir(&outside).unwrap().count(), 0);
        assert!(
            std::fs::symlink_metadata(&placed)
                .unwrap()
                .file_type()
                .is_symlink()
        );
    }
}

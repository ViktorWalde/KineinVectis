//! CONFIAR NO SERVIDOR na primeira conexao (`0.153.0`, 2026-10-04).
//!
//! O primeiro contato com um alvo SSH para no host key: o `ssh` em
//! `BatchMode` recusa ("Host key verification failed") e, ate' aqui, a IDE
//! tratava isso como chave recusada e mandava rodar `ssh-copy-id` no terminal,
//! onde a pessoa tinha de responder `yes` e a senha na mao. Relato do autor:
//! o Remoto visual tem de ser pratico, e nao o terminal puro.
//!
//! Agora a IDE MOSTRA a impressao digital do servidor (`ssh-keyscan` +
//! `ssh-keygen -lf`, sem logar) e, se a pessoa confia, grava exatamente essa
//! chave no `known_hosts` que o proprio `ssh` usaria (`ssh -G`). O que foi
//! visto e' o que fica: o `trustHost` escaneia de novo e so' grava as chaves
//! cujas impressoes a pessoa viu.
//!
//! Funcoes puras aqui; quem roda processo e' o `handlers/remote_trust.rs`.

use std::path::{Path, PathBuf};

use kinein_protocol::{RemoteHostKey, RemoteTarget};

/// O que o `ssh -G` diz sobre o alvo e importa para o host key.
#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct Effective {
    /// O `HostName` efetivo (o alias ja' resolvido).
    pub host_name: String,
    /// A porta efetiva.
    pub port: u16,
    /// O primeiro `UserKnownHostsFile`, ja' com o `~` expandido.
    pub known_hosts: Option<PathBuf>,
    /// `HashKnownHosts yes`: a linha gravada vai com o host em hash.
    pub hash: bool,
}

/// Os argumentos do `ssh -G` para o alvo: o mesmo `-p`/`-i`/destino da sonda,
/// para o `ssh` avaliar o mesmo config que avaliaria ao conectar.
#[must_use]
pub fn effective_args(target: &RemoteTarget) -> Vec<String> {
    let mut args = vec!["-G".to_owned()];
    args.extend(super::ssh_args(target, false));
    args
}

/// Le' o `ssh -G`. `home` expande o `~` do `UserKnownHostsFile`.
#[must_use]
pub fn parse_effective(text: &str, home: &Path) -> Effective {
    let mut effective = Effective {
        port: 22,
        ..Effective::default()
    };
    for line in text.lines() {
        let Some((key, value)) = line.trim().split_once(' ') else {
            continue;
        };
        let value = value.trim();
        match key.to_ascii_lowercase().as_str() {
            "hostname" => value.clone_into(&mut effective.host_name),
            "port" => effective.port = value.parse().unwrap_or(22),
            "userknownhostsfile" => {
                effective.known_hosts = value.split_whitespace().next().map(|first| {
                    first
                        .strip_prefix("~/")
                        .map_or_else(|| PathBuf::from(first), |rest| home.join(rest))
                });
            }
            "hashknownhosts" => effective.hash = value.eq_ignore_ascii_case("yes"),
            _ => {}
        }
    }
    effective
}

/// Como o `known_hosts` nomeia o alvo: `host`, ou `[host]:porta` fora da 22.
#[must_use]
pub fn known_hosts_name(host: &str, port: u16) -> String {
    if port == 22 {
        host.to_owned()
    } else {
        format!("[{host}]:{port}")
    }
}

/// Os argumentos do `ssh-keyscan`: so' le' as chaves publicas que o servidor
/// oferece; nao loga e nao grava nada. `-H` quando o config pede hash.
#[must_use]
pub fn keyscan_args(effective: &Effective) -> Vec<String> {
    let mut args = vec!["-T".to_owned(), "4".to_owned()];
    if effective.hash {
        args.push("-H".to_owned());
    }
    args.push("-p".to_owned());
    args.push(effective.port.to_string());
    args.push(effective.host_name.clone());
    args
}

/// As linhas de chave do `ssh-keyscan` (os comentarios `#` sao o banner).
#[must_use]
pub fn key_lines(stdout: &str) -> Vec<String> {
    stdout
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_owned)
        .collect()
}

/// Uma linha do `ssh-keygen -lf -`: `256 SHA256:abc host (ED25519)`.
#[must_use]
pub fn parse_fingerprint(line: &str) -> Option<RemoteHostKey> {
    let mut parts = line.split_whitespace();
    let _bits = parts.next()?;
    let fingerprint = parts.next()?;
    if !fingerprint.starts_with("SHA256:") {
        return None;
    }
    let kind = line
        .rsplit_once('(')
        .and_then(|(_, rest)| rest.strip_suffix(')'))
        .unwrap_or("?");
    Some(RemoteHostKey {
        kind: kind.to_owned(),
        fingerprint: fingerprint.to_owned(),
    })
}

/// A ordem de forca: ED25519 primeiro, como o `ssh` prefere.
#[must_use]
pub fn strength(kind: &str) -> u8 {
    match kind {
        "ED25519" => 0,
        "ECDSA" => 1,
        "RSA" => 2,
        _ => 3,
    }
}

/// A chave que o `ssh-copy-id` vai copiar e se ela JA' existe. A do perfil
/// (`-i`) quando ha'; senao a primeira padrao que existe no `~/.ssh`; senao a
/// `~/.ssh/id_ed25519`, que ainda vai nascer.
#[must_use]
pub fn user_key(target: &RemoteTarget, home: &Path) -> (String, bool) {
    let expand = |path: &str| {
        path.strip_prefix("~/")
            .map_or_else(|| PathBuf::from(path), |rest| home.join(rest))
    };
    if let Some(key) = &target.identity_file {
        let private = expand(key);
        let exists =
            private.exists() || PathBuf::from(format!("{}.pub", private.display())).exists();
        return (key.clone(), exists);
    }
    for name in ["id_ed25519", "id_ecdsa", "id_rsa"] {
        if home.join(".ssh").join(format!("{name}.pub")).exists() {
            return (format!("~/.ssh/{name}"), true);
        }
    }
    ("~/.ssh/id_ed25519".to_owned(), false)
}

/// A linha de "copiar minha chave" e se ela tambem CRIA a chave: sem chave
/// nenhuma, o `ssh-copy-id` so' diria "No identities found". O `ssh-keygen`
/// pergunta a frase-senha no terminal (Enter deixa sem).
#[must_use]
pub fn copy_id_plan(target: &RemoteTarget, home: &Path) -> (String, bool) {
    let (key, exists) = user_key(target, home);
    let mut with_key = target.clone();
    with_key.identity_file = Some(key.clone());
    let copy = super::ssh_copy_id_line(&with_key);
    if exists {
        (copy, false)
    } else {
        (format!("ssh-keygen -t ed25519 -f {key} && {copy}"), true)
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use kinein_protocol::RemoteTarget;

    use super::{
        copy_id_plan, key_lines, keyscan_args, known_hosts_name, parse_effective, parse_fingerprint,
    };

    #[test]
    fn the_effective_config_gives_host_port_file_and_hash() {
        let text = "user kinein\nhostname 127.0.0.1\nport 2222\n\
                     userknownhostsfile ~/.ssh/known_hosts ~/.ssh/known_hosts2\nhashknownhosts yes\n";
        let config = parse_effective(text, Path::new("/home/u"));
        assert_eq!(config.host_name, "127.0.0.1");
        assert_eq!(config.port, 2222);
        assert_eq!(
            config.known_hosts,
            Some(PathBuf::from("/home/u/.ssh/known_hosts"))
        );
        assert!(config.hash);
        assert_eq!(
            keyscan_args(&config),
            ["-T", "4", "-H", "-p", "2222", "127.0.0.1"]
        );
    }

    #[test]
    fn known_hosts_names_the_port_only_outside_22() {
        assert_eq!(known_hosts_name("pi.local", 22), "pi.local");
        assert_eq!(known_hosts_name("127.0.0.1", 2222), "[127.0.0.1]:2222");
    }

    #[test]
    fn the_scan_keeps_key_lines_and_the_fingerprint_reads_type() {
        let output =
            "# 127.0.0.1:2222 SSH-2.0-OpenSSH_9.9\n[127.0.0.1]:2222 ssh-ed25519 AAAAC3\n\n";
        assert_eq!(key_lines(output), ["[127.0.0.1]:2222 ssh-ed25519 AAAAC3"]);
        let key = parse_fingerprint("256 SHA256:Xyz+/w [127.0.0.1]:2222 (ED25519)").unwrap();
        assert_eq!(key.kind, "ED25519");
        assert_eq!(key.fingerprint, "SHA256:Xyz+/w");
        assert!(parse_fingerprint("not a key").is_none());
    }

    #[test]
    fn without_any_key_the_copy_also_creates_one() {
        let home = std::env::temp_dir()
            .join("kinein-trust-keys")
            .join(std::process::id().to_string());
        let _ = std::fs::remove_dir_all(&home);
        std::fs::create_dir_all(home.join(".ssh")).unwrap();
        let target = RemoteTarget {
            name: "pi".to_owned(),
            host: "pi.local".to_owned(),
            user: Some("pi".to_owned()),
            port: None,
            identity_file: None,
            deploy_dir: None,
            program: None,
            deploy_source: None,
        };
        let (line, creates) = copy_id_plan(&target, &home);
        assert!(creates);
        assert_eq!(
            line,
            "ssh-keygen -t ed25519 -f ~/.ssh/id_ed25519 && ssh-copy-id -i ~/.ssh/id_ed25519 pi@pi.local"
        );
        std::fs::write(home.join(".ssh").join("id_ed25519.pub"), "ssh-ed25519 AAAA").unwrap();
        let (line, creates) = copy_id_plan(&target, &home);
        assert!(!creates);
        assert_eq!(line, "ssh-copy-id -i ~/.ssh/id_ed25519 pi@pi.local");
        std::fs::remove_dir_all(&home).unwrap();
    }
}

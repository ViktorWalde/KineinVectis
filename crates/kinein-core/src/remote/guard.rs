//! O que um alvo pode levar ao `ssh`, ao `rsync` e ao shell (16.4a,
//! `roadmaps/59` §7.1.4).
//!
//! Medido em 2026-10-16: o `.kinein/remotes.json` de um projeto clonado
//! chegava sem validacao aos processos — so' o painel validava, ao salvar.
//! Um host `-oProxyCommand=…` vira opcao do `ssh` e executa comando ao
//! conectar; `;` ou `$()` no host quebram a linha que o `remote.command` arma
//! para `sh -c`; o `-e` do `rsync` e' uma string que o proprio `rsync` divide
//! por espaco, e uma chave `k -oProxyCommand=…` virava opcao. Por isso a
//! regra mora aqui e vale no SALVAR e no USAR, e toda linha de texto passa
//! cada argumento por [`shell_word`].

use kinein_protocol::RemoteTarget;

/// O alvo pode ir ao `ssh`? Erro com o campo e o motivo.
///
/// # Errors
///
/// O primeiro campo que nao pode: host, usuario, chave, pasta, programa.
pub fn usable(target: &RemoteTarget) -> Result<(), String> {
    check_host(target.host.trim())?;
    if let Some(user) = target
        .user
        .as_deref()
        .map(str::trim)
        .filter(|u| !u.is_empty())
    {
        let plain = user
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'));
        if user.starts_with('-') || !plain {
            return Err(format!(
                "o usuário `{user}` não pode ir ao ssh: só letras, dígitos, `.`, `_` e `-`, sem `-` no começo"
            ));
        }
    }
    if let Some(key) = target.identity_file.as_deref()
        && (key.contains(['\'', '"']) || key.trim_start().starts_with('-'))
    {
        return Err(format!(
            "a chave `{key}` não pode ir ao ssh: sem aspas no caminho e sem `-` no começo"
        ));
    }
    for (field, value) in [
        ("a chave", &target.identity_file),
        ("a pasta de deploy", &target.deploy_dir),
        ("o programa", &target.program),
        ("a origem do deploy", &target.deploy_source),
    ] {
        if value
            .as_deref()
            .is_some_and(|v| v.contains(char::is_control))
        {
            return Err(format!(
                "{field} tem caractere de controle (quebra de linha?)"
            ));
        }
    }
    Ok(())
}

/// Host: nome ou IPv4 (`[A-Za-z0-9._%:-]`), ou IPv6 entre colchetes; nunca
/// com `-` no comeco, que o `ssh` leria como opcao.
///
/// # Errors
///
/// O host que nao pode ir ao `ssh`, com o motivo.
pub fn check_host(host: &str) -> Result<(), String> {
    let inner = host
        .strip_prefix('[')
        .and_then(|rest| rest.strip_suffix(']'))
        .unwrap_or(host);
    let plain = !inner.is_empty()
        && inner
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '%' | ':' | '-'));
    if host.starts_with('-') || !plain {
        return Err(format!(
            "o host `{host}` não pode ir ao ssh: só nome, IPv4 ou [IPv6], sem `-` no começo"
        ));
    }
    Ok(())
}

/// Um argumento numa linha de shell POSIX: como esta' se so' tem caracteres
/// seguros, senao entre aspas simples. O `-e` do `rsync` aceita as mesmas
/// aspas simples (rsync(1), `--rsh`).
#[must_use]
pub fn shell_word(text: &str) -> String {
    let safe = !text.is_empty()
        && text
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "_@%+=:,./-".contains(c));
    if safe {
        text.to_owned()
    } else {
        super::quote(text)
    }
}

/// Um caminho para um shell (o local ou o do alvo): o `~/` do comeco fica fora
/// das aspas, para o shell o expandir; o resto passa por [`shell_word`].
#[must_use]
pub fn path_word(path: &str) -> String {
    match path.strip_prefix("~/") {
        Some(rest) if !rest.is_empty() => format!("~/{}", shell_word(rest)),
        _ if path == "~" || path == "~/" => path.to_owned(),
        _ => shell_word(path),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn target(host: &str) -> RemoteTarget {
        RemoteTarget {
            name: "pi".to_owned(),
            host: host.to_owned(),
            user: None,
            port: None,
            identity_file: None,
            deploy_dir: None,
            program: None,
            deploy_source: None,
        }
    }

    /// Os valores que tentam virar opcao ou comando sao recusados; os de
    /// verdade passam.
    #[test]
    fn hostile_values_are_refused_and_real_ones_pass() {
        for good in [
            "pi.local",
            "192.168.0.42",
            "[fe80::1%eth0]",
            "fe80::1",
            "rpi-4",
        ] {
            assert!(usable(&target(good)).is_ok(), "{good}");
        }
        for bad in [
            "-oProxyCommand=touch /tmp/x",
            "pi;touch /tmp/x",
            "$(touch /tmp/x)",
            "pi touch",
            "pi\ntouch",
            "pi`id`",
            "",
            "[]",
        ] {
            assert!(usable(&target(bad)).is_err(), "{bad:?}");
        }
        let mut user = target("pi.local");
        for bad in ["-oProxyCommand=x", "pi;id", "a b", "$(id)"] {
            user.user = Some(bad.to_owned());
            assert!(usable(&user).is_err(), "{bad:?}");
        }
        user.user = Some("pi_user.2".to_owned());
        assert!(usable(&user).is_ok());
        let mut key = target("pi.local");
        key.identity_file = Some("/home/u/My Keys/id_ed25519".to_owned());
        assert!(
            usable(&key).is_ok(),
            "espaco no caminho da chave e' legitimo"
        );
        for bad in ["-oProxyCommand=x", "k' -oX", "/k\n-oX"] {
            key.identity_file = Some(bad.to_owned());
            assert!(usable(&key).is_err(), "{bad:?}");
        }
        let mut dir = target("pi.local");
        dir.deploy_dir = Some("/opt/app\nrm -rf ~".to_owned());
        assert!(usable(&dir).is_err());
    }

    /// O seguro fica como esta'; o resto vai entre aspas, inclusive aspas.
    #[test]
    fn words_are_quoted_only_when_needed() {
        assert_eq!(shell_word("/home/u/.ssh/pi"), "/home/u/.ssh/pi");
        assert_eq!(shell_word("a b"), "'a b'");
        assert_eq!(shell_word("x;id"), "'x;id'");
        assert_eq!(shell_word("it's"), "'it'\\''s'");
        assert_eq!(shell_word(""), "''");
        assert_eq!(path_word("~/kinein/demo/app"), "~/kinein/demo/app");
        assert_eq!(path_word("~/kinein/a b; id"), "~/'kinein/a b; id'");
        assert_eq!(path_word("/opt/$(id)"), "'/opt/$(id)'");
        assert_eq!(path_word("~"), "~");
    }
}

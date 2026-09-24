//! Ler a linha `ssh` que o USUARIO colou (`0.134.0`, fatia R0.5).
//!
//! Arquivo proprio porque a responsabilidade e' outra que a do `discover.rs`:
//! la' se le' configuracao que o OpenSSH escreveu; aqui se le' uma linha que
//! uma PESSOA escreveu, possivelmente errada, possivelmente hostil.
//!
//! A ideia vem do `Remote-SSH: Add New SSH Host…` do VS Code, que aceita um
//! comando `ssh` inteiro em vez de um formulario (comparacao registrada na §3.1
//! de `especificacoes/remote-ssh-ui-hud.md`). Ela cabe aqui sem quebrar a regra
//! "a UI nao monta linha de ssh": quem fornece a linha e' o usuario, e quem a
//! interpreta e' o core.
//!
//! REGRA DE OURO: isto NAO executa nada, nunca. E' texto entrando, perfil
//! saindo. Por isso a linha e' RECUSADA — e nao "limpa" — quando traz qualquer
//! coisa que so' faria sentido para um shell, ou uma opcao que o perfil nao
//! saberia reproduzir. Aceitar e ignorar em silencio seria prometer um alvo que
//! nao se comporta como a linha que a pessoa colou.

use kinein_protocol::RemoteTarget;

/// O que um shell interpretaria, e um destino SSH nunca contem.
const METACARACTERES: [char; 10] = [';', '|', '&', '`', '>', '<', '\n', '\r', '(', ')'];

/// Le' `ssh [-p N] [-i chave] [-l usuario] [usuario@]host` e devolve o perfil
/// proposto, com a procedencia de cada campo.
///
/// Aceita tambem so' o destino (`pi@192.168.0.42`), como o VS Code faz.
///
/// # Errors
///
/// Linha vazia, com metacaractere de shell, com comando remoto no fim, com
/// opcao que o perfil nao modela, ou sem destino.
pub fn parse_ssh_command(linha: &str) -> Result<(RemoteTarget, Vec<String>), String> {
    let limpa = linha.trim().trim_start_matches("$ ").trim();
    if limpa.is_empty() {
        return Err(
            "cole a linha `ssh` que voce ja' usa (ex.: ssh -p 2222 pi@10.0.0.7)".to_owned(),
        );
    }
    if let Some(c) = limpa.chars().find(|c| METACARACTERES.contains(c)) {
        return Err(format!(
            "a linha tem `{c}`, que so' faz sentido para um shell — cole apenas o comando `ssh`"
        ));
    }
    let mut tokens = tokenizar(limpa).into_iter().peekable();
    if tokens
        .peek()
        .is_some_and(|t| t == "ssh" || t.ends_with("/ssh"))
    {
        tokens.next();
    }

    let mut fonte = Vec::new();
    let mut porta = None;
    let mut chave = None;
    let mut usuario = None;
    let mut destino = None;

    while let Some(token) = tokens.next() {
        match token.as_str() {
            "-p" => porta = Some(valor(&mut tokens, "-p", &mut fonte, "porta")?),
            "-i" => chave = Some(valor(&mut tokens, "-i", &mut fonte, "chave")?),
            "-l" => usuario = Some(valor(&mut tokens, "-l", &mut fonte, "usuario")?),
            "-o" => {
                let (campo, valor) = opcao(&mut tokens)?;
                match campo.as_str() {
                    "port" => porta = Some(valor),
                    "user" => usuario = Some(valor),
                    "identityfile" => chave = Some(valor),
                    outro => {
                        return Err(format!(
                            "o perfil nao modela `-o {outro}` — deixe essa opcao no \
                             `~/.ssh/config` e escolha o alias aqui"
                        ));
                    }
                }
                fonte.push(format!("`-o {campo}` da linha colada"));
            }
            outro if outro.starts_with('-') => {
                return Err(format!(
                    "o perfil nao modela `{outro}` — se a conexao depende dele, deixe-a \
                     no `~/.ssh/config` e escolha o alias aqui"
                ));
            }
            outro if destino.is_none() => destino = Some(outro.to_owned()),
            outro => {
                return Err(format!(
                    "`{outro}` viria depois do destino, o que faria disto um COMANDO no \
                     alvo — um perfil guarda so' como chegar"
                ));
            }
        }
    }

    let destino =
        destino.ok_or_else(|| "faltou o destino (`host` ou `usuario@host`)".to_owned())?;
    let (do_destino, host) = match destino.split_once('@') {
        Some((u, h)) => (Some(u.to_owned()), h.to_owned()),
        None => (None, destino),
    };
    if host.is_empty() {
        return Err("faltou o host depois do `@`".to_owned());
    }
    if do_destino.is_some() {
        fonte.push("usuario do `usuario@host`".to_owned());
    }
    fonte.push(format!("host `{host}` da linha colada"));

    let porta = match porta {
        Some(p) => Some(
            p.parse::<u16>()
                .ok()
                .filter(|p| *p > 0)
                .ok_or_else(|| format!("porta invalida: `{p}`"))?,
        ),
        None => None,
    };
    let alvo = RemoteTarget {
        // Nome proposto, nao decidido: a UI deixa editar antes de salvar.
        name: nome_proposto(&host),
        host,
        user: usuario.or(do_destino),
        port: porta,
        identity_file: chave,
        deploy_dir: None,
    };
    Ok((alvo, fonte))
}

/// Como a pessoa chama a maquina: o primeiro rotulo de um nome
/// (`raspberrypi.local` -> `raspberrypi`).
///
/// Num IP isso daria `192` para `192.168.0.42`, que nao e' nome de nada — por
/// isso um endereco inteiramente numerico vai inteiro.
fn nome_proposto(host: &str) -> String {
    let e_ipv4 = host.split('.').count() == 4
        && host
            .split('.')
            .all(|r| !r.is_empty() && r.chars().all(|c| c.is_ascii_digit()));
    if e_ipv4 {
        return host.to_owned();
    }
    host.split('.').next().unwrap_or(host).to_owned()
}

/// O valor de uma opcao que o pede, registrando a procedencia.
fn valor<I: Iterator<Item = String>>(
    tokens: &mut I,
    flag: &str,
    fonte: &mut Vec<String>,
    campo: &str,
) -> Result<String, String> {
    let valor = tokens
        .next()
        .ok_or_else(|| format!("`{flag}` ficou sem valor"))?;
    fonte.push(format!("{campo} do `{flag}` da linha colada"));
    Ok(valor)
}

/// `-o Chave=valor` ou `-o Chave value`, como o `ssh` aceita.
fn opcao<I: Iterator<Item = String>>(tokens: &mut I) -> Result<(String, String), String> {
    let bruto = tokens
        .next()
        .ok_or_else(|| "`-o` ficou sem opcao".to_owned())?;
    let (campo, valor) = bruto
        .split_once('=')
        .ok_or_else(|| format!("`-o {bruto}` sem `=`: escreva `-o Chave=valor`"))?;
    Ok((campo.to_ascii_lowercase(), valor.to_owned()))
}

/// Divide respeitando aspas, como um shell faria — sem executar nada.
fn tokenizar(linha: &str) -> Vec<String> {
    let mut saida = Vec::new();
    let mut atual = String::new();
    let mut aspas = None;
    for c in linha.chars() {
        match c {
            '"' | '\'' if aspas.is_none() => aspas = Some(c),
            c if aspas == Some(c) => aspas = None,
            c if c.is_whitespace() && aspas.is_none() => {
                if !atual.is_empty() {
                    saida.push(std::mem::take(&mut atual));
                }
            }
            c => atual.push(c),
        }
    }
    if !atual.is_empty() {
        saida.push(atual);
    }
    saida
}

#[cfg(test)]
mod tests {
    use super::parse_ssh_command;

    fn ok(linha: &str) -> kinein_protocol::RemoteTarget {
        parse_ssh_command(linha)
            .unwrap_or_else(|e| panic!("recusou `{linha}`: {e}"))
            .0
    }

    #[test]
    fn a_pasted_ssh_line_becomes_a_profile() {
        let alvo = ok("ssh -p 2222 -i ~/.ssh/pi pi@192.168.0.42");
        assert_eq!(alvo.host, "192.168.0.42");
        assert_eq!(alvo.user.as_deref(), Some("pi"));
        assert_eq!(alvo.port, Some(2222));
        assert_eq!(alvo.identity_file.as_deref(), Some("~/.ssh/pi"));
        // O nome e' PROPOSTO a partir do host; a UI deixa editar. Num IP o
        // primeiro rotulo (`192`) nao e' nome de nada, entao vai inteiro.
        assert_eq!(alvo.name, "192.168.0.42");

        // So' o destino, como o VS Code tambem aceita.
        let curto = ok("pi@raspberrypi.local");
        assert_eq!(curto.host, "raspberrypi.local");
        assert_eq!(curto.name, "raspberrypi", "o nome e' o primeiro rotulo");
        assert_eq!(curto.user.as_deref(), Some("pi"));
        assert_eq!(curto.port, None, "porta ausente = a do ssh, nao 22 gravado");

        // `-l` e `-o` sao as outras grafias que a pessoa pode ter usado.
        let outras = ok("ssh -l kinein -o Port=2022 bancada");
        assert_eq!(outras.user.as_deref(), Some("kinein"));
        assert_eq!(outras.port, Some(2022));

        // Um `$ ` colado junto do prompt nao e' erro da pessoa.
        assert_eq!(ok("$ ssh bancada").host, "bancada");
        // IPv6 e nome com hifen nao viram pedacos.
        assert_eq!(ok("ssh placa-de-teste.lab").name, "placa-de-teste");
        // Aspas com espaco no caminho da chave.
        assert_eq!(
            ok("ssh -i \"/home/u/minha chave\" pi@h")
                .identity_file
                .as_deref(),
            Some("/home/u/minha chave")
        );
    }

    #[test]
    fn a_line_that_a_shell_would_interpret_is_refused_not_cleaned() {
        // Limpar em silencio seria prometer um alvo que nao se comporta como a
        // linha colada; e o que vem depois do `;` nao e' assunto de um perfil.
        for hostil in [
            "ssh pi@h; rm -rf /",
            "ssh pi@h && curl x",
            "ssh $(whoami)@h",
            "ssh pi@h | tee /tmp/x",
            "ssh pi@h > /tmp/x",
            "ssh `id`@h",
        ] {
            let erro = parse_ssh_command(hostil).unwrap_err();
            assert!(erro.contains("shell"), "{hostil} -> {erro}");
        }
    }

    #[test]
    fn what_the_profile_cannot_reproduce_is_said_instead_of_dropped() {
        // ProxyJump e afins: o perfil nao os modela. Aceitar e ignorar daria um
        // alvo que nao conecta, e a pessoa nao saberia por que.
        let erro = parse_ssh_command("ssh -J bastion pi@h").unwrap_err();
        assert!(
            erro.contains("-J") && erro.contains("~/.ssh/config"),
            "{erro}"
        );
        let erro = parse_ssh_command("ssh -o ProxyCommand=nc pi@h").unwrap_err();
        assert!(erro.contains("proxycommand"), "{erro}");

        // Comando remoto no fim: isso e' um COMANDO, nao um perfil.
        let erro = parse_ssh_command("ssh pi@h uname -a").unwrap_err();
        assert!(erro.contains("COMANDO"), "{erro}");
    }

    #[test]
    fn the_incomplete_line_says_what_is_missing() {
        for (linha, esperado) in [
            ("", "cole a linha"),
            ("   ", "cole a linha"),
            ("ssh", "faltou o destino"),
            ("ssh -p", "`-p` ficou sem valor"),
            ("ssh -p abc pi@h", "porta invalida"),
            ("ssh -p 0 pi@h", "porta invalida"),
            ("ssh pi@", "faltou o host"),
            ("ssh -o Port 2022 pi@h", "sem `=`"),
        ] {
            let erro = parse_ssh_command(linha).unwrap_err();
            assert!(erro.contains(esperado), "`{linha}` -> {erro}");
        }
    }
}

//! Descoberta e explicacao do SSH que a maquina JA' tem (`0.132.0`, fatia R0.5
//! de `especificacoes/remote-ssh-ui-hud.md`).
//!
//! A lacuna que esta unidade fecha: o catalogo de `remote.*` sempre soube
//! guardar um alias em `RemoteTarget.host` e deixar user/port/identity
//! ausentes, mas ninguem DESCOBRIA os aliases nem EXPLICAVA o que o OpenSSH
//! faria com eles. Sem isso, "usar o SSH que ja' funciona" obrigava o autor a
//! redigitar usuario, porta e caminho de chave que o `~/.ssh/config` ja' diz.
//!
//! Regras que nao mudam:
//! - le' apenas configuracao LOCAL autorizada; nao varre rede nem conecta;
//! - so' `Host` concreto e' alvo selecionavel. Curinga (`*`, `?`, `!`)
//!   continua valendo na resolucao do OpenSSH, mas nao aparece como escolha;
//! - a resolucao e' `ssh -G`, que imprime a configuracao efetiva sem conectar,
//!   e devolve uma ALLOWLIST. O dump inteiro nunca sai daqui e o texto de um
//!   `ProxyCommand` — uma linha de comando arbitraria — nunca sai;
//! - limites explicitos: um `Include` mal escrito nao vira varredura de disco.

use std::path::{Path, PathBuf};

use kinein_protocol::{RemoteAlias, RemoteDiscoverResult, RemoteResolveResult};

/// Quantos arquivos de configuracao a descoberta aceita ler.
const MAX_ARQUIVOS: usize = 16;
/// Profundidade maxima de `Include` encadeado.
const MAX_NIVEL: usize = 8;
/// Tamanho maximo de um arquivo de configuracao lido.
const MAX_BYTES: u64 = 256 * 1024;
/// Quantos aliases a descoberta devolve.
const MAX_ALIASES: usize = 512;
/// Quantas chaves candidatas a resolucao devolve.
const MAX_IDENTIDADES: usize = 8;

/// Os aliases concretos do `~/.ssh/config` e dos arquivos que ele inclui.
///
/// Nunca falha: sem `~/.ssh/config`, a resposta e' uma lista vazia — "nao ha'
/// SSH configurado" e' um estado do produto, nao um erro do protocolo.
#[must_use]
pub fn aliases(home: &Path) -> RemoteDiscoverResult {
    let mut achados = Vec::new();
    let mut fontes = Vec::new();
    let mut lidos = 0usize;
    ler(
        &home.join(".ssh/config"),
        home,
        0,
        &mut lidos,
        &mut achados,
        &mut fontes,
    );
    achados.sort_by(|a, b| a.name.cmp(&b.name));
    RemoteDiscoverResult {
        aliases: achados,
        sources: fontes,
    }
}

/// Le' um arquivo e, no ponto em que aparecem, os seus `Include`.
fn ler(
    caminho: &Path,
    home: &Path,
    nivel: usize,
    lidos: &mut usize,
    achados: &mut Vec<RemoteAlias>,
    fontes: &mut Vec<String>,
) {
    if nivel > MAX_NIVEL || *lidos >= MAX_ARQUIVOS {
        return;
    }
    let Ok(meta) = std::fs::metadata(caminho) else {
        return;
    };
    if !meta.is_file() || meta.len() > MAX_BYTES {
        return;
    }
    let Ok(texto) = std::fs::read_to_string(caminho) else {
        return;
    };
    *lidos += 1;
    let rotulo = rotular(caminho, home);
    fontes.push(rotulo.clone());
    let (novos, includes) = parse_hosts(&texto, &rotulo);
    for alias in novos {
        if achados.len() >= MAX_ALIASES {
            break;
        }
        if !achados.iter().any(|a| a.name == alias.name) {
            achados.push(alias);
        }
    }
    for padrao in includes {
        for alvo in expandir(&padrao, home) {
            ler(&alvo, home, nivel + 1, lidos, achados, fontes);
        }
    }
}

/// O caminho como o usuario o reconhece: `~` no lugar da home.
fn rotular(caminho: &Path, home: &Path) -> String {
    caminho.strip_prefix(home).map_or_else(
        |_| caminho.to_string_lossy().into_owned(),
        |resto| format!("~/{}", resto.to_string_lossy()),
    )
}

/// Os `Host` concretos e os padroes de `Include` de UM texto de configuracao.
///
/// Puro de proposito: o teste do parser nao precisa de disco.
#[must_use]
pub fn parse_hosts(texto: &str, fonte: &str) -> (Vec<RemoteAlias>, Vec<String>) {
    let mut aliases = Vec::new();
    let mut includes = Vec::new();
    for linha in texto.lines() {
        let (chave, resto) = dividir(linha.split('#').next().unwrap_or("").trim());
        match chave.to_ascii_lowercase().as_str() {
            "host" => aliases.extend(padroes(resto).into_iter().filter(|p| concreto(p)).map(
                |name| RemoteAlias {
                    name,
                    source: fonte.to_owned(),
                },
            )),
            "include" => includes.extend(padroes(resto)),
            _ => {}
        }
    }
    (aliases, includes)
}

/// `Keyword valor` do `ssh_config`: o separador e' espaco ou `=`.
fn dividir(linha: &str) -> (&str, &str) {
    let fim = linha
        .find(|c: char| c.is_whitespace() || c == '=')
        .unwrap_or(linha.len());
    let (chave, resto) = linha.split_at(fim);
    (
        chave,
        resto.trim_matches(|c: char| c.is_whitespace() || c == '='),
    )
}

/// Os argumentos de `Host`/`Include`, respeitando as aspas que o `ssh_config`
/// aceita.
///
/// Dividir por espaco antes de tirar as aspas quebraria `Host "um dois"` em
/// dois aliases que nao existem. Aqui ele vira UM padrao, que o `concreto`
/// recusa por ter espaco — melhor nao oferecer do que oferecer invencao.
fn padroes(resto: &str) -> Vec<String> {
    let mut saida = Vec::new();
    let mut atual = String::new();
    let mut em_aspas = false;
    for c in resto.chars() {
        if c == '"' {
            em_aspas = !em_aspas;
        } else if c.is_whitespace() && !em_aspas {
            if !atual.is_empty() {
                saida.push(std::mem::take(&mut atual));
            }
        } else {
            atual.push(c);
        }
    }
    if !atual.is_empty() {
        saida.push(atual);
    }
    saida
}

/// Um padrao de `Host` que o usuario pode escolher como alvo.
fn concreto(padrao: &str) -> bool {
    !padrao.starts_with('!')
        && !padrao.contains('*')
        && !padrao.contains('?')
        && !padrao.contains(char::is_whitespace)
}

/// Os arquivos que um `Include` alcanca: relativo e' sob `~/.ssh`, `~/` vira a
/// home e `*` casa apenas dentro de UMA pasta — nunca uma varredura recursiva.
fn expandir(padrao: &str, home: &Path) -> Vec<PathBuf> {
    let base = padrao.strip_prefix("~/").map_or_else(
        || {
            if padrao.starts_with('/') {
                PathBuf::from(padrao)
            } else {
                home.join(".ssh").join(padrao)
            }
        },
        |resto| home.join(resto),
    );
    let Some(nome) = base.file_name().map(|n| n.to_string_lossy().into_owned()) else {
        return Vec::new();
    };
    if !nome.contains('*') {
        return vec![base];
    }
    let Some(pasta) = base.parent() else {
        return Vec::new();
    };
    let Ok(entradas) = std::fs::read_dir(pasta) else {
        return Vec::new();
    };
    let mut alvos: Vec<PathBuf> = entradas
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .is_some_and(|n| combina(&n.to_string_lossy(), &nome))
        })
        .collect();
    alvos.sort();
    alvos.truncate(MAX_ARQUIVOS);
    alvos
}

/// Casamento de `*` num nome de arquivo, sem depender de crate de glob.
fn combina(nome: &str, padrao: &str) -> bool {
    let mut resto = nome;
    let partes: Vec<&str> = padrao.split('*').collect();
    for (i, parte) in partes.iter().enumerate() {
        if parte.is_empty() {
            continue;
        }
        if i == 0 {
            let Some(r) = resto.strip_prefix(parte) else {
                return false;
            };
            resto = r;
        } else if i == partes.len() - 1 {
            return resto.ends_with(parte);
        } else if let Some(pos) = resto.find(parte) {
            resto = &resto[pos + parte.len()..];
        } else {
            return false;
        }
    }
    true
}

/// O que a UI mandou resolver — a mensagem diz o que corrigir.
///
/// # Errors
///
/// Vazio, com espaco ou `@`, com controle, longo demais, ou comecando por `-`
/// (o `ssh` leria como opcao, nao como host).
pub fn validate_host(host: &str) -> Result<(), String> {
    let host = host.trim();
    if host.is_empty() {
        return Err("informe o alias ou host a resolver".to_owned());
    }
    if host.starts_with('-') {
        return Err("o host nao pode comecar com `-`: o ssh leria como opcao".to_owned());
    }
    if host.len() > 255 {
        return Err("host longo demais para um alias de SSH".to_owned());
    }
    if host.contains(char::is_whitespace) || host.contains('@') {
        return Err("o host nao leva usuario nem espaco — o usuario tem campo proprio".to_owned());
    }
    if host.chars().any(char::is_control) {
        return Err("o host tem caractere de controle".to_owned());
    }
    Ok(())
}

/// Os argumentos de `ssh` que EXPLICAM um host sem conectar nele.
///
/// `-F` aponta o MESMO arquivo que a descoberta leu, e isso nao e' detalhe:
/// medido em 2026-09-24, o OpenSSH **nao honra `$HOME`** para achar o
/// `~/.ssh/config` — ele usa a base de senhas do sistema. Sem o `-F`, a
/// descoberta podia listar os aliases de um arquivo enquanto a resolucao
/// explicava outro, e a IDE afirmaria sobre um config que o `ssh` nao usaria.
/// Os dois contratos passam a falar do mesmo arquivo por construcao.
///
/// Sem arquivo nenhum, vai so' `-G`: `ssh -F <inexistente>` e' erro, e "nao ha'
/// config" e' um estado normal, nao uma falha.
#[must_use]
pub fn resolve_args(host: &str, config: &Path) -> Vec<String> {
    let mut args = Vec::new();
    if config.is_file() {
        args.push("-F".to_owned());
        args.push(config.to_string_lossy().into_owned());
    }
    args.push("-G".to_owned());
    args.push(host.trim().to_owned());
    args
}

/// O `~/.ssh/config` sob a home que a descoberta usa.
#[must_use]
pub fn config_path(home: &Path) -> PathBuf {
    home.join(".ssh/config")
}

/// A allowlist da saida de `ssh -G`.
///
/// Puro: o teste nao precisa de OpenSSH instalado. O que nao esta' na lista e'
/// descartado, inclusive o texto de um `ProxyCommand`.
#[must_use]
pub fn parse_resolved(host: &str, texto: &str) -> RemoteResolveResult {
    let mut fora = RemoteResolveResult {
        host: host.trim().to_owned(),
        host_name: None,
        user: None,
        port: None,
        identities: Vec::new(),
        proxy_jump: None,
        proxy_command: false,
    };
    for linha in texto.lines() {
        let (chave, valor) = dividir(linha.trim());
        if valor.is_empty() {
            continue;
        }
        let vazio = valor.eq_ignore_ascii_case("none");
        match chave.to_ascii_lowercase().as_str() {
            "hostname" => fora.host_name = Some(valor.to_owned()),
            "user" => fora.user = Some(valor.to_owned()),
            "port" => fora.port = valor.parse().ok(),
            "identityfile" => {
                if fora.identities.len() < MAX_IDENTIDADES
                    && !fora.identities.iter().any(|i| i == valor)
                {
                    fora.identities.push(valor.to_owned());
                }
            }
            "proxyjump" if !vazio => fora.proxy_jump = Some(valor.to_owned()),
            "proxycommand" if !vazio => fora.proxy_command = true,
            _ => {}
        }
    }
    fora
}

#[cfg(test)]
mod tests {
    use super::{aliases, combina, parse_hosts, parse_resolved, validate_host};

    fn home(nome: &str) -> std::path::PathBuf {
        let base = std::env::temp_dir()
            .join("kinein-core-tests")
            .join(format!("{}-discover-{nome}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(base.join(".ssh")).unwrap();
        base
    }

    #[test]
    fn only_concrete_host_patterns_become_selectable_aliases() {
        let texto = "\
# um comentario
Host pi bancada
    HostName 192.168.0.42
Host *
    ServerAliveInterval 30
Host *.lab
Host !proibido
Host=igual
Host \"entre aspas\"
Include config.d/*
";
        let (achados, includes) = parse_hosts(texto, "~/.ssh/config");
        let nomes: Vec<&str> = achados.iter().map(|a| a.name.as_str()).collect();
        // `*`, `*.lab` e `!proibido` continuam valendo para o OpenSSH, mas nao
        // sao alvos que o usuario possa escolher. `"entre aspas"` e' UM padrao
        // com espaco, recusado — nao dois aliases inventados.
        assert_eq!(nomes, ["pi", "bancada", "igual"]);
        assert!(!nomes.contains(&"entre") && !nomes.contains(&"aspas"));
        assert!(achados.iter().all(|a| a.source == "~/.ssh/config"));
        assert_eq!(includes, ["config.d/*"]);
    }

    #[test]
    fn include_is_followed_in_place_and_aliases_are_sorted_and_deduplicated() {
        let base = home("include");
        std::fs::create_dir_all(base.join(".ssh/config.d")).unwrap();
        std::fs::write(
            base.join(".ssh/config"),
            "Host zulu\nInclude config.d/*.conf\nHost alfa\n",
        )
        .unwrap();
        std::fs::write(
            base.join(".ssh/config.d/10-pi.conf"),
            "Host pi\nHost zulu\n",
        )
        .unwrap();
        // Nao casa com `*.conf`: nao deve ser lido.
        std::fs::write(base.join(".ssh/config.d/nota.txt"), "Host intruso\n").unwrap();
        let fora = aliases(&base);
        let nomes: Vec<&str> = fora.aliases.iter().map(|a| a.name.as_str()).collect();
        assert_eq!(nomes, ["alfa", "pi", "zulu"]);
        assert!(!nomes.contains(&"intruso"), "glob casou arquivo de fora");
        // A origem e' o arquivo que DECLAROU o alias, com `~` na home.
        let pi = fora.aliases.iter().find(|a| a.name == "pi").unwrap();
        assert_eq!(pi.source, "~/.ssh/config.d/10-pi.conf");
        assert_eq!(
            fora.sources,
            ["~/.ssh/config", "~/.ssh/config.d/10-pi.conf"]
        );
    }

    #[test]
    fn a_machine_without_ssh_config_discovers_nothing_instead_of_failing() {
        let fora = aliases(&home("vazia"));
        assert!(fora.aliases.is_empty() && fora.sources.is_empty());
    }

    #[test]
    fn glob_matches_inside_one_folder_only() {
        assert!(combina("10-pi.conf", "*.conf"));
        assert!(combina("pi", "*"));
        assert!(combina("a-b-c", "a*c"));
        assert!(!combina("nota.txt", "*.conf"));
        assert!(!combina("conf", "*.conf"));
    }

    #[test]
    fn host_that_ssh_would_read_as_an_option_or_carry_a_user_is_refused() {
        assert!(validate_host("pi").is_ok());
        assert!(validate_host("  pi  ").is_ok());
        for recusado in [
            "",
            "   ",
            "-oProxyCommand=id",
            "pi bancada",
            "pi@host",
            "a\nb",
        ] {
            assert!(
                validate_host(recusado).is_err(),
                "aceitou `{recusado}` como host"
            );
        }
        assert!(validate_host(&"x".repeat(256)).is_err());
    }

    #[test]
    fn resolve_points_ssh_at_the_same_file_discovery_read() {
        let base = home("resolve-args");
        let config = super::config_path(&base);
        // Sem arquivo, so' `-G`: `ssh -F <inexistente>` seria erro, e maquina
        // sem config e' estado normal.
        assert_eq!(super::resolve_args("pi", &config), ["-G", "pi"]);
        std::fs::write(&config, "Host pi\n").unwrap();
        assert_eq!(
            super::resolve_args("  pi  ", &config),
            ["-F", config.to_str().unwrap(), "-G", "pi"]
        );
    }

    #[test]
    fn resolve_keeps_the_allowlist_and_never_leaks_the_proxy_command() {
        let saida = "\
user pi
hostname 192.168.0.42
port 2222
identityfile ~/.ssh/pi
identityfile ~/.ssh/pi
identityfile ~/.ssh/id_ed25519
proxycommand /usr/bin/curl --secret TOKEN %h
localforward 9000 localhost:9000
sendenv LANG
";
        let fora = parse_resolved("pi", saida);
        assert_eq!(fora.host, "pi");
        assert_eq!(fora.user.as_deref(), Some("pi"));
        assert_eq!(fora.host_name.as_deref(), Some("192.168.0.42"));
        assert_eq!(fora.port, Some(2222));
        // Ordem preservada, repetida descartada.
        assert_eq!(fora.identities, ["~/.ssh/pi", "~/.ssh/id_ed25519"]);
        // Existe proxy; o texto do comando — com o segredo — nao sai daqui.
        assert!(fora.proxy_command && fora.proxy_jump.is_none());
        let json = serde_json::to_string(&fora).unwrap();
        assert!(!json.contains("TOKEN") && !json.contains("curl"), "{json}");
        assert!(
            !json.contains("localforward") && !json.contains("sendenv"),
            "{json}"
        );
    }

    #[test]
    fn resolve_treats_none_as_absent_and_ignores_an_unparsable_port() {
        let fora = parse_resolved(
            " pi ",
            "proxyjump none\nproxycommand none\nport nao-numero\n",
        );
        assert_eq!(fora.host, "pi");
        assert!(fora.proxy_jump.is_none() && !fora.proxy_command && fora.port.is_none());
        let com_salto = parse_resolved("pi", "proxyjump bastion\n");
        assert_eq!(com_salto.proxy_jump.as_deref(), Some("bastion"));
    }
}

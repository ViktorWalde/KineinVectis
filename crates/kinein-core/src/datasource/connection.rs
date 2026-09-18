//! Abrir (ou so' TESTAR) uma conexao a partir de um perfil.
//!
//! Este e' o unico lugar do repositorio que fala com um servidor de banco. Ele
//! recebe o perfil (que veio do disco, sem senha) e, quando ha' uma, um
//! [`Secret`] que veio da sessao — e monta a configuracao do driver.
//!
//! # O que NAO acontece aqui
//!
//! Nenhuma senha e' gravada, nem sequer registrada. Repare que as mensagens de
//! erro sao construidas a partir do erro do driver, e a `Config` do
//! `tokio-postgres` NUNCA e' formatada: um `{:?}` nela seria a maneira mais
//! curta de mandar a senha para o painel de saida.
//!
//! # TLS (0.121.0, 2026-09-18)
//!
//! Ate' 0.120.0 nao havia TLS, de proposito (a arvore auditada em 2026-09-04
//! nao tinha `rustls`; `DocsPublic/integracoes/37` §5.1). O `mongodb` trouxe o
//! `rustls` e as duas licencas que faltavam entraram no `deny.toml` por
//! decisao do autor; o `tokio-postgres-rustls` (MIT, +11 crates medidos em
//! 2026-09-18) liga o mesmo `rustls` ao `postgres`. A politica e' a do
//! perfil: `disable` (ausente) = [`postgres::NoTls`], como sempre; `require`
//! = cadeia E nome do host verificados (o `verify-full` do libpq) contra as
//! raizes publicas do `webpki-roots` ou o `caFile` do perfil. Nao existe
//! "cifra sem conferir" — o `sslmode=require` do libpq — porque ele da' a
//! sensacao de seguranca sem a garantia.

use std::error::Error;
use std::time::Duration;

use kinein_protocol::{DataSourceProfile, DataSourceTls};
use postgres::{Client, Config, NoTls};
use rustls::RootCertStore;
use rustls_pki_types::{CertificateDer, pem::PemObject};
use tokio_postgres_rustls::MakeRustlsConnect;

use super::secret::Secret;

/// Quanto esperar por uma conexao antes de desistir.
///
/// Cinco segundos e' escolha deliberada: o teste de conexao roda num job, mas
/// um host errado (ou um firewall que ENGOLE o pacote em vez de recusar)
/// bloquearia ate' o timeout do sistema, que e' de minutos. Errar o host e'
/// erro comum, e o autor precisa da resposta enquanto ainda lembra o que
/// digitou.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);

/// Monta a configuracao do driver para um perfil.
///
/// **Host que comeca com `/` vira SOCKET UNIX**, nao nome de maquina: e' assim
/// que o libpq entende `/var/run/postgresql`, e e' o caminho mais comum para um
/// servidor local com autenticacao `peer` — aquele que nao pede senha nenhuma
/// (`DocsPublic/seguranca/40` §7).
///
/// A senha so' entra se houver uma E ela nao for vazia. Mandar senha vazia nao
/// e' o mesmo que nao mandar senha: o servidor responde com uma falha de
/// autenticacao pior de entender que "faltou a senha".
#[must_use]
pub fn config_for(profile: &DataSourceProfile, secret: Option<&Secret>) -> Config {
    let mut config = Config::new();
    if profile.host.starts_with('/') {
        config.host_path(&profile.host);
    } else {
        config.host(&profile.host);
    }
    config
        .port(profile.port)
        .dbname(&profile.database)
        .user(&profile.user)
        .connect_timeout(CONNECT_TIMEOUT);
    if let Some(secret) = secret {
        if !secret.is_empty() {
            config.password(secret.expose());
        }
    }
    config
}

/// Abre a conexao pela politica de TLS do perfil.
///
/// # Errors
/// A [`ConnectionFailure`] do driver; um `caFile` ilegivel e' falha do lado
/// do cliente (sem `SQLSTATE`), com o caminho na mensagem.
pub fn connect(
    profile: &DataSourceProfile,
    secret: Option<&Secret>,
) -> Result<Client, ConnectionFailure> {
    let config = config_for(profile, secret);
    match profile.tls.unwrap_or_default() {
        DataSourceTls::Disable => config.connect(NoTls).map_err(|error| failure_from(&error)),
        DataSourceTls::Require => {
            let tls = MakeRustlsConnect::new(tls_config(profile.ca_file.as_deref())?);
            config.connect(tls).map_err(|error| failure_from(&error))
        }
    }
}

/// As raizes em que confiar: as publicas do `webpki-roots`, mais o `caFile`.
fn tls_config(ca_file: Option<&str>) -> Result<rustls::ClientConfig, ConnectionFailure> {
    let mut roots = RootCertStore {
        roots: webpki_roots::TLS_SERVER_ROOTS.to_vec(),
    };
    if let Some(caminho) = ca_file {
        let certificados = CertificateDer::pem_file_iter(caminho)
            .and_then(std::iter::Iterator::collect::<Result<Vec<_>, _>>)
            .map_err(|erro| ConnectionFailure {
                message: format!("nao li o certificado `{caminho}`: {erro}"),
                sql_state: None,
                secret_required: false,
            })?;
        let (adicionados, _) = roots.add_parsable_certificates(certificados);
        if adicionados == 0 {
            return Err(ConnectionFailure {
                message: format!("`{caminho}` nao tem nenhum certificado PEM utilizavel"),
                sql_state: None,
                secret_required: false,
            });
        }
    }
    Ok(rustls::ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth())
}

/// Conecta, pergunta a versao do servidor e desconecta.
///
/// E' a primeira acao util de um cliente de banco, e a unica que prova o
/// caminho inteiro: perfil -> politica de segredo -> driver -> servidor.
///
/// Por que a conexao nao aconteceu.
///
/// Existe em vez de uma `String` por um motivo medido: a mensagem do servidor
/// e' LOCALIZADA. Exercitando contra o `PostgreSQL` 18.6 desta maquina em
/// 2026-09-04, a falha de senha voltou como *"autenticação do tipo senha
/// falhou"* — em portugues. Decidir qualquer coisa lendo esse texto seria um
/// acoplamento ao idioma do servidor, que quebra calado na maquina do
/// proximo. O `SQLSTATE` nao muda com o idioma, e e' por ele que se decide.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectionFailure {
    /// O que mostrar ao autor, com a cadeia de causas ja' achatada.
    pub message: String,
    /// O `SQLSTATE` de cinco caracteres, quando o erro veio do SERVIDOR.
    ///
    /// Ausente quando a falha foi do lado do cliente (host inalcancavel,
    /// socket inexistente, configuracao incompleta).
    pub sql_state: Option<String>,
    /// `true` quando o caminho a seguir e' PEDIR A SENHA e tentar de novo.
    pub secret_required: bool,
}

/// Conecta, pergunta a versao do servidor e desconecta.
///
/// # Errors
/// Um [`ConnectionFailure`], com a cadeia de causas achatada ([`describe`]) e
/// o `SQLSTATE` quando o servidor respondeu.
pub fn probe_server(
    profile: &DataSourceProfile,
    secret: Option<&Secret>,
) -> Result<String, ConnectionFailure> {
    let mut client = connect(profile, secret)?;
    let linha = client
        .query_one("SELECT version()", &[])
        .map_err(|error| failure_from(&error))?;
    let versao: String = linha.try_get(0).map_err(|error| failure_from(&error))?;
    Ok(match extensao_temporal(&mut client) {
        Some(temporal) => format!("{versao} · {temporal}"),
        None => versao,
    })
}

/// A extensao de SERIE TEMPORAL instalada, quando ha' uma.
///
/// POR QUE ISTO EXISTE (2026-09-04). O autor pediu integracao nativa com bancos
/// relacionais, temporais e nao-relacionais. O `TimescaleDB` e' um caso
/// especial e bom: ele NAO e' outro banco — e' uma extensao do `PostgreSQL`,
/// fala o mesmo protocolo e usa o mesmo driver. Tudo que a IDE ja' faz vale
/// para ele sem uma linha nova.
///
/// O que faltava era DIZER. Sem isto o autor conecta num Timescale e a tela
/// responde "`PostgreSQL` 18.6", que e' verdade e esconde a metade que ele foi
/// procurar.
///
/// A consulta e' `pg_extension`, o catalogo do proprio servidor. Falha em
/// SILENCIO de proposito: um servidor sem a extensao — ou um usuario sem
/// permissao de ler o catalogo — nao pode transformar um teste de conexao
/// BEM-SUCEDIDO em erro.
fn extensao_temporal(client: &mut postgres::Client) -> Option<String> {
    const TEMPORAIS: [&str; 2] = ["timescaledb", "timescaledb_toolkit"];

    let linhas = client
        .query(
            "SELECT extname, extversion FROM pg_extension WHERE extname = ANY($1)",
            &[&TEMPORAIS.as_slice()],
        )
        .ok()?;
    let encontradas: Vec<String> = linhas
        .iter()
        .map(|linha| {
            let nome: String = linha.get(0);
            let versao: String = linha.get(1);
            format!("{nome} {versao}")
        })
        .collect();
    if encontradas.is_empty() {
        None
    } else {
        Some(encontradas.join(", "))
    }
}

/// Traduz um erro do driver na falha que a UI entende.
pub(super) fn failure_from(error: &postgres::Error) -> ConnectionFailure {
    let sql_state = error.code().map(|code| code.code().to_owned());
    let message = describe(error);
    ConnectionFailure {
        secret_required: secret_required(sql_state.as_deref(), &message),
        sql_state,
        message,
    }
}

/// `SQLSTATE` de senha invalida (`invalid_password`).
const SQLSTATE_INVALID_PASSWORD: &str = "28P01";

/// A falha pede que se PECA A SENHA e tente de novo?
///
/// Regra pura, e deliberadamente ESTREITA. Duas entradas, nesta ordem:
///
/// 1. `28P01` (`invalid_password`) — o servidor disse que a senha esta'
///    errada ou faltando. E' o unico `SQLSTATE` em que perguntar ajuda.
///    **Nao** vale `28000`: ele cobre falha de `peer`, de `ident` e papel
///    inexistente, onde pedir senha nao resolveria nada e so' atrapalharia.
/// 2. A configuracao incompleta do PROPRIO driver, que nao tem `SQLSTATE`:
///    quando o servidor pede senha e a `Config` nao tem nenhuma, o
///    `tokio-postgres` falha antes de sair da maquina.
///
/// O texto do item 2 e' do DRIVER, nao do servidor, e por isso nao e'
/// localizado — foi capturado do `postgres` 0.19.14 em 2026-09-04 e esta'
/// preso por teste. Se uma versao futura mudar essa frase, o teste reprova,
/// que e' exatamente o que se quer de um casamento por texto.
#[must_use]
fn secret_required(sql_state: Option<&str>, message: &str) -> bool {
    sql_state == Some(SQLSTATE_INVALID_PASSWORD) || message.contains("password missing")
}

/// Achata a cadeia de causas de um erro numa frase so'.
///
/// POR QUE ISTO EXISTE (2026-09-04). Exercitado contra o driver REAL, um
/// `error.to_string()` sozinho devolvia exatamente isto, para dois problemas
/// completamente diferentes:
///
/// ```text
/// porta fechada        "error connecting to server"
/// socket inexistente   "error connecting to server"
/// ```
///
/// A informacao util — `Connection refused`, `No such file or directory` —
/// mora na CAUSA, e o `Display` do `postgres::Error` nao a inclui. Sem
/// percorrer `source()`, o painel diria ao autor a MESMA frase inutil para
/// "esqueci de subir o servidor" e "o caminho do socket esta errado".
///
/// Quem mostrou isso foi a exercitacao contra ferramenta real; nenhum teste
/// com erro inventado teria mostrado.
#[must_use]
pub fn describe(error: &dyn Error) -> String {
    let mut partes = vec![error.to_string()];
    let mut causa = error.source();
    while let Some(atual) = causa {
        let texto = atual.to_string();
        // Driver que ja' embute a causa no proprio Display nao vira eco.
        if !partes.iter().any(|parte| parte == &texto) {
            partes.push(texto);
        }
        causa = atual.source();
    }
    partes.join(": ")
}

#[cfg(test)]
mod tests {
    use kinein_protocol::DataSourceEngine;
    use std::fmt;

    use kinein_protocol::SecretSource;
    use postgres::config::Host;

    use super::*;

    /// Dubles de erro com cadeia, para exercitar `describe` sem servidor.
    #[derive(Debug)]
    struct Topo;
    #[derive(Debug)]
    struct Causa;
    #[derive(Debug)]
    struct Eco;
    #[derive(Debug)]
    struct EcoCausa;

    impl fmt::Display for Topo {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("error connecting to server")
        }
    }
    impl fmt::Display for Causa {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("Connection refused (os error 111)")
        }
    }
    impl fmt::Display for Eco {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("mesma frase")
        }
    }
    impl fmt::Display for EcoCausa {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("mesma frase")
        }
    }

    impl Error for Topo {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            Some(&Causa)
        }
    }
    impl Error for Causa {}
    impl Error for Eco {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            Some(&EcoCausa)
        }
    }
    impl Error for EcoCausa {}

    fn perfil(host: &str) -> DataSourceProfile {
        DataSourceProfile {
            engine: DataSourceEngine::Postgres,
            name: "local".to_owned(),
            host: host.to_owned(),
            port: 5432,
            database: "app".to_owned(),
            user: "postgres".to_owned(),
            secret_source: SecretSource::Automatic,
            secret_variable: None,
            sample_size: None,
            tls: None,
            ca_file: None,
        }
    }

    /// O caso que a pergunta do autor em 2026-09-04 destravou: um `PostgreSQL`
    /// local por socket, sem senha nenhuma.
    #[test]
    fn host_com_barra_vira_socket_unix() {
        let config = config_for(&perfil("/var/run/postgresql"), None);
        match config.get_hosts() {
            [Host::Unix(caminho)] => assert_eq!(caminho.to_str(), Some("/var/run/postgresql")),
            outro => panic!("host de socket virou {outro:?}"),
        }
        assert!(config.get_password().is_none(), "socket nao leva senha");
    }

    #[test]
    fn host_comum_continua_tcp() {
        let config = config_for(&perfil("db.example.com"), None);
        match config.get_hosts() {
            [Host::Tcp(nome)] => assert_eq!(nome, "db.example.com"),
            outro => panic!("host TCP virou {outro:?}"),
        }
        assert_eq!(config.get_ports(), [5432]);
        assert_eq!(config.get_dbname(), Some("app"));
        assert_eq!(config.get_user(), Some("postgres"));
    }

    /// Senha VAZIA nao e' o mesmo que ausencia de senha: mandar `""` faz o
    /// servidor responder falha de autenticacao, que e' pior de entender.
    #[test]
    fn senha_vazia_nao_e_enviada() {
        let vazia = Secret::new("");
        assert!(
            config_for(&perfil("localhost"), Some(&vazia))
                .get_password()
                .is_none()
        );

        let cheia = Secret::new("hunter2");
        assert_eq!(
            config_for(&perfil("localhost"), Some(&cheia)).get_password(),
            Some(b"hunter2".as_slice())
        );
    }

    /// A cadeia de causas e' o que separa "servidor nao subiu" de "caminho do
    /// socket errado" — ver o cabecalho de [`describe`].
    /// A cadeia de causas e' o que separa "servidor nao subiu" de "caminho do
    /// socket errado" — ver o cabecalho de [`describe`].
    #[test]
    fn descricao_inclui_a_causa_e_nao_repete() {
        assert_eq!(
            describe(&Topo),
            "error connecting to server: Connection refused (os error 111)"
        );
        // Causa que so' repete o topo nao vira eco.
        assert_eq!(describe(&Eco), "mesma frase");
    }

    /// A decisao de PEDIR A SENHA, com os valores REAIS capturados do
    /// `PostgreSQL` 18.6 desta maquina em 2026-09-04.
    ///
    /// Nenhum deles foi inventado: cada linha saiu do `event.datasource.tested`
    /// exercitando o binario do core contra um servidor de verdade. Fixture
    /// inventada e' como os dois bugs do `probe.rs` sobreviveram
    /// (`DocsPublic/roadmaps/38` §3).
    #[test]
    fn so_pede_senha_quando_pedir_senha_resolve() {
        // Senha errada: o servidor respondeu 28P01. Perguntar RESOLVE.
        assert!(secret_required(
            Some("28P01"),
            "db error: FATAL: autenticação do tipo senha falhou para o usuário \"app\""
        ));

        // O servidor pediu senha e a configuracao nao tinha nenhuma. Erro do
        // DRIVER, sem SQLSTATE — e a frase e' dele, nao do servidor.
        assert!(secret_required(
            None,
            "invalid configuration: password missing"
        ));

        // `peer` falhou / papel inexistente: 28000. Perguntar NAO resolve, e
        // abrir um dialogo de senha aqui so' atrapalharia.
        assert!(!secret_required(
            Some("28000"),
            "db error: FATAL: A autenticação do tipo peer falhou para o usuário \"naoexiste\""
        ));

        // Servidor fora do ar nao e' problema de credencial.
        assert!(!secret_required(
            None,
            "error connecting to server: Connection refused (os error 111)"
        ));
    }

    /// A mensagem do servidor e' LOCALIZADA; o `SQLSTATE` nao.
    ///
    /// Este teste existe para que ninguem "simplifique" a regra casando por
    /// texto: na maquina do autor o servidor responde em portugues, e a mesma
    /// falha em ingles diria "password authentication failed".
    #[test]
    fn a_decisao_nao_depende_do_idioma_do_servidor() {
        assert!(secret_required(
            Some("28P01"),
            "password authentication failed"
        ));
        assert!(secret_required(
            Some("28P01"),
            "autenticação do tipo senha falhou"
        ));
        assert!(secret_required(Some("28P01"), ""));
    }

    /// Sem timeout, host errado atras de um firewall que engole pacote
    /// bloquearia por minutos.
    #[test]
    fn ha_timeout_de_conexao() {
        assert_eq!(
            config_for(&perfil("localhost"), None).get_connect_timeout(),
            Some(&CONNECT_TIMEOUT)
        );
    }
}

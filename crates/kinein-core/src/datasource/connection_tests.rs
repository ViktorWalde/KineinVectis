use kinein_protocol::DataSourceEngine;
use std::fmt;

use kinein_protocol::SecretSource;
use postgres::config::Host;

use super::*;

/// Dubles de erro com cadeia, para exercitar `describe` sem servidor.
#[derive(Debug)]
struct OuterError;
#[derive(Debug)]
struct InnerError;
#[derive(Debug)]
struct RepeatedError;
#[derive(Debug)]
struct RepeatedCause;

impl fmt::Display for OuterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("error connecting to server")
    }
}
impl fmt::Display for InnerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Connection refused (os error 111)")
    }
}
impl fmt::Display for RepeatedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("mesma frase")
    }
}
impl fmt::Display for RepeatedCause {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("mesma frase")
    }
}

impl Error for OuterError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&InnerError)
    }
}
impl Error for InnerError {}
impl Error for RepeatedError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&RepeatedCause)
    }
}
impl Error for RepeatedCause {}

fn profile(host: &str) -> DataSourceProfile {
    DataSourceProfile {
        production: false,
        read_only: false,
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
fn leading_slash_selects_unix_socket() {
    let config = config_for(&profile("/var/run/postgresql"), None);
    match config.get_hosts() {
        [Host::Unix(path)] => assert_eq!(path.to_str(), Some("/var/run/postgresql")),
        other_host => panic!("host de socket virou {other_host:?}"),
    }
    assert!(config.get_password().is_none(), "socket nao leva senha");
}

#[test]
fn host_name_selects_tcp() {
    let config = config_for(&profile("db.example.com"), None);
    match config.get_hosts() {
        [Host::Tcp(host_name)] => assert_eq!(host_name, "db.example.com"),
        other_host => panic!("host TCP virou {other_host:?}"),
    }
    assert_eq!(config.get_ports(), [5432]);
    assert_eq!(config.get_dbname(), Some("app"));
    assert_eq!(config.get_user(), Some("postgres"));
}

/// Senha VAZIA nao e' o mesmo que ausencia de senha: mandar `""` faz o
/// servidor responder falha de autenticacao, que e' pior de entender.
#[test]
fn empty_password_is_not_sent() {
    let empty_secret = Secret::new("");
    assert!(
        config_for(&profile("localhost"), Some(&empty_secret))
            .get_password()
            .is_none()
    );

    let present_secret = Secret::new("hunter2");
    assert_eq!(
        config_for(&profile("localhost"), Some(&present_secret)).get_password(),
        Some(b"hunter2".as_slice())
    );
}

/// A cadeia de causas e' o que separa "servidor nao subiu" de "caminho do
/// socket errado" — ver o cabecalho de [`describe`].
/// A cadeia de causas e' o que separa "servidor nao subiu" de "caminho do
/// socket errado" — ver o cabecalho de [`describe`].
#[test]
fn error_description_includes_distinct_causes() {
    assert_eq!(
        describe(&OuterError),
        "error connecting to server: Connection refused (os error 111)"
    );
    // Causa que so' repete o topo nao vira eco.
    assert_eq!(describe(&RepeatedError), "mesma frase");
}

/// A decisao de PEDIR A SENHA, com os valores REAIS capturados do
/// `PostgreSQL` 18.6 desta maquina em 2026-09-04.
///
/// Nenhum deles foi inventado: cada linha saiu do `event.datasource.tested`
/// exercitando o binario do core contra um servidor de verdade. Fixture
/// inventada e' como os dois bugs do `probe.rs` sobreviveram
/// (`DocsPublic/roadmaps/38` §3).
#[test]
fn password_prompt_depends_on_actionable_failure() {
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
fn password_prompt_ignores_server_locale() {
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
fn connection_timeout_is_bounded() {
    assert_eq!(
        config_for(&profile("localhost"), None).get_connect_timeout(),
        Some(&CONNECT_TIMEOUT)
    );
}

#[test]
fn required_tls_rejects_plain_fallback() {
    let mut profile = profile("localhost");
    profile.tls = Some(DataSourceTls::Require);
    assert_eq!(
        config_for(&profile, None).get_ssl_mode(),
        postgres::config::SslMode::Require
    );
}

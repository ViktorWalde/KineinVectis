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
//! # Sem TLS, de proposito
//!
//! A arvore auditada em 2026-09-04 (`docs/integracoes/37` §5.1) nao tem
//! `rustls` nem `openssl`, e a conexao usa [`postgres::NoTls`]. Ligar TLS traz
//! backend, cadeia de certificados e politica de verificacao — decisao propria,
//! de outra fatia. Enquanto isso, o alvo suportado e' banco local ou rede
//! confiavel, e este comentario existe para que ninguem descubra isso em
//! producao.

use std::error::Error;
use std::time::Duration;

use kinein_protocol::DataSourceProfile;
use postgres::{Config, NoTls};

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
/// (`docs/seguranca/40` §7).
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

/// Conecta, pergunta a versao do servidor e desconecta.
///
/// E' a primeira acao util de um cliente de banco, e a unica que prova o
/// caminho inteiro: perfil -> politica de segredo -> driver -> servidor.
///
/// # Errors
/// A cadeia inteira do erro do driver — ver [`describe`], e o porque de ela
/// existir.
pub fn probe_server(
    profile: &DataSourceProfile,
    secret: Option<&Secret>,
) -> Result<String, String> {
    let mut client = config_for(profile, secret)
        .connect(NoTls)
        .map_err(|error| describe(&error))?;
    let linha = client
        .query_one("SELECT version()", &[])
        .map_err(|error| describe(&error))?;
    let versao: String = linha.try_get(0).map_err(|error| describe(&error))?;
    Ok(versao)
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
            name: "local".to_owned(),
            host: host.to_owned(),
            port: 5432,
            database: "app".to_owned(),
            user: "postgres".to_owned(),
            secret_source: SecretSource::Automatic,
            secret_variable: None,
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

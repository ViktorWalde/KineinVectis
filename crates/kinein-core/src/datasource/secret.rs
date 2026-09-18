//! De onde vem a senha — e o tipo que impede ela de vazar num log.
//!
//! DECISAO REGISTRADA (autor, 2026-09-04, `DocsPublic/seguranca/40`): a IDE guarda o
//! PERFIL e nunca a senha. Este modulo e' a outra metade dessa decisao: ele diz
//! **onde procurar** a senha na hora de conectar, e da' um tipo em que ela pode
//! viver sem virar texto solto na memoria de um `Debug`.

use std::fmt;

use kinein_protocol::{DataSourceEngine, DataSourceProfile, SecretSource};

/// Uma senha em memoria, que NAO se imprime.
///
/// POR QUE UM TIPO PROPRIO. Uma `String` de senha vaza pelo caminho mais banal
/// que existe: alguem poe `?profile` ou `{:?}` num log de erro de conexao, o
/// log vai para o painel de saida, e o painel vai para um print de tela num
/// chamado. `Debug` e `Display` aqui imprimem `***`, entao esse caminho fecha
/// por construcao — nao por disciplina.
///
/// Ler o valor de verdade exige [`Secret::expose`], que e' um nome que alguem
/// nota numa revisao.
#[derive(Clone, PartialEq, Eq)]
pub struct Secret(String);

impl Secret {
    /// Embrulha um valor vindo de prompt, ambiente ou driver.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// O valor cru. Chame no ponto de uso, nunca para guardar de novo.
    #[must_use]
    pub fn expose(&self) -> &str {
        &self.0
    }

    /// `true` quando nao ha' o que enviar.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl fmt::Debug for Secret {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Secret(***)")
    }
}

impl fmt::Display for Secret {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("***")
    }
}

/// O que o core precisa fazer para obter a senha de um perfil.
///
/// Repare que `AskUser` NAO carrega valor: quem resolve o prompt e' a UI, e o
/// valor volta pela chamada de conexao, vivendo so' aquela sessao.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecretPlan {
    /// Pedir ao autor agora. A UI abre o dialogo.
    AskUser,
    /// Ler desta variavel de ambiente.
    ReadEnvironment(String),
    /// Nao mandar senha nenhuma e deixar o servidor decidir.
    ///
    /// Cobre TRES casos de uma vez, e o primeiro e' o mais comum num ambiente
    /// de desenvolvimento: socket unix com autenticacao `peer` (o usuario do
    /// SO ja' e' a identidade), `trust` numa maquina de dev, e o
    /// `~/.pgpass`/`PGPASSFILE`, que o proprio libpq le'.
    ///
    /// O modo de falha do `.pgpass` e' do `PostgreSQL` e e' BOM: permissao
    /// mais frouxa que 0600 faz o arquivo ser ignorado inteiro, em vez de lido
    /// com aviso (documentacao do `PostgreSQL` 18, `libpq-pgpass`). Falha
    /// fechada.
    DelegateToDriver,
}

/// Traduz a politica do perfil no que fazer agora.
///
/// `Environment` sem variavel nomeada CAI PARA O PROMPT em vez de falhar:
/// perfil meio configurado nao pode impedir o autor de conectar, e pedir a
/// senha e' sempre uma saida segura.
#[must_use]
pub fn plan_for(profile: &DataSourceProfile) -> SecretPlan {
    // `SQLite` nao tem autenticacao: quem abre o arquivo e' o processo, com a
    // permissao dele. Perguntar senha aqui seria um dialogo que nao resolve
    // nada — e a politica salva no perfil nao muda esse fato.
    if profile.engine == DataSourceEngine::Sqlite {
        return SecretPlan::DelegateToDriver;
    }
    match profile.secret_source {
        SecretSource::Prompt => SecretPlan::AskUser,
        SecretSource::Automatic => SecretPlan::DelegateToDriver,
        SecretSource::Environment => match profile.secret_variable.as_deref() {
            Some(nome) if !nome.trim().is_empty() => {
                SecretPlan::ReadEnvironment(nome.trim().to_owned())
            }
            _ => SecretPlan::AskUser,
        },
    }
}

/// Executa um [`SecretPlan::ReadEnvironment`] contra o ambiente do processo.
///
/// Uma linha so', de proposito: a REGRA esta' em [`secret_from`], que e' pura e
/// por isso testavel sem mexer no ambiente do processo — mutar `env` em teste
/// e' corrida garantida assim que a suite roda em paralelo.
#[must_use]
pub fn read_environment(variable: &str) -> Option<Secret> {
    secret_from(std::env::var(variable).ok().as_deref())
}

/// Valor de ambiente -> senha. Ausente ou VAZIO nao vira senha.
///
/// Vazio importa: `PGPASSWORD=` exportado sem valor e' configuracao errada
/// comum, e tentar conectar com senha vazia produz um erro do servidor bem
/// pior de entender que "nao achei a senha, digite".
#[must_use]
fn secret_from(value: Option<&str>) -> Option<Secret> {
    match value {
        Some(valor) if !valor.is_empty() => Some(Secret::new(valor)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use kinein_protocol::DataSourceEngine;

    use super::*;

    fn perfil(source: SecretSource, variable: Option<&str>) -> DataSourceProfile {
        DataSourceProfile {
            engine: DataSourceEngine::Postgres,
            name: "local".to_owned(),
            host: "localhost".to_owned(),
            port: 5432,
            database: "app".to_owned(),
            user: "postgres".to_owned(),
            secret_source: source,
            secret_variable: variable.map(str::to_owned),
            sample_size: None,
            tls: None,
            ca_file: None,
        }
    }

    #[test]
    fn secret_nao_aparece_em_debug_nem_display() {
        let secret = Secret::new("hunter2");
        assert_eq!(format!("{secret:?}"), "Secret(***)");
        assert_eq!(format!("{secret}"), "***");
        // E o valor continua acessivel por um nome que se ve' numa revisao.
        assert_eq!(secret.expose(), "hunter2");
    }

    /// O padrao NAO pode ser pedir senha.
    ///
    /// Um `PostgreSQL` local por socket unix com `peer` nao pergunta nada; se
    /// a IDE perguntasse, o obstaculo teria sido inventado por ela. Ver
    /// `DocsPublic/seguranca/40` §7.
    #[test]
    fn o_padrao_e_nao_mandar_senha_nenhuma() {
        assert_eq!(SecretSource::default(), SecretSource::Automatic);
        assert_eq!(
            plan_for(&perfil(SecretSource::Automatic, None)),
            SecretPlan::DelegateToDriver
        );
    }

    #[test]
    fn prompt_pede_ao_usuario_quando_escolhido() {
        assert_eq!(
            plan_for(&perfil(SecretSource::Prompt, None)),
            SecretPlan::AskUser
        );
    }

    #[test]
    fn ambiente_usa_a_variavel_nomeada() {
        assert_eq!(
            plan_for(&perfil(SecretSource::Environment, Some("  PGPASSWORD  "))),
            SecretPlan::ReadEnvironment("PGPASSWORD".to_owned())
        );
    }

    #[test]
    fn ambiente_sem_variavel_cai_para_o_prompt() {
        assert_eq!(
            plan_for(&perfil(SecretSource::Environment, None)),
            SecretPlan::AskUser
        );
        assert_eq!(
            plan_for(&perfil(SecretSource::Environment, Some("   "))),
            SecretPlan::AskUser
        );
    }

    #[test]
    fn variavel_ausente_ou_vazia_nao_vira_senha() {
        assert!(secret_from(None).is_none());
        assert!(secret_from(Some("")).is_none());
        assert_eq!(
            secret_from(Some("abc")).map(|s| s.expose().to_owned()),
            Some("abc".to_owned())
        );
    }

    #[test]
    fn ambiente_de_verdade_sem_a_variavel_nao_inventa_senha() {
        // Nome improvavel de proposito: o teste nao pode depender de quem roda,
        // e nao muta o ambiente — mutar `env` em teste e' corrida garantida.
        assert!(read_environment("KINEIN_VECTIS_VARIAVEL_QUE_NAO_EXISTE").is_none());
    }
}

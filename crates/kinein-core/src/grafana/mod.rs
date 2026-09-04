//! Observabilidade: o Grafana que observa este projeto.
//!
//! Este dominio nasceu em 2026-09-04 e fecha a etapa 27 do
//! `docs/roadmaps/35-ambiente-cpp-embarcados-simulacao.md`. Ele existe porque
//! a decisao do autor em 2026-09-03 foi *"banco e observabilidade sao NATIVOS,
//! nao plugins"* — e a licenca do Grafana decidiu o que "nativo" pode
//! significar aqui.
//!
//! # A licenca decide a FORMA
//!
//! ```text
//! PODE   a IDE CONVERSA com uma instancia de Grafana pela HTTP API dela.
//!        A Kinein nao redistribui, nao modifica e nao embute o Grafana.
//! NAO    embutir o Grafana na Kinein, ou distribui-lo no AppImage.
//! ```
//!
//! (`docs/integracoes/37-banco-e-observabilidade.md` §2, AGPL-3.0.)
//!
//! # O que faz este dominio valer mais que um atalho no navegador
//!
//! Um link para o Grafana e' um favorito. O que a IDE tem e o navegador nao
//! tem e' **o catalogo de fontes de dados deste workspace**. Cruzar os dois
//! responde a pergunta que o autor faria olhando as duas telas: *"o banco em
//! que estou desenvolvendo ja' esta' sendo observado?"*. E' o que [`cross_reference`]
//! faz, e e' a unica parte deste modulo que nao seria trivial noutro lugar.
//!
//! # Estrutura
//!
//! - [`client`]: o unico lugar que fala HTTP.
//! - [`store`]: `.kinein/grafana.json`, com `schemaVersion`.

pub mod client;
mod store;

use std::path::Path;

use kinein_protocol::{
    DataSourceEngine, DataSourceProfile, GrafanaDataSource, GrafanaMatch, GrafanaProfile,
    GrafanaTokenSource,
};

use crate::datasource::secret::SecretPlan;

/// Porta padrao do Grafana, usada como sugestao pela UI.
pub const DEFAULT_PORT: u16 = 3000;

/// Le a instancia salva neste workspace.
#[must_use]
pub fn get(root: &Path) -> Option<GrafanaProfile> {
    store::load(root)
}

/// Valida um perfil vindo da UI.
///
/// A mensagem e' o que o autor vai ler; por isso ela diz o que fazer.
///
/// # Errors
/// Devolve a primeira falha encontrada, na ordem em que o formulario e' lido.
pub fn validate(profile: &GrafanaProfile) -> Result<(), String> {
    let url = profile.url.trim();
    if url.is_empty() {
        return Err(format!(
            "informe o endereço do Grafana (o padrão local é http://localhost:{DEFAULT_PORT})"
        ));
    }
    // O ESQUEMA E' OBRIGATORIO, e recusar aqui e' melhor que adivinhar. Um
    // `localhost:3000` sem esquema viraria um caminho relativo dentro do
    // cliente HTTP e falharia com um erro que nao aponta para o campo errado.
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err("o endereço precisa começar com http:// ou https://".to_owned());
    }
    if profile.token_source == GrafanaTokenSource::Environment
        && profile
            .token_variable
            .as_ref()
            .is_none_or(|nome| nome.trim().is_empty())
    {
        return Err("informe o nome da variável de ambiente que guarda o token".to_owned());
    }
    Ok(())
}

/// Grava a instancia, substituindo a anterior.
///
/// # Errors
/// Perfil invalido ([`validate`]) ou falha de escrita em disco.
pub fn save(root: &Path, profile: &GrafanaProfile) -> Result<GrafanaProfile, String> {
    validate(profile)?;
    let normalizado = GrafanaProfile {
        url: client::normalize_url(&profile.url),
        token_source: profile.token_source,
        token_variable: profile
            .token_variable
            .as_ref()
            .map(|nome| nome.trim().to_owned())
            .filter(|nome| !nome.is_empty()),
    };
    store::save(root, Some(&normalizado))?;
    Ok(normalizado)
}

/// Esquece a instancia deste workspace.
///
/// # Errors
/// Falha de escrita em disco.
pub fn forget(root: &Path) -> Result<(), String> {
    store::save(root, None)
}

/// Traduz a politica do perfil no que fazer agora para obter o token.
///
/// `Environment` sem variavel nomeada CAI PARA O PROMPT em vez de falhar, pelo
/// mesmo motivo do `datasource`: perfil meio configurado nao pode impedir o
/// autor de trabalhar, e pedir e' sempre uma saida segura.
#[must_use]
pub fn plan_for(profile: &GrafanaProfile) -> SecretPlan {
    match profile.token_source {
        // `DelegateToDriver` aqui significa literalmente "mande nada". Nao ha'
        // ajudante de credencial em HTTP para delegar — e a sonda continua
        // util assim, porque `/api/health` responde sem token.
        GrafanaTokenSource::None => SecretPlan::DelegateToDriver,
        GrafanaTokenSource::Environment => match profile.token_variable.as_deref() {
            Some(nome) if !nome.trim().is_empty() => {
                SecretPlan::ReadEnvironment(nome.trim().to_owned())
            }
            _ => SecretPlan::AskUser,
        },
        GrafanaTokenSource::Prompt => SecretPlan::AskUser,
    }
}

/// Os hosts que significam "esta maquina".
///
/// Sao tratados como o mesmo host de proposito: um perfil da IDE apontando
/// para `localhost` e uma fonte do Grafana apontando para `127.0.0.1` sao, na
/// pratica de desenvolvimento, o mesmo banco. Fora desta lista a comparacao e'
/// literal — a IDE nao resolve DNS para adivinhar equivalencia.
const LOCAIS: [&str; 4] = ["localhost", "127.0.0.1", "::1", "[::1]"];

/// `true` quando dois nomes de host apontam para o mesmo lugar.
fn mesmo_host(um: &str, outro: &str) -> bool {
    let um = um.trim().to_lowercase();
    let outro = outro.trim().to_lowercase();
    if um == outro {
        return true;
    }
    LOCAIS.contains(&um.as_str()) && LOCAIS.contains(&outro.as_str())
}

/// Separa `host:porta` como o Grafana escreve o campo `url` de um banco.
///
/// O campo vem sem esquema (`localhost:5432`), mas um usuario pode ter digitado
/// com (`postgres://localhost:5432`) — a funcao aceita os dois, porque o valor
/// e' texto livre do outro lado.
fn host_e_porta(url: &str) -> (String, Option<u16>) {
    let sem_esquema = url.rsplit("://").next().unwrap_or(url).trim();
    let sem_caminho = sem_esquema.split('/').next().unwrap_or(sem_esquema);
    match sem_caminho.rsplit_once(':') {
        Some((host, porta)) => (host.to_owned(), porta.parse::<u16>().ok()),
        None => (sem_caminho.to_owned(), None),
    }
}

/// Cruza o catalogo deste workspace com o que o Grafana ja' observa.
///
/// # Por que o criterio e' conservador
///
/// Um falso positivo aqui e' pior que nenhum resultado: a tela diria *"seu
/// banco ja' esta' observado"* apontando para um dashboard de outro ambiente.
/// Por isso o nome do banco tem de bater E o host tem de bater — e o `reason`
/// carrega a comparacao que foi feita, para o autor conferir sem abrir o
/// Grafana.
#[must_use]
pub fn cross_reference(
    profiles: &[DataSourceProfile],
    sources: &[GrafanaDataSource],
) -> Vec<GrafanaMatch> {
    let mut achados = Vec::new();
    for profile in profiles {
        for source in sources {
            let Some(reason) = combina(profile, source) else {
                continue;
            };
            achados.push(GrafanaMatch {
                profile_name: profile.name.clone(),
                data_source_name: source.name.clone(),
                reason,
            });
        }
    }
    achados
}

/// Devolve a JUSTIFICATIVA da coincidencia, ou `None` quando nao ha' uma.
fn combina(profile: &DataSourceProfile, source: &GrafanaDataSource) -> Option<String> {
    if profile.engine == DataSourceEngine::Sqlite {
        // SQLITE NAO TEM FONTE NATIVA NO GRAFANA — a que existe e' plugin de
        // comunidade. Comparar caminho de arquivo e' o unico criterio honesto,
        // e ele so' vale se o outro lado escreveu exatamente o mesmo caminho.
        let caminho = profile.database.trim();
        if !caminho.is_empty()
            && (source.url.trim() == caminho || source.database.trim() == caminho)
        {
            return Some(format!("mesmo arquivo: {caminho}"));
        }
        return None;
    }

    let banco = profile.database.trim().to_lowercase();
    if banco.is_empty() || source.database.trim().to_lowercase() != banco {
        return None;
    }
    let (host, porta) = host_e_porta(&source.url);
    if !mesmo_host(&profile.host, &host) {
        return None;
    }
    match porta {
        // PORTA AUSENTE NAO REPROVA. O campo `url` do Grafana e' texto livre e
        // um usuario pode ter escrito so' o host; exigir a porta perderia a
        // coincidencia verdadeira. Mas a porta ausente aparece no `reason`,
        // para o autor saber que ela nao foi conferida.
        None => Some(format!(
            "banco `{banco}` no host `{host}` (sem porta declarada)"
        )),
        Some(valor) if valor == profile.port => Some(format!("banco `{banco}` em {host}:{valor}")),
        Some(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use kinein_protocol::SecretSource;

    use super::*;

    fn perfil_banco(nome: &str, host: &str, porta: u16, banco: &str) -> DataSourceProfile {
        DataSourceProfile {
            name: nome.to_owned(),
            engine: DataSourceEngine::Postgres,
            host: host.to_owned(),
            port: porta,
            database: banco.to_owned(),
            user: "hugh".to_owned(),
            secret_source: SecretSource::Automatic,
            secret_variable: None,
            sample_size: None,
        }
    }

    fn fonte(nome: &str, url: &str, banco: &str) -> GrafanaDataSource {
        GrafanaDataSource {
            uid: "u".to_owned(),
            name: nome.to_owned(),
            type_id: "grafana-postgresql-datasource".to_owned(),
            type_name: "PostgreSQL".to_owned(),
            url: url.to_owned(),
            database: banco.to_owned(),
            is_default: false,
        }
    }

    #[test]
    fn endereco_sem_esquema_e_recusado_apontando_o_conserto() {
        let erro = validate(&GrafanaProfile {
            url: "localhost:3000".to_owned(),
            ..GrafanaProfile::default()
        })
        .expect_err("deveria recusar");
        assert!(erro.contains("http://"));
    }

    #[test]
    fn ambiente_sem_variavel_nomeada_e_recusado() {
        let erro = validate(&GrafanaProfile {
            url: "http://localhost:3000".to_owned(),
            token_source: GrafanaTokenSource::Environment,
            token_variable: None,
        })
        .expect_err("deveria recusar");
        assert!(erro.contains("variável de ambiente"));
    }

    /// Politica meio preenchida nao pode travar o autor: cai para o prompt.
    #[test]
    fn ambiente_sem_variavel_no_disco_cai_para_o_prompt() {
        let plano = plan_for(&GrafanaProfile {
            url: "http://localhost:3000".to_owned(),
            token_source: GrafanaTokenSource::Environment,
            token_variable: Some("   ".to_owned()),
        });
        assert_eq!(plano, SecretPlan::AskUser);
    }

    #[test]
    fn sem_token_nao_pede_nada() {
        assert_eq!(
            plan_for(&GrafanaProfile::default()),
            SecretPlan::DelegateToDriver
        );
    }

    #[test]
    fn cruzamento_acha_o_banco_do_projeto() {
        let achados = cross_reference(
            &[perfil_banco("dev", "localhost", 5432, "kinein")],
            &[fonte("kinein-dev", "localhost:5432", "kinein")],
        );
        assert_eq!(achados.len(), 1);
        assert_eq!(achados[0].data_source_name, "kinein-dev");
        assert!(achados[0].reason.contains("5432"));
    }

    /// `localhost` e `127.0.0.1` sao o mesmo banco no dia a dia.
    #[test]
    fn loopback_conta_como_mesmo_host() {
        let achados = cross_reference(
            &[perfil_banco("dev", "localhost", 5432, "kinein")],
            &[fonte("k", "127.0.0.1:5432", "kinein")],
        );
        assert_eq!(achados.len(), 1);
    }

    /// O FALSO POSITIVO E' O DEFEITO CARO: dizer que o banco de producao ja'
    /// esta' observado quando o dashboard aponta para outro host.
    #[test]
    fn host_diferente_nao_combina() {
        let achados = cross_reference(
            &[perfil_banco("dev", "localhost", 5432, "kinein")],
            &[fonte("prod", "db.producao.interno:5432", "kinein")],
        );
        assert!(achados.is_empty());
    }

    #[test]
    fn porta_diferente_nao_combina() {
        let achados = cross_reference(
            &[perfil_banco("dev", "localhost", 5432, "kinein")],
            &[fonte("outro", "localhost:5433", "kinein")],
        );
        assert!(achados.is_empty());
    }

    /// Porta ausente no lado do Grafana ainda combina — e o `reason` avisa.
    #[test]
    fn porta_ausente_combina_mas_declara_a_lacuna() {
        let achados = cross_reference(
            &[perfil_banco("dev", "localhost", 5432, "kinein")],
            &[fonte("k", "localhost", "kinein")],
        );
        assert_eq!(achados.len(), 1);
        assert!(achados[0].reason.contains("sem porta"));
    }

    #[test]
    fn sqlite_combina_pelo_caminho_do_arquivo() {
        let mut perfil = perfil_banco("local", "", 0, "/home/hugh/dados.db");
        perfil.engine = DataSourceEngine::Sqlite;
        let achados = cross_reference(&[perfil], &[fonte("arquivo", "/home/hugh/dados.db", "")]);
        assert_eq!(achados.len(), 1);
        assert!(achados[0].reason.contains("dados.db"));
    }

    #[test]
    fn url_com_esquema_ainda_e_lida() {
        assert_eq!(
            host_e_porta("postgres://localhost:5432/kinein"),
            ("localhost".to_owned(), Some(5432))
        );
    }
}

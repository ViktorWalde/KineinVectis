//! O UNICO lugar deste repositorio que fala HTTP com um Grafana.
//!
//! Mesma regra do `datasource/connection.rs`: quem conversa com o mundo mora
//! sozinho, e todo o resto do dominio trabalha com o resultado da conversa.
//!
//! # O que este arquivo nunca faz
//!
//! Ele nao embute, nao redistribui e nao modifica o Grafana — a licenca
//! AGPL-3.0 decide a FORMA da integracao, e a forma e' esta: uma conversa HTTP
//! com um processo que e' do usuario (`docs/integracoes/37` §2, decisao do
//! autor em 2026-09-03).

use std::time::Duration;

use kinein_protocol::{GrafanaDashboard, GrafanaDataSource, GrafanaProbeResult};
use serde::Deserialize;

use crate::datasource::secret::Secret;

/// Teto de tempo de uma requisicao, igual ao do `datasource/connection.rs`.
///
/// Um Grafana que nao responde em cinco segundos e' um Grafana que o autor
/// precisa saber que nao respondeu — nao um painel travado.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(5);

/// Teto de corpo de resposta, por requisicao.
///
/// O `/api/search` do Grafana devolve ate' 5.000 itens por pagina por padrao.
/// Dois megabytes cobrem isso com folga e ainda assim recusam uma resposta
/// que so' pode ser um engano — um proxy devolvendo HTML, por exemplo.
const MAX_BODY: u64 = 2 * 1024 * 1024;

/// Quantos dashboards a IDE lista. Alem disto a lista deixa de ser navegavel
/// e vira despejo; o Grafana continua sendo o lugar de procurar entre milhares.
const MAX_DASHBOARDS: usize = 500;

/// A saude publica: `GET /api/health` responde sem nenhuma autenticacao.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Health {
    #[serde(default)]
    database: String,
    #[serde(default)]
    version: String,
}

/// Uma fonte de dados como o Grafana 13 a devolve.
///
/// A documentacao do endpoint lista um campo `password`, e servidores antigos
/// chegam a envia-lo. **Ele nao existe nesta struct**, entao o `serde` o
/// descarta na entrada: a garantia e' estrutural, nao uma lembranca de quem le'
/// o JSON.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawDataSource {
    #[serde(default)]
    uid: String,
    #[serde(default)]
    name: String,
    #[serde(default, rename = "type")]
    type_id: String,
    #[serde(default)]
    type_name: String,
    #[serde(default)]
    url: String,
    #[serde(default)]
    database: String,
    #[serde(default)]
    is_default: bool,
}

/// Um item de `GET /api/search`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawSearchItem {
    #[serde(default)]
    uid: String,
    #[serde(default)]
    title: String,
    #[serde(default)]
    url: String,
    #[serde(default)]
    folder_title: String,
}

/// Normaliza a URL do perfil: sem barra final, para os `format!` abaixo nao
/// produzirem `//api/health`.
#[must_use]
pub fn normalize_url(url: &str) -> String {
    url.trim().trim_end_matches('/').to_owned()
}

/// Monta o agente. Um por sonda: a sonda e' rara e nao paga pool.
fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_global(Some(REQUEST_TIMEOUT))
        // O CODIGO DE STATUS E' DIAGNOSTICO, NAO ERRO. Com o padrao do `ureq`,
        // um 401 vira `Err` e o 401 se perde no meio de todas as outras falhas
        // possiveis. Aqui ele precisa ser distinguido: 401 e' "o token nao
        // serve", e isso e' uma frase diferente de "nao achei o servidor".
        .http_status_as_error(false)
        .build()
        .into()
}

/// Um GET autenticado; devolve `(status, corpo)`.
fn get(agent: &ureq::Agent, url: &str, token: Option<&Secret>) -> Result<(u16, String), String> {
    let mut request = agent.get(url);
    if let Some(secret) = token {
        if !secret.is_empty() {
            request = request.header("Authorization", &format!("Bearer {}", secret.expose()));
        }
    }
    let mut response = request
        .call()
        .map_err(|error| describe(&error.to_string(), url))?;
    let status = response.status().as_u16();
    let body = response
        .body_mut()
        .with_config()
        .limit(MAX_BODY)
        .read_to_string()
        .map_err(|error| format!("resposta ilegível de {url}: {error}"))?;
    Ok((status, body))
}

/// Traduz a falha de rede crua para uma frase que diz o que fazer.
///
/// Mesma disciplina do `describe()` do `datasource/connection.rs`: a mensagem
/// que o autor le' precisa dizer o proximo passo, nao a estrutura interna da
/// biblioteca.
fn describe(error: &str, url: &str) -> String {
    let baixo = error.to_lowercase();
    if baixo.contains("connection refused") {
        return format!("nada atende em {url} — o Grafana está rodando nessa porta?");
    }
    if baixo.contains("timed out") || baixo.contains("timeout") {
        return format!("{url} não respondeu em 5s");
    }
    if baixo.contains("dns") || baixo.contains("name or service not known") {
        return format!("nome não resolvido em {url}");
    }
    format!("falha falando com {url}: {error}")
}

/// Alcanca a instancia e conta o que ela tem.
///
/// # A sonda tem DOIS resultados, e por isso dois booleanos
///
/// `/api/health` nao pede credencial; o resto pede. Juntar os dois num "ok" so'
/// colapsaria duas situacoes muito diferentes — *"nao ha' Grafana nesse
/// endereco"* e *"o Grafana esta' la' e recusou seu token"* — numa mensagem que
/// nao ajuda em nenhuma das duas.
#[must_use]
pub fn probe(url: &str, token: Option<&Secret>) -> GrafanaProbeResult {
    let url = normalize_url(url);
    let agent = agent();

    let (status, body) = match get(&agent, &format!("{url}/api/health"), None) {
        Ok(par) => par,
        Err(mensagem) => {
            return GrafanaProbeResult {
                message: mensagem,
                ..GrafanaProbeResult::default()
            };
        }
    };
    if status != 200 {
        return GrafanaProbeResult {
            message: format!("{url}/api/health respondeu {status} — isso é um Grafana?"),
            ..GrafanaProbeResult::default()
        };
    }
    let Ok(health) = serde_json::from_str::<Health>(&body) else {
        return GrafanaProbeResult {
            message: format!("{url} respondeu, mas não com a saúde de um Grafana"),
            ..GrafanaProbeResult::default()
        };
    };

    let mut resultado = GrafanaProbeResult {
        reachable: true,
        version: health.version,
        database: health.database,
        ..GrafanaProbeResult::default()
    };

    // SEM TOKEN NAO E' ERRO. E' o estado `GrafanaTokenSource::None`, e ele
    // responde uma pergunta util sozinho: existe um Grafana aqui, e ele esta'
    // saudavel. O que falta e' o que esta' DENTRO.
    let Some(secret) = token.filter(|secret| !secret.is_empty()) else {
        "sem token: dá para ver a versão, não o que há dentro da instância"
            .clone_into(&mut resultado.message);
        return resultado;
    };

    match get(&agent, &format!("{url}/api/datasources"), Some(secret)) {
        Ok((401 | 403, _)) => {
            "o Grafana respondeu, mas recusou o token — confira a conta de serviço \
             e a permissão `datasources:read`"
                .clone_into(&mut resultado.message);
            return resultado;
        }
        Ok((200, corpo)) => {
            resultado.data_sources = parse_data_sources(&corpo);
        }
        Ok((status, _)) => {
            resultado.message = format!("/api/datasources respondeu {status}");
            return resultado;
        }
        Err(mensagem) => {
            resultado.message = mensagem;
            return resultado;
        }
    }

    resultado.authenticated = true;
    match get(
        &agent,
        &format!("{url}/api/search?type=dash-db&limit={MAX_DASHBOARDS}"),
        Some(secret),
    ) {
        Ok((200, corpo)) => resultado.dashboards = parse_dashboards(&corpo),
        // O TOKEN JA' PROVOU QUE SERVE no passo anterior. Uma falha so' aqui e'
        // permissao de dashboard, e ela nao pode anular o que ja' foi lido.
        Ok((status, _)) => {
            resultado.message = format!("fontes de dados lidas; /api/search respondeu {status}");
        }
        Err(mensagem) => resultado.message = mensagem,
    }
    resultado
}

/// Converte a resposta crua em tipos do protocolo, descartando o resto.
fn parse_data_sources(body: &str) -> Vec<GrafanaDataSource> {
    let Ok(cruas) = serde_json::from_str::<Vec<RawDataSource>>(body) else {
        return Vec::new();
    };
    cruas
        .into_iter()
        .map(|crua| GrafanaDataSource {
            uid: crua.uid,
            name: crua.name,
            type_id: crua.type_id,
            type_name: crua.type_name,
            url: crua.url,
            database: crua.database,
            is_default: crua.is_default,
        })
        .collect()
}

/// Idem para os dashboards.
fn parse_dashboards(body: &str) -> Vec<GrafanaDashboard> {
    let Ok(cruas) = serde_json::from_str::<Vec<RawSearchItem>>(body) else {
        return Vec::new();
    };
    cruas
        .into_iter()
        .take(MAX_DASHBOARDS)
        .map(|crua| GrafanaDashboard {
            uid: crua.uid,
            title: crua.title,
            url: crua.url,
            folder_title: crua.folder_title,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn url_perde_a_barra_final() {
        assert_eq!(
            normalize_url("http://localhost:3000/"),
            "http://localhost:3000"
        );
        assert_eq!(
            normalize_url("  http://localhost:3000  "),
            "http://localhost:3000"
        );
    }

    /// `https` DEIXOU DE SER RECUSADO em 2026-09-04.
    ///
    /// Ate' aquele dia o `ureq` entrava sem TLS, e um endereco cifrado recebia
    /// uma recusa explicita — a alternativa era um erro de esquema desconhecido
    /// que nao explicava nada. Quando o autor decidiu a politica de licenca
    /// (`subtle` e `webpki-roots` na allowlist), o TLS entrou e a recusa virou
    /// mentira: ela dizia "compilado sem TLS" de um binario que tem TLS.
    ///
    /// Este teste guarda a inversao. Ele NAO alcanca a rede: um dominio
    /// reservado pela RFC 2606 nao resolve, entao o que se prova e' que a falha
    /// vem de RESOLUCAO — e nao de uma recusa nossa antes de tentar.
    #[test]
    fn https_nao_e_mais_recusado_de_saida() {
        let resultado = probe("https://grafana.invalid", None);
        assert!(!resultado.reachable);
        assert!(
            !resultado.message.contains("TLS"),
            "a recusa de TLS sobreviveu ao TLS: {}",
            resultado.message
        );
        assert!(
            resultado.message.contains("grafana.invalid"),
            "a falha deveria nomear o endereco tentado: {}",
            resultado.message
        );
    }

    /// O CAMPO QUE NAO PODE EXISTIR. A documentacao do `/api/datasources`
    /// lista `password`; se alguem acrescentar o campo a `RawDataSource` ou ao
    /// tipo do protocolo, este teste reprova.
    #[test]
    fn senha_da_fonte_de_dados_e_descartada_na_entrada() {
        let corpo = r#"[{"uid":"a","name":"n","type":"postgres","password":"segredo",
                         "secureJsonFields":{"password":true},"url":"h:5432"}]"#;
        let fontes = parse_data_sources(corpo);
        assert_eq!(fontes.len(), 1);
        let serializado = serde_json::to_string(&fontes[0]).expect("serializa");
        assert!(
            !serializado.contains("segredo"),
            "a senha atravessou o parser: {serializado}"
        );
        assert!(
            !serializado.to_lowercase().contains("password"),
            "o tipo do protocolo ganhou campo de senha: {serializado}"
        );
    }

    /// Corpo que nao e' a lista esperada devolve vazio, nunca panic: a resposta
    /// vem de um servidor que nao e' deste repositorio.
    #[test]
    fn corpo_estranho_nao_derruba_o_parser() {
        assert!(parse_data_sources("<html>proxy</html>").is_empty());
        assert!(parse_dashboards("null").is_empty());
    }

    /// Servidor inexistente vira frase acionavel, nao texto de biblioteca.
    #[test]
    fn porta_fechada_diz_o_que_conferir() {
        // Porta alta e' escolhida para nao esbarrar em servico real da maquina.
        let resultado = probe("http://127.0.0.1:1", None);
        assert!(!resultado.reachable);
        assert!(
            resultado.message.contains("127.0.0.1:1"),
            "mensagem sem o endereco: {}",
            resultado.message
        );
    }
}

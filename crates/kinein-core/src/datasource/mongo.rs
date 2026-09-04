//! O unico lugar que fala com um servidor `MongoDB`.
//!
//! Irmao do `connection.rs` (`PostgreSQL`) e do `sqlite.rs`, e com a mesma
//! divisao: aqui mora a conversa com o servidor; a conta que transforma
//! documentos em campos mora no [`super::mongo_infer`], que nao sabe que existe
//! rede.
//!
//! # Duas verdades, e so' uma delas e' palpite
//!
//! Uma chamada de `listCollections` — **zero documentos lidos** — ja' entrega o
//! tipo da colecao, o validador `$jsonSchema` quando existe e as opcoes de
//! serie temporal. Isso e' DECLARADO: e' o equivalente `MongoDB` do
//! `information_schema` que o `PostgreSQL` da'.
//!
//! O resto e' INFERIDO de uma amostra, e vem com probabilidade de presenca,
//! nunca com garantia. A decisao do autor em 2026-09-04 foi mostrar o declarado
//! quando ele existe e a amostra quando nao — e nunca deixar os dois parecidos
//! na tela.

use std::time::Duration;

use kinein_protocol::{DataSourceProfile, MongoCollection, MongoField};
use mongodb::{
    bson::{Bson, Document, doc},
    options::{ClientOptions, Credential, ServerAddress},
    sync::Client,
};

use super::mongo_infer::Inference;
use super::secret::Secret;

/// Porta padrao do `MongoDB`.
pub const DEFAULT_PORT: u16 = 27017;

/// Amostra padrao, e ela NAO e' os 1.000 do Compass.
///
/// O `$sample` do `MongoDB` so' usa o cursor pseudoaleatorio barato quando N e'
/// menor que 5% da colecao E ela tem mais de 100 documentos; fora disso ele
/// varre tudo e ordena. Com N = 1.000, isso significa varredura completa em
/// TODA colecao com menos de 20.000 documentos — trabalho no banco do autor,
/// disparado por abrir um painel. Com 200, o limiar cai para 4.000.
pub const DEFAULT_SAMPLE: u32 = 200;

/// Teto da amostra escolhida no perfil.
pub const MAX_SAMPLE: u32 = 2_000;

/// Limiar do `$sample` documentado pelo `MongoDB`: **nao e' configuravel**.
const SCAN_THRESHOLD: f64 = 0.05;

/// Abaixo disto o `$sample` tambem varre — e varrer 100 documentos e' de graca.
const SCAN_MIN_DOCUMENTS: u64 = 100;

/// Tempo maximo esperando o servidor, igual ao dos outros motores.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);

/// Colecoes internas do servidor. O autor nao as criou e nao pode mexer nelas.
///
/// O `system.buckets.*` e' o armazenamento real de uma colecao temporal: mostra-lo
/// exibiria o formato interno de balde ao lado da colecao que o autor de fato
/// escreve, e as duas linhas diriam a mesma coisa de dois jeitos.
fn is_internal(name: &str) -> bool {
    name.starts_with("system.")
}

/// Monta as opcoes de conexao SEM colocar a senha numa URI.
///
/// POR QUE NAO `mongodb://usuario:senha@host`. A URI seria uma `String` viva na
/// memoria com a senha em texto, pronta para cair num `Debug`, num log de erro
/// ou numa mensagem de falha de conexao. O `Credential` recebe o valor no ponto
/// de uso e o `Secret` continua sendo o unico dono ate' la'.
fn options_for(profile: &DataSourceProfile, secret: Option<&Secret>) -> ClientOptions {
    let porta = if profile.port == 0 {
        DEFAULT_PORT
    } else {
        profile.port
    };
    let mut options = ClientOptions::default();
    options.hosts = vec![ServerAddress::Tcp {
        host: profile.host.trim().to_owned(),
        port: Some(porta),
    }];
    options.connect_timeout = Some(CONNECT_TIMEOUT);
    options.server_selection_timeout = Some(CONNECT_TIMEOUT);
    options.app_name = Some("kinein-vectis".to_owned());
    let usuario = profile.user.trim();
    if !usuario.is_empty() {
        let mut credential = Credential::default();
        credential.username = Some(usuario.to_owned());
        credential.password = secret.map(|valor| valor.expose().to_owned());
        options.credential = Some(credential);
    }
    options
}

/// Uma falha de conexao, na mesma forma que o `PostgreSQL` devolve.
#[derive(Debug, Clone)]
pub struct MongoFailure {
    /// Frase que diz o que fazer.
    pub message: String,
    /// `true` quando o servidor recusou a credencial.
    pub secret_required: bool,
}

/// Traduz a falha crua numa frase que diz o proximo passo.
///
/// AUTENTICACAO E' UM CASO PROPRIO, como o `28P01` e' no `PostgreSQL`: a UI abre
/// o dialogo de senha por este booleano, nunca lendo o texto.
fn describe(error: &mongodb::error::Error, host: &str) -> MongoFailure {
    let bruto = error.to_string();
    let baixo = bruto.to_lowercase();
    if baixo.contains("authentication failed") || baixo.contains("auth failed") {
        return MongoFailure {
            message: "o servidor recusou a credencial".to_owned(),
            secret_required: true,
        };
    }
    if baixo.contains("connection refused") {
        return MongoFailure {
            message: format!(
                "nada atende nesse host e porta — o MongoDB está rodando?{}",
                dica_localhost(host)
            ),
            secret_required: false,
        };
    }
    if baixo.contains("server selection timeout") || baixo.contains("timed out") {
        return MongoFailure {
            message: format!("o servidor não respondeu em 5s{}", dica_localhost(host)),
            secret_required: false,
        };
    }
    if baixo.contains("connection reset") {
        return MongoFailure {
            message: format!(
                "o servidor fechou a conexão no meio do aperto de mão{}",
                dica_localhost(host)
            ),
            secret_required: false,
        };
    }
    MongoFailure {
        message: format!("{bruto}{}", dica_localhost(host)),
        secret_required: false,
    }
}

/// A DICA DO `localhost`, medida em 2026-09-04 nesta maquina.
///
/// Com um `MongoDB` em contêiner rootless, `localhost` resolve para `::1`
/// primeiro; o encaminhador de porta nao atende em IPv6 e o driver **nao cai
/// para IPv4**. O sintoma varia — ora timeout de cinco segundos, ora conexao
/// resetada no aperto de mao — e nenhum dos dois aponta para o nome do host.
/// Com `127.0.0.1` conecta na hora.
///
/// Por isso a dica vai junto de TODA falha de conexao quando o host e'
/// `localhost`, e nao so' do timeout: o sintoma nao e' estavel, e o conselho e'
/// barato e verdadeiro nos dois.
fn dica_localhost(host: &str) -> &'static str {
    if host.trim().eq_ignore_ascii_case("localhost") {
        " — tente `127.0.0.1`: `localhost` pode resolver para IPv6, e nem todo servidor atende nele"
    } else {
        ""
    }
}

/// Conecta uma vez e devolve a versao do servidor.
///
/// # Errors
/// Falha de rede, de autenticacao ou de permissao.
pub fn probe_server(
    profile: &DataSourceProfile,
    secret: Option<&Secret>,
) -> Result<String, MongoFailure> {
    let client = Client::with_options(options_for(profile, secret))
        .map_err(|e| describe(&e, &profile.host))?;
    let banco = client.database(banco_de(profile));
    let resposta = banco
        .run_command(doc! { "buildInfo": 1 })
        .run()
        .map_err(|e| describe(&e, &profile.host))?;
    let versao = resposta
        .get_str("version")
        .unwrap_or("versão desconhecida")
        .to_owned();
    Ok(format!("MongoDB {versao}"))
}

/// O banco do perfil, com o padrao do servidor quando vazio.
fn banco_de(profile: &DataSourceProfile) -> &str {
    let nome = profile.database.trim();
    if nome.is_empty() { "admin" } else { nome }
}

/// Le a estrutura: colecoes, com campos declarados ou inferidos.
///
/// # Errors
/// Falha de rede, de autenticacao ou de permissao.
pub fn read_structure(
    profile: &DataSourceProfile,
    secret: Option<&Secret>,
) -> Result<Vec<MongoCollection>, MongoFailure> {
    let client = Client::with_options(options_for(profile, secret))
        .map_err(|e| describe(&e, &profile.host))?;
    let banco = client.database(banco_de(profile));
    let amostra = profile
        .sample_size
        .unwrap_or(DEFAULT_SAMPLE)
        .clamp(1, MAX_SAMPLE);

    let cursor = banco
        .list_collections()
        .run()
        .map_err(|e| describe(&e, &profile.host))?;
    let mut colecoes = Vec::new();
    for especificacao in cursor {
        let especificacao = especificacao.map_err(|e| describe(&e, &profile.host))?;
        if is_internal(&especificacao.name) {
            continue;
        }
        let opcoes = mongodb::bson::to_document(&especificacao.options).unwrap_or_default();
        colecoes.push(read_collection(
            &banco,
            &especificacao.name,
            &format!("{:?}", especificacao.collection_type).to_lowercase(),
            &opcoes,
            amostra,
        ));
    }
    colecoes.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(colecoes)
}

/// Uma colecao: o declarado primeiro, a amostra so' se ele nao existir.
fn read_collection(
    banco: &mongodb::sync::Database,
    nome: &str,
    tipo: &str,
    opcoes: &Document,
    amostra: u32,
) -> MongoCollection {
    let mut colecao = MongoCollection {
        name: nome.to_owned(),
        kind: tipo.to_owned(),
        ..MongoCollection::default()
    };
    ler_temporal(opcoes, &mut colecao);

    // O DECLARADO GANHA. Ele nao custa documento nenhum e nao pode estar
    // errado; amostrar por cima seria trocar um fato por uma frequencia.
    if let Some(campos) = campos_declarados(opcoes) {
        colecao.declared = true;
        colecao.fields = campos;
        return colecao;
    }

    // VISAO NAO E' CONTADA: `estimatedDocumentCount` nao vale nela, e contar de
    // verdade seria executar o pipeline inteiro so' para preencher um numero.
    // Sem a contagem, o `$sample` sobre uma visao e' sempre o caminho caro, e
    // dizer isso e' mais honesto que omitir.
    let e_visao = tipo == "view";
    if !e_visao {
        colecao.document_count = banco
            .collection::<Document>(nome)
            .estimated_document_count()
            .run()
            .ok();
    }
    colecao.full_scan = colecao
        .document_count
        .is_none_or(|total| varre_tudo(amostra, total));

    let (campos, lidos, cortado) = amostrar(banco, nome, amostra);
    colecao.fields = campos;
    colecao.sampled = lidos;
    colecao.truncated = cortado;
    colecao
}

/// `true` quando o `$sample` vai varrer a colecao inteira em vez de sortear.
///
/// A regra e' a documentada pelo `MongoDB` e o limiar de 5% **nao e'
/// configuravel**. A IDE calcula antes e conta na tela — em vez de disparar o
/// trabalho em silencio e o autor descobrir pelo grafico de carga.
#[must_use]
pub fn varre_tudo(amostra: u32, total: u64) -> bool {
    if total <= SCAN_MIN_DOCUMENTS {
        return true;
    }
    #[allow(clippy::cast_precision_loss)]
    let fracao = f64::from(amostra) / total as f64;
    fracao >= SCAN_THRESHOLD
}

/// Le as opcoes de serie temporal, que sao DECLARADAS.
fn ler_temporal(opcoes: &Document, colecao: &mut MongoCollection) {
    let Ok(temporal) = opcoes.get_document("timeseries") else {
        return;
    };
    temporal
        .get_str("timeField")
        .unwrap_or_default()
        .clone_into(&mut colecao.time_field);
    temporal
        .get_str("metaField")
        .unwrap_or_default()
        .clone_into(&mut colecao.meta_field);
    temporal
        .get_str("granularity")
        .unwrap_or_default()
        .clone_into(&mut colecao.granularity);
}

/// Extrai os campos do validador `$jsonSchema`, quando ha' um.
fn campos_declarados(opcoes: &Document) -> Option<Vec<MongoField>> {
    let esquema = opcoes
        .get_document("validator")
        .ok()?
        .get_document("$jsonSchema")
        .ok()?;
    let mut campos = Vec::new();
    coletar_declarados(esquema, "", 0, &mut campos);
    if campos.is_empty() {
        None
    } else {
        Some(campos)
    }
}

/// Percorre o `$jsonSchema`, que aninha por `properties`.
fn coletar_declarados(
    esquema: &Document,
    prefixo: &str,
    profundidade: u16,
    saida: &mut Vec<MongoField>,
) {
    let obrigatorios: Vec<&str> = esquema
        .get_array("required")
        .map(|lista| lista.iter().filter_map(Bson::as_str).collect())
        .unwrap_or_default();
    let Ok(propriedades) = esquema.get_document("properties") else {
        return;
    };
    for (nome, valor) in propriedades {
        let Some(interno) = valor.as_document() else {
            continue;
        };
        let caminho = if prefixo.is_empty() {
            nome.clone()
        } else {
            format!("{prefixo}.{nome}")
        };
        saida.push(MongoField {
            depth: profundidade,
            types: tipo_declarado(interno),
            // SEM AMOSTRA NAO HA' PORCENTAGEM. Inventar uma seria a tela
            // afirmando uma medicao que nao aconteceu.
            presence: None,
            required: obrigatorios.contains(&nome.as_str()),
            path: caminho.clone(),
        });
        coletar_declarados(interno, &caminho, profundidade + 1, saida);
    }
}

/// O `bsonType` declarado, que pode ser um nome ou uma lista.
fn tipo_declarado(esquema: &Document) -> Vec<String> {
    match esquema.get("bsonType") {
        Some(Bson::String(nome)) => vec![nome.clone()],
        Some(Bson::Array(lista)) => lista
            .iter()
            .filter_map(|item| item.as_str().map(str::to_owned))
            .collect(),
        _ => Vec::new(),
    }
}

/// Pede a amostra e dobra documento a documento.
///
/// O cursor e' consumido em fluxo: cada documento e' dobrado no mapa e
/// DESCARTADO em seguida, entao o pico de memoria nao cresce com o tamanho da
/// colecao — so' com o numero de caminhos distintos, que tem teto proprio.
fn amostrar(
    banco: &mongodb::sync::Database,
    nome: &str,
    amostra: u32,
) -> (Vec<MongoField>, u32, String) {
    let pipeline = vec![doc! { "$sample": { "size": i64::from(amostra) } }];
    let Ok(cursor) = banco.collection::<Document>(nome).aggregate(pipeline).run() else {
        return (
            Vec::new(),
            0,
            "não consegui amostrar esta coleção".to_owned(),
        );
    };
    let mut inferencia = Inference::default();
    for documento in cursor {
        let Ok(documento) = documento else {
            break;
        };
        inferencia.add(&documento);
    }
    let lidos = inferencia.documents();
    let cortado = inferencia.truncated();
    (inferencia.into_fields(), lidos, cortado)
}

#[cfg(test)]
mod tests {
    use mongodb::bson::doc;

    use super::*;

    #[test]
    fn colecoes_internas_ficam_de_fora() {
        assert!(is_internal("system.views"));
        assert!(is_internal("system.buckets.leituras"));
        assert!(!is_internal("clientes"));
    }

    /// A REGRA DOS 5%, que decide se abrir o painel varre o banco do autor.
    #[test]
    fn a_regra_dos_cinco_por_cento_e_a_do_mongodb() {
        // Colecao pequena: varrer 100 documentos e' de graca, e o `$sample`
        // varre mesmo.
        assert!(varre_tudo(200, 100));
        // 200 de 4.000 e' exatamente 5% — NAO e' menor que 5%, entao varre.
        assert!(varre_tudo(200, 4_000));
        // 200 de 4.001 ja' e' menor que 5%: cursor pseudoaleatorio.
        assert!(!varre_tudo(200, 4_001));
        // E o motivo de o padrao nao ser 1.000: com ele o limiar sobe para
        // 20.000 documentos.
        assert!(varre_tudo(1_000, 20_000));
        assert!(!varre_tudo(1_000, 20_001));
    }

    #[test]
    fn a_senha_nao_entra_na_uri() {
        let perfil = DataSourceProfile {
            name: "m".to_owned(),
            engine: kinein_protocol::DataSourceEngine::Mongo,
            host: "localhost".to_owned(),
            port: 0,
            database: "kinein".to_owned(),
            user: "hugh".to_owned(),
            secret_source: kinein_protocol::SecretSource::Prompt,
            secret_variable: None,
            sample_size: None,
        };
        let opcoes = options_for(&perfil, Some(&Secret::new("segredo")));
        // Porta zero vira o padrao do motor, e nao uma conexao na porta 0.
        assert_eq!(
            opcoes.hosts[0],
            ServerAddress::Tcp {
                host: "localhost".to_owned(),
                port: Some(DEFAULT_PORT)
            }
        );
        // O `Debug` das opcoes nao pode carregar a senha em texto.
        let impresso = format!("{opcoes:?}");
        assert!(
            !impresso.contains("segredo"),
            "a senha vazou no Debug: {impresso}"
        );
    }

    #[test]
    fn sem_usuario_nao_ha_credencial() {
        let perfil = DataSourceProfile {
            name: "m".to_owned(),
            engine: kinein_protocol::DataSourceEngine::Mongo,
            host: "localhost".to_owned(),
            port: 27_017,
            database: "kinein".to_owned(),
            user: String::new(),
            secret_source: kinein_protocol::SecretSource::Automatic,
            secret_variable: None,
            sample_size: None,
        };
        assert!(options_for(&perfil, None).credential.is_none());
    }

    #[test]
    fn validador_vira_campo_declarado_com_obrigatoriedade() {
        let opcoes = doc! { "validator": { "$jsonSchema": {
            "bsonType": "object",
            "required": ["nome"],
            "properties": {
                "nome": { "bsonType": "string" },
                "idade": { "bsonType": ["int", "null"] },
                "meta": { "bsonType": "object", "properties": { "cor": { "bsonType": "string" } } }
            }
        }}};
        let campos = campos_declarados(&opcoes).expect("ha' esquema declarado");
        let nome = campos.iter().find(|c| c.path == "nome").unwrap();
        assert!(nome.required);
        assert_eq!(nome.types, vec!["string"]);
        // SEM AMOSTRA, SEM PORCENTAGEM.
        assert!(nome.presence.is_none());
        let idade = campos.iter().find(|c| c.path == "idade").unwrap();
        assert!(!idade.required);
        assert_eq!(idade.types, vec!["int", "null"]);
        // O declarado tambem aninha.
        assert!(campos.iter().any(|c| c.path == "meta.cor" && c.depth == 1));
    }

    #[test]
    fn colecao_sem_validador_nao_finge_ter_um() {
        assert!(campos_declarados(&doc! {}).is_none());
        assert!(
            campos_declarados(&doc! { "validator": { "nome": { "$type": "string" } } }).is_none()
        );
    }

    #[test]
    fn opcoes_temporais_sao_lidas_do_catalogo() {
        let mut colecao = MongoCollection::default();
        ler_temporal(
            &doc! { "timeseries": { "timeField": "ts", "metaField": "sensor",
            "granularity": "seconds" } },
            &mut colecao,
        );
        assert_eq!(colecao.time_field, "ts");
        assert_eq!(colecao.meta_field, "sensor");
        assert_eq!(colecao.granularity, "seconds");
    }
}

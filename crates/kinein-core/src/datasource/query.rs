//! Executar o que o autor escreveu (`datasource.query`, 0.121.0 — o desenho
//! esta' no `roadmaps/35` §7.4, escrito antes deste arquivo).
//!
//! ```text
//! leitura   SELECT | WITH | VALUES | TABLE | SHOW | EXPLAIN  (primeira palavra)
//!           o MOTOR impoe: BEGIN READ ONLY no PostgreSQL, SQLITE_OPEN_READ_ONLY
//!           no SQLite — um `WITH … INSERT` disfarcado e' recusado pelo servidor
//! teto      SELECT * FROM (<sql>) AS kinein_q LIMIT n+1  (uma instrucao so')
//!           no SQLite, `step` ate' n+1; `truncated` diz quando cortou
//! escrita   o que o autor escreveu e' o que roda; `affected` e' o que o
//!           motor contou. Quem confirma e' a UI (WRITE_CONFIRMATION_REQUIRED
//!           e' recusa SINCRONA do handler, antes do job)
//! celulas   texto (protocolo simples do PostgreSQL; ValueRef do SQLite;
//!           JSON do valor no Mongo); NULL e' `null`
//! mongo     `<colecao> <filtro JSON>` -> find com limit; so' leitura nesta fatia
//! ```

use std::time::Instant;

use kinein_protocol::{DataSourceEngine, DataSourceProfile};
use mongodb::bson::{Bson, Document};
use postgres::SimpleQueryMessage;
use rusqlite::types::ValueRef;

use super::connection::{ConnectionFailure, connect, failure_from};
use super::secret::Secret;
use super::{mongo, sqlite};

/// Teto padrao de linhas de uma leitura.
pub const DEFAULT_MAX_ROWS: u32 = 500;
/// Teto absoluto: acima disto a pipe e a tela sofrem antes de o autor ler.
pub const MAX_ROWS: u32 = 10_000;

/// O que uma instrucao devolveu.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct QueryResult {
    /// Nomes das colunas (vazio numa escrita).
    pub columns: Vec<String>,
    /// Linhas como texto; `None` e' NULL.
    pub rows: Vec<Vec<Option<String>>>,
    /// Linhas que uma escrita tocou, quando o motor conta.
    pub affected: Option<u64>,
    /// O teto cortou.
    pub truncated: bool,
    /// Tempo de parede, ms.
    pub elapsed_ms: u64,
}

/// A instrucao e' de LEITURA pela primeira palavra (comentarios `--` e
/// `/* */` iniciais pulados). Puro; o motor confirma.
#[must_use]
pub fn is_read(sql: &str) -> bool {
    let corpo = strip_leading_comments(sql);
    let palavra: String = corpo
        .chars()
        .take_while(char::is_ascii_alphabetic)
        .collect::<String>()
        .to_ascii_uppercase();
    matches!(
        palavra.as_str(),
        "SELECT" | "WITH" | "VALUES" | "TABLE" | "SHOW" | "EXPLAIN"
    )
}

fn strip_leading_comments(sql: &str) -> &str {
    let mut resto = sql.trim_start();
    loop {
        if let Some(depois) = resto.strip_prefix("--") {
            resto = depois.split_once('\n').map_or("", |(_, r)| r).trim_start();
        } else if let Some(depois) = resto.strip_prefix("/*") {
            resto = depois.split_once("*/").map_or("", |(_, r)| r).trim_start();
        } else {
            return resto;
        }
    }
}

/// Uma instrucao so' (sem `;` no meio), sem o `;` final.
#[must_use]
pub fn single_statement(sql: &str) -> Option<&str> {
    let corpo = strip_leading_comments(sql).trim_end();
    let corpo = corpo.strip_suffix(';').unwrap_or(corpo).trim_end();
    (!corpo.is_empty() && !corpo.contains(';')).then_some(corpo)
}

/// O teto vindo de FORA do texto do autor.
///
/// `SELECT * FROM (<sql>) AS kinein_q LIMIT n+1` para
/// `SELECT`/`WITH`/`VALUES`/`TABLE` de uma instrucao so'; o resto (`SHOW`,
/// `EXPLAIN`, varias instrucoes) vai como esta' e o corte e' na leitura.
#[must_use]
pub fn wrap_limit(sql: &str, max_rows: u32) -> String {
    let Some(corpo) = single_statement(sql) else {
        return sql.to_owned();
    };
    let palavra: String = corpo
        .chars()
        .take_while(char::is_ascii_alphabetic)
        .collect::<String>()
        .to_ascii_uppercase();
    if matches!(palavra.as_str(), "SELECT" | "WITH" | "VALUES" | "TABLE") {
        format!(
            "SELECT * FROM ({corpo}) AS kinein_q LIMIT {}",
            u64::from(max_rows) + 1
        )
    } else {
        corpo.to_owned()
    }
}

/// O teto pedido, preso entre 1 e [`MAX_ROWS`].
#[must_use]
pub fn clamp_rows(max_rows: Option<u32>) -> u32 {
    max_rows.unwrap_or(DEFAULT_MAX_ROWS).clamp(1, MAX_ROWS)
}

/// Corta em `max_rows` e diz se cortou.
fn aplicar_teto(rows: &mut Vec<Vec<Option<String>>>, max_rows: u32) -> bool {
    let teto = max_rows as usize;
    if rows.len() > teto {
        rows.truncate(teto);
        true
    } else {
        false
    }
}

/// Roda no `PostgreSQL` pelo protocolo SIMPLES (toda coluna vem em texto).
///
/// # Errors
/// A [`ConnectionFailure`] do driver, com o `SQLSTATE` — e `secret_required`
/// quando o servidor pediu senha.
pub fn run_postgres(
    profile: &DataSourceProfile,
    secret: Option<&Secret>,
    sql: &str,
    max_rows: u32,
) -> Result<QueryResult, ConnectionFailure> {
    let mut client = connect(profile, secret)?;
    let leitura = is_read(sql);
    let texto = if leitura {
        wrap_limit(sql, max_rows)
    } else {
        sql.to_owned()
    };
    let inicio = Instant::now();
    let mensagens = if leitura {
        let mut tx = client
            .build_transaction()
            .read_only(true)
            .start()
            .map_err(|e| failure_from(&e))?;
        let m = tx.simple_query(&texto).map_err(|e| failure_from(&e))?;
        tx.commit().map_err(|e| failure_from(&e))?;
        m
    } else {
        client.simple_query(&texto).map_err(|e| failure_from(&e))?
    };
    let elapsed_ms = u64::try_from(inicio.elapsed().as_millis()).unwrap_or(u64::MAX);
    let mut resultado = QueryResult {
        elapsed_ms,
        ..QueryResult::default()
    };
    for mensagem in mensagens {
        match mensagem {
            SimpleQueryMessage::RowDescription(colunas) => {
                // O ULTIMO conjunto de resultados e' o que a tela mostra.
                resultado.columns = colunas.iter().map(|c| c.name().to_owned()).collect();
                resultado.rows.clear();
            }
            SimpleQueryMessage::Row(linha) => {
                resultado.rows.push(
                    (0..linha.len())
                        .map(|i| linha.get(i).map(str::to_owned))
                        .collect(),
                );
            }
            SimpleQueryMessage::CommandComplete(n) if !leitura => {
                resultado.affected = Some(resultado.affected.unwrap_or(0) + n);
            }
            _ => {}
        }
    }
    resultado.truncated = aplicar_teto(&mut resultado.rows, max_rows);
    Ok(resultado)
}

/// Roda no `SQLite`: leitura com o arquivo aberto so' para ler; escrita por
/// `execute_batch` e o `changes()` da ULTIMA instrucao (o `SQLite` nao soma).
///
/// # Errors
/// A falha do `rusqlite`, em texto.
pub fn run_sqlite(
    profile: &DataSourceProfile,
    sql: &str,
    max_rows: u32,
) -> Result<QueryResult, ConnectionFailure> {
    let leitura = is_read(sql);
    let flags = if leitura {
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX
    } else {
        rusqlite::OpenFlags::default()
    };
    let conexao = sqlite::abrir_com(profile, flags)?;
    let inicio = Instant::now();
    let mut resultado = QueryResult::default();
    if leitura {
        let mut stmt = conexao.prepare(sql).map_err(|e| sqlite::falha(&e))?;
        resultado.columns = stmt
            .column_names()
            .iter()
            .map(|c| (*c).to_owned())
            .collect();
        let largura = resultado.columns.len();
        let mut linhas = stmt.query([]).map_err(|e| sqlite::falha(&e))?;
        while let Some(linha) = linhas.next().map_err(|e| sqlite::falha(&e))? {
            if resultado.rows.len() == max_rows as usize {
                resultado.truncated = true;
                break;
            }
            let celulas = (0..largura)
                .map(|i| linha.get_ref(i).ok().and_then(celula_sqlite))
                .collect();
            resultado.rows.push(celulas);
        }
    } else {
        conexao.execute_batch(sql).map_err(|e| sqlite::falha(&e))?;
        resultado.affected = Some(conexao.changes());
    }
    resultado.elapsed_ms = u64::try_from(inicio.elapsed().as_millis()).unwrap_or(u64::MAX);
    Ok(resultado)
}

fn celula_sqlite(valor: ValueRef<'_>) -> Option<String> {
    match valor {
        ValueRef::Null => None,
        ValueRef::Integer(i) => Some(i.to_string()),
        ValueRef::Real(f) => Some(f.to_string()),
        ValueRef::Text(t) => Some(String::from_utf8_lossy(t).into_owned()),
        ValueRef::Blob(b) => Some(format!("<{} bytes>", b.len())),
    }
}

/// Le uma colecao do `MongoDB`: `<colecao> <filtro JSON>` (filtro ausente =
/// `{}`), `find` com `limit`; colunas = uniao das chaves de primeiro nivel.
///
/// # Errors
/// Texto sem colecao, filtro que nao e' JSON, ou a falha do driver.
pub fn run_mongo(
    profile: &DataSourceProfile,
    secret: Option<&Secret>,
    texto: &str,
    max_rows: u32,
) -> Result<QueryResult, mongo::MongoFailure> {
    let texto = texto.trim();
    let (colecao, filtro) = texto
        .split_once(char::is_whitespace)
        .unwrap_or((texto, "{}"));
    if colecao.is_empty() {
        return Err(mongo::MongoFailure {
            message: "escreva `<colecao> <filtro JSON>` — ex.: `sensores {\"placa\": \"esp32\"}`"
                .to_owned(),
            secret_required: false,
        });
    }
    let filtro: Document = serde_json::from_str::<serde_json::Value>(filtro.trim())
        .ok()
        .and_then(|v| mongodb::bson::to_document(&v).ok())
        .ok_or_else(|| mongo::MongoFailure {
            message: "o filtro precisa ser um objeto JSON (ex.: `{\"campo\": 1}`)".to_owned(),
            secret_required: false,
        })?;
    let client = mongodb::sync::Client::with_options(mongo::options_for(profile, secret))
        .map_err(|e| mongo::describe(&e, &profile.host))?;
    let inicio = Instant::now();
    let cursor = client
        .database(mongo::banco_de(profile))
        .collection::<Document>(colecao)
        .find(filtro)
        .limit(i64::from(max_rows) + 1)
        .run()
        .map_err(|e| mongo::describe(&e, &profile.host))?;
    let documentos: Vec<Document> = cursor
        .collect::<Result<_, _>>()
        .map_err(|e| mongo::describe(&e, &profile.host))?;
    let elapsed_ms = u64::try_from(inicio.elapsed().as_millis()).unwrap_or(u64::MAX);
    Ok(tabela_de_documentos(&documentos, max_rows, elapsed_ms))
}

/// Documentos -> tabela: `_id` primeiro, depois as chaves em ordem.
fn tabela_de_documentos(documentos: &[Document], max_rows: u32, elapsed_ms: u64) -> QueryResult {
    let mut chaves: Vec<String> = Vec::new();
    for doc in documentos {
        for chave in doc.keys() {
            if !chaves.iter().any(|c| c == chave) {
                chaves.push(chave.clone());
            }
        }
    }
    chaves.sort_by(|a, b| (a != "_id").cmp(&(b != "_id")).then_with(|| a.cmp(b)));
    let mut rows: Vec<Vec<Option<String>>> = documentos
        .iter()
        .map(|doc| chaves.iter().map(|c| doc.get(c).map(celula_bson)).collect())
        .collect();
    let truncated = aplicar_teto(&mut rows, max_rows);
    QueryResult {
        columns: chaves,
        rows,
        affected: None,
        truncated,
        elapsed_ms,
    }
}

fn celula_bson(valor: &Bson) -> String {
    match valor {
        Bson::String(s) => s.clone(),
        Bson::ObjectId(id) => id.to_hex(),
        Bson::Null => "null".to_owned(),
        outro => outro.clone().into_relaxed_extjson().to_string(),
    }
}

/// O motor certo para o perfil.
///
/// # Errors
/// A falha do motor como `(mensagem, secret_required)`.
pub fn run(
    profile: &DataSourceProfile,
    secret: Option<&Secret>,
    sql: &str,
    max_rows: u32,
) -> Result<QueryResult, (String, bool)> {
    match profile.engine {
        DataSourceEngine::Postgres => {
            run_postgres(profile, secret, sql, max_rows).map_err(|f| (f.message, f.secret_required))
        }
        DataSourceEngine::Sqlite => {
            run_sqlite(profile, sql, max_rows).map_err(|f| (f.message, false))
        }
        DataSourceEngine::Mongo => {
            run_mongo(profile, secret, sql, max_rows).map_err(|f| (f.message, f.secret_required))
        }
    }
}

#[cfg(test)]
mod tests {
    use kinein_protocol::{DataSourceEngine, SecretSource};

    use super::*;

    #[test]
    fn reads_are_told_by_the_first_word_after_comments() {
        assert!(is_read("select 1"));
        assert!(is_read(
            "  -- comentario\n/* outro */ WITH x AS (SELECT 1) SELECT * FROM x"
        ));
        assert!(is_read("EXPLAIN SELECT 1"));
        assert!(is_read("show server_version"));
        assert!(!is_read("INSERT INTO t VALUES (1)"));
        assert!(!is_read("PRAGMA journal_mode=WAL"));
        assert!(!is_read(""));
        assert!(!is_read("-- so' comentario"));
    }

    #[test]
    fn the_limit_wraps_one_statement_and_leaves_the_rest_alone() {
        assert_eq!(
            wrap_limit("SELECT a FROM t;", 500),
            "SELECT * FROM (SELECT a FROM t) AS kinein_q LIMIT 501"
        );
        assert_eq!(
            wrap_limit("-- c\nvalues (1)", 2),
            "SELECT * FROM (values (1)) AS kinein_q LIMIT 3"
        );
        assert_eq!(wrap_limit("SHOW all", 5), "SHOW all");
        assert_eq!(wrap_limit("SELECT 1; SELECT 2", 5), "SELECT 1; SELECT 2");
        assert_eq!(single_statement("  ; "), None);
        assert_eq!(clamp_rows(None), 500);
        assert_eq!(clamp_rows(Some(0)), 1);
        assert_eq!(clamp_rows(Some(1_000_000)), MAX_ROWS);
    }

    fn perfil_sqlite(caminho: &std::path::Path) -> DataSourceProfile {
        DataSourceProfile {
            name: "arquivo".to_owned(),
            engine: DataSourceEngine::Sqlite,
            host: String::new(),
            port: 0,
            database: caminho.display().to_string(),
            user: String::new(),
            secret_source: SecretSource::Automatic,
            secret_variable: None,
            sample_size: None,
            tls: None,
            ca_file: None,
        }
    }

    fn banco_temporario(nome: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join("kinein-datasource-query");
        std::fs::create_dir_all(&dir).unwrap();
        let caminho = dir.join(format!("{}-{nome}.db", std::process::id()));
        let _ = std::fs::remove_file(&caminho);
        let conexao = rusqlite::Connection::open(&caminho).unwrap();
        conexao
            .execute_batch("CREATE TABLE leituras (id INTEGER PRIMARY KEY, placa TEXT, valor REAL, bruto BLOB); \
                            INSERT INTO leituras (placa, valor, bruto) VALUES ('esp32', 21.5, X'0102'), ('pico', NULL, NULL), ('pi', 3, NULL);")
            .unwrap();
        caminho
    }

    #[test]
    fn sqlite_reads_with_a_ceiling_and_writes_with_a_count() {
        let caminho = banco_temporario("leitura");
        let perfil = perfil_sqlite(&caminho);
        let r = run_sqlite(
            &perfil,
            "SELECT id, placa, valor, bruto FROM leituras ORDER BY id",
            2,
        )
        .unwrap();
        assert_eq!(r.columns, ["id", "placa", "valor", "bruto"]);
        assert_eq!(r.rows.len(), 2);
        assert!(r.truncated);
        assert_eq!(
            r.rows[0],
            [
                Some("1".to_owned()),
                Some("esp32".to_owned()),
                Some("21.5".to_owned()),
                Some("<2 bytes>".to_owned())
            ]
        );
        assert_eq!(r.rows[1][2], None);
        assert_eq!(r.affected, None);

        // A leitura que tenta escrever e' recusada pelo MOTOR (arquivo so' leitura).
        let erro = run_sqlite(
            &perfil,
            "WITH x AS (SELECT 1) INSERT INTO leituras (placa) SELECT 'nunca' FROM x",
            10,
        )
        .unwrap_err();
        assert!(erro.message.contains("readonly"), "{}", erro.message);
        assert_eq!(
            run_sqlite(&perfil, "SELECT count(*) FROM leituras", 10)
                .unwrap()
                .rows[0][0]
                .as_deref(),
            Some("3")
        );

        let escrita = run_sqlite(&perfil, "UPDATE leituras SET valor = 0 WHERE valor IS NULL; DELETE FROM leituras WHERE placa = 'pi'", 10).unwrap();
        assert_eq!(escrita.affected, Some(1));
        assert!(escrita.columns.is_empty());
        let depois = run_sqlite(&perfil, "SELECT count(*) FROM leituras", 10).unwrap();
        assert_eq!(depois.rows[0][0], Some("2".to_owned()));
        assert!(!depois.truncated);
        let ruim = run_sqlite(&perfil, "SELECT nada FROM lugar_nenhum", 10).unwrap_err();
        assert!(ruim.message.contains("lugar_nenhum"), "{}", ruim.message);
    }

    #[test]
    fn mongo_text_is_collection_plus_json_filter_and_documents_become_a_table() {
        let docs = vec![
            mongodb::bson::doc! { "_id": 1, "placa": "esp32", "meta": { "v": 2 } },
            mongodb::bson::doc! { "_id": 2, "temp": 21.5 },
            mongodb::bson::doc! { "_id": 3 },
        ];
        let t = tabela_de_documentos(&docs, 2, 7);
        assert_eq!(t.columns, ["_id", "meta", "placa", "temp"]);
        assert_eq!(t.rows.len(), 2);
        assert!(t.truncated);
        assert_eq!(t.rows[0][2].as_deref(), Some("esp32"));
        assert_eq!(t.rows[0][1].as_deref(), Some("{\"v\":2}"));
        assert_eq!(t.rows[1][3].as_deref(), Some("21.5"));
        assert_eq!(t.rows[1][1], None);
        assert_eq!(t.elapsed_ms, 7);
        let perfil = DataSourceProfile {
            engine: DataSourceEngine::Mongo,
            host: "localhost".to_owned(),
            port: 1,
            database: "db".to_owned(),
            ..perfil_sqlite(std::path::Path::new("x"))
        };
        assert!(
            run_mongo(&perfil, None, "", 10)
                .unwrap_err()
                .message
                .contains("<colecao>")
        );
        assert!(
            run_mongo(&perfil, None, "col nao-json", 10)
                .unwrap_err()
                .message
                .contains("objeto JSON")
        );
    }
}

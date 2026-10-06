//! Console ODBC: dialeto desconhecido, resultado limitado e aviso generico.

use super::connection::ConnectionFailure;
use super::{odbc, query::QueryResult, secret::Secret};
use kinein_protocol::{DataSourceProfile, SqlImpactSeverity, SqlStatementImpact};
use std::time::Instant;

/// ODBC nao herda a classificacao otimista de um motor nativo.
/// Uma leitura simples e unica e' reconhecida; o resto exige o aviso generico.
#[must_use]
pub fn classify(text: &str) -> Vec<SqlStatementImpact> {
    let masked = mask(text);
    let read = masked.as_ref().is_some_and(|masked| {
        let single = !masked.trim_end().trim_end_matches(';').contains(';');
        let tokens: Vec<_> = masked
            .split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
            .filter(|s| !s.is_empty())
            .collect();
        let forbidden = [
            "INTO",
            "INSERT",
            "UPDATE",
            "DELETE",
            "CREATE",
            "DROP",
            "ALTER",
            "MERGE",
            "EXEC",
            "CALL",
            "COMMIT",
            "COPY",
            "LOAD",
            "NEXT",
            "PREVIOUS",
            "NEXTVAL",
            "CURRVAL",
            "OUTFILE",
            "DUMPFILE",
            "PROCEDURE",
            "LOCK",
        ];
        single
            && tokens
                .first()
                .is_some_and(|s| s.eq_ignore_ascii_case("SELECT"))
            && !masked.contains('(')
            && !masked.contains(['$', '{', '}'])
            && !tokens
                .iter()
                .any(|t| forbidden.iter().any(|f| t.eq_ignore_ascii_case(f)))
    });
    vec![SqlStatementImpact {
            text: text.trim().to_owned(),
            kind: if read { "read" } else { "other" }.to_owned(),
            severity: if read { SqlImpactSeverity::Read } else { SqlImpactSeverity::Destructive },
            note: (!read).then(|| "ODBC: alcance desconhecido; nenhuma consulta de contagem foi enviada. Confirme pelo nome da conexão.".to_owned()),
            ..SqlStatementImpact::default()
        }]
}

// Dialeto desconhecido: comentarios executaveis, escapes e aspas incompletas
// exigem aviso. Identificadores delimitados nao viram comandos ou separadores.
fn mask(sql: &str) -> Option<String> {
    let bytes = sql.as_bytes();
    let mut out = bytes.to_vec();
    let mut i = 0;
    while i < bytes.len() {
        let rest = &bytes[i..];
        let end = if rest.starts_with(b"--") {
            i + rest
                .iter()
                .position(|&b| matches!(b, b'\n' | b'\r'))
                .unwrap_or(rest.len())
        } else if rest.starts_with(b"/*") {
            if rest.starts_with(b"/*!") || rest.starts_with(b"/*+") {
                return None;
            }
            let end = i + 2 + rest[2..].windows(2).position(|w| w == b"*/")?;
            if bytes[i + 2..end].windows(2).any(|w| w == b"/*") {
                return None;
            }
            end + 2
        } else if matches!(bytes[i], b'\'' | b'"' | b'`' | b'[') {
            let close = if bytes[i] == b'[' { b']' } else { bytes[i] };
            let mut j = i + 1;
            loop {
                let c = *bytes.get(j)?;
                if c == b'\\' {
                    return None;
                }
                if c == close {
                    if bytes.get(j + 1) != Some(&close) {
                        break;
                    }
                    j += 1;
                }
                j += 1;
            }
            j + 1
        } else {
            i += 1;
            continue;
        };
        out[i..end].fill(b' ');
        i = end;
    }
    String::from_utf8(out).ok()
}

/// Executa depois da autorizacao de driver e da politica de confirmacao do core.
pub fn run(
    profile: &DataSourceProfile,
    secret: Option<&Secret>,
    sql: &str,
    max_rows: u32,
) -> Result<QueryResult, ConnectionFailure> {
    let read = classify(sql)
        .iter()
        .all(|s| s.severity == SqlImpactSeverity::Read);
    super::policy::check_read_only(profile, !read).map_err(|rejection| ConnectionFailure {
        message: rejection.message.to_owned(),
        sql_state: None,
        secret_required: false,
    })?;
    let start = Instant::now();
    let connection = odbc::connect(profile, secret)?;
    if read {
        connection
            .set_autocommit(false)
            .map_err(|e| odbc::failure(&e, "preparar a leitura sem commit"))?;
    }
    let outcome = execute(&connection, sql, max_rows);
    if read {
        connection
            .rollback()
            .map_err(|e| odbc::failure(&e, "encerrar a leitura sem commit"))?;
    }
    outcome.map(|mut result| {
        result.elapsed_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
        result
    })
}

fn execute(
    connection: &odbc_api::Connection<'_>,
    sql: &str,
    max_rows: u32,
) -> Result<QueryResult, ConnectionFailure> {
    let mut statement = connection
        .preallocate()
        .map_err(|e| odbc::failure(&e, "preparar o comando"))?;
    statement
        .set_query_timeout_sec(5)
        .map_err(|e| odbc::failure(&e, "limitar o tempo da consulta"))?;
    if let Some(cursor) = statement
        .execute(sql, ())
        .map_err(|e| odbc::failure(&e, "executar o comando"))?
    {
        return super::odbc_rows::read(cursor, max_rows);
    }

    let affected = statement
        .row_count()
        .map_err(|e| odbc::failure(&e, "ler a contagem afetada"))?
        .and_then(|count| u64::try_from(count).ok());
    Ok(QueryResult {
        columns: Vec::new(),
        rows: Vec::new(),
        affected,
        truncated: false,
        elapsed_ms: 0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generic_impact_does_not_mistake_ctes_batches_or_functions_for_safe_reads() {
        for text in [
            "SELECT * FROM leituras",
            "SELECT 1",
            "-- leitura\nSELECT 'DROP INTO' AS texto",
            "SELECT * FROM \"x\"\"; DROP TABLE t;--\"",
            "SELECT * FROM [x]]; DROP TABLE t]",
            "SELECT * FROM `x``; DROP TABLE t`",
        ] {
            assert_eq!(
                classify(text)[0].severity,
                SqlImpactSeverity::Read,
                "{text}"
            );
        }
        for text in [
            "SELECT * INTO copia FROM t",
            "WITH x AS (DELETE FROM t RETURNING *) SELECT * FROM x",
            "SELECT mutate()",
            "SELECT 1; COMMIT; DROP TABLE t",
            "INSERT INTO t VALUES (1)",
            "UPDATE t SET x=1 WHERE id=2",
            "SELECT COUNT(*) FROM t",
            "SELECT NEXT VALUE FOR sequencia",
            "SELECT sequencia.NEXTVAL FROM dual",
            "SELECT 1 /*! INTO OUTFILE '/tmp/saida' */",
            "SELECT 1 /* comentario sem fim",
        ] {
            let impacts = classify(text);
            assert!(
                impacts
                    .iter()
                    .any(|s| s.severity == SqlImpactSeverity::Destructive),
                "{text}"
            );
            assert!(
                impacts
                    .iter()
                    .all(|s| s.rows.is_none() && s.total_rows.is_none())
            );
        }
    }
}

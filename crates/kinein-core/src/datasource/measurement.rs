//! Medir o impacto sem escrever (2026-10-05): dono da contagem para o
//! dialogo e para a verificacao silenciosa de alteracoes filtradas.
//! A rede espera num job; nenhum handler conta no laco de despacho.

use super::secret::Secret;
use super::{impact, mongo_command, mongo_write, query};
use kinein_protocol::{DataSourceEngine, DataSourceProfile, SqlImpactSeverity, SqlStatementImpact};

/// Classifica e mede: erro de contagem vira nota; nada aqui executa escrita.
#[must_use]
pub fn statements(
    profile: &DataSourceProfile,
    secret: Option<&Secret>,
    text: &str,
) -> Vec<SqlStatementImpact> {
    let measured = if profile.engine == DataSourceEngine::Mongo {
        measure_mongo(profile, secret, text)
    } else {
        impact::classify_all(text)
            .into_iter()
            .map(|s| measure(profile, secret, s))
            .collect()
    };
    measured
        .into_iter()
        .map(|mut s| {
            // Uma falha nao pode liberar uma operacao cujo alcance e' desconhecido.
            if s.note.is_some() || s.kind == "other" {
                s.severity = SqlImpactSeverity::Destructive;
            }
            s
        })
        .collect()
}

/// O Mongo (`0.155.0`): um comando por vez; texto invalido vira uma nota, e
/// nada vai ao banco.
fn measure_mongo(
    profile: &kinein_protocol::DataSourceProfile,
    secret: Option<&Secret>,
    text: &str,
) -> Vec<SqlStatementImpact> {
    match mongo_command::parse(text) {
        Ok(command) => {
            let statement = mongo_command::impact(&command, text);
            vec![mongo_write::measure(profile, secret, &command, statement)]
        }
        Err(message) => vec![SqlStatementImpact {
            text: text.trim().to_owned(),
            kind: "other".to_owned(),
            note: Some(message),
            ..SqlStatementImpact::default()
        }],
    }
}

/// Roda as contagens de uma instrucao e promove o `WHERE` que pega tudo.
fn measure(
    profile: &kinein_protocol::DataSourceProfile,
    secret: Option<&Secret>,
    mut statement: SqlStatementImpact,
) -> SqlStatementImpact {
    let Some((hit, total)) = impact::count_queries(&statement) else {
        return statement;
    };
    match count(profile, secret, &hit) {
        Ok(rows) => statement.rows = Some(rows),
        Err(message) => {
            statement.note = Some(message);
            return statement;
        }
    }
    if let Some(total) = total {
        match count(profile, secret, &total) {
            Ok(rows) => statement.total_rows = Some(rows),
            Err(message) => statement.note = Some(message),
        }
    }
    // `WHERE 1=1`, um filtro esquecido: pega TODAS as linhas de uma tabela
    // que tem linhas — e' tao destrutivo quanto nao ter WHERE.
    if let (Some(rows), Some(total)) = (statement.rows, statement.total_rows)
        && total > 0
        && rows == total
    {
        statement.severity = SqlImpactSeverity::Destructive;
    }
    statement
}

/// Uma contagem: a primeira celula da primeira linha, como numero.
fn count(
    profile: &kinein_protocol::DataSourceProfile,
    secret: Option<&Secret>,
    sql: &str,
) -> Result<u64, String> {
    let result = query::run(profile, secret, sql, 1).map_err(|(message, _)| message)?;
    result
        .rows
        .first()
        .and_then(|row| row.first().cloned().flatten())
        .and_then(|cell| cell.trim().parse::<u64>().ok())
        .ok_or_else(|| "o motor nao devolveu a contagem".to_owned())
}

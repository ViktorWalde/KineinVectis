//! Classificação por motor sem criar dependência circular com a execução.

use kinein_protocol::{SqlImpactSeverity, SqlStatementImpact};

/// Classificação completa por motor, sem conexão e sem resolução de segredo.
#[must_use]
pub fn classify(engine: kinein_protocol::DataSourceEngine, text: &str) -> Vec<SqlStatementImpact> {
    match engine {
        kinein_protocol::DataSourceEngine::Odbc => super::odbc_query::classify(text),
        kinein_protocol::DataSourceEngine::Mongo => super::mongo_command::parse(text).map_or_else(
            |_| {
                vec![SqlStatementImpact {
                    text: text.to_owned(),
                    kind: "other".to_owned(),
                    severity: SqlImpactSeverity::Destructive,
                    ..SqlStatementImpact::default()
                }]
            },
            |command| vec![super::mongo_command::impact(&command, text)],
        ),
        _ => super::impact::classify_all(text),
    }
}

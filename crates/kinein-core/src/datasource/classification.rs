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

/// Whether an attempted execution can leave the catalogue stale.
///
/// Uses the existing engine classification, never a second SQL parser.
/// Unknown relational instructions are conservative; Mongo needs a valid command.
#[must_use]
pub fn invalidates_catalog(
    engine: kinein_protocol::DataSourceEngine,
    statements: &[SqlStatementImpact],
) -> bool {
    use kinein_protocol::DataSourceEngine;
    statements.iter().any(|statement| match engine {
        DataSourceEngine::Mongo => {
            statement.kind != "other" && statement.severity != SqlImpactSeverity::Read
        }
        DataSourceEngine::Odbc => statement.severity != SqlImpactSeverity::Read,
        DataSourceEngine::Postgres | DataSourceEngine::Sqlite => {
            matches!(statement.kind.as_str(), "create" | "alter" | "other")
                || statement.kind.starts_with("drop")
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use kinein_protocol::DataSourceEngine;

    #[test]
    fn catalogue_invalidation_uses_real_statement_boundaries() {
        for engine in [DataSourceEngine::Postgres, DataSourceEngine::Sqlite] {
            for sql in [
                "-- SELECT\nCREATE TABLE \"objeto; DELETE\"(id int)",
                "SELECT 'CREATE; DROP'; ALTER TABLE t ADD COLUMN x int",
                "DROP VIEW \"a; b\"",
                "ALTER SCHEMA old RENAME TO new",
                "CALL unknown_effect()",
                "CREATE TABLE parcial(id); invalid command",
            ] {
                assert!(invalidates_catalog(engine, &classify(engine, sql)), "{sql}");
            }
            for sql in [
                "SELECT 'CREATE TABLE t'; -- DROP TABLE t",
                "SELECT \"DROP\", [ALTER], `CREATE` FROM t",
                "INSERT INTO t VALUES ('DROP; CREATE')",
                "UPDATE t SET x = 'ALTER' WHERE id = 1",
                "DELETE FROM t WHERE id = 1",
                "TRUNCATE TABLE t",
            ] {
                assert!(
                    !invalidates_catalog(engine, &classify(engine, sql)),
                    "{sql}"
                );
            }
        }
    }

    #[test]
    fn mongo_writes_refresh_sampled_fields_and_odbc_keeps_unknown_dialect() {
        for text in [
            "c.insertOne({\"x\":1})",
            "c.updateOne({}, {\"$set\":{\"y\":1}})",
            "c.deleteMany({})",
            "c.drop()",
        ] {
            assert!(invalidates_catalog(
                DataSourceEngine::Mongo,
                &classify(DataSourceEngine::Mongo, text)
            ));
        }
        for text in ["c.find({})", "c {}", "c.insertOne(<invalid>)"] {
            assert!(!invalidates_catalog(
                DataSourceEngine::Mongo,
                &classify(DataSourceEngine::Mongo, text)
            ));
        }
        for (text, expected) in [
            ("SELECT * FROM [db].[t]", false),
            ("CREATE TABLE [t](id int)", true),
            ("EXEC unknown_effect", true),
        ] {
            assert_eq!(
                invalidates_catalog(
                    DataSourceEngine::Odbc,
                    &classify(DataSourceEngine::Odbc, text)
                ),
                expected
            );
        }
    }
}

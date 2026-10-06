//! QUANDO a escrita pede confirmacao (`0.156.0`, decisao do autor em
//! 2026-10-04): so' o que REMOVE dados.
//!
//! Ate' o `0.155.0` toda escrita parava no aviso. O autor, usando o console:
//! "toda vez que tiver que escrever um comando e aparecer o pop-up fica muito
//! inconveniente … no maximo os avisos de apagar". Agora:
//!
//! - roda direto: inserir, alterar COM filtro, criar, mudar estrutura sem
//!   remover nada;
//! - pede o aviso: apagar linhas/documentos, esvaziar, remover tabela, coluna,
//!   esquema, banco, visao, indice ou colecao, e alterar TUDO (sem filtro);
//! - pede o aviso tambem o que a IDE nao sabe classificar (`other`: um
//!   `CALL`, um bloco `DO`): recusa por padrao, pela seguranca (59 §7).
//!
//! A decisao e' do core, pura, sobre a classificacao — a tela nunca decide.

use kinein_protocol::{SqlImpactSeverity, SqlStatementImpact};

/// Os tipos que removem dados ou objetos.
const REMOVES: [&str; 13] = [
    "delete",
    "replace",
    "truncate",
    "dropTable",
    "dropView",
    "dropIndex",
    "dropSchema",
    "dropDatabase",
    "dropColumn",
    "drop",
    "mongoDelete",
    "dropCollection",
    "other",
];

/// `true` quando alguma instrucao precisa do aviso antes de rodar.
#[must_use]
pub fn needs_confirmation(statements: &[SqlStatementImpact]) -> bool {
    statements.iter().any(|statement| {
        statement.severity == SqlImpactSeverity::Destructive
            || REMOVES.contains(&statement.kind.as_str())
    })
}

/// Alteracoes filtradas medem em silencio antes de rodar: ter filtro nao
/// garante que o filtro deixa algum registro de fora.
#[must_use]
pub fn needs_measurement(statements: &[SqlStatementImpact]) -> bool {
    statements
        .iter()
        .any(|s| matches!(s.kind.as_str(), "update" | "mongoUpdate") && !s.filter.is_empty())
}

#[cfg(test)]
mod tests {
    use super::needs_confirmation;
    use crate::datasource::impact::classify_all;
    use crate::datasource::mongo_command::{impact, parse};

    fn sql(text: &str) -> bool {
        needs_confirmation(&classify_all(text))
    }

    fn mongo(text: &str) -> bool {
        needs_confirmation(&[impact(&parse(text).unwrap(), text)])
    }

    #[test]
    fn writing_runs_straight_and_removing_asks() {
        // Roda direto.
        assert!(!sql("INSERT INTO t (a) VALUES (1)"));
        assert!(!sql("UPDATE t SET a = 2 WHERE id = 7"));
        assert!(!sql("CREATE TABLE x (id int)"));
        assert!(!sql("ALTER TABLE t ADD COLUMN b int"));
        assert!(!mongo(r#"s.insertOne({"a": 1})"#));
        assert!(!mongo(r#"s.updateMany({"a": 1}, {"$set": {"b": 2}})"#));
        // Pede o aviso.
        assert!(sql("DELETE FROM t WHERE id = 7"));
        assert!(sql("UPDATE t SET a = 2"), "alterar tudo sem filtro");
        assert!(sql("TRUNCATE t"));
        assert!(sql("DROP TABLE t"));
        assert!(sql("ALTER TABLE t DROP COLUMN b"));
        assert!(sql("CALL limpa_tudo()"), "o que a IDE nao classifica");
        assert!(
            sql("INSERT INTO t VALUES (1); DELETE FROM t"),
            "uma remocao no lote basta"
        );
        assert!(mongo(r#"s.deleteOne({"a": 1})"#));
        assert!(mongo(r#"s.updateMany({}, {"$set": {"b": 2}})"#));
        assert!(mongo("s.drop()"));
    }
}

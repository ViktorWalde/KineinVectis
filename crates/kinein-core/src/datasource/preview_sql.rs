//! `PostgreSQL` preview eligibility and RETURNING composition use the shared lexer.

use super::sql_syntax::{lex, words};

/// One direct INSERT/UPDATE/DELETE, without an ambiguous or extra statement.
/// This is a conservative eligibility rule, not a SQL parser or permission boundary.
#[must_use]
pub fn executed_sql(sql: &str) -> Option<String> {
    let scan = lex(sql);
    if !scan.valid {
        return None;
    }
    let end = scan.boundaries.first().copied().unwrap_or(sql.len());
    if scan.boundaries.len() > 1
        || !scan.boundaries.is_empty() && scan.code_end > end + 1
        || !scan.masked[end.saturating_add(1).min(sql.len())..]
            .trim()
            .is_empty()
    {
        return None;
    }
    let tokens = words(&sql[..end], &scan.masked[..end]);
    let first = tokens.first()?;
    if !["INSERT", "UPDATE", "DELETE"]
        .iter()
        .any(|keyword| first.is(keyword))
    {
        return None;
    }
    if tokens.iter().any(|word| {
        [
            "BEGIN",
            "START",
            "END",
            "COMMIT",
            "ROLLBACK",
            "SAVEPOINT",
            "RELEASE",
            "PREPARE",
            "EXECUTE",
            "CALL",
            "COPY",
            "VACUUM",
            "CREATE",
            "ALTER",
            "DROP",
            "TRUNCATE",
            "MERGE",
        ]
        .iter()
        .any(|keyword| word.is(keyword))
    }) {
        return None;
    }
    let returning = tokens
        .iter()
        .any(|word| word.depth == 0 && word.is("RETURNING"));
    if returning {
        return Some(sql[..end].to_owned());
    }
    // Insert before trailing comments (including -- without a final newline).
    // The common lexer keeps the end of literals as well as visible tokens.
    let insert_at = scan.code_end.min(end);
    Some(format!(
        "{} RETURNING *{}",
        &sql[..insert_at],
        &sql[insert_at..end]
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_dml_preserves_literals_comments_and_existing_returning() {
        assert_eq!(
            executed_sql("UPDATE t SET v = 4 WHERE id = 1; -- fim"),
            Some("UPDATE t SET v = 4 WHERE id = 1 RETURNING *".to_owned())
        );
        assert_eq!(
            executed_sql("DELETE FROM t WHERE id=1 -- sem LF"),
            Some("DELETE FROM t WHERE id=1 RETURNING * -- sem LF".to_owned())
        );
        assert_eq!(
            executed_sql("INSERT INTO t VALUES ('; RETURNING COMMIT') /* fim */"),
            Some("INSERT INTO t VALUES ('; RETURNING COMMIT') RETURNING * /* fim */".to_owned())
        );
        assert_eq!(
            executed_sql("UPDATE t SET v=s.v FROM s WHERE t.id=s.id RETURNING t.id;"),
            Some("UPDATE t SET v=s.v FROM s WHERE t.id=s.id RETURNING t.id".to_owned())
        );
        assert_eq!(
            executed_sql("INSERT INTO t VALUES ($tag$; COMMIT$tag$)"),
            Some("INSERT INTO t VALUES ($tag$; COMMIT$tag$) RETURNING *".to_owned())
        );
        assert_eq!(
            executed_sql("UPDATE t SET v='á' -- fim"),
            Some("UPDATE t SET v='á' RETURNING * -- fim".to_owned())
        );
    }

    #[test]
    fn rejects_batches_control_ambiguous_quotes_and_unsupported_commands() {
        for sql in [
            "SELECT 1",
            "WITH q AS (DELETE FROM t RETURNING *) SELECT * FROM q",
            "CREATE DATABASE d",
            "UPDATE t SET v=1; COMMIT",
            "DELETE FROM t; SELECT 1",
            "DELETE FROM t;;",
            "UPDATE t SET v=1; 'a second statement masked as a literal'",
            "INSERT INTO t VALUES ('x\\'); COMMIT; --')",
            "DELETE FROM t /*",
            "UPDATE t SET v=1 -- fim\r;COMMIT",
            "DELETE FROM t RETURNING 1 AS é$$; COMMIT; SELECT 1 AS fim$$",
        ] {
            assert!(executed_sql(sql).is_none(), "{sql}");
        }
    }
}

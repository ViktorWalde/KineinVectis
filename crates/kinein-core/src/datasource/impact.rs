//! O IMPACTO de uma escrita, medido antes de ela rodar.
//!
//! Pedido do autor (2026-10-03): "quando for um delete muito destrutivo,
//! aparecer um painel perguntando se o usuario quer executar, exibindo o
//! comando e a consequencia" — porque "ja' ocorreu e ocorre do desenvolvedor
//! apagar o banco de dados inteiro sem ter essa intencao".
//!
//! ```text
//! texto ──split──▶ instrucoes ──classify──▶ { kind, targets, filtro, gravidade }
//!                                             │
//!                       count_queries ◀───────┘   SELECT count(*) ... (LEITURA:
//!                                                 roda no caminho read-only)
//! ```
//!
//! Gravidade:
//!
//! | kind                                   | gravidade      |
//! | -------------------------------------- | -------------- |
//! | `delete`/`update` SEM `WHERE`          | `destructive`  |
//! | `truncate`, `dropTable`, `dropSchema`, `dropDatabase`, `dropColumn`, `drop` | `destructive` |
//! | `delete`/`update` com `WHERE`, `insert`, `create`, `dropView`, `dropIndex`, `alter`, `other` | `write` |
//! | leitura                                | `read`         |
//!
//! Puro: nada aqui fala com o banco. O job (`handlers/datasource_impact.rs`)
//! roda as contagens e promove a `destructive` o `WHERE` que pega TODAS as
//! linhas.

use kinein_protocol::{SqlImpactSeverity, SqlStatementImpact};

use super::sql_syntax::{self, Word, is_read, words};

/// As instrucoes do texto, separadas por `;` FORA de string (`'...'`),
/// identificador (`"..."`), comentario (`--`, `/* */`) e bloco `$$...$$`.
/// Vazias (so' espaco ou comentario) ficam de fora.
#[must_use]
pub fn split_statements(sql: &str) -> Vec<String> {
    let parsed = sql_syntax::lex(sql);
    if !parsed.valid {
        return vec![sql.trim().to_owned()];
    }
    let mut out = Vec::new();
    let mut start = 0;
    for index in parsed.boundaries {
        push_statement(&mut out, &sql[start..index], &parsed.masked[start..index]);
        start = index + 1;
    }
    push_statement(&mut out, &sql[start..], &parsed.masked[start..]);
    out
}

fn push_statement(out: &mut Vec<String>, text: &str, masked: &str) {
    if !masked.trim().is_empty() {
        out.push(text.trim().to_owned());
    }
}

/// Um alvo que vira SQL de contagem: so' nome (`a`, `a.b`, `"A"."b"`).
fn plain_target(name: &str) -> bool {
    !name.is_empty()
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'.' | b'"' | b'$'))
}

/// A gravidade de um tipo de instrucao.
#[must_use]
pub fn severity_of(kind: &str, filtered: bool) -> SqlImpactSeverity {
    match kind {
        "read" => SqlImpactSeverity::Read,
        "delete" | "update" if !filtered => SqlImpactSeverity::Destructive,
        "replace" | "truncate" | "dropTable" | "dropSchema" | "dropDatabase" | "dropColumn"
        | "drop" => SqlImpactSeverity::Destructive,
        _ => SqlImpactSeverity::Write,
    }
}

/// Classifica UMA instrucao (sem contagem: `rows` e `total_rows` vazios).
#[must_use]
pub fn classify(statement: &str) -> SqlStatementImpact {
    let parsed = sql_syntax::lex(statement);
    if !parsed.valid {
        return SqlStatementImpact {
            text: statement.trim().to_owned(),
            kind: "other".to_owned(),
            severity: SqlImpactSeverity::Destructive,
            note: Some(
                "Não foi possível determinar com segurança as fronteiras desta instrução."
                    .to_owned(),
            ),
            ..SqlStatementImpact::default()
        };
    }
    let list = words(statement, &parsed.masked);
    let word = |i: usize| list.get(i).map_or("", |w| w.text);
    let is = |i: usize, k: &str| list.get(i).is_some_and(|w| w.is(k));
    let mut kind = "other";
    let mut targets: Vec<String> = Vec::new();
    let mut filter = String::new();
    let mut column = String::new();
    if is_read(statement) {
        kind = "read";
    } else if is(0, "delete") {
        kind = "delete";
        let at = if is(1, "from") { 2 } else { 1 };
        targets.push(word(at).to_owned());
        filter = where_clause(statement, &list, at + 1);
    } else if is(0, "update") {
        kind = if is(1, "or") && is(2, "replace") {
            "replace"
        } else {
            "update"
        };
        // `UPDATE ONLY t` (PostgreSQL) e `UPDATE OR REPLACE t` (SQLite).
        let at = if is(1, "or") {
            3
        } else if is(1, "only") {
            2
        } else {
            1
        };
        targets.push(word(at).to_owned());
        filter = where_clause(statement, &list, at + 1);
    } else if is(0, "truncate") {
        kind = "truncate";
        targets = names_after(&list, 1, &["table", "only"]);
    } else if is(0, "drop") {
        let (found, at) = drop_kind(&list);
        kind = found;
        targets = names_after(&list, at, &["if", "exists", "concurrently"]);
    } else if is(0, "alter") && is(1, "table") {
        let at = if is(2, "if") { 4 } else { 2 };
        let at = if list.get(at).is_some_and(|w| w.is("only")) {
            at + 1
        } else {
            at
        };
        targets.push(word(at).to_owned());
        kind = "alter";
        if let Some(drop) = list.iter().skip(at + 1).position(|w| w.is("drop")) {
            let next = at + 2 + drop;
            if !is(next, "constraint") && !is(next, "default") && !is(next, "not") {
                kind = "dropColumn";
                let name_at = if is(next, "column") { next + 1 } else { next };
                let name_at = if is(name_at, "if") {
                    name_at + 2
                } else {
                    name_at
                };
                word(name_at).clone_into(&mut column);
            }
        }
    } else if is(0, "insert") || is(0, "replace") {
        kind = if is(0, "replace") || (is(1, "or") && is(2, "replace")) {
            "replace"
        } else {
            "insert"
        };
        if let Some(into) = list.iter().position(|w| w.is("into")) {
            targets.push(word(into + 1).to_owned());
        }
    } else if is(0, "create") {
        kind = "create";
    }
    let filtered = !filter.is_empty();
    SqlStatementImpact {
        text: statement.trim().to_owned(),
        kind: kind.to_owned(),
        targets,
        column,
        filter,
        severity: severity_of(kind, filtered),
        rows: None,
        total_rows: None,
        note: None,
    }
}

/// `DROP <o que>`: o tipo e onde comecam os nomes.
fn drop_kind(list: &[Word<'_>]) -> (&'static str, usize) {
    let is = |i: usize, k: &str| list.get(i).is_some_and(|w| w.is(k));
    if is(1, "table") {
        ("dropTable", 2)
    } else if is(1, "view") {
        ("dropView", 2)
    } else if is(1, "materialized") && is(2, "view") {
        ("dropView", 3)
    } else if is(1, "index") {
        ("dropIndex", 2)
    } else if is(1, "schema") {
        ("dropSchema", 2)
    } else if is(1, "database") {
        ("dropDatabase", 2)
    } else {
        ("drop", 2)
    }
}

/// Os nomes depois de `at`, pulando as palavras de ligacao, ate' a primeira
/// palavra que nao e' nome (`CASCADE`, `RESTRICT`...).
fn names_after(list: &[Word<'_>], at: usize, skip: &[&str]) -> Vec<String> {
    list.iter()
        .skip(at)
        .skip_while(|w| skip.iter().any(|k| w.is(k)))
        .take_while(|w| {
            ![
                "cascade", "restrict", "with", "continue", "restart", "identity",
            ]
            .iter()
            .any(|k| w.is(k))
        })
        .map(|w| w.text.to_owned())
        .collect()
}

/// O texto do `WHERE` de TOPO (depois de `from`), ate' `RETURNING` ou o fim.
/// Um `WHERE` dentro de subconsulta (`SET x = (SELECT ... WHERE ...)`) nao e'
/// o filtro da instrucao: conta-lo faria um UPDATE que pega tudo parecer
/// inofensivo — o lado perigoso do erro.
fn where_clause(statement: &str, list: &[Word<'_>], from: usize) -> String {
    let Some(at) = list
        .iter()
        .skip(from)
        .position(|w| w.depth == 0 && w.is("where"))
    else {
        return String::new();
    };
    let start = list[from + at].start + "where".len();
    let end = list
        .iter()
        .skip(from + at + 1)
        .find(|w| w.depth == 0 && w.is("returning"))
        .map_or(statement.len(), |w| w.start);
    statement[start..end]
        .trim()
        .trim_end_matches(';')
        .trim()
        .to_owned()
}

/// O texto inteiro, instrucao por instrucao.
#[must_use]
pub fn classify_all(sql: &str) -> Vec<SqlStatementImpact> {
    split_statements(sql).iter().map(|s| classify(s)).collect()
}

/// A maior gravidade entre as instrucoes.
#[must_use]
pub fn overall(statements: &[SqlStatementImpact]) -> SqlImpactSeverity {
    statements
        .iter()
        .map(|s| s.severity)
        .max()
        .unwrap_or(SqlImpactSeverity::Read)
}

/// As contagens (leitura pura) que medem a consequencia de uma instrucao:
/// `(linhas atingidas, linhas da tabela)`. `None` quando nao ha' o que contar
/// ou o alvo nao e' um nome simples.
#[must_use]
pub fn count_queries(statement: &SqlStatementImpact) -> Option<(String, Option<String>)> {
    let target = statement.targets.first()?;
    if !plain_target(target) {
        return None;
    }
    match statement.kind.as_str() {
        "delete" | "update" => {
            let total = format!("SELECT count(*) FROM {target}");
            if statement.filter.is_empty() {
                Some((total, None))
            } else {
                Some((format!("{total} WHERE {}", statement.filter), Some(total)))
            }
        }
        "truncate" | "dropTable" => {
            let all = statement
                .targets
                .iter()
                .filter(|t| plain_target(t))
                .map(|t| format!("(SELECT count(*) FROM {t})"))
                .collect::<Vec<_>>()
                .join(" + ");
            Some((format!("SELECT {all}"), None))
        }
        "dropColumn" if plain_target(&statement.column) => Some((
            format!(
                "SELECT count(*) FROM {target} WHERE {} IS NOT NULL",
                statement.column
            ),
            None,
        )),
        "dropSchema" => Some((
            format!(
                "SELECT count(*) FROM information_schema.tables WHERE table_schema = '{}'",
                target.trim_matches('"').replace('\'', "''")
            ),
            None,
        )),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn one(sql: &str) -> SqlStatementImpact {
        classify(sql)
    }

    #[test]
    fn statements_split_on_semicolons_outside_strings_comments_and_dollar_blocks() {
        let sql = "delete from a where x = ';'; -- ; comentario\n/* ; */ update b set y = 1;\n\
                   do $$ begin perform 1; end $$; select 1";
        let parts = split_statements(sql);
        assert_eq!(parts.len(), 4, "{parts:?}");
        assert_eq!(parts[0], "delete from a where x = ';'");
        assert!(parts[1].ends_with("update b set y = 1"));
        assert!(parts[2].starts_with("do $$"));
        assert!(split_statements("-- so comentario\n;  ;").is_empty());
    }

    #[test]
    fn delete_and_update_without_where_are_destructive_and_with_where_are_writes() {
        let all = one("DELETE FROM clientes");
        assert_eq!(
            (all.kind.as_str(), all.severity),
            ("delete", SqlImpactSeverity::Destructive)
        );
        assert_eq!(all.targets, vec!["clientes"]);
        let some = one("delete from public.clientes where id = 2 returning *;");
        assert_eq!(some.severity, SqlImpactSeverity::Write);
        assert_eq!(some.filter, "id = 2");
        assert_eq!(some.targets, vec!["public.clientes"]);
        let upd = one("update \"Clientes\" set nome = 'where' ");
        assert_eq!(
            (upd.kind.as_str(), upd.severity),
            ("update", SqlImpactSeverity::Destructive)
        );
        assert!(upd.filter.is_empty(), "o WHERE dentro da string nao conta");
        assert_eq!(
            one("UPDATE clientes SET nome = 'x' WHERE id = 1").filter,
            "id = 1"
        );
        let sub = one("update t set x = (select max(y) from u where u.k = 1)");
        assert_eq!(
            sub.severity,
            SqlImpactSeverity::Destructive,
            "WHERE de subconsulta nao filtra"
        );
        assert_eq!(one("UPDATE ONLY t SET a = 1").targets, vec!["t"]);
        assert_eq!(one("update or replace t set a = 1").targets, vec!["t"]);
    }

    #[test]
    fn drops_truncate_and_dropped_columns_name_what_they_destroy() {
        let drop = one("DROP TABLE IF EXISTS pedidos, clientes CASCADE");
        assert_eq!(drop.kind, "dropTable");
        assert_eq!(drop.targets, vec!["pedidos", "clientes"]);
        assert_eq!(drop.severity, SqlImpactSeverity::Destructive);
        assert_eq!(
            one("truncate table only logs restart identity").targets,
            vec!["logs"]
        );
        assert_eq!(one("DROP SCHEMA audit CASCADE").kind, "dropSchema");
        assert_eq!(one("drop database loja").kind, "dropDatabase");
        assert_eq!(one("DROP VIEW v").severity, SqlImpactSeverity::Write);
        assert_eq!(one("DROP INDEX ix").severity, SqlImpactSeverity::Write);
        let col = one("ALTER TABLE clientes DROP COLUMN IF EXISTS email");
        assert_eq!(
            (col.kind.as_str(), col.column.as_str()),
            ("dropColumn", "email")
        );
        assert_eq!(col.severity, SqlImpactSeverity::Destructive);
        assert_eq!(one("alter table clientes drop constraint fk").kind, "alter");
        assert_eq!(
            one("ALTER TABLE t ADD COLUMN c int").severity,
            SqlImpactSeverity::Write
        );
    }

    #[test]
    fn inserts_creates_and_reads_keep_their_weight() {
        assert_eq!(
            one("INSERT INTO pedidos VALUES (1)").targets,
            vec!["pedidos"]
        );
        assert_eq!(
            one("CREATE TABLE t (a int)").severity,
            SqlImpactSeverity::Write
        );
        assert_eq!(one("-- c\nselect 1").severity, SqlImpactSeverity::Read);
        let all = classify_all("insert into a values (1); delete from b");
        assert_eq!(overall(&all), SqlImpactSeverity::Destructive);
        assert_eq!(overall(&[]), SqlImpactSeverity::Read);
    }

    #[test]
    fn counts_are_reads_on_the_same_target_and_filter() {
        let (hit, total) = count_queries(&one("delete from clientes where id > 1")).unwrap();
        assert_eq!(hit, "SELECT count(*) FROM clientes WHERE id > 1");
        assert_eq!(total.as_deref(), Some("SELECT count(*) FROM clientes"));
        assert!(is_read(&hit));
        let (all, none) = count_queries(&one("DELETE FROM clientes")).unwrap();
        assert_eq!(
            (all.as_str(), none),
            ("SELECT count(*) FROM clientes", None)
        );
        let (drop, _) = count_queries(&one("drop table a, b")).unwrap();
        assert_eq!(
            drop,
            "SELECT (SELECT count(*) FROM a) + (SELECT count(*) FROM b)"
        );
        let (col, _) = count_queries(&one("alter table t drop column c")).unwrap();
        assert_eq!(col, "SELECT count(*) FROM t WHERE c IS NOT NULL");
        let (schema, _) = count_queries(&one("drop schema audit")).unwrap();
        assert!(schema.ends_with("table_schema = 'audit'"));
        assert!(count_queries(&one("drop database loja")).is_none());
        assert!(count_queries(&one("insert into t values (1)")).is_none());
    }
}

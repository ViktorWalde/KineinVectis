//! Léxico comum do console: fronteiras reais e recusa de entrada ambígua.

#[derive(Debug)]
/// Resultado léxico que preserva posições no texto original.
pub struct SqlScan {
    /// Texto sem literais/comentários, com o mesmo tamanho em bytes.
    pub masked: String,
    /// Pontos e vírgulas de topo, fora de aspas, blocos e comentários.
    pub boundaries: Vec<usize>,
    /// Todas as fronteiras são conhecidas; falso exige recusa conservadora.
    pub valid: bool,
}

/// Palavra ou identificador sem interpretar strings como comandos.
#[derive(Debug)]
pub struct Word<'a> {
    /// Texto original, incluindo delimitadores de identificadores.
    pub text: &'a str,
    /// Posição inicial em bytes.
    pub start: usize,
    /// Número de parênteses abertos antes desta palavra.
    pub depth: usize,
}

impl Word<'_> {
    /// Palavra-chave sem diferença de caixa; um identificador com aspas não coincide.
    #[must_use]
    pub fn is(&self, keyword: &str) -> bool {
        self.text.eq_ignore_ascii_case(keyword)
    }
}

/// Palavras externas aos literais; nomes entre aspas continuam sendo nomes.
#[must_use]
pub fn words<'a>(text: &'a str, masked: &str) -> Vec<Word<'a>> {
    let bytes = masked.as_bytes();
    let mut result = Vec::new();
    let mut at = 0;
    let mut depth = 0usize;
    while at < bytes.len() {
        match bytes[at] {
            b'(' => {
                depth += 1;
                at += 1;
            }
            b')' => {
                depth = depth.saturating_sub(1);
                at += 1;
            }
            byte if byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'"' | b'`' | b'[') => {
                let start = at;
                while at < bytes.len() {
                    let byte = bytes[at];
                    if matches!(byte, b'"' | b'`' | b'[') {
                        let close = if byte == b'[' { b']' } else { byte };
                        at = quoted_end(bytes, at, close).0;
                    } else if byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b'$') {
                        at += 1;
                    } else {
                        break;
                    }
                }
                result.push(Word {
                    text: &text[start..at],
                    start,
                    depth,
                });
            }
            _ => {
                at += 1;
            }
        }
    }
    result
}

/// Leitura nativa exige que o lote inteiro não contenha comandos mutantes.
/// O servidor/arquivo somente leitura continua sendo a segunda proteção.
#[must_use]
pub fn is_read(sql: &str) -> bool {
    let parsed = lex(sql);
    if !parsed.valid {
        return false;
    }
    let tokens = words(sql, &parsed.masked);
    if tokens.is_empty() {
        return false;
    }
    let mut start = 0;
    for end in parsed
        .boundaries
        .iter()
        .copied()
        .chain(std::iter::once(sql.len()))
    {
        let statement_words = words(&sql[start..end], &parsed.masked[start..end]);
        if statement_words.first().is_some_and(|first| {
            !["SELECT", "WITH", "VALUES", "TABLE", "SHOW", "EXPLAIN"]
                .iter()
                .any(|w| first.is(w))
        }) {
            return false;
        }
        start = end.saturating_add(1);
    }
    !tokens.iter().any(|word| {
        [
            "INSERT",
            "UPDATE",
            "DELETE",
            "MERGE",
            "REPLACE",
            "CREATE",
            "ALTER",
            "DROP",
            "TRUNCATE",
            "GRANT",
            "REVOKE",
            "CALL",
            "DO",
            "COPY",
            "VACUUM",
            "ANALYZE",
            "REINDEX",
            "REFRESH",
            "LOCK",
            "SET",
            "RESET",
            "BEGIN",
            "START",
            "END",
            "COMMIT",
            "ROLLBACK",
            "SAVEPOINT",
            "RELEASE",
            "ATTACH",
            "DETACH",
            "PRAGMA",
            "INTO",
            "EXEC",
            "EXECUTE",
            "LOAD",
            "INSTALL",
            "EXPORT",
            "IMPORT",
        ]
        .iter()
        .any(|keyword| word.is(keyword))
    })
}

/// Mantém posições em bytes e identificadores; apaga comentários e literais.
#[must_use]
pub fn lex(sql: &str) -> SqlScan {
    let bytes = sql.as_bytes();
    let mut masked = bytes.to_vec();
    let mut boundaries = Vec::new();
    let mut at = 0;
    let mut valid = true;
    let mut depth = 0usize;
    while at < bytes.len() {
        if bytes[at..].starts_with(b"--") {
            let end = bytes[at..]
                .iter()
                .position(|b| matches!(b, b'\n' | b'\r'))
                .map_or(bytes.len(), |n| at + n);
            blank(&mut masked, at, end);
            at = end;
        } else if bytes[at..].starts_with(b"/*") {
            let start = at;
            // MySQL/MariaDB executam estes comentários: não são espaços.
            valid &= !matches!(bytes.get(at + 2), Some(b'!' | b'+'));
            let mut nesting = 1usize;
            at += 2;
            while at < bytes.len() && nesting != 0 {
                if bytes[at..].starts_with(b"/*") {
                    nesting += 1;
                    at += 2;
                } else if bytes[at..].starts_with(b"*/") {
                    nesting -= 1;
                    at += 2;
                } else {
                    at += 1;
                }
            }
            valid &= nesting == 0;
            blank(&mut masked, start, at);
        } else if bytes[at] == b'\'' {
            let start = at;
            let (end, closed, escaped) = quoted_end(bytes, at, b'\'');
            // A interpretação de barra depende do motor/configuração.
            valid &= closed && !escaped;
            blank(&mut masked, start, end);
            at = end;
        } else if matches!(bytes[at], b'"' | b'`' | b'[') {
            let close = if bytes[at] == b'[' { b']' } else { bytes[at] };
            let (end, closed, escaped) = quoted_end(bytes, at, close);
            valid &= closed && !escaped;
            at = end;
        } else if let Some(delimiter) = dollar_delimiter(bytes, at) {
            let start = at;
            let after = at + delimiter.len();
            let closing = bytes[after..]
                .windows(delimiter.len())
                .position(|w| w == delimiter);
            valid &= closing.is_some();
            at = closing.map_or(bytes.len(), |n| after + n + delimiter.len());
            blank(&mut masked, start, at);
        } else {
            match bytes[at] {
                b';' if depth == 0 => boundaries.push(at),
                b'(' => depth += 1,
                b')' => {
                    if depth == 0 {
                        valid = false;
                    }
                    depth = depth.saturating_sub(1);
                }
                _ => {}
            }
            at += 1;
        }
    }
    valid &= depth == 0;
    SqlScan {
        masked: String::from_utf8(masked).unwrap_or_default(),
        boundaries,
        valid,
    }
}

fn blank(bytes: &mut [u8], from: usize, to: usize) {
    for byte in &mut bytes[from..to] {
        if *byte != b'\n' {
            *byte = b' ';
        }
    }
}

pub(super) fn quoted_end(bytes: &[u8], start: usize, close: u8) -> (usize, bool, bool) {
    let mut at = start + 1;
    let mut escaped = false;
    while at < bytes.len() {
        if bytes[at] == close {
            if bytes.get(at + 1) == Some(&close) {
                at += 2;
            } else {
                return (at + 1, true, escaped);
            }
        } else if bytes[at] == b'\\' {
            escaped = true;
            // Escapes ficam ambíguos, mas não expõem uma fronteira interior.
            at = (at + 2).min(bytes.len());
        } else {
            at += 1;
        }
    }
    (at, false, escaped)
}

fn dollar_delimiter(bytes: &[u8], at: usize) -> Option<&[u8]> {
    if bytes[at] != b'$'
        || at != 0
            && (bytes[at - 1].is_ascii_alphanumeric()
                || !bytes[at - 1].is_ascii()
                || matches!(bytes[at - 1], b'_' | b'$'))
    {
        return None;
    }
    let mut end = at + 1;
    if bytes.get(end) == Some(&b'$') {
        return Some(&bytes[at..=end]);
    }
    if !bytes
        .get(end)
        .is_some_and(|b| b.is_ascii_alphabetic() || *b == b'_')
    {
        return None;
    }
    while bytes
        .get(end)
        .is_some_and(|b| b.is_ascii_alphanumeric() || *b == b'_')
    {
        end += 1;
    }
    (bytes.get(end) == Some(&b'$')).then(|| &bytes[at..=end])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn statement_separators_respect_literals_identifiers_and_nested_comments() {
        for text in [
            "SELECT ';'; DELETE FROM t",
            "SELECT 'it''s; safe'; DELETE FROM t",
            "SELECT \"a;\"\"b\" FROM t; DELETE FROM t",
            "SELECT `a;``b` FROM t; DELETE FROM t",
            "SELECT [a;]]b] FROM t; DELETE FROM t",
            "SELECT $tag$a; b$tag$; DELETE FROM t",
            "SELECT $$a; b$$; DELETE FROM t",
            "SELECT 1 /* outer /* inner */ ; */; DELETE FROM t",
            "SELECT 'ação;'; -- ;\nDELETE FROM t",
        ] {
            let parsed = lex(text);
            assert!(parsed.valid, "{text}");
            assert_eq!(parsed.boundaries.len(), 1, "{text}");
            assert!(
                text[parsed.boundaries[0] + 1..]
                    .trim_start()
                    .starts_with("DELETE")
                    || text[parsed.boundaries[0] + 1..].starts_with(" --"),
                "{text}"
            );
            assert_eq!(parsed.masked.len(), text.len());
        }
    }

    #[test]
    fn ambiguous_input_cannot_be_mistaken_for_a_safe_read() {
        for text in [
            "SELECT 'unterminated",
            "SELECT \"unterminated",
            "SELECT `unterminated",
            "SELECT [unterminated",
            "SELECT $tag$unterminated",
            "SELECT 1 /* unterminated",
            "SELECT 1 /*! DELETE FROM t */",
            "SELECT 1 /*+ hint */",
            "SELECT E'escaped\\'; DELETE FROM t",
            "SELECT (1",
            "SELECT 1)",
        ] {
            assert!(!lex(text).valid, "{text}");
        }
    }

    #[test]
    fn unicode_positions_and_dollar_in_identifiers_are_preserved() {
        let text = "SELECT 'ação', a$tag$ FROM t; SELECT 2";
        let parsed = lex(text);
        assert!(parsed.valid);
        assert_eq!(parsed.masked.len(), text.len());
        assert_eq!(parsed.boundaries.len(), 1);
        assert!(parsed.masked.contains("a$tag$"));
        for text in [
            "SELECT $missing",
            "SELECT $",
            "SELECT $1",
            "SELECT a$tag$",
            "SELECT $tag$one$tag$; SELECT 2",
        ] {
            let _ = lex(text);
        }
    }

    #[test]
    fn carriage_returns_and_unicode_identifiers_cannot_hide_transaction_escape() {
        let unsafe_commands: Vec<_> = [
            "SELECT 1 -- comment\r; COMMIT; DELETE FROM t",
            "SELECT 1 AS é$$; COMMIT; DELETE FROM t; SELECT 1 AS fim$$",
        ]
        .into_iter()
        .filter(|text| is_read(text))
        .collect();
        assert!(unsafe_commands.is_empty(), "{unsafe_commands:?}");
    }

    #[test]
    fn reads_cannot_hide_a_write_in_a_cte_or_batch() {
        for text in [
            "WITH x AS (DELETE FROM t RETURNING *) SELECT * FROM x",
            "WITH x AS (SELECT 1) INSERT INTO t SELECT * FROM x",
            "SELECT 1; COMMIT; DELETE FROM t",
            "SELECT * INTO saved FROM t",
            "EXPLAIN ANALYZE DELETE FROM t",
            "SELECT 1; PRAGMA writable_schema=ON",
            "SELECT 1; CALL erase()",
            "SELECT 1; UNKNOWN COMMAND",
            "SELECT 1; DISCARD ALL",
        ] {
            assert!(!is_read(text), "{text}");
        }
        for text in [
            "WITH x AS (SELECT 1) SELECT * FROM x",
            "SELECT count(*) FROM t",
            "SELECT 'DELETE; COMMIT'",
            "SELECT \"DELETE\", [COMMIT], `UPDATE` FROM t",
            "SELECT $tag$DELETE; COMMIT$tag$",
            "/* DELETE */ SELECT 1; SELECT 2",
            "EXPLAIN SELECT * FROM t",
            "SHOW server_version",
            "VALUES (1), (2)",
        ] {
            assert!(is_read(text), "{text}");
        }
    }
}

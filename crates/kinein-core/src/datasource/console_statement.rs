//! Instruction extraction from UTF-16 editor positions using the core SQL lexer.
use super::sql_syntax;

/// Extract a selection, Mongo line or SQL paragraph/statement.
/// # Errors
/// Oversized buffer, invalid UTF-16 offsets or ambiguous SQL constructs.
pub fn extract(
    text: &str,
    cursor: usize,
    selection_start: usize,
    selection_end: usize,
    mongo: bool,
) -> Result<String, &'static str> {
    if text.len() > 1024 * 1024 {
        return Err("O console excede 1 MiB; selecione um arquivo menor.");
    }
    let cursor = byte_offset(text, cursor)?;
    let start = byte_offset(text, selection_start)?;
    let end = byte_offset(text, selection_end)?;
    if start > end {
        return Err("A seleção do console é inválida.");
    }
    if end > start {
        return Ok(text[start..end].trim().to_owned());
    }
    if mongo {
        let start = text[..cursor].rfind('\n').map_or(0, |at| at + 1);
        let end = text[cursor..]
            .find('\n')
            .map_or(text.len(), |at| cursor + at);
        return clean(&text[start..end], true);
    }
    let scan = sql_syntax::lex(text);
    if !scan.valid {
        return Err(
            "Não foi possível separar o SQL com segurança; corrija ou selecione explicitamente a instrução.",
        );
    }
    let mut separators: Vec<_> = scan.boundaries.into_iter().map(|at| (at, at + 1)).collect();
    separators.extend(scan.paragraphs);
    separators.sort_unstable();
    let mut pieces = Vec::new();
    let mut start = 0;
    for (end, after) in separators {
        pieces.push((start, end));
        start = after;
    }
    pieces.push((start, text.len()));
    let mut index = pieces
        .iter()
        .rposition(|(start, _)| *start <= cursor)
        .unwrap_or(0);
    loop {
        let (start, end) = pieces[index];
        let statement = clean(&text[start..end], false)?;
        if !statement.is_empty() || index == 0 {
            return Ok(statement);
        }
        index -= 1;
    }
}

fn clean(piece: &str, mongo: bool) -> Result<String, &'static str> {
    let piece = piece.trim();
    if mongo {
        return Ok(if piece.starts_with("//") {
            String::new()
        } else {
            piece.to_owned()
        });
    }
    let scan = sql_syntax::lex(piece);
    if !scan.valid {
        return Err("A instrução SQL está incompleta ou ambígua.");
    }
    Ok(scan
        .code_start
        .map_or_else(String::new, |at| piece[at..].trim().to_owned()))
}

fn byte_offset(text: &str, wanted: usize) -> Result<usize, &'static str> {
    let mut units = 0;
    for (at, character) in text.char_indices() {
        if units == wanted {
            return Ok(at);
        }
        units += character.len_utf16();
        if units > wanted {
            return Err("A posição do cursor corta um caractere Unicode.");
        }
    }
    if units == wanted {
        Ok(text.len())
    } else {
        Err("A posição do cursor está fora do texto.")
    }
}

#[cfg(test)]
mod tests {
    use super::extract;
    fn at(text: &str, marker: &str) -> usize {
        text[..text.find(marker).unwrap()].encode_utf16().count()
    }
    fn statement(text: &str, marker: &str) -> String {
        let cursor = at(text, marker);
        extract(text, cursor, cursor, cursor, false).unwrap()
    }

    #[test]
    fn real_separators_never_split_literals_identifiers_or_nested_comments() {
        for text in [
            "SELECT '; DELETE FROM t';\nSELECT 2;",
            "SELECT 'one\n\nDELETE FROM t';\nSELECT 2;",
            "SELECT $$one;\n\nDELETE FROM t$$;\nSELECT 2;",
            "SELECT $body$one;\n\nDELETE FROM t$body$;\nSELECT 2;",
            "SELECT \"a;\n\nDELETE FROM t\";\nSELECT 2;",
            "SELECT [a;\n\nDELETE FROM t];\nSELECT 2;",
            "SELECT `a;\n\nDELETE FROM t`;\nSELECT 2;",
            "SELECT 1 /* outer /* ; */\n\nDELETE FROM t */;\nSELECT 2;",
        ] {
            assert!(statement(text, "DELETE").starts_with("SELECT"), "{text}");
            assert_eq!(statement(text, "2;"), "SELECT 2");
        }
    }

    #[test]
    fn external_blank_lines_and_cursor_at_the_end_preserve_console_gestures() {
        let text = "-- header\nSELECT 1 -- trailing\n\n\nDELETE FROM t;\n";
        assert_eq!(statement(text, "SELECT"), "SELECT 1 -- trailing");
        assert_eq!(statement(text, "DELETE"), "DELETE FROM t");
        let end = text.encode_utf16().count();
        assert_eq!(
            extract(text, end, end, end, false).unwrap(),
            "DELETE FROM t"
        );
        assert_eq!(extract("-- comment\n", 3, 3, 3, false).unwrap(), "");
    }

    #[test]
    fn explicit_selection_and_unicode_positions_are_checked() {
        let text = "SELECT '😀; ação';\nSELECT 2;";
        assert_eq!(statement(text, "ação"), "SELECT '😀; ação'");
        let start = at(text, "SELECT 2");
        assert_eq!(
            extract(text, 0, start, start + 8, false).unwrap(),
            "SELECT 2"
        );
        let middle = at(text, "😀") + 1;
        assert!(extract(text, middle, middle, middle, false).is_err());
        assert!(extract(text, 999, 0, 0, false).is_err());
        assert!(extract(text, 0, 10, 2, false).is_err());
    }

    #[test]
    fn explicit_selection_preserves_unknown_dialect_for_the_existing_query_policy() {
        let text = "SELECT E'escaped\\n';";
        assert!(extract(text, 0, 0, 0, false).is_err());
        assert_eq!(
            extract(text, 0, 0, text.encode_utf16().count(), false).unwrap(),
            text
        );
    }

    #[test]
    fn incomplete_sql_is_refused_and_mongo_keeps_a_whole_json_line() {
        for text in [
            "SELECT 'bad; DELETE FROM t",
            "SELECT $$bad",
            "SELECT 1 /* open",
            "SELECT (1",
            "SELECT E'\\\\';",
        ] {
            assert!(extract(text, 0, 0, 0, false).is_err(), "{text}");
        }
        let text = "// header\nitems.insertOne({\"text\":\"; DELETE\"})\nitems {}";
        let cursor = at(text, "DELETE");
        assert_eq!(
            extract(text, cursor, cursor, cursor, true).unwrap(),
            "items.insertOne({\"text\":\"; DELETE\"})"
        );
        assert_eq!(extract(text, 0, 0, 0, true).unwrap(), "");
        assert!(extract(&"x".repeat(1024 * 1024 + 1), 0, 0, 0, true).is_err());
    }
}

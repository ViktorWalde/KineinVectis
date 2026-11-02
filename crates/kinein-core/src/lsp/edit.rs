//! Aplicacao de edits de texto LSP sobre conteudo UTF-8.
//!
//! O servidor entrega posicoes 0-based com colunas em unidades UTF-16; aqui
//! elas viram offsets de byte e os edits sao aplicados de tras para frente
//! para nao invalidar offsets. Ranges invertidos ou sobrepostos falham sem
//! resultado parcial.

use super::types::{LspError, TextSpanEdit};

/// Aplica edits LSP (0-based, colunas UTF-16) a um conteudo UTF-8.
///
/// Os edits sao aplicados de tras para frente para preservar offsets; ranges
/// invertidos ou sobrepostos geram erro sem resultado parcial.
pub fn apply_text_edits(content: &str, edits: &[TextSpanEdit]) -> Result<String, LspError> {
    let spans = validated_spans(content, edits, false)?;
    let mut result = String::with_capacity(content.len());
    let mut cursor = 0;
    for (start, end, new_text) in spans.into_iter().rev() {
        result.push_str(&content[cursor..start]);
        result.push_str(new_text);
        cursor = end;
    }
    result.push_str(&content[cursor..]);
    Ok(result)
}

/// Strict byte ranges for compiler mapping; malformed UTF-16 positions are rejected.
pub fn text_edit_byte_spans(
    content: &str,
    edits: &[TextSpanEdit],
) -> Result<Vec<(usize, usize, usize)>, LspError> {
    Ok(validated_spans(content, edits, true)?
        .into_iter()
        .map(|(s, e, t)| (s, e, t.len()))
        .collect())
}

struct Positions {
    length: usize,
    starts: Vec<usize>,
    columns: std::collections::BTreeMap<usize, Vec<usize>>,
    limits: std::collections::BTreeMap<usize, usize>,
}

impl Positions {
    fn new(content: &str, edits: &[TextSpanEdit]) -> Self {
        let starts = std::iter::once(0)
            .chain(content.match_indices('\n').map(|(i, _)| i + 1))
            .collect::<Vec<_>>();
        let requested = edits
            .iter()
            .flat_map(|e| [e.start_line, e.end_line])
            .filter_map(|n| usize::try_from(n).ok())
            .collect::<std::collections::BTreeSet<_>>();
        let mut columns = std::collections::BTreeMap::new();
        let mut limits = std::collections::BTreeMap::new();
        for line in requested {
            let Some(&start) = starts.get(line) else {
                continue;
            };
            let end = starts.get(line + 1).map_or(content.len(), |n| n - 1);
            let text = &content[start..end];
            let mut offsets = vec![0];
            for (byte, c) in text.char_indices() {
                for _ in 0..c.len_utf16() {
                    offsets.push(byte + c.len_utf8());
                }
            }
            limits.insert(line, offsets.len() - 1 - usize::from(text.ends_with('\r')));
            columns.insert(line, offsets);
        }
        Self {
            length: content.len(),
            starts,
            columns,
            limits,
        }
    }
    fn offset(&self, line: u64, character: u64, strict: bool) -> Result<usize, LspError> {
        let line = usize::try_from(line).unwrap_or(usize::MAX);
        let target = usize::try_from(character).unwrap_or(usize::MAX);
        let Some(columns) = self.columns.get(&line) else {
            return if strict {
                Err(invalid_position())
            } else {
                Ok(self.length)
            };
        };
        let middle = columns
            .get(target)
            .is_some_and(|value| columns.get(target.saturating_add(1)) == Some(value));
        if strict && (target > *self.limits.get(&line).unwrap_or(&0) || middle) {
            return Err(invalid_position());
        }
        Ok(self.starts[line]
            + columns
                .get(target)
                .copied()
                .unwrap_or_else(|| *columns.last().unwrap_or(&0)))
    }
}

fn invalid_position() -> LspError {
    LspError::Transport {
        message: "Range LSP fora do documento ou entre unidades UTF-16 de um caractere.".into(),
    }
}

fn validated_spans<'a>(
    content: &str,
    edits: &'a [TextSpanEdit],
    strict: bool,
) -> Result<Vec<(usize, usize, &'a str)>, LspError> {
    let positions = Positions::new(content, edits);
    let mut spans = Vec::with_capacity(edits.len());
    for edit in edits {
        let start = positions.offset(edit.start_line, edit.start_character, strict)?;
        let end = positions.offset(edit.end_line, edit.end_character, strict)?;
        if end < start {
            return Err(LspError::Transport {
                message: "edit LSP com range invertido".into(),
            });
        }
        spans.push((start, end, edit.new_text.as_str()));
    }
    spans.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| b.1.cmp(&a.1)));
    let mut previous = usize::MAX;
    for &(start, end, _) in &spans {
        if end > previous {
            return Err(LspError::Transport {
                message: "edits LSP sobrepostos".into(),
            });
        }
        previous = start;
    }
    Ok(spans)
}

#[cfg(test)]
mod tests {
    use super::super::types::TextSpanEdit;
    use super::apply_text_edits;

    fn span(
        start_line: u64,
        start_character: u64,
        end_line: u64,
        end_character: u64,
        new_text: &str,
    ) -> TextSpanEdit {
        TextSpanEdit {
            start_line,
            start_character,
            end_line,
            end_character,
            new_text: new_text.to_owned(),
        }
    }

    #[test]
    fn apply_text_edits_applies_reverse_order_on_same_line() {
        let content = "let valor = valor + valor;\n";
        let edits = vec![
            span(0, 4, 0, 9, "total"),
            span(0, 12, 0, 17, "total"),
            span(0, 20, 0, 25, "total"),
        ];

        let result = apply_text_edits(content, &edits).unwrap();

        assert_eq!(result, "let total = total + total;\n");
    }

    #[test]
    fn apply_text_edits_handles_utf16_columns_and_multiline() {
        // "á" ocupa 1 unidade UTF-16 e 2 bytes UTF-8; o edit troca `nome`.
        let content = "// á comentário\nfn nome() {}\nnome();\n";
        let edits = vec![span(1, 3, 1, 7, "inicio"), span(2, 0, 2, 4, "inicio")];

        let result = apply_text_edits(content, &edits).unwrap();

        assert_eq!(result, "// á comentário\nfn inicio() {}\ninicio();\n");
    }

    #[test]
    fn apply_text_edits_rejects_overlapping_and_inverted_ranges() {
        let content = "abcdef\n";
        let overlapping = vec![span(0, 0, 0, 4, "x"), span(0, 2, 0, 6, "y")];
        assert!(apply_text_edits(content, &overlapping).is_err());

        let inverted = vec![span(0, 5, 0, 2, "x")];
        assert!(apply_text_edits(content, &inverted).is_err());
    }

    #[test]
    fn apply_text_edits_clamps_positions_past_the_end() {
        let content = "fim";
        let edits = vec![span(9, 9, 9, 9, "!")];

        assert_eq!(apply_text_edits(content, &edits).unwrap(), "fim!");
    }

    #[test]
    fn edit_verification_mapping_rejects_invalid_utf16_and_outside_ranges() {
        let half = vec![span(0, 1, 0, 2, "x")];
        assert!(super::text_edit_byte_spans("😀a", &half).is_err());
        assert!(super::text_edit_byte_spans("😀a", &[span(2, 0, 2, 0, "x")]).is_err());
        assert_eq!(
            super::text_edit_byte_spans("😀a", &[span(0, 2, 0, 3, "x")]).unwrap(),
            [(4, 5, 1)]
        );
        assert!(super::text_edit_byte_spans("abc\r\n", &[span(0, 4, 0, 4, "x")]).is_err());
    }

    #[test]
    fn edit_verification_many_edits_on_one_line_are_bounded() {
        let content = "a".repeat(200_000);
        let edits = (0..2000)
            .map(|i| span(0, i * 100, 0, i * 100 + 1, "b"))
            .collect::<Vec<_>>();
        let started = std::time::Instant::now();
        let edited = apply_text_edits(&content, &edits).unwrap();
        assert_eq!(edited.len(), content.len());
        assert_eq!(edited.bytes().filter(|b| *b == b'b').count(), 2000);
        assert!(started.elapsed() < std::time::Duration::from_secs(2));
    }
}

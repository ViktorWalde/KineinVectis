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
    let starts = line_start_offsets(content);
    let mut spans = Vec::with_capacity(edits.len());
    for edit in edits {
        let start = position_to_offset(content, &starts, edit.start_line, edit.start_character);
        let end = position_to_offset(content, &starts, edit.end_line, edit.end_character);
        if end < start {
            return Err(LspError::Transport {
                message: "edit LSP com range invertido".to_owned(),
            });
        }
        spans.push((start, end, edit.new_text.as_str()));
    }
    spans.sort_by(|left, right| right.0.cmp(&left.0).then_with(|| right.1.cmp(&left.1)));

    let mut previous_start = usize::MAX;
    let mut result = content.to_owned();
    for (start, end, new_text) in spans {
        if end > previous_start {
            return Err(LspError::Transport {
                message: "edits LSP sobrepostos".to_owned(),
            });
        }
        previous_start = start;
        result.replace_range(start..end, new_text);
    }
    Ok(result)
}

/// Offset em bytes do inicio de cada linha do conteudo.
fn line_start_offsets(content: &str) -> Vec<usize> {
    let mut starts = vec![0];
    for (index, byte) in content.bytes().enumerate() {
        if byte == b'\n' {
            starts.push(index + 1);
        }
    }
    starts
}

/// Converte posicao LSP (linha 0-based, coluna UTF-16) em offset de byte.
///
/// Posicoes fora do conteudo sao grampeadas ao fim do texto/linha.
fn position_to_offset(content: &str, starts: &[usize], line: u64, character: u64) -> usize {
    let Ok(line_index) = usize::try_from(line) else {
        return content.len();
    };
    let Some(&line_start) = starts.get(line_index) else {
        return content.len();
    };
    let line_end = starts
        .get(line_index + 1)
        .map_or(content.len(), |next| next.saturating_sub(1));
    line_start + utf16_col_to_byte(&content[line_start..line_end], character)
}

/// Converte uma coluna em unidades UTF-16 para offset de byte dentro da linha.
fn utf16_col_to_byte(line: &str, character: u64) -> usize {
    let target = usize::try_from(character).unwrap_or(usize::MAX);
    let mut units = 0_usize;
    for (byte_index, symbol) in line.char_indices() {
        if units >= target {
            return byte_index;
        }
        units += symbol.len_utf16();
    }
    line.len()
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
}

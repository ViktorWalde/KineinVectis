//! UTF-8 byte and Qt UTF-16 position conversion helpers.
//!
//! # Invariante dos chamadores
//!
//! `byte` tem de cair em **fronteira de caractere UTF-8**. Fatiar `&text[..b]`
//! num `b` no meio de um caractere multibyte é pânico, e este projeto proíbe
//! pânico fora de teste.
//!
//! A invariante é satisfeita hoje, e vale registrar POR QUE, porque não é
//! óbvio: todos os chamadores passam offsets vindos do tree-sitter — início/fim
//! de nó (`lang/outline.rs`, `lang/service.rs`) e limites de captura de query,
//! que são sempre fronteira em entrada UTF-8 válida. O único chamador que
//! calcula offset por conta própria, `contiguous_edit`, **recua explicitamente
//! até a fronteira** antes de chamar `byte_point` (`lang/service.rs`).
//!
//! Estas funções **não grampeiam** o offset para a fronteira mais próxima, e a
//! escolha é deliberada: grampear transformaria um chamador errado em posição
//! silenciosamente torta — um bug de realce que ninguém liga à causa. Se um
//! chamador novo não puder garantir a fronteira, ele recua como o
//! `contiguous_edit` faz, no lugar onde o offset nasce e onde há contexto para
//! decidir.
//!
//! A mesma invariante vale para `push_highlight_segments` em `lang/service.rs`,
//! que fatia `source[start..range.end]` direto. Ela é da CAMADA, não deste
//! arquivo.

use tree_sitter::Point;

/// One-based line and zero-based UTF-16 column.
///
/// # Panics
///
/// Se `byte` não estiver em fronteira de caractere UTF-8 — ver a invariante no
/// topo do módulo.
pub(super) fn utf16_position(text: &str, byte: usize) -> (u64, u64) {
    debug_assert!(
        byte >= text.len() || text.is_char_boundary(byte),
        "offset {byte} nao esta em fronteira de caractere"
    );
    let byte = byte.min(text.len());
    let prefix = &text[..byte];
    let line_start = prefix.rfind('\n').map_or(0, |index| index + 1);
    let line = prefix.bytes().filter(|value| *value == b'\n').count() + 1;
    let column = text[line_start..byte].encode_utf16().count();
    (usize_to_u64(line), usize_to_u64(column))
}

/// Tree-sitter byte-based point for an input edit boundary.
///
/// # Panics
///
/// Se `byte` não estiver em fronteira de caractere UTF-8 — ver a invariante no
/// topo do módulo.
pub(super) fn byte_point(text: &str, byte: usize) -> Point {
    debug_assert!(
        byte >= text.len() || text.is_char_boundary(byte),
        "offset {byte} nao esta em fronteira de caractere"
    );
    let byte = byte.min(text.len());
    let prefix = &text[..byte];
    let row = prefix.bytes().filter(|value| *value == b'\n').count();
    let column = prefix.rfind('\n').map_or(byte, |index| byte - index - 1);
    Point::new(row, column)
}

pub(super) fn usize_to_u64(value: usize) -> u64 {
    u64::try_from(value).unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use super::{byte_point, utf16_position};

    /// A invariante do modulo, exercitada: offset em fronteira funciona, e o
    /// `debug_assert` reprova quem passar um offset torto.
    ///
    /// Este teste existe para que a invariante nao seja folclore. Se alguem
    /// trocar o `debug_assert` por um `clamp` silencioso, ele cai — e o corte
    /// certo, quando um chamador nao puder garantir a fronteira, e' recuar no
    /// lugar onde o offset NASCE (como `contiguous_edit` faz), nao aqui.
    #[test]
    #[should_panic(expected = "nao esta em fronteira de caractere")]
    fn offset_no_meio_de_um_caractere_reprova_em_debug() {
        let text = "cafe\u{301}zinho"; // o acento combinante ocupa 2 bytes
        let meio = text.find('\u{301}').unwrap() + 1;
        assert!(!text.is_char_boundary(meio));

        let _ = utf16_position(text, meio);
    }

    #[test]
    fn converts_utf8_bytes_to_qt_utf16_columns() {
        let text = "a😀b\nç";
        let b = text.find('b').unwrap_or_default();
        let cedilla_end = text.len();

        assert_eq!(utf16_position(text, b), (1, 3));
        assert_eq!(utf16_position(text, cedilla_end), (2, 1));
        assert_eq!(byte_point(text, b), tree_sitter::Point::new(0, 5));
    }
}

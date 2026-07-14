//! UTF-8 byte and Qt UTF-16 position conversion helpers.

use tree_sitter::Point;

/// One-based line and zero-based UTF-16 column.
pub(super) fn utf16_position(text: &str, byte: usize) -> (u64, u64) {
    let byte = byte.min(text.len());
    let prefix = &text[..byte];
    let line_start = prefix.rfind('\n').map_or(0, |index| index + 1);
    let line = prefix.bytes().filter(|value| *value == b'\n').count() + 1;
    let column = text[line_start..byte].encode_utf16().count();
    (usize_to_u64(line), usize_to_u64(column))
}

/// Tree-sitter byte-based point for an input edit boundary.
pub(super) fn byte_point(text: &str, byte: usize) -> Point {
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

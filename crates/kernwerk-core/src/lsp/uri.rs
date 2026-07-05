//! Conversao entre caminhos do filesystem e URIs `file://` do LSP.
//!
//! O encoding e proprio (percent-encoding minimo sobre bytes) para nao puxar
//! dependencia externa; cobre o que os servidores C/C++ e Rust usam no Linux.

use std::path::Path;

const fn hex_digit(value: u8) -> char {
    match value {
        0..=9 => (b'0' + value) as char,
        _ => (b'A' + value - 10) as char,
    }
}

/// Converte um caminho absoluto na URI `file://` percent-encoded do LSP.
pub(super) fn uri_for_path(path: &Path) -> String {
    let mut encoded = String::from("file://");
    for byte in path.to_string_lossy().bytes() {
        let is_plain = byte.is_ascii_alphanumeric() || matches!(byte, b'/' | b'.' | b'-' | b'_');
        if is_plain {
            encoded.push(char::from(byte));
        } else {
            encoded.push('%');
            encoded.push(hex_digit(byte >> 4_u8));
            encoded.push(hex_digit(byte & 0x0F));
        }
    }
    encoded
}

/// Decodifica uma URI `file://` de volta ao caminho absoluto do filesystem.
pub(super) fn path_for_uri(uri: &str) -> Option<String> {
    let raw = uri.strip_prefix("file://")?;
    let bytes = raw.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            let hex = std::str::from_utf8(&bytes[index + 1..index + 3]).ok()?;
            let value = u8::from_str_radix(hex, 16).ok()?;
            decoded.push(value);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(decoded).ok()
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{path_for_uri, uri_for_path};

    #[test]
    fn uri_conversion_roundtrip_with_spaces() {
        let path = Path::new("/home/user/meu projeto/main.rs");
        let uri = uri_for_path(path);

        assert_eq!(uri, "file:///home/user/meu%20projeto/main.rs");
        assert_eq!(
            path_for_uri(&uri).unwrap(),
            "/home/user/meu projeto/main.rs"
        );
    }
}

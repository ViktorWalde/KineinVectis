//! Conversao entre caminhos do filesystem e URIs `file://` do LSP.
//!
//! O encoding e proprio (percent-encoding minimo sobre bytes) para nao puxar
//! dependencia externa; cobre o que os servidores C/C++, Rust, Python e QML
//! usam.
//!
//! No Windows (DocsPublic/roadmaps/60 §3.3, W2b) o caminho `C:\a\b` vira
//! `file:///C:/a/b` e o UNC `\\host\share\x` vira `file://host/share/x`, a
//! forma da RFC 8089. Na volta, a URI que o servidor devolve pode vir com o
//! drive em minuscula e os dois-pontos codificados (`file:///c%3A/a/b`, a forma
//! do VS Code); o drive volta em maiuscula, como o `platform::canonicalize` o
//! escreve, para o caminho bater com o do core.

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
    let text = path.to_string_lossy();
    let rest = uri_head(&text, &mut encoded);
    for byte in rest.bytes() {
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

/// No Unix o caminho ja' e' o da URI.
#[cfg(unix)]
fn uri_head(text: &str, _encoded: &mut String) -> String {
    text.to_owned()
}

/// No Windows: as barras viram `/`; o drive vai sem codificar (`/C:`), e o
/// UNC leva o host para a autoridade da URI.
#[cfg(windows)]
fn uri_head(text: &str, encoded: &mut String) -> String {
    let text = text.replace('\\', "/");
    if let Some(unc) = text.strip_prefix("//") {
        return unc.to_owned();
    }
    let bytes = text.as_bytes();
    if bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' {
        encoded.push('/');
        encoded.push(char::from(bytes[0].to_ascii_uppercase()));
        encoded.push(':');
        return text[2..].to_owned();
    }
    text
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
    String::from_utf8(decoded)
        .ok()
        .map(|text| native_path(&text))
}

#[cfg(unix)]
fn native_path(decoded: &str) -> String {
    decoded.to_owned()
}

/// `/C:/a` ou `/c:/a` -> `C:\a`; `host/share/x` (com autoridade) -> `\\host\share\x`.
#[cfg(windows)]
fn native_path(decoded: &str) -> String {
    let Some(rest) = decoded.strip_prefix('/') else {
        return format!("\\\\{}", decoded.replace('/', "\\"));
    };
    let bytes = rest.as_bytes();
    if bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' {
        return format!(
            "{}{}",
            char::from(bytes[0].to_ascii_uppercase()),
            rest[1..].replace('/', "\\")
        );
    }
    decoded.replace('/', "\\")
}

/// O projeto de exemplo dos testes do LSP, na forma do sistema: a URI de um
/// arquivo dele...
#[cfg(test)]
pub(super) fn demo_uri(relative: &str) -> String {
    let root = if cfg!(windows) {
        "file:///C:/demo"
    } else {
        "file:///tmp/demo"
    };
    format!("{root}/{relative}")
}

/// ...e o caminho que o [`path_for_uri`] devolve para ela.
#[cfg(test)]
pub(super) fn demo_path(relative: &str) -> String {
    if cfg!(windows) {
        format!(r"C:\demo\{}", relative.replace('/', r"\"))
    } else {
        format!("/tmp/demo/{relative}")
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{path_for_uri, uri_for_path};

    #[test]
    #[cfg(unix)]
    fn uri_conversion_roundtrip_with_spaces() {
        let path = Path::new("/home/user/meu projeto/main.rs");
        let uri = uri_for_path(path);

        assert_eq!(uri, "file:///home/user/meu%20projeto/main.rs");
        assert_eq!(
            path_for_uri(&uri).unwrap(),
            "/home/user/meu projeto/main.rs"
        );
    }

    #[test]
    #[cfg(windows)]
    fn uri_conversion_roundtrip_with_a_drive_and_spaces() {
        let path = Path::new(r"C:\Users\ana\meu projeto\main.rs");
        let uri = uri_for_path(path);

        assert_eq!(uri, "file:///C:/Users/ana/meu%20projeto/main.rs");
        assert_eq!(
            path_for_uri(&uri).unwrap(),
            r"C:\Users\ana\meu projeto\main.rs"
        );
    }

    /// A forma do VS Code, que servidores como o `basedpyright` ecoam.
    #[test]
    #[cfg(windows)]
    fn a_lowercase_encoded_drive_comes_back_as_the_core_writes_it() {
        assert_eq!(
            path_for_uri("file:///c%3A/Users/ana/main.rs").unwrap(),
            r"C:\Users\ana\main.rs"
        );
        assert_eq!(
            path_for_uri("file:///c:/Users/ana/main.rs").unwrap(),
            r"C:\Users\ana\main.rs"
        );
    }

    #[test]
    #[cfg(windows)]
    fn a_unc_path_puts_the_host_in_the_authority() {
        let path = Path::new(r"\\servidor\projetos\app\main.rs");
        let uri = uri_for_path(path);

        assert_eq!(uri, "file://servidor/projetos/app/main.rs");
        assert_eq!(
            path_for_uri(&uri).unwrap(),
            r"\\servidor\projetos\app\main.rs"
        );
    }
}

//! Framing `Content-Length` do wire LSP e escrita concorrente no stdin.
//!
//! Toda mensagem LSP e serializada com cabecalho `Content-Length` e corpo JSON.
//! O stdin do servidor e compartilhado entre a thread leitora (respostas a
//! requests servidor->cliente) e o manager (requests cliente->servidor), por
//! isso fica protegido por `Mutex`.

use std::{
    io::{self, BufRead, Write},
    process::ChildStdin,
    sync::{Arc, Mutex},
};

use serde_json::{Value, json};

/// Monta os `params` de um `textDocument/didChange` com texto completo.
pub(super) fn full_change_params(uri: &str, version: i64, content: &str) -> Value {
    json!({
        "textDocument": { "uri": uri, "version": version },
        "contentChanges": [{ "text": content }],
    })
}

/// Envia uma notificacao LSP (sem `id`) pelo stdin compartilhado.
pub(super) fn send_notification(stdin: &Arc<Mutex<ChildStdin>>, method: &str, params: &Value) {
    let message = json!({ "jsonrpc": "2.0", "method": method, "params": params });
    drop(write_locked_message(stdin, &message));
}

/// Escreve uma mensagem no stdin compartilhado, travando o `Mutex`.
pub(super) fn write_locked_message(
    stdin: &Arc<Mutex<ChildStdin>>,
    message: &Value,
) -> io::Result<()> {
    let Ok(mut guard) = stdin.lock() else {
        return Err(io::Error::other("stdin do servidor envenenado"));
    };
    write_message(&mut *guard, message)
}

/// Escreve uma mensagem LSP com framing `Content-Length`.
pub(super) fn write_message(writer: &mut impl Write, message: &Value) -> io::Result<()> {
    let body = serde_json::to_vec(message).map_err(io::Error::other)?;
    write!(writer, "Content-Length: {}\r\n\r\n", body.len())?;
    writer.write_all(&body)?;
    writer.flush()
}

/// Le uma mensagem LSP com framing `Content-Length`. `None` significa EOF.
pub(super) fn read_message(reader: &mut impl BufRead) -> io::Result<Option<Value>> {
    let mut content_length: Option<usize> = None;

    loop {
        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 {
            return Ok(None);
        }
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            break;
        }
        if let Some(value) = trimmed
            .strip_prefix("Content-Length:")
            .map(str::trim)
            .and_then(|raw| raw.parse::<usize>().ok())
        {
            content_length = Some(value);
        }
    }

    let Some(length) = content_length else {
        return Err(io::Error::other("cabecalho Content-Length ausente"));
    };
    let mut body = vec![0_u8; length];
    reader.read_exact(&mut body)?;
    serde_json::from_slice(&body)
        .map(Some)
        .map_err(io::Error::other)
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use serde_json::json;

    use super::{read_message, write_message};

    #[test]
    fn framing_roundtrip_preserves_message() {
        let message = json!({ "jsonrpc": "2.0", "method": "initialized", "params": {} });
        let mut buffer = Vec::new();
        write_message(&mut buffer, &message).unwrap();

        let text = String::from_utf8(buffer.clone()).unwrap();
        assert!(text.starts_with("Content-Length: "));

        let decoded = read_message(&mut Cursor::new(buffer)).unwrap().unwrap();
        assert_eq!(decoded, message);
    }

    #[test]
    fn read_message_reports_eof_as_none() {
        let mut empty = Cursor::new(Vec::new());
        assert!(read_message(&mut empty).unwrap().is_none());
    }
}

//! Read the compiler's own Make dependency list, without shell interpretation.

use std::path::{Path, PathBuf};

pub(super) fn parse(text: &str, directory: &Path) -> Result<Vec<PathBuf>, String> {
    let text = text
        .strip_prefix("kinein:")
        .ok_or("Lista de dependências do compilador inválida.")?;
    let mut paths = Vec::new();
    let mut value = String::new();
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\\' => match chars.next() {
                Some('\n') => {}
                Some('\r') if chars.peek() == Some(&'\n') => {
                    chars.next();
                }
                Some(next) => value.push(next),
                None => return Err("Escape incompleto na lista de dependências.".into()),
            },
            '$' if chars.peek() == Some(&'$') => {
                chars.next();
                value.push('$');
            }
            c if c.is_whitespace() => {
                if !value.is_empty() {
                    paths.push(directory.join(std::mem::take(&mut value)));
                }
            }
            c => value.push(c),
        }
    }
    if !value.is_empty() {
        paths.push(directory.join(value));
    }
    Ok(paths)
}

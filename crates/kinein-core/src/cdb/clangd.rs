//! O `.clangd` da raiz que fixa a base (`CompileFlags.CompilationDatabase`).
//!
//! Decisao do autor em 2026-11-09 (consolidacao C §16): o `.clangd` que fixa a
//! base vence a CDB que a IDE escolheria. Medido com o clangd 23.1.4, o
//! `--compile-commands-dir` prevalece sobre o `.clangd`; por isso, quando ele
//! decide, a IDE deixa de passar a opcao.
//!
//! Nao ha' parser de YAML no core, e um proprio seria um parser a mais. A
//! leitura reconhece so' a forma simples: um unico fragmento com a chave, sem
//! `If`, com `CompilationDatabase: <pasta>` no bloco `CompileFlags:`. Qualquer
//! outra forma e' base manual: o clangd decide e a IDE nao atribui CDB.

use std::path::{Path, PathBuf};

/// O que o `.clangd` da raiz diz sobre a base.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Base {
    /// Sem `.clangd`, ou sem `CompilationDatabase` nele.
    Unset,
    /// A forma simples: esta pasta, relativa a' raiz.
    Directory(PathBuf),
    /// O `.clangd` decide numa forma que a IDE nao interpreta.
    Manual(&'static str),
}

/// O arquivo da raiz.
pub(super) const FILE: &str = ".clangd";

pub(super) fn base(root: &Path) -> Base {
    // Como o clangd o le, sem esperar num FIFO (INF-18).
    let Ok(text) = crate::fsops::read_project_text(&root.join(FILE)) else {
        return Base::Unset;
    };
    let mut fragments = documents(&text).into_iter().filter(|lines| {
        lines
            .iter()
            .any(|line| line.contains("CompilationDatabase"))
    });
    match (fragments.next(), fragments.next()) {
        (None, _) => Base::Unset,
        (Some(lines), None) => simple(root, &lines),
        (Some(_), Some(_)) => Base::Manual("CompilationDatabase em mais de um fragmento"),
    }
}

/// Os fragmentos (documentos YAML separados por `---`), sem comentarios e
/// sem linhas vazias.
fn documents(text: &str) -> Vec<Vec<&str>> {
    let mut documents = vec![Vec::new()];
    for line in text.lines().map(without_comment) {
        if matches!(line, "---" | "...") {
            documents.push(Vec::new());
        } else if !line.trim().is_empty()
            && let Some(current) = documents.last_mut()
        {
            current.push(line);
        }
    }
    documents
}

/// A linha sem o comentario: `#` no inicio ou depois de espaco, fora de aspas.
fn without_comment(line: &str) -> &str {
    let mut quote = None;
    let mut after_space = true;
    for (index, character) in line.char_indices() {
        match quote {
            Some(open) if character == open => quote = None,
            None if matches!(character, '\'' | '"') => quote = Some(character),
            None if character == '#' && after_space => return line[..index].trim_end(),
            _ => {}
        }
        after_space = character.is_whitespace();
    }
    line.trim_end()
}

fn top_level(line: &str) -> bool {
    !line.starts_with(char::is_whitespace)
}

fn simple(root: &Path, lines: &[&str]) -> Base {
    if lines
        .iter()
        .any(|line| top_level(line) && line.starts_with("If:"))
    {
        return Base::Manual("fragmento condicional (If)");
    }
    let Some(start) = lines.iter().position(|line| *line == "CompileFlags:") else {
        return Base::Manual("CompilationDatabase fora de um bloco CompileFlags");
    };
    let block: Vec<&str> = lines[start + 1..]
        .iter()
        .take_while(|line| !top_level(line))
        .copied()
        .collect();
    let indentation = |line: &str| line.len() - line.trim_start().len();
    let child = block.first().map_or(0, |line| indentation(line));
    let mut values = block.iter().filter_map(|line| {
        (indentation(line) == child)
            .then(|| line.trim_start().strip_prefix("CompilationDatabase:"))
            .flatten()
    });
    let (Some(value), None) = (values.next(), values.next()) else {
        return Base::Manual("CompilationDatabase fora de um bloco CompileFlags");
    };
    if lines
        .iter()
        .filter(|line| line.contains("CompilationDatabase"))
        .count()
        != 1
    {
        return Base::Manual("CompilationDatabase repetido no fragmento");
    }
    let value = value.trim();
    let value = ['"', '\'']
        .iter()
        .find_map(|quote| {
            value
                .strip_prefix(*quote)
                .and_then(|rest| rest.strip_suffix(*quote))
        })
        .unwrap_or(value);
    match value {
        "" => Base::Manual("CompilationDatabase sem valor"),
        "Ancestors" => Base::Manual("Ancestors: o clangd procura a base por arquivo"),
        "None" => Base::Manual("None: o clangd não usa base"),
        other if other.starts_with(['{', '[', '|', '>', '&', '*', '!']) => {
            Base::Manual("CompilationDatabase com valor que não é uma pasta")
        }
        other => Base::Directory(root.join(other)),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{Base, base};

    fn root_with(name: &str, clangd: Option<&str>) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("kinein-cdb-clangd")
            .join(format!("{}-{name}", std::process::id()));
        drop(std::fs::remove_dir_all(&dir));
        std::fs::create_dir_all(&dir).unwrap();
        if let Some(text) = clangd {
            std::fs::write(dir.join(".clangd"), text).unwrap();
        }
        dir
    }

    #[test]
    fn the_simple_form_names_a_directory_relative_to_the_root() {
        for (name, text, expected) in [
            (
                "plain",
                "CompileFlags:\n  CompilationDatabase: build/debug\n",
                "build/debug",
            ),
            (
                "quoted",
                "# base\nCompileFlags:\n    Add: [-Wall]\n    CompilationDatabase: \"out dir\" # x\n",
                "out dir",
            ),
            (
                "other-fragment",
                "Diagnostics:\n  UnusedIncludes: None\n---\nCompileFlags:\n  CompilationDatabase: 'b'\n",
                "b",
            ),
            (
                "absolute",
                "CompileFlags:\n  CompilationDatabase: /opt/kv/build\n",
                "/opt/kv/build",
            ),
        ] {
            let root = root_with(name, Some(text));
            assert_eq!(base(&root), Base::Directory(root.join(expected)), "{name}");
        }
    }

    #[test]
    fn other_forms_are_left_to_clangd() {
        for (name, text) in [
            (
                "conditional",
                "If:\n  PathMatch: src/.*\nCompileFlags:\n  CompilationDatabase: build\n",
            ),
            (
                "two-fragments",
                "CompileFlags:\n  CompilationDatabase: a\n---\nCompileFlags:\n  CompilationDatabase: b\n",
            ),
            (
                "ancestors",
                "CompileFlags:\n  CompilationDatabase: Ancestors\n",
            ),
            ("none", "CompileFlags:\n  CompilationDatabase: None\n"),
            ("flow", "CompileFlags: {CompilationDatabase: build}\n"),
            (
                "nested",
                "CompileFlags:\n  Add:\n    CompilationDatabase: build\n",
            ),
            ("empty", "CompileFlags:\n  CompilationDatabase:\n"),
            ("anchor", "CompileFlags:\n  CompilationDatabase: *dir\n"),
        ] {
            let root = root_with(name, Some(text));
            assert!(
                matches!(base(&root), Base::Manual(_)),
                "{name}: {:?}",
                base(&root)
            );
        }
    }

    #[test]
    fn without_the_key_the_clangd_file_does_not_decide() {
        for (name, text) in [
            ("absent", None),
            ("other-keys", Some("CompileFlags:\n  Add: [-DX]\n")),
            (
                "comment",
                Some("# CompilationDatabase: build\nCompileFlags:\n  Add: [-DX]\n"),
            ),
        ] {
            assert_eq!(base(&root_with(name, text)), Base::Unset, "{name}");
        }
    }
}

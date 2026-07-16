//! Buffer formatting by orchestrating the project's own formatters.
//!
//! `format.text` runs the same tools the verification gate uses (`rustfmt`,
//! `clang-format`) over the editor buffer via stdin/stdout: no file is
//! touched on disk, and the formatter runs with the workspace root as its
//! working directory so project configuration (`rustfmt.toml`,
//! `.clang-format`) applies. Formatting one buffer is a short operation, so
//! it stays a synchronous request/response instead of a job.

use std::{
    io::{self, Write},
    path::Path,
    process::{Command, Stdio},
    thread,
};

/// Formatter selected for a file extension.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum FormatterKind {
    /// `rustfmt`, for Rust sources.
    Rustfmt,
    /// `clang-format`, for C/C++ sources and headers.
    ClangFormat,
}

impl FormatterKind {
    /// Stable identifier reported in `format.text` responses; also the binary
    /// name looked up on `PATH`.
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::Rustfmt => "rustfmt",
            Self::ClangFormat => "clang-format",
        }
    }
}

/// Extensões que cada formatter atende. **Fonte única** do mapa
/// extensão → formatter: `formatter_for_path` decide por aqui e
/// `format.capabilities` publica isto para a UI. Uma segunda lista em qualquer
/// lugar (inclusive no QML) diverge desta por construção — foi exatamente o que
/// aconteceu até o protocolo 0.61.0.
const FORMATTER_EXTENSIONS: [(FormatterKind, &[&str]); 2] = [
    (FormatterKind::Rustfmt, &["rs"]),
    (
        FormatterKind::ClangFormat,
        &["c", "cc", "cpp", "cxx", "h", "hh", "hpp", "hxx"],
    ),
];

/// Catálogo de formatters registrados.
///
/// Existe para o core ser a única autoridade sobre o que é formatável. Estático:
/// não depende de workspace nem de o binário existir no `PATH` — isso é assunto
/// de `format.text`, não de capacidade.
#[must_use]
pub fn capabilities() -> Vec<(FormatterKind, &'static [&'static str])> {
    FORMATTER_EXTENSIONS.to_vec()
}

/// Picks the formatter for `path` by file extension, or `None` when the
/// extension has no registered formatter.
#[must_use]
pub fn formatter_for_path(path: &Path) -> Option<FormatterKind> {
    let extension = path.extension()?.to_str()?.to_lowercase();
    // Derivado do MESMO mapa que `capabilities()` publica: o que a UI recebe é,
    // por construção, o que esta função vai aceitar.
    FORMATTER_EXTENSIONS
        .iter()
        .find(|(_kind, extensions)| extensions.contains(&extension.as_str()))
        .map(|(kind, _extensions)| *kind)
}

/// Builds the formatter invocation for `path`, rooted at the workspace `root`.
///
/// `rustfmt` reads project style from `rustfmt.toml`/`.rustfmt.toml` in the
/// working directory; when the workspace has neither, `--edition 2021` keeps
/// modern syntax parseable (standalone `rustfmt` does not read `Cargo.toml`).
/// `clang-format` resolves `.clang-format` upwards from `--assume-filename`.
#[must_use]
pub fn formatter_command(kind: FormatterKind, root: &Path, path: &Path) -> Command {
    let mut command = Command::new(kind.id());
    command.current_dir(root);
    match kind {
        FormatterKind::Rustfmt => {
            command.args(["--emit", "stdout"]);
            if !root.join("rustfmt.toml").exists() && !root.join(".rustfmt.toml").exists() {
                command.args(["--edition", "2021"]);
            }
        }
        FormatterKind::ClangFormat => {
            command.arg(format!("--assume-filename={}", path.display()));
            command.args(["--style=file", "--fallback-style=LLVM"]);
        }
    }
    command
}

/// Failure while formatting a buffer.
#[derive(Debug)]
pub enum FormatError {
    /// The formatter binary could not be started (usually missing on `PATH`).
    Spawn {
        /// Formatter identifier (`rustfmt`, `clang-format`).
        tool: &'static str,
        /// Underlying spawn error.
        source: io::Error,
    },
    /// The formatter exited with a failure status.
    Failed {
        /// Formatter identifier.
        tool: &'static str,
        /// Trimmed stderr emitted by the formatter.
        stderr: String,
    },
    /// Reading or writing the formatter pipes failed.
    Io {
        /// Formatter identifier.
        tool: &'static str,
        /// Underlying I/O error.
        source: io::Error,
    },
    /// The formatter produced output that is not valid UTF-8.
    NotUtf8 {
        /// Formatter identifier.
        tool: &'static str,
    },
}

/// Runs `command`, writing `text` to its stdin and returning its stdout.
///
/// stdin is fed from a dedicated thread while `wait_with_output` drains
/// stdout/stderr, so large buffers can never deadlock on a full pipe.
pub fn run_formatter(
    mut command: Command,
    tool: &'static str,
    text: &str,
) -> Result<String, FormatError> {
    command.stdin(Stdio::piped());
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    let mut child = command
        .spawn()
        .map_err(|source| FormatError::Spawn { tool, source })?;
    let Some(mut stdin) = child.stdin.take() else {
        return Err(FormatError::Io {
            tool,
            source: io::Error::other("stdin do formatter indisponivel"),
        });
    };

    let payload = text.as_bytes().to_vec();
    let writer = thread::spawn(move || stdin.write_all(&payload));

    let output = child
        .wait_with_output()
        .map_err(|source| FormatError::Io { tool, source })?;

    match writer.join() {
        // A write failure with a successful exit means the tool ignored part
        // of its input; with a failure exit, stderr below explains it better.
        Ok(Err(source)) if output.status.success() => {
            return Err(FormatError::Io { tool, source });
        }
        Err(_panic) => {
            return Err(FormatError::Io {
                tool,
                source: io::Error::other("thread de escrita do formatter falhou"),
            });
        }
        Ok(_) => {}
    }

    if !output.status.success() {
        return Err(FormatError::Failed {
            tool,
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        });
    }
    String::from_utf8(output.stdout).map_err(|_source| FormatError::NotUtf8 { tool })
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::process::Command;

    use super::{FormatError, FormatterKind, formatter_for_path, run_formatter};

    #[test]
    fn formatter_is_picked_by_extension() {
        assert_eq!(
            formatter_for_path(Path::new("/w/src/main.rs")),
            Some(FormatterKind::Rustfmt)
        );
        for name in [
            "a.c", "a.cc", "a.cpp", "a.cxx", "a.h", "a.hh", "a.hpp", "a.hxx",
        ] {
            assert_eq!(
                formatter_for_path(Path::new(name)),
                Some(FormatterKind::ClangFormat),
                "extensao de {name} deveria usar clang-format"
            );
        }
        assert_eq!(formatter_for_path(Path::new("nota.txt")), None);
        assert_eq!(formatter_for_path(Path::new("Makefile")), None);
    }

    #[test]
    fn run_formatter_pipes_stdin_to_stdout() {
        let mut command = Command::new("tr");
        command.args(["a-z", "A-Z"]);
        let formatted = run_formatter(command, "tr", "abc\n").unwrap();
        assert_eq!(formatted, "ABC\n");
    }

    #[test]
    fn missing_binary_is_a_spawn_error() {
        let command = Command::new("kinein-formatter-que-nao-existe");
        let error = run_formatter(command, "fake", "abc").unwrap_err();
        assert!(matches!(error, FormatError::Spawn { tool: "fake", .. }));
    }

    #[test]
    fn failure_exit_carries_stderr() {
        let mut command = Command::new("sh");
        command.args(["-c", "echo estilo invalido >&2; exit 3"]);
        let error = run_formatter(command, "sh", "abc").unwrap_err();
        match error {
            FormatError::Failed { tool, stderr } => {
                assert_eq!(tool, "sh");
                assert_eq!(stderr, "estilo invalido");
            }
            other => panic!("esperava Failed, veio {other:?}"),
        }
    }
}

#[cfg(test)]
mod capability_tests {
    use std::path::PathBuf;

    use super::*;

    /// O ponto da fatia 0.61.0: o catálogo que a UI recebe e a decisão que o
    /// core toma vêm da MESMA fonte. Se alguém adicionar uma extensão em um
    /// lugar e esquecer o outro, isto quebra — que era o defeito que existia,
    /// só que entre core e QML, onde nenhum teste alcançava.
    #[test]
    fn catalogo_publicado_bate_com_a_decisao_real() {
        for (kind, extensions) in capabilities() {
            for extension in extensions {
                let caminho = PathBuf::from(format!("arquivo.{extension}"));
                assert_eq!(
                    formatter_for_path(&caminho),
                    Some(kind),
                    "extensao .{extension} esta no catalogo mas nao e aceita"
                );
                // Maiúsculas resolvem igual: o mapa normaliza.
                let alto = PathBuf::from(format!("arquivo.{}", extension.to_uppercase()));
                assert_eq!(formatter_for_path(&alto), Some(kind));
            }
        }
    }

    #[test]
    fn extensao_fora_do_catalogo_nao_tem_formatter() {
        for extensao in ["md", "txt", "json", "toml", "py", ""] {
            let caminho = PathBuf::from(format!("arquivo.{extensao}"));
            assert!(
                formatter_for_path(&caminho).is_none(),
                ".{extensao} nao deveria ter formatter"
            );
        }
        assert!(formatter_for_path(&PathBuf::from("SemExtensao")).is_none());
    }

    /// Nenhuma extensão pode pertencer a dois formatters: a decisão seria
    /// ambígua e dependeria da ordem da constante.
    #[test]
    fn extensoes_nao_se_repetem_entre_formatters() {
        let mut vistas: Vec<&str> = Vec::new();
        for (_kind, extensions) in capabilities() {
            for extension in extensions {
                assert!(
                    !vistas.contains(extension),
                    "extensao .{extension} aparece em dois formatters"
                );
                vistas.push(extension);
            }
        }
        assert!(!vistas.is_empty());
    }
}

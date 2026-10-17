//! A CHECAGEM DO COMPILADOR AO SALVAR (C1, `roadmaps/52` §5.2 e §14.3).
//!
//! A linha da checagem sai da unidade da `compile_commands.json` — a MESMA que
//! o clangd le' —, com `-fsyntax-only` no lugar da compilacao: o compilador do
//! ALVO diz o erro na linha, sem build. Este modulo e' puro: recebe a unidade
//! e a saida, devolve a linha e os diagnosticos; quem roda e' o job.
//!
//! O formato da saida e' o classico `arquivo:linha:coluna: error: ...`, que o
//! GCC 15.2 e o clang 21.1.8 dao igual (medido em 2026-10-16) e que o
//! [`super::parse::parse_gcc_like_line`] ja' le' — o dono do formato (52 §3.2).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use kinein_protocol::{CompileUnit, Diagnostic, DiagnosticSeverity, DiagnosticSource};

use super::parse::parse_gcc_like_line;

/// A linha da checagem: o programa, os argumentos e o diretorio da unidade.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntaxCheck {
    /// O compilador da unidade (`argv[0]` da CDB).
    pub program: String,
    /// Os argumentos, ja' sem saida e sem dependencias, com `-fsyntax-only`.
    pub args: Vec<String>,
    /// O diretorio em que a CDB diz que o comando roda.
    pub directory: PathBuf,
}

/// Flags que levam o proximo argumento como valor e saem junto.
const DROPPED_WITH_VALUE: [&str; 4] = ["-o", "-MF", "-MT", "-MQ"];
/// Flags que saem sozinhas.
const DROPPED: [&str; 6] = ["-c", "-MD", "-MMD", "-MP", "-M", "-MM"];

/// A linha da checagem, ou `None` quando a unidade nao e' do tipo GCC/clang
/// (o `cl.exe` do MSVC fica de fora nesta fatia).
#[must_use]
pub fn syntax_only_command(unit: &CompileUnit) -> Option<SyntaxCheck> {
    let (program, rest) = unit.arguments.split_first()?;
    if rest
        .iter()
        .any(|a| a.starts_with('/') && a.len() > 1 && is_msvc_flag(a))
    {
        return None;
    }
    let mut args = Vec::with_capacity(rest.len() + 2);
    let mut iter = rest.iter();
    while let Some(argument) = iter.next() {
        if DROPPED_WITH_VALUE.contains(&argument.as_str()) {
            iter.next();
            continue;
        }
        let inline_output = argument.len() > 2 && argument.starts_with("-o");
        let color = argument.starts_with("-fdiagnostics-color")
            || argument == "-fcolor-diagnostics"
            || argument == "-fno-color-diagnostics";
        if DROPPED.contains(&argument.as_str()) || inline_output || color {
            continue;
        }
        args.push(argument.clone());
    }
    args.push("-fsyntax-only".to_owned());
    args.push("-fdiagnostics-color=never".to_owned());
    Some(SyntaxCheck {
        program: program.clone(),
        args,
        directory: PathBuf::from(&unit.directory),
    })
}

/// `/Fo`, `/c`, `/std:`: a forma das flags do `cl.exe`. Um caminho absoluto
/// do Unix (`/usr/include`) tambem comeca com `/`; o que separa e' a letra
/// seguida de `:` ou o par conhecido.
fn is_msvc_flag(argument: &str) -> bool {
    matches!(argument, "/c" | "/nologo" | "/TP" | "/TC")
        || argument.starts_with("/Fo")
        || argument.starts_with("/std:")
        || argument.starts_with("/EH")
}

/// Os diagnosticos da saida, por arquivo absoluto. Notas ficam de fora: o
/// clangd as mostra como informacao relacionada, e na lista elas so' repetem
/// o erro.
#[must_use]
pub fn diagnostics_by_file(
    lines: &[String],
    directory: &Path,
    job_id: Option<&str>,
) -> BTreeMap<String, Vec<Diagnostic>> {
    let mut by_file: BTreeMap<String, Vec<Diagnostic>> = BTreeMap::new();
    for line in lines {
        let Some(parsed) = parse_gcc_like_line(line) else {
            continue;
        };
        if !matches!(
            parsed.severity,
            DiagnosticSeverity::Error | DiagnosticSeverity::Warning
        ) {
            continue;
        }
        let Some(file) = parsed.file.as_deref() else {
            continue;
        };
        let absolute = absolute_path(directory, file);
        let mut diagnostic =
            parsed.to_diagnostic(DiagnosticSource::Compiler, job_id.map(str::to_owned));
        diagnostic.file = Some(absolute.clone());
        by_file.entry(absolute).or_default().push(diagnostic);
    }
    by_file
}

fn absolute_path(directory: &Path, file: &str) -> String {
    let joined = directory.join(file);
    crate::platform::canonicalize(&joined)
        .unwrap_or(joined)
        .display()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit(arguments: &[&str]) -> CompileUnit {
        CompileUnit {
            compiler: arguments
                .first()
                .map_or_else(String::new, |a| (*a).to_owned()),
            directory: "/p/build".to_owned(),
            standard: Some("c++20".to_owned()),
            includes: Vec::new(),
            defines: Vec::new(),
            output: None,
            arguments: arguments.iter().map(|a| (*a).to_owned()).collect(),
        }
    }

    /// Sai o que compila e grava (saida, `-c`, dependencias com e sem
    /// argumento, cores); fica o que muda o significado (`-std`, `-I`, `-D`, o
    /// arquivo); entram `-fsyntax-only` e a cor desligada.
    #[test]
    fn the_check_keeps_the_meaning_and_drops_what_writes() {
        let check = syntax_only_command(&unit(&[
            "/usr/bin/g++",
            "-std=c++20",
            "-Iinc",
            "-DFOO=1",
            "-fdiagnostics-color=always",
            "-MD",
            "-MT",
            "obj.o",
            "-MF",
            "obj.d",
            "-o",
            "obj.o",
            "-c",
            "../src/a.cpp",
        ]))
        .unwrap();
        assert_eq!(check.program, "/usr/bin/g++");
        assert_eq!(check.directory, PathBuf::from("/p/build"));
        assert_eq!(
            check.args,
            [
                "-std=c++20",
                "-Iinc",
                "-DFOO=1",
                "../src/a.cpp",
                "-fsyntax-only",
                "-fdiagnostics-color=never"
            ]
        );
        let inline = syntax_only_command(&unit(&["cc", "-oobj.o", "a.c"])).unwrap();
        assert_eq!(
            inline.args,
            ["a.c", "-fsyntax-only", "-fdiagnostics-color=never"]
        );
    }

    /// O `cl.exe` do MSVC fica de fora nesta fatia; um caminho absoluto do
    /// Unix nao e' confundido com flag do MSVC.
    #[test]
    fn msvc_units_are_left_out_but_unix_paths_are_not_msvc() {
        assert!(
            syntax_only_command(&unit(&[
                "cl.exe", "/nologo", "/TP", "/Foa.obj", "/c", "a.cpp"
            ]))
            .is_none()
        );
        let unix =
            syntax_only_command(&unit(&["g++", "-I/usr/include/qt6", "/abs/a.cpp"])).unwrap();
        assert!(unix.args.contains(&"/abs/a.cpp".to_owned()));
        assert!(syntax_only_command(&unit(&[])).is_none());
    }

    /// A saida: relativo resolvido contra o diretorio da unidade, o cabecalho
    /// incluido tambem entra, notas e linhas soltas ficam de fora, e a fonte e'
    /// o compilador.
    #[test]
    fn the_output_becomes_diagnostics_by_absolute_file() {
        let lines: Vec<String> = [
            "a.cpp: In function 'int main()':",
            "src/a.cpp:3:11: error: invalid conversion from 'const char*' to 'int'",
            "src/a.cpp:4:10: warning: unused variable 'z'",
            "inc/b.h:1:1: error: 'tipo' does not name a type",
            "src/a.cpp:3:11: note: declared here",
            "2 errors generated.",
        ]
        .iter()
        .map(|l| (*l).to_owned())
        .collect();
        let by_file = diagnostics_by_file(&lines, Path::new("/p"), Some("job_7"));
        assert_eq!(by_file.len(), 2, "{by_file:?}");
        let a = &by_file["/p/src/a.cpp"];
        assert_eq!(a.len(), 2, "a nota fica de fora");
        assert_eq!(a[0].source, DiagnosticSource::Compiler);
        assert_eq!(a[0].severity, DiagnosticSeverity::Error);
        assert_eq!((a[0].line, a[0].column), (Some(3), Some(11)));
        assert_eq!(a[0].file.as_deref(), Some("/p/src/a.cpp"));
        assert_eq!(a[0].job_id.as_deref(), Some("job_7"));
        assert_eq!(a[1].severity, DiagnosticSeverity::Warning);
        assert_eq!(
            by_file["/p/inc/b.h"][0].message,
            "'tipo' does not name a type"
        );
    }
}

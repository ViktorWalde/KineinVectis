//! Interpretar a SAIDA das ferramentas: JSON do cargo e linhas gcc-like.
//!
//! Separado em 2026-09-03 (etapa 14 do `roadmaps/34`), quando o `build.rs`
//! passou de 500. A `LEITURA_TECNICA` §3 ja' descrevia este dominio como
//! "orquestracao + parse de saida" — sao duas coisas, e o corte so' deu a cada
//! uma o seu arquivo.
//!
//! Aqui nao se executa nada: e' funcao pura de texto para diagnostico, e por
//! isso e' a parte do `build` que se testa sem subir processo.

use serde::Deserialize;

use kinein_protocol::{BuildDiagnostic, BuildDiagnosticSeverity};

/// Parsed cargo JSON message: rendered text plus optional structured data.
pub(super) struct CargoParsed {
    pub(super) rendered: Option<String>,
    pub(super) structured: Option<BuildDiagnostic>,
}

#[derive(Deserialize)]
struct CargoMessage {
    reason: String,
    message: Option<CargoCompilerMessage>,
}

#[derive(Deserialize)]
struct CargoCompilerMessage {
    level: String,
    message: String,
    rendered: Option<String>,
    #[serde(default)]
    spans: Vec<CargoSpan>,
}

#[derive(Deserialize)]
struct CargoSpan {
    file_name: String,
    line_start: u64,
    column_start: u64,
    is_primary: bool,
}

pub(super) fn parse_cargo_json_line(line: &str) -> Option<CargoParsed> {
    let message = serde_json::from_str::<CargoMessage>(line).ok()?;
    if message.reason != "compiler-message" {
        return None;
    }
    let compiler_message = message.message?;

    let severity = match compiler_message.level.as_str() {
        "error" | "error: internal compiler error" => BuildDiagnosticSeverity::Error,
        "warning" => BuildDiagnosticSeverity::Warning,
        _ => {
            return Some(CargoParsed {
                rendered: compiler_message.rendered,
                structured: None,
            });
        }
    };

    let primary_span = compiler_message
        .spans
        .iter()
        .find(|span| span.is_primary)
        .or_else(|| compiler_message.spans.first());

    Some(CargoParsed {
        rendered: compiler_message.rendered.clone(),
        structured: Some(BuildDiagnostic {
            severity,
            message: compiler_message.message,
            file: primary_span.map(|span| span.file_name.clone()),
            line: primary_span.map(|span| span.line_start),
            column: primary_span.map(|span| span.column_start),
        }),
    })
}

/// Parses `file:line[:column]: level: message` lines from compilers and `CMake`.
pub(super) fn parse_gcc_like_line(line: &str) -> Option<BuildDiagnostic> {
    const MARKERS: [(&str, BuildDiagnosticSeverity); 4] = [
        (": fatal error: ", BuildDiagnosticSeverity::Error),
        (": error: ", BuildDiagnosticSeverity::Error),
        (": warning: ", BuildDiagnosticSeverity::Warning),
        (": note: ", BuildDiagnosticSeverity::Note),
    ];

    for (marker, severity) in MARKERS {
        let Some(position) = line.find(marker) else {
            continue;
        };
        let location = &line[..position];
        let message = line[position + marker.len()..].trim().to_owned();
        if message.is_empty() {
            return None;
        }

        // Tenta arquivo:linha:coluna, depois arquivo:linha.
        let mut pieces = location.rsplitn(3, ':');
        let last = pieces.next()?;
        let middle = pieces.next();
        let head = pieces.next();

        if let (Some(head), Some(middle)) = (head, middle) {
            if let (Ok(line_number), Ok(column)) = (middle.parse::<u64>(), last.parse::<u64>()) {
                return Some(BuildDiagnostic {
                    severity,
                    message,
                    file: Some(head.to_owned()),
                    line: Some(line_number),
                    column: Some(column),
                });
            }
        }

        let mut pieces = location.rsplitn(2, ':');
        let last = pieces.next()?;
        let head = pieces.next();
        if let (Some(head), Ok(line_number)) = (head, last.parse::<u64>()) {
            return Some(BuildDiagnostic {
                severity,
                message,
                file: Some(head.to_owned()),
                line: Some(line_number),
                column: None,
            });
        }

        return Some(BuildDiagnostic {
            severity,
            message,
            file: None,
            line: None,
            column: None,
        });
    }

    None
}

/// `ruff check --output-format concise`: `arquivo:linha:coluna: CODIGO [*] mensagem`
/// (docs.astral.sh/ruff/settings/#output-format, lido em 2026-09-12). O `[*]`
/// diz "corrigivel com --fix" e vai para a mensagem, porque a tela e' onde
/// isso importa. Erro de sintaxe (`SyntaxError`) e os `E9xx` sao erro; o
/// resto e' aviso — lint nao impede o programa de rodar.
pub(super) fn parse_ruff_concise_line(line: &str) -> Option<BuildDiagnostic> {
    let mut partes = line.splitn(4, ':');
    let file = partes.next()?.trim();
    let linha: u64 = partes.next()?.trim().parse().ok()?;
    let coluna: u64 = partes.next()?.trim().parse().ok()?;
    let resto = partes.next()?.trim();
    if file.is_empty() || resto.is_empty() {
        return None;
    }
    let (codigo, mensagem) = resto.split_once(' ').unwrap_or((resto, ""));
    let e_codigo = codigo
        .chars()
        .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
        && codigo.chars().any(|c| c.is_ascii_digit());
    if !e_codigo && codigo != "SyntaxError:" {
        return None;
    }
    let severity = if codigo.starts_with("E9") || codigo.starts_with("SyntaxError") {
        BuildDiagnosticSeverity::Error
    } else {
        BuildDiagnosticSeverity::Warning
    };
    let mensagem = mensagem.trim();
    let message = if codigo.starts_with("SyntaxError") {
        format!("SyntaxError: {mensagem}")
    } else if let Some(fix) = mensagem.strip_prefix("[*] ") {
        format!("{codigo}: {fix} (corrigivel: ruff check --fix)")
    } else {
        format!("{codigo}: {mensagem}")
    };
    Some(BuildDiagnostic {
        severity,
        message,
        file: Some(file.to_owned()),
        line: Some(linha),
        column: Some(coluna),
    })
}

#[cfg(test)]
mod tests {
    use kinein_protocol::BuildDiagnosticSeverity;

    use super::{parse_cargo_json_line, parse_gcc_like_line, parse_ruff_concise_line};

    #[test]
    fn gcc_line_with_column_is_parsed() {
        let diagnostic =
            parse_gcc_like_line("src/main.cpp:42:13: error: expected ';' after expression")
                .unwrap();

        assert_eq!(diagnostic.severity, BuildDiagnosticSeverity::Error);
        assert_eq!(diagnostic.file.as_deref(), Some("src/main.cpp"));
        assert_eq!(diagnostic.line, Some(42));
        assert_eq!(diagnostic.column, Some(13));
        assert!(diagnostic.message.contains("expected"));
    }

    #[test]
    fn gcc_line_without_column_is_parsed() {
        let diagnostic =
            parse_gcc_like_line("CMakeLists.txt:7: error: unknown command foo").unwrap();

        assert_eq!(diagnostic.file.as_deref(), Some("CMakeLists.txt"));
        assert_eq!(diagnostic.line, Some(7));
        assert_eq!(diagnostic.column, None);
    }

    #[test]
    fn ordinary_lines_are_not_diagnostics() {
        assert!(parse_gcc_like_line("[2/10] Building CXX object foo.o").is_none());
        assert!(parse_gcc_like_line("Compiling kinein-core v0.1.0").is_none());
    }

    #[test]
    fn cargo_compiler_message_becomes_diagnostic() {
        let line = r#"{"reason":"compiler-message","message":{"level":"error","message":"mismatched types","rendered":"error[E0308]: mismatched types\n","spans":[{"file_name":"src/lib.rs","line_start":10,"column_start":5,"is_primary":true}]}}"#;

        let parsed = parse_cargo_json_line(line).unwrap();
        let diagnostic = parsed.structured.unwrap();

        assert_eq!(diagnostic.severity, BuildDiagnosticSeverity::Error);
        assert_eq!(diagnostic.file.as_deref(), Some("src/lib.rs"));
        assert_eq!(diagnostic.line, Some(10));
        assert!(parsed.rendered.unwrap().contains("E0308"));
    }

    #[test]
    fn cargo_non_compiler_messages_are_ignored() {
        assert!(parse_cargo_json_line(r#"{"reason":"build-finished","success":true}"#).is_none());
        assert!(parse_cargo_json_line("nao e json").is_none());
    }

    /// `ruff check --output-format concise` (docs.astral.sh, 2026-09-12):
    /// `arquivo:linha:coluna: CODIGO [*] mensagem`; erros de sintaxe vem como
    /// `arquivo:l:c: SyntaxError: mensagem`. O `[*]` = corrigivel por `--fix`.
    #[test]
    fn ruff_concise_lines_become_diagnostics() {
        let aviso = parse_ruff_concise_line("src/app.py:1:8: F401 [*] `os` imported but unused")
            .expect("linha concise");
        assert_eq!(aviso.severity, BuildDiagnosticSeverity::Warning);
        assert_eq!(aviso.file.as_deref(), Some("src/app.py"));
        assert_eq!((aviso.line, aviso.column), (Some(1), Some(8)));
        assert_eq!(
            aviso.message,
            "F401: `os` imported but unused (corrigivel: ruff check --fix)"
        );

        let sem_fix = parse_ruff_concise_line("app.py:3:1: E402 Module level import not at top")
            .expect("sem [*]");
        assert_eq!(sem_fix.message, "E402: Module level import not at top");
        assert!(!sem_fix.message.contains("corrigivel"));

        let e9 = parse_ruff_concise_line("app.py:2:5: E902 No such file").unwrap();
        assert_eq!(e9.severity, BuildDiagnosticSeverity::Error, "E9xx e' erro");

        let sintaxe = parse_ruff_concise_line("app.py:4:9: SyntaxError: Expected an expression")
            .expect("erro de sintaxe");
        assert_eq!(sintaxe.severity, BuildDiagnosticSeverity::Error);
        assert_eq!(sintaxe.message, "SyntaxError: Expected an expression");
        assert_eq!((sintaxe.line, sintaxe.column), (Some(4), Some(9)));
    }

    /// O resumo e o ruido do ruff nao viram diagnostico: `Found 2 errors.`,
    /// `[*] 1 fixable with the --fix option`, linhas vazias, e caminhos com
    /// dois-pontos sem numero (nao e' `arquivo:linha:coluna`).
    #[test]
    fn ruff_summary_and_noise_are_not_diagnostics() {
        for linha in [
            "Found 2 errors.",
            "[*] 1 fixable with the `--fix` option.",
            "",
            "All checks passed!",
            "warning: The top-level linter settings are deprecated",
            "app.py:x:1: F401 nao numerico",
            "app.py:1:2: minusculo nao e codigo",
            "app.py:1:2: TODO sem digito nao e codigo",
            ":1:2: F401 sem arquivo",
            "app.py:1:2:",
        ] {
            assert!(
                parse_ruff_concise_line(linha).is_none(),
                "nao deveria virar diagnostico: {linha:?}"
            );
        }
    }
}

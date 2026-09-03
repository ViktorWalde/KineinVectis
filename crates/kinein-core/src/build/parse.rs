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

#[cfg(test)]
mod tests {
    use kinein_protocol::BuildDiagnosticSeverity;

    use super::{parse_cargo_json_line, parse_gcc_like_line};

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
}

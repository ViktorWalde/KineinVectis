//! Cobertura de testes (L2 fatia 3): orquestra `cargo-llvm-cov` (Rust) e
//! `lcov` (C/C++) e normaliza os numeros no contrato do protocolo.
//!
//! Nada aqui instrumenta build: o Rust vem instrumentado pelo proprio
//! `cargo llvm-cov`; o C/C++ depende de o projeto compilar com `--coverage`
//! — sem dados, o erro explica isso em vez de fingir 0%.

use std::{
    error::Error,
    fmt,
    path::Path,
    process::Command,
    sync::{Arc, atomic::AtomicBool},
};

use kinein_protocol::{CoverageFileSummary, CoverageTotals, ProjectKind};

use crate::{
    build::project_kind_name,
    process::{self, ProcessError},
};

/// Resultado normalizado de uma medicao de cobertura.
#[derive(Debug, Clone, PartialEq)]
pub struct CoverageReport {
    /// Totais do workspace.
    pub totals: CoverageTotals,
    /// Cobertura por arquivo, na ordem reportada pela ferramenta.
    pub files: Vec<CoverageFileSummary>,
}

/// Erro ao medir cobertura.
#[derive(Debug)]
pub enum CoverageError {
    /// O tipo de projeto nao tem integracao de cobertura.
    Unsupported {
        /// Nome do tipo, como no protocolo.
        kind: String,
    },
    /// A ferramenta nao pode ser iniciada (provavelmente ausente).
    Spawn {
        /// Comando que falhou.
        command: String,
        /// Erro de IO subjacente.
        source: std::io::Error,
    },
    /// Falha de IO durante a execucao.
    Io(std::io::Error),
    /// A ferramenta rodou mas nao produziu dados de cobertura.
    NoData {
        /// Explicacao acionavel ("compile com --coverage", etc.).
        detail: String,
    },
}

impl fmt::Display for CoverageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unsupported { kind } => write!(
                formatter,
                "cobertura ainda nao e suportada para projetos do tipo {kind}"
            ),
            Self::Spawn { command, source } => {
                write!(formatter, "falha ao iniciar '{command}': {source}")
            }
            Self::Io(error) => write!(formatter, "falha de IO durante a cobertura: {error}"),
            Self::NoData { detail } => write!(formatter, "sem dados de cobertura: {detail}"),
        }
    }
}

impl Error for CoverageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Spawn { source, .. } | Self::Io(source) => Some(source),
            Self::Unsupported { .. } | Self::NoData { .. } => None,
        }
    }
}

/// Mede a cobertura do workspace, streamando as linhas de progresso.
pub fn run_coverage(
    root: &Path,
    kind: ProjectKind,
    cancel: &Arc<AtomicBool>,
    on_output: &mut dyn FnMut(&str),
) -> Result<CoverageReport, CoverageError> {
    match kind {
        ProjectKind::RustCargo => run_cargo_llvm_cov(root, cancel, on_output),
        ProjectKind::Cmake => run_lcov(root, cancel, on_output),
        other => Err(CoverageError::Unsupported {
            kind: project_kind_name(other),
        }),
    }
}

fn run_cargo_llvm_cov(
    root: &Path,
    cancel: &Arc<AtomicBool>,
    on_output: &mut dyn FnMut(&str),
) -> Result<CoverageReport, CoverageError> {
    // Subcomando do cargo: instala como binario `cargo-llvm-cov` e roda os
    // testes ja instrumentados. `--json` emite o export do llvm-cov.
    let mut command = Command::new("cargo");
    command
        .arg("llvm-cov")
        .arg("--workspace")
        .arg("--json")
        .current_dir(root);
    let mut stdout_json = String::new();
    let status = stream(command, "cargo llvm-cov", cancel, &mut |stream, line| {
        if stream == "stdout" {
            stdout_json.push_str(line);
            stdout_json.push('\n');
        } else {
            on_output(line);
        }
    })?;
    if !status {
        return Err(CoverageError::NoData {
            detail: "cargo llvm-cov terminou com falha (testes quebrados?)".to_owned(),
        });
    }
    parse_llvm_cov_json(&stdout_json).ok_or_else(|| CoverageError::NoData {
        detail: "saida do cargo llvm-cov sem o export JSON esperado".to_owned(),
    })
}

fn run_lcov(
    root: &Path,
    cancel: &Arc<AtomicBool>,
    on_output: &mut dyn FnMut(&str),
) -> Result<CoverageReport, CoverageError> {
    // Captura o que o build instrumentado ja gravou (.gcda). O lcov escreve
    // o tracefile; o parse e nosso. A IDE NAO injeta --coverage no build do
    // usuario (mesma regra do -Werror, docsprivate/diario/18 M4.5).
    let info_path = root.join(".kinein").join("coverage.info");
    let mut command = Command::new("lcov");
    command
        .arg("--capture")
        .arg("--directory")
        .arg(root.join(".kinein").join("build"))
        .arg("--output-file")
        .arg(&info_path)
        .current_dir(root);
    let status = stream(command, "lcov --capture", cancel, &mut |_, line| {
        on_output(line);
    })?;
    if !status {
        return Err(CoverageError::NoData {
            detail: "lcov nao capturou dados — o projeto foi compilado com --coverage?".to_owned(),
        });
    }
    let info = std::fs::read_to_string(&info_path).map_err(CoverageError::Io)?;
    let report = parse_lcov_info(&info);
    if report.files.is_empty() {
        return Err(CoverageError::NoData {
            detail: "tracefile vazio — o projeto foi compilado com --coverage?".to_owned(),
        });
    }
    Ok(report)
}

fn stream(
    command: Command,
    display: &str,
    cancel: &Arc<AtomicBool>,
    on_line: &mut dyn FnMut(&'static str, &str),
) -> Result<bool, CoverageError> {
    let mut sink = |stream: &'static str, line: String| on_line(stream, &line);
    let status =
        process::stream_command_lines_cancelable(command, cancel, &mut sink).map_err(|error| {
            match error {
                ProcessError::Spawn(source) => CoverageError::Spawn {
                    command: display.to_owned(),
                    source,
                },
                ProcessError::Wait(source) => CoverageError::Io(source),
            }
        })?;
    Ok(status.success())
}

/// Parse do export JSON do `llvm-cov` (`cargo llvm-cov --json`):
/// `data[0].totals.lines.{count,covered}` + `data[0].files[]`.
#[must_use]
pub fn parse_llvm_cov_json(raw: &str) -> Option<CoverageReport> {
    let value: serde_json::Value = serde_json::from_str(raw.trim()).ok()?;
    let data = value.get("data")?.get(0)?;
    let lines = data.get("totals")?.get("lines")?;
    let total = lines.get("count")?.as_u64()?;
    let covered = lines.get("covered")?.as_u64()?;
    let files = data
        .get("files")
        .and_then(|files| files.as_array())
        .map(|files| {
            files
                .iter()
                .filter_map(|file| {
                    let path = file.get("filename")?.as_str()?.to_owned();
                    let lines = file.get("summary")?.get("lines")?;
                    Some(CoverageFileSummary {
                        path,
                        totals: CoverageTotals::from_lines(
                            lines.get("covered")?.as_u64()?,
                            lines.get("count")?.as_u64()?,
                        ),
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    Some(CoverageReport {
        totals: CoverageTotals::from_lines(covered, total),
        files,
    })
}

/// Parse do tracefile do lcov: registros `SF:` (arquivo), `LH:` (linhas
/// cobertas) e `LF:` (linhas instrumentadas), fechados por `end_of_record`.
#[must_use]
pub fn parse_lcov_info(raw: &str) -> CoverageReport {
    let mut files = Vec::new();
    let mut current: Option<(String, u64, u64)> = None;
    for line in raw.lines() {
        let line = line.trim();
        if let Some(path) = line.strip_prefix("SF:") {
            current = Some((path.to_owned(), 0, 0));
        } else if let Some(value) = line.strip_prefix("LH:") {
            if let Some(entry) = current.as_mut() {
                entry.1 = value.trim().parse().unwrap_or(0);
            }
        } else if let Some(value) = line.strip_prefix("LF:") {
            if let Some(entry) = current.as_mut() {
                entry.2 = value.trim().parse().unwrap_or(0);
            }
        } else if line == "end_of_record" {
            if let Some((path, covered, total)) = current.take() {
                files.push(CoverageFileSummary {
                    path,
                    totals: CoverageTotals::from_lines(covered, total),
                });
            }
        }
    }
    let covered = files.iter().map(|f| f.totals.lines_covered).sum();
    let total = files.iter().map(|f| f.totals.lines_total).sum();
    CoverageReport {
        totals: CoverageTotals::from_lines(covered, total),
        files,
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_lcov_info, parse_llvm_cov_json};

    #[test]
    fn parse_llvm_cov_json_le_totais_e_arquivos() {
        let raw = r#"{"data":[{"totals":{"lines":{"count":100,"covered":80,"percent":80.0}},
            "files":[{"filename":"src/main.rs",
                      "summary":{"lines":{"count":10,"covered":9,"percent":90.0}}}]}]}"#;
        let report = parse_llvm_cov_json(raw).unwrap();
        assert_eq!(report.totals.lines_total, 100);
        assert_eq!(report.totals.lines_covered, 80);
        assert_eq!(report.files.len(), 1);
        assert_eq!(report.files[0].path, "src/main.rs");
        assert!((report.files[0].totals.percent - 90.0).abs() < 0.001);
        // JSON que nao e o export nao vira relatorio fantasioso.
        assert!(parse_llvm_cov_json("{}").is_none());
        assert!(parse_llvm_cov_json("nao e json").is_none());
    }

    #[test]
    fn parse_lcov_info_soma_arquivos_e_fecha_por_end_of_record() {
        let raw = "TN:\nSF:/ws/src/a.c\nLF:10\nLH:7\nend_of_record\n\
                   SF:/ws/src/b.c\nLF:30\nLH:15\nend_of_record\n";
        let report = parse_lcov_info(raw);
        assert_eq!(report.files.len(), 2);
        assert_eq!(report.totals.lines_total, 40);
        assert_eq!(report.totals.lines_covered, 22);
        assert!((report.totals.percent - 55.0).abs() < 0.001);
        // Registro sem end_of_record NAO conta (arquivo truncado).
        let truncado = parse_lcov_info("SF:/ws/x.c\nLF:5\nLH:5\n");
        assert!(truncado.files.is_empty());
    }
}

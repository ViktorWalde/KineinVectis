//! Coverage payloads (`coverage.run`, L2 fatia 3).
//!
//! A cobertura e um JOB: `coverage.run` aceita o workspace aberto e o
//! resultado chega por `event.coverage.finished`. O core orquestra as
//! ferramentas maduras (`cargo-llvm-cov` para Rust, `lcov` para C/C++) e
//! normaliza os numeros; nada aqui reimplementa instrumentacao.

use serde::{Deserialize, Serialize};

use crate::BuildSystem;

/// Parametros de `coverage.run`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CoverageRunParams {
    /// Sistema de build alvo em workspace hibrido (`cargo` | `cmake`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub build_system: Option<BuildSystem>,
}

/// Totais de linhas cobertas de um arquivo ou do workspace.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoverageTotals {
    /// Linhas executadas ao menos uma vez.
    pub lines_covered: u64,
    /// Linhas instrumentadas.
    pub lines_total: u64,
    /// Percentual coberto (0-100), derivado dos dois acima.
    pub percent: f64,
}

impl CoverageTotals {
    /// Deriva o percentual; zero linhas instrumentadas = 0%.
    #[must_use]
    pub fn from_lines(lines_covered: u64, lines_total: u64) -> Self {
        #[allow(clippy::cast_precision_loss)]
        let percent = if lines_total == 0 {
            0.0
        } else {
            (lines_covered as f64 / lines_total as f64) * 100.0
        };
        Self {
            lines_covered,
            lines_total,
            percent,
        }
    }
}

/// Cobertura de um arquivo.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoverageFileSummary {
    /// Caminho do arquivo como reportado pela ferramenta.
    pub path: String,
    /// Totais do arquivo.
    #[serde(flatten)]
    pub totals: CoverageTotals,
}

#[cfg(test)]
mod tests {
    use super::{CoverageFileSummary, CoverageTotals};

    #[test]
    fn totals_derivam_percent_e_zero_instrumentado_nao_divide_por_zero() {
        let metade = CoverageTotals::from_lines(50, 100);
        assert!((metade.percent - 50.0).abs() < f64::EPSILON);
        let vazio = CoverageTotals::from_lines(0, 0);
        assert!((vazio.percent - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn file_summary_achata_totais_em_camel_case() {
        let json = serde_json::to_value(CoverageFileSummary {
            path: "src/main.rs".to_owned(),
            totals: CoverageTotals::from_lines(8, 10),
        })
        .unwrap();
        assert_eq!(json["path"], "src/main.rs");
        assert_eq!(json["linesCovered"], 8);
        assert_eq!(json["linesTotal"], 10);
    }
}

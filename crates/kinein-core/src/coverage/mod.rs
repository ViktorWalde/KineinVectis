//! Cobertura de linhas dos testes (D8 do `roadmaps/41`, P5 do 40 §4.1,
//! 2026-09-17), com o LCOV como lingua comum.
//!
//! ```text
//! Rust     cargo llvm-cov --lcov --output-path .kinein/coverage.lcov
//!          (cargo-llvm-cov 0.9.1 desta maquina: `--lcov` + `--output-path`;
//!          exige o componente llvm-tools do rustup — o proprio cargo-llvm-cov
//!          diz `rustup component add llvm-tools-preview` quando falta)
//! Python   <python> -m coverage run -m pytest -q && <python> -m coverage lcov
//!          -o .kinein/coverage.lcov   (coverage.py: `coverage lcov`; instalado
//!          NO ambiente do projeto, como o pytest)
//! C/C++    nao entra ainda: exige compilar com `--coverage` (gcov/lcov ou
//!          llvm-cov) — o build do usuario nao e' tocado sem decisao dele
//! ```
//!
//! O arquivo fica em `<root>/.kinein/coverage.lcov` e e' lido de volta: um
//! resumo por arquivo no `event.coverage.finished`, e as linhas de UM arquivo
//! sob pedido (`coverage.lines`), para o editor pintar a calha.

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    process::Command,
    sync::{Arc, atomic::AtomicBool},
};

use kinein_protocol::{CoverageFileSummary, CoverageLinesResult, ProjectKind};

use crate::process;
use crate::python::run::PythonLauncher;

/// Onde o LCOV fica, relativo a raiz.
pub const LCOV_PATH: &str = ".kinein/coverage.lcov";

/// O que uma execucao contou.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoverageEvent {
    /// Um comando comecou.
    Started {
        /// A linha, para a tela.
        command: String,
    },
    /// Uma linha de saida.
    Output {
        /// `stdout`|`stderr`.
        stream: &'static str,
        /// A linha.
        line: String,
    },
}

/// Por que nao houve cobertura.
#[derive(Debug)]
pub enum CoverageError {
    /// Tipo de projeto sem ferramenta de cobertura.
    Unsupported {
        /// O tipo.
        kind: String,
        /// O que faria funcionar.
        hint: String,
    },
    /// A ferramenta nao esta' aqui.
    ToolMissing {
        /// A ferramenta.
        tool: String,
        /// O passo.
        hint: String,
    },
    /// O comando falhou.
    Failed {
        /// O comando.
        command: String,
        /// O status.
        status: String,
    },
    /// Disco/processo.
    Io(std::io::Error),
}

impl std::fmt::Display for CoverageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unsupported { kind, hint } => {
                write!(f, "cobertura nao suportada para {kind}: {hint}")
            }
            Self::ToolMissing { tool, hint } => write!(f, "{tool} nao esta' nesta maquina: {hint}"),
            Self::Failed { command, status } => write!(f, "`{command}` saiu com {status}"),
            Self::Io(e) => write!(f, "{e}"),
        }
    }
}

/// O LCOV lido: por arquivo, linha -> hits.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Report {
    /// `SF:` -> (`DA:` linha, hits).
    pub files: BTreeMap<String, BTreeMap<u32, u64>>,
}

impl Report {
    /// Le um texto LCOV (`SF:`, `DA:linha,hits`, `end_of_record`; `LF:`/`LH:`
    /// sao redundantes com os `DA:` e nao sao lidos).
    #[must_use]
    pub fn parse(texto: &str) -> Self {
        let mut files: BTreeMap<String, BTreeMap<u32, u64>> = BTreeMap::new();
        let mut atual: Option<String> = None;
        for linha in texto.lines() {
            let linha = linha.trim();
            if let Some(sf) = linha.strip_prefix("SF:") {
                atual = Some(sf.to_owned());
                files.entry(sf.to_owned()).or_default();
            } else if let Some(da) = linha.strip_prefix("DA:")
                && let Some(arquivo) = &atual
                && let Some((l, h)) = da.split(',').next().zip(da.split(',').nth(1))
                && let (Ok(l), Ok(h)) = (l.trim().parse::<u32>(), h.trim().parse::<u64>())
            {
                let entrada = files.entry(arquivo.clone()).or_default();
                *entrada.entry(l).or_insert(0) += h;
            } else if linha == "end_of_record" {
                atual = None;
            }
        }
        Self { files }
    }

    /// Le `<root>/.kinein/coverage.lcov`; vazio quando nao existe.
    #[must_use]
    pub fn load(root: &Path) -> Self {
        std::fs::read_to_string(root.join(LCOV_PATH))
            .map(|t| Self::parse(&t))
            .unwrap_or_default()
    }

    /// Um resumo por arquivo, na ordem do relatorio.
    #[must_use]
    pub fn summary(&self) -> Vec<CoverageFileSummary> {
        self.files
            .iter()
            .map(|(file, linhas)| CoverageFileSummary {
                file: file.clone(),
                lines_found: linhas.len() as u64,
                lines_hit: linhas.values().filter(|h| **h > 0).count() as u64,
            })
            .collect()
    }

    /// As linhas de um arquivo (caminho exato, ou o mesmo canonico).
    #[must_use]
    pub fn lines(&self, file: &str) -> CoverageLinesResult {
        let canonico = std::fs::canonicalize(file).ok();
        let achado = self.files.get(file).or_else(|| {
            let alvo = canonico.as_deref()?;
            self.files
                .iter()
                .find(|(f, _)| std::fs::canonicalize(f).ok().as_deref() == Some(alvo))
                .map(|(_, l)| l)
        });
        let Some(linhas) = achado else {
            return CoverageLinesResult {
                file: file.to_owned(),
                known: false,
                covered: Vec::new(),
                missed: Vec::new(),
            };
        };
        CoverageLinesResult {
            file: file.to_owned(),
            known: true,
            covered: linhas
                .iter()
                .filter(|(_, h)| **h > 0)
                .map(|(l, _)| *l)
                .collect(),
            missed: linhas
                .iter()
                .filter(|(_, h)| **h == 0)
                .map(|(l, _)| *l)
                .collect(),
        }
    }
}

/// Com que a cobertura roda, resolvido pelo handler.
#[derive(Debug, Clone, Default)]
pub struct CoverageTools {
    /// `cargo` (o do kit ou o nu).
    pub cargo: Option<PathBuf>,
    /// `cargo-llvm-cov` detectado.
    pub cargo_llvm_cov: Option<PathBuf>,
    /// O lancador Python do projeto.
    pub python: Option<PythonLauncher>,
}

/// Roda a ferramenta do tipo e escreve o LCOV; devolve o nome da ferramenta
/// e o relatorio lido.
///
/// # Errors
///
/// Tipo sem ferramenta, ferramenta ausente, comando com erro, disco.
pub fn run(
    root: &Path,
    kind: ProjectKind,
    tools: &CoverageTools,
    cancel: &Arc<AtomicBool>,
    sink: &mut dyn FnMut(CoverageEvent),
) -> Result<(String, Report), CoverageError> {
    let lcov = root.join(LCOV_PATH);
    if let Some(pasta) = lcov.parent() {
        std::fs::create_dir_all(pasta).map_err(CoverageError::Io)?;
    }
    let _ = std::fs::remove_file(&lcov);
    let tool = match kind {
        ProjectKind::RustCargo => {
            if tools.cargo_llvm_cov.is_none() {
                return Err(CoverageError::ToolMissing {
                    tool: "cargo-llvm-cov".to_owned(),
                    hint: "`cargo install cargo-llvm-cov` e `rustup component add llvm-tools-preview` \
                           (README do cargo-llvm-cov)"
                        .to_owned(),
                });
            }
            let mut c = Command::new(
                tools
                    .cargo
                    .clone()
                    .unwrap_or_else(|| PathBuf::from("cargo")),
            );
            c.args(["llvm-cov", "--lcov", "--output-path"])
                .arg(&lcov)
                .current_dir(root);
            rodar(
                c,
                "cargo llvm-cov --lcov --output-path .kinein/coverage.lcov",
                cancel,
                sink,
            )?;
            "cargo llvm-cov"
        }
        ProjectKind::Python => {
            let Some(python) = &tools.python else {
                return Err(CoverageError::ToolMissing {
                    tool: "python".to_owned(),
                    hint: "sem interpretador para este projeto: crie o ambiente (.venv)".to_owned(),
                });
            };
            let (program, prefix) = python.program();
            let mut c = Command::new(&program);
            c.args(&prefix)
                .args(["-m", "coverage", "run", "-m", "pytest", "-q"])
                .current_dir(root);
            let rotulo = format!("{} -m coverage run -m pytest -q", python.display(root));
            rodar(c, &rotulo, cancel, sink).map_err(sem_coverage_py)?;
            let mut c = Command::new(&program);
            c.args(&prefix)
                .args(["-m", "coverage", "lcov", "-o"])
                .arg(&lcov)
                .current_dir(root);
            rodar(
                c,
                &format!(
                    "{} -m coverage lcov -o .kinein/coverage.lcov",
                    python.display(root)
                ),
                cancel,
                sink,
            )?;
            "coverage.py"
        }
        other => {
            return Err(CoverageError::Unsupported {
                kind: format!("{other:?}").to_lowercase(),
                hint: "C/C++ exige compilar com `--coverage` (gcov/lcov) — nao entra sem a sua \
                       decisao no build; Rust usa cargo-llvm-cov, Python o coverage.py"
                    .to_owned(),
            });
        }
    };
    let texto = std::fs::read_to_string(&lcov).map_err(CoverageError::Io)?;
    Ok((tool.to_owned(), Report::parse(&texto)))
}

/// `No module named coverage` vira o passo de instalar no ambiente.
fn sem_coverage_py(erro: CoverageError) -> CoverageError {
    match erro {
        CoverageError::Failed { command, status } if command.contains("coverage run") => {
            CoverageError::ToolMissing {
                tool: "coverage".to_owned(),
                hint: format!(
                    "instale-o NO ambiente do projeto: `uv add --dev coverage pytest` ou \
                     `.venv/bin/python -m pip install coverage pytest` (o comando saiu com {status})"
                ),
            }
        }
        outro => outro,
    }
}

fn rodar(
    command: Command,
    display: &str,
    cancel: &Arc<AtomicBool>,
    sink: &mut dyn FnMut(CoverageEvent),
) -> Result<(), CoverageError> {
    sink(CoverageEvent::Started {
        command: display.to_owned(),
    });
    let mut on_line =
        |stream: &'static str, line: String| sink(CoverageEvent::Output { stream, line });
    let status = process::stream_command_lines_cancelable(command, cancel, &mut on_line).map_err(
        |e| match e {
            process::ProcessError::Spawn(e) | process::ProcessError::Wait(e) => {
                CoverageError::Io(e)
            }
        },
    )?;
    if !status.success() {
        return Err(CoverageError::Failed {
            command: display.to_owned(),
            status: status.to_string(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::Report;

    #[test]
    fn lcov_is_read_per_file_and_summarised() {
        let r = Report::parse(
            "TN:\nSF:/p/src/lib.rs\nDA:1,3\nDA:2,0\nDA:5,1\nLF:3\nLH:2\nend_of_record\nSF:/p/src/b.rs\nDA:7,0\nDA:7,2\nend_of_record\nlixo\n",
        );
        let resumo = r.summary();
        assert_eq!(resumo.len(), 2);
        assert_eq!(resumo[1].file, "/p/src/lib.rs");
        assert_eq!((resumo[1].lines_found, resumo[1].lines_hit), (3, 2));
        // A mesma linha duas vezes soma (os `DA:` de blocos se repetem).
        assert_eq!((resumo[0].lines_found, resumo[0].lines_hit), (1, 1));
        let linhas = r.lines("/p/src/lib.rs");
        assert!(linhas.known);
        assert_eq!(linhas.covered, [1, 5]);
        assert_eq!(linhas.missed, [2]);
        assert!(!r.lines("/p/src/nao.rs").known);
        assert!(Report::parse("").files.is_empty());
    }
}

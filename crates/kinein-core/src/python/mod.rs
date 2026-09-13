//! O dominio `python`: o AMBIENTE do projeto Python.
//!
//! Bloco B do `roadmaps/41`, cadeia iniciada em 2026-09-12: que interpretador
//! vale, se ha' um ambiente proprio, e como criar um sem editar arquivo nenhum.
//!
//! ```text
//! env       a precedencia do interpretador (roadmaps/29 §4.1), compartilhada
//!           com o index.context
//! status()  o que o painel mostra: interpretador, se e' ambiente proprio ou o
//!           Python do sistema, que ferramenta cria ambiente (uv > venv), que
//!           arquivos de projeto existem
//! create_environment_command()   `uv venv .venv` ou `python3 -m venv .venv`,
//!           como a fonte oficial escreve — rodado como JOB, com a saida na tela
//! ```

pub mod env;

use std::path::{Path, PathBuf};
use std::process::Command;

use kinein_protocol::{PythonEnvironmentTool, PythonStatus};

use self::env::PythonTools;

/// Nome da pasta que a IDE cria; e' a convencao que o uv cria por padrao e a
/// que todo editor procura primeiro (29 §4.1).
pub const PASTA_DO_AMBIENTE: &str = ".venv";

/// Arquivos que dizem "isto e' um projeto Python", na ordem em que a tela os lista.
const ARQUIVOS_DE_PROJETO: &[&str] = &[
    "pyproject.toml",
    "requirements.txt",
    "setup.py",
    "uv.lock",
    "poetry.lock",
    "Pipfile",
];

/// O que a IDE tem para criar ambiente nesta maquina.
#[derive(Debug, Clone, Default)]
pub struct CriadoresDeAmbiente {
    /// O `uv`, quando detectado.
    pub uv: Option<PathBuf>,
}

/// O estado do Python do workspace.
#[must_use]
pub fn status(root: &Path, tools: &PythonTools, criadores: &CriadoresDeAmbiente) -> PythonStatus {
    let interpreter = env::python_env(root, tools);
    let has_environment = interpreter.as_ref().is_some_and(|e| e.origin != "sistema");
    let project_files: Vec<String> = ARQUIVOS_DE_PROJETO
        .iter()
        .filter(|f| root.join(f).is_file())
        .map(|f| (*f).to_owned())
        .collect();
    let environment_tool = if criadores.uv.is_some() {
        Some(PythonEnvironmentTool::Uv)
    } else if tools.python_sistema.is_some() {
        Some(PythonEnvironmentTool::Venv)
    } else {
        None
    };
    let hint = match (&interpreter, environment_tool) {
        // O uv cria o ambiente mesmo sem python3 no PATH: ele baixa um Python
        // (docs.astral.sh/uv/concepts/python-versions, 2026-09-12).
        (None, Some(PythonEnvironmentTool::Uv)) => Some(
            "nenhum Python no PATH, mas ha' uv: crie o ambiente (.venv) com ele — o uv baixa um \
             Python se preciso"
                .to_owned(),
        ),
        (None, _) => Some(
            "nenhum Python encontrado: instale o python3 da sua distro (o painel de instalacao \
             mostra o passo oficial)"
                .to_owned(),
        ),
        (Some(e), Some(PythonEnvironmentTool::Uv)) if e.origin == "sistema" => Some(
            "o projeto usa o Python do SISTEMA: crie um ambiente proprio (.venv) com o uv — um \
             clique, o comando aparece no job"
                .to_owned(),
        ),
        (Some(e), Some(PythonEnvironmentTool::Venv)) if e.origin == "sistema" => Some(
            "o projeto usa o Python do SISTEMA: crie um ambiente proprio (.venv) com `python3 -m \
             venv`; o uv (mais rapido) esta' no painel de instalacao"
                .to_owned(),
        ),
        _ => None,
    };
    PythonStatus {
        interpreter,
        has_environment,
        environment_tool,
        uv: criadores.uv.as_ref().map(|p| p.display().to_string()),
        project_files,
        hint,
    }
}

/// O comando que cria o ambiente, exatamente como a fonte oficial escreve.
///
/// `uv venv .venv` (docs.astral.sh/uv/pip/environments, 2026-09-12) ou
/// `python3 -m venv .venv` (docs.python.org/3/library/venv.html). `None` sem
/// ferramenta nenhuma.
#[must_use]
pub fn create_environment_command(
    root: &Path,
    tool: PythonEnvironmentTool,
    criadores: &CriadoresDeAmbiente,
    python_sistema: Option<&Path>,
) -> Option<(Command, String)> {
    let (mut comando, rotulo) = match tool {
        PythonEnvironmentTool::Uv => {
            let mut c = Command::new(criadores.uv.as_deref()?);
            c.args(["venv", PASTA_DO_AMBIENTE]);
            (c, format!("uv venv {PASTA_DO_AMBIENTE}"))
        }
        PythonEnvironmentTool::Venv => {
            let py = python_sistema?;
            let mut c = Command::new(py);
            c.args(["-m", "venv", PASTA_DO_AMBIENTE]);
            (c, format!("{} -m venv {PASTA_DO_AMBIENTE}", py.display()))
        }
    };
    comando.current_dir(root);
    Some((comando, rotulo))
}

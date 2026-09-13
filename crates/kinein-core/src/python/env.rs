//! O interpretador Python do projeto, na precedencia do `roadmaps/29` §4.1.
//!
//! E' o `compile_commands.json` do Python: sem o interpretador certo, o
//! language server indexa a stdlib errada, o pytest roda no ambiente errado e
//! o debugpy nao acha o modulo. Por isso ele e' resolvido UMA vez aqui e lido
//! por todos — o `index.context`, o `python.status`, e (nas fatias seguintes)
//! o basedpyright, o run, o pytest e o debugpy.

use std::path::{Path, PathBuf};
use std::process::Command;

use kinein_protocol::PythonEnv;

/// O que o resolvedor pode EXECUTAR e LER, injetado: o teste passa `None` e
/// nada da maquina entra — a regra do `Ferramentas` do indice.
#[derive(Debug, Clone, Default)]
pub struct PythonTools {
    /// O `poetry` a perguntar pelo ambiente; `None` = nao perguntar.
    pub poetry: Option<PathBuf>,
    /// O `python3` do sistema, ultimo recurso; `None` = sem sistema.
    pub python_sistema: Option<PathBuf>,
    /// `$VIRTUAL_ENV`, lido por quem chama.
    pub virtual_env: Option<String>,
    /// Rodar `python --version` no interpretador achado.
    pub medir_versao: bool,
}

/// A precedencia do `roadmaps/29` §4.1. Cada degrau so' vale se o
/// interpretador EXISTE; `poetry` e `sistema` so' se a ferramenta foi dada.
#[must_use]
pub fn python_env(root: &Path, f: &PythonTools) -> Option<PythonEnv> {
    let com_versao = |interpreter: PathBuf, origin: &str, warning: Option<&str>| PythonEnv {
        version: f.medir_versao.then(|| versao_de(&interpreter)).flatten(),
        interpreter: interpreter.display().to_string(),
        origin: origin.to_owned(),
        warning: warning.map(str::to_owned),
    };
    if let Some(venv) = f.virtual_env.as_deref().filter(|v| !v.is_empty()) {
        let py = Path::new(venv).join("bin").join("python");
        if py.is_file() {
            return Some(com_versao(py, "VIRTUAL_ENV", None));
        }
    }
    for pasta in [".venv", "venv", "env"] {
        let py = root.join(pasta).join("bin").join("python");
        if py.is_file() {
            return Some(com_versao(py, pasta, None));
        }
    }
    if root.join("poetry.lock").is_file() {
        if let Some(poetry) = f.poetry.as_deref() {
            if let Ok(saida) = Command::new(poetry)
                .args(["env", "info", "-p"])
                .current_dir(root)
                .output()
            {
                if saida.status.success() {
                    let caminho = String::from_utf8_lossy(&saida.stdout).trim().to_owned();
                    let py = Path::new(&caminho).join("bin").join("python");
                    if py.is_file() {
                        return Some(com_versao(py, "poetry", None));
                    }
                }
            }
        }
    }
    f.python_sistema.clone().map(|py| {
        com_versao(
            py,
            "sistema",
            Some("Python do sistema: instalar pacote nele quebra a distro; crie um .venv"),
        )
    })
}

fn versao_de(interpreter: &Path) -> Option<String> {
    let saida = Command::new(interpreter).arg("--version").output().ok()?;
    let texto = String::from_utf8_lossy(&saida.stdout);
    let texto = if texto.trim().is_empty() {
        String::from_utf8_lossy(&saida.stderr).into_owned()
    } else {
        texto.into_owned()
    };
    let linha = texto.trim();
    (!linha.is_empty()).then(|| linha.to_owned())
}

//! Como a IDE EXECUTA Python (fatia 3 da cadeia do `roadmaps/41` bloco B).
//!
//! Duas perguntas, respondidas por evidencia no disco e nunca por adivinhacao:
//!
//! ```text
//! COM QUE          o interpretador do projeto (python::env, a precedencia do
//!                  29 §4.1) — ou `uv run` quando o projeto E' do uv (tem
//!                  `uv.lock`) e o uv existe nesta maquina: o uv sincroniza o
//!                  ambiente com o lock antes de rodar, que e' o que o autor do
//!                  projeto quis ao adotar o uv
//! O QUE            o ponto de entrada do botao Executar: `main.py`, `app.py`
//!                  ou `__main__.py` na raiz; um pacote com `__main__.py`
//!                  (`<pacote>/` ou `src/<pacote>/`) vira `-m <pacote>`; um
//!                  script de `[project.scripts]` ja' instalado no ambiente
//!                  (`.venv/bin/<nome>`) roda por ele. Sem nada disso, a IDE
//!                  diz o que procurou — nao inventa
//! ```
//!
//! O `run.script` de um `.py` (clique direito em "Executar") usa o mesmo COM
//! QUE, com o arquivo como argumento.
//!
//! **`MicroPython`** (fatia 5, 2026-09-13) e' outro COM QUE: o arquivo roda NA
//! PLACA, por `mpremote [connect <porta>] run <arquivo>` (mpremote 1.29.0,
//! `mpremote run --help`: `run [--follow] path`; sem `connect` ele usa a
//! primeira porta serial que acha). Um `main.py` que importa `machine` nao
//! roda no Python do desktop — o host nao tem os pinos.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use crate::run::RunError;

use super::env::{PythonTools, python_env};

/// Com que o Python do projeto e' lancado.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PythonLauncher {
    /// `uv run python …` — o projeto tem `uv.lock` e o uv existe.
    Uv(PathBuf),
    /// O interpretador resolvido pela precedencia do 29 §4.1.
    Interpreter(PathBuf),
    /// `mpremote [connect <porta>] …` — o projeto e' `MicroPython` e o arquivo
    /// roda na placa. `device` = a porta escolhida; `None` = a primeira que
    /// o mpremote achar.
    Mpremote {
        /// O executavel detectado.
        program: PathBuf,
        /// A porta serial, quando a UI a deu.
        device: Option<String>,
    },
}

impl PythonLauncher {
    /// Resolve o lancador para `root`: `uv` (quando dado E o projeto tem
    /// `uv.lock`) vence o interpretador; sem nenhum dos dois, `None`.
    #[must_use]
    pub fn resolve(root: &Path, tools: &PythonTools, uv: Option<PathBuf>) -> Option<Self> {
        if let Some(uv) = uv.filter(|_| root.join("uv.lock").is_file()) {
            return Some(Self::Uv(uv));
        }
        let env = python_env(root, tools)?;
        Some(Self::Interpreter(PathBuf::from(env.interpreter)))
    }

    /// O lancador de um projeto `MicroPython`: o `mpremote` detectado e a
    /// porta escolhida (`None` = a primeira que ele achar). Sem mpremote o
    /// chamador diz para instala-lo, em vez de rodar um `main.py` de placa no
    /// Python do desktop.
    #[must_use]
    pub fn mpremote(program: PathBuf, device: Option<&str>) -> Self {
        Self::Mpremote {
            program,
            device: device.map(str::to_owned),
        }
    }

    /// O programa a executar e os argumentos que vem ANTES dos do usuario.
    ///
    /// No mpremote o "argumento do usuario" e' o arquivo: os prefixos sao
    /// `connect <porta>` (quando ha') e `run`.
    #[must_use]
    pub fn program(&self) -> (PathBuf, Vec<OsString>) {
        match self {
            Self::Uv(uv) => (uv.clone(), vec!["run".into(), "python".into()]),
            Self::Interpreter(python) => (python.clone(), Vec::new()),
            Self::Mpremote { program, device } => {
                let mut prefix: Vec<OsString> = Vec::new();
                if let Some(device) = device {
                    prefix.push("connect".into());
                    prefix.push(device.into());
                }
                prefix.push("run".into());
                (program.clone(), prefix)
            }
        }
    }

    /// Como a tela mostra o lancador: `uv run python`, `mpremote [connect
    /// <porta>] run`, ou o interpretador relativo ao root quando mora dentro
    /// dele (`.venv/bin/python`).
    #[must_use]
    pub fn display(&self, root: &Path) -> String {
        match self {
            Self::Uv(_) => "uv run python".to_owned(),
            Self::Interpreter(python) => python
                .strip_prefix(root)
                .unwrap_or(python)
                .display()
                .to_string(),
            Self::Mpremote { device, .. } => device.as_ref().map_or_else(
                || "mpremote run".to_owned(),
                |porta| format!("mpremote connect {porta} run"),
            ),
        }
    }

    /// Como a tela mostra a execucao de UM arquivo: o lancador e o caminho
    /// relativo ao root entre aspas (`.venv/bin/python 'tools/gera.py'`).
    #[must_use]
    pub fn script_display(&self, root: &Path, script: &Path) -> String {
        let relativo = script.strip_prefix(root).unwrap_or(script);
        format!(
            "{} {}",
            self.display(root),
            aspas(&relativo.display().to_string())
        )
    }

    /// A linha de shell (para `run.start`) que executa `args` com este
    /// lancador, cada argumento entre aspas simples.
    #[must_use]
    pub fn shell_command(&self, args: &[&str]) -> String {
        let (program, prefix) = self.program();
        let mut partes = vec![aspas(&program.display().to_string())];
        partes.extend(prefix.iter().map(|p| p.to_string_lossy().into_owned()));
        partes.extend(args.iter().map(|a| aspas(a)));
        partes.join(" ")
    }
}

fn aspas(texto: &str) -> String {
    format!("'{}'", texto.replace('\'', "'\\''"))
}

/// O ponto de entrada que o botao Executar usa, por evidencia.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntryPoint {
    /// Um arquivo na raiz (`main.py`, `app.py`, `__main__.py`).
    File(String),
    /// Um pacote com `__main__.py`: `python -m <pacote>`.
    Module(String),
    /// Um script de `[project.scripts]` instalado em `.venv/bin/<nome>`.
    InstalledScript(String),
}

/// Arquivos na raiz que valem como ponto de entrada, nesta ordem.
const ARQUIVOS_DE_ENTRADA: &[&str] = &["main.py", "app.py", "__main__.py"];

/// Procura o ponto de entrada de `root`.
///
/// Arquivo na raiz > pacote com `__main__.py` (raiz, depois `src/`) > script
/// de `[project.scripts]` instalado. Dois pacotes candidatos = ambiguidade =
/// `None` (a IDE nao escolhe por ordem alfabetica).
#[must_use]
pub fn entry_point(root: &Path) -> Option<EntryPoint> {
    if let Some(arquivo) = ARQUIVOS_DE_ENTRADA
        .iter()
        .find(|nome| root.join(nome).is_file())
    {
        return Some(EntryPoint::File((*arquivo).to_owned()));
    }
    for base in [root.to_path_buf(), root.join("src")] {
        let pacotes = pacotes_com_main(&base);
        match pacotes.as_slice() {
            [unico] => return Some(EntryPoint::Module(unico.clone())),
            [] => {}
            _ambiguos => return None,
        }
    }
    project_scripts(root)
        .into_iter()
        .find(|nome| root.join(".venv/bin").join(nome).is_file())
        .map(EntryPoint::InstalledScript)
}

/// Pastas filhas de `base` que sao pacotes com `__main__.py`, ordenadas.
fn pacotes_com_main(base: &Path) -> Vec<String> {
    let Ok(entradas) = std::fs::read_dir(base) else {
        return Vec::new();
    };
    let mut pacotes: Vec<String> = entradas
        .flatten()
        .filter(|e| e.path().join("__main__.py").is_file())
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|nome| !nome.starts_with('.'))
        .collect();
    pacotes.sort();
    pacotes
}

/// Os nomes de `[project.scripts]` do `pyproject.toml`, na ordem do arquivo.
/// Leitura de linhas (chave = valor dentro da tabela), sem parser TOML: o
/// que se quer e' o NOME, e ele e' a chave.
fn project_scripts(root: &Path) -> Vec<String> {
    let Ok(texto) = std::fs::read_to_string(root.join("pyproject.toml")) else {
        return Vec::new();
    };
    let mut dentro = false;
    let mut nomes = Vec::new();
    for linha in texto.lines().map(str::trim) {
        if linha.starts_with('[') {
            dentro = linha == "[project.scripts]";
            continue;
        }
        if !dentro || linha.is_empty() || linha.starts_with('#') {
            continue;
        }
        if let Some((chave, _valor)) = linha.split_once('=') {
            nomes.push(chave.trim().trim_matches('"').to_owned());
        }
    }
    nomes
}

/// O comando padrao do botao Executar para um projeto Python, como linha de
/// shell. Erra dizendo o que procurou quando nao ha' lancador ou entrada.
pub fn default_command(root: &Path, launcher: Option<&PythonLauncher>) -> Result<String, RunError> {
    let Some(launcher) = launcher else {
        return Err(RunError::NoDefaultCommand {
            message: "sem interpretador Python para este projeto: crie o ambiente (.venv) \
                      pela faixa de saude ou instale o python3"
                .to_owned(),
        });
    };
    match entry_point(root) {
        Some(EntryPoint::File(arquivo)) => Ok(launcher.shell_command(&[&arquivo])),
        // Na placa nao ha' `-m` nem script instalado: o mpremote roda ARQUIVOS.
        Some(EntryPoint::Module(pacote)) if matches!(launcher, PythonLauncher::Mpremote { .. }) => {
            Err(RunError::NoDefaultCommand {
                message: format!(
                    "projeto MicroPython sem main.py na raiz (o ponto de entrada achado e' o \
                     pacote `{pacote}`, que a placa nao roda com -m); clique com o direito \
                     num .py e escolha Executar"
                ),
            })
        }
        Some(EntryPoint::Module(pacote)) => Ok(launcher.shell_command(&["-m", &pacote])),
        Some(EntryPoint::InstalledScript(nome)) => Ok(aspas(
            &root.join(".venv/bin").join(nome).display().to_string(),
        )),
        None => Err(RunError::NoDefaultCommand {
            message: "nenhum ponto de entrada Python: procurei main.py, app.py e __main__.py na \
                      raiz, UM pacote com __main__.py (raiz ou src/) e um script de \
                      [project.scripts] instalado em .venv/bin; clique com o direito num .py \
                      e escolha Executar, ou digite o comando no painel Terminal"
                .to_owned(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::{EntryPoint, PythonLauncher, default_command, entry_point};
    use crate::python::env::PythonTools;

    fn raiz(nome: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("kinein-core-tests")
            .join(format!("{}-python-run-{nome}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn com_venv(root: &Path) -> PathBuf {
        let py = root.join(".venv/bin/python");
        std::fs::create_dir_all(py.parent().unwrap()).unwrap();
        std::fs::write(&py, "").unwrap();
        py
    }

    /// O uv so' lanca quando o projeto E' do uv (uv.lock): ter o uv na
    /// maquina nao muda como um projeto com requirements.txt roda.
    #[test]
    fn uv_only_launches_projects_that_have_a_lockfile() {
        let root = raiz("uv-lock");
        let py = com_venv(&root);
        let tools = PythonTools::default();
        let uv = Some(PathBuf::from("/opt/uv"));
        assert_eq!(
            PythonLauncher::resolve(&root, &tools, uv.clone()),
            Some(PythonLauncher::Interpreter(py.clone()))
        );
        std::fs::write(root.join("uv.lock"), "").unwrap();
        assert_eq!(
            PythonLauncher::resolve(&root, &tools, uv),
            Some(PythonLauncher::Uv(PathBuf::from("/opt/uv")))
        );
        // Sem uv dado, o lock nao basta: volta ao interpretador.
        assert_eq!(
            PythonLauncher::resolve(&root, &tools, None),
            Some(PythonLauncher::Interpreter(py))
        );
        // Sem interpretador e sem uv: nada.
        let vazio = raiz("uv-lock-vazio");
        assert_eq!(PythonLauncher::resolve(&vazio, &tools, None), None);
    }

    #[test]
    fn the_launcher_builds_program_display_and_shell_line() {
        let root = raiz("launcher");
        let py = com_venv(&root);
        let interp = PythonLauncher::Interpreter(py.clone());
        assert_eq!(interp.program(), (py.clone(), vec![]));
        assert_eq!(interp.display(&root), ".venv/bin/python");
        assert_eq!(
            interp.shell_command(&["-m", "pacote"]),
            format!("'{}' '-m' 'pacote'", py.display())
        );
        let fora = PythonLauncher::Interpreter(PathBuf::from("/usr/bin/python3"));
        assert_eq!(fora.display(&root), "/usr/bin/python3");

        let uv = PythonLauncher::Uv(PathBuf::from("/opt/uv"));
        assert_eq!(
            uv.program(),
            (
                PathBuf::from("/opt/uv"),
                vec!["run".into(), "python".into()]
            )
        );
        assert_eq!(uv.display(&root), "uv run python");
        assert_eq!(
            interp.script_display(&root, &root.join("tools/it's.py")),
            ".venv/bin/python 'tools/it'\\''s.py'"
        );
        assert_eq!(
            uv.script_display(&root, &root.join("a.py")),
            "uv run python 'a.py'"
        );
        assert_eq!(
            uv.shell_command(&["main.py"]),
            "'/opt/uv' run python 'main.py'"
        );
        // Aspas simples no argumento sao escapadas do jeito do shell.
        assert_eq!(
            interp.shell_command(&["it's.py"]),
            format!("'{}' 'it'\\''s.py'", py.display())
        );
    }

    /// `MicroPython`: `mpremote [connect <porta>] run <arquivo>`; sem mpremote
    /// detectado nao ha' lancador (o handler manda instalar); o `-m` de um
    /// pacote nao existe na placa.
    #[test]
    fn mpremote_runs_the_file_on_the_board() {
        let root = raiz("mpremote");
        let sem_porta = PythonLauncher::mpremote(PathBuf::from("/x/mpremote"), None);
        assert_eq!(
            sem_porta.program(),
            (PathBuf::from("/x/mpremote"), vec!["run".into()])
        );
        assert_eq!(sem_porta.display(&root), "mpremote run");
        assert_eq!(
            sem_porta.shell_command(&["main.py"]),
            "'/x/mpremote' run 'main.py'"
        );
        let com_porta =
            PythonLauncher::mpremote(PathBuf::from("/x/mpremote"), Some("/dev/ttyUSB0"));
        assert_eq!(
            com_porta.program(),
            (
                PathBuf::from("/x/mpremote"),
                vec!["connect".into(), "/dev/ttyUSB0".into(), "run".into()]
            )
        );
        assert_eq!(
            com_porta.display(&root),
            "mpremote connect /dev/ttyUSB0 run"
        );
        assert_eq!(
            com_porta.script_display(&root, &root.join("main.py")),
            "mpremote connect /dev/ttyUSB0 run 'main.py'"
        );

        std::fs::write(root.join("main.py"), "import machine\n").unwrap();
        assert_eq!(
            default_command(&root, Some(&sem_porta)).unwrap(),
            "'/x/mpremote' run 'main.py'"
        );
        std::fs::remove_file(root.join("main.py")).unwrap();
        std::fs::create_dir_all(root.join("app")).unwrap();
        std::fs::write(root.join("app/__main__.py"), "").unwrap();
        let erro = default_command(&root, Some(&sem_porta))
            .unwrap_err()
            .to_string();
        assert!(
            erro.contains("MicroPython") && erro.contains("`app`"),
            "{erro}"
        );
    }

    /// A precedencia do ponto de entrada, e a recusa de escolher entre dois
    /// pacotes.
    #[test]
    fn entry_point_follows_evidence_and_refuses_ambiguity() {
        let root = raiz("entrada");
        assert_eq!(entry_point(&root), None);

        std::fs::create_dir_all(root.join("src/pacote_a")).unwrap();
        std::fs::write(root.join("src/pacote_a/__main__.py"), "").unwrap();
        assert_eq!(
            entry_point(&root),
            Some(EntryPoint::Module("pacote_a".into()))
        );

        // Um pacote na RAIZ vence o de src/.
        std::fs::create_dir_all(root.join("ferramenta")).unwrap();
        std::fs::write(root.join("ferramenta/__main__.py"), "").unwrap();
        assert_eq!(
            entry_point(&root),
            Some(EntryPoint::Module("ferramenta".into()))
        );

        // Dois pacotes na raiz: ambiguidade, e a IDE nao escolhe.
        std::fs::create_dir_all(root.join("outra")).unwrap();
        std::fs::write(root.join("outra/__main__.py"), "").unwrap();
        assert_eq!(entry_point(&root), None);

        // Pasta oculta com __main__.py nao e' pacote.
        std::fs::create_dir_all(root.join(".cache")).unwrap();
        std::fs::write(root.join(".cache/__main__.py"), "").unwrap();
        std::fs::remove_dir_all(root.join("outra")).unwrap();
        assert_eq!(
            entry_point(&root),
            Some(EntryPoint::Module("ferramenta".into()))
        );

        // Arquivo na raiz vence pacote; main.py vence app.py.
        std::fs::write(root.join("app.py"), "").unwrap();
        assert_eq!(entry_point(&root), Some(EntryPoint::File("app.py".into())));
        std::fs::write(root.join("main.py"), "").unwrap();
        assert_eq!(entry_point(&root), Some(EntryPoint::File("main.py".into())));
    }

    /// `[project.scripts]` so' vale INSTALADO: o nome tem de existir em
    /// `.venv/bin`; e a tabela certa e' lida (nao `[project]`, nao
    /// `[tool.x.scripts]`).
    #[test]
    fn project_scripts_count_only_when_installed() {
        let root = raiz("scripts");
        std::fs::write(
            root.join("pyproject.toml"),
            "[project]\nname = \"demo\"\n\n[project.scripts]\n# comentario\ndemo-cli = \"demo.cli:main\"\n\"demo-web\" = \"demo.web:main\"\n\n[tool.outra.scripts]\nfalso = \"x:y\"\n",
        )
        .unwrap();
        assert_eq!(entry_point(&root), None, "nada instalado ainda");
        std::fs::create_dir_all(root.join(".venv/bin")).unwrap();
        std::fs::write(root.join(".venv/bin/falso"), "").unwrap();
        assert_eq!(entry_point(&root), None, "script de outra tabela nao conta");
        std::fs::write(root.join(".venv/bin/demo-web"), "").unwrap();
        assert_eq!(
            entry_point(&root),
            Some(EntryPoint::InstalledScript("demo-web".into()))
        );
        std::fs::write(root.join(".venv/bin/demo-cli"), "").unwrap();
        assert_eq!(
            entry_point(&root),
            Some(EntryPoint::InstalledScript("demo-cli".into())),
            "a ordem do arquivo decide"
        );
    }

    #[test]
    fn default_command_says_what_it_looked_for() {
        let root = raiz("padrao");
        let sem = default_command(&root, None).unwrap_err().to_string();
        assert!(
            sem.contains("interpretador") && sem.contains(".venv"),
            "{sem}"
        );

        let py = com_venv(&root);
        let launcher = PythonLauncher::Interpreter(py.clone());
        let nada = default_command(&root, Some(&launcher))
            .unwrap_err()
            .to_string();
        assert!(
            nada.contains("main.py") && nada.contains("[project.scripts]"),
            "{nada}"
        );

        std::fs::create_dir_all(root.join("src/demo")).unwrap();
        std::fs::write(root.join("src/demo/__main__.py"), "").unwrap();
        assert_eq!(
            default_command(&root, Some(&launcher)).unwrap(),
            format!("'{}' '-m' 'demo'", py.display())
        );
        std::fs::write(root.join("main.py"), "").unwrap();
        assert_eq!(
            default_command(&root, Some(&launcher)).unwrap(),
            format!("'{}' 'main.py'", py.display())
        );
        std::fs::remove_file(root.join("main.py")).unwrap();
        std::fs::remove_dir_all(root.join("src")).unwrap();
        std::fs::write(
            root.join("pyproject.toml"),
            "[project.scripts]\ndemo = \"demo:main\"\n",
        )
        .unwrap();
        std::fs::write(root.join(".venv/bin/demo"), "").unwrap();
        assert_eq!(
            default_command(&root, Some(&launcher)).unwrap(),
            format!("'{}'", root.join(".venv/bin/demo").display())
        );
        // Com o uv: o script instalado roda direto (o uv ja' o instalou); o
        // arquivo roda por `uv run python`.
        let uv = PythonLauncher::Uv(PathBuf::from("/opt/uv"));
        std::fs::write(root.join("app.py"), "").unwrap();
        assert_eq!(
            default_command(&root, Some(&uv)).unwrap(),
            "'/opt/uv' run python 'app.py'"
        );
    }
}

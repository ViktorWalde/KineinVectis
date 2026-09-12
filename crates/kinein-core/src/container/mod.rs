//! Containers como dominio NATIVO: Docker e Podman pela mesma CLI.
//!
//! Decisao do autor em 2026-07-17 (`roadmaps/28` §0) e priorizada em
//! 2026-09-12. As invariantes registradas la' moram aqui:
//!
//! ```text
//! a UI nunca chama `docker`     -> so' o core sobe processo, e e' ESTE modulo
//! toda acao e' Job cancelavel   -> start/stop/restart/rm/compose viram jobs
//!                                  (handlers/container.rs) com evento no fim
//! permissao VISIVEL             -> `status()` diz motor, versao, rootless,
//!                                  socket, se responde, e o passo oficial
//! Podman e' alternativa         -> e' o motor DESTA maquina (medido em
//!                                  2026-09-12: `docker` e' o shim podman-docker)
//! ```
//!
//! # Deteccao do motor, e por que ela nao confia no nome do binario
//!
//! No Fedora, `docker` no `PATH` costuma ser o `podman-docker`: um shim que
//! imprime *"Emulate Docker CLI using podman"* em stderr e responde como
//! Podman. Tratar isso como Docker Engine erraria o formato de `ps` (o Podman
//! devolve array em `--format json`; o Docker, um objeto por linha) e o
//! diagnostico de permissao (nao ha' daemon nem grupo `docker` num Podman
//! rootless). Por isso a deteccao PERGUNTA ao binario e prefere o `podman`
//! real quando o `docker` e' o shim.

pub mod parse;

use std::path::{Path, PathBuf};
use std::process::Command;

use kinein_protocol::{
    ComposeAction, ContainerAction, ContainerEngine, ContainerImagesResult, ContainerInfo,
    ContainerListResult, ContainerOpenMode, ContainerStatus,
};

use crate::tools::ToolDetector;

/// O que o shim `podman-docker` imprime em stderr a cada chamada.
const SHIM_BANNER: &str = "Emulate Docker CLI using podman";

/// Quantas linhas de log o `logs -f` traz do passado ao abrir a aba.
const LOG_TAIL: &str = "200";

/// O motor que responde nesta maquina.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Engine {
    /// Docker ou Podman — o que RESPONDE, nao o nome do binario.
    pub kind: ContainerEngine,
    /// O executavel que o core chama.
    pub binary: PathBuf,
    /// `docker` no PATH era o shim do Podman.
    pub emulated: bool,
}

impl Engine {
    fn command(&self) -> Command {
        Command::new(&self.binary)
    }

    /// `ps`/`images` no formato que cada motor sabe emitir como JSON.
    const fn json_format_args(&self) -> [&'static str; 2] {
        match self.kind {
            ContainerEngine::Podman => ["--format", "json"],
            ContainerEngine::Docker => ["--format", "{{json .}}"],
        }
    }
}

/// Detecta o motor pelo `PATH` do processo.
#[must_use]
pub fn detect() -> Option<Engine> {
    detect_with(&ToolDetector::from_environment())
}

/// Detecta o motor com o detector dado (o teste passa um `PATH` falso).
#[must_use]
pub fn detect_with(detector: &ToolDetector) -> Option<Engine> {
    let docker = detector.find_in_path("docker");
    let podman = detector.find_in_path("podman");
    match (docker, podman) {
        (Some(docker), podman) => {
            if is_podman_shim(&docker) {
                Some(Engine {
                    kind: ContainerEngine::Podman,
                    binary: podman.unwrap_or(docker),
                    emulated: true,
                })
            } else {
                Some(Engine {
                    kind: ContainerEngine::Docker,
                    binary: docker,
                    emulated: false,
                })
            }
        }
        (None, Some(podman)) => Some(Engine {
            kind: ContainerEngine::Podman,
            binary: podman,
            emulated: false,
        }),
        (None, None) => None,
    }
}

/// `docker --version` do shim escreve o aviso em stderr e "podman version" em
/// stdout; o Docker de verdade escreve "Docker version".
fn is_podman_shim(docker: &Path) -> bool {
    Command::new(docker)
        .arg("--version")
        .output()
        .is_ok_and(|saida| {
            String::from_utf8_lossy(&saida.stderr).contains(SHIM_BANNER)
                || String::from_utf8_lossy(&saida.stdout)
                    .trim_start()
                    .starts_with("podman")
        })
}

/// Saida de um comando curto: stdout aparado, ou `None` se falhou.
fn pergunta(engine: &Engine, args: &[&str]) -> Option<String> {
    let saida = engine.command().args(args).output().ok()?;
    saida
        .status
        .success()
        .then(|| String::from_utf8_lossy(&saida.stdout).trim().to_owned())
}

/// O estado do motor: existe, que versao, rootless, socket, responde, compose.
#[must_use]
pub fn status(engine: Option<&Engine>) -> ContainerStatus {
    let Some(engine) = engine else {
        return ContainerStatus {
            engine: None,
            binary: None,
            version: None,
            emulated: false,
            rootless: None,
            socket: None,
            reachable: false,
            compose: None,
            hint: Some(
                "nenhum motor de container no PATH. No Fedora, `sudo dnf install podman` (rootless, \
                 sem daemon); o Docker Engine vem do repositorio oficial em docs.docker.com. A IDE \
                 nao roda isso."
                    .to_owned(),
            ),
            raw_output: String::new(),
        };
    };

    let version = pergunta(engine, &["version", "--format", "{{.Client.Version}}"]);
    let (rootless, socket) = match engine.kind {
        ContainerEngine::Podman => (
            pergunta(engine, &["info", "--format", "{{.Host.Security.Rootless}}"])
                .and_then(|v| v.parse::<bool>().ok()),
            pergunta(engine, &["info", "--format", "{{.Host.RemoteSocket.Path}}"])
                .filter(|s| !s.is_empty()),
        ),
        ContainerEngine::Docker => (
            pergunta(engine, &["info", "--format", "{{json .SecurityOptions}}"])
                .map(|v| v.contains("rootless")),
            pergunta(
                engine,
                &[
                    "context",
                    "inspect",
                    "--format",
                    "{{(index .Endpoints \"docker\").Host}}",
                ],
            )
            .filter(|s| !s.is_empty()),
        ),
    };

    // `ps` e' a pergunta que decide: daemon de pe' E este usuario autorizado.
    let sonda = engine
        .command()
        .args(["ps", "--format", "{{.ID}}"])
        .output();
    let (reachable, raw_output) = match sonda {
        Ok(saida) => (
            saida.status.success(),
            parse::strip_ansi(&String::from_utf8_lossy(&saida.stderr)),
        ),
        Err(erro) => (false, erro.to_string()),
    };

    let compose = compose_tool(engine);
    let hint = (!reachable).then(|| hint_for(engine, &raw_output));

    ContainerStatus {
        engine: Some(engine.kind),
        binary: Some(engine.binary.display().to_string()),
        version,
        emulated: engine.emulated,
        rootless,
        socket,
        reachable,
        compose,
        hint,
        raw_output,
    }
}

/// O primeiro `compose` que responde: o subcomando do motor (plugin do Docker,
/// ou o `podman compose` que delega), senao os binarios avulsos.
#[must_use]
pub fn compose_tool(engine: &Engine) -> Option<String> {
    if pergunta(engine, &["compose", "version"]).is_some() {
        return Some(format!("{} compose", engine.binary.display()));
    }
    ["docker-compose", "podman-compose"]
        .into_iter()
        .find(|nome| {
            Command::new(nome)
                .arg("--version")
                .output()
                .is_ok_and(|s| s.status.success())
        })
        .map(ToOwned::to_owned)
}

/// O passo oficial quando o motor nao responde. Nunca um `sudo` que a IDE rode.
fn hint_for(engine: &Engine, stderr: &str) -> String {
    let baixo = stderr.to_ascii_lowercase();
    match engine.kind {
        ContainerEngine::Docker if baixo.contains("permission denied") => {
            "o daemon esta' de pe' mas este usuario nao pode usar o socket. Passo oficial: \
             `sudo usermod -aG docker $USER` e sair/entrar da sessao — ou o modo rootless do \
             Docker. A IDE nao roda isso."
                .to_owned()
        }
        ContainerEngine::Docker => {
            "o daemon nao respondeu. Passo oficial: `sudo systemctl enable --now docker` (e \
             `systemctl status docker` para ver por que parou). A IDE nao roda isso."
                .to_owned()
        }
        ContainerEngine::Podman => format!(
            "o podman nao respondeu ao `ps`. Rootless nao precisa de daemon; veja a saida crua. \
             Para a API por socket: `systemctl --user enable --now podman.socket`. Saida: {}",
            stderr.trim()
        ),
    }
}

/// Os containers, parados inclusive quando `all`.
#[must_use]
pub fn list(engine: &Engine, all: bool) -> ContainerListResult {
    let mut command = engine.command();
    command.arg("ps");
    if all {
        command.arg("-a");
    }
    command.args(engine.json_format_args());
    let (stdout, stderr) = run_capturing(command);
    let containers: Vec<ContainerInfo> = parse::containers(&stdout);
    let hint = containers.is_empty().then(|| {
        if stdout.trim().is_empty() || stdout.trim() == "[]" {
            "nenhum container neste motor. `compose up` ou `run` cria um; sem `all`, os parados \
             nao aparecem."
                .to_owned()
        } else {
            "o motor respondeu e o formato nao foi reconhecido — a saida crua esta' abaixo."
                .to_owned()
        }
    });
    ContainerListResult {
        containers,
        engine: Some(engine.kind),
        raw_output: raw(&stdout, &stderr),
        hint,
    }
}

/// As imagens locais.
#[must_use]
pub fn images(engine: &Engine) -> ContainerImagesResult {
    let mut command = engine.command();
    command.arg("images").args(engine.json_format_args());
    let (stdout, stderr) = run_capturing(command);
    let images = parse::images(&stdout);
    let hint = images
        .is_empty()
        .then(|| "nenhuma imagem local.".to_owned());
    ContainerImagesResult {
        images,
        engine: Some(engine.kind),
        raw_output: raw(&stdout, &stderr),
        hint,
    }
}

fn run_capturing(mut command: Command) -> (String, String) {
    match command.output() {
        Ok(saida) => (
            String::from_utf8_lossy(&saida.stdout).into_owned(),
            parse::strip_ansi(&String::from_utf8_lossy(&saida.stderr)),
        ),
        Err(erro) => (String::new(), erro.to_string()),
    }
}

/// stderr do shim entra na saida crua, mas o aviso repetido nao vira ruido:
/// so' quando nada mais foi dito.
fn raw(stdout: &str, stderr: &str) -> String {
    let sem_banner: Vec<&str> = stderr
        .lines()
        .filter(|l| !l.contains(SHIM_BANNER) && !l.trim().is_empty())
        .collect();
    if sem_banner.is_empty() {
        stdout.to_owned()
    } else {
        format!("{}\n{}", stdout.trim_end(), sem_banner.join("\n"))
    }
}

/// O comando de uma acao de ciclo de vida. `rm` SEM `-f`: remover o que esta'
/// rodando e' decisao em dois gestos (parar, depois remover), de proposito.
#[must_use]
pub fn action_command(engine: &Engine, action: ContainerAction, id: &str) -> Command {
    let mut command = engine.command();
    command.arg(action.verb()).arg(id);
    command
}

/// Programa e argumentos para abrir NUMA ABA DE TERMINAL: os logs seguindo,
/// ou um shell dentro do container.
#[must_use]
pub fn open_program_args(
    engine: &Engine,
    mode: ContainerOpenMode,
    id: &str,
) -> (String, Vec<String>) {
    let programa = engine.binary.display().to_string();
    let args = match mode {
        ContainerOpenMode::Logs => vec![
            "logs".to_owned(),
            "-f".to_owned(),
            "--tail".to_owned(),
            LOG_TAIL.to_owned(),
            id.to_owned(),
        ],
        ContainerOpenMode::Shell => vec![
            "exec".to_owned(),
            "-it".to_owned(),
            id.to_owned(),
            "/bin/sh".to_owned(),
        ],
    };
    (programa, args)
}

/// O comando de compose.
///
/// `compose_tool` e' o que `status()` achou: `<bin> compose` (subcomando) ou
/// um binario avulso. `up` e' `-d`: a saida viva mora na aba de logs, e um job
/// que nunca termina nao e' job.
#[must_use]
pub fn compose_command(
    compose_tool: &str,
    action: ComposeAction,
    file: Option<&str>,
    root: &Path,
) -> Command {
    let mut partes = compose_tool.split_whitespace();
    let mut command = Command::new(partes.next().unwrap_or("docker"));
    command.args(partes);
    if let Some(file) = file {
        command.arg("-f").arg(file);
    }
    match action {
        ComposeAction::Up => {
            command.args(["up", "-d"]);
        }
        ComposeAction::Down => {
            command.arg("down");
        }
    }
    command.current_dir(root);
    command
}

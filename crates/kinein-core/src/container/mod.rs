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
pub mod probe;
pub mod volumes;

use std::path::{Path, PathBuf};
use std::process::Command;

pub use probe::{Asked, answering, select_with};

use kinein_protocol::{
    ComposeAction, ContainerAction, ContainerEngine, ContainerImagesResult, ContainerInfo,
    ContainerListResult, ContainerOpenMode, ContainerStatus,
};

use crate::tools::ToolDetector;

/// O que o shim `podman-docker` imprime em stderr a cada chamada.
const SHIM_BANNER: &str = "Emulate Docker CLI using podman";

/// Quantas linhas de log o `logs -f` traz do passado ao abrir a aba.
const LOG_TAIL: &str = "200";

/// Os nomes que a ferramenta de compose procura sozinha, na ordem do
/// podman-compose 1.6.0 (`COMPOSE_DEFAULT_LS`, lido em 2026-09-13; os
/// `*.override.*` ficam de fora porque sozinhos nao sobem nada). O `docker
/// compose` conhece so' os quatro primeiros nomes sem prefixo `podman-`/
/// `container-`, e os dois concordam no primeiro: `compose.yaml`.
const COMPOSE_FILES: &[&str] = &[
    "compose.yaml",
    "compose.yml",
    "podman-compose.yaml",
    "podman-compose.yml",
    "docker-compose.yml",
    "docker-compose.yaml",
    "container-compose.yml",
    "container-compose.yaml",
];

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

/// O primeiro motor do `PATH`, SEM perguntar se responde: so' para o que
/// nao fala com o motor (o comando que aparece antes de rodar). Quem vai
/// falar com ele usa [`select_with`], que pergunta com prazo.
#[must_use]
pub fn detect_with(detector: &ToolDetector) -> Option<Engine> {
    probe::candidates(detector).into_iter().next()
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

/// Saida de um comando curto, com prazo: stdout aparado, ou `None`.
fn pergunta(engine: &Engine, args: &[&str]) -> Option<String> {
    let reply = probe::ask(engine, args);
    reply.ok.then(|| reply.stdout.trim().to_owned())
}

/// O motor nao respondeu a `list`/`images`: a dica e a saida crua, para
/// virar erro em vez de lista vazia (defeito 3, `roadmaps/59` §2.5.2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineFailure {
    /// O motor que falhou.
    pub engine: ContainerEngine,
    /// O passo oficial.
    pub hint: String,
    /// O que o motor disse (ou o estouro do prazo).
    pub raw_output: String,
}

impl EngineFailure {
    /// A frase para a tela: "o Podman nao respondeu: <a dica>".
    #[must_use]
    pub fn message(&self) -> String {
        let name = match self.engine {
            ContainerEngine::Docker => "Docker",
            ContainerEngine::Podman => "Podman",
        };
        format!("o {name} nao respondeu: {}", self.hint)
    }

    /// A falha a partir da resposta do motor.
    #[must_use]
    pub fn from_asked(engine: &Engine, asked: &Asked) -> Self {
        Self {
            engine: engine.kind,
            hint: hint_for(engine, asked),
            raw_output: asked.reason(),
        }
    }
}

/// O endpoint do contexto ativo do `docker` (o socket que ele usa).
fn docker_endpoint(engine: &Engine) -> Option<String> {
    pergunta(
        engine,
        &[
            "context",
            "inspect",
            "--format",
            "{{(index .Endpoints \"docker\").Host}}",
        ],
    )
    .filter(|s| !s.is_empty())
}

/// O arquivo de compose que a ferramenta pegaria sozinha em `root`, relativo
/// a raiz — ou `None`, e ai' `compose up` sem `-f` so' pode falhar.
#[must_use]
pub fn compose_file_in(root: &Path) -> Option<String> {
    COMPOSE_FILES
        .iter()
        .find(|nome| root.join(nome).is_file())
        .map(|nome| (*nome).to_owned())
}

/// O estado do motor: existe, quem atende e em que versao, rootless, socket,
/// responde, compose.
///
/// Com um workspace aberto, tambem o arquivo de compose que o projeto tem.
/// `selected` e' o que [`select_with`] escolheu, com a sonda.
#[must_use]
pub fn status(
    selected: Option<&(Engine, Asked)>,
    workspace_root: Option<&Path>,
) -> ContainerStatus {
    let compose_file = workspace_root.and_then(compose_file_in);
    let Some((engine, check)) = selected else {
        return ContainerStatus {
            engine: None,
            binary: None,
            version: None,
            client_version: None,
            server_name: None,
            emulated: false,
            rootless: None,
            socket: None,
            reachable: false,
            compose: None,
            compose_file,
            hint: Some(
                "nenhum motor de container no PATH. No Fedora, `sudo dnf install podman` (rootless, \
                 sem daemon); o Docker Engine vem do repositorio oficial em docs.docker.com. A IDE \
                 nao roda isso."
                    .to_owned(),
            ),
            raw_output: String::new(),
        };
    };

    let reachable = check.ok;
    let (server_name, version) = if reachable {
        probe::server_identity(engine)
    } else {
        (None, None)
    };
    let client_version = probe::client_version(engine);
    // `info` so' com o motor respondendo: mudo, cada uma gastaria o prazo.
    let info = |template: &str| {
        if reachable {
            pergunta(engine, &["info", "--format", template])
        } else {
            None
        }
    };
    let (rootless, socket) = match engine.kind {
        ContainerEngine::Podman => (
            info("{{.Host.Security.Rootless}}").and_then(|v| v.parse::<bool>().ok()),
            info("{{.Host.RemoteSocket.Path}}").filter(|s| !s.is_empty()),
        ),
        ContainerEngine::Docker => (
            info("{{json .SecurityOptions}}").map(|v| v.contains("rootless")),
            docker_endpoint(engine),
        ),
    };

    // `ps` (a sonda do `select_with`) e' a pergunta que decide: daemon de pe'
    // E este usuario autorizado.
    let raw_output = if reachable {
        check.stderr.clone()
    } else {
        check.reason()
    };
    let compose = compose_tool(engine);
    let hint = (!reachable).then(|| hint_for(engine, check));

    ContainerStatus {
        engine: Some(engine.kind),
        binary: Some(engine.binary.display().to_string()),
        version,
        client_version,
        server_name,
        emulated: engine.emulated,
        rootless,
        socket,
        reachable,
        compose,
        compose_file,
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
///
/// O `docker` cujo contexto aponta o socket do Podman (`podman.sock`) nao tem
/// daemon do Docker para ligar: a dica e' a do socket do Podman (defeito 2).
fn hint_for(engine: &Engine, asked: &Asked) -> String {
    let baixo = asked.stderr.to_ascii_lowercase();
    let deadline_note = if asked.timed_out {
        format!(
            "o `ps` nao respondeu em {} s. ",
            probe::ASK_DEADLINE.as_secs()
        )
    } else {
        String::new()
    };
    match engine.kind {
        ContainerEngine::Docker if !asked.timed_out && baixo.contains("permission denied") => {
            "o daemon esta' de pe' mas este usuario nao pode usar o socket. Passo oficial: \
             `sudo usermod -aG docker $USER` e sair/entrar da sessao — ou o modo rootless do \
             Docker, ou o Podman (rootless, sem grupo). A IDE nao roda isso."
                .to_owned()
        }
        ContainerEngine::Docker => match docker_endpoint(engine) {
            Some(endpoint) if endpoint.contains("podman") => format!(
                "{deadline_note}o contexto do Docker aponta o socket do Podman ({endpoint}), e ele nao \
                 respondeu. Passo oficial: `systemctl --user enable --now podman.socket`. A IDE \
                 nao roda isso."
            ),
            _ => format!(
                "{deadline_note}o daemon nao respondeu. Passo oficial: `sudo systemctl enable --now \
                 docker` (e `systemctl status docker` para ver por que parou). A IDE nao roda \
                 isso."
            ),
        },
        ContainerEngine::Podman => format!(
            "{deadline_note}o podman nao respondeu ao `ps`. Rootless nao precisa de daemon; veja a saida \
             crua. Para a API por socket: `systemctl --user enable --now podman.socket`. Saida: {}",
            asked.reason()
        ),
    }
}

/// Os containers, parados inclusive quando `all`. O motor que nao responde e'
/// erro, nunca "nenhum container" (defeito 3).
pub fn list(engine: &Engine, all: bool) -> Result<ContainerListResult, EngineFailure> {
    let mut args = vec!["ps"];
    if all {
        args.push("-a");
    }
    args.extend(engine.json_format_args());
    let reply = probe::ask(engine, &args);
    if !reply.ok {
        return Err(EngineFailure::from_asked(engine, &reply));
    }
    let (stdout, stderr) = (reply.stdout, reply.stderr);
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
    Ok(ContainerListResult {
        containers,
        engine: Some(engine.kind),
        raw_output: raw(&stdout, &stderr),
        hint,
    })
}

/// As imagens locais; o motor que nao responde e' erro, como no [`list`].
pub fn images(engine: &Engine) -> Result<ContainerImagesResult, EngineFailure> {
    let mut args = vec!["images"];
    args.extend(engine.json_format_args());
    let reply = probe::ask(engine, &args);
    if !reply.ok {
        return Err(EngineFailure::from_asked(engine, &reply));
    }
    let (stdout, stderr) = (reply.stdout, reply.stderr);
    let images = parse::images(&stdout);
    let hint = images
        .is_empty()
        .then(|| "nenhuma imagem local.".to_owned());
    Ok(ContainerImagesResult {
        images,
        engine: Some(engine.kind),
        raw_output: raw(&stdout, &stderr),
        hint,
    })
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
pub fn action_command(
    engine: &Engine,
    action: ContainerAction,
    id: &str,
    volumes: bool,
) -> Command {
    let mut command = engine.command();
    command.arg(action.verb());
    // `rm -v`: os volumes ANONIMOS do container vao junto (16b.3); quem
    // chama so' liga com `remove`.
    if volumes {
        command.arg("-v");
    }
    command.arg(id);
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

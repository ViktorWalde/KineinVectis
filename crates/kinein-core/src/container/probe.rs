//! Perguntar ao motor com prazo, e escolher o motor que responde (16b.1,
//! `roadmaps/59` §2.5.2, defeitos 2, 4 e 5).
//!
//! # Por que prazo
//!
//! Medido em 2026-10-08 no Arch: com o contexto do Docker apontando um socket
//! que ninguem servia, `container.list` levou 32 s e travou o core inteiro,
//! porque rodava no laco e o `docker ps` nao tinha teto. A pergunta curta
//! (`ps`, `version`, `info`, `context`) passa a ter [`ASK_DEADLINE`]; no
//! estouro, a arvore do processo morre e a resposta diz "nao respondeu".
//!
//! # Por que escolher pelo `ps`
//!
//! Antes, o `docker` ganhava sempre que existia e nao era o shim do Podman —
//! mesmo negando o socket, com um Podman funcionando ao lado. Agora vale o
//! primeiro, na ordem `docker`, `podman`, cujo `ps` responde; se nenhum
//! responde, fica o primeiro, com a dica dele.

use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::time::Duration;

use kinein_protocol::ContainerEngine;

use super::{Engine, EngineFailure, SHIM_BANNER, is_podman_shim, parse};
use crate::tools::ToolDetector;

/// Quanto uma pergunta curta ao motor pode levar.
pub const ASK_DEADLINE: Duration = Duration::from_secs(5);

/// O que o motor respondeu a uma pergunta curta.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Asked {
    /// Saiu com zero dentro do prazo.
    pub ok: bool,
    /// A saida padrao, linha a linha, juntada com `\n`.
    pub stdout: String,
    /// A saida de erro, sem ANSI e sem o aviso do shim.
    pub stderr: String,
    /// Estourou o prazo e foi morto.
    pub timed_out: bool,
}

impl Asked {
    /// O motivo, para a dica e a saida crua: o stderr, ou o estouro.
    #[must_use]
    pub fn reason(&self) -> String {
        if self.timed_out {
            format!(
                "nao respondeu em {} s (o processo foi encerrado)",
                ASK_DEADLINE.as_secs()
            )
        } else {
            self.stderr.trim().to_owned()
        }
    }
}

/// Pergunta ao motor com [`ASK_DEADLINE`].
#[must_use]
pub fn ask(engine: &Engine, args: &[&str]) -> Asked {
    let mut command = engine.command();
    command.args(args);
    ask_command(command, ASK_DEADLINE)
}

/// Roda `command` ate' `deadline`: um vigia liga o cancelamento no estouro, e
/// o `stream_command_lines_cancelable` mata a arvore do processo.
pub(crate) fn ask_command(command: Command, deadline: Duration) -> Asked {
    let cancel = Arc::new(AtomicBool::new(false));
    let timed_out = Arc::new(AtomicBool::new(false));
    let (done, finished) = mpsc::channel::<()>();
    let watchdog = {
        let cancel = Arc::clone(&cancel);
        let timed_out = Arc::clone(&timed_out);
        std::thread::spawn(move || {
            if matches!(
                finished.recv_timeout(deadline),
                Err(mpsc::RecvTimeoutError::Timeout)
            ) {
                timed_out.store(true, Ordering::SeqCst);
                cancel.store(true, Ordering::SeqCst);
            }
        })
    };
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let mut on_line = |stream: &'static str, line: String| {
        if stream == "stdout" {
            stdout.push(line);
        } else if !line.contains(SHIM_BANNER) {
            stderr.push(parse::strip_ansi(&line));
        }
    };
    let status = crate::process::stream_command_lines_cancelable(command, &cancel, &mut on_line);
    drop(done);
    drop(watchdog.join());
    let expired = timed_out.load(Ordering::SeqCst);
    match status {
        Ok(status) => Asked {
            ok: status.success() && !expired,
            stdout: stdout.join("\n"),
            stderr: stderr.join("\n"),
            timed_out: expired,
        },
        Err(error) => Asked {
            ok: false,
            stdout: String::new(),
            stderr: match error {
                crate::process::ProcessError::Spawn(e) => format!("nao foi possivel iniciar: {e}"),
                crate::process::ProcessError::Wait(e) => format!("nao foi possivel esperar: {e}"),
            },
            timed_out: false,
        },
    }
}

/// Os motores do `PATH`, na ordem em que se tentam: `docker`, depois
/// `podman`. O `docker` que e' o shim do Podman vira um Podman so'.
#[must_use]
pub fn candidates(detector: &ToolDetector) -> Vec<Engine> {
    let docker = detector.find_in_path("docker");
    let podman = detector.find_in_path("podman");
    match (docker, podman) {
        (Some(docker), podman) if is_podman_shim(&docker) => vec![Engine {
            kind: ContainerEngine::Podman,
            binary: podman.unwrap_or(docker),
            emulated: true,
        }],
        (docker, podman) => docker
            .map(|binary| Engine {
                kind: ContainerEngine::Docker,
                binary,
                emulated: false,
            })
            .into_iter()
            .chain(podman.map(|binary| Engine {
                kind: ContainerEngine::Podman,
                binary,
                emulated: false,
            }))
            .collect(),
    }
}

/// O motor que responde ao `ps`, com a resposta dele; nenhum responde, o
/// primeiro do `PATH` com a falha (a dica sai dela). `None` sem motor.
#[must_use]
pub fn select_with(detector: &ToolDetector) -> Option<(Engine, Asked)> {
    let mut first: Option<(Engine, Asked)> = None;
    for engine in candidates(detector) {
        let check = ask(&engine, &["ps", "--format", "{{.ID}}"]);
        if check.ok {
            return Some((engine, check));
        }
        if first.is_none() {
            first = Some((engine, check));
        }
    }
    first
}

/// O motor pronto para o trabalho: `None` sem motor no `PATH`; `Err` quando
/// nenhum responde, com a falha (e a dica) do primeiro.
#[must_use]
pub fn answering(detector: &ToolDetector) -> Option<Result<Engine, EngineFailure>> {
    select_with(detector).map(|(engine, check)| {
        if check.ok {
            Ok(engine)
        } else {
            Err(EngineFailure::from_asked(&engine, &check))
        }
    })
}

/// Quem atende do outro lado do CLI: o nome e a versao do SERVIDOR.
///
/// Um `docker` cujo contexto aponta o socket do Podman diz "Podman Engine"
/// aqui, e nao a versao do CLI (defeito 5). O Podman local nao tem
/// servidor separado: o nome e' o dele e a versao, a do proprio Podman.
#[must_use]
pub fn server_identity(engine: &Engine) -> (Option<String>, Option<String>) {
    match engine.kind {
        ContainerEngine::Podman => {
            let version = ask(engine, &["version", "--format", "{{.Client.Version}}"]);
            (
                Some("Podman".to_owned()),
                version
                    .ok
                    .then(|| version.stdout.trim().to_owned())
                    .filter(|v| !v.is_empty()),
            )
        }
        ContainerEngine::Docker => {
            let server = ask(engine, &["version", "--format", "{{json .Server}}"]);
            if !server.ok {
                return (None, None);
            }
            parse::server_identity(&server.stdout)
        }
    }
}

/// A versao do CLI, pelo `--version`, que nao fala com o servidor: o
/// `docker version` pergunta ao servidor tambem, e com o motor mudo gastaria
/// outro prazo inteiro (16b.1, `roadmaps/59` §2.5.2).
#[must_use]
pub fn client_version(engine: &Engine) -> Option<String> {
    let reply = ask(engine, &["--version"]);
    if reply.timed_out {
        return None;
    }
    reply.stdout.lines().next().and_then(parse::cli_version)
}

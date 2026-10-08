//! Processo filho de LONGA VIDA com dono unico (D1b.1, `DocsPublic/arquitetura/39` §6.1).
//!
//! O `process.rs` atende quem TERMINA (build, test): drena e espera o fim. Um
//! adaptador de banco vive enquanto a conexao vive e conversa nos dois sentidos,
//! e o encerramento dele tem de ser PROVADO: processo colhido, descendentes
//! mortos, leitores terminados. Esta base e' dona do processo, do stdin, da
//! cauda do stderr e da thread que le o stdout. O enquadramento do stdout e' do
//! dono do protocolo (linhas JSON no adaptador, `Content-Length` no LSP), por
//! isso ele chega como uma funcao que roda numa thread daqui.
//!
//! Tres regras que nascem de 39 §6:
//!
//! - **ambiente explicito:** `env_clear` e so' a lista permitida; uma variavel
//!   de banco de outro perfil nao chega ao processo;
//! - **grupo proprio:** o encerramento alcanca os descendentes (um daemon
//!   auxiliar); o PID do filho nao prova desligamento. Um auxiliar que cria a
//!   propria sessao escapa do grupo, e o fechamento dele e' do adaptador;
//! - **encerrar com prazo e dizer o que houve:** stdin fechado, espera,
//!   `SIGTERM` no grupo, espera curta, `SIGKILL` no grupo, coleta. So'
//!   "colhido e leitor junto" conta como encerrado.

use std::{
    io,
    os::unix::process::CommandExt,
    process::{Child, ChildStdin, ChildStdout, Command, ExitStatus, Stdio},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use rustix::process::{Pid, Signal, kill_process_group};

use crate::stderr_tail::{DEFAULT_CAPACITY, StderrTail};

/// Variaveis que um processo de longa vida herda; o resto vem explicito.
pub const ALLOWED_ENV: [&str; 7] = [
    "PATH",
    "HOME",
    "LANG",
    "LC_ALL",
    "TZ",
    "TMPDIR",
    "XDG_RUNTIME_DIR",
];

/// Espera depois do `SIGTERM`, antes do `SIGKILL`.
const TERMINATE_GRACE: Duration = Duration::from_millis(500);

/// Prazo para a thread leitora terminar depois da coleta.
const READER_GRACE: Duration = Duration::from_secs(2);

/// Como o processo terminou.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Ending {
    /// Saiu sozinho dentro do prazo, depois do EOF no stdin.
    Graceful,
    /// Saiu com o `SIGTERM` no grupo.
    Terminated,
    /// Precisou do `SIGKILL` no grupo.
    Killed,
}

/// O que o encerramento conseguiu provar.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Closed {
    /// Como o processo terminou.
    pub ending: Ending,
    /// O status colhido do processo.
    pub status: Option<ExitStatus>,
    /// A thread do stdout terminou dentro do prazo.
    pub reader_joined: bool,
}

impl Closed {
    /// Processo colhido e leitor junto: so' entao a instancia esta' livre.
    #[must_use]
    pub const fn collected(&self) -> bool {
        self.status.is_some() && self.reader_joined
    }
}

/// Um processo filho de longa vida, com dono unico.
#[derive(Debug)]
pub struct OwnedChild {
    child: Child,
    group: Option<Pid>,
    stdin: Option<ChildStdin>,
    stderr: StderrTail,
    reader: Option<JoinHandle<()>>,
}

impl OwnedChild {
    /// Sobe `command` com o ambiente permitido mais `env`, grupo proprio e os
    /// tres pipes; o stdout vai para `read_stdout`, numa thread desta base.
    ///
    /// # Errors
    /// Falha do `spawn` (executavel ausente, permissao) ou pipe indisponivel.
    pub fn spawn(
        mut command: Command,
        env: &[(&str, &str)],
        read_stdout: impl FnOnce(ChildStdout) + Send + 'static,
    ) -> io::Result<Self> {
        command.env_clear();
        for name in ALLOWED_ENV {
            if let Some(value) = std::env::var_os(name) {
                command.env(name, value);
            }
        }
        for (name, value) in env {
            command.env(name, value);
        }
        command
            .process_group(0)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child = command.spawn()?;
        let group = i32::try_from(child.id()).ok().and_then(Pid::from_raw);
        let (Some(stdin), Some(stdout), Some(stderr)) =
            (child.stdin.take(), child.stdout.take(), child.stderr.take())
        else {
            drop(child.kill());
            drop(child.wait());
            return Err(io::Error::other("pipe do processo indisponivel"));
        };
        let stderr = StderrTail::spawn(stderr, DEFAULT_CAPACITY, None);
        let reader = thread::spawn(move || read_stdout(stdout));
        Ok(Self {
            child,
            group,
            stdin: Some(stdin),
            stderr,
            reader: Some(reader),
        })
    }

    /// O stdin do processo, enquanto ele nao foi fechado.
    pub const fn stdin(&mut self) -> Option<&mut ChildStdin> {
        self.stdin.as_mut()
    }

    /// A cauda limitada do stderr; o dono decide se ela pode sair.
    #[must_use]
    pub fn stderr_tail(&self) -> String {
        self.stderr.tail()
    }

    /// O processo ja' saiu? Nao bloqueia.
    pub fn exited(&mut self) -> Option<ExitStatus> {
        self.child.try_wait().ok().flatten()
    }

    /// Encerra com prazo e diz o que conseguiu provar.
    #[must_use]
    pub fn close(mut self, grace: Duration) -> Closed {
        drop(self.stdin.take());
        let mut ending = Ending::Graceful;
        let mut status = wait_until(&mut self.child, grace);
        if status.is_none() {
            ending = Ending::Terminated;
            self.signal_group(Signal::TERM);
            status = wait_until(&mut self.child, TERMINATE_GRACE);
        }
        if status.is_none() {
            ending = Ending::Killed;
            self.signal_group(Signal::KILL);
            status = self.child.wait().ok();
        }
        // O lider saiu; um descendente no mesmo grupo ainda pode viver (e
        // segurar o stdout aberto). No Linux o PGID nao e' reaproveitado
        // enquanto o grupo existe, entao o sinal so' alcanca o que restou.
        self.signal_group(Signal::KILL);
        let reader_joined = self.reader.take().is_none_or(join_until);
        Closed {
            ending,
            status,
            reader_joined,
        }
    }

    fn signal_group(&self, signal: Signal) {
        if let Some(group) = self.group {
            // ESRCH: o grupo ja' acabou, que e' o resultado esperado.
            kill_process_group(group, signal).ok();
        }
    }
}

fn wait_until(child: &mut Child, grace: Duration) -> Option<ExitStatus> {
    let deadline = Instant::now() + grace;
    loop {
        if let Ok(Some(status)) = child.try_wait() {
            return Some(status);
        }
        if Instant::now() >= deadline {
            return None;
        }
        thread::sleep(Duration::from_millis(10));
    }
}

fn join_until(reader: JoinHandle<()>) -> bool {
    let deadline = Instant::now() + READER_GRACE;
    while !reader.is_finished() {
        if Instant::now() >= deadline {
            return false;
        }
        thread::sleep(Duration::from_millis(10));
    }
    reader.join().is_ok()
}

#[cfg(test)]
mod tests;

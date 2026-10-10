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
//!
//! O grupo e o ambiente que muda por sistema moram em `crate::platform`
//! (60 §3.2): no Windows nao ha' `SIGTERM`, e o encerramento vai do EOF direto
//! ao `kill` da arvore.

use std::{
    io,
    process::{Child, ChildStdin, ChildStdout, Command, ExitStatus, Stdio},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use crate::platform::{self, Group};
use crate::stderr_tail::{DEFAULT_CAPACITY, LineSink, StderrTail};

/// Variaveis que um processo de longa vida herda; o resto vem explicito. A
/// lista muda por sistema (`platform::INHERITED_ENV`).
pub const ALLOWED_ENV: &[&str] = platform::INHERITED_ENV;

/// Espera depois do `SIGTERM`, antes do `SIGKILL`.
const TERMINATE_GRACE: Duration = Duration::from_millis(500);

/// Prazo para a thread leitora terminar depois da coleta.
const READER_GRACE: Duration = Duration::from_secs(2);

/// O ambiente do processo.
#[derive(Debug, Clone, Copy)]
pub enum Env<'a> {
    /// So' [`ALLOWED_ENV`] e as variaveis dadas: processo com credencial
    /// (adaptador de banco).
    Allowlist(&'a [(&'a str, &'a str)]),
    /// O ambiente do core inteiro: ferramenta de desenvolvimento sem
    /// credencial (language server), que precisa de `CARGO_HOME` e afins.
    Inherit,
}

/// Mata o grupo do processo de qualquer thread, sem colher nem esperar: para
/// a thread leitora que descobre uma falha e nao pode encerrar a si mesma.
///
/// `Clone` e nao `Copy`: no Windows o grupo e' um Job Object compartilhado.
#[derive(Debug, Clone)]
pub struct GroupKiller(Option<Group>);

impl GroupKiller {
    /// `SIGKILL` no grupo; a coleta fica com o dono do [`OwnedChild`].
    pub fn kill(self) {
        if let Some(group) = &self.0 {
            platform::kill_group(group);
        }
    }
}

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

/// Um processo filho de longa vida, com dono unico. Sem [`OwnedChild::close`],
/// o `Drop` mata o grupo e colhe o processo: nada fica orfao nem zumbi.
#[derive(Debug)]
pub struct OwnedChild {
    closed: bool,
    child: Child,
    group: Option<Group>,
    stdin: Option<ChildStdin>,
    stderr: StderrTail,
    reader: Option<JoinHandle<()>>,
}

impl OwnedChild {
    /// Sobe `command` com o ambiente pedido, grupo proprio e os tres pipes; o
    /// stdout vai para `read_stdout`, numa thread desta base, e cada linha do
    /// stderr para `stderr_lines`, quando o dono quer.
    ///
    /// # Errors
    /// Falha do `spawn` (executavel ausente, permissao) ou pipe indisponivel.
    pub fn spawn(
        mut command: Command,
        env: Env<'_>,
        stderr_lines: Option<LineSink>,
        read_stdout: impl FnOnce(ChildStdout) + Send + 'static,
    ) -> io::Result<Self> {
        if let Env::Allowlist(extra) = env {
            command.env_clear();
            for name in ALLOWED_ENV {
                if let Some(value) = std::env::var_os(name) {
                    command.env(name, value);
                }
            }
            for (name, value) in extra {
                command.env(name, value);
            }
        }
        platform::own_group(&mut command);
        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child = command.spawn()?;
        let group = platform::group_of(&child);
        let (Some(stdin), Some(stdout), Some(stderr)) =
            (child.stdin.take(), child.stdout.take(), child.stderr.take())
        else {
            drop(child.kill());
            drop(child.wait());
            return Err(io::Error::other("pipe do processo indisponivel"));
        };
        let stderr = StderrTail::spawn(stderr, DEFAULT_CAPACITY, stderr_lines);
        let reader = thread::spawn(move || read_stdout(stdout));
        Ok(Self {
            closed: false,
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

    /// Entrega o stdin ao dono do protocolo, que escreve de varias threads.
    /// Fechar a ponta dele e' o EOF que [`OwnedChild::close`] daria.
    pub const fn take_stdin(&mut self) -> Option<ChildStdin> {
        self.stdin.take()
    }

    /// A cauda limitada do stderr; o dono decide se ela pode sair.
    #[must_use]
    pub fn stderr_tail(&self) -> String {
        self.stderr.tail()
    }

    /// A mesma cauda, para uma thread do dono anexar a um erro.
    #[must_use]
    pub fn stderr_handle(&self) -> StderrTail {
        self.stderr.clone()
    }

    /// Mata o grupo de outra thread; ver [`GroupKiller`].
    #[must_use]
    pub fn killer(&self) -> GroupKiller {
        GroupKiller(self.group.clone())
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
            if self.terminate_group() {
                status = wait_until(&mut self.child, TERMINATE_GRACE);
            }
        }
        if status.is_none() {
            ending = Ending::Killed;
            self.kill_group();
            status = self.child.wait().ok();
        }
        // O lider saiu; um descendente no mesmo grupo ainda pode viver (e
        // segurar o stdout aberto). No Linux o PGID nao e' reaproveitado
        // enquanto o grupo existe, entao o sinal so' alcanca o que restou.
        self.kill_group();
        let reader_joined = self.reader.take().is_none_or(join_until);
        self.closed = true;
        Closed {
            ending,
            status,
            reader_joined,
        }
    }

    /// O pedido gentil ao grupo; `false` quando o sistema nao tem um.
    fn terminate_group(&self) -> bool {
        self.group.as_ref().is_some_and(platform::terminate_group)
    }

    fn kill_group(&self) {
        if let Some(group) = &self.group {
            platform::kill_group(group);
        }
    }
}

impl Drop for OwnedChild {
    fn drop(&mut self) {
        if !self.closed {
            self.kill_group();
            drop(self.child.wait());
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

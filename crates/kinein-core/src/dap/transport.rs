//! Ownership of the DAP connection: spawned stdio adapter or existing TCP adapter.
use std::{
    io::{self, Read, Write},
    net::{Shutdown, TcpStream, ToSocketAddrs},
    path::Path,
    process::{Child, ChildStdin, Command, Stdio},
    time::{Duration, Instant},
};

use kinein_protocol::DebugConnectParams;
use serde_json::json;

use super::{DebugError, adapter::Adapter, reader::send_event, target::DebugTarget};
use crate::{lsp::EventSender, stderr_tail::StderrTail};

const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

/// Dropping a TCP connection never kills the externally owned debuggee.
#[derive(Debug)]
pub(super) enum Transport {
    /// O adaptador que a IDE criou, com o stderr dele ouvido (2026-09-17):
    /// cada linha vira `event.debug.output { category: "adapter" }` e a
    /// cauda entra na mensagem quando o handshake falha. Antes era
    /// `Stdio::null()`, e "o adapter nao respondeu" vinha sem causa.
    Process {
        child: Child,
        stderr: StderrTail,
    },
    Tcp(TcpStream),
}

/// The shared wire writes framed requests through either transport.
#[derive(Debug)]
pub(super) enum Writer {
    Stdio(ChildStdin),
    Tcp(TcpStream),
}

impl Write for Writer {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        match self {
            Self::Stdio(stdin) => stdin.write(bytes),
            Self::Tcp(socket) => socket.write(bytes),
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        match self {
            Self::Stdio(stdin) => stdin.flush(),
            Self::Tcp(socket) => socket.flush(),
        }
    }
}

impl Drop for Transport {
    fn drop(&mut self) {
        match self {
            Self::Process { child, .. } => {
                drop(child.kill());
                drop(child.wait());
            }
            Self::Tcp(socket) => drop(socket.shutdown(Shutdown::Both)),
        }
    }
}

type Opened = (Transport, Writer, Box<dyn Read + Send>);

impl Transport {
    pub(super) fn open(
        root: &Path,
        adapter: &Adapter,
        target: &DebugTarget,
        events: &EventSender,
    ) -> Result<Opened, DebugError> {
        if let DebugTarget::PythonAttach(endpoint) = target {
            let socket = connect(endpoint).map_err(|error| DebugError::Adapter {
                message: format!(
                    "falha ao conectar debugpy em {}:{}: {error}",
                    endpoint.host, endpoint.port
                ),
            })?;
            let writer = socket.try_clone().map_err(|error| io_error(&error))?;
            let reader = socket.try_clone().map_err(|error| io_error(&error))?;
            return Ok((Self::Tcp(socket), Writer::Tcp(writer), Box::new(reader)));
        }
        let mut child = Command::new(&adapter.program)
            .args(adapter.arguments)
            .current_dir(root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| {
                if error.kind() == io::ErrorKind::NotFound {
                    DebugError::MissingAdapter {
                        program: adapter.program.display().to_string(),
                    }
                } else {
                    DebugError::Adapter {
                        message: format!("falha ao iniciar {}: {error}", adapter.program.display()),
                    }
                }
            })?;
        let pipes = (child.stdin.take(), child.stdout.take(), child.stderr.take());
        let (Some(stdin), Some(stdout), Some(stderr)) = pipes else {
            // O `Child` cai aqui e mata o processo.
            return Err(DebugError::Adapter {
                message: "adapter subiu sem stdin/stdout/stderr utilizaveis".to_owned(),
            });
        };
        let stderr = StderrTail::spawn(
            stderr,
            crate::stderr_tail::CAPACIDADE_PADRAO,
            Some(coletor(events)),
        );
        let transport = Self::Process { child, stderr };
        Ok((transport, Writer::Stdio(stdin), Box::new(stdout)))
    }

    /// A cauda do stderr do adaptador; vazia num attach TCP (o processo e'
    /// de outro dono e o stderr dele nao passa por aqui).
    pub(super) const fn stderr_tail(&self) -> Option<&StderrTail> {
        match self {
            Self::Process { stderr, .. } => Some(stderr),
            Self::Tcp(_) => None,
        }
    }
}

/// Cada linha do stderr do adaptador sai como `event.debug.output` com a
/// categoria `adapter` — o mesmo evento que o DAP `output` usa, para a aba
/// Debug mostrar sem canal novo.
fn coletor(events: &EventSender) -> crate::stderr_tail::Coletor {
    let events = events.clone();
    Box::new(move |linha: &str| {
        send_event(
            &events,
            "event.debug.output",
            json!({ "category": "adapter", "line": linha }),
        );
    })
}

fn io_error(error: &io::Error) -> DebugError {
    DebugError::Adapter {
        message: format!("falha no transporte DAP: {error}"),
    }
}

fn connect(endpoint: &DebugConnectParams) -> io::Result<TcpStream> {
    let addresses = (endpoint.host.as_str(), endpoint.port).to_socket_addrs()?;
    let deadline = Instant::now() + CONNECT_TIMEOUT;
    let mut last_error = io::Error::new(io::ErrorKind::AddrNotAvailable, "host sem enderecos TCP");
    for address in addresses {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            break;
        }
        match TcpStream::connect_timeout(&address, remaining) {
            Ok(socket) => {
                socket.set_write_timeout(Some(CONNECT_TIMEOUT))?;
                socket.set_nodelay(true)?;
                return Ok(socket);
            }
            Err(error) => last_error = error,
        }
    }
    Err(last_error)
}

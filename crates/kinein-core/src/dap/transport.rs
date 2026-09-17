//! Ownership of the DAP connection: spawned stdio adapter or existing TCP adapter.
use std::{
    io::{self, Read, Write},
    net::{Shutdown, TcpStream, ToSocketAddrs},
    path::Path,
    process::{Child, ChildStdin, Command, Stdio},
    time::{Duration, Instant},
};

use kinein_protocol::DebugConnectParams;

use super::{DebugError, adapter::Adapter, target::DebugTarget};

const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

/// Dropping a TCP connection never kills the externally owned debuggee.
#[derive(Debug)]
pub(super) enum Transport {
    Process(Child),
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
            Self::Process(child) => {
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
            .stderr(Stdio::null())
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
        let pipes = (child.stdin.take(), child.stdout.take());
        let transport = Self::Process(child);
        let (Some(stdin), Some(stdout)) = pipes else {
            return Err(DebugError::Adapter {
                message: "adapter subiu sem stdin/stdout utilizaveis".to_owned(),
            });
        };
        Ok((transport, Writer::Stdio(stdin), Box::new(stdout)))
    }
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

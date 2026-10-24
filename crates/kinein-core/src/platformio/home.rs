//! The PIO Home server the IDE shows (52 §14.45, `03-dominios/platformio.md`).
//!
//! One server per core, started on demand and reused: `pio home` on
//! `127.0.0.1`, a free port, a random 128-bit session and an idle shutdown,
//! beside the `pio` executable. PIO Home is `PlatformIO`'s own interface, not
//! project code. The owner stops it on `platformio.homeStop` and when it is
//! dropped (the core exiting).
//!
//! The server and its IDE agent (`ide_agent`, 52 §14.46) are one unit: ready
//! once the session page answers 200 and the agent connected, stopped
//! together, the agent first, and when either ends on its own the other is
//! ended too and the exit is reported once. The agent is a client of the
//! server, so the idle shutdown only ends a server whose core died.

use std::{
    fmt::Write as _,
    io::{BufRead, BufReader},
    net::TcpListener,
    path::Path,
    process::Command,
    sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError},
    thread,
    time::{Duration, Instant},
};

use super::ide_agent;
use crate::owned_child::{Env, OwnedChild};

/// `--shutdown-timeout`: the server stops after this long with no client.
///
/// The agent is a client while the core lives, so this ends only a server
/// whose core died (the UI stops an unused one, 52 §14.46).
pub const IDLE_SHUTDOWN_SECONDS: u32 = 300;
/// The first start installs `contrib-piohome` from the network.
pub const START_DEADLINE: Duration = Duration::from_secs(120);
/// Between the shutdown request and killing the process group.
const STOP_GRACE: Duration = Duration::from_secs(2);
const POLL: Duration = Duration::from_millis(200);
/// Output kept for an error message.
const OUTPUT_LIMIT: usize = 8 * 1024;

/// The running server, as `platformio.homeStart` answers it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Session {
    /// `http://127.0.0.1:<port>/session/<session>/`.
    pub url: String,
    /// The port the server listens on.
    pub port: u16,
}

#[derive(Default)]
enum State {
    #[default]
    Idle,
    Starting,
    Running(Running),
}

struct Running {
    session: Session,
    // `stop` takes the processes out of here before ending them, so the
    // watcher only ever finds processes that ended on their own.
    child: Arc<Mutex<Option<Processes>>>,
}

/// The server and its agent.
struct Processes {
    server: OwnedChild,
    agent: ide_agent::Agent,
}

impl Processes {
    /// The exit of whichever ended first, `None` while both run.
    fn exited(&mut self) -> Option<std::process::ExitStatus> {
        self.server.exited().or_else(|| self.agent.exited())
    }

    /// The agent first (its stdin closes and it leaves), then the server.
    fn close(self, shutdown: Option<&str>) {
        self.agent.close(STOP_GRACE);
        if let Some(url) = shutdown {
            drop(http().post(url).send_empty());
        }
        let _ = self.server.close(if shutdown.is_some() {
            STOP_GRACE
        } else {
            Duration::ZERO
        });
    }
}

#[derive(Default)]
struct Shared {
    state: Mutex<State>,
    changed: Condvar,
    /// Bumped by every release and start: a pending release only acts while
    /// the number is still its own, so the latest of the two wins.
    release: Mutex<u64>,
    released: Condvar,
}

impl Shared {
    fn lock(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Cancel a pending release; the new number.
    fn bump_release(&self) -> u64 {
        let mut generation = self.release.lock().unwrap_or_else(PoisonError::into_inner);
        *generation += 1;
        self.released.notify_all();
        *generation
    }
}

/// Owner of the PIO Home server; dropping it stops the server.
#[derive(Default)]
pub struct Home {
    shared: Arc<Shared>,
    /// The open workspace, for the agent's `get_pio_project_dirs`.
    workspace: Arc<Mutex<Option<std::path::PathBuf>>>,
}

impl Home {
    /// The live server, or a new one from `pio`. A call that arrives while
    /// another one starts the server waits for that start.
    ///
    /// `on_exit` runs once if the server or its agent later ends without
    /// [`Home::stop`], with the server's exit code when it was the server,
    /// else the agent's (`None` for a signal).
    ///
    /// # Errors
    /// No free port, no randomness, a process that cannot start or ends
    /// before answering, an agent that cannot connect, or `deadline` passing
    /// first; the message carries the tail of the output.
    pub fn start(
        &self,
        pio: &Path,
        deadline: Duration,
        events: Option<crate::lsp::EventSender>,
        on_exit: impl Fn(Option<i32>) + Send + 'static,
    ) -> Result<Session, String> {
        // The page is back: a pending release no longer applies.
        self.shared.bump_release();
        let mut state = self.shared.lock();
        loop {
            match &*state {
                State::Running(running) if alive(&running.child) => {
                    return Ok(running.session.clone());
                }
                State::Running(_) | State::Idle => break,
                State::Starting => {
                    state = self
                        .shared
                        .changed
                        .wait(state)
                        .unwrap_or_else(PoisonError::into_inner);
                }
            }
        }
        *state = State::Starting;
        drop(state);
        let context = ide_agent::Context {
            events,
            workspace: Arc::clone(&self.workspace),
        };
        let result = launch(pio, deadline, context).map(|(session, processes)| {
            let child = Arc::new(Mutex::new(Some(processes)));
            // Running before the watcher exists: an exit it sees always finds
            // its own server in the slot.
            *self.shared.lock() = State::Running(Running {
                session: session.clone(),
                child: Arc::clone(&child),
            });
            watch(Arc::clone(&self.shared), child, on_exit);
            session
        });
        if result.is_err() {
            *self.shared.lock() = State::Idle;
        }
        self.shared.changed.notify_all();
        result
    }

    /// Stop the server: the shutdown request, then the process group after a
    /// grace. `false` when none was running.
    #[must_use]
    pub fn stop(&self) -> bool {
        let Some((shutdown, processes)) = take_running(&self.shared) else {
            return false;
        };
        processes.close(Some(&shutdown));
        self.shared.changed.notify_all();
        true
    }

    /// The page left the screen: stop the server `after` from now, on the
    /// monotonic clock, unless a start comes first; another release restarts
    /// the countdown. The deadline stops whichever server runs then (after a
    /// stop, none). `on_idle` runs once if the deadline stops one. `false`
    /// when no server runs (or one is still starting).
    pub fn release(&self, after: Duration, on_idle: impl FnOnce() + Send + 'static) -> bool {
        if !matches!(&*self.shared.lock(), State::Running(_)) {
            return false;
        }
        let generation = self.shared.bump_release();
        let shared = Arc::clone(&self.shared);
        thread::spawn(move || {
            let deadline = Instant::now() + after;
            let mut current = shared
                .release
                .lock()
                .unwrap_or_else(PoisonError::into_inner);
            loop {
                if *current != generation {
                    return;
                }
                let left = deadline.saturating_duration_since(Instant::now());
                if left.is_zero() {
                    break;
                }
                current = shared
                    .released
                    .wait_timeout(current, left)
                    .unwrap_or_else(PoisonError::into_inner)
                    .0;
            }
            // Out of the slot before the number is released: a start that
            // comes now finds no server to reuse and launches a new one.
            let taken = take_running(&shared);
            drop(current);
            if let Some((shutdown, processes)) = taken {
                processes.close(Some(&shutdown));
                shared.changed.notify_all();
                on_idle();
            }
        });
        true
    }

    /// The workspace now open (`None` when closed): the agent answers
    /// `get_pio_project_dirs` with it when it is a `PlatformIO` project.
    pub fn set_workspace(&self, root: Option<&Path>) {
        *self
            .workspace
            .lock()
            .unwrap_or_else(PoisonError::into_inner) = root.map(Path::to_path_buf);
    }
}

impl std::fmt::Debug for Home {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let state = match &*self.shared.lock() {
            State::Idle => "idle",
            State::Starting => "starting",
            State::Running(_) => "running",
        };
        formatter
            .debug_struct("Home")
            .field("state", &state)
            .field("workspace", &self.workspace)
            .finish()
    }
}

impl Drop for Home {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

/// Take the running server out of the slot: its shutdown URL and its
/// processes, for the caller to close.
fn take_running(shared: &Shared) -> Option<(String, Processes)> {
    let running = {
        let mut state = shared.lock();
        if !matches!(&*state, State::Running(_)) {
            return None;
        }
        match std::mem::take(&mut *state) {
            State::Running(running) => running,
            _ => return None,
        }
    };
    let processes = running
        .child
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .take()?;
    Some((running.session.url + "__shutdown__", processes))
}

fn alive(child: &Mutex<Option<Processes>>) -> bool {
    child
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .as_mut()
        .is_some_and(|processes| processes.exited().is_none())
}

/// Spawn `pio home`, wait until the session page answers 200, then start
/// the IDE agent on the session's WebSocket.
fn launch(
    pio: &Path,
    deadline: Duration,
    context: ide_agent::Context,
) -> Result<(Session, Processes), String> {
    let started = Instant::now();
    let (session, server) = serve(pio, deadline)?;
    let socket = session.url.replacen("http://", "ws://", 1) + "wsrpc";
    let agent = ide_agent::interpreter(pio)
        .and_then(|python| ide_agent::spawn(&python, &socket, context, started + deadline));
    match agent {
        Ok(agent) => Ok((session, Processes { server, agent })),
        Err(message) => {
            let _ = server.close(Duration::ZERO);
            Err(message)
        }
    }
}

/// Spawn `pio home` and wait until the session page answers 200.
fn serve(pio: &Path, deadline: Duration) -> Result<(Session, OwnedChild), String> {
    let port = TcpListener::bind(("127.0.0.1", 0))
        .and_then(|listener| listener.local_addr())
        .map_err(|error| format!("nenhuma porta livre em 127.0.0.1: {error}"))?
        .port();
    let mut bytes = [0_u8; 16];
    getrandom::fill(&mut bytes)
        .map_err(|error| format!("o sistema não deu bytes aleatórios para a sessão: {error}"))?;
    let mut session = String::with_capacity(2 * bytes.len());
    for byte in bytes {
        let _ = write!(session, "{byte:02x}");
    }
    let mut command = Command::new(pio);
    command
        .args(["home", "--host", "127.0.0.1", "--port"])
        .arg(port.to_string())
        .args(["--no-open", "--session-id", &session, "--shutdown-timeout"])
        .arg(IDLE_SHUTDOWN_SECONDS.to_string())
        .current_dir(
            pio.parent()
                .map_or_else(std::env::temp_dir, Path::to_path_buf),
        )
        .env("PLATFORMIO_NO_ANSI", "1");
    let output = Arc::new(Mutex::new(String::new()));
    let sink = Arc::clone(&output);
    let mut owned = OwnedChild::spawn(command, Env::Inherit, None, move |stdout| {
        for line in BufReader::new(stdout).lines().map_while(Result::ok) {
            let mut text = sink.lock().unwrap_or_else(PoisonError::into_inner);
            if text.len() < OUTPUT_LIMIT {
                text.push_str(&line);
                text.push('\n');
            }
        }
    })
    .map_err(|error| format!("não foi possível iniciar {}: {error}", pio.display()))?;
    let url = format!("http://127.0.0.1:{port}/session/{session}/");
    let client = http();
    let started = Instant::now();
    let said = |owned: &OwnedChild| {
        let text = output
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone();
        format!("{}{}", text, owned.stderr_tail()).trim().to_owned()
    };
    loop {
        if let Some(status) = owned.exited() {
            return Err(format!(
                "o PIO Home terminou ({status}) antes de responder: {}",
                said(&owned)
            ));
        }
        if client
            .get(&url)
            .call()
            .is_ok_and(|response| response.status().as_u16() == 200)
        {
            return Ok((Session { url, port }, owned));
        }
        if started.elapsed() >= deadline {
            let message = format!(
                "o PIO Home não respondeu em {} s: {}",
                deadline.as_secs(),
                said(&owned)
            );
            let _ = owned.close(Duration::ZERO);
            return Err(message);
        }
        thread::sleep(POLL);
    }
}

/// Report an exit on its own once, and free the slot for a new server.
fn watch(
    shared: Arc<Shared>,
    child: Arc<Mutex<Option<Processes>>>,
    on_exit: impl Fn(Option<i32>) + Send + 'static,
) {
    thread::spawn(move || {
        loop {
            thread::sleep(POLL);
            let mut slot = child.lock().unwrap_or_else(PoisonError::into_inner);
            let Some(owned) = slot.as_mut() else {
                return;
            };
            let Some(status) = owned.exited() else {
                continue;
            };
            // Out of the slot before the grace: a start meanwhile sees it
            // gone and launches a new one.
            let processes = slot.take();
            drop(slot);
            if let Some(processes) = processes {
                processes.close(None);
            }
            let mut state = shared.lock();
            if matches!(&*state, State::Running(running) if Arc::ptr_eq(&running.child, &child)) {
                *state = State::Idle;
            }
            drop(state);
            shared.changed.notify_all();
            on_exit(status.code());
            return;
        }
    });
}

/// Local requests only: no proxy from the environment, short timeouts and the
/// status as data.
pub(super) fn http() -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(2)))
        .proxy(None)
        .http_status_as_error(false)
        .build()
        .into()
}

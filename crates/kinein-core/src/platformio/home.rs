//! The PIO Home server the IDE shows (52 §14.45, `03-dominios/platformio.md`).
//!
//! One server per core, started on demand and reused: `pio home` on
//! `127.0.0.1`, a free port, a random 128-bit session and an idle shutdown,
//! beside the `pio` executable. PIO Home is `PlatformIO`'s own interface, not
//! project code. Ready means the session page answers 200. The owner stops it
//! on `platformio.homeStop` and when it is dropped (the core exiting). An exit
//! on its own, the idle timeout or a failure, is reported once.

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

use crate::owned_child::{Env, OwnedChild};

/// `--shutdown-timeout`: with no page connected for this long, the server
/// stops on its own, also when the core died without stopping it.
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
    // `stop` takes the process out of here before ending it, so the watcher
    // only ever finds a process that ended on its own.
    child: Arc<Mutex<Option<OwnedChild>>>,
}

#[derive(Default)]
struct Shared {
    state: Mutex<State>,
    changed: Condvar,
}

impl Shared {
    fn lock(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// Owner of the PIO Home server; dropping it stops the server.
#[derive(Default)]
pub struct Home {
    shared: Arc<Shared>,
}

impl Home {
    /// The live server, or a new one from `pio`. A call that arrives while
    /// another one starts the server waits for that start.
    ///
    /// `on_exit` runs once if the server later ends without [`Home::stop`],
    /// with its exit code (`None` for a signal).
    ///
    /// # Errors
    /// No free port, no randomness, a process that cannot start or ends
    /// before answering, or `deadline` passing first; the message carries the
    /// tail of its output.
    pub fn start(
        &self,
        pio: &Path,
        deadline: Duration,
        on_exit: impl Fn(Option<i32>) + Send + 'static,
    ) -> Result<Session, String> {
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
        let result = launch(pio, deadline).map(|(session, owned)| {
            let child = Arc::new(Mutex::new(Some(owned)));
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
    pub fn stop(&self) -> bool {
        let running = {
            let mut state = self.shared.lock();
            if !matches!(&*state, State::Running(_)) {
                return false;
            }
            match std::mem::take(&mut *state) {
                State::Running(running) => running,
                _ => return false,
            }
        };
        let owned = running
            .child
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take();
        let Some(owned) = owned else {
            return false;
        };
        let shutdown = running.session.url + "__shutdown__";
        drop(agent().post(&shutdown).send_empty());
        let _ = owned.close(STOP_GRACE);
        self.shared.changed.notify_all();
        true
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
            .finish()
    }
}

impl Drop for Home {
    fn drop(&mut self) {
        self.stop();
    }
}

fn alive(child: &Mutex<Option<OwnedChild>>) -> bool {
    child
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .as_mut()
        .is_some_and(|owned| owned.exited().is_none())
}

/// Spawn `pio home` and wait until the session page answers 200.
fn launch(pio: &Path, deadline: Duration) -> Result<(Session, OwnedChild), String> {
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
    let agent = agent();
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
        if agent
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
    child: Arc<Mutex<Option<OwnedChild>>>,
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
            if let Some(owned) = slot.take() {
                let _ = owned.close(Duration::ZERO);
            }
            drop(slot);
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
fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(2)))
        .proxy(None)
        .http_status_as_error(false)
        .build()
        .into()
}

#[cfg(test)]
mod tests {
    use std::{
        path::{Path, PathBuf},
        sync::mpsc,
        time::Duration,
    };

    use super::{Home, agent};

    /// A fake `pio` that serves the session page (200), the root (403) and
    /// the shutdown, like PIO Home; files beside it change how it behaves.
    const FAKE: &str = r#"#!/usr/bin/env python3
import http.server, json, os, sys, threading, time
from pathlib import Path
here = Path(__file__).resolve().parent
args = sys.argv[1:]
with open(here / "home.launches", "a") as launches:
    launches.write(json.dumps(args) + "\n")
(here / "home.cwd").write_text(os.getcwd())
if (here / "mode-exit").exists():
    print("Error: fixture refuses to start", flush=True)
    sys.exit(3)
port = int(args[args.index("--port") + 1])
prefix = "/session/%s/" % args[args.index("--session-id") + 1]
class Handler(http.server.BaseHTTPRequestHandler):
    def log_message(self, *ignored):
        pass
    def do_GET(self):
        ok = self.path == prefix and not (here / "mode-wrong").exists()
        self.send_response(200 if ok else 403)
        self.end_headers()
        self.wfile.write(b"home" if ok else b"Protected PlatformIO Home session")
    def do_POST(self):
        if self.path != prefix + "__shutdown__":
            self.send_response(403)
            self.end_headers()
            return
        self.send_response(200)
        self.end_headers()
        self.wfile.write(b"Server has been shutdown!")
        self.wfile.flush()
        (here / "home.shutdown").write_text("yes")
        os._exit(0)
server = http.server.HTTPServer(("127.0.0.1", port), Handler)
if (here / "mode-silent").exists():
    time.sleep(60)
    sys.exit(0)
if (here / "mode-short").exists():
    threading.Timer(1.0, lambda: os._exit(7)).start()
print("PIO Home has been started. Press Ctrl+C to shutdown.", flush=True)
server.serve_forever()
"#;

    fn bench(name: &str) -> (PathBuf, PathBuf) {
        let bin = std::env::temp_dir()
            .join("kinein-platformio-home")
            .join(format!("{}-{name}", std::process::id()));
        drop(std::fs::remove_dir_all(&bin));
        std::fs::create_dir_all(&bin).unwrap();
        let pio = crate::write_executable(bin.join("pio"), FAKE);
        (bin, pio)
    }

    fn launches(bin: &Path) -> Vec<Vec<String>> {
        std::fs::read_to_string(bin.join("home.launches"))
            .unwrap_or_default()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect()
    }

    fn status(url: &str) -> Option<u16> {
        agent()
            .get(url)
            .call()
            .ok()
            .map(|response| response.status().as_u16())
    }

    #[test]
    fn the_session_page_is_served_reused_and_the_root_refused() {
        let (bin, pio) = bench("session");
        let home = Home::default();
        let session = home.start(&pio, Duration::from_secs(20), |_| {}).unwrap();
        let launched = launches(&bin);
        assert_eq!(launched.len(), 1);
        let id = &launched[0][launched[0]
            .iter()
            .position(|arg| arg == "--session-id")
            .unwrap()
            + 1];
        assert_eq!(id.len(), 32, "128 bits in hex");
        assert!(id.bytes().all(|byte| byte.is_ascii_hexdigit()));
        let port = session.port.to_string();
        assert_eq!(
            launched[0],
            [
                "home",
                "--host",
                "127.0.0.1",
                "--port",
                &port,
                "--no-open",
                "--session-id",
                id,
                "--shutdown-timeout",
                "300"
            ]
        );
        assert_eq!(
            session.url,
            format!("http://127.0.0.1:{port}/session/{id}/")
        );
        assert_eq!(
            std::fs::read_to_string(bin.join("home.cwd")).unwrap(),
            crate::platform::canonicalize(&bin)
                .unwrap()
                .display()
                .to_string(),
            "beside the executable, outside any project"
        );
        assert_eq!(status(&session.url), Some(200));
        assert_eq!(status(&format!("http://127.0.0.1:{port}/")), Some(403));
        let again = home.start(&pio, Duration::from_secs(20), |_| {}).unwrap();
        assert_eq!(again, session);
        assert_eq!(launches(&bin).len(), 1, "the live server is reused");
    }

    #[test]
    fn stopping_shuts_the_server_down_and_frees_its_port() {
        let (bin, pio) = bench("stop");
        let home = Home::default();
        let (sender, exits) = mpsc::channel();
        let session = home
            .start(&pio, Duration::from_secs(20), move |code| {
                let _ = sender.send(code);
            })
            .unwrap();
        assert!(home.stop());
        assert!(
            bin.join("home.shutdown").exists(),
            "the shutdown request came first"
        );
        assert_eq!(status(&session.url), None, "nothing listens any more");
        assert!(!home.stop(), "nothing left to stop");
        assert!(
            exits.recv_timeout(Duration::from_secs(1)).is_err(),
            "a requested stop is not an exit on its own"
        );
        let next = home.start(&pio, Duration::from_secs(20), |_| {}).unwrap();
        let id = |url: &str| {
            url.trim_end_matches('/')
                .rsplit('/')
                .next()
                .unwrap()
                .to_owned()
        };
        assert_ne!(
            id(&next.url),
            id(&session.url),
            "a new server, a new random session"
        );
        drop(home);
        assert_eq!(status(&next.url), None, "dropping the owner stops it");
    }

    #[test]
    fn an_exit_on_its_own_is_reported_once_and_frees_the_slot() {
        let (bin, pio) = bench("short");
        std::fs::write(bin.join("mode-short"), "").unwrap();
        let home = Home::default();
        let (sender, exits) = mpsc::channel();
        home.start(&pio, Duration::from_secs(20), move |code| {
            let _ = sender.send(code);
        })
        .unwrap();
        assert_eq!(exits.recv_timeout(Duration::from_secs(10)), Ok(Some(7)));
        assert!(
            exits.recv_timeout(Duration::from_millis(600)).is_err(),
            "once"
        );
        std::fs::remove_file(bin.join("mode-short")).unwrap();
        home.start(&pio, Duration::from_secs(20), |_| {}).unwrap();
        assert_eq!(launches(&bin).len(), 2, "the next start launches again");
    }

    #[test]
    fn a_server_that_ends_or_never_answers_is_an_error_with_its_output() {
        let (bin, pio) = bench("failures");
        std::fs::write(bin.join("mode-exit"), "").unwrap();
        let home = Home::default();
        let error = home
            .start(&pio, Duration::from_secs(20), |_| {})
            .unwrap_err();
        assert!(error.contains("fixture refuses to start"), "{error}");
        std::fs::remove_file(bin.join("mode-exit")).unwrap();
        std::fs::write(bin.join("mode-silent"), "").unwrap();
        let error = home
            .start(&pio, Duration::from_secs(2), |_| {})
            .unwrap_err();
        assert!(error.contains("não respondeu em 2 s"), "{error}");
        assert!(!home.stop(), "a failed start leaves nothing running");
        // A server that answers, but not 200 on this session, is not ready.
        std::fs::remove_file(bin.join("mode-silent")).unwrap();
        std::fs::write(bin.join("mode-wrong"), "").unwrap();
        let error = home
            .start(&pio, Duration::from_secs(2), |_| {})
            .unwrap_err();
        assert!(error.contains("não respondeu em 2 s"), "{error}");
    }

    #[test]
    fn concurrent_starts_share_one_launch() {
        let (bin, pio) = bench("concurrent");
        let home = std::sync::Arc::new(Home::default());
        // All three start before any is joined: that is the race under test.
        let mut threads = Vec::new();
        for _ in 0..3 {
            let home = std::sync::Arc::clone(&home);
            let pio = pio.clone();
            threads.push(std::thread::spawn(move || {
                home.start(&pio, Duration::from_secs(20), |_| {})
            }));
        }
        let sessions: Vec<_> = threads
            .into_iter()
            .map(|thread| thread.join().unwrap().unwrap())
            .collect();
        assert!(sessions.windows(2).all(|pair| pair[0] == pair[1]));
        assert_eq!(launches(&bin).len(), 1);
    }

    fn rpc(core: &mut crate::Core, method: &str) -> kinein_protocol::JsonRpcResponse {
        core.handle_request(&kinein_protocol::JsonRpcRequest::new(
            7_i64,
            method,
            Some(serde_json::json!({})),
        ))
        .response()
        .clone()
    }

    /// Through the protocol: a missing `pio`, start and stop, and the event
    /// of a server that ends on its own.
    #[test]
    fn the_methods_answer_and_an_exit_on_its_own_becomes_the_event() {
        let empty = std::env::temp_dir()
            .join("kinein-platformio-home")
            .join(format!("{}-empty", std::process::id()));
        std::fs::create_dir_all(&empty).unwrap();
        let mut core =
            crate::Core::with_detector(crate::tools::ToolDetector::with_search_path(empty));
        let missing = rpc(&mut core, "platformio.homeStart");
        assert_eq!(
            missing.error.map(|error| error.code),
            Some(kinein_protocol::JsonRpcErrorCode::ToolNotFound)
        );

        let (bin, _) = bench("rpc");
        let mut core =
            crate::Core::with_detector(crate::tools::ToolDetector::with_search_path(bin.clone()));
        let (sender, events) = mpsc::channel();
        core.enable_lsp(sender);
        let started = rpc(&mut core, "platformio.homeStart");
        let url = started.result.as_ref().unwrap()["url"]
            .as_str()
            .unwrap()
            .to_owned();
        assert_eq!(status(&url), Some(200));
        let stopped = rpc(&mut core, "platformio.homeStop");
        assert_eq!(stopped.result.unwrap()["stopped"], true);

        std::fs::write(bin.join("mode-short"), "").unwrap();
        assert!(rpc(&mut core, "platformio.homeStart").error.is_none());
        let deadline = std::time::Instant::now() + Duration::from_secs(10);
        let exited = loop {
            let event = events.recv_timeout(Duration::from_millis(200));
            if let Ok(event) = event
                && event.method == "event.platformio.homeExited"
            {
                break event;
            }
            assert!(std::time::Instant::now() < deadline, "no exit event");
        };
        assert_eq!(exited.params.unwrap()["code"], 7);
        assert_eq!(
            rpc(&mut core, "platformio.homeStop").result.unwrap()["stopped"],
            false
        );
    }
}

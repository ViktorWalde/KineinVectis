//! O lançador dos servidores de linguagem e o relé.
//!
//! Um processo por servidor e raiz, criado aqui, nunca pelo Neovim. O Neovim
//! conecta num socket do relé, que repassa os quadros LSP sem alterá-los e
//! observa o `publishDiagnostics` (o painel de problemas) e o tempo de cada
//! resposta. Uma segunda conexão simultânea para a mesma chave é recusada e
//! contada: é a sessão duplicada do R7, impedida pela estrutura. A conexão
//! que chega enquanto a sessão anterior termina (o Neovim dela já fechou: um
//! reinício do cliente ou um nvim novo depois de uma queda) espera o servidor
//! velho sair e então assume a chave.

use crate::sandbox;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::Shutdown;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Clone, Default)]
pub struct CppContext {
    pub compile_commands_dir: Option<PathBuf>,
    pub query_driver: Option<String>,
}

pub struct Server {
    pub name: String,
    pub root: PathBuf,
    pub socket: PathBuf,
    pub connections: AtomicU64,
    pub refused: AtomicU64,
    pub launches: AtomicU64,
    pub pid: Mutex<Option<u32>>,
    pub args: Mutex<Vec<String>>,
    pub latencies: Mutex<HashMap<String, Vec<f64>>>,
    active: AtomicBool,
    /// O Neovim da sessão ativa fechou a conexão; o servidor está saindo.
    draining: AtomicBool,
    child: Mutex<Option<Child>>,
}

/// Quanto uma conexão nova espera a sessão anterior: o servidor tem 2 s
/// para sair sozinho depois do fim do stdin, e o vigia o mata depois disso.
const TAKEOVER_WAIT: Duration = Duration::from_secs(5);
/// Uma conexão nova pode chegar antes de o relé ver o fim da anterior (a
/// queda do nvim fecha o socket, o nvim novo conecta logo): por este tempo,
/// uma sessão ativa ainda pode virar uma sessão saindo.
const DRAIN_NOTICE: Duration = Duration::from_millis(300);

/// O maior nome de socket que o relé cria na pasta `run` (até 99 chaves).
pub const LONGEST_SOCKET_NAME: &str = "lsp-rust-analyzer-99.sock";

pub struct Lsp {
    servers: Mutex<HashMap<(String, PathBuf), Arc<Server>>>,
    run_dir: PathBuf,
    log_dir: PathBuf,
    project: PathBuf,
    trusted: bool,
    pub context: Mutex<CppContext>,
    pub diagnostics: Mutex<HashMap<String, Value>>,
    pub progress_events: AtomicU64,
    stop: AtomicBool,
    /// O que o relé fez com cada conexão, em ordem: a evidência do R7.
    events: Mutex<Vec<Value>>,
    started: Instant,
}

fn find_upward(start: &Path, limit: &Path, marker: &str) -> Option<PathBuf> {
    let mut dir = start.parent()?;
    loop {
        if dir.join(marker).exists() {
            return Some(dir.to_path_buf());
        }
        if dir == limit {
            return None;
        }
        dir = dir.parent()?;
    }
}

fn read_frame(reader: &mut impl BufRead) -> io::Result<Option<(Vec<u8>, Vec<u8>)>> {
    let mut header = Vec::new();
    let mut length = None;
    loop {
        let mut line = Vec::new();
        if reader.read_until(b'\n', &mut line)? == 0 {
            return Ok(None);
        }
        header.extend_from_slice(&line);
        let text = String::from_utf8_lossy(&line);
        let text = text.trim_end();
        if text.is_empty() {
            break;
        }
        if let Some(v) = text.strip_prefix("Content-Length:") {
            length = v.trim().parse::<usize>().ok();
        }
    }
    let length = length.ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "quadro LSP sem Content-Length"))?;
    let mut body = vec![0; length];
    reader.read_exact(&mut body)?;
    Ok(Some((header, body)))
}

impl Lsp {
    pub fn new(run_dir: PathBuf, log_dir: PathBuf, project: PathBuf, trusted: bool) -> Arc<Self> {
        Arc::new(Self {
            servers: Mutex::new(HashMap::new()),
            run_dir,
            log_dir,
            project,
            trusted,
            context: Mutex::new(CppContext::default()),
            diagnostics: Mutex::new(HashMap::new()),
            progress_events: AtomicU64::new(0),
            stop: AtomicBool::new(false),
            events: Mutex::new(Vec::new()),
            started: Instant::now(),
        })
    }

    /// A configuração que o Lua da Kinein usa no `vim.lsp.start`, ou nada.
    pub fn config(self: &Arc<Self>, filetype: &str, path: &Path) -> Option<Value> {
        let (name, root) = match filetype {
            "c" | "cpp" => ("clangd", self.project.clone()),
            "rust" => {
                // Sem confiança, nada do projeto roda: o rust-analyzer roda o
                // build.rs e o rustc-wrapper do projeto (INF-23), então não sobe.
                if !self.trusted {
                    return None;
                }
                ("rust-analyzer", find_upward(path, &self.project, "Cargo.toml")?)
            }
            _ => return None,
        };
        if !path.starts_with(&self.project) {
            return None;
        }
        let server = match self.server(name, &root) {
            Ok(s) => s,
            Err(e) => {
                self.note(name, "relé não criado", &e.to_string());
                return None;
            }
        };
        let settings = if name == "rust-analyzer" {
            json!({ "rust-analyzer": { "checkOnSave": true } })
        } else {
            Value::Null
        };
        Some(json!({
            "name": name,
            "socket": server.socket,
            "root_dir": root,
            "settings": settings,
        }))
    }

    fn server(self: &Arc<Self>, name: &str, root: &Path) -> io::Result<Arc<Server>> {
        let mut servers = self.servers.lock().unwrap();
        let key = (name.to_owned(), root.to_path_buf());
        if let Some(s) = servers.get(&key) {
            return Ok(s.clone());
        }
        let socket = self.run_dir.join(format!("lsp-{}-{}.sock", name, servers.len()));
        let _ = std::fs::remove_file(&socket);
        let listener = UnixListener::bind(&socket)?;
        std::fs::set_permissions(&socket, std::fs::Permissions::from_mode(0o600))?;
        let server = Arc::new(Server {
            name: name.to_owned(),
            root: root.to_path_buf(),
            socket,
            connections: AtomicU64::new(0),
            refused: AtomicU64::new(0),
            launches: AtomicU64::new(0),
            pid: Mutex::new(None),
            args: Mutex::new(Vec::new()),
            latencies: Mutex::new(HashMap::new()),
            active: AtomicBool::new(false),
            draining: AtomicBool::new(false),
            child: Mutex::new(None),
        });
        servers.insert(key, server.clone());
        let lsp = self.clone();
        let s = server.clone();
        // Esta thread vive até o fim do supervisor: o PDEATHSIG do servidor
        // está preso a ela.
        std::thread::Builder::new()
            .name(format!("relé {name}"))
            .spawn(move || lsp.accept_loop(&s, listener))?;
        Ok(server)
    }

    fn command(&self, server: &Server) -> Command {
        let mut args = Vec::new();
        let program = if server.name == "clangd" {
            args.push("--background-index=false".to_owned());
            args.push("--log=error".to_owned());
            args.push("--pch-storage=memory".to_owned());
            let ctx = self.context.lock().unwrap().clone();
            if let Some(dir) = ctx.compile_commands_dir {
                args.push(format!("--compile-commands-dir={}", dir.display()));
            }
            // O --query-driver executa o compilador: só com confiança (INF-23, E5).
            if let (true, Some(driver)) = (self.trusted, ctx.query_driver) {
                args.push(format!("--query-driver={driver}"));
            }
            std::env::var("KINEIN_PROVA_CLANGD").unwrap_or_else(|_| "clangd".into())
        } else {
            std::env::var("KINEIN_PROVA_RUST_ANALYZER").unwrap_or_else(|_| "rust-analyzer".into())
        };
        *server.args.lock().unwrap() = args.clone();
        let mut cmd = Command::new(program);
        cmd.args(&args).current_dir(&server.root).stdin(Stdio::piped()).stdout(Stdio::piped());
        let log = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.log_dir.join(format!("{}.log", server.name)));
        cmd.stderr(log.map(Stdio::from).unwrap_or_else(|_| Stdio::null()));
        sandbox::supervised(&mut cmd);
        cmd
    }

    fn accept_loop(self: &Arc<Self>, server: &Arc<Server>, listener: UnixListener) {
        for stream in listener.incoming() {
            if self.stop.load(Ordering::Acquire) {
                return;
            }
            let Ok(stream) = stream else { continue };
            if let Err(reason) = self.claim(server) {
                server.refused.fetch_add(1, Ordering::Relaxed);
                self.note(&server.name, "recusada", reason);
                let _ = stream.shutdown(Shutdown::Both);
                continue;
            }
            self.note(&server.name, "aceita", "");
            server.connections.fetch_add(1, Ordering::Relaxed);
            let mut child = match self.command(server).spawn() {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("supervisor: {} não subiu: {e}", server.name);
                    server.active.store(false, Ordering::Release);
                    continue;
                }
            };
            server.launches.fetch_add(1, Ordering::Relaxed);
            *server.pid.lock().unwrap() = Some(child.id());
            let stdin = child.stdin.take().unwrap();
            let stdout = child.stdout.take().unwrap();
            *server.child.lock().unwrap() = Some(child);
            self.session(server.clone(), stream, stdin, stdout);
        }
    }

    /// Assume a chave do servidor: livre, ou ocupada por uma sessão que está
    /// saindo. Uma sessão viva (o Neovim dela ainda conectado) não é tomada.
    fn claim(&self, server: &Server) -> Result<(), &'static str> {
        if !server.active.swap(true, Ordering::AcqRel) {
            return Ok(());
        }
        let start = Instant::now();
        loop {
            std::thread::sleep(Duration::from_millis(10));
            if !server.active.swap(true, Ordering::AcqRel) {
                self.note(&server.name, "esperou a sessão anterior", &format!("{} ms", start.elapsed().as_millis()));
                return Ok(());
            }
            if start.elapsed() > TAKEOVER_WAIT {
                return Err("a sessão anterior não terminou");
            }
            // O vigia baixa o `active` antes do `draining`: quem vê o
            // `draining` baixo vê também a chave livre, se a sessão acabou.
            if !server.draining.load(Ordering::Acquire) && start.elapsed() > DRAIN_NOTICE {
                if !server.active.swap(true, Ordering::AcqRel) {
                    return Ok(());
                }
                return Err("sessão viva");
            }
        }
    }

    fn note(&self, server: &str, event: &str, detail: &str) {
        self.events.lock().unwrap().push(json!({
            "t_ms": self.started.elapsed().as_millis() as u64,
            "servidor": server,
            "evento": event,
            "detalhe": detail,
        }));
    }

    fn session(self: &Arc<Self>, server: Arc<Server>, stream: UnixStream, mut stdin: impl Write + Send + 'static, stdout: impl Read + Send + 'static) {
        let pending: Arc<Mutex<HashMap<String, (String, Instant)>>> = Arc::default();
        let from_nvim = stream.try_clone().unwrap();
        let mut to_nvim = stream.try_clone().unwrap();
        let p = pending.clone();
        let up = std::thread::spawn(move || {
            let mut reader = BufReader::new(from_nvim);
            while let Ok(Some((header, body))) = read_frame(&mut reader) {
                if let Ok(msg) = serde_json::from_slice::<Value>(&body) {
                    if let (Some(id), Some(method)) = (msg.get("id"), msg.get("method").and_then(Value::as_str)) {
                        p.lock().unwrap().insert(id.to_string(), (method.to_owned(), Instant::now()));
                    }
                }
                if stdin.write_all(&header).and_then(|_| stdin.write_all(&body)).and_then(|_| stdin.flush()).is_err() {
                    break;
                }
            }
            // O Neovim saiu: o stdin do servidor fecha, e ele termina.
        });
        let lsp = self.clone();
        let s = server.clone();
        let down = std::thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            while let Ok(Some((header, body))) = read_frame(&mut reader) {
                if let Ok(msg) = serde_json::from_slice::<Value>(&body) {
                    match msg.get("method").and_then(Value::as_str) {
                        Some("textDocument/publishDiagnostics") => {
                            let uri = msg["params"]["uri"].as_str().unwrap_or_default().to_owned();
                            lsp.diagnostics.lock().unwrap().insert(uri, msg["params"]["diagnostics"].clone());
                        }
                        Some("$/progress") => {
                            lsp.progress_events.fetch_add(1, Ordering::Relaxed);
                        }
                        Some(_) => {}
                        None => {
                            if let Some(id) = msg.get("id") {
                                if let Some((method, at)) = pending.lock().unwrap().remove(&id.to_string()) {
                                    let ms = at.elapsed().as_secs_f64() * 1000.0;
                                    s.latencies.lock().unwrap().entry(method).or_default().push(ms);
                                }
                            }
                        }
                    }
                }
                if to_nvim.write_all(&header).and_then(|_| to_nvim.write_all(&body)).is_err() {
                    break;
                }
            }
            // O servidor saiu: o Neovim precisa ver o fim da conexão.
            let _ = to_nvim.shutdown(Shutdown::Both);
        });
        let watcher_server = server;
        let lsp = self.clone();
        std::thread::spawn(move || {
            let _ = up.join();
            watcher_server.draining.store(true, Ordering::Release);
            lsp.note(&watcher_server.name, "o Neovim fechou a conexão", "");
            // O stdin fechou; o servidor tem dois segundos para sair sozinho.
            let deadline = Instant::now() + Duration::from_secs(2);
            loop {
                let done = watcher_server
                    .child
                    .lock()
                    .unwrap()
                    .as_mut()
                    .map(|c| c.try_wait().map(|s| s.is_some()).unwrap_or(true))
                    .unwrap_or(true);
                if done {
                    break;
                }
                if Instant::now() > deadline {
                    if let Some(c) = watcher_server.child.lock().unwrap().as_mut() {
                        let _ = c.kill();
                        let _ = c.wait();
                    }
                    break;
                }
                std::thread::sleep(Duration::from_millis(20));
            }
            let _ = down.join();
            let _ = stream.shutdown(Shutdown::Both);
            *watcher_server.child.lock().unwrap() = None;
            *watcher_server.pid.lock().unwrap() = None;
            watcher_server.active.store(false, Ordering::Release);
            watcher_server.draining.store(false, Ordering::Release);
            lsp.note(&watcher_server.name, "sessão terminou", "");
        });
    }

    pub fn shutdown(&self) {
        self.stop.store(true, Ordering::Release);
        for s in self.servers.lock().unwrap().values() {
            if let Some(c) = s.child.lock().unwrap().as_mut() {
                let _ = c.kill();
                let _ = c.wait();
            }
            // Acorda o accept para a thread ver o fim.
            let _ = UnixStream::connect(&s.socket);
        }
    }

    pub fn trusted(&self) -> bool {
        self.trusted
    }

    pub fn status(&self) -> Value {
        let servers: Vec<Value> = self
            .servers
            .lock()
            .unwrap()
            .values()
            .map(|s| {
                let lat: serde_json::Map<String, Value> = s
                    .latencies
                    .lock()
                    .unwrap()
                    .iter()
                    .map(|(m, v)| (m.clone(), json!(v)))
                    .collect();
                json!({
                    "nome": s.name,
                    "raiz": s.root,
                    "socket": s.socket,
                    "pid": *s.pid.lock().unwrap(),
                    "args": *s.args.lock().unwrap(),
                    "conexoes": s.connections.load(Ordering::Relaxed),
                    "recusadas": s.refused.load(Ordering::Relaxed),
                    "lancamentos": s.launches.load(Ordering::Relaxed),
                    "ativo": s.active.load(Ordering::Relaxed),
                    "latencias_ms": lat,
                })
            })
            .collect();
        json!({
            "confiavel": self.trusted,
            "servidores": servers,
            "progresso": self.progress_events.load(Ordering::Relaxed),
            "eventos": *self.events.lock().unwrap(),
        })
    }
}

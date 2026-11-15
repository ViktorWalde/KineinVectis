//! O supervisor da prova: faz o papel do core diante do Neovim.
//!
//! Cria o `nvim` pelo lançador (Landlock e PDEATHSIG), confere a contenção
//! antes de entregar o editor (falha fechado), espelha os buffers, grava o
//! rascunho, lança os servidores de linguagem e faz o relé, e recupera o texto
//! quando o `nvim` cai. A UI recebe só o endereço do socket do `nvim`; o
//! driver dos testes fala com o supervisor por um socket de controle.

mod lsp;
mod mirror;
mod rpc;
mod sandbox;

use mirror::{Drafts, Mirror};
use rmpv::Value as Mp;
use rpc::{Incoming, Rpc};
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(20);

#[derive(Clone, Copy, PartialEq)]
enum Recovery {
    Mirror,
    Swap,
}

struct Config {
    project: PathBuf,
    state: PathBuf,
    run: PathBuf,
    runtime: PathBuf,
    nvim: PathBuf,
    trusted: bool,
    recovery: Recovery,
    core_writes: bool,
    plant_no_landlock: bool,
    plant_no_probe: bool,
}

struct NvimProc {
    pid: u32,
    rpc: Arc<Rpc>,
    socket: PathBuf,
    generation: u64,
}

enum Job {
    Attach(Arc<Rpc>, i64),
}

struct Sup {
    cfg: Config,
    current: Mutex<Option<Arc<NvimProc>>>,
    mirror: Mutex<Mirror>,
    drafts: Drafts,
    lsp: Arc<lsp::Lsp>,
    out: Mutex<std::io::Stdout>,
    stop: AtomicBool,
    ready: Mutex<Option<(u64, Value)>>,
    ready_cv: Condvar,
    probe: Mutex<Value>,
    log: Mutex<Vec<Value>>,
    jobs: Mutex<mpsc::Sender<Job>>,
    protocol_errors: AtomicU64,
    started: Instant,
    order: Mutex<Vec<PathBuf>>,
}

fn env_flag(name: &str) -> bool {
    std::env::var(name).is_ok_and(|v| v == "1")
}

fn arg(args: &[String], name: &str) -> Option<String> {
    args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).cloned()
}

fn tail(bytes: &[u8]) -> String {
    let s = String::from_utf8_lossy(bytes);
    let mut start = s.len().saturating_sub(4000);
    while !s.is_char_boundary(start) {
        start += 1;
    }
    s[start..].to_owned()
}

impl Sup {
    fn emit(&self, event: Value) {
        let mut out = self.out.lock().unwrap();
        let _ = writeln!(out, "{event}");
        let _ = out.flush();
    }

    fn record(&self, mut event: Value) {
        event["t_ms"] = json!(self.started.elapsed().as_millis() as u64);
        self.log.lock().unwrap().push(event);
    }

    fn nvim(&self) -> Result<Arc<NvimProc>, String> {
        self.current.lock().unwrap().clone().ok_or_else(|| "nvim indisponível".to_owned())
    }

    fn exec_lua(&self, code: &str, args: Vec<Mp>) -> Result<Value, String> {
        let p = self.nvim()?;
        p.rpc
            .request("nvim_exec_lua", vec![code.into(), Mp::Array(args)], REQUEST_TIMEOUT)
            .map(|v| rpc::to_json(&v))
    }

    fn touch(&self, path: &Path) {
        let mut order = self.order.lock().unwrap();
        order.retain(|p| p != path);
        order.push(path.to_path_buf());
    }

    // --- o que chega do nvim pelo canal do core ---------------------------

    fn on_incoming(self: &Arc<Self>, proc_rpc: &Arc<Rpc>, generation: u64, msg: Incoming) {
        match msg {
            Incoming::Notification { method, params } => match method.as_str() {
                "nvim_buf_lines_event" => {
                    let buf = params.first().and_then(rpc::handle).unwrap_or(-1);
                    let tick = params.get(1).and_then(Mp::as_i64).unwrap_or(0);
                    let first = params.get(2).and_then(Mp::as_i64).unwrap_or(0);
                    let last = params.get(3).and_then(Mp::as_i64).unwrap_or(0);
                    let lines = params
                        .get(4)
                        .and_then(Mp::as_array)
                        .map(|a| a.iter().map(rpc::text).collect())
                        .unwrap_or_default();
                    if self.mirror.lock().unwrap().apply(buf, tick, first, last, lines) {
                        self.drafts.poke();
                    }
                }
                "nvim_buf_changedtick_event" => {
                    let buf = params.first().and_then(rpc::handle).unwrap_or(-1);
                    let tick = params.get(1).and_then(Mp::as_i64).unwrap_or(0);
                    self.mirror.lock().unwrap().tick(buf, tick);
                }
                "kinein_ready" => {
                    let containment = params.first().map(rpc::to_json).unwrap_or(Value::Null);
                    *self.ready.lock().unwrap() = Some((generation, containment));
                    self.ready_cv.notify_all();
                }
                "kinein_buf_open" => {
                    let buf = params.first().and_then(rpc::handle).unwrap_or(-1);
                    let path = params.get(1).map(rpc::text).unwrap_or_default();
                    if buf > 0 && !path.is_empty() {
                        let path = PathBuf::from(path);
                        self.mirror.lock().unwrap().open(buf, path.clone());
                        self.touch(&path);
                        let _ = self.jobs.lock().unwrap().send(Job::Attach(proc_rpc.clone(), buf));
                    }
                }
                "kinein_buf_written" => {
                    let path = PathBuf::from(params.get(1).map(rpc::text).unwrap_or_default());
                    self.mirror.lock().unwrap().written(&path);
                    self.drafts.poke();
                    self.record(json!({"evento": "gravado_pelo_nvim", "path": path}));
                }
                "kinein_ui_open" => {
                    self.record(json!({"evento": "ui_open", "alvo": params.first().map(rpc::text)}));
                }
                _ => {}
            },
            Incoming::Request { id, method, params } => {
                let result = match method.as_str() {
                    "kinein_lsp_config" => {
                        let ft = params.first().map(rpc::text).unwrap_or_default();
                        let path = PathBuf::from(params.get(1).map(rpc::text).unwrap_or_default());
                        Ok(self.lsp.config(&ft, &path).map(|v| rpc::from_json(&v)).unwrap_or(Mp::Nil))
                    }
                    "kinein_write" => Ok(rpc::from_json(&self.write_file(&params))),
                    other => Err(format!("método desconhecido: {other}")),
                };
                proc_rpc.respond(id, result);
            }
        }
    }

    /// A gravação pelo core (§18, item 2): confere que o disco não mudou
    /// desde a última leitura e grava por arquivo temporário e rename.
    fn write_file(&self, params: &[Mp]) -> Value {
        let path = PathBuf::from(params.get(1).map(rpc::text).unwrap_or_default());
        let lines: Vec<String> =
            params.get(2).and_then(Mp::as_array).map(|a| a.iter().map(rpc::text).collect()).unwrap_or_default();
        let eol = params.get(3).and_then(Mp::as_bool).unwrap_or(true);
        let mut mirror = self.mirror.lock().unwrap();
        if let Some(doc) = mirror.docs.get(&path) {
            if path.exists() && mirror::read_lines(&path) != doc.disk {
                return json!({"ok": false, "motivo": "o arquivo mudou no disco desde a leitura"});
            }
        }
        let mut text = lines.join("\n");
        if eol {
            text.push('\n');
        }
        let name = path.file_name().unwrap_or_default().to_string_lossy().into_owned();
        let tmp = path.with_file_name(format!(".{name}.kinein-tmp"));
        let perms = std::fs::metadata(&path).map(|m| m.permissions()).ok();
        let result = std::fs::write(&tmp, &text)
            .and_then(|_| match perms {
                Some(p) => std::fs::set_permissions(&tmp, p),
                None => Ok(()),
            })
            .and_then(|_| std::fs::rename(&tmp, &path));
        match result {
            Ok(()) => {
                mirror.written(&path);
                drop(mirror);
                self.drafts.poke();
                self.record(json!({"evento": "gravado_pelo_core", "path": path}));
                json!({"ok": true})
            }
            Err(e) => json!({"ok": false, "motivo": e.to_string()}),
        }
    }

    // --- o ciclo de vida do nvim ------------------------------------------

    fn spawn_nvim(self: &Arc<Self>, generation: u64) -> Result<(std::process::Child, Arc<NvimProc>), String> {
        let socket = self.cfg.run.join(format!("nvim-{generation}.sock"));
        let _ = std::fs::remove_file(&socket);
        let xdg = self.cfg.state.join("xdg");
        let mut cmd = Command::new(&self.cfg.nvim);
        cmd.arg("--embed")
            .arg("--headless")
            .arg("--listen")
            .arg(&socket)
            .arg("-u")
            .arg(self.cfg.runtime.join("init.lua"))
            .arg("-i")
            .arg(self.cfg.state.join("shada"));
        if self.cfg.recovery == Recovery::Mirror {
            cmd.arg("-n");
        }
        for var in ["VIMINIT", "EXINIT", "MYVIMRC", "VIMRUNTIME", "VIM", "NVIM", "NVIM_APPNAME", "NVIM_LISTEN_ADDRESS"] {
            cmd.env_remove(var);
        }
        for (var, sub) in [
            ("XDG_CONFIG_HOME", "config"),
            ("XDG_DATA_HOME", "data"),
            ("XDG_STATE_HOME", "state"),
            ("XDG_CACHE_HOME", "cache"),
        ] {
            let dir = xdg.join(sub);
            let _ = std::fs::create_dir_all(&dir);
            cmd.env(var, dir);
        }
        cmd.env("KINEIN_PROVA_ESTADO", &self.cfg.state)
            .env("NVIM_LOG_FILE", self.cfg.state.join("nvim.log"))
            .env("KINEIN_PROVA_RECUPERACAO", if self.cfg.recovery == Recovery::Swap { "swap" } else { "espelho" })
            .env("KINEIN_PROVA_GRAVACAO", if self.cfg.core_writes { "core" } else { "nvim" })
            .current_dir(&self.cfg.project)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped());
        let stderr = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.cfg.state.join("nvim-stderr.log"))
            .map_err(|e| e.to_string())?;
        cmd.stderr(stderr);
        // Os descritores da lista ficam abertos até o spawn: o filho os usa
        // entre o fork e o exec.
        let allow;
        if self.cfg.plant_no_landlock {
            cmd.env("KINEIN_PROVA_PLANTIO_SEM_LANDLOCK", "1");
            sandbox::supervised(&mut cmd);
        } else {
            allow = sandbox::ExecAllowList::for_binary(&self.cfg.nvim).map_err(|e| e.to_string())?;
            self.record(json!({"evento": "camada1", "executaveis": allow.paths}));
            sandbox::supervised_without_exec(&mut cmd, &allow);
        }
        let mut child = cmd.spawn().map_err(|e| format!("nvim não subiu: {e}"))?;
        let stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();
        let rpc = Rpc::new(Box::new(stdin));
        let proc = Arc::new(NvimProc { pid: child.id(), rpc: rpc.clone(), socket, generation });
        let sup = self.clone();
        std::thread::Builder::new()
            .name(format!("nvim-{generation}"))
            .spawn(move || {
                let r = rpc.clone();
                rpc.read_loop(stdout, |msg| sup.on_incoming(&r, generation, msg));
            })
            .map_err(|e| e.to_string())?;
        Ok((child, proc))
    }

    fn wait_ready(&self, generation: u64, timeout: Duration) -> Option<Value> {
        let deadline = Instant::now() + timeout;
        let mut ready = self.ready.lock().unwrap();
        loop {
            if let Some((g, c)) = ready.as_ref() {
                if *g == generation {
                    return Some(c.clone());
                }
            }
            let now = Instant::now();
            if now >= deadline {
                return None;
            }
            ready = self.ready_cv.wait_timeout(ready, deadline - now).unwrap().0;
        }
    }

    fn probe(&self, p: &NvimProc) -> Value {
        p.rpc
            .request(
                "nvim_exec_lua",
                vec!["return require('kinein').exec_probe()".into(), Mp::Array(vec![])],
                REQUEST_TIMEOUT,
            )
            .map(|v| rpc::to_json(&v))
            .unwrap_or_else(|e| json!({"erro": e}))
    }

    fn probe_contained(v: &Value) -> bool {
        ["true", "ldso", "nvim"].iter().all(|k| v[k].as_str().is_some_and(|s| s.starts_with("EACCES")))
            && v["system"].as_str() != Some("v:shell_error=0")
            && v["containment"]["layer2"] == json!(true)
    }

    /// Reabre os documentos depois de um nvim novo: o texto do espelho (ou do
    /// rascunho, na primeira vez) volta com o buffer modificado.
    fn restore(&self, first_start: bool) {
        let mut todo: Vec<(PathBuf, Option<Vec<String>>)> = Vec::new();
        if first_start {
            for (path, lines) in self.drafts.load_all() {
                if path.exists() && lines != mirror::read_lines(&path) {
                    todo.push((path, Some(lines)));
                }
            }
        } else {
            let mut mirror = self.mirror.lock().unwrap();
            mirror.forget_buffers();
            let order = self.order.lock().unwrap().clone();
            for path in order {
                if let Some(doc) = mirror.docs.get(&path) {
                    todo.push((path.clone(), doc.dirty().then(|| doc.lines.clone())));
                }
            }
        }
        for (path, lines) in todo {
            let path_mp: Mp = path.to_string_lossy().as_ref().into();
            let restored = match (lines, self.cfg.recovery) {
                (Some(_), Recovery::Swap) => self.exec_lua("return require('kinein').recover(...)", vec![path_mp]),
                (Some(lines), Recovery::Mirror) => self.exec_lua(
                    "return require('kinein').open(...)",
                    vec![path_mp, Mp::Array(lines.into_iter().map(Mp::from).collect())],
                ),
                (None, _) => self.exec_lua("return require('kinein').open(...)", vec![path_mp]),
            };
            let restored = match restored {
                Ok(v) => json!({"ok": v}),
                Err(e) => json!({"erro": e}),
            };
            self.record(json!({"evento": "reaberto", "path": path, "resultado": restored}));
        }
    }

    fn lifecycle(self: &Arc<Self>) {
        let mut generation = 0;
        let mut quick_exits = 0;
        while !self.stop.load(Ordering::Acquire) {
            generation += 1;
            let t0 = Instant::now();
            if !self.cfg.trusted && !self.cfg.plant_no_landlock {
                if let Err(e) = sandbox::landlock_abi() {
                    // §18, item 4: sem Landlock, nada de editor novo para projeto
                    // não confiável.
                    self.fail_closed(json!({"landlock": e}));
                    return;
                }
            }
            let (mut child, proc) = match self.spawn_nvim(generation) {
                Ok(v) => v,
                Err(e) => {
                    self.emit(json!({"evento": "nvim_falhou", "erro": e}));
                    self.record(json!({"evento": "nvim_falhou", "erro": e}));
                    return;
                }
            };
            let pid = proc.pid;
            let Some(containment) = self.wait_ready(generation, Duration::from_secs(15)) else {
                let _ = child.kill();
                let _ = child.wait();
                self.fail_closed(json!({"erro": "o nvim não ficou pronto em 15 s"}));
                return;
            };
            let ready_ms = t0.elapsed().as_secs_f64() * 1000.0;
            let probe = self.probe(&proc);
            *self.probe.lock().unwrap() = probe.clone();
            let probe_ms = t0.elapsed().as_secs_f64() * 1000.0 - ready_ms;
            if !Self::probe_contained(&probe) && !self.cfg.plant_no_probe {
                let _ = child.kill();
                let _ = child.wait();
                self.fail_closed(json!({"contencao": containment, "sonda": probe}));
                return;
            }
            *self.current.lock().unwrap() = Some(proc.clone());
            self.restore(generation == 1);
            self.emit(json!({"evento": "nvim_pronto", "socket": proc.socket, "geracao": generation, "pid": pid}));
            self.record(json!({
                "evento": "nvim_pronto", "geracao": generation, "pid": pid,
                "pronto_ms": ready_ms, "sonda_ms": probe_ms, "contencao": containment,
            }));
            let status = child.wait();
            self.protocol_errors.fetch_add(proc.rpc.protocol_errors.load(Ordering::Relaxed), Ordering::Relaxed);
            *self.current.lock().unwrap() = None;
            let _ = std::fs::remove_file(&proc.socket);
            let lived = t0.elapsed();
            let status = status.map(|s| s.to_string()).unwrap_or_else(|e| e.to_string());
            self.emit(json!({"evento": "nvim_saiu", "geracao": generation, "status": status}));
            self.record(json!({"evento": "nvim_saiu", "geracao": generation, "status": status}));
            if lived < Duration::from_secs(2) {
                quick_exits += 1;
                if quick_exits >= 3 {
                    self.record(json!({"evento": "desistiu", "motivo": "três saídas seguidas em menos de 2 s"}));
                    return;
                }
            } else {
                quick_exits = 0;
            }
        }
    }

    fn fail_closed(&self, detail: Value) {
        self.emit(json!({"evento": "contencao_falhou", "detalhe": detail}));
        self.record(json!({"evento": "contencao_falhou", "detalhe": detail}));
    }

    fn shutdown(&self) {
        if self.stop.swap(true, Ordering::AcqRel) {
            return;
        }
        self.drafts.flush(&mut self.mirror.lock().unwrap());
        self.lsp.shutdown();
        if let Some(p) = self.current.lock().unwrap().clone() {
            // SAFETY: sinal para um pid que este processo criou.
            unsafe { libc::kill(p.pid as i32, libc::SIGTERM) };
        }
    }

    // --- o canal de controle do driver ------------------------------------

    fn handle(self: &Arc<Self>, req: &Value) -> Result<Value, String> {
        let path = || PathBuf::from(req["path"].as_str().unwrap_or_default());
        let path_mp = || -> Mp { path().to_string_lossy().as_ref().into() };
        match req["op"].as_str().unwrap_or_default() {
            "estado" => Ok(self.status()),
            "abrir" => self.exec_lua("return require('kinein').open(...)", vec![path_mp()]),
            "texto" => self.exec_lua("return require('kinein').buffer_text(...)", vec![path_mp()]),
            "espelho" => {
                let mirror = self.mirror.lock().unwrap();
                let doc = mirror.docs.get(&path()).ok_or("documento fora do espelho")?;
                Ok(json!({"lines": doc.lines, "changedtick": doc.changedtick, "sujo": doc.dirty()}))
            }
            "aplicar" => self.exec_lua(
                "local path, tick, first, last, lines = ...\n\
                 local buf = vim.fn.bufnr(path)\n\
                 return require('kinein').apply(buf, tick, first, last, lines)",
                vec![
                    path_mp(),
                    rpc::from_json(&req["tick"]),
                    rpc::from_json(&req["first"]),
                    rpc::from_json(&req["last"]),
                    rpc::from_json(&req["lines"]),
                ],
            ),
            "lua" => {
                let args = req["args"].as_array().map(|a| a.iter().map(rpc::from_json).collect()).unwrap_or_default();
                self.exec_lua(req["code"].as_str().unwrap_or_default(), args)
            }
            "matar_nvim" => {
                let p = self.nvim()?;
                // SAFETY: sinal para um pid que este processo criou.
                unsafe { libc::kill(p.pid as i32, libc::SIGKILL) };
                Ok(json!({"pid": p.pid, "geracao": p.generation}))
            }
            "contexto_cpp" => {
                {
                    let mut ctx = self.lsp.context.lock().unwrap();
                    ctx.compile_commands_dir = req["compile_commands_dir"].as_str().map(PathBuf::from);
                    ctx.query_driver = req["query_driver"].as_str().map(str::to_owned);
                }
                // O Neovim para o cliente e reconecta cada buffer dele; o relé
                // vê a conexão velha fechar, o clangd velho sai, e a conexão
                // nova lança o clangd com o contexto novo. Matar o servidor
                // antes faria o cliente sair sem deixar o que reconectar.
                self.exec_lua("return require('kinein').lsp_restart('clangd')", vec![])
            }
            "diagnosticos" => Ok(json!(*self.lsp.diagnostics.lock().unwrap())),
            "build" => self.build(req),
            "sonda" => {
                let p = self.nvim()?;
                Ok(self.probe(&p))
            }
            "encerrar" => {
                self.shutdown();
                Ok(json!(true))
            }
            other => Err(format!("operação desconhecida: {other}")),
        }
    }

    /// Compilar pelo lançador, como o core faz: só com confiança.
    fn build(&self, req: &Value) -> Result<Value, String> {
        if !self.lsp.trusted() {
            return Err("projeto sem confiança: nada do projeto é executado".into());
        }
        let argv: Vec<String> =
            req["cmd"].as_array().ok_or("cmd ausente")?.iter().filter_map(|v| v.as_str().map(str::to_owned)).collect();
        let (program, args) = argv.split_first().ok_or("cmd vazio")?;
        let cwd = req["cwd"].as_str().map(PathBuf::from).unwrap_or_else(|| self.cfg.project.clone());
        let mut cmd = Command::new(program);
        cmd.args(args).current_dir(cwd).stdin(Stdio::null());
        sandbox::supervised(&mut cmd);
        let t0 = Instant::now();
        let out = cmd.output().map_err(|e| e.to_string())?;
        Ok(json!({
            "status": out.status.code(),
            "ms": t0.elapsed().as_secs_f64() * 1000.0,
            "stdout": tail(&out.stdout),
            "stderr": tail(&out.stderr),
        }))
    }

    fn status(&self) -> Value {
        let current = self.current.lock().unwrap().clone();
        let mirror = self.mirror.lock().unwrap();
        let docs: Vec<Value> = mirror
            .docs
            .values()
            .map(|d| {
                json!({"path": d.path, "buf": d.buf, "changedtick": d.changedtick, "sujo": d.dirty(), "linhas": d.lines.len(), "eventos": d.events})
            })
            .collect();
        let live_errors = current.as_ref().map(|p| p.rpc.protocol_errors.load(Ordering::Relaxed)).unwrap_or(0);
        json!({
            "geracao": current.as_ref().map(|p| p.generation),
            "pid": current.as_ref().map(|p| p.pid),
            "socket": current.as_ref().map(|p| p.socket.clone()),
            "landlock_abi": sandbox::landlock_abi().ok(),
            "sonda": *self.probe.lock().unwrap(),
            "espelho": docs,
            "rascunhos_em_disco": self.drafts.count(),
            "gravacoes_de_rascunho": *self.drafts.writes.lock().unwrap(),
            "erros_de_protocolo": self.protocol_errors.load(Ordering::Relaxed) + live_errors,
            "lsp": self.lsp.status(),
            "eventos": *self.log.lock().unwrap(),
        })
    }
}

fn serve_control(sup: Arc<Sup>, listener: UnixListener) {
    for stream in listener.incoming().flatten() {
        let sup = sup.clone();
        std::thread::spawn(move || {
            let Ok(mut writer) = stream.try_clone() else { return };
            let reader = BufReader::new(stream);
            for line in reader.lines() {
                let Ok(line) = line else { break };
                let reply = match serde_json::from_str::<Value>(&line) {
                    Ok(req) => match sup.handle(&req) {
                        Ok(v) => json!({"id": req["id"], "ok": true, "resultado": v}),
                        Err(e) => json!({"id": req["id"], "ok": false, "erro": e}),
                    },
                    Err(e) => json!({"ok": false, "erro": e.to_string()}),
                };
                if writeln!(writer, "{reply}").is_err() {
                    break;
                }
            }
        });
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let need = |name: &str| -> PathBuf {
        PathBuf::from(arg(&args, name).unwrap_or_else(|| {
            eprintln!(
                "uso: kinein-prova-supervisor --projeto D --estado D --run D --runtime D --nvim B [--nao-confiavel]"
            );
            std::process::exit(2);
        }))
    };
    let cfg = Config {
        project: need("--projeto").canonicalize().expect("--projeto"),
        state: need("--estado"),
        run: need("--run"),
        runtime: need("--runtime"),
        nvim: need("--nvim"),
        trusted: !args.iter().any(|a| a == "--nao-confiavel"),
        recovery: if std::env::var("KINEIN_PROVA_RECUPERACAO").is_ok_and(|v| v == "swap") {
            Recovery::Swap
        } else {
            Recovery::Mirror
        },
        core_writes: std::env::var("KINEIN_PROVA_GRAVACAO").is_ok_and(|v| v == "core"),
        plant_no_landlock: env_flag("KINEIN_PROVA_PLANTIO_SEM_LANDLOCK"),
        plant_no_probe: env_flag("KINEIN_PROVA_PLANTIO_SEM_SONDA"),
    };
    // O caminho de um socket Unix cabe em 108 bytes (sun_path, com o NUL).
    // Uma pasta `run` funda demais faria o bind do relé falhar no meio da
    // sessão; a partida recusa antes.
    let longest = cfg.run.join(lsp::LONGEST_SOCKET_NAME);
    if longest.as_os_str().len() > 107 {
        eprintln!("supervisor: --run fundo demais para sockets Unix ({} bytes): {}", longest.as_os_str().len(), longest.display());
        std::process::exit(2);
    }
    for dir in [&cfg.state, &cfg.run] {
        std::fs::create_dir_all(dir).expect("pasta");
    }
    std::fs::set_permissions(&cfg.run, std::fs::Permissions::from_mode(0o700)).expect("permissão do run");
    let logs = cfg.state.join("logs");
    let _ = std::fs::create_dir_all(&logs);
    let drafts = Drafts::new(cfg.state.join("rascunhos"), !env_flag("KINEIN_PROVA_PLANTIO_SEM_RASCUNHO"));
    let lsp = lsp::Lsp::new(cfg.run.clone(), logs, cfg.project.clone(), cfg.trusted);
    let control_path = cfg.run.join("controle.sock");
    let _ = std::fs::remove_file(&control_path);
    let control = UnixListener::bind(&control_path).expect("socket de controle");
    let (jobs_tx, jobs_rx) = mpsc::channel::<Job>();
    let sup = Arc::new(Sup {
        cfg,
        current: Mutex::new(None),
        mirror: Mutex::new(Mirror::default()),
        drafts,
        lsp,
        out: Mutex::new(std::io::stdout()),
        stop: AtomicBool::new(false),
        ready: Mutex::new(None),
        ready_cv: Condvar::new(),
        probe: Mutex::new(Value::Null),
        log: Mutex::new(Vec::new()),
        jobs: Mutex::new(jobs_tx),
        protocol_errors: AtomicU64::new(0),
        started: Instant::now(),
        order: Mutex::new(Vec::new()),
    });
    sup.emit(json!({"evento": "supervisor_pronto", "controle": control_path, "pid": std::process::id()}));

    let s = sup.clone();
    std::thread::spawn(move || s.drafts.run(&s.mirror, &s.stop));
    std::thread::spawn(move || {
        for job in jobs_rx {
            match job {
                Job::Attach(rpc, buf) => {
                    let _ = rpc.request("nvim_buf_attach", vec![buf.into(), true.into(), Mp::Map(vec![])], REQUEST_TIMEOUT);
                }
            }
        }
    });
    let s = sup.clone();
    std::thread::spawn(move || serve_control(s, control));
    // A UI é dona do supervisor: o stdin fechado é a UI que saiu (ou caiu).
    let s = sup.clone();
    std::thread::spawn(move || {
        let mut line = String::new();
        let stdin = std::io::stdin();
        while stdin.lock().read_line(&mut line).is_ok_and(|n| n > 0) {
            if line.trim() == "encerrar" {
                break;
            }
            line.clear();
        }
        s.shutdown();
    });

    sup.lifecycle();
    // Sem editor (contenção falhou ou desistiu), o supervisor segue de pé para
    // o driver ler o estado, até a UI sair.
    while !sup.stop.load(Ordering::Acquire) {
        std::thread::sleep(Duration::from_millis(100));
    }
    sup.shutdown();
    // O nvim sai pelo SIGTERM; o PDEATHSIG cobre o resto.
    std::thread::sleep(Duration::from_millis(300));
    std::process::exit(0);
}

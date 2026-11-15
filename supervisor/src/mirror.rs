//! O espelho do buffer e o rascunho.
//!
//! O core recebe as mudanças pelo `nvim_buf_attach`, fora do caminho da
//! tecla, e guarda o texto de cada documento aberto. O texto fica num segundo
//! processo: se o `nvim` cai, o que já chegou ao espelho não se perde. O
//! rascunho em disco cobre a queda do próprio core; ele sai no máximo
//! `DRAFT_DELAY` depois da mudança.

use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Condvar, Mutex};
use std::time::{Duration, Instant};

pub const DRAFT_DELAY: Duration = Duration::from_millis(50);

pub struct Doc {
    pub path: PathBuf,
    pub buf: i64,
    pub lines: Vec<String>,
    pub changedtick: i64,
    pub disk: Vec<String>,
    pub draft_pending: bool,
    pub events: u64,
}

impl Doc {
    pub fn dirty(&self) -> bool {
        self.lines != self.disk
    }
}

#[derive(Default)]
pub struct Mirror {
    pub docs: HashMap<PathBuf, Doc>,
    by_buf: HashMap<i64, PathBuf>,
}

pub fn read_lines(path: &Path) -> Vec<String> {
    let text = fs::read_to_string(path).unwrap_or_default();
    split_lines(&text)
}

pub fn split_lines(text: &str) -> Vec<String> {
    let body = text.strip_suffix('\n').unwrap_or(text);
    if text.is_empty() {
        return vec![String::new()];
    }
    body.split('\n').map(str::to_owned).collect()
}

impl Mirror {
    pub fn open(&mut self, buf: i64, path: PathBuf) {
        self.by_buf.insert(buf, path.clone());
        let disk = read_lines(&path);
        let doc = self.docs.entry(path.clone()).or_insert_with(|| Doc {
            path: path.clone(),
            buf,
            lines: disk.clone(),
            changedtick: 0,
            disk: disk.clone(),
            draft_pending: false,
            events: 0,
        });
        doc.buf = buf;
    }

    /// Um `nvim_buf_lines_event`: troca as linhas [first, last) por `data`.
    /// `last == -1` é o buffer inteiro (o envio inicial).
    pub fn apply(&mut self, buf: i64, changedtick: i64, first: i64, last: i64, data: Vec<String>) -> bool {
        let Some(path) = self.by_buf.get(&buf) else { return false };
        let Some(doc) = self.docs.get_mut(path) else { return false };
        let len = doc.lines.len() as i64;
        let first = first.clamp(0, len) as usize;
        let last = if last < 0 { len as usize } else { last.clamp(first as i64, len) as usize };
        doc.lines.splice(first..last, data);
        doc.changedtick = changedtick;
        doc.draft_pending = true;
        doc.events += 1;
        true
    }

    pub fn tick(&mut self, buf: i64, changedtick: i64) {
        if let Some(doc) = self.by_buf.get(&buf).and_then(|p| self.docs.get_mut(p)) {
            doc.changedtick = changedtick;
        }
    }

    pub fn written(&mut self, path: &Path) {
        if let Some(doc) = self.docs.get_mut(path) {
            doc.disk = read_lines(path);
            doc.draft_pending = true;
        }
    }

    pub fn forget_buffers(&mut self) {
        self.by_buf.clear();
    }
}

pub struct Drafts {
    dir: PathBuf,
    pub enabled: bool,
    wake: Condvar,
    pub writes: Mutex<u64>,
}

fn draft_name(path: &Path) -> String {
    // FNV-1a do caminho: nome estável, sem dependência nova.
    let mut h: u64 = 0xcbf29ce484222325;
    for b in path.to_string_lossy().bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    format!("{h:016x}.json")
}

impl Drafts {
    pub fn new(dir: PathBuf, enabled: bool) -> Self {
        let _ = fs::create_dir_all(&dir);
        Self { dir, enabled, wake: Condvar::new(), writes: Mutex::new(0) }
    }

    pub fn poke(&self) {
        self.wake.notify_all();
    }

    /// Grava os rascunhos pendentes; o texto igual ao do disco apaga o seu.
    pub fn flush(&self, mirror: &mut Mirror) {
        if !self.enabled {
            // O defeito plantado do R4: o espelho segue, o disco não recebe nada.
            mirror.docs.values_mut().for_each(|d| d.draft_pending = false);
            return;
        }
        for doc in mirror.docs.values_mut().filter(|d| d.draft_pending) {
            doc.draft_pending = false;
            let file = self.dir.join(draft_name(&doc.path));
            if !doc.dirty() {
                let _ = fs::remove_file(&file);
                continue;
            }
            let body = json!({
                "path": doc.path,
                "changedtick": doc.changedtick,
                "lines": doc.lines,
            });
            let tmp = file.with_extension("tmp");
            let ok = fs::File::create(&tmp)
                .and_then(|mut f| f.write_all(body.to_string().as_bytes()))
                .and_then(|_| fs::rename(&tmp, &file));
            if ok.is_ok() {
                *self.writes.lock().unwrap() += 1;
            }
        }
    }

    /// A thread que grava: acorda numa mudança, espera `DRAFT_DELAY` para
    /// juntar as teclas seguintes e grava.
    pub fn run(&self, mirror: &Mutex<Mirror>, stop: &std::sync::atomic::AtomicBool) {
        let mut guard = mirror.lock().unwrap();
        loop {
            if stop.load(std::sync::atomic::Ordering::Acquire) {
                return;
            }
            if guard.docs.values().any(|d| d.draft_pending) {
                drop(guard);
                let until = Instant::now() + DRAFT_DELAY;
                while Instant::now() < until {
                    std::thread::sleep(until - Instant::now());
                }
                guard = mirror.lock().unwrap();
                self.flush(&mut guard);
                continue;
            }
            guard = self.wake.wait_timeout(guard, Duration::from_millis(500)).unwrap().0;
        }
    }

    pub fn load_all(&self) -> Vec<(PathBuf, Vec<String>)> {
        let mut out = Vec::new();
        let Ok(entries) = fs::read_dir(&self.dir) else { return out };
        for e in entries.flatten() {
            if e.path().extension().is_some_and(|x| x == "json") {
                if let Ok(text) = fs::read_to_string(e.path()) {
                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
                        let path = PathBuf::from(v["path"].as_str().unwrap_or_default());
                        let lines = v["lines"]
                            .as_array()
                            .map(|a| a.iter().map(|l| l.as_str().unwrap_or_default().to_owned()).collect())
                            .unwrap_or_default();
                        out.push((path, lines));
                    }
                }
            }
        }
        out
    }

    pub fn count(&self) -> usize {
        fs::read_dir(&self.dir)
            .map(|d| d.flatten().filter(|e| e.path().extension().is_some_and(|x| x == "json")).count())
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_events_are_applied_in_order() {
        let dir = std::env::temp_dir().join(format!("kinein-mirror-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let f = dir.join("a.txt");
        fs::write(&f, "um\ndois\ntrês\n").unwrap();
        let mut m = Mirror::default();
        m.open(1, f.clone());
        m.apply(1, 2, 0, -1, vec!["um".into(), "dois".into(), "três".into()]);
        m.apply(1, 3, 1, 2, vec!["DOIS".into(), "meio".into()]);
        m.apply(1, 4, 3, 4, vec![]);
        let doc = &m.docs[&f];
        assert_eq!(doc.lines, vec!["um", "DOIS", "meio"]);
        assert_eq!(doc.changedtick, 4);
        assert!(doc.dirty());
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn split_matches_the_buffer_view_of_a_file() {
        assert_eq!(split_lines("a\nb\n"), vec!["a", "b"]);
        assert_eq!(split_lines("a\nb"), vec!["a", "b"]);
        assert_eq!(split_lines(""), vec![""]);
    }
}

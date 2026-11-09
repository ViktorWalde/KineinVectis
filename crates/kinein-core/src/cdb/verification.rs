//! Content snapshots and include-search identity, revalidated before apply.
#![allow(
    clippy::redundant_pub_crate,
    reason = "Keep shared verifier details internal to the core"
)]

use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    io::Read,
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant, SystemTime},
};

const MAX_CONTEXT_BYTES: usize = 64 * 1_048_576;

#[derive(Debug, Clone, Eq, PartialEq)]
struct Stamp {
    canonical: PathBuf,
    len: u64,
    modified: SystemTime,
    digest: String,
}

fn read(path: &Path, deadline: Instant) -> Option<(Stamp, Vec<u8>)> {
    let mut file = std::fs::File::open(path).ok()?;
    let metadata = file.metadata().ok()?;
    if !metadata.is_file() || metadata.len() > MAX_CONTEXT_BYTES as u64 {
        return None;
    }
    let mut bytes = Vec::new();
    let mut buffer = [0_u8; 16384];
    loop {
        if Instant::now() >= deadline {
            return None;
        }
        let count = file.read(&mut buffer).ok()?;
        if count == 0 {
            break;
        }
        if bytes.len().checked_add(count)? > MAX_CONTEXT_BYTES {
            return None;
        }
        bytes.extend_from_slice(&buffer[..count]);
    }
    let after = file.metadata().ok()?;
    if metadata.len() != after.len() || metadata.modified().ok() != after.modified().ok() {
        return None;
    }
    let stamp = Stamp {
        canonical: crate::platform::canonicalize(path).ok()?,
        len: metadata.len(),
        modified: metadata.modified().ok()?,
        digest: digest(&bytes),
    };
    Some((stamp, bytes))
}

pub(crate) fn cdb_bytes(path: &Path) -> Result<Vec<u8>, String> {
    let file = std::fs::File::open(path).map_err(|e| format!("CDB indisponível: {e}"))?;
    let mut bytes = Vec::new();
    file.take(16 * 1_048_576 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > 16 * 1_048_576 {
        return Err("CDB excede 16 MiB.".into());
    }
    Ok(bytes)
}

pub(crate) fn digest(bytes: &[u8]) -> String {
    hex(Sha256::digest(bytes).as_ref())
}
pub(crate) fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut result = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(result, "{byte:02x}");
    }
    result
}

#[derive(Debug, Clone)]
struct Observation {
    stamp: Stamp,
    content: Option<Arc<[u8]>>,
}

#[derive(Debug, Clone)]
pub(crate) struct ContextGuard {
    root: PathBuf,
    cdb: PathBuf,
    digest: String,
    observed: BTreeMap<PathBuf, Observation>,
    directories: BTreeMap<PathBuf, String>,
    deadline: Instant,
    valid: bool,
    bytes: usize,
}

impl ContextGuard {
    pub(crate) fn new(root: &Path, cdb: PathBuf, bytes: &[u8], deadline: Instant) -> Self {
        Self {
            root: root.into(),
            cdb,
            digest: digest(bytes),
            observed: BTreeMap::new(),
            directories: BTreeMap::new(),
            deadline,
            valid: true,
            bytes: 0,
        }
    }
    pub(crate) fn observe(&mut self, path: &Path) -> Result<(), String> {
        self.record(path, false)
    }
    pub(crate) fn observe_input(&mut self, path: &Path) -> Result<(), String> {
        self.record(path, true)
    }
    fn record(&mut self, path: &Path, freeze: bool) -> Result<(), String> {
        if self.observed.len() >= 4096 && !self.observed.contains_key(path) {
            return Err("Limite de arquivos observados excedido.".into());
        }
        let (stamp, bytes) = read(path, self.deadline)
            .ok_or("Dependência não pôde ser capturada dentro dos limites.")?;
        if self
            .observed
            .get(path)
            .is_some_and(|old| old.stamp != stamp)
        {
            self.valid = false;
            return Err("Contexto alterado durante a verificação.".into());
        }
        let frozen = self.observed.get(path).is_some_and(|o| o.content.is_some());
        if freeze && !frozen {
            self.bytes = self
                .bytes
                .checked_add(bytes.len())
                .ok_or("Contexto excede o limite de memória.")?;
            if self.bytes > MAX_CONTEXT_BYTES {
                return Err("Headers excedem o limite de 64 MiB.".into());
            }
            self.observed.insert(
                path.into(),
                Observation {
                    stamp,
                    content: Some(Arc::from(bytes)),
                },
            );
        } else if !self.observed.contains_key(path) {
            self.observed.insert(
                path.into(),
                Observation {
                    stamp,
                    content: None,
                },
            );
        }
        Ok(())
    }
    pub(crate) fn observe_directory(&mut self, path: &Path) -> Result<(), String> {
        if super::directory::covered(self.directories.keys(), path) {
            return Ok(());
        }
        let signature = super::directory::signature(path, self.deadline)
            .ok_or("Busca de includes excede os limites de captura.")?;
        self.directories.retain(|old, _| !old.starts_with(path));
        self.directories.insert(path.to_path_buf(), signature);
        Ok(())
    }
    pub(crate) fn frozen_files(&self) -> impl Iterator<Item = (&Path, &[u8], SystemTime)> {
        self.observed.iter().filter_map(|(p, o)| {
            o.content
                .as_deref()
                .map(|b| (p.as_path(), b, o.stamp.modified))
        })
    }
    pub(crate) fn modified(&self, path: &Path) -> Option<SystemTime> {
        self.observed.get(path).map(|o| o.stamp.modified)
    }
    pub(crate) fn frozen_count(&self) -> usize {
        self.observed
            .values()
            .filter(|o| o.content.is_some())
            .count()
    }
    pub(crate) fn is_current(&self) -> bool {
        self.is_current_before(Instant::now() + Duration::from_secs(2))
    }
    pub(crate) fn is_current_before(&self, deadline: Instant) -> bool {
        let status = crate::cdb::status(&self.root);
        let selected = status.file(&self.root);
        self.valid
            && !status.stale
            && selected.as_ref() == Some(&self.cdb)
            && cdb_bytes(&self.cdb).is_ok_and(|b| digest(&b) == self.digest)
            && self
                .observed
                .iter()
                .all(|(p, o)| read(p, deadline).is_some_and(|(s, _)| s == o.stamp))
            && self
                .directories
                .iter()
                .all(|(p, s)| super::directory::signature(p, deadline).as_ref() == Some(s))
    }
}

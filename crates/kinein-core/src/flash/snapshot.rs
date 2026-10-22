//! Bounded content snapshots and cheap change checks; no device is opened here.
use sha2::{Digest, Sha256};
use std::fmt::Write as _;
use std::{
    io::Read,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

pub(super) const MAX_IMAGE_BYTES: u64 = 1024 * 1024 * 1024;

#[derive(Debug)]
pub(super) struct Observation {
    path: PathBuf,
    revision: String,
}

impl Observation {
    pub(super) fn unchanged(&self) -> bool {
        crate::platform::file_revision(&self.path).is_ok_and(|revision| revision == self.revision)
    }
}

pub(super) fn digest(bytes: &[u8]) -> String {
    hex(&Sha256::digest(bytes))
}

fn hex(bytes: &[u8]) -> String {
    let mut result = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(&mut result, "{byte:02x}");
    }
    result
}

pub(super) fn file(
    path: &Path,
    expected: Option<&str>,
    cancel: &Arc<AtomicBool>,
) -> Result<(String, Observation), String> {
    let revision = crate::platform::file_revision(path)
        .map_err(|error| format!("{}: {error}", path.display()))?;
    if expected.is_some_and(|expected| expected != revision) {
        return Err("um arquivo mudou desde a leitura dos metadados; gere outra prévia".to_owned());
    }
    let resolved = crate::platform::canonicalize(path).map_err(|error| error.to_string())?;
    let mut input =
        crate::fsops::open_regular_file(&resolved).map_err(|error| error.to_string())?;
    let metadata = input.metadata().map_err(|error| error.to_string())?;
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > MAX_IMAGE_BYTES {
        return Err(format!(
            "{} não é um arquivo regular não vazio de até 1 GiB",
            path.display()
        ));
    }
    let mut hash = Sha256::new();
    let mut buffer = [0_u8; 8192];
    let mut total = 0_u64;
    loop {
        if cancel.load(Ordering::SeqCst) {
            return Err("preparação de gravação cancelada".to_owned());
        }
        let count = input.read(&mut buffer).map_err(|error| error.to_string())?;
        if count == 0 {
            break;
        }
        total += u64::try_from(count).unwrap_or(u64::MAX);
        if total > MAX_IMAGE_BYTES {
            return Err("arquivo cresceu além de 1 GiB durante a leitura".to_owned());
        }
        hash.update(&buffer[..count]);
    }
    let observation = Observation {
        path: path.to_path_buf(),
        revision,
    };
    if !observation.unchanged() {
        return Err("arquivo mudou durante a leitura; gere outra prévia".to_owned());
    }
    Ok((hex(&hash.finalize()), observation))
}

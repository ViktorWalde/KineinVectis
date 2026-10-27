//! Regular bounded artifact bytes and their observed revision, shared by native USB engines.
use std::{io::Read, path::Path};

pub(super) fn read_bounded(path: &Path, limit: usize) -> Result<(Vec<u8>, String), String> {
    let revision = crate::platform::file_revision(path).map_err(|error| error.to_string())?;
    let mut bytes = Vec::new();
    let file = crate::fsops::open_regular_file(path).map_err(|error| error.to_string())?;
    let length = file.metadata().map_err(|error| error.to_string())?.len();
    if length == 0 || length > limit as u64 {
        return Err(format!(
            "{} precisa ser regular, não vazio e de até {limit} bytes",
            path.display()
        ));
    }
    file.take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    if bytes.is_empty() || bytes.len() > limit {
        return Err(format!(
            "{} precisa ser regular, não vazio e de até {limit} bytes",
            path.display()
        ));
    }
    if crate::platform::file_revision(path).map_err(|error| error.to_string())? != revision {
        return Err("arquivo mudou durante a leitura; gere outra prévia".into());
    }
    Ok((bytes, revision))
}

//! Private immutable compiler inputs, preserving their original filesystem names.

use crate::cdb::verification::{ContextGuard, digest, hex};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

pub(super) struct Scratch(PathBuf);

impl Scratch {
    pub(super) fn dependency_file(&self) -> PathBuf {
        self.0.join("inputs.d")
    }
    pub(super) fn new() -> Result<Self, String> {
        let cache = std::env::var_os("XDG_CACHE_HOME")
            .map(PathBuf::from)
            .or_else(|| crate::platform::home_dir().map(|p| p.join(".cache")))
            .ok_or("Cache em disco indisponível.")?;
        let mut random = [0_u8; 16];
        getrandom::fill(&mut random).map_err(|e| e.to_string())?;
        let path = cache
            .join("kinein-vectis/edit-verification")
            .join(hex(&random));
        crate::platform::create_private_directory(&path)
            .map_err(|e| format!("Área temporária indisponível: {e}"))?;
        Ok(Self(path))
    }

    pub(super) fn mappings(
        &self,
        path: &Path,
        source: &Path,
        content: &str,
        guard: &ContextGuard,
    ) -> Result<Vec<(PathBuf, PathBuf)>, String> {
        let mut result = BTreeMap::new();
        for (input, bytes, modified) in guard.frozen_files() {
            let real = crate::platform::canonicalize(input).map_err(|e| e.to_string())?;
            if real == path {
                continue;
            }
            let name = digest(real.to_string_lossy().as_bytes());
            let physical = self.0.join(name);
            if !physical.exists() {
                std::fs::write(&physical, bytes).map_err(|e| e.to_string())?;
                std::fs::File::options()
                    .write(true)
                    .open(&physical)
                    .and_then(|f| f.set_modified(modified))
                    .map_err(|e| e.to_string())?;
            }
            result.insert(input.to_path_buf(), physical.clone());
            result.insert(real, physical);
        }
        let physical = self.0.join("main");
        std::fs::write(&physical, content).map_err(|e| e.to_string())?;
        let modified = guard
            .modified(path)
            .ok_or("Data original do documento indisponível.")?;
        std::fs::File::options()
            .write(true)
            .open(&physical)
            .and_then(|f| f.set_modified(modified))
            .map_err(|e| e.to_string())?;
        result.insert(path.into(), physical.clone());
        result.insert(source.into(), physical);
        Ok(result.into_iter().collect())
    }

    pub(super) fn overlay(&self, mappings: &[(PathBuf, PathBuf)]) -> Result<PathBuf, String> {
        let roots=mappings.iter().map(|(virtual_path,physical)|serde_json::json!({"type":"file","name":virtual_path,"external-contents":physical})).collect::<Vec<_>>();
        let path = self.0.join("overlay.json");
        std::fs::write(
            &path,
            serde_json::json!({"version":0,"use-external-names":false,"roots":roots}).to_string(),
        )
        .map_err(|e| e.to_string())?;
        Ok(path)
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

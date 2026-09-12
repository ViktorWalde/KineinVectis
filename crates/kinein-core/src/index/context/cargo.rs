//! O leitor do `cargo metadata --no-deps`: os alvos com `src_path`, para
//! saber a que pacote/alvo cada arquivo Rust pertence.

use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

/// Um alvo do cargo com o que preciso para casar arquivos.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct AlvoCargo {
    pub(super) package: String,
    pub(super) target: String,
    pub(super) kind: String,
    pub(super) edition: String,
    pub(super) manifest: String,
    pub(super) src_path: PathBuf,
    pub(super) features: Vec<String>,
}

/// `cargo metadata --no-deps`: (pacotes, alvos com `src_path`).
pub(super) fn cargo_metadata(cargo: &Path, root: &Path) -> Option<(u64, Vec<AlvoCargo>)> {
    let saida = Command::new(cargo)
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .current_dir(root)
        .output()
        .ok()?;
    if !saida.status.success() {
        return None;
    }
    let texto = String::from_utf8_lossy(&saida.stdout);
    Some(parse_cargo_metadata(&texto))
}

/// Puro: o que `cargo_metadata` interpreta.
fn parse_cargo_metadata(texto: &str) -> (u64, Vec<AlvoCargo>) {
    let Ok(valor) = serde_json::from_str::<Value>(texto) else {
        return (0, Vec::new());
    };
    let mut alvos = Vec::new();
    let pacotes = valor.get("packages").and_then(Value::as_array);
    for pacote in pacotes.into_iter().flatten() {
        let nome = pacote
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_owned();
        let edition = pacote
            .get("edition")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_owned();
        let manifest = pacote
            .get("manifest_path")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_owned();
        let mut features: Vec<String> = pacote
            .get("features")
            .and_then(Value::as_object)
            .map(|m| m.keys().cloned().collect())
            .unwrap_or_default();
        features.sort();
        for alvo in pacote
            .get("targets")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let (Some(target), Some(src_path)) = (
                alvo.get("name").and_then(Value::as_str),
                alvo.get("src_path").and_then(Value::as_str),
            ) else {
                continue;
            };
            alvos.push(AlvoCargo {
                package: nome.clone(),
                target: target.to_owned(),
                kind: alvo
                    .pointer("/kind/0")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown")
                    .to_owned(),
                edition: edition.clone(),
                manifest: manifest.clone(),
                src_path: PathBuf::from(src_path),
                features: features.clone(),
            });
        }
    }
    (pacotes.map_or(0, |p| p.len() as u64), alvos)
}

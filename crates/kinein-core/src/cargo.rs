//! Servico Cargo: resumo do `cargo metadata` para a UI.
//!
//! O JSON completo do `cargo metadata` e grande e cheio de detalhe que a UI
//! nao consome; o core resume para pacotes do workspace (nome, versao,
//! features, targets) e a lista de membros. Sempre com `--no-deps`: leitura
//! local dos manifests, sem rede nem resolucao de registry.

use std::{path::Path, process::Command};

use kinein_protocol::{CargoMetadataResult, CargoPackageInfo, CargoTargetInfo, ToolchainRole};
use serde_json::Value;

/// Executa `cargo metadata --no-deps` no root e resume o resultado.
pub fn run_metadata(
    root: &Path,
    toolchain: &crate::toolchain::Toolchain,
) -> Result<CargoMetadataResult, String> {
    let programa = toolchain
        .program_for(ToolchainRole::Cargo)
        .unwrap_or_else(|| std::path::PathBuf::from("cargo"));
    let output = Command::new(programa)
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .current_dir(root)
        .output()
        .map_err(|error| format!("cargo nao pode ser executado: {error}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "cargo metadata falhou: {}",
            stderr.trim().lines().last().unwrap_or("erro desconhecido")
        ));
    }
    let body = String::from_utf8_lossy(&output.stdout);
    parse_metadata(&body)
}

/// Resume o JSON do `cargo metadata` no payload do protocolo.
pub fn parse_metadata(body: &str) -> Result<CargoMetadataResult, String> {
    let value: Value =
        serde_json::from_str(body).map_err(|error| format!("metadata invalido: {error}"))?;

    let mut packages = Vec::new();
    for package in value
        .get("packages")
        .and_then(Value::as_array)
        .unwrap_or(&Vec::new())
    {
        let Some(name) = package.get("name").and_then(Value::as_str) else {
            continue;
        };
        let version = package
            .get("version")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_owned();
        let mut features = package
            .get("features")
            .and_then(Value::as_object)
            .map(|map| map.keys().cloned().collect::<Vec<_>>())
            .unwrap_or_default();
        features.sort();
        let targets = package
            .get("targets")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(|target| {
                        let target_name = target.get("name").and_then(Value::as_str)?;
                        let kind = target
                            .pointer("/kind/0")
                            .and_then(Value::as_str)
                            .unwrap_or("unknown");
                        Some(CargoTargetInfo {
                            name: target_name.to_owned(),
                            kind: kind.to_owned(),
                        })
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        packages.push(CargoPackageInfo {
            name: name.to_owned(),
            version,
            features,
            targets,
        });
    }

    let workspace_members = packages
        .iter()
        .map(|package| package.name.clone())
        .collect();

    Ok(CargoMetadataResult {
        packages,
        workspace_members,
    })
}

#[cfg(test)]
mod tests {
    use super::parse_metadata;

    #[test]
    fn parse_summarizes_packages_features_and_targets() {
        let body = r#"{
            "packages": [
                {
                    "name": "demo",
                    "version": "0.1.0",
                    "features": { "extra": [], "default": [] },
                    "targets": [
                        { "name": "demo", "kind": ["bin"] },
                        { "name": "demo-lib", "kind": ["lib"] }
                    ]
                },
                {
                    "name": "util",
                    "version": "0.2.0",
                    "features": {},
                    "targets": [ { "name": "util", "kind": ["lib"] } ]
                }
            ],
            "workspace_members": ["demo 0.1.0", "util 0.2.0"]
        }"#;

        let summary = parse_metadata(body).unwrap();

        assert_eq!(summary.packages.len(), 2);
        assert_eq!(summary.packages[0].name, "demo");
        assert_eq!(summary.packages[0].features, vec!["default", "extra"]);
        assert_eq!(summary.packages[0].targets.len(), 2);
        assert_eq!(summary.packages[0].targets[0].kind, "bin");
        assert_eq!(summary.workspace_members, vec!["demo", "util"]);
    }

    #[test]
    fn parse_rejects_invalid_json_with_human_message() {
        let error = parse_metadata("nao é json").unwrap_err();
        assert!(error.contains("metadata invalido"));
    }
}

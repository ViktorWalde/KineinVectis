//! Cargo service payloads (`cargo.*`).

use serde::{Deserialize, Serialize};

/// One build target of a workspace package.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CargoTargetInfo {
    /// Target name (binary, library or test name).
    pub name: String,
    /// Primary target kind (`bin`, `lib`, `test`, `example`, ...).
    pub kind: String,
}

/// One workspace package summarized from `cargo metadata`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CargoPackageInfo {
    /// Package name.
    pub name: String,
    /// Package version.
    pub version: String,
    /// Declared feature names, sorted.
    pub features: Vec<String>,
    /// Build targets of the package.
    pub targets: Vec<CargoTargetInfo>,
}

/// Result payload for `cargo.metadata`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CargoMetadataResult {
    /// Workspace packages (no external dependencies).
    pub packages: Vec<CargoPackageInfo>,
    /// Names of the workspace member packages.
    pub workspace_members: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::{CargoMetadataResult, CargoPackageInfo, CargoTargetInfo};

    #[test]
    fn metadata_result_serializes_camel_case() {
        let value = serde_json::to_value(CargoMetadataResult {
            packages: vec![CargoPackageInfo {
                name: "demo".to_owned(),
                version: "0.1.0".to_owned(),
                features: vec!["extra".to_owned()],
                targets: vec![CargoTargetInfo {
                    name: "demo".to_owned(),
                    kind: "bin".to_owned(),
                }],
            }],
            workspace_members: vec!["demo".to_owned()],
        })
        .unwrap();

        assert_eq!(value["packages"][0]["name"], "demo");
        assert_eq!(value["packages"][0]["targets"][0]["kind"], "bin");
        assert_eq!(value["workspaceMembers"][0], "demo");
    }
}

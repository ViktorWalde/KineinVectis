//! `CMake` service payloads (`cmake.*`).

use serde::{Deserialize, Serialize};

/// Parameters for `cmake.configure`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CmakeConfigureParams {
    /// Configure preset name from `cmake.presets.list`, when any. The build
    /// directory is always `<root>/.kinein/build` regardless of the preset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preset: Option<String>,
}

/// One configure preset advertised by the project.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CmakePresetInfo {
    /// Preset name accepted by `cmake --preset`.
    pub name: String,
    /// Human-friendly label, when the preset declares one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
}

/// Result payload for `cmake.presets.list`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CmakePresetsResult {
    /// Non-hidden configure presets, in file order
    /// (`CMakePresets.json` then `CMakeUserPresets.json`).
    pub presets: Vec<CmakePresetInfo>,
}

/// One build target reported by the `CMake` file API.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CmakeTargetInfo {
    /// Target name (`add_executable`/`add_library` name).
    pub name: String,
    /// Target kind (`executable`, `staticLibrary`, `sharedLibrary`, ...).
    pub kind: String,
}

/// Result payload for `cmake.targets.list`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CmakeTargetsResult {
    /// Targets from the last configure's file-api reply; empty before the
    /// first `cmake.configure`.
    pub targets: Vec<CmakeTargetInfo>,
}

/// Result payload for `cmake.status`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CmakeStatusResult {
    /// Whether `<buildDir>/CMakeCache.txt` exists.
    pub configured: bool,
    /// Whether `<buildDir>/compile_commands.json` exists (clangd uses it).
    pub has_compile_commands: bool,
    /// Canonical build directory used by configure/build/run.
    pub build_dir: String,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{CmakeConfigureParams, CmakePresetInfo, CmakePresetsResult, CmakeStatusResult};

    #[test]
    fn configure_params_accept_missing_preset_and_reject_unknown_fields() {
        let empty: CmakeConfigureParams = serde_json::from_value(json!({})).unwrap();
        assert_eq!(empty.preset, None);

        let with_preset: CmakeConfigureParams =
            serde_json::from_value(json!({ "preset": "dev" })).unwrap();
        assert_eq!(with_preset.preset.as_deref(), Some("dev"));

        let rejected = serde_json::from_value::<CmakeConfigureParams>(json!({ "extra": 1 }));
        assert!(rejected.is_err());
    }

    #[test]
    fn presets_result_serializes_camel_case_and_omits_empty_display_name() {
        let value = serde_json::to_value(CmakePresetsResult {
            presets: vec![
                CmakePresetInfo {
                    name: "dev".to_owned(),
                    display_name: Some("Dev Local".to_owned()),
                },
                CmakePresetInfo {
                    name: "ci".to_owned(),
                    display_name: None,
                },
            ],
        })
        .unwrap();

        assert_eq!(value["presets"][0]["displayName"], "Dev Local");
        assert!(value["presets"][1].get("displayName").is_none());
    }

    #[test]
    fn status_result_serializes_camel_case() {
        let value = serde_json::to_value(CmakeStatusResult {
            configured: true,
            has_compile_commands: false,
            build_dir: "/w/.kinein/build".to_owned(),
        })
        .unwrap();

        assert_eq!(value["configured"], true);
        assert_eq!(value["hasCompileCommands"], false);
        assert_eq!(value["buildDir"], "/w/.kinein/build");
    }
}

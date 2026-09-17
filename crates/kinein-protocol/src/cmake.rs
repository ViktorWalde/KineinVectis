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
///
/// Since 0.96.0 (2026-09-12) the target carries its MODEL: what it compiles,
/// with which flags, and what it produces — read from the `codemodel-v2`
/// reply, never from `CMakeLists.txt`. The fields after `kind` are empty when
/// the origin is `"source"` (the project was not configured yet).
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CmakeTargetInfo {
    /// Target name (`add_executable`/`add_library` name).
    pub name: String,
    /// Target kind (`executable`, `staticLibrary`, `sharedLibrary`, ...).
    pub kind: String,
    /// Artifacts the build produces (ELF, library), absolute.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub artifacts: Vec<String>,
    /// Source files written by the author (generated ones excluded).
    #[serde(default)]
    pub sources: u64,
    /// Generated sources (moc, rcc, ...).
    #[serde(default)]
    pub generated_sources: u64,
    /// Languages compiled (`C`, `CXX`, `ASM`), in file-api spelling.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub languages: Vec<String>,
    /// Language standard of the first compile group (`23`, `17`), when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub standard: Option<String>,
    /// Distinct include directories across compile groups.
    #[serde(default)]
    pub includes: u64,
    /// Distinct defines across compile groups.
    #[serde(default)]
    pub defines: u64,
    /// `CMAKE_SYSROOT` in effect, when any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sysroot: Option<String>,
    /// Names of the targets this one depends on.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dependencies: Vec<String>,
    /// Source directory of the target, absolute.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_dir: Option<String>,
}

/// Result payload for `cmake.targets.list`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CmakeTargetsResult {
    /// Where the names came from. `"fileApi"` is authoritative (kinds are
    /// real); `"source"` was parsed from `CMakeLists.txt` because the project
    /// has not been configured yet; `"none"` means neither found anything.
    ///
    /// The UI needs this to be honest with the author: a name read from the
    /// source is a name the author WROTE, not a target `CMake` confirmed.
    #[serde(default)]
    pub origin: String,
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
    /// The configure preset the IDE used on the last `cmake.configure`
    /// (`0.115.0`): the one asked for, else the kit's, else the project's
    /// default (first non-hidden `configurePresets` of `CMakeUserPresets.json`,
    /// then `CMakePresets.json`, whose `condition` does not exclude Linux).
    /// Absent when the last configure ran without a preset, or never ran.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preset: Option<String>,
    /// Directory holding the compilation database actually reachable by
    /// clangd, relative to the workspace root (`"."` for the root itself);
    /// absent when the workspace has none.
    ///
    /// This is **not** the same as `has_compile_commands`, which only looks at
    /// the IDE's own build directory. clangd also finds a database in parent
    /// directories and in `build/` subdirectories on its own, so a Meson or
    /// `bear` project can be fully working with `hasCompileCommands: false`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cdb_directory: Option<String>,
    /// The reachable compilation database is older than a build file that
    /// defines it, so clangd is using flags for a project that changed.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub cdb_stale: bool,
    /// Which build file made the database stale, when `cdbStale`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cdb_stale_because: Option<String>,
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
            preset: Some("linux-clang".to_owned()),
            cdb_directory: Some("build".to_owned()),
            cdb_stale: true,
            cdb_stale_because: Some("CMakeLists.txt".to_owned()),
        })
        .unwrap();

        assert_eq!(value["configured"], true);
        assert_eq!(value["hasCompileCommands"], false);
        assert_eq!(value["buildDir"], "/w/.kinein/build");
        assert_eq!(value["cdbDirectory"], "build");
        assert_eq!(value["cdbStale"], true);
        assert_eq!(value["cdbStaleBecause"], "CMakeLists.txt");
        assert_eq!(value["preset"], "linux-clang");
    }

    /// O caso saudavel nao carrega campo nenhum de diagnostico.
    ///
    /// `cdbStale: false` e `cdbStaleBecause: null` no fio seriam ruido em todo
    /// `cmake.status` de todo projeto sadio — e a UI teria de distinguir
    /// "ausente" de "falso". Os tres campos sao OMITIDOS quando nao ha o que
    /// dizer, e e' isso que mantem o contrato aditivo: um cliente antigo le a
    /// resposta nova sem mudar uma linha.
    #[test]
    fn status_result_omits_the_diagnosis_when_there_is_nothing_to_report() {
        let value = serde_json::to_value(CmakeStatusResult {
            configured: true,
            has_compile_commands: true,
            build_dir: "/w/.kinein/build".to_owned(),
            preset: None,
            cdb_directory: None,
            cdb_stale: false,
            cdb_stale_because: None,
        })
        .unwrap();

        assert!(value.get("cdbDirectory").is_none());
        assert!(value.get("cdbStale").is_none());
        assert!(value.get("cdbStaleBecause").is_none());
        assert!(value.get("preset").is_none());
    }
}

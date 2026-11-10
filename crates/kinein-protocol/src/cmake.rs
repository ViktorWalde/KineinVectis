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
    /// (`CMakePresets.json` then `CMakeUserPresets.json`), each file before
    /// the files it includes (`0.207.0`).
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
    /// The folder of the active configuration, which configure, build, run,
    /// debug and ctest use (`0.209.0`, C1-19–C1-21): `.kinein/build` without a
    /// preset, the `binaryDir` of a preset, else `.kinein/presets/<preset>`.
    /// Absent when there is no active configuration (`configurationRefused`).
    /// Until `0.208.0` always `.kinein/build`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub build_dir: Option<String>,
    /// The configure preset the IDE used on the last successful configure of
    /// `buildDir` (`0.115.0`, per folder since `0.209.0`). Absent when it ran
    /// without a preset, or never ran there.
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
    /// Stale reason: a newer build input or an incomplete IDE configure.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cdb_stale_because: Option<String>,
    /// The configure presets the person chooses among (`0.174.0`): present
    /// when a `cmake.configure` without a preset would be refused because the
    /// project has several and nothing was recorded.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub preset_choices: Vec<String>,
    /// The presets file `presetChoices` came from.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preset_choices_file: Option<String>,
    /// Where the compilation database in use came from (`0.206.0`, C-05).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cdb_origin: Option<CdbOrigin>,
    /// The configure preset whose `binaryDir` holds the database in use.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cdb_preset: Option<String>,
    /// Every directory found with a compilation database, the one in use
    /// included, in precedence order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cdb_candidates: Vec<CdbCandidate>,
    /// Why no database is in use although there is evidence of one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cdb_absence: Option<CdbAbsence>,
    /// The preset recorded by `cmake.selectConfiguration` or, since
    /// `0.209.0`, by the configure: the active configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_configuration: Option<String>,
    /// The preset of the active configuration (`0.209.0`); absent for one
    /// without preset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub configuration_preset: Option<String>,
    /// Why it is the active one (`0.209.0`): `"configuration"` (the record),
    /// `"buildDir"` (`.kinein/build` with the IDE's evidence), the presets
    /// file of the default, or `"noPresets"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub configuration_source: Option<String>,
    /// Why there is no active configuration (`0.209.0`); the consumers refuse
    /// with this sentence.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub configuration_refused: Option<String>,
    /// Every non-hidden configure preset with the folder of its configuration
    /// (`0.210.0`, C1-23): what the configuration menu offers. Absent without
    /// presets, or with presets `CMake` would refuse.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub configurations: Vec<CmakeConfigurationInfo>,
}

/// One configuration of the menu (`0.210.0`).
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CmakeConfigurationInfo {
    /// The configure preset.
    pub preset: String,
    /// Its `displayName`, when it declares one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// The folder, labelled as `buildDir`; absent with `buildDirRefused`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub build_dir: Option<String>,
    /// Why the folder is not known: a `binaryDir` that does not resolve, or a
    /// name that cannot be a folder.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub build_dir_refused: Option<String>,
    /// Whether the folder has a `CMakeCache.txt`.
    pub configured: bool,
}

/// Where the compilation database in use came from (`0.206.0`).
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CdbOrigin {
    /// The root `.clangd` fixes `CompileFlags.CompilationDatabase`.
    Clangd,
    /// The configuration the person chose with `cmake.selectConfiguration`.
    Configuration,
    /// The IDE's own configure directory (`.kinein/build`).
    Ide,
    /// A conventional directory: the root, `build/`, `builddir/`, `out/build/`.
    Directory,
    /// The `binaryDir` of the only configure preset that has a database.
    Preset,
}

/// No database in use although there is evidence of one (`0.206.0`).
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CdbAbsence {
    /// What the person can do about it.
    pub kind: CdbAbsenceKind,
    /// The explanation shown as is.
    pub reason: String,
}

/// The kinds of [`CdbAbsence`].
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CdbAbsenceKind {
    /// Several presets have a database and nothing was chosen: none is used
    /// until the person picks one (`cmake.selectConfiguration`).
    ChoiceRequired,
    /// The chosen configuration is unreadable, removed from the presets, or
    /// has no database in its directory; no other one replaces it.
    ChosenConfiguration,
    /// The root `.clangd` fixes a base without a database, or decides it in a
    /// form the IDE does not interpret.
    Clangd,
}

/// One directory holding a `compile_commands.json`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CdbCandidate {
    /// Relative to the workspace root (`"."` for the root); absolute outside it.
    pub directory: String,
    /// The rule that found it.
    pub origin: CdbOrigin,
    /// The configure preset whose `binaryDir` is this directory, when any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preset: Option<String>,
}

/// Parameters for `cmake.selectConfiguration` (`0.206.0`).
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CmakeSelectConfigurationParams {
    /// A non-hidden configure preset, or `null` to return to the automatic
    /// choice. The field is required: an empty request is not a reset.
    #[serde(deserialize_with = "required_nullable")]
    pub preset: Option<String>,
}

/// `Option` fields may be missing in serde; this one may only be `null`.
fn required_nullable<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Option::<String>::deserialize(deserializer)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{
        CdbAbsence, CdbAbsenceKind, CdbCandidate, CdbOrigin, CmakeConfigurationInfo,
        CmakeConfigureParams, CmakePresetInfo, CmakePresetsResult, CmakeSelectConfigurationParams,
        CmakeStatusResult,
    };

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
            build_dir: Some("/w/build/linux-clang".to_owned()),
            preset: Some("linux-clang".to_owned()),
            cdb_directory: Some("build".to_owned()),
            cdb_stale: true,
            cdb_stale_because: Some("CMakeLists.txt".to_owned()),
            preset_choices: Vec::new(),
            preset_choices_file: None,
            cdb_origin: None,
            cdb_preset: None,
            cdb_candidates: Vec::new(),
            cdb_absence: None,
            selected_configuration: Some("linux-clang".to_owned()),
            configuration_preset: Some("linux-clang".to_owned()),
            configuration_source: Some("configuration".to_owned()),
            configuration_refused: None,
            configurations: vec![
                CmakeConfigurationInfo {
                    preset: "linux-clang".to_owned(),
                    display_name: Some("Linux Clang".to_owned()),
                    build_dir: Some("/w/build/linux-clang".to_owned()),
                    build_dir_refused: None,
                    configured: true,
                },
                CmakeConfigurationInfo {
                    preset: "a/b".to_owned(),
                    display_name: None,
                    build_dir: None,
                    build_dir_refused: Some("o nome não serve de pasta".to_owned()),
                    configured: false,
                },
            ],
        })
        .unwrap();

        assert_eq!(
            value["configurations"],
            json!([
                {"preset":"linux-clang","displayName":"Linux Clang",
                 "buildDir":"/w/build/linux-clang","configured":true},
                {"preset":"a/b","buildDirRefused":"o nome não serve de pasta","configured":false}
            ])
        );
        assert_eq!(value["configured"], true);
        assert_eq!(value["hasCompileCommands"], false);
        assert_eq!(value["buildDir"], "/w/build/linux-clang");
        assert_eq!(value["configurationPreset"], "linux-clang");
        assert_eq!(value["configurationSource"], "configuration");
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
            build_dir: None,
            preset: None,
            cdb_directory: None,
            cdb_stale: false,
            cdb_stale_because: None,
            preset_choices: Vec::new(),
            preset_choices_file: None,
            cdb_origin: None,
            cdb_preset: None,
            cdb_candidates: Vec::new(),
            cdb_absence: None,
            selected_configuration: None,
            configuration_preset: None,
            configuration_source: None,
            configuration_refused: None,
            configurations: Vec::new(),
        })
        .unwrap();

        assert!(value.get("cdbDirectory").is_none());
        assert!(value.get("cdbStale").is_none());
        assert!(value.get("cdbStaleBecause").is_none());
        assert!(value.get("preset").is_none());
        for absent in [
            "cdbOrigin",
            "cdbPreset",
            "cdbCandidates",
            "cdbAbsence",
            "selectedConfiguration",
            "buildDir",
            "configurationPreset",
            "configurationSource",
            "configurationRefused",
            "configurations",
        ] {
            assert!(value.get(absent).is_none(), "{absent}");
        }
    }

    #[test]
    fn compile_database_origin_and_candidates_serialize_camel_case() {
        let candidate = CdbCandidate {
            directory: "build/debug".to_owned(),
            origin: CdbOrigin::Preset,
            preset: Some("debug".to_owned()),
        };
        assert_eq!(
            serde_json::to_value(&candidate).unwrap(),
            json!({"directory":"build/debug","origin":"preset","preset":"debug"})
        );
        assert_eq!(
            serde_json::to_value(CdbOrigin::Configuration).unwrap(),
            json!("configuration")
        );
        let ide = CdbCandidate {
            directory: ".kinein/build".to_owned(),
            origin: CdbOrigin::Ide,
            preset: None,
        };
        assert!(serde_json::to_value(ide).unwrap().get("preset").is_none());
        let absence = CdbAbsence {
            kind: CdbAbsenceKind::ChoiceRequired,
            reason: "escolha uma".to_owned(),
        };
        assert_eq!(
            serde_json::to_value(absence).unwrap(),
            json!({"kind":"choiceRequired","reason":"escolha uma"})
        );
        assert_eq!(
            serde_json::to_value(CdbAbsenceKind::ChosenConfiguration).unwrap(),
            json!("chosenConfiguration")
        );
    }

    #[test]
    fn select_configuration_requires_the_field_and_accepts_null() {
        let chosen: CmakeSelectConfigurationParams =
            serde_json::from_value(json!({"preset":"debug"})).unwrap();
        assert_eq!(chosen.preset.as_deref(), Some("debug"));
        let automatic: CmakeSelectConfigurationParams =
            serde_json::from_value(json!({"preset":null})).unwrap();
        assert_eq!(automatic.preset, None);
        assert!(serde_json::from_value::<CmakeSelectConfigurationParams>(json!({})).is_err());
        assert!(
            serde_json::from_value::<CmakeSelectConfigurationParams>(
                json!({"preset":"x","extra":1})
            )
            .is_err()
        );
    }
}

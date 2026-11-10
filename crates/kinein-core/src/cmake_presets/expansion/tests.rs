//! C1-24: every preset as `CMake` 4.4.5 reads it (consolidacao C §17.4).
//! Each case was measured with `cmake --list-presets` and `cmake --preset`;
//! `tests/c_project/preset_macros.rs` repeats them against the real `CMake`.
//! These test the native reading: since C1-25 the `CMake` of the project
//! answers first (`verdict/tests.rs`), so each test turns it off.

use std::path::{Path, PathBuf};

use super::super::{
    PresetDefault, binary_dir, default_preset, list_presets, native_reading, preset_value,
};

fn project(name: &str, presets: &str) -> PathBuf {
    let dir = std::env::temp_dir()
        .join("kinein-cmake-expansion")
        .join(format!("{}-{name} ação", std::process::id()));
    drop(std::fs::remove_dir_all(&dir));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("CMakePresets.json"), presets).unwrap();
    dir
}

fn write(root: &Path, relative: &str, body: &str) {
    let path = root.join(relative);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, body).unwrap();
}

fn names(root: &Path) -> Vec<String> {
    list_presets(root)
        .unwrap()
        .into_iter()
        .map(|preset| preset.name)
        .collect()
}

const OK: &str = r#"{"name":"ok","generator":"Ninja","binaryDir":"${sourceDir}/b/ok"}"#;

/// A file with `ok` and one more preset of `kind`, in `version`.
fn file(version: u64, kind: &str, preset: &str) -> String {
    if kind == "configurePresets" {
        format!(r#"{{"version":{version},"configurePresets":[{OK},{preset}]}}"#)
    } else {
        format!(r#"{{"version":{version},"configurePresets":[{OK}],"{kind}":[{preset}]}}"#)
    }
}

/// Each field `CMake` expands, of each kind, with `${nope}`: the preset is
/// hidden or has no use, and still the whole file is refused.
const EXPANDED: &[(&str, &str)] = &[
    (
        "configurePresets",
        r#"{"name":"x","hidden":true,"binaryDir":"${nope}"}"#,
    ),
    (
        "configurePresets",
        r#"{"name":"x","hidden":true,"installDir":"${nope}"}"#,
    ),
    (
        "configurePresets",
        r#"{"name":"x","hidden":true,"toolchainFile":"${nope}"}"#,
    ),
    (
        "configurePresets",
        r#"{"name":"x","hidden":true,"graphviz":"${nope}"}"#,
    ),
    (
        "configurePresets",
        r#"{"name":"x","hidden":true,"cacheVariables":{"A":"${nope}"}}"#,
    ),
    (
        "configurePresets",
        r#"{"name":"x","hidden":true,"cacheVariables":{"A":{"type":"PATH","value":"${nope}"}}}"#,
    ),
    (
        "configurePresets",
        r#"{"name":"x","hidden":true,"environment":{"A":"${nope}"}}"#,
    ),
    (
        "configurePresets",
        r#"{"name":"x","hidden":true,"condition":{"type":"not","condition":{"type":"inList","string":"a","list":["${nope}"]}}}"#,
    ),
    (
        "buildPresets",
        r#"{"name":"x","configurePreset":"ok","environment":{"A":"${nope}"}}"#,
    ),
    (
        "buildPresets",
        r#"{"name":"x","configurePreset":"ok","targets":"${nope}"}"#,
    ),
    (
        "buildPresets",
        r#"{"name":"x","hidden":true,"targets":["a","${nope}"]}"#,
    ),
    (
        "buildPresets",
        r#"{"name":"x","configurePreset":"ok","nativeToolOptions":["${nope}"]}"#,
    ),
    (
        "buildPresets",
        r#"{"name":"x","configurePreset":"ok","condition":{"type":"equals","lhs":"a","rhs":"${nope}"}}"#,
    ),
    (
        "testPresets",
        r#"{"name":"x","configurePreset":"ok","overwriteConfigurationFile":["${nope}"]}"#,
    ),
    (
        "testPresets",
        r#"{"name":"x","configurePreset":"ok","output":{"outputLogFile":"${nope}"}}"#,
    ),
    (
        "testPresets",
        r#"{"name":"x","configurePreset":"ok","output":{"outputJUnitFile":"${nope}"}}"#,
    ),
    (
        "testPresets",
        r#"{"name":"x","configurePreset":"ok","filter":{"include":{"name":"${nope}"}}}"#,
    ),
    (
        "testPresets",
        r#"{"name":"x","configurePreset":"ok","filter":{"include":{"label":"${nope}"}}}"#,
    ),
    (
        "testPresets",
        r#"{"name":"x","configurePreset":"ok","filter":{"include":{"index":"${nope}"}}}"#,
    ),
    (
        "testPresets",
        r#"{"name":"x","configurePreset":"ok","filter":{"exclude":{"name":"${nope}"}}}"#,
    ),
    (
        "testPresets",
        r#"{"name":"x","configurePreset":"ok","filter":{"exclude":{"label":"${nope}"}}}"#,
    ),
    (
        "testPresets",
        r#"{"name":"x","configurePreset":"ok","filter":{"exclude":{"fixtures":{"cleanup":"${nope}"}}}}"#,
    ),
    (
        "testPresets",
        r#"{"name":"x","configurePreset":"ok","execution":{"resourceSpecFile":"${nope}"}}"#,
    ),
    (
        "packagePresets",
        r#"{"name":"x","configurePreset":"ok","variables":{"A":"${nope}"}}"#,
    ),
    (
        "packagePresets",
        r#"{"name":"x","configurePreset":"ok","configFile":"${nope}"}"#,
    ),
    (
        "packagePresets",
        r#"{"name":"x","configurePreset":"ok","packageName":"${nope}"}"#,
    ),
    (
        "packagePresets",
        r#"{"name":"x","configurePreset":"ok","packageVersion":"${nope}"}"#,
    ),
    (
        "packagePresets",
        r#"{"name":"x","configurePreset":"ok","packageDirectory":"${nope}"}"#,
    ),
    (
        "packagePresets",
        r#"{"name":"x","configurePreset":"ok","vendorName":"${nope}"}"#,
    ),
];

/// Fields `CMake` takes as written, whatever they hold.
const LITERAL: &[(&str, &str)] = &[
    (
        "configurePresets",
        r#"{"name":"x","displayName":"${nope}","description":"${nope","binaryDir":"o"}"#,
    ),
    (
        "configurePresets",
        r#"{"name":"x","hidden":true,"cmakeExecutable":"$env{}"}"#,
    ),
    (
        "configurePresets",
        r#"{"name":"x","hidden":true,"cacheVariables":{"A":{"type":"${nope}","value":"a"},"${nope}":"b"}}"#,
    ),
    (
        "configurePresets",
        r#"{"name":"x","hidden":true,"cacheVariables":{"B":true,"N":null}}"#,
    ),
    (
        "buildPresets",
        r#"{"name":"x","configurePreset":"ok","configuration":"${nope}","displayName":"${nope}"}"#,
    ),
    (
        "testPresets",
        r#"{"name":"x","configurePreset":"ok","configuration":"${nope}","execution":{"testPassthroughArguments":["${nope}"]}}"#,
    ),
    (
        "packagePresets",
        r#"{"name":"x","configurePreset":"ok","generators":["${nope}"],"configurations":["${nope}"]}"#,
    ),
    (
        "workflowPresets",
        r#"{"name":"w","displayName":"${nope}","steps":[{"type":"configure","name":"ok"}]}"#,
    ),
];

#[test]
fn an_invalid_macro_in_any_field_of_any_preset_refuses_every_preset() {
    let _native = native_reading();
    for (index, (kind, preset)) in EXPANDED.iter().enumerate() {
        let root = project(&format!("expanded-{index}"), &file(10, kind, preset));
        let refused = list_presets(&root).unwrap_err();
        assert!(
            refused.starts_with("macro inválida no preset de ")
                && refused.ends_with("\"x\": macro desconhecida: ${nope}"),
            "{kind} {preset}: {refused}"
        );
        assert_eq!(
            default_preset(&root),
            PresetDefault::Refused(refused.clone()),
            "{preset}"
        );
        assert_eq!(binary_dir(&root, "ok").unwrap_err(), refused, "{preset}");
        assert_eq!(preset_value(&root, "ok", &["binaryDir"]), None, "{preset}");
    }
    for (index, (kind, preset)) in LITERAL.iter().enumerate() {
        let root = project(&format!("literal-{index}"), &file(10, kind, preset));
        assert!(
            list_presets(&root).is_ok(),
            "{kind} {preset}: {:?}",
            list_presets(&root)
        );
    }
}

#[test]
fn every_kind_of_invalid_macro_is_refused_with_its_reason() {
    let _native = native_reading();
    for (text, reason) in [
        ("x${presetName", "macro sem fechamento: ${presetName"),
        ("$env{}", "$env{} sem nome"),
        ("$penv{}", "$penv{} sem nome"),
        ("${nope}", "macro desconhecida: ${nope}"),
    ] {
        let preset = format!(r#"{{"name":"h","hidden":true,"cacheVariables":{{"A":"{text}"}}}}"#);
        let root = project("kinds", &file(10, "configurePresets", &preset));
        assert_eq!(
            list_presets(&root).unwrap_err(),
            format!("macro inválida no preset de configure \"h\": {reason}")
        );
    }
}

/// The macros of a newer version refuse an older file; the version is that
/// of the preset being read, also for an inherited field.
#[test]
fn macros_follow_the_version_of_the_preset_file() {
    let _native = native_reading();
    for (version, macro_, since) in [
        (2, "hostSystemName", 3),
        (3, "fileDir", 4),
        (4, "pathListSep", 5),
    ] {
        let preset =
            format!(r#"{{"name":"h","hidden":true,"cacheVariables":{{"M":"${{{macro_}}}"}}}}"#);
        let old = project("gate-old", &file(version, "configurePresets", &preset));
        assert_eq!(
            list_presets(&old).unwrap_err(),
            format!(
                "macro inválida no preset de configure \"h\": a macro ${{{macro_}}} exige version {since} ou maior"
            )
        );
        let new = project("gate-new", &file(since, "configurePresets", &preset));
        assert_eq!(names(&new), ["ok"]);
    }
    let base = r#"{"version":5,"configurePresets":[{"name":"base","hidden":true,"cacheVariables":{"M":"${pathListSep}"}}]}"#;
    let root = project(
        "gate-inherited",
        r#"{"version":4,"include":["sub/inc.json"],"configurePresets":[{"name":"kid","inherits":"base","binaryDir":"o"}]}"#,
    );
    write(&root, "sub/inc.json", base);
    assert_eq!(
        list_presets(&root).unwrap_err(),
        "macro inválida no preset de configure \"kid\": a macro ${pathListSep} exige version 5 ou maior"
    );
    // Sem o filho, o pai da versão 5 vale.
    std::fs::write(
        root.join("CMakePresets.json"),
        r#"{"version":4,"include":["sub/inc.json"],"configurePresets":[]}"#,
    )
    .unwrap();
    assert!(names(&root).is_empty());
    // Os build presets também.
    let build = root_build();
    assert_eq!(
        list_presets(&build).unwrap_err(),
        "macro inválida no preset de build \"x\": a macro ${pathListSep} exige version 5 ou maior"
    );
}

fn root_build() -> PathBuf {
    project(
        "gate-build",
        &file(
            4,
            "buildPresets",
            r#"{"name":"x","configurePreset":"ok","environment":{"X":"${pathListSep}"}}"#,
        ),
    )
}

#[test]
fn environment_cycles_refuse_and_a_removed_variable_falls_back_to_the_core() {
    let _native = native_reading();
    for (name, preset, refused_in, variable) in [
        (
            "cycle",
            r#"{"name":"h","hidden":true,"environment":{"A":"$env{B}","B":"$env{A}"}}"#,
            "configure \"h\"",
            "A",
        ),
        (
            "self",
            r#"{"name":"h","hidden":true,"environment":{"A":"x$env{A}"}}"#,
            "configure \"h\"",
            "A",
        ),
    ] {
        let root = project(name, &file(10, "configurePresets", preset));
        assert_eq!(
            list_presets(&root).unwrap_err(),
            format!(
                "macro inválida no preset de {refused_in}: ciclo de macros em environment: {variable}"
            ),
            "{name}"
        );
    }
    // O pai sozinho vale; o ciclo nasce no filho, que junta os dois.
    let root = project(
        "cycle-inherited",
        r#"{"version":10,"configurePresets":[
          {"name":"base","hidden":true,"environment":{"A":"$env{B}"}},
          {"name":"kid","inherits":"base","binaryDir":"o","environment":{"B":"$env{A}"}}]}"#,
    );
    assert_eq!(
        list_presets(&root).unwrap_err(),
        "macro inválida no preset de configure \"kid\": ciclo de macros em environment: A"
    );
    // O build preset herda o environment do configure preset, salvo
    // `inheritConfigureEnvironment: false`.
    let with_configure = r#"{"version":10,"configurePresets":[{"name":"ok","binaryDir":"o","environment":{"A":"$env{B}"}}],
      "buildPresets":[{"name":"x","configurePreset":"ok","environment":{"B":"$env{A}"}}]}"#;
    let root_build = project("cycle-build", with_configure);
    assert_eq!(
        list_presets(&root_build).unwrap_err(),
        "macro inválida no preset de build \"x\": ciclo de macros em environment: A"
    );
    std::fs::write(
        root_build.join("CMakePresets.json"),
        with_configure.replace(
            r#""configurePreset":"ok","#,
            r#""configurePreset":"ok","inheritConfigureEnvironment":false,"#,
        ),
    )
    .unwrap();
    assert_eq!(names(&root_build), ["ok"]);

    let home = std::env::var("HOME").unwrap();
    let root = project(
        "null",
        r#"{"version":10,"configurePresets":[
          {"name":"base","hidden":true,"environment":{"KV_FROM_BASE":"pai","HOME":"/nao"}},
          {"name":"kid","inherits":"base","environment":{"KV_FROM_BASE":null,"HOME":null},
           "binaryDir":"$env{HOME}/k[$env{KV_FROM_BASE}]"},
          {"name":"path","environment":{"PATH":"/x:$penv{PATH}"},"binaryDir":"${sourceDir}/p"}]}"#,
    );
    assert_eq!(
        binary_dir(&root, "kid").unwrap().unwrap(),
        PathBuf::from(format!("{home}/k[]"))
    );
    assert_eq!(names(&root), ["kid", "path"]);
}

/// `$vendor{}` stops the preset in the order `CMake` expands: environment
/// (variables by name), condition, then the fields of the kind. What comes
/// after is never expanded, and a hidden preset is no loss.
#[test]
fn vendor_stops_the_preset_in_the_order_cmake_expands() {
    let _native = native_reading();
    for (name, preset, accepted) in [
        (
            "vendor-then-invalid",
            r#"{"name":"h","hidden":true,"cacheVariables":{"A":"$vendor{x}${nope}"}}"#,
            true,
        ),
        (
            "invalid-then-vendor",
            r#"{"name":"h","hidden":true,"cacheVariables":{"A":"${nope}$vendor{x}"}}"#,
            false,
        ),
        (
            "binary-before-cache",
            r#"{"name":"h","hidden":true,"binaryDir":"$vendor{x}","cacheVariables":{"A":"${nope}"}}"#,
            true,
        ),
        (
            "cache-after-binary",
            r#"{"name":"h","hidden":true,"binaryDir":"${nope}","cacheVariables":{"A":"$vendor{x}"}}"#,
            false,
        ),
        (
            "env-by-name",
            r#"{"name":"h","hidden":true,"environment":{"B":"$vendor{x}","A":"${nope}"}}"#,
            false,
        ),
        (
            "env-first-name",
            r#"{"name":"h","hidden":true,"environment":{"A":"$vendor{x}","B":"${nope}"}}"#,
            true,
        ),
        (
            "env-before-condition",
            r#"{"name":"h","hidden":true,"environment":{"A":"$vendor{x}"},"condition":{"type":"equals","lhs":"${nope}","rhs":"a"}}"#,
            true,
        ),
        (
            "condition-before-fields",
            r#"{"name":"h","hidden":true,"condition":{"type":"equals","lhs":"$vendor{x}","rhs":"${nope}"},"binaryDir":"${nope}"}"#,
            true,
        ),
        (
            "gate-after-vendor",
            r#"{"name":"h","hidden":true,"cacheVariables":{"A":"$vendor{x}${pathListSep}"}}"#,
            true,
        ),
    ] {
        let root = project(name, &file(4, "configurePresets", preset));
        assert_eq!(
            list_presets(&root).is_ok(),
            accepted,
            "{name}: {:?}",
            list_presets(&root)
        );
    }
    let root = project(
        "vendor-listing",
        r#"{"version":10,"configurePresets":[
          {"name":"base","hidden":true,"cacheVariables":{"V":"$vendor{acme}"}},
          {"name":"kid","inherits":"base","binaryDir":"k"},
          {"name":"own","binaryDir":"o","environment":{"A":"$vendor{acme}"}},
          {"name":"ok","binaryDir":"ok"}],
          "buildPresets":[{"name":"b","configurePreset":"ok","environment":{"A":"$vendor{acme}"}}]}"#,
    );
    assert_eq!(names(&root), ["ok"]);
    assert_eq!(
        default_preset(&root),
        PresetDefault::One("ok".to_owned(), "CMakePresets.json")
    );
    assert_eq!(
        binary_dir(&root, "kid").unwrap_err(),
        "o CMake ignora o preset kid: ele usa $vendor{}"
    );
}

#[test]
fn conditions_are_inherited_and_evaluated_like_cmake() {
    let _native = native_reading();
    let root = project(
        "conditions",
        r#"{"version":10,"configurePresets":[
          {"name":"off","hidden":true,"condition":{"type":"const","value":false}},
          {"name":"inherits-off","inherits":"off","binaryDir":"a"},
          {"name":"overrides","inherits":"off","binaryDir":"b","condition":true},
          {"name":"undoes","inherits":"off","binaryDir":"c","condition":null},
          {"name":"env","binaryDir":"d","environment":{"K":"sim"},"condition":{"type":"equals","lhs":"$env{K}","rhs":"sim"}},
          {"name":"any","binaryDir":"e","condition":{"type":"anyOf","conditions":[true,{"type":"equals","lhs":"${nope}","rhs":"a"}]}},
          {"name":"all","binaryDir":"f","condition":{"type":"allOf","conditions":[false,{"type":"equals","lhs":"${nope}","rhs":"a"}]}},
          {"name":"any-empty","binaryDir":"g","condition":{"type":"anyOf","conditions":[]}},
          {"name":"all-empty","binaryDir":"h","condition":{"type":"allOf","conditions":[]}},
          {"name":"in-empty","binaryDir":"i","condition":{"type":"inList","string":"a","list":[]}},
          {"name":"not-in","binaryDir":"j","condition":{"type":"notInList","string":"${presetName}","list":["x","not-in"]}},
          {"name":"host","binaryDir":"k","condition":{"type":"notEquals","lhs":"${hostSystemName}","rhs":"Windows"}},
          {"name":"regex","binaryDir":"l","condition":{"type":"matches","string":"a1","regex":"^a\\d$"}},
          {"name":"false","binaryDir":"m","condition":false}]}"#,
    );
    // `regex` fica: a expressão regular do CMake não é decidida aqui (C1-26).
    assert_eq!(
        names(&root),
        [
            "overrides",
            "undoes",
            "env",
            "any",
            "all-empty",
            "host",
            "regex"
        ]
    );
    assert_eq!(
        binary_dir(&root, "inherits-off").unwrap_err(),
        "a condition do preset inherits-off é falsa: o CMake não o usa"
    );
    // A condition falsa não interrompe a expansão (medido).
    let root = project(
        "condition-false-expands",
        &file(
            10,
            "configurePresets",
            r#"{"name":"c","condition":false,"binaryDir":"o","cacheVariables":{"X":"${nope}"}}"#,
        ),
    );
    assert!(list_presets(&root).is_err());
}

/// C1-26: `CMake` stops an `anyOf` at a true `matches` and an `allOf` at a
/// false one, and never reads what comes after (measured with 4.4.5). After
/// an undecided form, an invalid macro or a `$vendor{}` leave the `condition`
/// undecided, also nested; before it they still refuse or stop the preset,
/// and a form after it that decides alone still decides. Without `CMake`, the
/// undecided ones count as true and say so.
#[test]
fn after_an_unknown_form_an_error_leaves_the_condition_unknown() {
    let _native = native_reading();
    let rx = r#"{"type":"matches","string":"a1","regex":"^a1$"}"#;
    let bad = r#"{"type":"equals","lhs":"${nope}","rhs":"x"}"#;
    let vendor = r#"{"type":"equals","lhs":"$vendor{acme}","rhs":"x"}"#;
    let preset = |name: &str, condition: String| {
        format!(r#"{{"name":"{name}","binaryDir":"{name}","condition":{condition}}}"#)
    };
    let any = |forms: &[&str]| format!(r#"{{"type":"anyOf","conditions":[{}]}}"#, forms.join(","));
    let all = |forms: &[&str]| format!(r#"{{"type":"allOf","conditions":[{}]}}"#, forms.join(","));
    let regex_macro = r#"{"type":"matches","string":"a","regex":"${nope}"}"#;
    let nested = format!(
        r#"{{"type":"not","condition":{}}}"#,
        all(&[&any(&[rx, bad]), "true"])
    );
    let presets = [
        preset("any-bad", any(&[rx, bad])),
        preset("all-bad", all(&[rx, bad])),
        preset("any-regex-macro", any(&[rx, regex_macro])),
        preset("any-vendor", any(&[rx, vendor])),
        preset("nested", nested),
        preset("any-true-after", any(&[rx, "true", bad])),
        preset("all-false-after", all(&[rx, "false", bad])),
        preset("vendor-first", any(&[vendor, rx])),
    ];
    let root = project(
        "undecided",
        &format!(
            r#"{{"version":10,"configurePresets":[{}]}}"#,
            presets.join(",")
        ),
    );
    let listed: Vec<(String, bool)> = list_presets(&root)
        .unwrap()
        .into_iter()
        .map(|preset| (preset.name, preset.condition_note.is_some()))
        .collect();
    let noted = |name: &str| (name.to_owned(), true);
    assert_eq!(
        listed,
        [
            noted("any-bad"),
            noted("all-bad"),
            noted("any-regex-macro"),
            noted("any-vendor"),
            noted("nested"),
            ("any-true-after".to_owned(), false),
        ]
    );
    assert_eq!(
        list_presets(&root).unwrap()[0].condition_note.as_deref(),
        Some(
            "a condition usa matches ou notMatches, que só o CMake decide; sem ele, a IDE a dá \
             por verdadeira"
        )
    );
    assert_eq!(
        binary_dir(&root, "all-false-after").unwrap_err(),
        "a condition do preset all-false-after é falsa: o CMake não o usa"
    );
    assert_eq!(
        binary_dir(&root, "vendor-first").unwrap_err(),
        "o CMake ignora o preset vendor-first: ele usa $vendor{}"
    );
    // Before the undecided form, the invalid macro still refuses them all.
    let root = project(
        "undecided-after",
        &format!(
            r#"{{"version":10,"configurePresets":[{OK},{}]}}"#,
            preset("before", any(&[bad, rx]))
        ),
    );
    assert!(
        list_presets(&root)
            .unwrap_err()
            .starts_with("macro inválida no preset de configure \"before\": "),
        "{:?}",
        list_presets(&root)
    );
}

/// `${generator}` in build and test presets is the generator of the configure
/// preset, also inherited; in package presets it is always empty (measured).
/// The `anyOf` shows it: when the first test is true, `${nope}` is never
/// expanded and the file is accepted.
#[test]
fn build_and_test_presets_see_the_configure_generator_and_package_presets_none() {
    let _native = native_reading();
    for (kind, configure, generator, accepted) in [
        ("buildPresets", "ok", "Ninja", true),
        ("buildPresets", "heir", "Ninja", true),
        ("buildPresets", "bare", "", true),
        ("buildPresets", "ok", "", false),
        ("testPresets", "ok", "Ninja", true),
        ("testPresets", "heir", "Ninja", true),
        ("testPresets", "ok", "", false),
        ("packagePresets", "ok", "Ninja", false),
        ("packagePresets", "heir", "Ninja", false),
        ("packagePresets", "ok", "", true),
    ] {
        let root = project(
            &format!("generator-{kind}-{configure}-{generator}"),
            &format!(
                r#"{{"version":10,"configurePresets":[{OK},
                  {{"name":"heir","inherits":"ok","binaryDir":"h"}},
                  {{"name":"bare","binaryDir":"b"}}],
                  "{kind}":[{{"name":"x","configurePreset":"{configure}","condition":{{"type":"anyOf","conditions":[
                    {{"type":"equals","lhs":"${{generator}}","rhs":"{generator}"}},
                    {{"type":"equals","lhs":"${{nope}}","rhs":"a"}}]}}}}]}}"#
            ),
        );
        assert_eq!(
            list_presets(&root).is_ok(),
            accepted,
            "{kind} {configure} {generator:?}"
        );
    }
}

/// Version 12 (measured with `CMake` 4.4.5): a text whose only macros are
/// `${fileDir}` names the folder of the file that writes it, also when
/// inherited, in `binaryDir`, `installDir`, `toolchainFile`, `graphviz` and
/// `cacheVariables`. Any other macro, and every `environment`, leaves it to
/// the file of the preset in use. Until C1-24 the IDE always used the file
/// of the preset in use, and configured with `-B` where `cmake --preset`
/// does not.
#[test]
fn file_dir_in_version_12_names_the_file_that_writes_it() {
    let _native = native_reading();
    for (root_version, sub_version) in [(12, 12), (11, 12), (12, 11), (6, 6)] {
        let root = project(
            &format!("filedir-{root_version}-{sub_version}"),
            &format!(
                r#"{{"version":{root_version},"include":["sub/inc.json"],"configurePresets":[
                  {{"name":"kid","inherits":"base"}},
                  {{"name":"named","inherits":"base","binaryDir":"${{fileDir}}/out-${{presetName}}"}},
                  {{"name":"env","inherits":"base","binaryDir":"$env{{FD}}/e"}}]}}"#
            ),
        );
        write(
            &root,
            "sub/inc.json",
            &format!(
                r#"{{"version":{sub_version},"configurePresets":[{{"name":"base","hidden":true,
                  "binaryDir":"${{fileDir}}/out","toolchainFile":"${{fileDir}}/t.cmake",
                  "environment":{{"FD":"${{fileDir}}"}},
                  "cacheVariables":{{"CMAKE_C_COMPILER":{{"type":"FILEPATH","value":"${{fileDir}}/cc"}},
                                     "MIXED":"${{fileDir}}|${{dollar}}"}}}}]}}"#
            ),
        );
        let own = if sub_version >= 12 {
            root.join("sub")
        } else {
            root.clone()
        };
        assert_eq!(
            binary_dir(&root, "kid").unwrap().unwrap(),
            own.join("out"),
            "{root_version}/{sub_version}"
        );
        assert_eq!(
            preset_value(&root, "kid", &["toolchainFile"]).unwrap(),
            own.join("t.cmake").to_string_lossy()
        );
        assert_eq!(
            preset_value(&root, "kid", &["cacheVariables", "CMAKE_C_COMPILER"]).unwrap(),
            own.join("cc").to_string_lossy()
        );
        assert_eq!(
            preset_value(&root, "kid", &["cacheVariables", "MIXED"]).unwrap(),
            format!("{}|$", root.display())
        );
        // Escrito no arquivo do preset em uso, com `${presetName}`, e no
        // environment: a pasta do preset em uso.
        assert_eq!(
            binary_dir(&root, "named").unwrap().unwrap(),
            root.join("out-named")
        );
        assert_eq!(binary_dir(&root, "env").unwrap().unwrap(), root.join("e"));
    }
}

/// A toolchain lê os valores expandidos (C1-24): antes mostrava o texto com
/// as macros.
#[test]
fn the_toolchain_reads_expanded_values() {
    let _native = native_reading();
    let root = project(
        "toolchain",
        r#"{"version":10,"configurePresets":[{"name":"arm","binaryDir":"o",
          "environment":{"CC_DIR":"${sourceDir}/bin"},
          "toolchainFile":"${sourceDir}/cmake/${presetName}.cmake",
          "cacheVariables":{"CMAKE_C_COMPILER":"$env{CC_DIR}/gcc"}}]}"#,
    );
    assert_eq!(
        preset_value(&root, "arm", &["toolchainFile"]).unwrap(),
        root.join("cmake/arm.cmake").to_string_lossy()
    );
    assert_eq!(
        preset_value(&root, "arm", &["cacheVariables", "CMAKE_C_COMPILER"]).unwrap(),
        root.join("bin/gcc").to_string_lossy()
    );
}

//! C1-25: the `CMake` of the project answers for the presets (consolidacao C
//! §18). The schema cases and the generators run against the real `CMake`
//! 4.4.5, as measured on 2026-11-10; the cache, the parsing and the absence
//! of `CMake` use fake executables that write down each call.

use std::path::{Path, PathBuf};

use super::super::{
    PresetDefault, binary_dir, binary_dirs, default_preset, list_from, list_presets, load_with,
    native_reading,
};
use crate::kits::{
    pinned_program,
    store::{Kit, save},
};

fn project(name: &str, presets: &str) -> PathBuf {
    let dir = std::env::temp_dir()
        .join("kinein-cmake-verdict")
        .join(format!("{}-{name} ação", std::process::id()));
    drop(std::fs::remove_dir_all(&dir));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("CMakePresets.json"), presets).unwrap();
    dir
}

fn names(root: &Path) -> Vec<String> {
    list_presets(root)
        .unwrap()
        .into_iter()
        .map(|preset| preset.name)
        .collect()
}

const OK: &str = r#"{"name":"ok","generator":"Ninja","binaryDir":"o"}"#;

/// Each case the native reading takes and `CMake` 4.4.5 refuses, with the
/// line it gives after "Could not read presets from" (measured).
const SCHEMA: &[(&str, &str, &str)] = &[
    (
        "extra-root",
        r#"{"version":10,"configurePresets":[OK],"nope":1}"#,
        r#"CMakePresets.json:1: Invalid extra field "nope" in root object"#,
    ),
    (
        "extra-preset",
        r#"{"version":10,"configurePresets":[{"name":"ok","generator":"Ninja","binaryDir":"o","nope":1}]}"#,
        r#"CMakePresets.json:1: Invalid extra field "nope" in Preset"#,
    ),
    (
        "installDir-v2",
        r#"{"version":2,"configurePresets":[{"name":"ok","generator":"Ninja","binaryDir":"o","installDir":"i"}]}"#,
        "File version must be 3 or higher for installDir preset support",
    ),
    (
        "graphviz-v9",
        r#"{"version":9,"configurePresets":[{"name":"ok","generator":"Ninja","binaryDir":"o","graphviz":"g"}]}"#,
        "File version must be 10 or higher for graphviz preset support",
    ),
    (
        "binaryDir-number",
        r#"{"version":10,"configurePresets":[{"name":"ok","generator":"Ninja","binaryDir":3}]}"#,
        r#"CMakePresets.json:1: "binaryDir" expected a string, got: 3"#,
    ),
    (
        "missing-configurePreset",
        r#"{"version":10,"configurePresets":[OK],"buildPresets":[{"name":"b","configurePreset":"x"}]}"#,
        r#"Invalid "configurePreset": "b""#,
    ),
    (
        "condition-without-rhs",
        r#"{"version":10,"configurePresets":[{"name":"ok","generator":"Ninja","binaryDir":"o","condition":{"type":"equals","lhs":"a"}}]}"#,
        r#"CMakePresets.json:1: Missing required field "rhs" in  condition for preset "ok""#,
    ),
    (
        "unknown-condition",
        r#"{"version":10,"configurePresets":[{"name":"ok","generator":"Ninja","binaryDir":"o","condition":{"type":"maybe"}}]}"#,
        r#"CMakePresets.json:1: Invalid condition for preset "ok""#,
    ),
    (
        "empty-name",
        r#"{"version":10,"configurePresets":[OK,{"name":""}]}"#,
        "CMakePresets.json:1: Invalid Preset Name",
    ),
    (
        "invalid-regex",
        r#"{"version":10,"configurePresets":[{"name":"ok","generator":"Ninja","binaryDir":"o","condition":{"type":"matches","string":"a","regex":"("}}]}"#,
        r#"Invalid configure preset: "ok": Invalid condition"#,
    ),
];

/// Until C1-25 the IDE listed these presets, offered them in the menu and
/// configured with them, and the `CMake` failed after.
#[test]
fn the_cmake_of_the_project_refuses_what_its_schema_does_not_accept() {
    for (name, presets, reason) in SCHEMA {
        let root = project(name, &presets.replace("OK", OK));
        // The native reading took the file: the `condition` without `rhs`
        // only as false.
        let native = native_reading();
        assert!(list_presets(&root).is_ok(), "{name}");
        drop(native);
        let refused = format!("o CMake recusa os presets: {reason}");
        assert_eq!(list_presets(&root).unwrap_err(), refused, "{name}");
        assert_eq!(
            default_preset(&root),
            PresetDefault::Refused(refused.clone()),
            "{name}"
        );
        assert_eq!(binary_dir(&root, "ok").unwrap_err(), refused, "{name}");
    }
}

/// Measured with 4.4.5: `cmake --list-presets` leaves out a preset whose
/// generator this `CMake` does not have, also inherited, and the one whose
/// `matches` is false (`\d` is no class for it).
#[test]
fn presets_this_cmake_cannot_generate_are_neither_listed_nor_configured() {
    let root = project(
        "generators",
        r#"{"version":10,"configurePresets":[
          {"name":"ok","generator":"Ninja","binaryDir":"o"},
          {"name":"vs","generator":"Visual Studio 17 2022","binaryDir":"vs"},
          {"name":"base","hidden":true,"generator":"Nope"},
          {"name":"heir","inherits":"base","binaryDir":"h"},
          {"name":"fixed","inherits":"base","generator":"Ninja","binaryDir":"f"},
          {"name":"plain","binaryDir":"n"},
          {"name":"rx","generator":"Ninja","binaryDir":"r","condition":{"type":"matches","string":"a1","regex":"^a\\d$"}}]}"#,
    );
    let native = native_reading();
    assert_eq!(names(&root), ["ok", "vs", "heir", "fixed", "plain", "rx"]);
    drop(native);
    assert_eq!(names(&root), ["ok", "fixed", "plain"]);
    for (preset, reason) in [
        (
            "vs",
            "o CMake não tem o gerador \"Visual Studio 17 2022\" do preset vs",
        ),
        ("heir", "o CMake não tem o gerador \"Nope\" do preset heir"),
        ("rx", "a condition do preset rx é falsa para o CMake"),
        // C1-27: hidden keeps its own reason, not the generator.
        (
            "base",
            "o preset base é oculto: o CMake não o usa pelo nome",
        ),
    ] {
        assert_eq!(binary_dir(&root, preset).unwrap_err(), reason);
    }
    let dirs: Vec<String> = binary_dirs(&root).into_iter().map(|dir| dir.name).collect();
    assert_eq!(dirs, ["ok", "fixed", "plain"]);
    assert!(matches!(
        default_preset(&root),
        PresetDefault::Several("CMakePresets.json", names) if names == ["ok", "fixed", "plain"]
    ));
}

/// C1-26 against the real `CMake` 4.4.5, the cases measured on 2026-11-10:
/// what the IDE leaves undecided (`matches`, `notMatches`, and an error
/// after them) the `CMake` decides, and no preset carries the note. The
/// regular expression is compiled only when `CMake` reaches it.
#[test]
fn the_cmake_decides_the_conditions_the_ide_does_not() {
    type Case<'a> = (&'a str, String, Result<Option<&'a str>, &'a str>);
    let yes = r#"{"type":"matches","string":"a1","regex":"^a1$"}"#;
    let no = r#"{"type":"matches","string":"a1","regex":"^a\\d$"}"#;
    let not_yes = r#"{"type":"notMatches","string":"a1","regex":"^b$"}"#;
    let not_no = r#"{"type":"notMatches","string":"a1","regex":"^a1$"}"#;
    let bad = r#"{"type":"equals","lhs":"${nope}","rhs":"x"}"#;
    let bad_regex = r#"{"type":"matches","string":"a1","regex":"("}"#;
    let host = r#"{"type":"matches","string":"${hostSystemName}","regex":"^${hostSystemName}$"}"#;
    let any = |forms: &[&str]| format!(r#"{{"type":"anyOf","conditions":[{}]}}"#, forms.join(","));
    let all = |forms: &[&str]| format!(r#"{{"type":"allOf","conditions":[{}]}}"#, forms.join(","));
    let not = |form: &str| format!(r#"{{"type":"not","condition":{form}}}"#);
    let invalid = r#"o CMake recusa os presets: Invalid configure preset: "c": Invalid condition"#;
    let false_for_cmake = "a condition do preset c é falsa para o CMake";
    // (case, condition of `c`, its reason when out of the list, or the refusal)
    let cases: Vec<Case> = vec![
        ("any-yes-bad", any(&[yes, bad]), Ok(None)),
        ("any-no-bad", any(&[no, bad]), Err(invalid)),
        ("all-no-bad", all(&[no, bad]), Ok(Some(false_for_cmake))),
        ("all-yes-bad", all(&[yes, bad]), Err(invalid)),
        ("any-not-yes-bad", any(&[not_yes, bad]), Ok(None)),
        (
            "all-not-no-bad",
            all(&[not_no, bad]),
            Ok(Some(false_for_cmake)),
        ),
        ("not-no", not(no), Ok(None)),
        ("not-yes", not(yes), Ok(Some(false_for_cmake))),
        ("bad-regex", bad_regex.to_owned(), Err(invalid)),
        ("any-true-bad-regex", any(&["true", bad_regex]), Ok(None)),
        (
            "all-false-bad-regex",
            all(&["false", bad_regex]),
            Ok(Some("a condition do preset c é falsa: o CMake não o usa")),
        ),
        ("any-yes-bad-regex", any(&[yes, bad_regex]), Ok(None)),
        (
            "macro-in-string",
            r#"{"type":"matches","string":"${nope}","regex":"^a$"}"#.to_owned(),
            Err(invalid),
        ),
        (
            "any-yes-macro-in-regex",
            any(&[yes, r#"{"type":"matches","string":"a","regex":"${nope}"}"#]),
            Ok(None),
        ),
        ("macro-in-regex", host.to_owned(), Ok(None)),
    ];
    for (name, condition, expected) in cases {
        let root = project(
            &format!("decides-{name}"),
            &format!(
                r#"{{"version":6,"configurePresets":[{{"name":"ok","binaryDir":"o"}},{{"name":"c","binaryDir":"c","condition":{condition}}}]}}"#
            ),
        );
        match expected {
            Err(refusal) => assert_eq!(list_presets(&root).unwrap_err(), refusal, "{name}"),
            Ok(reason) => {
                let listed = list_presets(&root).unwrap();
                assert!(
                    listed.iter().all(|preset| preset.condition_note.is_none()),
                    "{name}: {listed:?}"
                );
                let names: Vec<&str> = listed.iter().map(|preset| preset.name.as_str()).collect();
                match reason {
                    None => assert_eq!(names, ["ok", "c"], "{name}"),
                    Some(reason) => {
                        assert_eq!(names, ["ok"], "{name}");
                        assert_eq!(binary_dir(&root, "c").unwrap_err(), reason, "{name}");
                    }
                }
            }
        }
    }
    // A hidden preset is read too: its invalid expression refuses the file.
    let root = project(
        "decides-hidden",
        &format!(
            r#"{{"version":6,"configurePresets":[{{"name":"ok","binaryDir":"o"}},{{"name":"base","hidden":true,"condition":{bad_regex}}},{{"name":"c","inherits":"base","binaryDir":"c"}}]}}"#
        ),
    );
    assert_eq!(
        list_presets(&root).unwrap_err(),
        r#"o CMake recusa os presets: Invalid configure preset: "base": Invalid condition"#
    );
}

/// A fake `CMake` that writes down each call in `calls` and then does what
/// `body` says.
fn fake_cmake(dir: &Path, calls: &Path, body: &str) -> PathBuf {
    std::fs::create_dir_all(dir).unwrap();
    crate::write_executable(
        dir.join("cmake"),
        format!("#!/bin/sh\necho \"$*\" >> '{}'\n{body}\n", calls.display()),
    )
}

/// Why the presets of `root` are refused, with `program` as the `CMake`.
fn refusal(root: &Path, program: &Path) -> String {
    match load_with(root, Some(program)) {
        Ok(_) => panic!("os presets de {} valeram", root.display()),
        Err(reason) => reason,
    }
}

fn calls(path: &Path) -> Vec<String> {
    std::fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .map(str::to_owned)
        .collect()
}

/// The answer holds while neither the executable nor a presets file the
/// reading opened changes: also an included file, a file that appears, and
/// one that goes.
#[test]
fn the_answer_holds_until_the_cmake_or_a_presets_file_changes() {
    let root = project(
        "cache",
        r#"{"version":6,"include":["more.json"],"configurePresets":[{"name":"p","binaryDir":"o"}]}"#,
    );
    std::fs::write(
        root.join("more.json"),
        r#"{"version":6,"configurePresets":[{"name":"m","binaryDir":"m"}]}"#,
    )
    .unwrap();
    let log = root.join("calls.log");
    let program = fake_cmake(&root.join("bin"), &log, "exec cmake \"$@\"");
    let names = |expected: usize| {
        let (presets, _) = load_with(&root, Some(&program)).unwrap();
        assert_eq!(calls(&log).len(), expected, "{:?}", calls(&log));
        presets.entries.len()
    };
    assert_eq!(names(1), 2);
    assert_eq!(calls(&log), ["--list-presets"]);
    names(1);
    names(1);
    // The included file, with the same length: the text decides.
    std::fs::write(
        root.join("more.json"),
        r#"{"version":6,"configurePresets":[{"name":"n","binaryDir":"m"}]}"#,
    )
    .unwrap();
    names(2);
    names(2);
    // A file that appears at the root.
    std::fs::write(
        root.join("CMakeUserPresets.json"),
        r#"{"version":6,"configurePresets":[{"name":"u","binaryDir":"u"}]}"#,
    )
    .unwrap();
    assert_eq!(names(3), 3);
    // Another executable at the same path.
    fake_cmake(&root.join("bin"), &log, "# outro\nexec cmake \"$@\"");
    names(4);
    names(4);
    // An included file that goes: the `CMake` refuses, and asks again only
    // when something changes.
    std::fs::remove_file(root.join("more.json")).unwrap();
    for expected in [5, 5] {
        let refused = refusal(&root, &program);
        assert!(
            refused.starts_with("o CMake recusa os presets: File not found: "),
            "{refused}"
        );
        assert_eq!(calls(&log).len(), expected);
    }
}

/// No `CMake` that runs: the native reading, as in C1-24.
#[test]
fn without_a_cmake_that_runs_the_native_reading_answers() {
    let root = project(
        "absent",
        r#"{"version":10,"configurePresets":[{"name":"ok","binaryDir":"o"},{"name":"vs","generator":"Visual Studio 17 2022","binaryDir":"vs"}]}"#,
    );
    for program in [
        root.join("não existe"),
        root.clone(),
        root.join("CMakePresets.json"),
    ] {
        let (presets, reading) = load_with(&root, Some(&program)).unwrap();
        assert_eq!(presets.entries.len(), 2);
        assert!(reading.listed("vs"), "{}", program.display());
    }
    let (_, reading) = load_with(&root, None).unwrap();
    assert!(reading.listed("vs"));
}

/// A fake `CMake` that prints `list` and exits with 0.
fn listing(root: &Path, list: &str) -> PathBuf {
    fake_cmake(
        &root.join("bin"),
        &root.join("calls.log"),
        &format!("printf '%s' '{list}'"),
    )
}

/// The lines as `CMake` prints them (measured): the spaces after the name
/// align the column of `- displayName`, quotes in a name are not escaped,
/// and the order is the one of the `CMake`.
#[test]
fn the_list_is_read_as_cmake_prints_it() {
    let root = project(
        "lines",
        r#"{"version":10,"configurePresets":[
          {"name":"a","displayName":"Alpha","binaryDir":"a"},
          {"name":"q\"uote","binaryDir":"q"},
          {"name":"longer name","binaryDir":"l"},
          {"name":"long","binaryDir":"g"},
          {"name":"off","binaryDir":"f","condition":false},
          {"name":"gone","generator":"Ninja","binaryDir":"x"},
          {"name":"h","hidden":true},
          {"name":"x","binaryDir":"x1"},
          {"name":"q","binaryDir":"q1"},
          {"name":"x - y","binaryDir":"x2"}]}"#,
    );
    let program = listing(
        &root,
        "Available configure presets:\n\n  \"longer name\"\n  \"q\"uote\"\n  \"x - y\"\n  \"off\"\n  \"a\"           - Alpha\n",
    );
    let (presets, reading) = load_with(&root, Some(&program)).unwrap();
    let listed: Vec<&str> = presets
        .entries
        .iter()
        .filter(|entry| reading.listed(&entry.name) && entry.name != "h")
        .map(|entry| entry.name.as_str())
        .collect();
    assert_eq!(listed, ["a", "q\"uote", "longer name", "off", "x - y"]);
    // `long` is a prefix of `longer name`, and the `CMake` did not list it.
    assert_eq!(
        reading.unusable("long").unwrap(),
        "o CMake não lista o preset long"
    );
    assert_eq!(
        reading.unusable("gone").unwrap(),
        "o CMake não tem o gerador \"Ninja\" do preset gone"
    );
    // `x` begins the line of `x - y`, which looks like a `displayName`, and
    // `q` the line of `q"uote`, which looks like `"q"` and more.
    for name in ["x", "q"] {
        assert_eq!(
            reading.unusable(name).unwrap(),
            format!("o CMake não lista o preset {name}")
        );
    }
    // The order and the false `condition` are the ones of the `CMake`, in
    // the list as well, and not the order of the file.
    assert_eq!(
        reading.order,
        ["longer name", "q\"uote", "x - y", "off", "a"]
    );
    let names: Vec<String> = list_from(load_with(&root, Some(&program)))
        .unwrap()
        .into_iter()
        .map(|preset| preset.name)
        .collect();
    assert_eq!(names, ["longer name", "q\"uote", "x - y", "off", "a"]);
}

/// The reason of a refusal: the line after "Could not read presets from",
/// else the first line with text, else the exit.
#[test]
fn the_reason_is_the_line_cmake_gives() {
    let root = project(
        "reasons",
        &format!(r#"{{"version":10,"configurePresets":[{OK}]}}"#),
    );
    for (body, reason) in [
        (
            "printf 'CMake Error: Could not read presets from /x:\\nCMakePresets.json:3: Invalid extra field \"q\" in Preset\\n{ \"q\": 1 }\\n  ^\\n' >&2; exit 1",
            "CMakePresets.json:3: Invalid extra field \"q\" in Preset",
        ),
        (
            "printf '\\nCMake Error: Unknown argument --list-presets\\nCMake Error: Run cmake --help\\n' >&2; exit 1",
            "Unknown argument --list-presets",
        ),
        (
            "exit 3",
            "o cmake --list-presets terminou com exit status: 3",
        ),
    ] {
        let program = fake_cmake(&root.join("bin"), &root.join("calls.log"), body);
        assert_eq!(
            refusal(&root, &program),
            format!("o CMake recusa os presets: {reason}")
        );
    }
}

/// The `CMake` lists what the native reading refuses: the IDE cannot read
/// the values it configures with, and says both.
#[test]
fn a_cmake_that_accepts_what_the_ide_cannot_read_is_a_refusal_with_its_reason() {
    let root = project(
        "diverge",
        r#"{"version":10,"configurePresets":[{"name":"x","binaryDir":"${nope}"}]}"#,
    );
    let program = listing(&root, "Available configure presets:\n\n  \"x\"\n");
    assert_eq!(
        refusal(&root, &program),
        "o CMake aceita os presets, mas a IDE não os lê: macro inválida no preset de configure \
         \"x\": macro desconhecida: ${nope}"
    );
}

/// Without presets files the `CMake` is not asked.
#[test]
fn without_presets_files_no_cmake_is_asked() {
    let root = project("none", "");
    std::fs::remove_file(root.join("CMakePresets.json")).unwrap();
    let log = root.join("calls.log");
    let program = fake_cmake(&root.join("bin"), &log, "exit 1");
    let (presets, _) = load_with(&root, Some(&program)).unwrap();
    assert!(presets.entries.is_empty());
    assert!(calls(&log).is_empty());
}

/// A presets file that exists and is not a regular file, a folder or a FIFO,
/// at the root or included (INF-18): the `CMake` would wait on the FIFO, and
/// the core with it, so it is not asked; the refusal is the native one, with
/// the path.
#[test]
fn a_presets_file_that_is_not_a_regular_file_asks_no_cmake() {
    for file in ["CMakePresets.json", "more.json"] {
        for fifo in [false, true] {
            let root = project(
                &format!("irregular-{file}-{fifo}"),
                r#"{"version":6,"include":["more.json"],"configurePresets":[{"name":"p","binaryDir":"o"}]}"#,
            );
            std::fs::write(root.join("more.json"), r#"{"version":6}"#).unwrap();
            let path = root.join(file);
            std::fs::remove_file(&path).unwrap();
            if !fifo {
                std::fs::create_dir(&path).unwrap();
            } else if !crate::fsops::fifo(&path) {
                continue;
            }
            let log = root.join("calls.log");
            let program = fake_cmake(&root.join("bin"), &log, "exit 1");
            let (project, cmake) = (root.clone(), program.clone());
            let refused = crate::fsops::unblocked(&path, move || refusal(&project, &cmake));
            assert_eq!(refused, format!("{file} não é um arquivo regular"));
            assert!(calls(&log).is_empty(), "{:?}", calls(&log));
        }
    }
}

/// The `CMake` that answers is the one the default kit pins; another kit, or
/// no pin, leaves it to the `PATH` of the core.
#[test]
fn the_default_kit_pins_the_cmake_that_answers() {
    use kinein_protocol::ToolchainRole;

    let root = project(
        "kit",
        &format!(r#"{{"version":10,"configurePresets":[{OK}]}}"#),
    );
    assert_eq!(pinned_program(&root, "", ToolchainRole::Cmake), None);
    let pin = |kit: &str| {
        let selections = [("cmake".to_owned(), "cmake".to_owned())].into();
        let kits = [(
            kit.to_owned(),
            Kit {
                selections,
                ..Kit::default()
            },
        )]
        .into();
        save(&root, &kits).unwrap();
    };
    pin("ok");
    assert_eq!(pinned_program(&root, "", ToolchainRole::Cmake), None);
    pin("");
    let pinned = pinned_program(&root, "", ToolchainRole::Cmake).unwrap();
    assert!(
        pinned.is_file() && pinned.ends_with("cmake"),
        "{}",
        pinned.display()
    );
    assert_eq!(super::program(&root), Some(pinned.clone()));
    // The pin wins over the `PATH`; without it, the `PATH` answers.
    let bin = root.join("bin");
    std::fs::create_dir_all(&bin).unwrap();
    let path = crate::write_executable(bin.join("cmake"), "#!/bin/sh\n");
    assert_eq!(super::program_on(&root, bin.clone().into()), Some(pinned));
    pin("ok");
    assert_eq!(super::program_on(&root, bin.into()), Some(path));
    assert_eq!(super::program_on(&root, "".into()), None);
}

/// Two executables with the same text, length and date, at different paths:
/// the answer of one does not hold for the other.
#[test]
fn each_executable_keeps_a_separate_answer() {
    let root = project(
        "program",
        r#"{"version":10,"configurePresets":[{"name":"a","binaryDir":"a"},{"name":"b","binaryDir":"b"}]}"#,
    );
    let log = root.join("calls.log");
    // The same script lists `a` from one folder and `b` from the other.
    let body =
        "case \"$0\" in\n  */one/*) printf '  \"a\"\\n' ;;\n  *) printf '  \"b\"\\n' ;;\nesac";
    let one = fake_cmake(&root.join("one"), &log, body);
    let two = fake_cmake(&root.join("two"), &log, body);
    let date = std::fs::metadata(&one).unwrap().modified().unwrap();
    std::fs::File::options()
        .write(true)
        .open(&two)
        .unwrap()
        .set_modified(date)
        .unwrap();
    let listed = |program: &Path| {
        let (presets, reading) = load_with(&root, Some(program)).unwrap();
        presets
            .entries
            .iter()
            .filter(|entry| reading.listed(&entry.name))
            .map(|entry| entry.name.clone())
            .collect::<Vec<_>>()
    };
    assert_eq!(listed(&one), ["a"]);
    assert_eq!(listed(&two), ["b"]);
    assert_eq!(listed(&one), ["a"]);
    assert_eq!(calls(&log).len(), 3);
}

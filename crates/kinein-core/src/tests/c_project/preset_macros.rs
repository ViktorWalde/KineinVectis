//! C1-24: as macros de todos os presets como o `CMake` as lê (consolidação C
//! §17.4), com o `cmake` real de juiz.
//!
//! Para cada arquivo de presets, o `cmake --list-presets` real e a IDE têm de
//! concordar: os dois recusam, ou os dois listam os mesmos presets. Os casos
//! repetem as medidas de 2026-11-09 com o `CMake` 4.4.5: cada campo que
//! expande, de cada tipo de preset, com cada macro, oculto ou não; a versão do
//! arquivo; o ciclo no `environment`; a ordem do `$vendor{}`; a `condition`
//! herdada. Até o C1-24 a IDE só expandia o `binaryDir` do preset usado e
//! listava presets de arquivos que o `CMake` recusa. Depois, o `${fileDir}` da
//! versão 12: a IDE configura, com GCC real, na pasta em que o
//! `cmake --preset` configura, e recusa sem job o arquivo que o `CMake` recusa.

use super::{Project, environment::project};
use kinein_protocol::JsonRpcRequest;
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, Instant},
};

const LISTS: &str = "cmake_minimum_required(VERSION 3.24)\nproject(macros LANGUAGES C)\nadd_executable(macros src/main.c)\n";

/// O que o `cmake --list-presets` real diz: os nomes, ou a recusa.
fn cmake_listing(root: &Path) -> Result<Vec<String>, String> {
    let output = Command::new("cmake")
        .arg("--list-presets")
        .current_dir(root)
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&output.stdout).into_owned();
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into_owned());
    }
    Ok(text
        .lines()
        .filter_map(|line| line.strip_prefix("  \""))
        .filter_map(|line| line.split('"').next())
        .map(str::to_owned)
        .collect())
}

/// A leitura NATIVA: desde o C1-25 a lista pública pergunta ao `CMake`, e a
/// comparação seria a do `CMake` consigo mesmo.
fn ide_listing(root: &Path) -> Result<Vec<String>, String> {
    crate::cmake_presets::native_list_presets(root)
        .map(|presets| presets.into_iter().map(|preset| preset.name).collect())
}

/// Um caso: o nome, o `CMakePresets.json` e os outros arquivos.
struct Case {
    name: String,
    presets: String,
    others: Vec<(&'static str, String)>,
}

/// Os campos que expandem, de cada tipo, com `@` no lugar do texto.
const FIELDS: &[(&str, &str)] = &[
    ("configurePresets", r#""binaryDir":"${sourceDir}/@""#),
    ("configurePresets", r#""installDir":"@""#),
    ("configurePresets", r#""toolchainFile":"@""#),
    ("configurePresets", r#""graphviz":"@""#),
    (
        "configurePresets",
        r#""cacheVariables":{"B":"@","A":{"type":"STRING","value":"@"}}"#,
    ),
    ("configurePresets", r#""environment":{"A":"@"}"#),
    (
        "configurePresets",
        r#""condition":{"type":"equals","lhs":"@","rhs":"a"}"#,
    ),
    (
        "configurePresets",
        r#""condition":{"type":"anyOf","conditions":[{"type":"inList","string":"a","list":["@","a"]}]}"#,
    ),
    (
        "configurePresets",
        r#""displayName":"@","description":"@","cmakeExecutable":"@""#,
    ),
    ("buildPresets", r#""environment":{"A":"@"},"targets":["@"]"#),
    (
        "buildPresets",
        r#""nativeToolOptions":["@"],"configuration":"@""#,
    ),
    (
        "buildPresets",
        r#""condition":{"type":"notEquals","lhs":"a","rhs":"@"}"#,
    ),
    (
        "testPresets",
        r#""overwriteConfigurationFile":["@"],"output":{"outputLogFile":"@"}"#,
    ),
    (
        "testPresets",
        r#""filter":{"include":{"name":"@","label":"@"},"exclude":{"fixtures":{"setup":"@"}}}"#,
    ),
    (
        "testPresets",
        r#""execution":{"resourceSpecFile":"@"},"configuration":"@""#,
    ),
    (
        "packagePresets",
        r#""variables":{"A":"@"},"packageName":"@","vendorName":"@""#,
    ),
    (
        "packagePresets",
        r#""configFile":"@","packageDirectory":"@","generators":["@"]"#,
    ),
];

/// As macros de cada caso: inválidas, ignoradas e válidas.
const MACROS: &[&str] = &[
    "${nope}",
    "x${presetName",
    "$env{}",
    "$penv{}",
    "$vendor{acme}",
    "${presetName}",
    "${generator}",
    "a$b",
    "${dollar}$env{KV_C124_UNSET}",
];

fn generated() -> Vec<Case> {
    let mut cases = Vec::new();
    for (index, (kind, field)) in FIELDS.iter().enumerate() {
        for (number, text) in MACROS.iter().enumerate() {
            for hidden in [false, true] {
                let body = field.replace('@', text);
                let ok = r#"{"name":"ok","generator":"Ninja","binaryDir":"${sourceDir}/b/ok"}"#;
                let presets = if *kind == "configurePresets" {
                    let dir = if body.contains("binaryDir") {
                        ""
                    } else {
                        r#","binaryDir":"${sourceDir}/b/x""#
                    };
                    format!(
                        r#"{{"version":10,"configurePresets":[{ok},{{"name":"x","hidden":{hidden},"generator":"Ninja"{dir},{body}}}]}}"#
                    )
                } else {
                    let link = if hidden {
                        ""
                    } else {
                        r#","configurePreset":"ok""#
                    };
                    format!(
                        r#"{{"version":10,"configurePresets":[{ok}],"{kind}":[{{"name":"x","hidden":{hidden}{link},{body}}}]}}"#
                    )
                };
                cases.push(Case {
                    name: format!("campo {index} macro {number} oculto {hidden}"),
                    presets,
                    others: Vec::new(),
                });
            }
        }
    }
    cases
}

/// Casos de um arquivo só: o `environment`, a ordem do `$vendor{}` e a
/// `condition`.
const SINGLE_FILE: &[(&str, &str)] = &[
    (
        "ciclo",
        r#"{"version":10,"configurePresets":[{"name":"h","hidden":true,"environment":{"A":"$env{B}","B":"$env{A}"}}]}"#,
    ),
    (
        "auto-referência",
        r#"{"version":10,"configurePresets":[{"name":"h","hidden":true,"environment":{"A":"x$env{A}"}}]}"#,
    ),
    (
        "penv da própria",
        r#"{"version":10,"configurePresets":[{"name":"h","generator":"Ninja","binaryDir":"o","environment":{"PATH":"/x:$penv{PATH}"}}]}"#,
    ),
    (
        "ciclo herdado",
        r#"{"version":10,"configurePresets":[{"name":"base","hidden":true,"environment":{"A":"$env{B}"}},
        {"name":"kid","inherits":"base","generator":"Ninja","binaryDir":"k","environment":{"B":"$env{A}"}}]}"#,
    ),
    (
        "ciclo do build",
        r#"{"version":10,"configurePresets":[{"name":"ok","generator":"Ninja","binaryDir":"o","environment":{"A":"$env{B}"}}],
        "buildPresets":[{"name":"x","configurePreset":"ok","environment":{"B":"$env{A}"}}]}"#,
    ),
    (
        "build sem herdar",
        r#"{"version":10,"configurePresets":[{"name":"ok","generator":"Ninja","binaryDir":"o","environment":{"A":"$env{B}"}}],
        "buildPresets":[{"name":"x","configurePreset":"ok","inheritConfigureEnvironment":false,"environment":{"B":"$env{A}"}}]}"#,
    ),
    (
        "build herda inválida",
        r#"{"version":10,"configurePresets":[{"name":"ok","generator":"Ninja","binaryDir":"o"}],
        "buildPresets":[{"name":"bx","hidden":true,"environment":{"X":"${nope}"}},{"name":"x","inherits":"bx","configurePreset":"ok"}]}"#,
    ),
    (
        "vendor e inválida",
        r#"{"version":10,"configurePresets":[{"name":"h","hidden":true,"cacheVariables":{"A":"$vendor{x}${nope}"}}]}"#,
    ),
    (
        "inválida e vendor",
        r#"{"version":10,"configurePresets":[{"name":"h","hidden":true,"cacheVariables":{"A":"${nope}$vendor{x}"}}]}"#,
    ),
    (
        "vendor antes",
        r#"{"version":10,"configurePresets":[{"name":"h","hidden":true,"binaryDir":"$vendor{x}","cacheVariables":{"A":"${nope}"}}]}"#,
    ),
    (
        "vendor depois",
        r#"{"version":10,"configurePresets":[{"name":"h","hidden":true,"binaryDir":"${nope}","cacheVariables":{"A":"$vendor{x}"}}]}"#,
    ),
    (
        "chaves em ordem",
        r#"{"version":10,"configurePresets":[{"name":"h","hidden":true,"environment":{"B":"$vendor{x}","A":"${nope}"}}]}"#,
    ),
    (
        "env antes da condition",
        r#"{"version":10,"configurePresets":[{"name":"h","hidden":true,"environment":{"A":"$vendor{x}"},
        "condition":{"type":"equals","lhs":"${nope}","rhs":"a"}}]}"#,
    ),
    (
        "vendor herdado",
        r#"{"version":10,"configurePresets":[{"name":"base","hidden":true,"cacheVariables":{"V":"$vendor{acme}"}},
        {"name":"kid","inherits":"base","generator":"Ninja","binaryDir":"k"},{"name":"ok","generator":"Ninja","binaryDir":"o"}]}"#,
    ),
    (
        "condition herdada",
        r#"{"version":10,"configurePresets":[{"name":"off","hidden":true,"condition":false},
        {"name":"kid","inherits":"off","generator":"Ninja","binaryDir":"a"},
        {"name":"over","inherits":"off","generator":"Ninja","binaryDir":"b","condition":true},
        {"name":"undo","inherits":"off","generator":"Ninja","binaryDir":"c","condition":null}]}"#,
    ),
    (
        "condition curta",
        r#"{"version":10,"configurePresets":[
        {"name":"any","generator":"Ninja","binaryDir":"e","condition":{"type":"anyOf","conditions":[true,{"type":"equals","lhs":"${nope}","rhs":"a"}]}},
        {"name":"all","generator":"Ninja","binaryDir":"f","condition":{"type":"allOf","conditions":[false,{"type":"equals","lhs":"${nope}","rhs":"a"}]}},
        {"name":"env","generator":"Ninja","binaryDir":"d","environment":{"K":"sim"},"condition":{"type":"equals","lhs":"$env{K}","rhs":"sim"}},
        {"name":"not-in","generator":"Ninja","binaryDir":"j","condition":{"type":"notInList","string":"${presetName}","list":["not-in"]}}]}"#,
    ),
    (
        "condition falsa expande",
        r#"{"version":10,"configurePresets":[{"name":"c","generator":"Ninja","binaryDir":"o","condition":false,
        "cacheVariables":{"X":"${nope}"}}]}"#,
    ),
    // O `${generator}` do configure preset chega ao build e ao test, herdado
    // também, e nunca ao package: o `anyOf` só para antes do `${nope}` quando
    // o gerador é o esperado.
    (
        "generator do build herdado",
        r#"{"version":10,"configurePresets":[{"name":"ok","generator":"Ninja","binaryDir":"o"},{"name":"heir","inherits":"ok","binaryDir":"h"}],
        "buildPresets":[{"name":"x","configurePreset":"heir","condition":{"type":"anyOf","conditions":[
        {"type":"equals","lhs":"${generator}","rhs":"Ninja"},{"type":"equals","lhs":"${nope}","rhs":"a"}]}}]}"#,
    ),
    (
        "generator do test",
        r#"{"version":10,"configurePresets":[{"name":"ok","generator":"Ninja","binaryDir":"o"}],
        "testPresets":[{"name":"x","configurePreset":"ok","condition":{"type":"anyOf","conditions":[
        {"type":"equals","lhs":"${generator}","rhs":"Ninja"},{"type":"equals","lhs":"${nope}","rhs":"a"}]}}]}"#,
    ),
    (
        "generator do package",
        r#"{"version":10,"configurePresets":[{"name":"ok","generator":"Ninja","binaryDir":"o"}],
        "packagePresets":[{"name":"x","configurePreset":"ok","condition":{"type":"anyOf","conditions":[
        {"type":"equals","lhs":"${generator}","rhs":"Ninja"},{"type":"equals","lhs":"${nope}","rhs":"a"}]}}]}"#,
    ),
    (
        "generator vazio do package",
        r#"{"version":10,"configurePresets":[{"name":"ok","generator":"Ninja","binaryDir":"o"}],
        "packagePresets":[{"name":"x","configurePreset":"ok","condition":{"type":"anyOf","conditions":[
        {"type":"equals","lhs":"${generator}","rhs":""},{"type":"equals","lhs":"${nope}","rhs":"a"}]}}]}"#,
    ),
];

fn targeted() -> Vec<Case> {
    let one = |name: &str, presets: &str| Case {
        name: name.to_owned(),
        presets: presets.to_owned(),
        others: Vec::new(),
    };
    let mut cases = Vec::new();
    for version in 2..=5 {
        for macro_ in [
            "${hostSystemName}",
            "${fileDir}",
            "${pathListSep}",
            "$penv{HOME}",
        ] {
            cases.push(one(
                &format!("versão {version} {macro_}"),
                &format!(
                    r#"{{"version":{version},"configurePresets":[{{"name":"ok","generator":"Ninja","binaryDir":"o"}},
                      {{"name":"h","hidden":true,"cacheVariables":{{"M":"{macro_}"}}}}]}}"#
                ),
            ));
        }
    }
    let inherited = |parent: u64, child: u64, with_child: bool| Case {
        name: format!("pai {parent} filho {child} {with_child}"),
        presets: format!(
            r#"{{"version":{child},"include":["sub/inc.json"],"configurePresets":[{}]}}"#,
            if with_child {
                r#"{"name":"kid","inherits":"base","generator":"Ninja","binaryDir":"k"}"#
            } else {
                ""
            }
        ),
        others: vec![(
            "sub/inc.json",
            format!(
                r#"{{"version":{parent},"configurePresets":[{{"name":"base","hidden":true,"cacheVariables":{{"M":"${{pathListSep}}"}}}}]}}"#
            ),
        )],
    };
    cases.extend([
        inherited(5, 4, true),
        inherited(4, 5, true),
        inherited(5, 4, false),
    ]);
    cases.extend(SINGLE_FILE.iter().map(|(name, presets)| one(name, presets)));
    cases.push(Case {
        name: "usuário".to_owned(),
        presets: r#"{"version":10,"configurePresets":[{"name":"ok","generator":"Ninja","binaryDir":"o"}]}"#.to_owned(),
        others: vec![(
            "CMakeUserPresets.json",
            r#"{"version":10,"configurePresets":[{"name":"u","hidden":true,"cacheVariables":{"X":"${nope}"}}]}"#.to_owned(),
        )],
    });
    cases
}

#[test]
fn c1_24_the_ide_refuses_and_lists_exactly_what_cmake_does() {
    let parent = std::env::temp_dir().join(format!("kinein C1-24 ação {}", std::process::id()));
    let cases: Vec<Case> = generated().into_iter().chain(targeted()).collect();
    let (mut refused, mut listed) = (0, 0);
    for (index, case) in cases.iter().enumerate() {
        let root = parent.join(format!("caso {index}"));
        drop(fs::remove_dir_all(&root));
        fs::create_dir_all(&root).unwrap();
        fs::write(
            root.join("CMakeLists.txt"),
            "cmake_minimum_required(VERSION 3.24)\nproject(x NONE)\n",
        )
        .unwrap();
        fs::write(root.join("CMakePresets.json"), &case.presets).unwrap();
        for (relative, body) in &case.others {
            let path = root.join(relative);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, body).unwrap();
        }
        match (cmake_listing(&root), ide_listing(&root)) {
            (Ok(cmake), Ok(ide)) => {
                assert_eq!(ide, cmake, "{}: {}", case.name, case.presets);
                listed += 1;
            }
            (Err(_), Err(_)) => refused += 1,
            (cmake, ide) => panic!(
                "{}: {}\nCMake: {cmake:?}\nIDE: {ide:?}",
                case.name, case.presets
            ),
        }
    }
    // As duas saídas aparecem: o teste não passa só recusando ou só listando.
    assert!(
        refused > 100 && listed > 100,
        "{refused} recusados, {listed} listados"
    );
    drop(fs::remove_dir_all(&parent));
}

/// `cmake --preset` real; a pasta é a que o `CMake` diz ter escrito.
fn cli_configure(root: &Path, preset: &str) -> PathBuf {
    let output = Command::new("cmake")
        .arg("--preset")
        .arg(preset)
        .current_dir(root)
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .find_map(|line| line.strip_prefix("-- Build files have been written to: "))
        .map(PathBuf::from)
        .unwrap()
}

fn fixture() -> Project {
    let mut p = project("macros de presets");
    fs::create_dir_all(p.root.join("src")).unwrap();
    fs::create_dir_all(p.root.join("cmake")).unwrap();
    fs::write(p.root.join("src/main.c"), "int main(void) { return 0; }\n").unwrap();
    fs::write(p.root.join("CMakeLists.txt"), LISTS).unwrap();
    let presets = json!({"version":12,"include":["cmake/base.json"],"configurePresets":[
        {"name":"kid","inherits":"base"},
        {"name":"named","inherits":"base","binaryDir":"${fileDir}/saídas-${presetName}"}]});
    fs::write(p.root.join("CMakePresets.json"), presets.to_string()).unwrap();
    let base = json!({"version":12,"configurePresets":[{"name":"base","hidden":true,"generator":"Ninja",
        "binaryDir":"${fileDir}/saídas",
        "cacheVariables":{"CMAKE_EXPORT_COMPILE_COMMANDS":"ON","CMAKE_C_COMPILER":"gcc"}}]});
    fs::write(p.root.join("cmake/base.json"), base.to_string()).unwrap();
    p.call("workspace.open", json!({"path":p.root}));
    p.call("workspace.trust", json!({}));
    p.wait("event.index.finished", |_| true);
    p
}

/// O configure da IDE: o início, as linhas e o sucesso.
fn configure(p: &mut Project, preset: &str) -> (Value, bool) {
    let id = p.call("cmake.configure", json!({"preset":preset}))["jobId"].clone();
    let deadline = Instant::now() + Duration::from_secs(60);
    let mut start = Value::Null;
    loop {
        let left = deadline
            .checked_duration_since(Instant::now())
            .expect("configure deadline");
        let event = p.events.recv_timeout(left).unwrap();
        p.core.observe_notification(&event);
        let params = event.params.unwrap_or_default();
        if params["jobId"] != id {
            continue;
        }
        match event.method.as_str() {
            "event.cmake.started" => start = params,
            "event.cmake.finished" => return (start, params["success"] == true),
            _ => {}
        }
    }
}

fn compiler_of(directory: &Path) -> String {
    let cdb: Value =
        serde_json::from_slice(&fs::read(directory.join("compile_commands.json")).unwrap())
            .unwrap();
    cdb[0]["command"]
        .as_str()
        .unwrap()
        .split_whitespace()
        .next()
        .unwrap()
        .to_owned()
}

#[test]
fn c1_24_file_dir_of_version_12_configures_where_cmake_does() {
    let mut p = fixture();
    // O `CMake` real: a pasta do arquivo incluído, e a do preset em uso
    // quando o texto tem `${presetName}`.
    let cli = cli_configure(&p.root, "kid");
    assert_eq!(cli, p.root.join("cmake/saídas"));
    assert_eq!(cli_configure(&p.root, "named"), p.root.join("saídas-named"));
    fs::remove_dir_all(&cli).unwrap();

    let (start, success) = configure(&mut p, "kid");
    assert!(success, "{start}");
    assert_eq!(start["buildDir"], "cmake/saídas", "{start}");
    assert!(compiler_of(&cli).ends_with("gcc"), "{}", compiler_of(&cli));
    let status = p.call("cmake.status", json!({}));
    assert_eq!(
        status["configurations"],
        json!([
            {"preset":"kid","buildDir":"cmake/saídas","configured":true},
            {"preset":"named","buildDir":"saídas-named","configured":true}
        ]),
        "{status}"
    );

    // Um build preset oculto com macro inválida: o `CMake` recusa o arquivo
    // inteiro, e a IDE também, sem job.
    let broken = json!({"version":12,"include":["cmake/base.json"],"configurePresets":[
        {"name":"kid","inherits":"base"}],
        "buildPresets":[{"name":"all","hidden":true,"targets":["${nope}"]}]});
    fs::write(p.root.join("CMakePresets.json"), broken.to_string()).unwrap();
    assert!(cmake_listing(&p.root).is_err());
    let refused = p
        .core
        .handle_request(&JsonRpcRequest::new(
            2_i64,
            "cmake.configure",
            Some(json!({"preset":"kid"})),
        ))
        .response()
        .clone();
    // O motivo é o do `CMake` do projeto (C1-25).
    let message = refused.error.expect("presets recusados").message;
    assert!(
        message.contains("o CMake recusa os presets: Invalid macro expansion in \"all\""),
        "{message}"
    );
    let deadline = Instant::now() + Duration::from_millis(300);
    while let Some(left) = deadline.checked_duration_since(Instant::now()) {
        if let Ok(event) = p.events.recv_timeout(left) {
            assert_ne!(event.method, "event.cmake.started", "a recusa não roda job");
        }
    }
    let status = p.call("cmake.status", json!({}));
    assert!(status.get("configurations").is_none(), "{status}");
}

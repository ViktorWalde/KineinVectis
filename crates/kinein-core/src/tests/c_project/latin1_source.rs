//! INF-09 (62 §3.4): um fonte em Latin-1 com um aviso numa linha acentuada,
//! com `CMake`, Ninja e GCC reais.
//!
//! O GCC 16 copia crua no diagnóstico a linha do fonte. Até o 0.217.0, o
//! leitor do job parava nessa linha e fechava o pipe: o Ninja morria por
//! `SIGPIPE` na linha seguinte, o build falhava e o executável não existia. O
//! projeto desliga o `COMPILE_WARNING_AS_ERROR` que o perfil Estrito liga, e o
//! aviso fica aviso. O teste que imprime esses bytes derrubava o
//! `ctest --output-on-failure` do mesmo jeito, sem o resumo.

use super::configurations::{build, configure, job};
use super::environment::project;
use serde_json::json;
use std::fs;

const LISTS: &str = r"cmake_minimum_required(VERSION 3.24)
project(latin LANGUAGES C)
enable_testing()
add_executable(latin src/latin.c src/after.c)
target_compile_options(latin PRIVATE -Wall)
set_target_properties(latin PROPERTIES COMPILE_WARNING_AS_ERROR OFF)
add_test(NAME kv_latin COMMAND latin)
";

const PRESETS: &str = r#"{"version":6,"configurePresets":[
  {"name":"dev","generator":"Ninja","binaryDir":"${sourceDir}/build/dev",
   "cacheVariables":{"CMAKE_C_COMPILER":"gcc"}}]}"#;

/// O comentário em Latin-1 fica na linha do aviso, e o teste imprime um byte
/// que não é UTF-8, muita saída depois e falha, para o `ctest` mostrar tudo.
const LATIN: &[u8] = b"#include <stdio.h>\n\
int after(void);\n\
int main(void) {\n\
    int unused; /* a\xe7\xe3o */\n\
    printf(\"antes\\n\");\n\
    printf(\"bin\\xffrio\\n\");\n\
    for (int i = 0; i < 20000; i++) printf(\"l%d\\n\", i);\n\
    printf(\"fim\\n\");\n\
    return after();\n\
}\n";

#[test]
fn inf_09_a_latin1_source_builds_and_its_test_output_arrives_whole() {
    let mut p = project("latin1-source");
    fs::create_dir_all(p.root.join("src")).unwrap();
    fs::write(p.root.join("src/latin.c"), LATIN).unwrap();
    for (name, text) in [
        ("src/after.c", "int after(void) { return 1; }\n"),
        ("CMakeLists.txt", LISTS),
        ("CMakePresets.json", PRESETS),
    ] {
        fs::write(p.root.join(name), text).unwrap();
    }
    p.call("workspace.open", json!({"path":p.root}));
    p.call("workspace.trust", json!({}));
    p.wait("event.index.finished", |_| true);
    let (_, lines, success) = configure(&mut p, "dev");
    assert!(success, "{lines:?}");

    let lines = build(&mut p);
    let quoted = lines
        .iter()
        .position(|line| line.contains("int unused; /* a\u{FFFD}\u{FFFD}o */"))
        .unwrap_or_else(|| panic!("a linha citada pelo GCC: {lines:?}"));
    assert!(
        lines[quoted..]
            .iter()
            .any(|line| line.contains("Linking C executable")),
        "{lines:?}"
    );
    let executable = p
        .root
        .join("build/dev")
        .join(format!("latin{}", std::env::consts::EXE_SUFFIX));
    assert!(executable.is_file(), "{lines:?}");

    let id = p.call("test.run", json!({}))["jobId"].clone();
    let (_, lines, success) = job(&mut p, &id, "event.test.started", "event.test.finished");
    assert!(!success, "o teste falha: {lines:?}");
    let invalid = lines
        .iter()
        .position(|line| line.ends_with("bin\u{FFFD}rio"))
        .unwrap_or_else(|| panic!("a linha do byte: {lines:?}"));
    let after: Vec<_> = lines[invalid..].iter().map(String::as_str).collect();
    let counted: Vec<_> = after
        .iter()
        .filter(|line| {
            line.strip_prefix('l')
                .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
        })
        .copied()
        .collect();
    let expected: Vec<_> = (0..20000).map(|i| format!("l{i}")).collect();
    assert!(
        counted == expected,
        "{} linhas depois do byte",
        counted.len()
    );
    assert!(after.contains(&"fim"), "{after:?}");
    assert!(
        after
            .iter()
            .any(|line| line.contains("0% tests passed, 1 tests failed out of 1")),
        "o resumo do ctest: {:?}",
        &after[after.len().saturating_sub(10)..]
    );
}

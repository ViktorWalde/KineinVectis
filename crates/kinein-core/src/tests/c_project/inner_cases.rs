//! C1-28: os casos de dentro dos binários do ctest (consolidação C §20), com
//! `CMake`, GCC, gtest e, quando instalado, Catch2 reais.
//!
//! Até o 0.213.0 todo binário que respondia `--list-tests --verbosity quiet`
//! com uma linha por argumento virava Catch2: um programa que ecoa os
//! argumentos ganhava os casos `--list-tests`, `--verbosity` e `quiet`. O
//! Catch2 se identifica no `--libidentify`, e só então a lista é pedida.
//!
//! C1-29: rodar um caso gtest que falha contava duas falhas, a da linha do
//! caso e a do resumo do fim, e o parametrizado que falha saía com o nome
//! `Some/P.Odd/1, where GetParam() = 2`, que não é o da lista. O `main` deste
//! gtest desliga o tempo, como o `GTEST_PRINT_TIME=0` faria, e só a linha do
//! caso tem o tempo quando a IDE o pede.
//!
//! C1-31: o nome do caso Catch2 ia sem escape como filtro, e o Catch2 3.16.1
//! rodava todos os casos para `~tilde` e nenhum para `a, b [x]`. Cada caso
//! deste Catch2 imprime o próprio nome, e a prova conta o que rodou. A lista
//! do Catch2 3 sai em ordem aleatória, e a IDE pede a da declaração.

use super::configurations::{build, configure};
use super::environment::project;
use crate::test::{TestEvent, frameworks::run_inner_case};
use kinein_protocol::ProjectKind;
use serde_json::json;
use std::{fs, path::Path, sync::Arc};

const ECHOES: &str = "#include <stdio.h>\n\
int main(int argc, char **argv) { for (int i = 1; i < argc; i++) puts(argv[i]); return 0; }\n";

const GTEST: &str = "#include <gtest/gtest.h>\n\
TEST(Math, Adds) { EXPECT_EQ(2, 1 + 1); }\n\
TEST(Math, Fails) { EXPECT_EQ(3, 1 + 1); }\n\
TEST(Math, Skips) { GTEST_SKIP() << \"skipped\"; }\n\
class P : public ::testing::TestWithParam<int> {};\n\
TEST_P(P, Odd) { EXPECT_EQ(GetParam() % 2, 1); }\n\
INSTANTIATE_TEST_SUITE_P(Some, P, ::testing::Values(1, 2));\n\
int main(int argc, char **argv) {\n\
  GTEST_FLAG_SET(print_time, false);\n\
  ::testing::InitGoogleTest(&argc, argv);\n\
  return RUN_ALL_TESTS();\n\
}\n";

/// Os casos do Catch2 na ordem da declaração; o oculto não é listado.
const CATCH2_CASES: [&str; 8] = [
    "vectors can be sized",
    "Math.Fails",
    "a, b [x]",
    "~tilde",
    "back\\slash",
    "exclude:this",
    "Case",
    "case",
];

const CATCH2: &str = r#"#include <catch2/catch_test_macros.hpp>
#include <cstdio>
#define CASE(name, check) TEST_CASE(name) { std::printf("RAN<%s>\n", name); REQUIRE(check); }
CASE("vectors can be sized", 1 == 1)
CASE("Math.Fails", 3 == 1 + 1)
CASE("a, b [x]", true)
CASE("~tilde", true)
CASE("back\\slash", true)
CASE("exclude:this", true)
CASE("Case", true)
CASE("case", true)
TEST_CASE("hidden one", "[.]") { std::printf("RAN<hidden one>\n"); }
"#;

const LISTS: &str = r"cmake_minimum_required(VERSION 3.24)
project(inner LANGUAGES C CXX)
enable_testing()
add_executable(echoes src/echoes.c)
add_test(NAME kv_echoes COMMAND echoes)
find_package(GTest QUIET)
if(GTest_FOUND)
  add_executable(gt src/gt.cpp)
  target_link_libraries(gt GTest::gtest)
  add_test(NAME kv_gtest COMMAND gt)
endif()
find_package(Catch2 3 QUIET)
if(Catch2_FOUND)
  add_executable(c2 src/c2.cpp)
  target_link_libraries(c2 Catch2::Catch2WithMain)
  add_test(NAME kv_catch2 COMMAND c2)
endif()
";

const PRESETS: &str = r#"{"version":6,"configurePresets":[
  {"name":"dev","generator":"Ninja","binaryDir":"${sourceDir}/build/dev",
   "cacheVariables":{"CMAKE_C_COMPILER":"gcc","CMAKE_CXX_COMPILER":"g++",
                     "CMAKE_EXPORT_COMPILE_COMMANDS":"ON"}}]}"#;

/// O que um `run_inner_case` deu: o sucesso e as contagens de aprovados,
/// reprovados e ignorados, os nomes dos casos que a IDE marca e as linhas
/// `RAN<…>` dos casos Catch2 que de fato rodaram.
type Inner = ((bool, u64, u64, u64), Vec<String>, Vec<String>);

fn inner_run(folder: &Path, id: &str) -> Result<Inner, String> {
    let mut names = Vec::new();
    let mut ran = Vec::new();
    let outcome = run_inner_case(
        Path::new("ctest"),
        folder,
        id,
        &Arc::default(),
        &mut |event| match event {
            TestEvent::Case { name, .. } => names.push(name),
            TestEvent::Output { line, .. } if line.starts_with("RAN<") => ran.push(line),
            _ => {}
        },
    )
    .expect("o id tem o separador")
    .map_err(|error| error.to_string())?;
    let counts = (
        outcome.success,
        outcome.passed,
        outcome.failed,
        outcome.ignored,
    );
    Ok((counts, names, ran))
}

#[test]
fn c1_28_c1_29_c1_31_inner_cases_come_from_the_framework_and_run_alone() {
    let mut p = project("inner-cases");
    fs::create_dir_all(p.root.join("src")).unwrap();
    for (name, text) in [
        ("src/echoes.c", ECHOES),
        ("src/gt.cpp", GTEST),
        ("src/c2.cpp", CATCH2),
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
    build(&mut p);
    let folder = p.root.join("build/dev");

    let mut said = Vec::new();
    let listed: Vec<_> = crate::test::discover_tests(
        &p.root,
        ProjectKind::Cmake,
        None,
        &Arc::default(),
        &mut |event| {
            if let TestEvent::Output { line, .. } = event {
                said.push(line);
            }
        },
    )
    .unwrap()
    .into_iter()
    .map(|case| case.id)
    .collect();
    let expected = expected(&listed);
    let mut ids = listed;
    ids.sort();
    // O programa que ecoa os argumentos fica só a linha do ctest.
    assert_eq!(ids, expected, "{said:?}");
    let inner = expected.iter().filter(|id| id.contains("::")).count();
    if inner > 0 {
        assert!(
            said.contains(&format!(
                "{inner} caso(s) de gtest/Catch2 dentro dos binários do ctest"
            )),
            "{said:?}"
        );
    }
    runs_alone(&folder, &ids);
}

/// Os ids que a descoberta tem de dar, em ordem, com o que esta máquina tem;
/// os do Catch2 também na ordem da declaração.
fn expected(listed: &[String]) -> Vec<String> {
    let mut expected = vec!["kv_echoes".to_owned()];
    if listed.iter().any(|id| id == "kv_gtest") {
        expected.extend(
            [
                "kv_gtest",
                "kv_gtest::Math.Adds",
                "kv_gtest::Math.Fails",
                "kv_gtest::Math.Skips",
                "kv_gtest::Some/P.Odd/0",
                "kv_gtest::Some/P.Odd/1",
            ]
            .map(str::to_owned),
        );
    } else {
        eprintln!("NAO PROVADO: gtest ausente nesta maquina");
    }
    if listed.iter().any(|id| id == "kv_catch2") {
        let declared: Vec<_> = CATCH2_CASES
            .iter()
            .map(|case| format!("kv_catch2::{case}"))
            .collect();
        // A ordem da declaração, que o Catch2 3 só dá com `--order decl`.
        let order: Vec<_> = listed
            .iter()
            .filter(|id| id.starts_with("kv_catch2::"))
            .cloned()
            .collect();
        assert_eq!(order, declared);
        expected.push("kv_catch2".to_owned());
        expected.extend(declared);
    } else {
        eprintln!("NAO PROVADO: Catch2 ausente nesta maquina");
    }
    expected.sort_unstable();
    expected
}

/// Cada caso conta uma vez, com o nome que a descoberta deu, e só ele roda.
fn runs_alone(folder: &Path, ids: &[String]) {
    for (id, counts) in [
        ("kv_gtest::Math.Adds", (true, 1, 0, 0)),
        ("kv_gtest::Math.Fails", (false, 0, 1, 0)),
        ("kv_gtest::Math.Skips", (true, 0, 0, 1)),
        ("kv_gtest::Some/P.Odd/1", (false, 0, 1, 0)),
        ("kv_catch2::vectors can be sized", (true, 1, 0, 0)),
        ("kv_catch2::Math.Fails", (false, 0, 1, 0)),
        ("kv_catch2::a, b [x]", (true, 1, 0, 0)),
        ("kv_catch2::~tilde", (true, 1, 0, 0)),
        ("kv_catch2::back\\slash", (true, 1, 0, 0)),
    ] {
        if !ids.iter().any(|known| known == id) {
            continue;
        }
        let case = id.split_once("::").unwrap().1.to_owned();
        let ran = if id.starts_with("kv_catch2::") {
            vec![format!("RAN<{case}>")]
        } else {
            Vec::new()
        };
        assert_eq!(inner_run(folder, id), Ok((counts, vec![case], ran)), "{id}");
    }
    // O que o Catch2 não isola é recusado antes de rodar o binário.
    if ids.iter().any(|id| id == "kv_catch2") {
        assert_eq!(
            inner_run(folder, "kv_catch2::exclude:this"),
            Err(
                "o Catch2 lê o caso `exclude:this` como uma exclusão e rodaria os outros \
                 casos; rode a linha do ctest do binário"
                    .to_owned()
            )
        );
        assert_eq!(
            inner_run(folder, "kv_catch2::case"),
            Err(
                "o Catch2 compara os nomes sem caixa e rodaria também o caso `Case`; \
                 rode a linha do ctest do binário"
                    .to_owned()
            )
        );
    }
}

//! C1-32 e C1-35: os testes do ctest que já são casos e o caso desativado
//! (consolidação C §21), com `CMake`, GCC, gtest e Catch2 reais.
//!
//! Até o 0.214.0 a descoberta abria em casos todo teste cujo binário
//! respondia à listagem: os testes do `gtest_discover_tests` e do
//! `catch_discover_tests`, um por caso, davam cada caso uma vez por caso. O
//! id `teste::caso` era cortado no primeiro `::`, e o teste do ctest
//! `ns::unit`, ou o caso Catch2 `Widget::draw works`, diziam que o ctest não
//! os listava. O caso gtest desativado dizia que passou sem rodar, e a linha
//! do ctest dele saía reprovada.

use super::configurations::{build, configure};
use super::environment::project;
use crate::test::{Selection, TestEvent, frameworks::run_inner_case};
use kinein_protocol::ProjectKind;
use serde_json::json;
use std::{fs, path::Path, sync::Arc};

/// O `main` liga o `brief`, como o `GTEST_BRIEF=1` faria: o caso que passa
/// só tem linha quando a IDE pede `--gtest_brief=0`.
const GTEST: &str = "#include <gtest/gtest.h>\n\
TEST(Math, Adds) { EXPECT_EQ(2, 1 + 1); }\n\
TEST(Math, Fails) { EXPECT_EQ(3, 1 + 1); }\n\
TEST(Text, DISABLED_Off) { FAIL() << \"desativado\"; }\n\
int main(int argc, char **argv) {\n\
  GTEST_FLAG_SET(brief, true);\n\
  ::testing::InitGoogleTest(&argc, argv);\n\
  return RUN_ALL_TESTS();\n\
}\n";

const CATCH2: &str = r#"#include <catch2/catch_test_macros.hpp>
#include <cstdio>
#define CASE(name) TEST_CASE(name) { std::printf("RAN<%s>\n", name); }
CASE("Widget::draw works")
CASE("plain")
CASE("x::y")
CASE("y")
"#;

const LISTS: &str = r#"cmake_minimum_required(VERSION 3.24)
project(discovered LANGUAGES CXX)
enable_testing()
find_package(GTest QUIET)
if(GTest_FOUND)
  include(GoogleTest)
  add_executable(gt src/gt.cpp)
  target_link_libraries(gt GTest::gtest)
  gtest_discover_tests(gt)
  add_test(NAME ns::unit COMMAND gt)
  add_test(NAME gt_math COMMAND gt --gtest_filter=Math.*)
  add_test(NAME gt_env COMMAND gt)
  set_tests_properties(gt_env PROPERTIES ENVIRONMENT "GTEST_FILTER=Math.*")
  add_test(NAME gt_color COMMAND gt --gtest_color=yes)
  add_test(NAME gt_color::Math.Adds COMMAND gt --gtest_filter=Math.Adds)
endif()
find_package(Catch2 3 QUIET)
if(Catch2_FOUND)
  include(Catch)
  add_executable(c2 src/c2.cpp)
  target_link_libraries(c2 Catch2::Catch2WithMain)
  catch_discover_tests(c2)
  add_test(NAME dup COMMAND c2)
  add_test(NAME dup::x COMMAND c2)
  add_test(NAME c2_all COMMAND c2 -s -r compact)
  add_test(NAME c2_tag COMMAND c2 plain)
  add_test(NAME c2_unknown COMMAND c2 --mystery)
endif()
"#;

const PRESETS: &str = r#"{"version":6,"configurePresets":[
  {"name":"dev","generator":"Ninja","binaryDir":"${sourceDir}/build/dev",
   "cacheVariables":{"CMAKE_CXX_COMPILER":"g++","CMAKE_EXPORT_COMPILE_COMMANDS":"ON"}}]}"#;

/// O desfecho e as contagens de aprovados, reprovados e ignorados.
type Counts = (bool, u64, u64, u64);

/// O que rodar um id deu: as contagens, os casos marcados, as linhas
/// `RAN<…>` do Catch2 e as linhas que a IDE escreveu.
type Ran = (Counts, Vec<String>, Vec<String>, Vec<String>);

fn record(
    names: &mut Vec<String>,
    ran: &mut Vec<String>,
    said: &mut Vec<String>,
    event: TestEvent,
) {
    match event {
        TestEvent::Case { name, .. } => names.push(name),
        TestEvent::Output { line, .. } if line.starts_with("RAN<") => ran.push(line),
        TestEvent::Output { line, .. } if line.starts_with("o gtest ") => said.push(line),
        _ => {}
    }
}

/// Um caso de dentro, pelo binário; `None` quando o id é uma linha do ctest.
fn inner(folder: &Path, id: &str) -> Option<Result<Ran, String>> {
    let (mut names, mut ran, mut said) = (Vec::new(), Vec::new(), Vec::new());
    let outcome = run_inner_case(
        Path::new("ctest"),
        folder,
        id,
        &Arc::default(),
        &mut |event| {
            record(&mut names, &mut ran, &mut said, event);
        },
    )?;
    Some(outcome.map_err(|error| error.to_string()).map(|outcome| {
        let counts = (
            outcome.success,
            outcome.passed,
            outcome.failed,
            outcome.ignored,
        );
        (counts, names, ran, said)
    }))
}

/// Pelo caminho do `test.run`: o caso pelo binário, a linha pelo ctest.
fn run(root: &Path, selection: Selection<'_>) -> (Counts, Vec<String>) {
    let (mut names, mut ran, mut said) = (Vec::new(), Vec::new(), Vec::new());
    let outcome = crate::test::run_tests(
        root,
        ProjectKind::Cmake,
        selection,
        None,
        &Arc::default(),
        &mut |event| record(&mut names, &mut ran, &mut said, event),
    )
    .unwrap_or_else(|error| panic!("{selection:?}: {error}"));
    let counts = (
        outcome.success,
        outcome.passed,
        outcome.failed,
        outcome.ignored,
    );
    (counts, names)
}

#[test]
fn c1_32_c1_35_tests_that_are_cases_stay_lines_and_ids_find_their_test() {
    let mut p = project("discovered-cases");
    fs::create_dir_all(p.root.join("src")).unwrap();
    for (name, text) in [
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

    let mut said = Vec::new();
    let mut ids: Vec<_> = crate::test::discover_tests(
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
    ids.sort();
    let gtest = ids.iter().any(|id| id == "ns::unit");
    let catch2 = ids.iter().any(|id| id == "c2_all");
    let (expected, inner_count) = expected(gtest, catch2);
    assert_eq!(ids, expected, "{said:?}");
    if inner_count > 0 {
        let line = format!("{inner_count} caso(s) de gtest/Catch2 dentro dos binários do ctest");
        assert!(said.contains(&line), "{said:?}");
    }
    let folder = p.root.join("build/dev");
    if gtest {
        gtest_runs(&p.root, &folder);
    } else {
        eprintln!("NAO PROVADO: gtest ausente nesta maquina");
    }
    if catch2 {
        catch2_runs(&p.root, &folder);
    } else {
        eprintln!("NAO PROVADO: Catch2 ausente nesta maquina");
    }
    // C1-36: CTest aligns `Test  #1` in a suite of ten tests or more, and
    // the run counts the tests #1 to #9 as well.
    if gtest && catch2 {
        let (counts, names) = run(&p.root, Selection::All);
        assert_eq!((counts, names.len()), ((false, 10, 6, 1), 17), "{names:?}");
        assert!(names.iter().any(|name| name == "Math.Adds"), "{names:?}");
    }
    assert_eq!(
        inner(&folder, "nope::x"),
        Some(Err(
            "o ctest não lista um teste para `nope::x`; configure e compile de novo".to_owned()
        ))
    );
}

/// As linhas do ctest e os casos que só os testes que rodam o binário
/// inteiro dão, uma vez cada; `dup::x::y`, de dois testes, fica de fora.
fn expected(gtest: bool, catch2: bool) -> (Vec<String>, usize) {
    let mut expected = Vec::new();
    let mut inner_count = 0;
    let mut add = |lines: &[&str], cases: &[&str]| {
        expected.extend(lines.iter().chain(cases).map(|&id| id.to_owned()));
        inner_count += cases.len();
    };
    if gtest {
        add(
            &[
                "Math.Adds",
                "Math.Fails",
                "Text.Off",
                "ns::unit",
                "gt_math",
                "gt_env",
                "gt_color",
                "gt_color::Math.Adds",
            ],
            &[
                "ns::unit::Math.Adds",
                "ns::unit::Math.Fails",
                "ns::unit::Text.DISABLED_Off",
                "gt_color::Math.Fails",
                "gt_color::Text.DISABLED_Off",
            ],
        );
    }
    if catch2 {
        add(
            &[
                "Widget::draw works",
                "plain",
                "x::y",
                "y",
                "dup",
                "dup::x",
                "c2_all",
                "c2_tag",
                "c2_unknown",
            ],
            &[
                "dup::Widget::draw works",
                "dup::plain",
                "dup::y",
                "dup::x::Widget::draw works",
                "dup::x::plain",
                "dup::x::x::y",
                "c2_all::Widget::draw works",
                "c2_all::plain",
                "c2_all::x::y",
                "c2_all::y",
            ],
        );
    }
    expected.sort();
    (expected, inner_count)
}

fn gtest_runs(root: &Path, folder: &Path) {
    let owned =
        |items: &[&str]| -> Vec<String> { items.iter().map(|&item| item.to_owned()).collect() };
    // O teste do ctest com `::` no nome acha os casos dele, e o evento leva
    // o id da linha, não o nome de dentro (C1-37).
    assert_eq!(
        inner(folder, "ns::unit::Math.Adds"),
        Some(Ok((
            (true, 1, 0, 0),
            owned(&["ns::unit::Math.Adds"]),
            vec![],
            vec![]
        )))
    );
    assert_eq!(
        inner(folder, "ns::unit::Math.Fails"),
        Some(Ok((
            (false, 0, 1, 0),
            owned(&["ns::unit::Math.Fails"]),
            vec![],
            vec![]
        )))
    );
    assert_eq!(
        inner(folder, "ns::unit::Text.DISABLED_Off"),
        Some(Ok((
            (true, 0, 0, 1),
            owned(&["ns::unit::Text.DISABLED_Off"]),
            vec![],
            owned(&[
                "o gtest não roda o caso desativado `Text.DISABLED_Off` sem \
                     --gtest_also_run_disabled_tests; ele conta como ignorado"
            ])
        )))
    );
    // O nome de um teste do ctest é esse teste, também com `::`.
    assert!(inner(folder, "gt_color::Math.Adds").is_none());
    assert_eq!(
        run(root, Selection::Exact("gt_color::Math.Adds")),
        ((true, 1, 0, 0), owned(&["gt_color::Math.Adds"]))
    );
    // A linha do desativado do `gtest_discover_tests` não roda e não reprova.
    assert_eq!(
        run(root, Selection::Exact("Text.Off")),
        ((true, 0, 0, 1), owned(&["Text.Off"]))
    );
    assert_eq!(
        inner(folder, "gt_math::Math.Adds"),
        Some(Err(
            "o teste `gt_math` do ctest já escolhe os casos e não se abre em casos; \
                  rode a linha dele"
                .to_owned()
        ))
    );
    assert_eq!(
        inner(folder, "ns::unit::Math.Gone"),
        Some(Err(
            "o binário do teste `ns::unit` não lista mais o caso `Math.Gone`; recompile \
                  e descubra os testes de novo"
                .to_owned()
        ))
    );
}

fn catch2_runs(root: &Path, folder: &Path) {
    let owned =
        |items: &[&str]| -> Vec<String> { items.iter().map(|&item| item.to_owned()).collect() };
    for (id, case) in [
        ("c2_all::x::y", "x::y"),
        ("dup::x::x::y", "x::y"),
        ("dup::x::Widget::draw works", "Widget::draw works"),
        ("c2_all::y", "y"),
    ] {
        assert_eq!(
            inner(folder, id),
            Some(Ok((
                (true, 1, 0, 0),
                owned(&[id]),
                vec![format!("RAN<{case}>")],
                vec![]
            ))),
            "{id}"
        );
    }
    // A linha do `catch_discover_tests` com `::` roda pelo ctest.
    assert!(inner(folder, "Widget::draw works").is_none());
    assert_eq!(
        run(root, Selection::Exact("Widget::draw works")),
        ((true, 1, 0, 0), owned(&["Widget::draw works"]))
    );
    assert_eq!(
        inner(folder, "dup::x::y"),
        Some(Err(
            "o id `dup::x::y` é de mais de um caso: `x::y` do teste `dup` e `y` do teste \
                  `dup::x`; rode a linha do ctest"
                .to_owned()
        ))
    );
    for closed in ["c2_tag", "c2_unknown"] {
        assert_eq!(
            inner(folder, &format!("{closed}::plain")),
            Some(Err(format!(
                "o teste `{closed}` do ctest já escolhe os casos e não se abre em casos; rode a \
                 linha dele"
            ))),
            "{closed}"
        );
    }
}

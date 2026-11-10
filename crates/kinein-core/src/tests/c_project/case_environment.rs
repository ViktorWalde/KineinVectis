//! C1-34, C1-38 e C1-40: o caso de dentro como o ctest roda o teste dele
//! (consolidação C §22), com `CMake`, GCC, gtest e Catch2 reais.
//!
//! Até o 0.215.0 o caso rodava o binário só com o filtro e a pasta: sem o
//! `ENVIRONMENT`, o `ENVIRONMENT_MODIFICATION` e os argumentos do `add_test`.
//! O comando mostrado não era o que rodava. O teste cujo desfecho o `CTest`
//! decide de outro jeito, como o `WILL_FAIL`, abria em casos, e o `TIMEOUT`
//! não valia para o caso.

use super::configurations::{build, configure};
use super::environment::project;
use crate::test::{Selection, TestEvent, frameworks::run_inner_case};
use kinein_protocol::ProjectKind;
use serde_json::json;
use std::{
    fs,
    path::Path,
    sync::Arc,
    time::{Duration, Instant},
};

/// O binário diz o ambiente e os argumentos que viu, na linha do ctest e no
/// caso; o `InitGoogleTest` tira as flags do gtest e deixa os outros.
const GTEST: &str = r#"#include <gtest/gtest.h>
#include <cstdio>
#include <cstdlib>
#include <string>
static std::string rest;
TEST(Env, Reads) {
  for (const char *name : {"KV_DATA", "KV_EMPTY", "HOME", "PATH", "CTEST_INTERACTIVE_DEBUG_MODE"}) {
    const char *value = std::getenv(name);
    std::printf("SAW<%s=%s>\n", name, value ? value : "(unset)");
  }
  std::printf("SAW<args=%s>\n", rest.c_str());
}
TEST(Env, Fails) { FAIL() << "a linha do ctest mostra a saida"; }
int main(int argc, char **argv) {
  ::testing::InitGoogleTest(&argc, argv);
  for (int i = 1; i < argc; ++i) rest += std::string("[") + argv[i] + "]";
  return RUN_ALL_TESTS();
}
"#;

/// Com `KV_HANG`, o binário para depois de o caso passar: o gtest escreve o
/// `[       OK ]` e o `TearDown` global não termina.
const SLOW: &str = r#"#include <gtest/gtest.h>
#include <chrono>
#include <cstdlib>
#include <thread>
class Hang : public ::testing::Environment {
 public:
  void TearDown() override {
    if (std::getenv("KV_HANG")) std::this_thread::sleep_for(std::chrono::seconds(60));
  }
};
static ::testing::Environment *const hang = ::testing::AddGlobalTestEnvironment(new Hang);
TEST(Slow, Passes) {}
TEST(Slow, Sleeps) { std::this_thread::sleep_for(std::chrono::seconds(60)); }
"#;

const CATCH2: &str = r#"#include <catch2/catch_test_macros.hpp>
#include <cstdio>
#include <cstdlib>
TEST_CASE("reads") {
  const char *value = std::getenv("KV_DATA");
  std::printf("SAW<KV_DATA=%s>\n", value ? value : "(unset)");
}
"#;

const LISTS: &str = r#"cmake_minimum_required(VERSION 3.24)
project(case_environment LANGUAGES CXX)
enable_testing()
find_package(GTest QUIET)
if(GTest_FOUND)
  add_executable(ge src/ge.cpp)
  target_link_libraries(ge GTest::gtest)
  add_test(NAME env_unit COMMAND ge "${CMAKE_SOURCE_DIR}/os dados" --gtest_brief=1)
  set_tests_properties(env_unit PROPERTIES
    ENVIRONMENT "KV_DATA=do env;HOME;KV_EMPTY=x"
    ENVIRONMENT_MODIFICATION "KV_DATA=string_append:+mod;PATH=path_list_prepend:/kv/nada;KV_EMPTY=set:")
  add_test(NAME wf COMMAND ge)
  set_tests_properties(wf PROPERTIES WILL_FAIL TRUE)
  add_test(NAME off COMMAND ge)
  set_tests_properties(off PROPERTIES DISABLED TRUE)
  add_test(NAME fx_setup COMMAND ${CMAKE_COMMAND} -E touch fixture.txt)
  set_tests_properties(fx_setup PROPERTIES FIXTURES_SETUP fx)
  add_test(NAME fx_user COMMAND ge)
  set_tests_properties(fx_user PROPERTIES FIXTURES_REQUIRED fx)
  add_test(NAME bad COMMAND ge)
  set_tests_properties(bad PROPERTIES ENVIRONMENT_MODIFICATION "KV_DATA=bogus:x")
  add_executable(gslow src/slow.cpp)
  target_link_libraries(gslow GTest::gtest_main)
  add_test(NAME slow COMMAND gslow)
  set_tests_properties(slow PROPERTIES TIMEOUT 1)
  add_test(NAME slow_end COMMAND gslow)
  set_tests_properties(slow_end PROPERTIES TIMEOUT 1 ENVIRONMENT "KV_HANG=1")
endif()
find_package(Catch2 3 QUIET)
if(Catch2_FOUND)
  add_executable(c2 src/c2.cpp)
  target_link_libraries(c2 Catch2::Catch2WithMain)
  add_test(NAME c2_rep COMMAND c2 -r console)
  set_tests_properties(c2_rep PROPERTIES ENVIRONMENT "KV_DATA=do c2")
endif()
"#;

const PRESETS: &str = r#"{"version":6,"configurePresets":[
  {"name":"dev","generator":"Ninja","binaryDir":"${sourceDir}/build/dev",
   "cacheVariables":{"CMAKE_CXX_COMPILER":"g++","CMAKE_EXPORT_COMPILE_COMMANDS":"ON"}}]}"#;

/// O desfecho e as contagens de aprovados, reprovados e ignorados.
type Counts = (bool, u64, u64, u64);

/// O que rodar deu: as contagens, os casos marcados, as linhas `SAW<…>` do
/// binário, as linhas da IDE e o comando mostrado.
#[derive(Debug, Default, PartialEq, Eq)]
struct Ran {
    counts: Counts,
    cases: Vec<String>,
    saw: Vec<String>,
    said: Vec<String>,
    shown: Vec<String>,
}

impl Ran {
    fn record(&mut self, event: TestEvent) {
        match event {
            TestEvent::Case { name, .. } => self.cases.push(name),
            TestEvent::Output { line, .. } if line.trim_start().starts_with("SAW<") => {
                self.saw.push(line.trim_start().to_owned());
            }
            TestEvent::Output { line, .. } if line.starts_with("o caso ") => self.said.push(line),
            TestEvent::Output { .. } => {}
            TestEvent::Started { command } => self.shown.push(command),
        }
    }
}

/// Um caso de dentro, pelo binário.
fn inner(folder: &Path, id: &str) -> Result<Ran, String> {
    let mut ran = Ran::default();
    let outcome = run_inner_case(
        Path::new("ctest"),
        folder,
        id,
        &Arc::default(),
        &mut |event| ran.record(event),
    )
    .unwrap_or_else(|| panic!("{id} é um caso de dentro"))
    .map_err(|error| error.to_string())?;
    ran.counts = (
        outcome.success,
        outcome.passed,
        outcome.failed,
        outcome.ignored,
    );
    Ok(ran)
}

/// A linha do ctest, pelo caminho do `test.run`.
fn line(root: &Path, id: &str) -> Ran {
    let mut ran = Ran::default();
    let outcome = crate::test::run_tests(
        root,
        ProjectKind::Cmake,
        Selection::Exact(id),
        None,
        &Arc::default(),
        &mut |event| ran.record(event),
    )
    .unwrap_or_else(|error| panic!("{id}: {error}"));
    ran.counts = (
        outcome.success,
        outcome.passed,
        outcome.failed,
        outcome.ignored,
    );
    ran
}

#[test]
fn c1_34_c1_40_a_case_runs_as_the_ctest_runs_its_test() {
    let mut p = project("case-environment");
    fs::create_dir_all(p.root.join("src")).unwrap();
    fs::create_dir_all(p.root.join("os dados")).unwrap();
    for (name, text) in [
        ("src/ge.cpp", GTEST),
        ("src/slow.cpp", SLOW),
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

    let mut ids: Vec<_> = crate::test::discover_tests(
        &p.root,
        ProjectKind::Cmake,
        None,
        &Arc::default(),
        &mut |_| {},
    )
    .unwrap()
    .into_iter()
    .map(|case| case.id)
    .collect();
    ids.sort();
    let gtest = ids.iter().any(|id| id == "env_unit");
    let catch2 = ids.iter().any(|id| id == "c2_rep");
    let mut expected = Vec::new();
    if gtest {
        expected.extend([
            "bad",
            "env_unit",
            "env_unit::Env.Fails",
            "env_unit::Env.Reads",
            "fx_setup",
            "fx_user",
            "off",
            "slow",
            "slow::Slow.Passes",
            "slow::Slow.Sleeps",
            "slow_end",
            "slow_end::Slow.Passes",
            "slow_end::Slow.Sleeps",
            "wf",
        ]);
    }
    if catch2 {
        expected.extend(["c2_rep", "c2_rep::reads"]);
    }
    expected.sort_unstable();
    assert_eq!(ids, expected);
    let folder = p.root.join("build/dev");
    if gtest {
        gtest_runs(&p.root, &folder);
    } else {
        eprintln!("NAO PROVADO: gtest ausente nesta maquina");
    }
    if catch2 {
        // O reporter do `add_test` vale, e o `-r compact` não entra: com os
        // dois na saída padrão, o Catch2 3.16.1 aborta.
        let case = inner(&folder, "c2_rep::reads").unwrap();
        assert_eq!(case.counts, (true, 1, 0, 0), "{case:?}");
        assert_eq!(case.saw, ["SAW<KV_DATA=do c2>"]);
        assert!(
            case.shown[0].ends_with("/build/dev/c2' -r console reads"),
            "{:?}",
            case.shown
        );
    } else {
        eprintln!("NAO PROVADO: Catch2 ausente nesta maquina");
    }
}

fn gtest_runs(root: &Path, folder: &Path) {
    // O próprio binário compara: o caso vê o ambiente e os argumentos que a
    // linha do ctest vê, e o `--gtest_brief=1` do teste perde para o da IDE.
    let whole = line(root, "env_unit");
    assert_eq!(whole.counts, (false, 0, 1, 0), "{whole:?}");
    let case = inner(folder, "env_unit::Env.Reads").unwrap();
    assert_eq!(case.counts, (true, 1, 0, 0), "{case:?}");
    assert_eq!(case.cases, ["env_unit::Env.Reads"]);
    assert_eq!(case.saw, whole.saw);
    let path = std::env::var("PATH").unwrap_or_default();
    let data = root.join("os dados").display().to_string();
    assert_eq!(
        case.saw,
        [
            "SAW<KV_DATA=do env+mod>".to_owned(),
            "SAW<KV_EMPTY=>".to_owned(),
            "SAW<HOME=(unset)>".to_owned(),
            format!("SAW<PATH=/kv/nada:{path}>"),
            "SAW<CTEST_INTERACTIVE_DEBUG_MODE=1>".to_owned(),
            format!("SAW<args=[{data}]>"),
        ]
    );
    // C1-38: o comando mostrado é o que roda, com as aspas do shell.
    let exe = folder.join("ge").display().to_string();
    assert_eq!(
        case.shown,
        [format!(
            "'{exe}' '{data}' --gtest_brief=1 --gtest_filter=Env.Reads --gtest_color=no \
             --gtest_print_time=1 --gtest_brief=0"
        )]
    );

    // C1-40: o CTest decide o desfecho destes de outro jeito, e eles rodam
    // inteiros pela linha deles.
    let refused = |test: &str, property: &str| {
        format!(
            "o CTest decide o desfecho do teste `{test}` também pelo `{property}`, que o caso \
             sozinho não reproduz, e ele não se abre em casos; rode a linha dele"
        )
    };
    for (test, property) in [
        ("wf", "WILL_FAIL"),
        ("off", "DISABLED"),
        ("fx_user", "FIXTURES_REQUIRED"),
    ] {
        assert_eq!(
            inner(folder, &format!("{test}::Env.Reads")),
            Err(refused(test, property))
        );
    }
    assert_eq!(line(root, "wf").counts, (true, 1, 0, 0));
    assert_eq!(line(root, "off").counts, (true, 0, 0, 1));
    assert_eq!(
        inner(folder, "bad::Env.Reads"),
        Err(
            "o CTest não roda o teste `bad`: o `ENVIRONMENT_MODIFICATION` `KV_DATA=bogus:x` tem a \
             operação `bogus`, que ele não conhece; corrija o teste no CMake"
                .to_owned()
        )
    );
    assert_eq!(line(root, "bad").counts, (false, 0, 1, 0));

    // O `TIMEOUT` do teste para o caso, que conta como reprovado.
    let started = Instant::now();
    let slow = inner(folder, "slow::Slow.Sleeps").unwrap();
    assert!(started.elapsed() < Duration::from_secs(30));
    assert_eq!(slow.counts, (false, 0, 1, 0), "{slow:?}");
    assert_eq!(slow.cases, ["slow::Slow.Sleeps"]);
    assert_eq!(
        slow.said,
        ["o caso passou do `TIMEOUT` de 1 s do teste `slow` e foi parado, como o CTest faz"]
    );
    assert_eq!(line(root, "slow").counts, (false, 0, 1, 0));
    // O caso passou e o binário não terminou: o CTest diz `***Timeout`, e o
    // caso conta só como reprovado.
    let end = inner(folder, "slow_end::Slow.Passes").unwrap();
    assert_eq!(end.counts, (false, 0, 1, 0), "{end:?}");
    assert_eq!(
        end.cases,
        ["slow_end::Slow.Passes", "slow_end::Slow.Passes"]
    );
}

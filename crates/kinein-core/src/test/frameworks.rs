//! gtest e Catch2 na arvore de testes (D7 do `roadmaps/41`, P5 do 40 §4.1).
//!
//! Um `add_test` do `CMake` e' UM binario com N casos dentro; o `ctest -N`
//! lista o binario, e a arvore ficava com uma linha por executavel. Aqui
//! (2026-09-17) os casos de dentro sao perguntados ao proprio binario — e' o
//! que o `TestMate`/Test Explorer fazem:
//!
//! ```text
//! ctest --test-dir <build> --show-only=json-v1   (ctest 4.2.3, medido em
//!     2026-09-17): tests[].name, tests[].command[], properties[WORKING_DIRECTORY]
//! <exe> --gtest_list_tests           gtest: "Suite." na coluna 0, "  Case" abaixo
//!                                    (parametrizados: "Suite/Inst." e "Case/0  # GetParam()")
//! <exe> --list-tests --verbosity quiet   Catch2 v3: um nome por linha
//! <exe> --gtest_filter=Suite.Case    roda um caso gtest; "[       OK ] Suite.Case (n ms)"
//!                                    e "[  FAILED  ] Suite.Case" por caso
//! <exe> "Nome do caso" -r compact     roda um caso Catch2; o exit code diz o desfecho
//! ```
//!
//! Ids: `<teste do ctest>::<Suite.Case>` — o `::` separa o binario do caso,
//! e nao aparece em nomes de gtest (`.`) nem, na pratica, nos de Catch2.
//! Um binario que nao responde a nenhuma das duas listagens continua sendo
//! so' a linha do ctest (nada e' adivinhado).

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    process::Command,
    sync::{Arc, atomic::AtomicBool},
    time::Duration,
};

use kinein_protocol::TestCaseInfo;

use super::{CaseStatus, TestError, TestEvent, TestOutcome, entries, launch, stream_command_until};
use crate::process;

/// O separador entre o teste do ctest e o caso de dentro do binario.
pub const SEP: &str = "::";

/// Um teste do ctest como o `json-v1` o descreve.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CtestEntry {
    /// `tests[].name`.
    pub name: String,
    /// `tests[].command` (executavel e argumentos).
    pub command: Vec<String>,
    /// `WORKING_DIRECTORY`, quando declarada.
    pub working_directory: Option<PathBuf>,
    /// `ENVIRONMENT`: `NAME=value` (C1-32).
    pub environment: Vec<String>,
    /// `ENVIRONMENT_MODIFICATION`: `NAME=op:value` (C1-32).
    pub environment_modification: Vec<String>,
    /// The properties of `entries::OUTCOME_PROPERTIES` that are on (C1-40).
    pub outcome_properties: Vec<String>,
    /// `TIMEOUT`, when greater than zero and a duration (C1-40).
    pub timeout: Option<Duration>,
}

/// O framework que o binario revelou.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Framework {
    /// `GoogleTest` (`--gtest_list_tests` respondeu).
    Gtest,
    /// Catch2 (`--list-tests --verbosity quiet` respondeu).
    Catch2,
}

/// Le o `--show-only=json-v1` do ctest.
#[must_use]
pub fn parse_ctest_json(texto: &str) -> Vec<CtestEntry> {
    let Ok(v) = serde_json::from_str::<serde_json::Value>(texto) else {
        return Vec::new();
    };
    v.get("tests")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|t| {
            let name = t.get("name")?.as_str()?.to_owned();
            let command = t
                .get("command")
                .and_then(serde_json::Value::as_array)
                .map(|c| {
                    c.iter()
                        .filter_map(|a| a.as_str().map(str::to_owned))
                        .collect()
                })
                .unwrap_or_default();
            let property = |name: &str| {
                t.get("properties")?
                    .as_array()?
                    .iter()
                    .find(|p| p.get("name").and_then(serde_json::Value::as_str) == Some(name))?
                    .get("value")
            };
            let working_directory = property("WORKING_DIRECTORY")
                .and_then(serde_json::Value::as_str)
                .map(PathBuf::from);
            // The ctest writes a list property as an array of strings.
            let list = |name: &str| -> Vec<String> {
                property(name)
                    .and_then(serde_json::Value::as_array)
                    .into_iter()
                    .flatten()
                    .filter_map(|value| value.as_str().map(str::to_owned))
                    .collect()
            };
            // The ctest leaves out the property that is off; `false`, an
            // empty list and an empty text are off too.
            let on = |value: &serde_json::Value| match value {
                serde_json::Value::Null | serde_json::Value::Bool(false) => false,
                serde_json::Value::Array(values) => !values.is_empty(),
                serde_json::Value::String(text) => !text.is_empty(),
                _ => true,
            };
            let outcome_properties = entries::OUTCOME_PROPERTIES
                .iter()
                .filter(|name| property(name).is_some_and(on))
                .map(|name| (*name).to_owned())
                .collect();
            // Zero and less is no limit, as the ctest does; a number too big
            // for a duration is none either.
            let timeout = property("TIMEOUT")
                .and_then(serde_json::Value::as_f64)
                .filter(|seconds| *seconds > 0.0)
                .and_then(|seconds| Duration::try_from_secs_f64(seconds).ok());
            Some(CtestEntry {
                name,
                command,
                working_directory,
                environment: list("ENVIRONMENT"),
                environment_modification: list("ENVIRONMENT_MODIFICATION"),
                outcome_properties,
                timeout,
            })
        })
        .collect()
}

/// Os casos de `--gtest_list_tests`: `Suite.` na coluna 0, `  Caso` indentado
/// (o `# GetParam()` dos parametrizados e' comentario e cai).
#[must_use]
pub fn parse_gtest_list(texto: &str) -> Vec<String> {
    let mut suite = String::new();
    let mut cases = Vec::new();
    for linha in texto.lines() {
        if linha.is_empty() || linha.starts_with("Running main()") {
            continue;
        }
        if !linha.starts_with(' ') {
            linha
                .split('#')
                .next()
                .unwrap_or_default()
                .trim()
                .clone_into(&mut suite);
            continue;
        }
        let case = linha.split('#').next().unwrap_or_default().trim();
        if !case.is_empty() && !suite.is_empty() {
            cases.push(format!("{suite}{case}"));
        }
    }
    cases
}

/// Os casos de `--list-tests --verbosity quiet` (Catch2 v3): um por linha.
#[must_use]
pub fn parse_catch2_list(texto: &str) -> Vec<String> {
    texto
        .lines()
        .map(str::trim)
        .filter(|l| {
            !l.is_empty()
                && !l.starts_with("All available test cases")
                && !l.ends_with(" test cases")
        })
        .map(str::to_owned)
        .collect()
}

/// `[       OK ] Suite.Case (3 ms)` / `[  FAILED  ] Suite.Case (1 ms)`.
///
/// C1-29: only the line of the case ends in the time; the summary at the end
/// lists the failed and skipped cases again without it, so the case runs with
/// `--gtest_print_time=1`. A parameterized case says `, where GetParam() = 2`
/// after its name, and that comment is not part of it.
#[must_use]
pub fn parse_gtest_case(line: &str) -> Option<(String, CaseStatus)> {
    let (status, rest) = if let Some(r) = line.strip_prefix("[       OK ] ") {
        (CaseStatus::Passed, r)
    } else if let Some(r) = line.strip_prefix("[  FAILED  ] ") {
        (CaseStatus::Failed, r)
    } else if let Some(r) = line.strip_prefix("[  SKIPPED ] ") {
        (CaseStatus::Ignored, r)
    } else {
        return None;
    };
    let (case, time) = rest.strip_suffix(" ms)")?.rsplit_once(" (")?;
    if time.is_empty() || !time.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    let name = case.split(", where ").next()?.trim();
    (!name.is_empty() && name.contains('.')).then(|| (name.to_owned(), status))
}

/// Pergunta ao binario do teste quais casos ele tem: gtest primeiro, Catch2
/// depois.
///
/// `None` = o binario nao e' de nenhum dos dois (ou nao roda). A listagem tem
/// a pasta e o ambiente do teste, sem os argumentos, como o
/// `gtest_discover_tests` (C1-34).
#[must_use]
pub fn list_cases(entry: &CtestEntry) -> Option<(Framework, Vec<String>)> {
    let exe = entry.command.first()?;
    let run = |args: &[&str]| -> Option<String> {
        let mut c = Command::new(exe);
        c.args(args);
        if let Some(cwd) = &entry.working_directory {
            c.current_dir(cwd);
        }
        launch::apply_environment(&mut c, entry).ok()?;
        let saida = c.output().ok()?;
        saida
            .status
            .success()
            .then(|| String::from_utf8_lossy(&saida.stdout).into_owned())
    };
    if let Some(texto) = run(&["--gtest_list_tests"]) {
        let cases = parse_gtest_list(&texto);
        if !cases.is_empty() {
            return Some((Framework::Gtest, cases));
        }
    }
    // C1-28: the listing of Catch2 is one name per line, which any test that
    // prints a line for every argument also gives. Catch2 says who it is in
    // `--libidentify`, the flag it keeps for tools. Catch2 3 lists in random
    // order, and the tree keeps the order of the declarations (C1-31).
    if run(&["--libidentify"]).is_some_and(|answer| is_catch2(&answer))
        && let Some(texto) = run(&["--list-tests", "--verbosity", "quiet", "--order", "decl"])
    {
        let cases = parse_catch2_list(&texto);
        if !cases.is_empty() {
            return Some((Framework::Catch2, cases));
        }
    }
    None
}

/// Why Catch2 cannot run only this case, when it cannot (C1-31): a spec
/// that starts with `exclude:` excludes, also escaped, and Catch2 compares the
/// names without the ASCII case, so the other one would run too (measured with Catch2
/// 3.16.1).
fn catch2_refusal(name: &str, names: &[String]) -> Option<String> {
    if name.starts_with("exclude:") {
        return Some(format!(
            "o Catch2 lê o caso `{name}` como uma exclusão e rodaria os outros casos; \
             rode a linha do ctest do binário"
        ));
    }
    let other = names
        .iter()
        .find(|other| *other != name && other.eq_ignore_ascii_case(name))?;
    Some(format!(
        "o Catch2 compara os nomes sem caixa e rodaria também o caso `{other}`; \
         rode a linha do ctest do binário"
    ))
}

/// What gtest 1.18 writes when the filter of one case picked a disabled
/// one: the count of the disabled cases the filter picked, never zero.
const DISABLED_NOTE: &str = "YOU HAVE 1 DISABLED TEST";

/// `framework: Catch2` in the `--libidentify` answer, aligned with spaces.
fn is_catch2(identity: &str) -> bool {
    identity.lines().any(|line| {
        line.split_once(':')
            .is_some_and(|(key, value)| key.trim() == "framework" && value.trim() == "Catch2")
    })
}

/// Os testes do ctest com os comandos, pelo `json-v1`. `ctest` e' o programa
/// (o do PATH, ou um falso no teste).
pub fn ctest_entries(ctest: &Path, build_dir: &Path, cancel: &Arc<AtomicBool>) -> Vec<CtestEntry> {
    let mut c = Command::new(ctest);
    c.arg("--test-dir")
        .arg(build_dir)
        .arg("--show-only=json-v1");
    super::ctest_configuration(&mut c, build_dir);
    crate::msvc::apply_environment(&mut c);
    let mut texto = String::new();
    let _ = process::stream_command_lines_cancelable(c, cancel, &mut |stream, linha| {
        if stream == "stdout" {
            texto.push_str(&linha);
            texto.push('\n');
        }
    });
    parse_ctest_json(&texto)
}

/// What a binary said of its cases: the framework and the names.
pub(super) type Listing = Option<(Framework, Vec<String>)>;

/// The listing of the binary of a test of the ctest, asked once per binary,
/// folder and environment: `gtest_discover_tests` gives one test per case.
type Listings = HashMap<(String, Option<PathBuf>, Vec<String>, Vec<String>), Listing>;

/// Who lists the cases of the binary of a test: `list_cases`, or a fake.
type Lister<'a> = &'a mut dyn FnMut(&CtestEntry) -> Listing;

fn listing(listings: &mut Listings, entry: &CtestEntry, list: Lister<'_>) -> Listing {
    let exe = entry.command.first()?;
    listings
        .entry((
            exe.clone(),
            entry.working_directory.clone(),
            entry.environment.clone(),
            entry.environment_modification.clone(),
        ))
        .or_insert_with(|| Path::new(exe).is_file().then(|| list(entry)).flatten())
        .clone()
}

/// Os casos de dentro de cada binario do ctest, ja' com o id `teste::caso`:
/// só os do teste que roda todos os casos (C1-32).
pub fn discover_inner_cases(entries: &[CtestEntry], cancel: &Arc<AtomicBool>) -> Vec<TestCaseInfo> {
    inner_cases(entries, cancel, &mut list_cases)
}

fn inner_cases(
    entries: &[CtestEntry],
    cancel: &Arc<AtomicBool>,
    list: Lister<'_>,
) -> Vec<TestCaseInfo> {
    let mut listings = Listings::new();
    let mut cases = Vec::new();
    for entry in entries {
        if cancel.load(std::sync::atomic::Ordering::SeqCst) {
            break;
        }
        if entries::closed(entry).is_some() {
            continue;
        }
        let Some((framework, names)) = listing(&mut listings, entry, list) else {
            continue;
        };
        if !entries::runs_every_case(framework, entry) {
            continue;
        }
        for name in names {
            cases.push(TestCaseInfo {
                id: format!("{}{SEP}{name}", entry.name),
                name,
                file: None,
            });
        }
    }
    entries::unique(cases, entries)
}

/// Roda UM caso de dentro de um binario: `teste::caso`. `None` quando o id
/// e' um teste do ctest de sempre: sem o separador, ou o nome de um teste do
/// ctest com `::` (C1-32).
pub fn run_inner_case(
    ctest: &Path,
    build_dir: &Path,
    id: &str,
    cancel: &Arc<AtomicBool>,
    sink: &mut dyn FnMut(TestEvent),
) -> Option<Result<TestOutcome, TestError>> {
    if !id.contains(SEP) {
        return None;
    }
    let entries = ctest_entries(ctest, build_dir, cancel);
    let mut listings = Listings::new();
    let named = match entries::resolve(&entries, id, &mut |entry| {
        listing(&mut listings, entry, &mut list_cases)
    })? {
        Ok(named) => named,
        Err(error) => return Some(Err(error)),
    };
    let (entry, case) = (named.entry, named.case);
    let catch2 = named.framework == Framework::Catch2;
    if catch2 && let Some(reason) = catch2_refusal(case, &named.cases) {
        return Some(Err(TestError::Refused { reason }));
    }
    // C1-34: the arguments and the environment of the test, as the ctest
    // runs it.
    let (mut command, display) = launch::case_command(entry, named.framework, case)?;
    if let Err(reason) = launch::apply_environment(&mut command, entry) {
        return Some(Err(TestError::Refused { reason }));
    }
    let parse: fn(&str) -> Option<(String, CaseStatus)> =
        if catch2 { |_| None } else { parse_gtest_case };
    let test = &entry.name;
    let mut disabled = false;
    let run = stream_command_until(
        command,
        &display,
        parse,
        cancel,
        entry.timeout,
        &mut |event| {
            let event = match event {
                // C1-37: the row of the tree is the id `test::case`, not the
                // name inside the binary.
                TestEvent::Case { name, status } => TestEvent::Case {
                    name: format!("{test}{SEP}{name}"),
                    status,
                },
                TestEvent::Output { stream, line } => {
                    disabled |= line.trim() == DISABLED_NOTE;
                    TestEvent::Output { stream, line }
                }
                other @ TestEvent::Started { .. } => other,
            };
            sink(event);
        },
    );
    let mut result = match (run, entry.timeout) {
        // C1-40: stopped at the `TIMEOUT`, the case failed, as the ctest says
        // of the test.
        (Ok((mut outcome, true)), Some(timeout)) => {
            let seconds = timeout.as_secs_f64();
            sink(TestEvent::Output {
                stream: "stderr",
                line: format!(
                    "o caso passou do `TIMEOUT` de {seconds} s do teste `{test}` e foi parado, \
                     como o CTest faz"
                ),
            });
            sink(TestEvent::Case {
                name: id.to_owned(),
                status: CaseStatus::Failed,
            });
            (outcome.passed, outcome.failed, outcome.ignored) = (0, 1, 0);
            return Some(Ok(outcome));
        }
        (run, _) => run.map(|(outcome, _)| outcome),
    };
    // C1-35: gtest runs no disabled case without
    // `--gtest_also_run_disabled_tests` and says so; it does not say it of a
    // case that ran.
    if !catch2
        && disabled
        && let Ok(outcome) = &mut result
    {
        outcome.ignored += 1;
        sink(TestEvent::Output {
            stream: "stdout",
            line: format!(
                "o gtest não roda o caso desativado `{case}` sem \
                 --gtest_also_run_disabled_tests; ele conta como ignorado"
            ),
        });
        sink(TestEvent::Case {
            name: id.to_owned(),
            status: CaseStatus::Ignored,
        });
    }
    // O Catch2 nao diz o caso linha a linha (o `-r compact` diz asserções):
    // o desfecho do caso e' o exit code.
    if catch2 && let Ok(outcome) = &mut result {
        let status = if outcome.success {
            outcome.passed += 1;
            CaseStatus::Passed
        } else {
            outcome.failed += 1;
            CaseStatus::Failed
        };
        sink(TestEvent::Case {
            name: id.to_owned(),
            status,
        });
    }
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::{
        CtestEntry, catch2_refusal, parse_catch2_list, parse_ctest_json, parse_gtest_case,
        parse_gtest_list,
    };
    use crate::test::{CaseStatus, launch::catch2_filter};

    /// A test of the ctest that runs `exe`, as the `json-v1` gives it.
    fn entry_of(exe: &std::path::Path) -> CtestEntry {
        CtestEntry {
            name: "t".to_owned(),
            command: vec![exe.display().to_string()],
            ..CtestEntry::default()
        }
    }

    #[test]
    fn the_ctest_json_gives_name_command_and_working_directory() {
        let e = parse_ctest_json(
            r#"{"tests":[{"name":"unit_tests","command":["/b/unit","--gtest_color=no"],
                "properties":[{"name":"WORKING_DIRECTORY","value":"/tmp"}]},
                {"name":"sem_comando"}],"version":{"major":1,"minor":0}}"#,
        );
        assert_eq!(e.len(), 2);
        assert_eq!(e[0].name, "unit_tests");
        assert_eq!(e[0].command, ["/b/unit", "--gtest_color=no"]);
        assert_eq!(
            e[0].working_directory.as_deref(),
            Some(std::path::Path::new("/tmp"))
        );
        assert!(e[1].command.is_empty());
        assert!(parse_ctest_json("nao e json").is_empty());
    }

    /// C1-40: the properties of the outcome as the `json-v1` of `CMake`
    /// 4.4.5 writes them (measured), and the `TIMEOUT` that is a limit.
    #[test]
    fn the_ctest_json_gives_the_properties_of_the_outcome_and_the_timeout() {
        let test = |properties: &str| {
            let text = format!(
                r#"{{"tests":[{{"name":"t","command":["/b/gt"],"properties":[{properties}]}}]}}"#
            );
            parse_ctest_json(&text).remove(0)
        };
        let all = test(
            r#"{"name":"WILL_FAIL","value":true},
               {"name":"PASS_REGULAR_EXPRESSION","value":["never"]},
               {"name":"FAIL_REGULAR_EXPRESSION","value":["a","b"]},
               {"name":"SKIP_REGULAR_EXPRESSION","value":["c"]},
               {"name":"SKIP_RETURN_CODE","value":0},
               {"name":"DISABLED","value":true},
               {"name":"FIXTURES_REQUIRED","value":["db"]},
               {"name":"REQUIRED_FILES","value":["/x"]},
               {"name":"TIMEOUT_AFTER_MATCH","value":{"regex":["x"],"timeout":1.0}},
               {"name":"TIMEOUT","value":7.5}"#,
        );
        assert_eq!(
            all.outcome_properties,
            [
                "WILL_FAIL",
                "PASS_REGULAR_EXPRESSION",
                "FAIL_REGULAR_EXPRESSION",
                "SKIP_REGULAR_EXPRESSION",
                "SKIP_RETURN_CODE",
                "DISABLED",
                "FIXTURES_REQUIRED",
                "REQUIRED_FILES",
                "TIMEOUT_AFTER_MATCH"
            ]
        );
        assert_eq!(all.timeout, Some(std::time::Duration::from_millis(7500)));
        // What is off, and what does not change the outcome.
        let none = test(
            r#"{"name":"WILL_FAIL","value":false},
               {"name":"PASS_REGULAR_EXPRESSION","value":[]},
               {"name":"REQUIRED_FILES","value":""},
               {"name":"DISABLED","value":null},
               {"name":"DEPENDS","value":["setup"]},
               {"name":"FIXTURES_SETUP","value":["db"]},
               {"name":"RESOURCE_GROUPS","value":[{"requirements":[]}]},
               {"name":"TIMEOUT_SIGNAL_NAME","value":"SIGINT"},
               {"name":"TIMEOUT","value":0.0}"#,
        );
        assert!(none.outcome_properties.is_empty(), "{none:?}");
        assert_eq!(none.timeout, None);
        for (timeout, expected) in [
            ("-1.0", None),
            ("1e300", None),
            ("\"7\"", None),
            ("1", Some(std::time::Duration::from_secs(1))),
        ] {
            let entry = test(&format!(r#"{{"name":"TIMEOUT","value":{timeout}}}"#));
            assert_eq!(entry.timeout, expected, "{timeout}");
        }
    }

    #[test]
    fn gtest_and_catch2_listings_become_case_names() {
        let gtest = "Running main() from gmock_main.cc\nMath.\n  Adds\n  Subtracts\nStr/Param.\n  Case/0  # GetParam() = 1\n  Case/1  # GetParam() = 2\n";
        assert_eq!(
            parse_gtest_list(gtest),
            [
                "Math.Adds",
                "Math.Subtracts",
                "Str/Param.Case/0",
                "Str/Param.Case/1"
            ]
        );
        assert!(parse_gtest_list("").is_empty());
        assert_eq!(
            parse_catch2_list("vectors can be sized\n  \nfactorials are computed\n"),
            ["vectors can be sized", "factorials are computed"]
        );
        assert_eq!(
            parse_gtest_case("[       OK ] Math.Adds (0 ms)"),
            Some(("Math.Adds".to_owned(), CaseStatus::Passed))
        );
        assert_eq!(
            parse_gtest_case("[  FAILED  ] Math.Subtracts (2 ms)"),
            Some(("Math.Subtracts".to_owned(), CaseStatus::Failed))
        );
        assert_eq!(parse_gtest_case("[----------] 2 tests from Math"), None);
        assert_eq!(parse_gtest_case("[  FAILED  ] 1 test, listed below:"), None);
    }

    /// C1-31: the filter that runs only the case, and the cases Catch2 3.16.1
    /// cannot isolate (measured: `exclude:` is lowercase, and the case is
    /// folded only in ASCII).
    #[test]
    fn a_catch2_name_runs_only_its_case() {
        for (name, filter) in [
            ("plain name", "plain name"),
            ("a, b [x]", "a\\, b \\[x\\]"),
            ("back\\slash", "back\\\\slash"),
            ("*star*", "\\*star\\*"),
            ("~tilde", "\\~tilde"),
            ("has \"quote\"", "has \\\"quote\\\""),
            ("ação [π]", "ação \\[π\\]"),
            ("exclude:this", "exclude:this"),
        ] {
            assert_eq!(catch2_filter(name), filter, "{name}");
        }
        let names: Vec<String> = ["Case", "case", "Ação", "aÇão", "EXCLUDE:upper"]
            .map(str::to_owned)
            .to_vec();
        let refusal = |name: &str| catch2_refusal(name, &names);
        assert_eq!(
            refusal("exclude:this").as_deref(),
            Some(
                "o Catch2 lê o caso `exclude:this` como uma exclusão e rodaria os outros casos; \
                 rode a linha do ctest do binário"
            )
        );
        assert_eq!(
            refusal("case").as_deref(),
            Some(
                "o Catch2 compara os nomes sem caixa e rodaria também o caso `Case`; \
                 rode a linha do ctest do binário"
            )
        );
        assert!(refusal("Case").unwrap().contains("`case`"));
        for alone in [
            "EXCLUDE:upper",
            "Ação",
            "aÇão",
            "plain name",
            "an exclude:inside",
        ] {
            assert_eq!(refusal(alone), None, "{alone}");
        }
        assert!(catch2_refusal("ação", &names).unwrap().contains("`Ação`"));
    }

    /// C1-29: gtest 1.18 lists a failed or skipped case again in the summary,
    /// without the time, and a parameterized case with its parameter.
    #[test]
    fn a_gtest_case_counts_once_and_by_its_listed_name() {
        let case = |name: &str, status| Some((name.to_owned(), status));
        for (line, expected) in [
            ("[  FAILED  ] Math.Fails", None),
            ("[  SKIPPED ] Math.Skips", None),
            ("[       OK ] Math.Adds", None),
            (
                "[  SKIPPED ] Math.Skips (0 ms)",
                case("Math.Skips", CaseStatus::Ignored),
            ),
            (
                "[       OK ] Math.Adds (12 ms)",
                case("Math.Adds", CaseStatus::Passed),
            ),
            (
                "[  FAILED  ] Some/P.Odd/1, where GetParam() = 2 (0 ms)",
                case("Some/P.Odd/1", CaseStatus::Failed),
            ),
            ("[  FAILED  ] Some/P.Odd/1, where GetParam() = 2", None),
            (
                "[  FAILED  ] Typed/0.Does, where TypeParam = int (1 ms)",
                case("Typed/0.Does", CaseStatus::Failed),
            ),
            (
                "[  FAILED  ] Some/P.Odd/2, where GetParam() = (3 ms) (4 ms)",
                case("Some/P.Odd/2", CaseStatus::Failed),
            ),
            ("[       OK ] Math.Adds (x ms)", None),
            ("[       OK ] Math.Adds ( ms)", None),
            ("[       OK ] Math.Adds (0 ms) ", None),
        ] {
            assert_eq!(parse_gtest_case(line), expected, "{line}");
        }
    }

    /// Um binario gtest falso e um Catch2 falso respondem a listagem certa;
    /// um binario mudo nao vira caso. E o caso de dentro roda pelo binario,
    /// com o filtro do framework, a partir do `json-v1` de um ctest falso.
    #[test]
    #[cfg_attr(
        windows,
        ignore = "W6: build, qualidade e testes de C/C++ no Windows (Visual Studio, LLVM) sao da W6 (60 §3.3)"
    )]
    fn binaries_are_asked_for_their_cases_and_one_case_runs_by_the_binary() {
        use super::{
            Framework, SEP, ctest_entries, discover_inner_cases, list_cases, run_inner_case,
        };
        use crate::test::TestEvent;

        let _serial = crate::serialize_executables();

        let root = std::env::temp_dir().join(format!("kinein-frameworks-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let exe = |nome: &str, corpo: &str| -> std::path::PathBuf {
            let p = root.join(nome);
            crate::write_executable(&p, format!("#!/bin/sh\n{corpo}"));
            p
        };
        let gtest = exe(
            "unit",
            "case \"$1\" in\n  --gtest_list_tests) printf 'Math.\\n  Adds\\n  Subtracts\\n'; exit 0;;\n  --gtest_filter=*) echo \"argv: $*\"; echo '[       OK ] Math.Adds (0 ms)'; exit 0;;\n  *) exit 1;;\nesac\n",
        );
        let catch2 = exe(
            "c2",
            "case \"$1\" in\n  --gtest_list_tests) exit 1;;\n  --libidentify) printf 'description:    A Catch2 test executable\\ncategory:       testframework\\nframework:      Catch2\\nversion:        3.16.1\\n'; exit 0;;\n  --list-tests) [ \"$*\" = '--list-tests --verbosity quiet --order decl' ] || exit 1; printf 'vectors can be sized\\n'; exit 0;;\n  *) echo \"argv: $*\"; exit 0;;\nesac\n",
        );
        let mudo = exe("mudo", "exit 1\n");
        // C1-28: um teste comum que imprime uma linha para qualquer argumento,
        // e um que se diz de outro framework, não viram casos Catch2.
        let printing = exe("printing", "echo 'kv-run-Release'\n");
        let other = exe(
            "other",
            "case \"$1\" in\n  --libidentify) printf 'description:    Catch2\\nframework:      Catch\\n'; exit 0;;\n  *) echo 'x'; exit 0;;\nesac\n",
        );
        assert_eq!(
            list_cases(&entry_of(&gtest)),
            Some((
                Framework::Gtest,
                vec!["Math.Adds".to_owned(), "Math.Subtracts".to_owned()]
            ))
        );
        assert_eq!(
            list_cases(&entry_of(&catch2)),
            Some((Framework::Catch2, vec!["vectors can be sized".to_owned()]))
        );
        assert_eq!(list_cases(&entry_of(&mudo)), None);
        assert_eq!(list_cases(&entry_of(&printing)), None);
        assert_eq!(list_cases(&entry_of(&other)), None);

        // O ctest falso: o json-v1 com os tres binarios.
        let ctest = exe(
            "ctest",
            &format!(
                "printf '%s' '{{\"tests\":[{{\"name\":\"unit_tests\",\"command\":[\"{g}\"]}},{{\"name\":\"catch\",\"command\":[\"{c}\"]}},{{\"name\":\"mudo\",\"command\":[\"{m}\"]}}],\"version\":{{\"major\":1,\"minor\":0}}}}'\n",
                g = gtest.display(),
                c = catch2.display(),
                m = mudo.display()
            ),
        );
        let cancel = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let cases = discover_inner_cases(&ctest_entries(&ctest, &root, &cancel), &cancel);
        let ids: Vec<&str> = cases.iter().map(|c| c.id.as_str()).collect();
        assert_eq!(
            ids,
            [
                "unit_tests::Math.Adds",
                "unit_tests::Math.Subtracts",
                "catch::vectors can be sized"
            ]
        );
        assert_eq!(cases[0].name, "Math.Adds");

        // Um caso gtest: `--gtest_filter=` e o `[ OK ]` vira caso passado.
        let mut events = Vec::new();
        let r = run_inner_case(
            &ctest,
            &root,
            &format!("unit_tests{SEP}Math.Adds"),
            &cancel,
            &mut |e| events.push(e),
        )
        .expect("e' um case de dentro")
        .unwrap();
        assert!(r.success);
        assert_eq!(r.passed, 1);
        assert!(events.iter().any(|e| matches!(e, TestEvent::Output { line, .. } if line == "argv: --gtest_filter=Math.Adds --gtest_color=no --gtest_print_time=1 --gtest_brief=0")));
        assert!(
            events.iter().any(
                |e| matches!(e, TestEvent::Case { name, .. } if name == "unit_tests::Math.Adds")
            )
        );
        // Um caso Catch2: o nome e' o filtro; o exit code e' o desfecho.
        let mut events = Vec::new();
        let r = run_inner_case(
            &ctest,
            &root,
            "catch::vectors can be sized",
            &cancel,
            &mut |e| events.push(e),
        )
        .unwrap()
        .unwrap();
        assert_eq!(r.passed, 1);
        assert!(events.iter().any(|e| matches!(e, TestEvent::Output { line, .. } if line == "argv: vectors can be sized -r compact")));
        // Sem separador: nao e' caso de dentro.
        assert!(run_inner_case(&ctest, &root, "unit_tests", &cancel, &mut |_| {}).is_none());
        // Teste que o ctest nao lista: erro que diz.
        assert!(
            run_inner_case(&ctest, &root, "nao::x", &cancel, &mut |_| {})
                .unwrap()
                .is_err()
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    /// C1-34: a binary that registers its cases from the environment of the
    /// test lists them only with it, and the case runs with it too. C1-40: a
    /// case that passed or was skipped and did not end by the `TIMEOUT`
    /// counts only as failed.
    #[test]
    fn a_case_lists_and_runs_with_the_environment_and_the_timeout_of_its_test() {
        use super::{ctest_entries, discover_inner_cases, run_inner_case};
        use crate::test::TestEvent;

        let _serial = crate::serialize_executables();
        let root = std::env::temp_dir().join(format!("kinein-launch-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let exe = |name: &str, body: &str| -> std::path::PathBuf {
            let path = root.join(name);
            crate::write_executable(&path, format!("#!/bin/sh\n{body}"));
            path
        };
        let dynamic = exe(
            "dynamic",
            "case \"$1\" in\n  --gtest_list_tests) [ \"$KV_LIST\" = 1 ] || exit 1; printf 'Dyn.\\n  Case\\n'; exit 0;;\n  --gtest_filter=*) echo \"env: $KV_LIST\"; echo '[       OK ] Dyn.Case (0 ms)'; exit 0;;\n  *) exit 1;;\nesac\n",
        );
        let blocked = exe(
            "blocked",
            "case \"$1\" in\n  --gtest_list_tests) printf 'Late.\\n  Ok\\n  Skip\\n  Fail\\n'; exit 0;;\n  --gtest_filter=Late.Ok) echo '[       OK ] Late.Ok (0 ms)'; exec sleep 30;;\n  --gtest_filter=Late.Fail) echo '[  FAILED  ] Late.Fail (0 ms)'; exec sleep 30;;\n  --gtest_filter=Late.Skip) echo '[  SKIPPED ] Late.Skip (0 ms)'; exec sleep 30;;\n  *) exit 1;;\nesac\n",
        );
        // The paths go through the JSON escape: on Windows they have a `\`.
        let json = serde_json::json!({"tests": [
            {"name": "dyn", "command": [dynamic], "properties": [
                {"name": "ENVIRONMENT", "value": ["KV_LIST=1"]}]},
            {"name": "late", "command": [blocked], "properties": [
                {"name": "TIMEOUT", "value": 0.3}]},
        ]})
        .to_string();
        assert!(!json.contains('\''), "{json}");
        let ctest = exe("ctest", &format!("printf '%s' '{json}'\n"));
        let cancel = std::sync::Arc::default();
        let ids: Vec<_> = discover_inner_cases(&ctest_entries(&ctest, &root, &cancel), &cancel)
            .into_iter()
            .map(|case| case.id)
            .collect();
        assert_eq!(
            ids,
            [
                "dyn::Dyn.Case",
                "late::Late.Ok",
                "late::Late.Skip",
                "late::Late.Fail"
            ]
        );
        let mut events = Vec::new();
        let r = run_inner_case(&ctest, &root, "dyn::Dyn.Case", &cancel, &mut |e| {
            events.push(e);
        })
        .unwrap()
        .unwrap();
        assert_eq!(r.passed, 1);
        assert!(
            events
                .iter()
                .any(|e| matches!(e, TestEvent::Output { line, .. } if line == "env: 1"))
        );
        for id in ["late::Late.Ok", "late::Late.Skip", "late::Late.Fail"] {
            let r = run_inner_case(&ctest, &root, id, &cancel, &mut |_| {})
                .unwrap()
                .unwrap();
            assert_eq!(
                (r.success, r.passed, r.failed, r.ignored),
                (false, 0, 1, 0),
                "{id}"
            );
        }
        let _ = std::fs::remove_dir_all(&root);
    }

    /// C1-32: the listing is asked once per binary and folder. The same
    /// binary can list other cases in another folder, as a gtest that
    /// registers cases from the files there.
    #[test]
    fn the_listing_is_asked_once_per_binary_and_folder() {
        use super::{Framework, inner_cases};

        let exe = std::env::temp_dir().join(format!("kinein-listing-{}", std::process::id()));
        std::fs::write(&exe, "").unwrap();
        let entry = |name: &str, folder: &str| CtestEntry {
            name: name.to_owned(),
            working_directory: Some(folder.into()),
            ..entry_of(&exe)
        };
        let entries = [entry("here", "a"), entry("again", "a"), entry("there", "b")];
        let mut asked = Vec::new();
        let ids: Vec<_> = inner_cases(&entries, &std::sync::Arc::default(), &mut |entry| {
            let folder = entry.working_directory.as_ref()?.display().to_string();
            asked.push(folder.clone());
            Some((Framework::Gtest, vec![format!("Dir.{folder}")]))
        })
        .into_iter()
        .map(|case| case.id)
        .collect();
        assert_eq!(ids, ["here::Dir.a", "again::Dir.a", "there::Dir.b"]);
        assert_eq!(asked, ["a", "b"]);
        // C1-34: another environment can register other cases; C1-40: a
        // closed test does not ask its binary.
        let entries = [
            entry("plain", "a"),
            CtestEntry {
                environment: vec!["DATA=x".to_owned()],
                ..entry("data", "a")
            },
            CtestEntry {
                environment_modification: vec!["DATA=set:x".to_owned()],
                ..entry("modified", "a")
            },
            CtestEntry {
                outcome_properties: vec!["WILL_FAIL".to_owned()],
                ..entry("will_fail", "c")
            },
            CtestEntry {
                environment_modification: vec!["M=bogus:x".to_owned()],
                ..entry("bad", "d")
            },
        ];
        let mut asked = Vec::new();
        let ids: Vec<_> = inner_cases(&entries, &std::sync::Arc::default(), &mut |entry| {
            asked.push(entry.name.clone());
            Some((Framework::Gtest, vec!["Dir.x".to_owned()]))
        })
        .into_iter()
        .map(|case| case.id)
        .collect();
        assert_eq!(ids, ["plain::Dir.x", "data::Dir.x", "modified::Dir.x"]);
        assert_eq!(asked, ["plain", "data", "modified"]);
        let _ = std::fs::remove_file(&exe);
    }
}

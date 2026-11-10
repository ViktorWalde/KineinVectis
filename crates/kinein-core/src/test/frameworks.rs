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
    path::{Path, PathBuf},
    process::Command,
    sync::{Arc, atomic::AtomicBool},
};

use kinein_protocol::TestCaseInfo;

use super::{CaseStatus, TestError, TestEvent, TestOutcome, stream_command};
use crate::process;

/// O separador entre o teste do ctest e o caso de dentro do binario.
pub const SEP: &str = "::";

/// Um teste do ctest como o `json-v1` o descreve.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CtestEntry {
    /// `tests[].name`.
    pub name: String,
    /// `tests[].command` (executavel e argumentos).
    pub command: Vec<String>,
    /// `WORKING_DIRECTORY`, quando declarada.
    pub working_directory: Option<PathBuf>,
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
            let working_directory = t
                .get("properties")
                .and_then(serde_json::Value::as_array)
                .and_then(|ps| {
                    ps.iter().find_map(|p| {
                        if p.get("name")?.as_str()? != "WORKING_DIRECTORY" {
                            return None;
                        }
                        Some(PathBuf::from(p.get("value")?.as_str()?))
                    })
                });
            Some(CtestEntry {
                name,
                command,
                working_directory,
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

/// Pergunta ao binario quais casos ele tem: gtest primeiro, Catch2 depois.
/// `None` = o binario nao e' de nenhum dos dois (ou nao roda).
#[must_use]
pub fn list_cases(exe: &Path, cwd: Option<&Path>) -> Option<(Framework, Vec<String>)> {
    let run = |args: &[&str]| -> Option<String> {
        let mut c = Command::new(exe);
        c.args(args);
        if let Some(cwd) = cwd {
            c.current_dir(cwd);
        }
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

/// C1-31: Catch2 reads the name as a test spec, in which `\`, `,`, `[`,
/// `]`, `*`, `~` and `"` mean something. A backslash before each one keeps
/// the name (measured with Catch2 3.16.1).
fn catch2_filter(name: &str) -> String {
    let mut filter = String::with_capacity(name.len());
    for character in name.chars() {
        if matches!(character, '\\' | ',' | '[' | ']' | '*' | '~' | '"') {
            filter.push('\\');
        }
        filter.push(character);
    }
    filter
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

/// Os casos de dentro de cada binario do ctest, ja' com o id `teste::caso`.
pub fn discover_inner_cases(
    ctest: &Path,
    build_dir: &Path,
    cancel: &Arc<AtomicBool>,
) -> Vec<TestCaseInfo> {
    let mut cases = Vec::new();
    for entrada in ctest_entries(ctest, build_dir, cancel) {
        if cancel.load(std::sync::atomic::Ordering::SeqCst) {
            break;
        }
        let Some(exe) = entrada.command.first().map(Path::new) else {
            continue;
        };
        if !exe.is_file() {
            continue;
        }
        let Some((_, nomes)) = list_cases(exe, entrada.working_directory.as_deref()) else {
            continue;
        };
        for nome in nomes {
            cases.push(TestCaseInfo {
                id: format!("{}{SEP}{nome}", entrada.name),
                name: nome,
                file: None,
            });
        }
    }
    cases
}

/// Roda UM caso de dentro de um binario: `teste::caso`. `None` quando o id
/// nao tem o separador (e' um teste do ctest de sempre).
pub fn run_inner_case(
    ctest: &Path,
    build_dir: &Path,
    id: &str,
    cancel: &Arc<AtomicBool>,
    sink: &mut dyn FnMut(TestEvent),
) -> Option<Result<TestOutcome, TestError>> {
    let (teste, case) = id.split_once(SEP)?;
    let entradas = ctest_entries(ctest, build_dir, cancel);
    let Some(entrada) = entradas.iter().find(|e| e.name == teste) else {
        return Some(Err(TestError::ToolMissing {
            tool: teste.to_owned(),
            hint: format!("o ctest não lista o teste `{teste}` — reconfigure/recompile"),
        }));
    };
    let Some(exe) = entrada.command.first() else {
        return Some(Err(TestError::ToolMissing {
            tool: teste.to_owned(),
            hint: "o teste do ctest não tem comando".to_owned(),
        }));
    };
    let listed = list_cases(Path::new(exe), entrada.working_directory.as_deref());
    let catch2 = matches!(listed, Some((Framework::Catch2, _)));
    if catch2
        && let Some(reason) = listed
            .as_ref()
            .and_then(|(_, names)| catch2_refusal(case, names))
    {
        return Some(Err(TestError::Refused { reason }));
    }
    let mut command = Command::new(exe);
    let display = if catch2 {
        let filter = catch2_filter(case);
        command.arg(&filter).args(["-r", "compact"]);
        format!("{exe} \"{filter}\" -r compact")
    } else {
        command
            .arg(format!("--gtest_filter={case}"))
            .args(["--gtest_color=no", "--gtest_print_time=1"]);
        format!("{exe} --gtest_filter={case}")
    };
    if let Some(cwd) = &entrada.working_directory {
        command.current_dir(cwd);
    }
    let parse: fn(&str) -> Option<(String, CaseStatus)> =
        if catch2 { |_| None } else { parse_gtest_case };
    let mut resultado = stream_command(command, &display, parse, cancel, sink);
    // O Catch2 nao diz o caso linha a linha (o `-r compact` diz asserções):
    // o desfecho do caso e' o exit code.
    if catch2 && let Ok(outcome) = &mut resultado {
        let status = if outcome.success {
            outcome.passed += 1;
            CaseStatus::Passed
        } else {
            outcome.failed += 1;
            CaseStatus::Failed
        };
        sink(TestEvent::Case {
            name: case.to_owned(),
            status,
        });
    }
    Some(resultado)
}

#[cfg(test)]
mod tests {
    use super::{
        catch2_filter, catch2_refusal, parse_catch2_list, parse_ctest_json, parse_gtest_case,
        parse_gtest_list,
    };
    use crate::test::CaseStatus;

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
        use super::{Framework, SEP, discover_inner_cases, list_cases, run_inner_case};
        use crate::test::TestEvent;

        let _serial = crate::serialize_executables();

        let raiz = std::env::temp_dir().join(format!("kinein-frameworks-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&raiz);
        std::fs::create_dir_all(&raiz).unwrap();
        let exe = |nome: &str, corpo: &str| -> std::path::PathBuf {
            let p = raiz.join(nome);
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
            list_cases(&gtest, None),
            Some((
                Framework::Gtest,
                vec!["Math.Adds".to_owned(), "Math.Subtracts".to_owned()]
            ))
        );
        assert_eq!(
            list_cases(&catch2, None),
            Some((Framework::Catch2, vec!["vectors can be sized".to_owned()]))
        );
        assert_eq!(list_cases(&mudo, None), None);
        assert_eq!(list_cases(&printing, None), None);
        assert_eq!(list_cases(&other, None), None);

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
        let cases = discover_inner_cases(&ctest, &raiz, &cancel);
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
        let mut eventos = Vec::new();
        let r = run_inner_case(
            &ctest,
            &raiz,
            &format!("unit_tests{SEP}Math.Adds"),
            &cancel,
            &mut |e| eventos.push(e),
        )
        .expect("e' um case de dentro")
        .unwrap();
        assert!(r.success);
        assert_eq!(r.passed, 1);
        assert!(eventos.iter().any(|e| matches!(e, TestEvent::Output { line, .. } if line == "argv: --gtest_filter=Math.Adds --gtest_color=no --gtest_print_time=1")));
        assert!(
            eventos
                .iter()
                .any(|e| matches!(e, TestEvent::Case { name, .. } if name == "Math.Adds"))
        );
        // Um caso Catch2: o nome e' o filtro; o exit code e' o desfecho.
        let mut eventos = Vec::new();
        let r = run_inner_case(
            &ctest,
            &raiz,
            "catch::vectors can be sized",
            &cancel,
            &mut |e| eventos.push(e),
        )
        .unwrap()
        .unwrap();
        assert_eq!(r.passed, 1);
        assert!(eventos.iter().any(|e| matches!(e, TestEvent::Output { line, .. } if line == "argv: vectors can be sized -r compact")));
        // Sem separador: nao e' caso de dentro.
        assert!(run_inner_case(&ctest, &raiz, "unit_tests", &cancel, &mut |_| {}).is_none());
        // Teste que o ctest nao lista: erro que diz.
        assert!(
            run_inner_case(&ctest, &raiz, "nao::x", &cancel, &mut |_| {})
                .unwrap()
                .is_err()
        );
        let _ = std::fs::remove_dir_all(&raiz);
    }
}

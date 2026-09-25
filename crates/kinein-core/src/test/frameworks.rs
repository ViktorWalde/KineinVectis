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
    let mut casos = Vec::new();
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
        let caso = linha.split('#').next().unwrap_or_default().trim();
        if !caso.is_empty() && !suite.is_empty() {
            casos.push(format!("{suite}{caso}"));
        }
    }
    casos
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
#[must_use]
pub fn parse_gtest_case(line: &str) -> Option<(String, CaseStatus)> {
    let (status, resto) = if let Some(r) = line.strip_prefix("[       OK ] ") {
        (CaseStatus::Passed, r)
    } else if let Some(r) = line.strip_prefix("[  FAILED  ] ") {
        (CaseStatus::Failed, r)
    } else if let Some(r) = line.strip_prefix("[  SKIPPED ] ") {
        (CaseStatus::Ignored, r)
    } else {
        return None;
    };
    let nome = resto.split(" (").next()?.trim();
    (!nome.is_empty() && nome.contains('.')).then(|| (nome.to_owned(), status))
}

/// Pergunta ao binario quais casos ele tem: gtest primeiro, Catch2 depois.
/// `None` = o binario nao e' de nenhum dos dois (ou nao roda).
#[must_use]
pub fn list_cases(exe: &Path, cwd: Option<&Path>) -> Option<(Framework, Vec<String>)> {
    let rodar = |args: &[&str]| -> Option<String> {
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
    if let Some(texto) = rodar(&["--gtest_list_tests"]) {
        let casos = parse_gtest_list(&texto);
        if !casos.is_empty() {
            return Some((Framework::Gtest, casos));
        }
    }
    if let Some(texto) = rodar(&["--list-tests", "--verbosity", "quiet"]) {
        let casos = parse_catch2_list(&texto);
        if !casos.is_empty() {
            return Some((Framework::Catch2, casos));
        }
    }
    None
}

/// Os testes do ctest com os comandos, pelo `json-v1`. `ctest` e' o programa
/// (o do PATH, ou um falso no teste).
pub fn ctest_entries(ctest: &Path, build_dir: &Path, cancel: &Arc<AtomicBool>) -> Vec<CtestEntry> {
    let mut c = Command::new(ctest);
    c.arg("--test-dir")
        .arg(build_dir)
        .arg("--show-only=json-v1");
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
    let mut casos = Vec::new();
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
            casos.push(TestCaseInfo {
                id: format!("{}{SEP}{nome}", entrada.name),
                name: nome,
                file: None,
            });
        }
    }
    casos
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
    let (teste, caso) = id.split_once(SEP)?;
    let entradas = ctest_entries(ctest, build_dir, cancel);
    let Some(entrada) = entradas.iter().find(|e| e.name == teste) else {
        return Some(Err(TestError::ToolMissing {
            tool: teste.to_owned(),
            hint: format!("o ctest nao lista o teste `{teste}` — reconfigure/recompile"),
        }));
    };
    let Some(exe) = entrada.command.first() else {
        return Some(Err(TestError::ToolMissing {
            tool: teste.to_owned(),
            hint: "o teste do ctest nao tem comando".to_owned(),
        }));
    };
    let framework =
        list_cases(Path::new(exe), entrada.working_directory.as_deref()).map(|(f, _)| f);
    let mut command = Command::new(exe);
    let catch2 = framework == Some(Framework::Catch2);
    let display = if catch2 {
        command.arg(caso).args(["-r", "compact"]);
        format!("{exe} \"{caso}\" -r compact")
    } else {
        command
            .arg(format!("--gtest_filter={caso}"))
            .arg("--gtest_color=no");
        format!("{exe} --gtest_filter={caso}")
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
            name: caso.to_owned(),
            status,
        });
    }
    Some(resultado)
}

#[cfg(test)]
mod tests {
    use super::{parse_catch2_list, parse_ctest_json, parse_gtest_case, parse_gtest_list};
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

    /// Um binario gtest falso e um Catch2 falso respondem a listagem certa;
    /// um binario mudo nao vira caso. E o caso de dentro roda pelo binario,
    /// com o filtro do framework, a partir do `json-v1` de um ctest falso.
    #[test]
    #[cfg(unix)]
    fn binaries_are_asked_for_their_cases_and_one_case_runs_by_the_binary() {
        use std::os::unix::fs::PermissionsExt;

        use super::{Framework, SEP, discover_inner_cases, list_cases, run_inner_case};
        use crate::test::TestEvent;

        let _serial = crate::serializar_executaveis();

        let raiz = std::env::temp_dir().join(format!("kinein-frameworks-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&raiz);
        std::fs::create_dir_all(&raiz).unwrap();
        let exe = |nome: &str, corpo: &str| -> std::path::PathBuf {
            let p = raiz.join(nome);
            std::fs::write(&p, format!("#!/bin/sh\n{corpo}")).unwrap();
            std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o755)).unwrap();
            p
        };
        let gtest = exe(
            "unit",
            "case \"$1\" in\n  --gtest_list_tests) printf 'Math.\\n  Adds\\n  Subtracts\\n'; exit 0;;\n  --gtest_filter=*) echo \"argv: $*\"; echo '[       OK ] Math.Adds (0 ms)'; exit 0;;\n  *) exit 1;;\nesac\n",
        );
        let catch2 = exe(
            "c2",
            "case \"$1\" in\n  --gtest_list_tests) exit 1;;\n  --list-tests) printf 'vectors can be sized\\n'; exit 0;;\n  *) echo \"argv: $*\"; exit 0;;\nesac\n",
        );
        let mudo = exe("mudo", "exit 1\n");
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
        let casos = discover_inner_cases(&ctest, &raiz, &cancel);
        let ids: Vec<&str> = casos.iter().map(|c| c.id.as_str()).collect();
        assert_eq!(
            ids,
            [
                "unit_tests::Math.Adds",
                "unit_tests::Math.Subtracts",
                "catch::vectors can be sized"
            ]
        );
        assert_eq!(casos[0].name, "Math.Adds");

        // Um caso gtest: `--gtest_filter=` e o `[ OK ]` vira caso passado.
        let mut eventos = Vec::new();
        let r = run_inner_case(
            &ctest,
            &raiz,
            &format!("unit_tests{SEP}Math.Adds"),
            &cancel,
            &mut |e| eventos.push(e),
        )
        .expect("e' um caso de dentro")
        .unwrap();
        assert!(r.success);
        assert_eq!(r.passed, 1);
        assert!(eventos.iter().any(|e| matches!(e, TestEvent::Output { line, .. } if line == "argv: --gtest_filter=Math.Adds --gtest_color=no")));
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

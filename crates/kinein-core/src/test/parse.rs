//! Os PARSERS da saida de cada runner: uma linha de `cargo test`, `pytest
//! -v` ou `ctest` vira `(nome, estado)` — ou nada, para o resumo e o ruido.
//! Saiu do `mod.rs` em 2026-09-13 junto com os runners.

use super::CaseStatus;

/// Parses a libtest case line: `test <name> ... <outcome>`.
///
/// The summary line `test result: ...` is intentionally not matched: it lacks
/// the ` ... ` separator that every case line carries.
pub(super) fn parse_cargo_case(line: &str) -> Option<(String, CaseStatus)> {
    let rest = line.trim().strip_prefix("test ")?;
    let separator = rest.rfind(" ... ")?;
    let name = rest[..separator].trim();
    if name.is_empty() {
        return None;
    }
    let outcome = rest[separator + " ... ".len()..].trim();
    let status = match outcome {
        "ok" => CaseStatus::Passed,
        "FAILED" => CaseStatus::Failed,
        // "ignored" or "ignored, <reason>".
        _ if outcome == "ignored" || outcome.starts_with("ignored,") => CaseStatus::Ignored,
        // Benchmarks and anything else are not pass/fail cases.
        _ => return None,
    };
    Some((name.to_owned(), status))
}

/// Parses a pytest `-v` case line: `tests/test_x.py::test_a PASSED [ 50%]`.
///
/// O nome vem ANTES do estado e sempre tem `::` (`arquivo::caso`, com
/// parametros entre colchetes, que podem ter espaco); o estado e' a ULTIMA
/// palavra-chave da linha antes do `[ nn%]`. Linhas do resumo (`FAILED
/// tests/x.py::a - assert`) comecam pelo estado e nao casam — senao cada
/// falha contaria duas vezes.
pub(super) fn parse_pytest_case(line: &str) -> Option<(String, CaseStatus)> {
    const ESTADOS: [(&str, CaseStatus); 6] = [
        ("PASSED", CaseStatus::Passed),
        ("XPASS", CaseStatus::Passed),
        ("FAILED", CaseStatus::Failed),
        ("ERROR", CaseStatus::Failed),
        ("SKIPPED", CaseStatus::Ignored),
        ("XFAIL", CaseStatus::Ignored),
    ];
    let line = line.trim();
    let mut melhor: Option<(usize, CaseStatus)> = None;
    for (palavra, status) in ESTADOS {
        let marcador = format!(" {palavra}");
        if let Some(pos) = line.rfind(&marcador) {
            if melhor.is_none_or(|(p, _)| pos > p) {
                melhor = Some((pos, status));
            }
        }
    }
    let (pos, status) = melhor?;
    let name = line[..pos].trim();
    if name.is_empty() || !name.contains("::") {
        return None;
    }
    Some((name.to_owned(), status))
}

/// Parses a ctest case line: `1/3 Test #1: Name .... Passed|***Failed`.
pub(super) fn parse_ctest_case(line: &str) -> Option<(String, CaseStatus)> {
    let marker = line.find("Test #")?;
    let after_hash = line[marker + "Test #".len()..].split_once(':')?.1;
    // `after_hash` is like `  Name .................   Passed    0.01 sec`.
    let status = if line.contains("***Failed") || line.contains("***Not Run") {
        CaseStatus::Failed
    } else if line.contains("***Skipped") || line.contains("***Disabled") {
        CaseStatus::Ignored
    } else if line.contains("   Passed") {
        CaseStatus::Passed
    } else {
        return None;
    };
    let name = after_hash
        .trim_start()
        .split(" ..")
        .next()?
        .split("   ")
        .next()?
        .trim();
    if name.is_empty() {
        return None;
    }
    Some((name.to_owned(), status))
}

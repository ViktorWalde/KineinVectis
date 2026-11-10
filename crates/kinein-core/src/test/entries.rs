//! C1-32: which tests of the ctest the tree opens into the cases of their
//! binary, and which case an id `test::case` names (consolidação C §21).
//!
//! `gtest_discover_tests` writes one test of the ctest per case, with
//! `--gtest_filter=<case>` in the command, and `catch_discover_tests` one per
//! case, with the escaped name first (measured with `CMake` 4.4.5, gtest 1.18
//! and Catch2 3.16.1). Opened, such a test gave every case of the binary once
//! per case. Only the test that runs every case is opened; an option the IDE
//! does not know leaves it closed, and it still runs whole by its line.

use std::collections::HashMap;

use kinein_protocol::TestCaseInfo;

use super::TestError;
use super::frameworks::{CtestEntry, Framework, Listing, SEP};

/// What gtest 1.18 reads in the command to pick the cases. With a space,
/// `--gtest_filter Math.Adds` prints the help and runs none.
const GTEST_PICKING: [&str; 3] = ["--gtest_filter", "--gtest_flagfile", "--gtest_list_tests"];

/// The variables gtest 1.18 reads to pick the cases; it does not read
/// `GTEST_FLAGFILE`.
const GTEST_PICKING_VARIABLES: [&str; 3] =
    ["GTEST_FILTER", "GTEST_TOTAL_SHARDS", "GTEST_SHARD_INDEX"];

/// The options of Catch2 3.16.1 that take a value and do not pick the cases
/// (`--help`). `-f`, `--shard-count` and `--shard-index` pick them.
const CATCH2_VALUE_OPTIONS: &[&str] = &[
    "-o",
    "--out",
    "-r",
    "--reporter",
    "-n",
    "--name",
    "-x",
    "--abortx",
    "-w",
    "--warn",
    "-d",
    "--durations",
    "-D",
    "--min-duration",
    "-c",
    "--section",
    "-g",
    "--generator-index",
    "-p",
    "--path-filter",
    "-v",
    "--verbosity",
    "--order",
    "--rng-seed",
    "--colour-mode",
    "--wait-for-keypress",
    "--benchmark-samples",
    "--benchmark-resamples",
    "--benchmark-confidence-interval",
    "--benchmark-warmup-time",
    "--premature-exit-guard-file",
];

/// The options of Catch2 3.16.1 without a value that still run every case.
/// The listings and the help run none.
const CATCH2_FLAGS: &[&str] = &[
    "-s",
    "--success",
    "-b",
    "--break",
    "-e",
    "--nothrow",
    "-i",
    "--invisibles",
    "-a",
    "--abort",
    "-#",
    "--filenames-as-tags",
    "--skip-benchmarks",
    "--benchmark-no-analysis",
    "--allow-running-no-tests",
];

/// Whether the test of the ctest runs every case of its binary: the test the
/// tree opens into cases.
pub(super) fn runs_every_case(framework: Framework, entry: &CtestEntry) -> bool {
    let arguments = entry.command.get(1..).unwrap_or_default();
    match framework {
        Framework::Gtest => {
            // Windows compares the names of the variables without the case.
            let picking = |variable: &String| {
                let name = variable.split('=').next().unwrap_or_default();
                GTEST_PICKING_VARIABLES
                    .iter()
                    .any(|known| known.eq_ignore_ascii_case(name))
            };
            !arguments
                .iter()
                .any(|argument| GTEST_PICKING.iter().any(|flag| argument.starts_with(flag)))
                && !entry
                    .environment
                    .iter()
                    .chain(&entry.environment_modification)
                    .any(picking)
        }
        Framework::Catch2 => catch2_runs_every_case(arguments),
    }
}

/// Catch2 reads the arguments as Clara does (measured with 3.16.1): one that
/// does not start with `-` is a test spec; `--order decl`, `--order=decl`
/// and `--order:decl` are the option with its value; `-sa` is `-s -a`.
fn catch2_runs_every_case(arguments: &[String]) -> bool {
    let mut arguments = arguments.iter();
    while let Some(argument) = arguments.next() {
        if !argument.starts_with('-') {
            return false;
        }
        let (names, inline) = match argument.find([' ', ':', '=']) {
            Some(at) => (vec![argument[..at].to_owned()], true),
            None if !argument.starts_with("--") && argument.len() > 2 => (
                argument[1..]
                    .chars()
                    .map(|letter| format!("-{letter}"))
                    .collect(),
                false,
            ),
            None => (vec![argument.clone()], false),
        };
        for (index, name) in names.iter().enumerate() {
            if CATCH2_VALUE_OPTIONS.contains(&name.as_str()) {
                // Clara gives the next argument to the last of `-sr compact`.
                if index + 1 < names.len() || (!inline && arguments.next().is_none()) {
                    return false;
                }
            } else if inline || !CATCH2_FLAGS.contains(&name.as_str()) {
                // The value of a flag is left as a test spec.
                return false;
            }
        }
    }
    true
}

/// The cases without an id that another test or case has: running such an
/// id could not tell which one it is.
pub(super) fn unique(cases: Vec<TestCaseInfo>, entries: &[CtestEntry]) -> Vec<TestCaseInfo> {
    let mut count: HashMap<String, usize> = HashMap::new();
    for case in &cases {
        *count.entry(case.id.clone()).or_default() += 1;
    }
    cases
        .into_iter()
        .filter(|case| count[&case.id] == 1 && !entries.iter().any(|entry| entry.name == case.id))
        .collect()
}

/// The one case an id names, with the framework and the listing of its
/// binary.
pub(super) struct Named<'a> {
    pub(super) entry: &'a CtestEntry,
    pub(super) case: &'a str,
    pub(super) framework: Framework,
    pub(super) cases: Vec<String>,
}

/// What an id `test::case` names: `None` when it is a test of the ctest,
/// also with `::` in its name, as `catch_discover_tests` writes
/// `Widget::draw works`.
pub(super) fn resolve<'a>(
    entries: &'a [CtestEntry],
    id: &'a str,
    list: &mut dyn FnMut(&CtestEntry) -> Listing,
) -> Option<Result<Named<'a>, TestError>> {
    if entries.iter().any(|entry| entry.name == id) {
        return None;
    }
    let mut named = Vec::new();
    let mut reason = None;
    // The test with the longest name is the one the id names best.
    let mut keep = |name: &str, text: &dyn Fn() -> String| {
        if reason
            .as_ref()
            .is_none_or(|(length, _): &(usize, String)| name.len() > *length)
        {
            reason = Some((name.len(), text()));
        }
    };
    for entry in entries {
        let Some(case) = id
            .strip_prefix(entry.name.as_str())
            .and_then(|rest| rest.strip_prefix(SEP))
        else {
            continue;
        };
        let name = &entry.name;
        match list(entry) {
            Some((framework, _)) if !runs_every_case(framework, entry) => {
                keep(name, &|| {
                    format!(
                        "o teste `{name}` do ctest já escolhe os casos e não se abre em \
                         casos; rode a linha dele"
                    )
                });
            }
            Some((framework, cases)) if cases.iter().any(|known| known == case) => {
                named.push(Named {
                    entry,
                    case,
                    framework,
                    cases,
                });
            }
            _ => {
                keep(name, &|| {
                    format!(
                        "o binário do teste `{name}` não lista mais o caso `{case}`; \
                         recompile e descubra os testes de novo"
                    )
                });
            }
        }
    }
    let refused = |reason: String| Some(Err(TestError::Refused { reason }));
    match named.len() {
        1 => named.pop().map(Ok),
        0 => refused(reason.map_or_else(
            || format!("o ctest não lista um teste para `{id}`; configure e compile de novo"),
            |(_, text)| text,
        )),
        _ => {
            let both: Vec<_> = named
                .iter()
                .map(|named| format!("`{}` do teste `{}`", named.case, named.entry.name))
                .collect();
            refused(format!(
                "o id `{id}` é de mais de um caso: {}; rode a linha do ctest",
                both.join(" e ")
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{resolve, runs_every_case, unique};
    use crate::test::frameworks::{CtestEntry, Framework, Listing};
    use kinein_protocol::TestCaseInfo;

    fn entry(
        name: &str,
        command: &[&str],
        environment: &[&str],
        modification: &[&str],
    ) -> CtestEntry {
        let strings = |items: &[&str]| items.iter().map(|&item| item.to_owned()).collect();
        CtestEntry {
            name: name.to_owned(),
            command: strings(command),
            working_directory: None,
            environment: strings(environment),
            environment_modification: strings(modification),
        }
    }

    /// What gtest 1.18 reads to pick the cases, measured on 2026-11-10.
    #[test]
    fn a_gtest_test_opens_only_when_nothing_picks_the_cases() {
        let opens = |arguments: &[&str], environment: &[&str], modification: &[&str]| {
            let mut command = vec!["/b/gt"];
            command.extend(arguments);
            runs_every_case(
                Framework::Gtest,
                &entry("t", &command, environment, modification),
            )
        };
        for arguments in [
            &[][..],
            &["--gtest_color=no"],
            &["--gtest_also_run_disabled_tests"],
            &["Math.Adds"],
            &["-gtest_filter=Math.Adds"],
        ] {
            assert!(opens(arguments, &[], &[]), "{arguments:?}");
        }
        for arguments in [
            // `gtest_discover_tests`.
            &[
                "--gtest_filter=Math.Adds",
                "--gtest_also_run_disabled_tests",
            ][..],
            &["--gtest_color=no", "--gtest_filter=Math.*"],
            &["--gtest_filter", "Math.Adds"],
            &["--gtest_flagfile=flags.txt"],
            &["--gtest_list_tests"],
        ] {
            assert!(!opens(arguments, &[], &[]), "{arguments:?}");
        }
        assert!(opens(
            &[],
            &["OTHER=1", "GTEST_COLOR=no", "GTEST_FLAGFILE=f"],
            &["PATH=path_list_append:/x"]
        ));
        for (environment, modification) in [
            (&["OTHER=1", "GTEST_FILTER=Math.*"][..], &[][..]),
            (&["gtest_filter=Math.*"], &[]),
            (&["GTEST_TOTAL_SHARDS=2"], &[]),
            (&[], &["GTEST_SHARD_INDEX=set:0"]),
            (&[], &["GTEST_FILTER=unset:"]),
        ] {
            assert!(
                !opens(&[], environment, modification),
                "{environment:?} {modification:?}"
            );
        }
    }

    /// How Clara reads the arguments of Catch2 3.16.1, measured on 2026-11-10.
    #[test]
    fn a_catch2_test_opens_only_when_every_argument_is_known_and_picks_no_case() {
        let opens = |arguments: &[&str]| {
            let mut command = vec!["/b/c2"];
            command.extend(arguments);
            runs_every_case(Framework::Catch2, &entry("t", &command, &[], &[]))
        };
        for arguments in [
            &[][..],
            &["-s"],
            &["-r", "compact"],
            &["-sr", "compact"],
            &["--order decl"],
            &["--order=decl"],
            &["--order:decl"],
            &["-o", "out.xml", "-r", "junit", "-#"],
            &["-c", "a section"],
            &["-p", "c:plain"],
            &["--rng-seed", "7", "--allow-running-no-tests"],
            &["--success"],
        ] {
            assert!(opens(arguments), "{arguments:?}");
        }
        for arguments in [
            // `catch_discover_tests`: the escaped name, `EXTRA_ARGS` and the order.
            &["a\\, b \\[x\\]", "-s", "--order decl"][..],
            &["-r", "compact", "plain"],
            &["[fast]"],
            &["-r"],
            &["-rcompact"],
            &["-rs", "compact"],
            &["--success=yes"],
            &["-f", "specs.txt"],
            &["--input-file=specs.txt"],
            &["--shard-count", "2", "--shard-index", "0"],
            &["--list-tests"],
            &["--libidentify"],
            &["-h"],
            &["--mystery"],
            &["-"],
            &["--"],
            &[""],
        ] {
            assert!(!opens(arguments), "{arguments:?}");
        }
    }

    #[test]
    fn an_id_another_test_or_case_has_is_left_out() {
        let case = |id: &str| TestCaseInfo {
            id: id.to_owned(),
            name: id.rsplit("::").next().unwrap_or_default().to_owned(),
            file: None,
        };
        let entries = [
            entry("dup", &[], &[], &[]),
            entry("dup::x", &[], &[], &[]),
            entry("gt::Math.Adds", &[], &[], &[]),
        ];
        let kept: Vec<_> = unique(
            [
                "dup::x::y",
                "dup::y",
                "dup::x::y",
                "gt::Math.Adds",
                "gt::Math.Fails",
                "dup::x::x::y",
            ]
            .map(case)
            .into(),
            &entries,
        )
        .into_iter()
        .map(|case| case.id)
        .collect();
        assert_eq!(kept, ["dup::y", "gt::Math.Fails", "dup::x::x::y"]);
    }

    /// The tests of the ctest of the ids below; `ns` comes first, and the
    /// reason is still that of `ns::unit`, the longer name.
    fn named_entries() -> Vec<CtestEntry> {
        vec![
            entry("ns", &["/b/none"], &[], &[]),
            entry("ns::unit", &["/b/gt"], &[], &[]),
            entry(
                "Widget::draw works",
                &["/b/c2", "Widget::draw works"],
                &[],
                &[],
            ),
            entry("gt_math", &["/b/gt", "--gtest_filter=Math.*"], &[], &[]),
            entry("dup", &["/b/c2"], &[], &[]),
            entry("dup::x", &["/b/c2"], &[], &[]),
        ]
    }

    /// What `resolve` gave for each id, and the tests whose binary it asked.
    type Resolved = (
        Vec<Option<Result<(String, String, Framework), String>>>,
        Vec<String>,
    );

    fn resolved(ids: &[&str]) -> Resolved {
        let entries = named_entries();
        let mut asked = Vec::new();
        let mut results = Vec::new();
        for id in ids {
            let mut list = |entry: &CtestEntry| -> Listing {
                asked.push(entry.name.clone());
                let names = |items: &[&str]| items.iter().map(|&item| item.to_owned()).collect();
                match entry.command.first().map(String::as_str) {
                    Some("/b/gt") => {
                        Some((Framework::Gtest, names(&["Math.Adds", "Text.DISABLED_Off"])))
                    }
                    Some("/b/c2") => Some((
                        Framework::Catch2,
                        names(&["Widget::draw works", "x::y", "y"]),
                    )),
                    _ => None,
                }
            };
            results.push(resolve(&entries, id, &mut list).map(|named| {
                named
                    .map(|named| {
                        (
                            named.entry.name.clone(),
                            named.case.to_owned(),
                            named.framework,
                        )
                    })
                    .map_err(|error| error.to_string())
            }));
        }
        (results, asked)
    }

    #[test]
    fn an_id_names_the_known_test_and_one_listed_case() {
        let (results, asked) =
            resolved(&["Widget::draw works", "ns::unit::Math.Adds", "dup::x::x::y"]);
        let named = |test: &str, case: &str, framework| {
            Some(Ok((test.to_owned(), case.to_owned(), framework)))
        };
        assert_eq!(
            results,
            [
                None,
                named("ns::unit", "Math.Adds", Framework::Gtest),
                named("dup::x", "x::y", Framework::Catch2),
            ]
        );
        // Only a test whose name starts the id is listed, never the line of
        // the ctest the id is.
        assert_eq!(asked, ["ns", "ns::unit", "dup", "dup::x"]);
    }

    #[test]
    fn an_id_without_one_case_is_refused_with_the_reason() {
        let (results, asked) = resolved(&[
            "dup::x::y",
            "gt_math::Math.Adds",
            "ns::unit::Math.Gone",
            "nope::Math.Adds",
            // `ns::unit` does not start `ns::unitx::…`: the name ends at `::`.
            "ns::unitx::Math.Adds",
        ]);
        let refused = |reason: &str| Some(Err(reason.to_owned()));
        assert_eq!(
            results,
            [
                refused(
                    "o id `dup::x::y` é de mais de um caso: `x::y` do teste `dup` e `y` do teste \
                     `dup::x`; rode a linha do ctest"
                ),
                refused(
                    "o teste `gt_math` do ctest já escolhe os casos e não se abre em casos; rode \
                     a linha dele"
                ),
                refused(
                    "o binário do teste `ns::unit` não lista mais o caso `Math.Gone`; recompile \
                     e descubra os testes de novo"
                ),
                refused(
                    "o ctest não lista um teste para `nope::Math.Adds`; configure e compile de novo"
                ),
                refused(
                    "o binário do teste `ns` não lista mais o caso `unitx::Math.Adds`; recompile \
                     e descubra os testes de novo"
                ),
            ]
        );
        assert_eq!(asked, ["dup", "dup::x", "gt_math", "ns", "ns::unit", "ns"]);
    }
}

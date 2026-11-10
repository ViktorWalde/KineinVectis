//! C1-34: one case of a binary runs as the ctest runs its test (consolidação
//! C §22). The environment is the one the ctest gives the test, also to the
//! listing; the arguments of the `add_test` come first and the flags of the
//! IDE last; the command shown is the one that runs (C1-38).

use std::{
    collections::HashMap,
    ffi::{OsStr, OsString},
    process::Command,
};

use super::entries;
use super::frameworks::{CtestEntry, Framework};

/// What the ctest of `CMake` 4.4.5 puts in the environment of every test,
/// before the `ENVIRONMENT`.
const INTERACTIVE: [&str; 2] = ["CTEST_INTERACTIVE_DEBUG_MODE", "1"];

/// The separator of `path_list_append` and `path_list_prepend`: the one of
/// the `PATH`.
const PATH_LIST: char = if cfg!(windows) { ';' } else { ':' };

/// One entry of the `ENVIRONMENT_MODIFICATION`.
enum Operation<'a> {
    /// Back to the value the `ENVIRONMENT` left.
    Reset,
    Set(&'a str),
    Unset,
    /// The value after or before the current one, with the separator when
    /// the current one is not empty.
    Join {
        value: &'a str,
        separator: Option<char>,
        before: bool,
    },
}

/// The entries of the `ENVIRONMENT_MODIFICATION` of the test, cut as the
/// ctest cuts them: at the first `=` and then at the first `:`. `Err` is why
/// the ctest does not run the test.
fn modifications(entry: &CtestEntry) -> Result<Vec<(&str, Operation<'_>)>, String> {
    entry
        .environment_modification
        .iter()
        .map(|text| {
            let refused = |why: &str| {
                format!(
                    "o CTest não roda o teste `{}`: o `ENVIRONMENT_MODIFICATION` `{text}` \
                     {why}; corrija o teste no CMake",
                    entry.name
                )
            };
            let (name, rest) = text
                .split_once('=')
                .ok_or_else(|| refused("não tem o `=` depois do nome"))?;
            let (operation, value) = rest
                .split_once(':')
                .ok_or_else(|| refused("não tem o `:` depois da operação"))?;
            let join = |separator, before| Operation::Join {
                value,
                separator,
                before,
            };
            let operation = match operation {
                "reset" => Operation::Reset,
                "set" => Operation::Set(value),
                "unset" => Operation::Unset,
                "string_append" => join(None, false),
                "string_prepend" => join(None, true),
                "path_list_append" => join(Some(PATH_LIST), false),
                "path_list_prepend" => join(Some(PATH_LIST), true),
                "cmake_list_append" => join(Some(';'), false),
                "cmake_list_prepend" => join(Some(';'), true),
                other => {
                    return Err(refused(&format!(
                        "tem a operação `{other}`, que ele não conhece"
                    )));
                }
            };
            Ok((name, operation))
        })
        .collect()
}

/// Why the ctest does not run the test: an `ENVIRONMENT_MODIFICATION` it
/// refuses (C1-40).
pub(super) fn environment_refusal(entry: &CtestEntry) -> Option<String> {
    modifications(entry).err()
}

/// Puts on `command` the environment the ctest gives the test: the one of
/// Visual Studio that the `ctest` gets, `CTEST_INTERACTIVE_DEBUG_MODE=1`, the
/// `ENVIRONMENT` in order, and then the `ENVIRONMENT_MODIFICATION`. `Err` is
/// why the ctest does not run the test.
pub(super) fn apply_environment(command: &mut Command, entry: &CtestEntry) -> Result<(), String> {
    let modifications = modifications(entry)?;
    crate::msvc::apply_environment(command);
    command.env(INTERACTIVE[0], INTERACTIVE[1]);
    for variable in &entry.environment {
        // Measured on Linux: an entry without `=` takes the variable out.
        match variable.split_once('=') {
            Some((name, value)) => command.env(name, value),
            None => command.env_remove(variable),
        };
    }
    // `None` takes the variable out. A name that is not here keeps what the
    // `ENVIRONMENT` left, which is what `reset` goes back to.
    let mut changed: HashMap<String, (&str, Option<OsString>)> = HashMap::new();
    for (name, operation) in modifications {
        let key = variable_key(name);
        let value = match operation {
            Operation::Reset => {
                changed.remove(&key);
                continue;
            }
            Operation::Set(value) => Some(OsString::from(value)),
            Operation::Unset => None,
            Operation::Join {
                value,
                separator,
                before,
            } => {
                let current = match changed.get(&key) {
                    Some((_, current)) => current.clone(),
                    None => current_value(command, name),
                };
                Some(join(current.unwrap_or_default(), value, separator, before))
            }
        };
        changed.insert(key, (name, value));
    }
    for (name, value) in changed.into_values() {
        match value {
            Some(value) => command.env(name, value),
            None => command.env_remove(name),
        };
    }
    Ok(())
}

fn join(current: OsString, value: &str, separator: Option<char>, before: bool) -> OsString {
    let separator = separator
        .filter(|_| !current.is_empty())
        .map(String::from)
        .unwrap_or_default();
    let mut joined = OsString::new();
    if before {
        joined.push(value);
        joined.push(separator);
        joined.push(current);
    } else {
        joined.push(current);
        joined.push(separator);
        joined.push(value);
    }
    joined
}

/// The name as the system compares it: Windows without the case.
fn variable_key(name: &str) -> String {
    if cfg!(windows) {
        name.to_ascii_uppercase()
    } else {
        name.to_owned()
    }
}

/// The value the test would get for `name` now: what `command` sets or takes
/// out, or the one of the process.
fn current_value(command: &Command, name: &str) -> Option<OsString> {
    let key = variable_key(name);
    command
        .get_envs()
        .find(|(known, _)| {
            known
                .to_str()
                .is_some_and(|known| variable_key(known) == key)
        })
        .map_or_else(
            || std::env::var_os(name),
            |(_, value)| value.map(OsStr::to_owned),
        )
}

/// C1-31: Catch2 reads the name as a test spec, in which `\`, `,`, `[`,
/// `]`, `*`, `~` and `"` mean something. A backslash before each one keeps
/// the name (measured with Catch2 3.16.1).
pub(super) fn catch2_filter(name: &str) -> String {
    let mut filter = String::with_capacity(name.len());
    for character in name.chars() {
        if matches!(character, '\\' | ',' | '[' | ']' | '*' | '~' | '"') {
            filter.push('\\');
        }
        filter.push(character);
    }
    filter
}

/// The command that runs one case of the binary of the test, with the
/// working directory, and the text shown of it, from the same arguments
/// (C1-38). The environment is [`apply_environment`].
pub(super) fn case_command(
    entry: &CtestEntry,
    framework: Framework,
    case: &str,
) -> Option<(Command, String)> {
    let (exe, arguments) = entry.command.split_first()?;
    let mut words = arguments.to_vec();
    match framework {
        // gtest 1.18 uses the last of two equal flags. C1-35:
        // `--gtest_brief=0` wins over a `GTEST_BRIEF=1`, which hides the line
        // of the case that passed.
        Framework::Gtest => words.extend([
            format!("--gtest_filter={case}"),
            "--gtest_color=no".to_owned(),
            "--gtest_print_time=1".to_owned(),
            "--gtest_brief=0".to_owned(),
        ]),
        // Two reporters on the standard output abort Catch2 3.16.1.
        Framework::Catch2 => {
            words.push(catch2_filter(case));
            if !entries::catch2_picks_reporter(arguments) {
                words.extend(["-r".to_owned(), "compact".to_owned()]);
            }
        }
    }
    let mut command = Command::new(exe);
    command.args(&words);
    if let Some(cwd) = &entry.working_directory {
        command.current_dir(cwd);
    }
    let shown = std::iter::once(crate::platform::program_invocation(exe))
        .chain(
            words
                .iter()
                .map(|word| crate::platform::shell_argument(word)),
        )
        .collect::<Vec<_>>()
        .join(" ");
    Some((command, shown))
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, process::Command};

    use super::{PATH_LIST, apply_environment, case_command, environment_refusal};
    use crate::test::frameworks::{CtestEntry, Framework};

    /// The names of the measure, with a prefix no process has.
    fn name(letter: &str) -> String {
        format!("KV34_{letter}")
    }

    fn entry(environment: &[&str], modification: &[&str]) -> CtestEntry {
        let named = |items: &[&str]| -> Vec<String> {
            items
                .iter()
                .map(|item| {
                    if item.starts_with('=') || item.starts_with("CTEST_") {
                        (*item).to_owned()
                    } else {
                        name(item)
                    }
                })
                .collect()
        };
        CtestEntry {
            name: "t".to_owned(),
            command: vec!["/b/gt".to_owned()],
            environment: named(environment),
            environment_modification: named(modification),
            ..CtestEntry::default()
        }
    }

    /// The environment the test gets: what the command sets or takes out,
    /// over the one of the process.
    fn seen(command: &Command) -> HashMap<String, Option<String>> {
        command
            .get_envs()
            .map(|(name, value)| {
                (
                    name.to_string_lossy().into_owned(),
                    value.map(|value| value.to_string_lossy().into_owned()),
                )
            })
            .collect()
    }

    /// The test with the parent environment `parent`, as the ctest of
    /// `CMake` 4.4.5 ran `env` on 2026-11-10.
    fn run(parent: &[(&str, &str)], environment: &[&str], modification: &[&str]) -> Command {
        let mut command = Command::new("env");
        for (letter, value) in parent {
            command.env(name(letter), value);
        }
        apply_environment(&mut command, &entry(environment, modification)).unwrap();
        command
    }

    fn check(command: &Command, expected: &[(&str, Option<&str>)]) {
        let seen = seen(command);
        for (letter, value) in expected {
            let name = name(letter);
            let got = seen
                .get(&name)
                .cloned()
                .unwrap_or_else(|| std::env::var(&name).ok());
            assert_eq!(got.as_deref(), *value, "{name}");
        }
    }

    /// The first measure: each operation, the order, `reset` after the
    /// `ENVIRONMENT` and after a `set`, the text cut at the first `=` and `:`.
    #[test]
    fn the_environment_is_the_one_the_ctest_gives_the_test() {
        let command = run(
            &[
                ("B", "keep"),
                ("D", "pre"),
                ("F", ""),
                ("G", "/a"),
                ("H", "a"),
                ("K", "p"),
                ("L", "p"),
                ("W", ""),
            ],
            &["J=env", "R=1", "R=2", "T=e", "Z=a=b"],
            &[
                "A=set:x",
                "B=unset:",
                "C=string_append:x",
                "D=string_prepend:x",
                "E=path_list_append:/x",
                "F=path_list_append:/x",
                "G=path_list_prepend:/x",
                "H=cmake_list_append:x",
                "I=cmake_list_append:x",
                "J=string_append:m",
                "J=reset:",
                "K=set:s",
                "K=reset:",
                "L=unset:",
                "L=string_append:x",
                "N=set:a=b",
                "O=set:a:b",
                "T=string_append:m",
                "U=reset:",
                "V=set:a;b",
                "W=path_list_prepend:/x",
                "X=string_append:",
                "Y=set:",
            ],
        );
        let joined = format!("/x{PATH_LIST}/a");
        check(
            &command,
            &[
                ("A", Some("x")),
                ("B", None),
                ("C", Some("x")),
                ("D", Some("xpre")),
                ("E", Some("/x")),
                ("F", Some("/x")),
                ("G", Some(&joined)),
                ("H", Some("a;x")),
                ("I", Some("x")),
                ("J", Some("env")),
                ("K", Some("p")),
                ("L", Some("x")),
                ("N", Some("a=b")),
                ("O", Some("a:b")),
                ("R", Some("2")),
                ("T", Some("em")),
                ("U", None),
                ("V", Some("a;b")),
                ("W", Some("/x")),
                ("X", Some("")),
                ("Y", Some("")),
                ("Z", Some("a=b")),
            ],
        );
        assert_eq!(
            seen(&command)["CTEST_INTERACTIVE_DEBUG_MODE"].as_deref(),
            Some("1")
        );
    }

    /// The second measure: the empty value, the entry of the `ENVIRONMENT`
    /// without `=`, `reset` after `unset`, and `unset` after a `set`.
    #[test]
    fn an_empty_value_and_an_unset_variable_join_as_the_ctest_joins_them() {
        let command = run(
            &[
                ("S", "keepS"),
                ("B", "/a"),
                ("D", ""),
                ("E", "a"),
                ("Q", "p"),
            ],
            &["S", "P1=env", "P2=env", "P3=env"],
            &[
                "A=path_list_append:",
                "B=path_list_append:",
                "C=cmake_list_append:",
                "D=cmake_list_append:x",
                "E=cmake_list_prepend:x",
                "F=string_append:",
                "G=path_list_prepend:",
                "P1=unset:",
                "P1=reset:",
                "P2=set:s",
                "P2=unset:",
                "P2=string_prepend:x",
                "P3=unset:",
                "Q=set:1",
                "Q=unset:",
                "Y=reset:junk",
                "Z=unset:junk",
            ],
        );
        let appended = format!("/a{PATH_LIST}");
        check(
            &command,
            &[
                ("A", Some("")),
                ("B", Some(&appended)),
                ("C", Some("")),
                ("D", Some("x")),
                ("E", Some("x;a")),
                ("F", Some("")),
                ("G", Some("")),
                ("P1", Some("env")),
                ("P2", Some("x")),
                ("P3", None),
                ("Q", None),
                ("S", None),
                ("Y", None),
                ("Z", None),
            ],
        );
    }

    /// The ctest puts `CTEST_INTERACTIVE_DEBUG_MODE=1` before the
    /// `ENVIRONMENT`, which wins, and a modification takes it out.
    #[test]
    fn the_variable_of_the_ctest_comes_before_the_environment_of_the_test() {
        let mut command = Command::new("env");
        apply_environment(
            &mut command,
            &entry(&["CTEST_INTERACTIVE_DEBUG_MODE=0"], &[]),
        )
        .unwrap();
        assert_eq!(
            seen(&command)["CTEST_INTERACTIVE_DEBUG_MODE"].as_deref(),
            Some("0")
        );
        let mut command = Command::new("env");
        apply_environment(
            &mut command,
            &entry(&[], &["CTEST_INTERACTIVE_DEBUG_MODE=unset:"]),
        )
        .unwrap();
        assert_eq!(seen(&command)["CTEST_INTERACTIVE_DEBUG_MODE"], None);
    }

    /// On Unix the case of the name is another variable; Windows compares
    /// the names without it.
    #[test]
    fn the_name_of_a_variable_is_compared_as_the_system_compares_it() {
        let mut command = Command::new("env");
        command.env("KV34_LOW", "p");
        let test = CtestEntry {
            environment: vec!["kv34_low=1".to_owned()],
            environment_modification: vec![
                "KV34_LOW=string_append:x".to_owned(),
                "kv34_low=string_append:y".to_owned(),
            ],
            ..entry(&[], &[])
        };
        apply_environment(&mut command, &test).unwrap();
        let seen = seen(&command);
        if cfg!(windows) {
            let value = seen
                .iter()
                .find(|(name, _)| name.eq_ignore_ascii_case("KV34_LOW"))
                .and_then(|(_, value)| value.clone());
            assert_eq!(value.as_deref(), Some("1xy"));
        } else {
            assert_eq!(seen["KV34_LOW"].as_deref(), Some("px"));
            assert_eq!(seen["kv34_low"].as_deref(), Some("1y"));
        }
    }

    /// Measured: after an entry of the `ENVIRONMENT` without `=`, a join
    /// starts from the empty, not from the value of the process, and `reset`
    /// keeps the variable out.
    #[test]
    fn a_variable_the_environment_takes_out_starts_empty() {
        assert!(std::env::var_os("PATH").is_some_and(|path| !path.is_empty()));
        let mut command = Command::new("env");
        let test = CtestEntry {
            environment: vec!["PATH".to_owned(), "HOME".to_owned()],
            environment_modification: vec![
                "PATH=string_append:x".to_owned(),
                "HOME=reset:".to_owned(),
            ],
            ..entry(&[], &[])
        };
        apply_environment(&mut command, &test).unwrap();
        let seen = seen(&command);
        assert_eq!(seen["PATH"].as_deref(), Some("x"));
        assert_eq!(seen["HOME"], None);
    }

    /// What the ctest refuses, it does not run: the command is left as it
    /// was, and the reason says the entry.
    #[test]
    fn a_modification_the_ctest_refuses_leaves_the_command_alone() {
        for (modification, why) in [
            ("M=bogus:x", "tem a operação `bogus`, que ele não conhece"),
            ("M=set", "não tem o `:` depois da operação"),
            ("Mset:x", "não tem o `=` depois do nome"),
            ("M:x=set", "não tem o `:` depois da operação"),
        ] {
            let entry = CtestEntry {
                environment_modification: vec!["A=set:1".to_owned(), modification.to_owned()],
                ..entry(&["E=1"], &[])
            };
            let reason = format!(
                "o CTest não roda o teste `t`: o `ENVIRONMENT_MODIFICATION` `{modification}` \
                 {why}; corrija o teste no CMake"
            );
            assert_eq!(environment_refusal(&entry).as_ref(), Some(&reason));
            let mut command = Command::new("env");
            assert_eq!(apply_environment(&mut command, &entry), Err(reason));
            assert_eq!(command.get_envs().count(), 0, "{modification}");
        }
        assert_eq!(
            environment_refusal(&entry(&[], &["A=reset:", "B=unset:"])),
            None
        );
    }

    fn words(command: &Command) -> Vec<String> {
        command
            .get_args()
            .map(|argument| argument.to_string_lossy().into_owned())
            .collect()
    }

    /// C1-34 and C1-38: the arguments of the `add_test` first, the flags of
    /// the IDE last, and the command shown is the one that runs.
    #[test]
    fn the_case_runs_with_the_arguments_of_the_test_and_shows_them() {
        let test = |command: &[&str]| CtestEntry {
            command: command.iter().map(|&word| word.to_owned()).collect(),
            working_directory: Some("/b/dados".into()),
            ..CtestEntry::default()
        };
        let (command, shown) = case_command(
            &test(&["/b/meu gt", "/b/os dados", "--gtest_brief=1", "it's"]),
            Framework::Gtest,
            "Math.Adds",
        )
        .unwrap();
        assert_eq!(command.get_program(), "/b/meu gt");
        assert_eq!(
            words(&command),
            [
                "/b/os dados",
                "--gtest_brief=1",
                "it's",
                "--gtest_filter=Math.Adds",
                "--gtest_color=no",
                "--gtest_print_time=1",
                "--gtest_brief=0"
            ]
        );
        assert_eq!(
            command.get_current_dir(),
            Some(std::path::Path::new("/b/dados"))
        );
        // The quotes of the shell of the system: POSIX, or the PowerShell.
        let expected = if cfg!(windows) {
            "& '/b/meu gt' '/b/os dados' '--gtest_brief=1' 'it''s' '--gtest_filter=Math.Adds' \
             '--gtest_color=no' '--gtest_print_time=1' '--gtest_brief=0'"
        } else {
            "'/b/meu gt' '/b/os dados' --gtest_brief=1 'it'\\''s' --gtest_filter=Math.Adds \
             --gtest_color=no --gtest_print_time=1 --gtest_brief=0"
        };
        assert_eq!(shown, expected);
        // Catch2: the reporter of the test, or `-r compact` when it has none.
        for (arguments, reporter) in [
            (&[][..], true),
            (&["-s"][..], true),
            (&["-o", "out.txt"][..], true),
            (&["-r", "junit"][..], false),
            (&["--reporter=junit::out=j.xml"][..], false),
            (&["--reporter:junit"][..], false),
            (&["-sr", "junit"][..], false),
        ] {
            let mut command = vec!["/b/c2"];
            command.extend(arguments);
            let (run, shown) = case_command(&test(&command), Framework::Catch2, "a, b").unwrap();
            let mut expected: Vec<String> = arguments.iter().map(|&word| word.to_owned()).collect();
            expected.push("a\\, b".to_owned());
            if reporter {
                expected.extend(["-r".to_owned(), "compact".to_owned()]);
            }
            assert_eq!(words(&run), expected, "{arguments:?}");
            assert!(shown.contains(" 'a\\, b'"), "{shown}");
        }
        assert!(case_command(&test(&[]), Framework::Gtest, "Math.Adds").is_none());
    }
}

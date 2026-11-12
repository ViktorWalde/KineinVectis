//! O que o binario de um teste do ctest diz dos casos dele: gtest primeiro,
//! Catch2 depois (D7 do `roadmaps/41`; saiu do `frameworks` com o C1-43).
//!
//! C1-43: cada chamada roda pelo `process` dos jobs, com o cancel do job, o
//! prazo do `DISCOVERY_TIMEOUT` padrao do `gtest_discover_tests` e um teto
//! para a saida guardada. O binario que nao responde a tempo, como um
//! servidor posto como teste que ignora `--gtest_list_tests`, fica so' com a
//! linha do ctest, e a saida diz por que.

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    process::Command,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use super::frameworks::{CtestEntry, Framework};
use super::launch;
use crate::process;

/// How long a call of the listing may take and how much of its output it
/// keeps (C1-43).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    /// The deadline of one call.
    pub deadline: Duration,
    /// The bytes of stdout one call keeps.
    pub cap: usize,
}

/// Each call has its own limits.
///
/// The deadline is the `DISCOVERY_TIMEOUT` that `gtest_discover_tests` gives
/// by default (`GoogleTest.cmake` of `CMake` 4.4.5), and the cap is 64 MiB:
/// in 5 s a program can write gigabytes, which the core would hold.
pub const LIMITS: Limits = Limits {
    deadline: Duration::from_secs(5),
    cap: 64 * 1024 * 1024,
};

/// What a binary said of its cases: the framework and the names.
pub type Listing = Option<(Framework, Vec<String>)>;

/// The listing of the binary of a test of the ctest, asked once per binary,
/// folder and environment: `gtest_discover_tests` gives one test per case.
pub(super) type Listings = HashMap<(String, Option<PathBuf>, Vec<String>, Vec<String>), Listing>;

/// Who lists the cases of the binary of a test: `list_cases`, or a fake.
pub(super) type Lister<'a> = &'a mut dyn FnMut(&CtestEntry) -> Listing;

pub(super) fn listing(listings: &mut Listings, entry: &CtestEntry, list: Lister<'_>) -> Listing {
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

/// Pergunta ao binario do teste quais casos ele tem: gtest primeiro, Catch2
/// depois.
///
/// `None` = o binario nao e' de nenhum dos dois, nao roda, nao respondeu
/// dentro de `limits` ou o job foi cancelado. A listagem tem a pasta e o
/// ambiente do teste, sem os argumentos, como o `gtest_discover_tests`
/// (C1-34). `note` recebe o motivo do binario que passou do prazo ou do teto.
pub fn list_cases(
    entry: &CtestEntry,
    cancel: &Arc<AtomicBool>,
    limits: Limits,
    note: &mut dyn FnMut(String),
) -> Listing {
    let exe = entry.command.first()?;
    let mut run = |args: &[&str]| -> Option<String> {
        if cancel.load(Ordering::SeqCst) {
            return None;
        }
        let mut command = Command::new(exe);
        command.args(args);
        if let Some(cwd) = &entry.working_directory {
            command.current_dir(cwd);
        }
        launch::apply_environment(&mut command, entry).ok()?;
        let mut text = String::new();
        let mut over = false;
        let (status, expired) = process::stream_command_lines_with_timeout_cancelable(
            command,
            limits.deadline,
            Arc::clone(cancel),
            &mut |stream, line| {
                if stream != "stdout" || over {
                    return;
                }
                if text.len() + line.len() + 1 > limits.cap {
                    over = true;
                    return;
                }
                text.push_str(&line);
                text.push('\n');
            },
        )
        .ok()?;
        let call = args.join(" ");
        if over {
            note(format!(
                "o binário `{exe}` escreveu mais de {} em `{call}`; os casos dele ficam de \
                 fora, e a linha do ctest roda o binário inteiro",
                size(limits.cap)
            ));
            return None;
        }
        if expired {
            note(format!(
                "o binário `{exe}` não respondeu a `{call}` em {} s, o `DISCOVERY_TIMEOUT` \
                 padrão do `gtest_discover_tests`; os casos dele ficam de fora, e a linha do \
                 ctest roda o binário inteiro",
                limits.deadline.as_secs_f64()
            ));
            return None;
        }
        status.success().then_some(text)
    };
    if let Some(text) = run(&["--gtest_list_tests"]) {
        let cases = parse_gtest_list(&text);
        if !cases.is_empty() {
            return Some((Framework::Gtest, cases));
        }
    }
    // C1-28: the listing of Catch2 is one name per line, which any test that
    // prints a line for every argument also gives. Catch2 says who it is in
    // `--libidentify`, the flag it keeps for tools. Catch2 3 lists in random
    // order, and the tree keeps the order of the declarations (C1-31).
    if run(&["--libidentify"]).is_some_and(|answer| is_catch2(&answer))
        && let Some(text) = run(&["--list-tests", "--verbosity", "quiet", "--order", "decl"])
    {
        let cases = parse_catch2_list(&text);
        if !cases.is_empty() {
            return Some((Framework::Catch2, cases));
        }
    }
    None
}

/// `bytes` in MiB when they are whole ones.
fn size(bytes: usize) -> String {
    const MIB: usize = 1024 * 1024;
    if bytes >= MIB && bytes.is_multiple_of(MIB) {
        format!("{} MiB", bytes / MIB)
    } else {
        format!("{bytes} bytes")
    }
}

/// `framework: Catch2` in the `--libidentify` answer, aligned with spaces.
fn is_catch2(identity: &str) -> bool {
    identity.lines().any(|line| {
        line.split_once(':')
            .is_some_and(|(key, value)| key.trim() == "framework" && value.trim() == "Catch2")
    })
}

/// Os casos de `--gtest_list_tests`: `Suite.` na coluna 0, `  Caso` indentado
/// (o `# GetParam()` dos parametrizados e' comentario e cai).
#[must_use]
pub fn parse_gtest_list(text: &str) -> Vec<String> {
    let mut suite = String::new();
    let mut cases = Vec::new();
    for line in text.lines() {
        if line.is_empty() || line.starts_with("Running main()") {
            continue;
        }
        if !line.starts_with(' ') {
            line.split('#')
                .next()
                .unwrap_or_default()
                .trim()
                .clone_into(&mut suite);
            continue;
        }
        let case = line.split('#').next().unwrap_or_default().trim();
        if !case.is_empty() && !suite.is_empty() {
            cases.push(format!("{suite}{case}"));
        }
    }
    cases
}

/// Os casos de `--list-tests --verbosity quiet` (Catch2 v3): um por linha.
#[must_use]
pub fn parse_catch2_list(text: &str) -> Vec<String> {
    text.lines()
        .map(str::trim)
        .filter(|l| {
            !l.is_empty()
                && !l.starts_with("All available test cases")
                && !l.ends_with(" test cases")
        })
        .map(str::to_owned)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{LIMITS, Limits, Listing, list_cases, size};
    use crate::test::frameworks::{CtestEntry, Framework};
    use std::{
        path::{Path, PathBuf},
        sync::{
            Arc,
            atomic::{AtomicBool, Ordering},
        },
        time::{Duration, Instant},
    };

    /// Short enough for a test, and long enough for a binary that answers.
    const SHORT: Limits = Limits {
        deadline: Duration::from_millis(300),
        cap: 1024,
    };

    fn folder(name: &str) -> PathBuf {
        let root =
            std::env::temp_dir().join(format!("kinein-listing-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        root
    }

    /// A test of the ctest whose binary writes each call in `calls`.
    fn binary(root: &Path, body: &str) -> CtestEntry {
        let calls = root.join("calls");
        let path = crate::write_executable(
            root.join("bin"),
            format!("#!/bin/sh\necho \"$1\" >> '{}'\n{body}\n", calls.display()),
        );
        CtestEntry {
            name: "t".to_owned(),
            command: vec![path.display().to_string()],
            ..CtestEntry::default()
        }
    }

    fn calls(root: &Path) -> Vec<String> {
        std::fs::read_to_string(root.join("calls"))
            .unwrap_or_default()
            .lines()
            .map(str::to_owned)
            .collect()
    }

    /// The `sh` line that appends the pid of the last process put in the
    /// background to `file`.
    fn append_pid(file: &Path) -> String {
        crate::sh_save_background_pid(file).replacen(" > ", " >> ", 1)
    }

    fn pids(file: &Path) -> Vec<String> {
        std::fs::read_to_string(file)
            .unwrap_or_default()
            .split_whitespace()
            .map(str::to_owned)
            .collect()
    }

    /// The process ended within 5 s; the test kills it either way.
    fn gone(pid: &str) -> bool {
        let deadline = Instant::now() + Duration::from_secs(5);
        while crate::process_alive(pid) && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(20));
        }
        let survived = crate::process_alive(pid);
        crate::kill_process(pid);
        !survived
    }

    fn list(
        entry: &CtestEntry,
        cancel: &Arc<AtomicBool>,
        limits: Limits,
    ) -> (Listing, Vec<String>) {
        let mut notes = Vec::new();
        let listing = list_cases(entry, cancel, limits, &mut |note| notes.push(note));
        (listing, notes)
    }

    #[test]
    fn the_limits_are_the_discovery_timeout_of_cmake_and_64_mib() {
        assert_eq!(
            LIMITS,
            Limits {
                deadline: Duration::from_secs(5),
                cap: 64 * 1024 * 1024
            }
        );
        assert_eq!(size(LIMITS.cap), "64 MiB");
        assert_eq!(size(1024), "1024 bytes");
        assert_eq!(size(3 * 1024 * 1024 + 1), "3145729 bytes");
    }

    /// C1-43: a server put as a test, which ignores the flags and `SIGTERM`
    /// and keeps a child, stops at the deadline of each call, with its tree.
    #[test]
    fn a_binary_that_never_answers_stops_at_the_deadline_of_each_call() {
        let _serial = crate::serialize_executables();
        let root = folder("never");
        let file = root.join("pids");
        let entry = binary(
            &root,
            &format!("trap '' TERM\nsleep 30 &\n{}\nwait", append_pid(&file)),
        );
        let started = Instant::now();
        let (listing, notes) = list(&entry, &Arc::default(), SHORT);
        let elapsed = started.elapsed();
        assert_eq!(listing, None);
        assert!(elapsed >= SHORT.deadline * 2, "{elapsed:?}");
        assert!(elapsed < Duration::from_secs(5), "{elapsed:?}");
        assert_eq!(calls(&root), ["--gtest_list_tests", "--libidentify"]);
        assert_eq!(notes.len(), 2, "{notes:?}");
        for (note, call) in notes.iter().zip(["--gtest_list_tests", "--libidentify"]) {
            assert!(
                note.contains(&format!("não respondeu a `{call}` em 0.3 s")),
                "{note}"
            );
            assert!(note.contains(&entry.command[0]), "{note}");
        }
        let pids = pids(&file);
        assert_eq!(pids.len(), 2, "{pids:?}");
        for pid in &pids {
            assert!(gone(pid), "o filho {pid} sobreviveu ao prazo");
        }
    }

    #[test]
    fn the_output_past_the_cap_counts_as_no_answer_and_the_cap_itself_does_not() {
        let _serial = crate::serialize_executables();
        let root = folder("cap");
        // 13 bytes, with the breaks.
        let entry = binary(&root, "printf 'Math.\\n  Adds\\n'");
        let at = Limits { cap: 13, ..SHORT };
        assert_eq!(
            list(&entry, &Arc::default(), at),
            (
                Some((Framework::Gtest, vec!["Math.Adds".to_owned()])),
                Vec::new()
            )
        );
        let (listing, notes) = list(&entry, &Arc::default(), Limits { cap: 12, ..SHORT });
        assert_eq!(listing, None);
        assert!(
            notes[0].contains("escreveu mais de 12 bytes em `--gtest_list_tests`"),
            "{notes:?}"
        );

        // Only stdout is the listing, as `gtest_discover_tests` reads it, and
        // only stdout counts for the cap.
        let stderr = binary(&root, "printf 'Math.\\n  Adds\\n' >&2");
        assert_eq!(list(&stderr, &Arc::default(), at), (None, Vec::new()));
        let both = binary(
            &root,
            "printf 'Math.\\n  Adds\\n'; printf 'warning: x\\n' >&2",
        );
        assert_eq!(
            list(&both, &Arc::default(), at),
            (
                Some((Framework::Gtest, vec!["Math.Adds".to_owned()])),
                Vec::new()
            )
        );

        // A listing without end stops at the deadline and keeps only the cap.
        let forever = binary(&root, "while :; do echo '  Case'; done");
        let started = Instant::now();
        let (listing, notes) = list(&forever, &Arc::default(), SHORT);
        assert_eq!(listing, None);
        assert!(
            started.elapsed() < Duration::from_secs(5),
            "{:?}",
            started.elapsed()
        );
        assert!(
            notes[0].contains("escreveu mais de 1024 bytes em `--gtest_list_tests`"),
            "{notes:?}"
        );
    }

    /// The cancel of the job stops the call that hangs, and no call comes
    /// after it.
    #[test]
    fn the_cancel_stops_each_call_and_the_ones_after_it() {
        let _serial = crate::serialize_executables();
        let all = ["--gtest_list_tests", "--libidentify", "--list-tests"];
        for blocked in 0..all.len() {
            let root = folder(&format!("cancel-{blocked}"));
            let file = root.join("pids");
            let entry = binary(
                &root,
                &format!(
                    "case \"$1\" in\n  {}) sleep 30 &\n    {}\n    wait;;\n  --libidentify) printf 'framework:      Catch2\\n';;\n  *) exit 1;;\nesac",
                    all[blocked],
                    append_pid(&file)
                ),
            );
            let cancel = Arc::new(AtomicBool::new(false));
            let canceller = {
                let (cancel, file) = (Arc::clone(&cancel), file.clone());
                std::thread::spawn(move || {
                    let deadline = Instant::now() + Duration::from_secs(10);
                    while pids(&file).is_empty() {
                        assert!(Instant::now() < deadline, "o binário não começou");
                        std::thread::sleep(Duration::from_millis(20));
                    }
                    cancel.store(true, Ordering::SeqCst);
                    Instant::now()
                })
            };
            let limits = Limits {
                deadline: Duration::from_secs(30),
                ..SHORT
            };
            let (listing, notes) = list(&entry, &cancel, limits);
            let stopped = canceller.join().unwrap();
            assert_eq!(listing, None);
            assert!(stopped.elapsed() < Duration::from_secs(3), "{blocked}");
            assert_eq!(notes, Vec::<String>::new(), "{blocked}");
            assert_eq!(calls(&root), all[..=blocked], "{blocked}");
            for pid in pids(&file) {
                assert!(gone(&pid), "o filho {pid} sobreviveu ao cancel ({blocked})");
            }
        }
    }

    /// A binary that answers and leaves a child with the pipe open counts,
    /// and the listing does not wait for the child.
    #[test]
    fn a_binary_that_answers_and_leaves_a_child_with_the_pipe_counts() {
        let _serial = crate::serialize_executables();
        let root = folder("child");
        let file = root.join("pids");
        let entry = binary(
            &root,
            &format!(
                "case \"$1\" in\n  --gtest_list_tests) sleep 30 &\n    {}\n    printf 'Math.\\n  Adds\\n';;\n  *) exit 1;;\nesac",
                append_pid(&file)
            ),
        );
        let started = Instant::now();
        let listing = list(&entry, &Arc::default(), LIMITS);
        let elapsed = started.elapsed();
        for pid in pids(&file) {
            crate::kill_process(&pid);
        }
        assert_eq!(
            listing,
            (
                Some((Framework::Gtest, vec!["Math.Adds".to_owned()])),
                Vec::new()
            )
        );
        assert!(elapsed < LIMITS.deadline, "{elapsed:?}");
    }
}

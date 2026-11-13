//! Executable host corpus. Real Clang is required; synthetic tools test failures only.

use super::*;
use std::{path::PathBuf, sync::atomic::AtomicU64};

fn verify(
    root: &Path,
    path: &Path,
    before: &str,
    after: &str,
    version: Option<i64>,
    cancel: &Arc<AtomicBool>,
    budget: Duration,
) -> VerificationRun {
    let mut prefix = before
        .bytes()
        .zip(after.bytes())
        .take_while(|(a, b)| a == b)
        .count();
    while !before.is_char_boundary(prefix) || !after.is_char_boundary(prefix) {
        prefix -= 1;
    }
    let mut suffix = before[prefix..]
        .bytes()
        .rev()
        .zip(after[prefix..].bytes().rev())
        .take_while(|(a, b)| a == b)
        .count();
    while !before.is_char_boundary(before.len() - suffix)
        || !after.is_char_boundary(after.len() - suffix)
    {
        suffix -= 1;
    }
    let edits = [(prefix, before.len() - suffix, after.len() - prefix - suffix)];
    super::verify(
        Comparison {
            root,
            path,
            before,
            after,
            version,
            edits: &edits,
        },
        cancel,
        budget,
        || crate::index::context::verification_context(root, path),
    )
}

static NEXT: AtomicU64 = AtomicU64::new(0);

pub(super) struct Fixture {
    pub root: PathBuf,
    pub path: PathBuf,
}

impl Fixture {
    fn new(extension: &str, before: &str, flags: &[&str]) -> Self {
        let root = std::env::temp_dir().join(format!(
            "kinein-edit-corpus-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(root.join("src")).unwrap();
        let path = root.join(format!("src/main.{extension}"));
        std::fs::write(&path, before).unwrap();
        let compiler = if extension == "c" { "clang" } else { "clang++" };
        let mut arguments = vec![compiler.to_owned(), "-c".into(), path.display().to_string()];
        arguments.extend(flags.iter().map(|v| (*v).to_owned()));
        std::fs::write(
            root.join("compile_commands.json"),
            serde_json::to_vec(&serde_json::json!([{
                "directory": root, "file": path, "arguments": arguments
            }]))
            .unwrap(),
        )
        .unwrap();
        Self { root, path }
    }

    fn verify(&self, before: &str, after: &str) -> VerificationRun {
        verify(
            &self.root,
            &self.path,
            before,
            after,
            Some(1),
            &Arc::new(AtomicBool::new(false)),
            Duration::from_secs(5),
        )
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

#[test]
fn edit_verification_real_host_corpus() {
    let cases = [
        (
            "c",
            "int main(void) { return miss; }\n",
            "int main(void) { return 0; }\n",
            EditVerificationStatus::NoNewError,
        ),
        (
            "cpp",
            "int main() { return miss; }\n",
            "int main() { return 0; }\n",
            EditVerificationStatus::NoNewError,
        ),
        (
            "cpp",
            "int main() { return 0; }\n",
            "int main() { return ; + }\n",
            EditVerificationStatus::NewError,
        ),
        (
            "c",
            "#include \"value.h\"\nint value = VALUE;\n",
            "int value = VALUE;\n",
            EditVerificationStatus::NewError,
        ),
        (
            "cpp",
            "// ação 😀\nint value = 1;\n",
            "// ação 😀\nint value = FLAG;\n",
            EditVerificationStatus::NoNewError,
        ),
        (
            "cpp",
            "int value = missing;\n",
            "\n\nint value = missing;\n",
            EditVerificationStatus::NoNewError,
        ),
        (
            "cpp",
            "int value = missing;\n",
            "int value = missing;\nint second = another;\n",
            EditVerificationStatus::NewError,
        ),
    ];
    let (mut valid, mut false_blocks) = (0, 0);
    for (index, (extension, before, after, expected)) in cases.into_iter().enumerate() {
        let fixture = Fixture::new(extension, before, &["-DFLAG=7"]);
        std::fs::write(fixture.root.join("src/value.h"), "#define VALUE 42\n").unwrap();
        let result = fixture.verify(before, after);
        eprintln!(
            "host case {index}: {:?}, {} ms, {}",
            result.evidence.status, result.evidence.duration_ms, result.evidence.reason
        );
        if expected == EditVerificationStatus::NoNewError {
            valid += 1;
            false_blocks += usize::from(result.evidence.status == EditVerificationStatus::NewError);
        }
        assert_eq!(result.evidence.status, expected, "{:?}", result.evidence);
        assert!(result.evidence.duration_ms < 5000);
        assert_eq!(std::fs::read_to_string(&fixture.path).unwrap(), before);
        assert!(!fixture.root.join("main.o").exists());
    }
    eprintln!("false newError: {false_blocks}/{valid} valid host cases");
    assert_eq!(false_blocks, 0);
}

#[test]
fn edit_verification_keeps_relative_and_forced_includes_and_flags() {
    let before = "#include \"value.h\"\nstatic_assert(VALUE == 42);\nstatic_assert(FORCED == 3);\nstatic_assert(FLAG == 7);\n";
    let fixture = Fixture::new(
        "cpp",
        before,
        &["-std=c++20", "-DFLAG=7", "-include", "forced.h"],
    );
    std::fs::write(fixture.root.join("src/value.h"), "#define VALUE 42\n").unwrap();
    std::fs::write(fixture.root.join("forced.h"), "#define FORCED 3\n").unwrap();
    let result = fixture.verify(before, &format!("{before}// changed\n"));
    assert_eq!(
        result.evidence.status,
        EditVerificationStatus::NoNewError,
        "{:?}",
        result.evidence
    );
    assert_eq!(result.evidence.before_errors, Some(0));
    assert!(result.guard.as_ref().unwrap().is_current());
    std::fs::write(fixture.root.join("src/value.h"), "#define VALUE 43\n").unwrap();
    assert!(!result.guard.unwrap().is_current());
}

#[test]
fn edit_verification_missing_context_and_unsupported_commands_are_unknown() {
    let before = "int value;\n";
    let fixture = Fixture::new("c", before, &[]);
    std::fs::remove_file(fixture.root.join("compile_commands.json")).unwrap();
    assert_eq!(
        fixture.verify(before, before).evidence.status,
        EditVerificationStatus::NotVerified
    );
    for flags in [
        vec!["@args.rsp"],
        vec!["-Xclang", "-load", "plugin.so"],
        vec!["-save-temps"],
        vec!["-MJ", "original.json"],
        vec!["-fmodules"],
        vec!["-include-pch", "a.pch"],
    ] {
        let fixture = Fixture::new("c", before, &flags);
        let result = fixture.verify(before, before);
        assert_eq!(
            result.evidence.status,
            EditVerificationStatus::NotVerified,
            "{flags:?}"
        );
        assert!(!fixture.root.join("original.json").exists());
    }
}

#[test]
fn edit_verification_cdb_change_invalidates_evidence() {
    let fixture = Fixture::new("cpp", "int value = FLAG;\n", &["-DFLAG=7"]);
    let result = fixture.verify("int value = FLAG;\n", "int value = FLAG+1;\n");
    assert_eq!(
        result.evidence.status,
        EditVerificationStatus::NoNewError,
        "{:?}",
        result.evidence
    );
    let cdb = fixture.root.join("compile_commands.json");
    let changed = std::fs::read_to_string(&cdb)
        .unwrap()
        .replace("-DFLAG=7", "-UFLAG");
    std::fs::write(&cdb, changed).unwrap();
    assert!(!result.guard.unwrap().is_current());
    let result = fixture.verify("int value = FLAG;\n", "int value = 0;\n");
    assert_eq!(result.evidence.before_errors, Some(1));
    assert_eq!(result.evidence.after_errors, Some(0));
}

#[test]
fn edit_verification_cancellation_and_size_are_never_positive() {
    let fixture = Fixture::new("c", "int value;", &[]);
    let cancelled = Arc::new(AtomicBool::new(true));
    let result = verify(
        &fixture.root,
        &fixture.path,
        "int value;",
        "int value;",
        Some(1),
        &cancelled,
        Duration::from_secs(5),
    );
    assert_eq!(result.evidence.status, EditVerificationStatus::NotVerified);
    let result = fixture.verify("int value;", &" ".repeat(1_048_577));
    assert_eq!(result.evidence.status, EditVerificationStatus::NotVerified);
}

#[test]
fn edit_verification_timeout_and_missing_tool_are_unknown() {
    let fixture = Fixture::new("c", "int value;", &[]);
    let cdb = fixture.root.join("compile_commands.json");
    let mut entries: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&cdb).unwrap()).unwrap();
    entries[0]["arguments"][0] = serde_json::json!(fixture.root.join("missing/clang"));
    std::fs::write(&cdb, entries.to_string()).unwrap();
    let result = fixture.verify("int value;", "int value;");
    assert_eq!(result.evidence.status, EditVerificationStatus::NotVerified);
    assert!(
        result.evidence.reason.contains("ausente"),
        "{:?}",
        result.evidence
    );
    let slow = crate::write_executable(fixture.root.join("clang"), "#!/bin/sh\nsleep 20\n");
    entries[0]["arguments"][0] = serde_json::json!(slow);
    std::fs::write(&cdb, entries.to_string()).unwrap();
    let started = Instant::now();
    let result = verify(
        &fixture.root,
        &fixture.path,
        "int value;",
        "int value;",
        Some(1),
        &Arc::new(AtomicBool::new(false)),
        Duration::from_millis(150),
    );
    assert_eq!(result.evidence.status, EditVerificationStatus::NotVerified);
    assert!(
        result.evidence.reason.contains("Prazo"),
        "{:?}",
        result.evidence
    );
    assert!(
        started.elapsed() < Duration::from_secs(3),
        "process group must be collected"
    );
}

#[test]
fn edit_verification_ambiguous_unit_is_unknown() {
    let fixture = Fixture::new("cpp", "int value;", &[]);
    let cdb = fixture.root.join("compile_commands.json");
    let mut entries: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&cdb).unwrap()).unwrap();
    let second = entries[0].clone();
    entries.as_array_mut().unwrap().push(second);
    std::fs::write(&cdb, entries.to_string()).unwrap();
    assert_eq!(
        fixture.verify("int value;", "int value;").evidence.status,
        EditVerificationStatus::NotVerified
    );
}

#[test]
fn edit_verification_virtual_filename_and_compiler_driver_are_preserved() {
    let fixture = Fixture::new("c", "int value;\n", &[]);
    let cdb = fixture.root.join("compile_commands.json");
    let before = format!(
        "static_assert(__builtin_strcmp(__FILE__, \"{}\") == 0);\n",
        fixture.path.display()
    );
    std::fs::write(&fixture.path, &before).unwrap();
    let mut entries: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&cdb).unwrap()).unwrap();
    // clang++ on a .c file is deliberately different from canonicalizing the executable to clang.
    entries[0]["arguments"][0] = serde_json::json!("clang++");
    std::fs::write(&cdb, entries.to_string()).unwrap();
    let result = fixture.verify(&before, &format!("{before}// change\n"));
    assert_eq!(
        result.evidence.status,
        EditVerificationStatus::NoNewError,
        "{:?}",
        result.evidence
    );
    assert_eq!(result.evidence.before_errors, Some(0));
}

#[test]
fn edit_verification_unicode_filename_and_command_cdb_are_supported() {
    let mut fixture = Fixture::new("cpp", "int value = FLAG;\n", &[]);
    let changed = fixture.root.join("src/ação space.cpp");
    std::fs::rename(&fixture.path, &changed).unwrap();
    fixture.path = changed;
    std::fs::write(
        fixture.root.join("compile_commands.json"),
        serde_json::json!([{
            "directory":fixture.root,"file":fixture.path,
            "command":format!("clang++ -DFLAG=7 -c '{}'",fixture.path.display())
        }])
        .to_string(),
    )
    .unwrap();
    let result = fixture.verify("int value = FLAG;\n", "int value = FLAG+1;\n");
    assert_eq!(
        result.evidence.status,
        EditVerificationStatus::NoNewError,
        "{:?}",
        result.evidence
    );
    assert_eq!(result.evidence.before_errors, Some(0));
}

#[test]
fn edit_verification_cancels_an_active_compiler() {
    let fixture = Fixture::new("c", "int value;", &[]);
    let slow = crate::write_executable(fixture.root.join("clang"), "#!/bin/sh\nsleep 20\n");
    let cdb = fixture.root.join("compile_commands.json");
    let mut entries: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&cdb).unwrap()).unwrap();
    entries[0]["arguments"][0] = serde_json::json!(slow);
    std::fs::write(&cdb, entries.to_string()).unwrap();
    let cancel = Arc::new(AtomicBool::new(false));
    let stopper = {
        let cancel = cancel.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(100));
            cancel.store(true, Ordering::SeqCst);
        })
    };
    let started = Instant::now();
    let result = verify(
        &fixture.root,
        &fixture.path,
        "int value;",
        "int value;",
        Some(1),
        &cancel,
        Duration::from_secs(5),
    );
    stopper.join().unwrap();
    assert_eq!(result.evidence.status, EditVerificationStatus::NotVerified);
    assert!(result.evidence.reason.contains("cancelada"));
    assert!(started.elapsed() < Duration::from_secs(3));
}

#[test]
fn edit_verification_a_driver_without_the_overlay_is_unknown() {
    let fixture = Fixture::new("c", "int value;", &[]);
    let fake = crate::write_executable(fixture.root.join("clang"), "#!/bin/sh\nexit 0\n");
    let cdb = fixture.root.join("compile_commands.json");
    let mut entries: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&cdb).unwrap()).unwrap();
    entries[0]["arguments"][0] = serde_json::json!(fake);
    std::fs::write(cdb, entries.to_string()).unwrap();
    let result = fixture.verify("int value;", "int value=missing;");
    assert_eq!(result.evidence.status, EditVerificationStatus::NotVerified);
    assert!(result.evidence.before_errors.is_none());
}

#[test]
fn edit_verification_identical_message_at_a_new_expression_is_new() {
    let before = "int old_value = missing;\n";
    let after = "int old_value = 0;\nint new_value = missing;\n";
    let fixture = Fixture::new("cpp", before, &[]);
    let edits = [
        (16, 23, 1),
        (
            before.len(),
            before.len(),
            "int new_value = missing;\n".len(),
        ),
    ];
    let result = super::verify(
        Comparison {
            root: &fixture.root,
            path: &fixture.path,
            before,
            after,
            version: Some(1),
            edits: &edits,
        },
        &Arc::new(AtomicBool::new(false)),
        Duration::from_secs(5),
        || crate::index::context::verification_context(&fixture.root, &fixture.path),
    );
    assert_eq!(
        result.evidence.status,
        EditVerificationStatus::NewError,
        "{:?}",
        result.evidence
    );
}

#[test]
fn edit_verification_observes_errors_beyond_the_compiler_default_limit() {
    use std::fmt::Write;
    let mut before = String::new();
    for n in 0..25 {
        writeln!(before, "int value{n} = missing{n};").unwrap();
    }
    let after = format!("{before}int last_value = last_missing;\n");
    let fixture = Fixture::new("cpp", &before, &[]);
    let result = fixture.verify(&before, &after);
    assert_eq!(
        result.evidence.status,
        EditVerificationStatus::NewError,
        "{:?}",
        result.evidence
    );
}

#[test]
fn edit_verification_header_change_with_restored_size_and_time_is_stale() {
    let before = "#include \"value.h\"\nint value = VALUE;\n";
    let fixture = Fixture::new("cpp", before, &[]);
    let header = fixture.root.join("src/value.h");
    std::fs::write(&header, "#define VALUE 1\n").unwrap();
    let result = fixture.verify(before, &format!("{before}// changed\n"));
    assert_eq!(result.evidence.status, EditVerificationStatus::NoNewError);
    let modified = std::fs::metadata(&header).unwrap().modified().unwrap();
    std::fs::write(&header, "#define VALUE 2\n").unwrap();
    std::fs::File::options()
        .write(true)
        .open(&header)
        .unwrap()
        .set_modified(modified)
        .unwrap();
    assert!(
        !result.guard.unwrap().is_current(),
        "content identity must survive restored timestamps"
    );
}

#[test]
fn edit_verification_gcc_and_gxx_keep_the_actual_driver_and_original_files() {
    for (extension, compiler) in [("c", "gcc"), ("cpp", "g++")] {
        let before = "#include \"value.h\"\nint value = missing;\n";
        let fixture = Fixture::new(extension, before, &["-DFLAG=7", "-MD", "-MF", "original.d"]);
        std::fs::write(fixture.root.join("src/value.h"), "#define VALUE 3\n").unwrap();
        let cdb = fixture.root.join("compile_commands.json");
        let mut entries: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&cdb).unwrap()).unwrap();
        entries[0]["arguments"][0] = serde_json::json!(compiler);
        std::fs::write(&cdb, entries.to_string()).unwrap();
        let valid = fixture.verify(before, &before.replace("missing", "VALUE+FLAG"));
        assert_eq!(
            valid.evidence.status,
            EditVerificationStatus::NoNewError,
            "{:?}",
            valid.evidence
        );
        assert_eq!(valid.evidence.compiler.as_deref(), Some(compiler));
        let invalid = fixture.verify(before, &before.replace("missing", "different_missing"));
        assert_eq!(
            invalid.evidence.status,
            EditVerificationStatus::NewError,
            "{:?}",
            invalid.evidence
        );
        assert_eq!(std::fs::read_to_string(&fixture.path).unwrap(), before);
        assert!(!fixture.root.join("original.d").exists());
    }
}

#[test]
fn edit_verification_new_optional_header_invalidates_the_context() {
    let before = "#if __has_include(\"optional.h\")\n#error optional changed\n#endif\nint value;\n";
    let fixture = Fixture::new("cpp", before, &[]);
    let result = fixture.verify(before, &format!("{before}// changed\n"));
    assert_eq!(
        result.evidence.status,
        EditVerificationStatus::NoNewError,
        "{:?}",
        result.evidence
    );
    std::fs::write(fixture.root.join("src/optional.h"), "// newly available\n").unwrap();
    assert!(!result.guard.unwrap().is_current());
}

#[test]
fn edit_verification_bounds_a_single_output_line_without_newline() {
    let fixture = Fixture::new("c", "int value;", &[]);
    let fake = crate::write_executable(
        fixture.root.join("clang"),
        "#!/bin/sh\nhead -c 2097152 /dev/zero\nsleep 20\n",
    );
    let cdb = fixture.root.join("compile_commands.json");
    let mut entries: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&cdb).unwrap()).unwrap();
    entries[0]["arguments"][0] = serde_json::json!(fake);
    std::fs::write(cdb, entries.to_string()).unwrap();
    let started = Instant::now();
    let result = fixture.verify("int value;", "int value=missing;");
    assert_eq!(result.evidence.status, EditVerificationStatus::NotVerified);
    assert!(
        result.evidence.reason.contains("1 MiB"),
        "{:?}",
        result.evidence
    );
    assert!(started.elapsed() < Duration::from_secs(3));
}

#[test]
fn edit_verification_macro_errors_use_the_call_site_and_byte_columns() {
    for compiler in ["clang++", "g++"] {
        let before = "#include \"macro.h\"\n/* 😀 */ int old = BROKEN;\n";
        let after = "#include \"macro.h\"\n/* 😀 */ int old = 0;\nint newer = BROKEN;\n";
        let fixture = Fixture::new("cpp", before, &[]);
        std::fs::write(fixture.root.join("src/macro.h"), "#define BROKEN missing\n").unwrap();
        let cdb = fixture.root.join("compile_commands.json");
        let mut entries: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&cdb).unwrap()).unwrap();
        entries[0]["arguments"][0] = serde_json::json!(compiler);
        std::fs::write(cdb, entries.to_string()).unwrap();
        let start = before.find("BROKEN").unwrap();
        let edits = [
            (start, start + 6, 1),
            (before.len(), before.len(), "int newer = BROKEN;\n".len()),
        ];
        let result = super::verify(
            Comparison {
                root: &fixture.root,
                path: &fixture.path,
                before,
                after,
                version: Some(1),
                edits: &edits,
            },
            &Arc::new(AtomicBool::new(false)),
            Duration::from_secs(5),
            || crate::index::context::verification_context(&fixture.root, &fixture.path),
        );
        assert_eq!(
            result.evidence.status,
            EditVerificationStatus::NewError,
            "{compiler}: {:?}",
            result.evidence
        );
        assert_eq!(result.evidence.before_errors, Some(1));
    }
}

#[test]
fn edit_verification_preserves_the_original_source_timestamp() {
    let before = "static_assert(__TIMESTAMP__[20] == '1');\n";
    let fixture = Fixture::new("cpp", before, &[]);
    std::fs::File::options()
        .write(true)
        .open(&fixture.path)
        .unwrap()
        .set_modified(std::time::UNIX_EPOCH)
        .unwrap();
    let result = fixture.verify(before, &format!("{before}// changed\n"));
    assert_eq!(
        result.evidence.status,
        EditVerificationStatus::NoNewError,
        "{:?}",
        result.evidence
    );
    assert_eq!(result.evidence.before_errors, Some(0));
    assert_eq!(result.evidence.after_errors, Some(0));
}

#[test]
fn edit_verification_accepts_the_effective_host_hardening_flags() {
    let before = "#ifndef __CET__\n#error missing hardening context\n#endif\nint value;\n";
    let fixture = Fixture::new(
        "cpp",
        before,
        &[
            "-fstack-protector-strong",
            "-fno-omit-frame-pointer",
            "-fstrict-flex-arrays=3",
            "-fcf-protection=full",
            "-fno-direct-access-external-data",
            "-Werror",
        ],
    );
    let result = fixture.verify(before, &format!("{before}// changed\n"));
    assert_eq!(
        result.evidence.status,
        EditVerificationStatus::NoNewError,
        "{:?}",
        result.evidence
    );
    assert_eq!(result.evidence.before_errors, Some(0));
}

/// INF-18: a lista de dependências que o compilador deixa e que não é
/// arquivo regular, uma pasta ou um link para um FIFO, é a que ele não
/// forneceu, sem esperar no FIFO. O compilador é o de verdade, embrulhado.
#[test]
fn edit_verification_a_dependency_list_that_is_not_a_regular_file_is_missing() {
    for kind in ["pasta", "fifo"] {
        let fixture = Fixture::new("c", "int value;", &[]);
        let held = fixture.root.join("held");
        let replace = if kind == "pasta" {
            "mkdir \"$dep\"".to_owned()
        } else if crate::fsops::fifo(&held) {
            format!("ln -s '{}' \"$dep\"", held.display())
        } else {
            continue;
        };
        let wrapper = crate::write_executable(
            fixture.root.join("clang"),
            format!(
                "#!/bin/sh\ndep=\ntake=\nfor arg in \"$@\"; do\n  if [ -n \"$take\" ]; then dep=$arg; take=; fi\n  [ \"$arg\" = \"-MF\" ] && take=1\ndone\nclang \"$@\" || exit $?\nrm -f \"$dep\"\n{replace}\n"
            ),
        );
        let cdb = fixture.root.join("compile_commands.json");
        let mut entries: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&cdb).unwrap()).unwrap();
        entries[0]["arguments"][0] = serde_json::json!(wrapper);
        std::fs::write(cdb, entries.to_string()).unwrap();
        let evidence = crate::fsops::unblocked(&held, move || {
            fixture.verify("int value;", "int value=1;").evidence
        });
        assert_eq!(
            evidence.status,
            EditVerificationStatus::NotVerified,
            "{kind}: {evidence:?}"
        );
        assert!(
            evidence
                .reason
                .contains("não forneceu a lista de dependências"),
            "{kind}: {evidence:?}"
        );
    }
}

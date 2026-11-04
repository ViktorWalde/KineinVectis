#!/usr/bin/env python3
"""Run configuration/context/correction guard mutations in a disposable checkout.

Requires a warmed QA target, fixed Rust and the host corpus dependencies.
The working checkout is only read. Run inside the documented memory scope.
"""

import argparse
import os
from pathlib import Path
import shutil
import subprocess
import tempfile


ROOT = Path(__file__).resolve().parent.parent
CASES = [
    (
        "comparison",
        "crates/kinein-core/src/build/edit_verification/mod.rs",
        "evidence.new_errors = mapping.additional(&before_errors.errors, &after_errors.errors);",
        "evidence.new_errors = mapping.additional(&before_errors.errors, &before_errors.errors);",
        "build::edit_verification::tests::edit_verification_real_host_corpus",
    ),
    (
        "context",
        "crates/kinein-core/src/lsp/transaction.rs",
        ".is_some_and(|guard| !guard.is_current())",
        ".is_some_and(|_guard| false)",
        "tests::edit_verification::edit_verification_obsolete_buffer_and_context_refuse_apply",
    ),
    (
        "buffer",
        "crates/kinein-core/src/lsp/manager.rs",
        "active.path != path || active.content != content",
        "active.path != path || (active.content != content && path != path)",
        "tests::edit_verification::edit_verification_action_content_must_match_the_query",
    ),
    (
        "version",
        "crates/kinein-core/src/handlers/lsp/edicao.rs",
        "lsp.code_action_generation() != generation",
        "lsp.code_action_generation() != generation && generation != generation",
        "tests::edit_verification::edit_verification_late_lsp_query_cannot_restore_actions_for_an_old_version",
    ),
    (
        "header_content",
        "crates/kinein-core/src/cdb/verification.rs",
        "digest: digest(&bytes)",
        "digest: digest(&[])",
        "build::edit_verification::tests::edit_verification_header_change_with_restored_size_and_time_is_stale",
    ),
    (
        "include_namespace",
        "crates/kinein-core/src/cdb/directory.rs",
        "Some(super::verification::hex(hasher.finalize().as_ref()))",
        "Some(super::verification::hex(hasher.finalize().as_ref()).chars().take(0).collect())",
        "build::edit_verification::tests::edit_verification_new_optional_header_invalidates_the_context",
    ),
]

C_HOST_CASES = [
    (
        "cdb_content",
        "crates/kinein-core/src/index/context/mod.rs",
        "self.cdb_digest.as_deref() == Some(&crate::cdb::verification::digest(&bytes))",
        "self.cdb_digest.is_some() && !bytes.is_empty()",
        "tests::c_host::c_host_tracks_cdb_content_and_origin_with_restored_metadata",
    ),
    (
        "initial_server",
        "crates/kinein-core/src/handlers/cmake.rs",
        "if changed || server_needs_context {",
        "if changed || (server_needs_context && changed) {",
        "tests::c_host::c_host_refreshes_an_initial_server_even_when_the_index_already_has_the_new_cdb",
    ),
    (
        "stable_documents",
        "crates/kinein-core/src/handlers/cmake.rs",
        "if changed || server_needs_context {",
        "if changed || server_needs_context || root.is_dir() {",
        "tests::c_host::c_host_keeps_build_lsp_context_and_verification_on_the_same_configuration",
    ),
]

C_STANDARD_CASES = [
    (
        "manual_properties",
        "crates/kinein-core/src/configaction/c_standard_source.rs",
        "if conflicts(command, target) {",
        "if conflicts(command, target) && target.is_empty() {",
        "configaction::c_standard::tests::c_standard_refuses_manual_dialect_properties_and_dynamic_sources",
    ),
    (
        "selected_target",
        "crates/kinein-core/src/configaction/c_standard.rs",
        "set_target_properties({target} PROPERTIES\\n",
        "set_target_properties(sample PROPERTIES\\n",
        "tests::c_host::standard::c_standard_drives_gcc_clang_cdb_context_and_tests_without_changing_other_targets",
    ),
]

SUITES = {
    "edit-verification": ("edit_verification", CASES),
    "c-host": ("c_host", C_HOST_CASES),
    "c-standard": ("c_standard", C_STANDARD_CASES),
    "c-project": ("c_project", [
        (
            "future_identifier",
            "crates/kinein-core/src/configaction/c_standard.rs",
            "if !numeric_standard(standard) {",
            "if !SUGGESTED_STANDARDS.contains(&standard) {",
            "tests::c_project::c_project_creation_build_context_standards_and_recovery_use_real_tools",
        ),
        (
            "dialect_allowlist",
            "crates/kinein-core/src/index/context/cdb.rs",
            "unit.standard = Some(v.to_owned());",
            "if [\"c11\", \"c17\", \"c23\"].contains(&v) { unit.standard = Some(v.to_owned()); }",
            "tests::c_project::dialects::c_project_dialects_from_real_compilers_are_preserved_without_a_version_allowlist",
        ),
        (
            "c_template_language",
            "crates/kinein-core/src/workspace/create.rs",
            "create_cmake_project(parent, name, super::cmake_template::Language::C)",
            "create_cmake_project(parent, name, super::cmake_template::Language::Cpp)",
            "tests::c_project::c_project_creation_build_context_standards_and_recovery_use_real_tools",
        ),
    ]),
}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--target-dir", type=Path, required=True)
    parser.add_argument("--logs", type=Path, required=True)
    parser.add_argument("--suite", choices=tuple(SUITES), default="edit-verification")
    args = parser.parse_args()
    args.logs.mkdir(parents=True, exist_ok=True)
    env = {**os.environ, "CARGO_TARGET_DIR": str(args.target_dir.resolve())}
    with tempfile.TemporaryDirectory(prefix="kinein-edit-mutations-") as temporary:
        checkout = Path(temporary)
        for name in ("Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "README.md"):
            shutil.copy2(ROOT / name, checkout / name)
        for name in ("crates", ".cargo", "scripts"):
            shutil.copytree(ROOT / name, checkout / name,
                            ignore=shutil.ignore_patterns("target", "__pycache__"))

        def run(label, test, exact):
            command = ["cargo", "test", "--offline", "--locked", "-p", "kinein-core",
                       "--lib", "--jobs", "2", test, "--"]
            if exact:
                command.append("--exact")
            result = subprocess.run(command, cwd=checkout, env=env, text=True,
                                    stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                                    timeout=300, check=False)
            (args.logs / f"{label}.log").write_text(result.stdout)
            return result

        baseline_test, cases = SUITES[args.suite]
        baseline = run("baseline", baseline_test, False)
        if baseline.returncode:
            raise SystemExit("Baseline failed; no mutation verdict is valid. See baseline.log")
        for label, filename, original, replacement, test in cases:
            source = checkout / filename
            text = source.read_text()
            if text.count(original) != 1:
                raise SystemExit(f"Mutation {label}: expected one source match")
            try:
                source.write_text(text.replace(original, replacement))
                result = run(label, test, True)
                killed = result.returncode == 101 and f"test {test} ... FAILED" in result.stdout
                if not killed:
                    raise SystemExit(f"Mutation {label} survived or failed to compile; inspect {label}.log")
                print(f"killed: {label} ({test})", flush=True)
            finally:
                source.write_text(text)
    print(f"{len(cases)}/{len(cases)} mutations killed")


if __name__ == "__main__":
    main()

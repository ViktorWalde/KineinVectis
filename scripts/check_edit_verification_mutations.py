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
    "c1-existing": ("c1_existing", [
        ("incomplete_evidence", "crates/kinein-core/src/cdb.rs",
         'if directory == ".kinein/build"', 'if directory == "never"',
         "tests::c_project::existing::c1_existing_failed_configure_preserves_selection_and_marks_previous_commands"),
        ("retry_selection", "crates/kinein-core/src/cmake/preset_selection.rs",
         'return Ok((preset, Some("incompleteConfigure")));',
         'return Ok((status(root).preset, Some("buildDir")));',
         "tests::c_project::existing::c1_existing_failed_configure_preserves_selection_and_marks_previous_commands"),
        ("missing_compiler", "crates/kinein-core/src/index/context/freshness.rs",
         'missing.then(||', '(missing && path.as_os_str().is_empty()).then(||',
         "tests::c_project::existing::c1_existing_removed_compiler_and_replaced_database_never_claim_current_context"),
        ("obsolete_effective", "crates/kinein-core/src/handlers/index.rs",
         'response.stale_because.is_none()', '(response.stale_because.is_none() || response.unit.is_some())',
         "tests::c_project::existing::c1_existing_failed_configure_preserves_selection_and_marks_previous_commands"),
        ("late_workspace_context", "crates/kinein-core/src/handlers/index.rs",
         'if indice.root != expected_root {', 'if indice.root != expected_root && expected_root.as_os_str().is_empty() {',
         "tests::c_project::existing_recovery::c1_existing_late_context_reload_cannot_replace_the_new_workspaces_context"),
        ("late_workspace_index", "crates/kinein-core/src/handlers/index.rs",
         'if indice.root != workspace_root {', 'if indice.root != workspace_root && workspace_root.as_os_str().is_empty() {',
         "tests::c_project::existing_recovery::c1_existing_late_context_reload_cannot_replace_the_new_workspaces_context"),
        ("automatic_kit", "crates/kinein-core/src/cmake/attempt.rs",
         'kit: kit.to_owned(),', 'kit: if kit.is_empty() { preset.unwrap_or_default().to_owned() } else { kit.to_owned() },',
         "tests::c_project::existing_recovery::c1_existing_automatic_retry_keeps_the_default_kit_and_the_absence_of_a_preset"),
        ("configure_lock", "crates/kinein-core/src/cmake/attempt.rs",
         'let lock = crate::platform::configuration_lock(&directory.join(".kinein-configure-lock"))?;',
         'let lock = fs::File::create(directory.join(".kinein-configure-lock"))?;',
         "tests::c_project::existing_recovery::c1_existing_cancelled_configure_retains_obsolete_state_until_successful_retry"),
    ]),
    "c1-environment": ("c1_", [
        ("c1_driver_mode", "crates/kinein-core/src/workspace/c_options.rs",
         "if !status.success() {", "if !status.success() && standard == Some(\"never\") {",
         "tests::c_project::environment::c1_invalid_driver_dialect_and_collision_preserve_workspace_and_allow_retry"),
        ("c1_literal_persistence", "crates/kinein-core/src/workspace/c_options.rs",
         "PRIVATE -std={standard})", "PRIVATE -std=c17)",
         "tests::c_project::environment::c1_literal_dialects_build_and_survive_reopen_configuration_and_workspace_changes"),
        ("c1_token_syntax", "crates/kinein-core/src/workspace/c_options.rs",
         "numeric || literal", "numeric || literal || !value.is_empty()",
         "tests::c_project::environment::c1_future_literal_without_driver_is_preserved_but_invalid_syntax_is_refused"),
        ("c1_workspace_guard", "crates/kinein-core/src/handlers/workspace_creation.rs",
         "if core.workspace_root() != previous_root {",
         "if core.workspace_root() != previous_root && previous_root.is_none() {",
         "tests::c_project::environment::c1_validation_uses_continuations_and_refuses_a_changed_workspace_before_creation"),
    ]),
    "edit-verification": ("edit_verification", CASES),
    "c-host": ("c_host", C_HOST_CASES),
    "c-standard": ("c_standard", C_STANDARD_CASES),
    "c-identity": ("c_identity", [
        (
            "c_document_language_id",
            "crates/kinein-core/src/lsp/document.rs",
            "spec.language_id = language;",
            'spec.language_id = if language == "c" { "cpp" } else { language };',
            "tests::c_identity::wire::c_identity_wire_uses_one_clangd_and_reopens_when_only_context_changes",
        ),
        (
            "c_header_owner",
            "crates/kinein-core/src/index/context/document.rs",
            '\"C\" => Some(\"c\"),',
            '\"C\" => Some(\"cpp\"),',
            "tests::c_identity::c_identity_mixed_targets_grammars_and_clangd_use_real_producers",
        ),
        (
            "same_text_new_language",
            "crates/kinein-core/src/lsp/sync.rs",
            "*previous != language_id",
            "*previous != language_id && false",
            "tests::c_identity::wire::c_identity_wire_uses_one_clangd_and_reopens_when_only_context_changes",
        ),
    ]),
    "c-headers": ("c_identity", [
        ("header_owner", "crates/kinein-core/src/index/context/header.rs",
         "owners.len() != 1", "owners.is_empty()",
         "tests::c_identity::headers::c_identity_header_references_refuse_ambiguity_and_manual_databases"),
        ("header_overlay", "crates/kinein-core/src/lsp/document.rs",
         '(spec.key == "cpp")', '(spec.key == "cpp" && path.as_os_str().is_empty())',
         "tests::c_identity::headers::c_identity_header_wire_preserves_argv_and_invalidates_unsaved_buffers"),
        ("header_generation", "crates/kinein-core/src/lsp/sync.rs",
         "&& self.header_matches(path)", "&& (self.header_matches(path) || !content.is_empty())",
         "tests::c_identity::headers::c_identity_header_wire_preserves_argv_and_invalidates_unsaved_buffers"),
    ]),
    "project-creation": ("project_creation", [
        ("creation_options", "crates/kinein-core/src/handlers/workspace_creation.rs",
         "&params.options", "&Default::default()",
         "tests::project_creation::project_creation_enumerates_parallel_versions_and_preserves_python_selection"),
        ("python_selection", "crates/kinein-core/src/python/env.rs",
         'if root.join(".kinein/python.json").exists() {',
         'if root.join(".kinein/python.json").exists() && false {',
         "tests::project_creation::project_creation_enumerates_parallel_versions_and_preserves_python_selection"),
        ("rustup_download", "crates/kinein-core/src/workspace/create.rs",
         '.env("RUSTUP_AUTO_INSTALL", "0")', '.env("RUSTUP_AUTO_INSTALL", "1")',
         "tests::project_creation::project_creation_local_rust_toolchain_and_edition_are_used_without_downloads"),
    ]),
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
            "create_cmake_project(parent, name, super::cmake_template::Language::C, options)",
            "create_cmake_project(parent, name, super::cmake_template::Language::Cpp, options)",
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

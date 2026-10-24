#!/usr/bin/env python3
"""Exercise the real gate orchestrator and the cargo runner with temporary doubles."""

import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest


SOURCE = Path(__file__).with_name("verificar.sh")
RUST = [
    "cargo fmt --all --check",
    "cargo test --workspace --all-features --no-fail-fast --locked",
    "cargo clippy --workspace --all-targets --all-features --locked -- -D warnings",
]
STATIC = [
    "verificar-deny.sh", "verificar-shell.sh", "verificar-appimage.sh",
    "verificar-qml-fiacao.sh", "verificar-qml-propriedades.sh",
    "verificar-qml-mortas.sh", "verificar-qml-duplicacao.sh",
    "verificar-qml-tokens.sh", "verificar-qml-alcance.sh",
    "verificar-fiacao-ipc.sh", "verificar-atalhos.sh", "verificar-docs.sh",
    "verificar-links-docs.sh", "verificar-arquitetura.sh",
    "verificar-adiados-windows.sh", "verificar-transicao-workspace.sh",
]
LOCAL = [
    "verificar-cpp.sh", "verificar-cpp-testes.sh", "verificar-qml.sh",
    "verificar-exercitacao.sh", "verificar-embarcado.sh",
    "verificar-python-debug.sh", "verificar-clangd-cross.sh",
    "verificar-qml-logica.sh", "verificar-binario-abre.sh",
]
FAKE_TOOL = """#!/bin/sh
name="${0##*/}"
call="$name${*:+ }$*"
printf '%s\\n' "$call" >> "$KINEIN_PROFILE_LOG"
printf '%s %s %s %s %s %s\\n' "$name" "$TMPDIR" "${CMAKE_BUILD_PARALLEL_LEVEL:-}" \\
    "${CARGO_BUILD_JOBS:-}" "${KINEIN_TIDY_JOBS:-}" "${KINEIN_QML_JOBS:-}" \\
    >> "$KINEIN_PROFILE_ENVIRONMENT"
: > "$TMPDIR/$name.left-behind"
if [ "$call" = "${KINEIN_PROFILE_FAIL:-}" ]; then exit 37; fi
if [ "$name" = "verificar-deny.sh" ] && [ "${KINEIN_PROFILE_UNPROVEN:-0}" = 1 ]; then
    printf 'probe\\tlocal integration\\tunavailable\\n' >> "$KINEIN_UNPROVEN_FILE"
fi
"""
FAKE_SCOPE = """#!/bin/sh
printf '%s\\n' "$*" >> "$KINEIN_PROFILE_SCOPE"
while [ $# -gt 0 ]; do
    case "$1" in
        --user | --scope | --quiet | --collect) shift ;;
        -p) shift 2 ;;
        *) break ;;
    esac
done
exec "$@"
"""
GIB = 1024 * 1024


def memory_report(available_kib, swap_used_kib=0):
    return (f"MemTotal: {20 * GIB} kB\nMemAvailable: {available_kib} kB\n"
            f"SwapTotal: {8 * GIB} kB\nSwapFree: {8 * GIB - swap_used_kib} kB\n")


class GateProfiles(unittest.TestCase):
    """Check coverage and failure semantics without Qt, compilers or network."""

    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="kinein-profiles-")
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        scripts = self.root / "scripts"
        scripts.mkdir()
        shutil.copy(SOURCE, scripts / SOURCE.name)
        binaries = self.root / "bin"
        binaries.mkdir()
        for path in ([scripts / name for name in STATIC + LOCAL]
                     + [binaries / name for name in ("cargo", "cmake", "python3")]):
            path.write_text(FAKE_TOOL)
            path.chmod(0o755)
        self.trace = self.root / "trace"
        self.variables = self.root / "variables"
        self.temporary_base = self.root / "tmp"
        self.temporary_base.mkdir()
        self.memory = self.root / "meminfo"
        self.memory.write_text(memory_report(16 * GIB))
        self.environment = {
            **{name: value for name, value in os.environ.items()
               if not name.startswith("KINEIN_")
               and name not in ("CMAKE_BUILD_PARALLEL_LEVEL", "CARGO_BUILD_JOBS")},
            "PATH": f"{binaries}:/usr/bin:/bin",
            "TMPDIR": str(self.temporary_base),
            "KINEIN_MEMINFO": str(self.memory),
            "KINEIN_GATE_SCOPE": "0",
            "KINEIN_PROFILE_ENVIRONMENT": str(self.variables),
            "KINEIN_PROFILE_LOG": str(self.trace),
            "KINEIN_PROFILE_FAIL": "",
            "KINEIN_PROFILE_UNPROVEN": "0",
            "KINEIN_PRESET_DEBUG": "profile-debug",
            "KINEIN_PRESET_RELEASE": "profile-release",
        }

    def run_gate(self, *arguments, builds=False, **environment):
        if builds:
            for preset in ("profile-debug", "profile-release"):
                (self.root / "build" / preset).mkdir(parents=True, exist_ok=True)
        self.trace.unlink(missing_ok=True)
        self.variables.unlink(missing_ok=True)
        result = subprocess.run(
            ["bash", str(self.root / "scripts" / SOURCE.name), *arguments],
            env={**self.environment, **environment}, text=True,
            capture_output=True, timeout=15, check=False,
        )
        calls = self.trace.read_text().splitlines() if self.trace.exists() else []
        return result, calls

    def test_ci_runs_all_static_checks_without_build_directories(self):
        result, calls = self.run_gate("--ci-rapida", "--estrito")
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertEqual([call for call in calls if call.startswith("cargo ")], RUST)
        expected = [name + (" --invariantes" if name == "verificar-appimage.sh" else "")
                    for name in STATIC]
        self.assertEqual([call for call in calls if call.split()[0] in STATIC], expected)
        self.assertFalse(any(call.split()[0] in LOCAL for call in calls))
        self.assertIn("python3 scripts/test_identifier_language.py", calls)
        self.assertIn("python3 scripts/check_identifier_language.py", calls)
        self.assertIn("python3 scripts/module_map.py --check", calls)
        self.assertFalse(any(call.startswith("cmake ") for call in calls))
        self.assertIn("VERDE no perfil CI rapida", result.stdout)
        self.assertNotIn("TUDO VERDE", result.stdout)

    def test_local_quick_keeps_build_and_external_requirements(self):
        missing, _ = self.run_gate("--rapido", "--estrito")
        self.assertNotEqual(missing.returncode, 0)
        result, calls = self.run_gate("--rapido", "--estrito", builds=True)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn("cmake --build --preset profile-debug", calls)
        self.assertIn("verificar-appimage.sh", calls)
        self.assertNotIn("verificar-appimage.sh --invariantes", calls)
        for gate in LOCAL[:-1]:
            self.assertTrue(any(call.split()[0] == gate for call in calls), gate)
        self.assertFalse(any(call.startswith("verificar-binario-abre.sh") for call in calls))
        self.assertFalse(any("--release" in call for call in calls))
        self.assertIn("TUDO VERDE", result.stdout)

    def test_default_complete_builds_before_both_smokes(self):
        result, calls = self.run_gate("--estrito", builds=True)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        sequence = [
            "cmake --build --preset profile-debug",
            "verificar-binario-abre.sh --preset profile-debug",
            "cargo build --release -p kinein-core -p kinein-adapter-sqlite",
            "cmake --build --preset profile-release",
            "verificar-binario-abre.sh --preset profile-release",
        ]
        positions = [calls.index(call) for call in sequence]
        self.assertEqual(positions, sorted(positions))
        self.assertIn("python3 scripts/check_terminal_quiet.py --preset profile-debug", calls)

    def test_a_failed_step_stops_ci_immediate(self):
        result, calls = self.run_gate("--ci-rapida", "--estrito", KINEIN_PROFILE_FAIL=RUST[1])
        self.assertEqual(result.returncode, 37)
        self.assertEqual([call for call in calls if call.startswith("cargo ")], RUST[:2])
        self.assertNotIn("verificar-deny.sh", calls)
        self.assertIn(f"FALHOU em: {RUST[1]}", result.stdout)

    def test_strict_rejects_unproven_in_every_profile(self):
        for mode in ("--ci-rapida", "--rapido", "--completo"):
            with self.subTest(mode=mode):
                result, _ = self.run_gate(
                    mode, "--estrito", builds=True, KINEIN_PROFILE_UNPROVEN="1",
                )
                self.assertEqual(result.returncode, 1)
                self.assertIn("NAO PROVADO nesta maquina (1)", result.stdout)
                self.assertIn("local integration: unavailable", result.stdout)
                self.assertNotIn("TUDO VERDE", result.stdout)
        result, _ = self.run_gate("--ci-rapida", KINEIN_PROFILE_UNPROVEN="1")
        self.assertEqual(result.returncode, 0)
        self.assertIn("VERDE no que esta maquina prova", result.stdout)

    def seen_by_steps(self):
        rows = [line.split(" ") for line in self.variables.read_text().splitlines()]
        return {tuple(row[1:]) for row in rows}

    def test_run_temporaries_live_in_one_folder_removed_on_success_and_failure(self):
        for failing in ("", RUST[1]):
            with self.subTest(failing=failing):
                result, _ = self.run_gate("--ci-rapida", KINEIN_PROFILE_FAIL=failing)
                self.assertEqual(result.returncode, 37 if failing else 0, result.stdout)
                folders = {row[0] for row in self.seen_by_steps()}
                self.assertEqual(len(folders), 1, folders)
                folder = Path(folders.pop())
                self.assertEqual(folder.parent, self.temporary_base)
                self.assertRegex(folder.name, r"^kinein-gate\.[0-9]+$")
                self.assertFalse(folder.exists())
                self.assertEqual(list(self.temporary_base.iterdir()), [])

    def test_a_dead_run_folder_is_removed_and_a_live_one_kept(self):
        dead = self.temporary_base / "kinein-gate.4194305"
        live = self.temporary_base / f"kinein-gate.{os.getpid()}"
        foreign = self.temporary_base / "kinein-platformio-home"
        for folder in (dead, live, foreign):
            folder.mkdir()
            (folder / "content").write_text("x")
        result, _ = self.run_gate("--ci-rapida")
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertFalse(dead.exists())
        self.assertTrue((live / "content").exists())
        self.assertTrue((foreign / "content").exists())

    def test_jobs_follow_available_memory_and_respect_the_environment(self):
        cores = int(subprocess.run(["nproc"], capture_output=True, text=True,
                                   check=True).stdout)
        # C++, cargo, clang-tidy e harnesses QML: teto / (500, 1000, 700, 200),
        # limitados por nucleos + 2, nucleos, nucleos e nucleos.
        caps = (cores + 2, cores, cores, cores)
        for available, by_memory in ((64 * GIB, caps), (6 * GIB, (8, 4, 5, 20)),
                                     (2 * GIB, (2, 1, 1, 5))):
            with self.subTest(available=available):
                self.memory.write_text(memory_report(available))
                result, _ = self.run_gate("--ci-rapida")
                self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
                expected = tuple(str(min(jobs, cap)) for jobs, cap in zip(by_memory, caps))
                self.assertEqual({row[1:] for row in self.seen_by_steps()}, {expected})
                self.assertIn("(pela memoria)", result.stdout)
        result, _ = self.run_gate("--ci-rapida", KINEIN_GATE_CEILING_MIB="999999")
        self.assertEqual({row[1] for row in self.seen_by_steps()}, {"2"},
                         "um teto herdado fora do escopo nao vale")
        result, _ = self.run_gate("--ci-rapida", CMAKE_BUILD_PARALLEL_LEVEL="3",
                                  CARGO_BUILD_JOBS="5", KINEIN_TIDY_JOBS="7",
                                  KINEIN_QML_JOBS="9")
        self.assertEqual({row[1:] for row in self.seen_by_steps()}, {("3", "5", "7", "9")})
        self.assertIn("C++ 3 (do ambiente), cargo 5 (do ambiente), clang-tidy 7 (do ambiente), "
                      f"harnesses QML 9 (do ambiente), {cores} nucleos", result.stdout)

    def test_the_run_asks_for_a_scope_without_swap_under_the_ceiling(self):
        scope = self.root / "bin" / "systemd-run"
        scope.write_text(FAKE_SCOPE)
        scope.chmod(0o755)
        calls = self.root / "scope"
        self.memory.write_text(memory_report(10 * GIB, swap_used_kib=GIB))
        result, _ = self.run_gate("--ci-rapida", KINEIN_GATE_SCOPE="1",
                                  KINEIN_PROFILE_SCOPE=str(calls))
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        probe, run = calls.read_text().splitlines()
        self.assertEqual(probe, "--user --scope --quiet --collect true")
        self.assertTrue(run.startswith(
            "--user --scope --quiet --collect -p MemorySwapMax=0 -p MemoryMax=8192M bash "),
            run)
        self.assertIn("10240 MiB disponiveis, 1024 MiB de swap em uso", result.stdout)

    def test_a_green_run_reports_the_longest_steps(self):
        result, _ = self.run_gate("--ci-rapida")
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertRegex(result.stdout, r"== tempo: [0-9]+ s; as etapas mais longas ==")

    def test_unknown_argument_is_rejected_before_any_step(self):
        result, calls = self.run_gate("--unknown")
        self.assertEqual(result.returncode, 2)
        self.assertEqual(calls, [])


class CargoRunner(unittest.TestCase):
    """The cargo runner gives every test binary a private TMPDIR it removes."""

    RUNNER = Path(__file__).with_name("isolar-temporarios.sh")
    BINARY = """#!/bin/sh
printf '%s' "$TMPDIR" > "$KINEIN_RUNNER_SEEN"
mkdir -p "$TMPDIR/kinein-core-tests/1-left-behind"
exit "${KINEIN_RUNNER_EXIT:-0}"
"""

    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="kinein-runner-")
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.base = self.root / "tmp"
        self.base.mkdir()
        self.seen = self.root / "seen"

    def run_binary(self, folder, **environment):
        binary = self.root / folder / "binary"
        binary.parent.mkdir(parents=True, exist_ok=True)
        binary.write_text(self.BINARY)
        binary.chmod(0o755)
        result = subprocess.run(
            ["sh", str(self.RUNNER), str(binary), "--exact", "name"],
            env={**os.environ, "TMPDIR": str(self.base),
                 "KINEIN_RUNNER_SEEN": str(self.seen), **environment},
            capture_output=True, text=True, timeout=15, check=False,
        )
        return result.returncode, Path(self.seen.read_text())

    def test_a_test_binary_gets_a_private_folder_removed_after_it(self):
        for code in ("0", "101"):
            with self.subTest(code=code):
                status, seen = self.run_binary("target/debug/deps",
                                               KINEIN_RUNNER_EXIT=code)
                self.assertEqual(status, int(code))
                self.assertEqual(seen.parent, self.base)
                self.assertTrue(seen.name.startswith("kinein-teste."), seen)
                self.assertFalse(seen.exists())
                self.assertEqual(list(self.base.iterdir()), [])

    def test_a_program_or_an_explicit_request_keeps_the_caller_folder(self):
        for folder, environment in (("target/debug", {}),
                                    ("target/debug/deps", {"KINEIN_KEEP_TEST_TMP": "1"})):
            with self.subTest(folder=folder):
                status, seen = self.run_binary(folder, **environment)
                self.assertEqual(status, 0)
                self.assertEqual(seen, self.base)
                self.assertTrue((self.base / "kinein-core-tests").exists())


if __name__ == "__main__":
    unittest.main()

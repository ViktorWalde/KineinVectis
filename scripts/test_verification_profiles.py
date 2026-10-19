#!/usr/bin/env python3
"""Exercise the real gate orchestrator with temporary command doubles."""

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
if [ "$call" = "${KINEIN_PROFILE_FAIL:-}" ]; then exit 37; fi
if [ "$name" = "verificar-deny.sh" ] && [ "${KINEIN_PROFILE_UNPROVEN:-0}" = 1 ]; then
    printf 'probe\\tlocal integration\\tunavailable\\n' >> "$KINEIN_UNPROVEN_FILE"
fi
"""


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
        self.environment = {
            **os.environ,
            "PATH": f"{binaries}:/usr/bin:/bin",
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

    def test_unknown_argument_is_rejected_before_any_step(self):
        result, calls = self.run_gate("--unknown")
        self.assertEqual(result.returncode, 2)
        self.assertEqual(calls, [])


if __name__ == "__main__":
    unittest.main()

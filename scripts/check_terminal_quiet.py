#!/usr/bin/env python3
"""G0.5 (roadmap 53 §0.2 e §5.1): abrir pelo terminal se comporta como `code .`.

Chamado de um terminal, `kinein <pasta>` devolve o prompt na hora e nao imprime
nada; a IDE segue sozinha, sem terminal. Ate' 2026-10-01 o terminal ficava
preso e recebia todo o stderr do Qt — o usuario via `wayland-egl` e
"Component is not ready" sem ter feito nada errado.

O teste roda o binario do checkout num PSEUDO-TERMINAL (o desacoplamento so'
acontece quando o stderr e' um terminal) e com XDG_* isolados num diretorio
temporario: nada do perfil real do usuario e' lido ou escrito.

  1. `kinein-vectis <pasta>`   volta em < 300 ms, codigo 0, saida vazia; o filho
                               desacoplado esta' vivo, CHEGA ao primeiro frame
                               (o arquivo de KINEIN_PERF_MARKER_FILE existe),
                               sai sozinho (KINEIN_PERF_EXIT) e o log de
                               diagnostico nao recebeu NADA.

POR QUE O ARQUIVO (2026-10-01). O filho desacoplado tem stdout/stderr em
/dev/null; ate' esta data o teste so' via o PID sumir e concluia "chegou ao
primeiro frame". Sumir tambem e' crash. Provado por mutacao: um `std::abort()`
depois do `engine.load`, so' no processo desacoplado, passava aqui com "a IDE
abre sozinha" — e o verificar-binario-abre nao pega, porque roda sem terminal e
nunca desacopla. Agora o filho grava a linha do marcador num arquivo, e a
ausencia dele reprova.

O XDG_RUNTIME_DIR tambem e' isolado (0700, como o logind cria): sem ele o Qt
avisa "XDG_RUNTIME_DIR not set" no log, e o teste reprovava em container, ssh
sem sessao e CI — maquinas onde a IDE nao tem defeito nenhum.
  2. `--verbose <pasta>`       fica preso ao terminal (e' o modo diagnostico).
  3. `--wait <pasta>`          fica preso ao terminal (como `code -w`).
  4. `<caminho inexistente>`   imprime o motivo e sai com codigo != 0.
  5. `--help`                  imprime a ajuda e sai 0.

Uso: python3 scripts/check_terminal_quiet.py [--preset dev-local]
"""

from __future__ import annotations

import argparse
import os
import pty
import select
import signal
import subprocess
import sys
import tempfile
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from cmake_presets import UI_BINARY, preset_binary_dir  # noqa: E402

RETURN_BUDGET_SECONDS = 0.3
ATTACHED_PROBE_SECONDS = 1.5
CHILD_EXIT_TIMEOUT_SECONDS = 30.0
LOG_RELATIVE_PATH = Path("kinein-vectis/logs/kinein-ui-erros.txt")
FIRST_FRAME_FILE = "first-frame.txt"
FIRST_FRAME_PREFIX = "KINEIN_PERF first_frame_ms="


class PtyRun:
    """Um processo com stdin/stdout/stderr no lado escravo de um pty."""

    def __init__(self, argv: list[str], env: dict[str, str]) -> None:
        master, slave = pty.openpty()
        self.master = master
        self.output = b""
        self.started = time.monotonic()
        self.process = subprocess.Popen(  # noqa: S603 - argv fixo do proprio teste
            argv,
            stdin=slave,
            stdout=slave,
            stderr=slave,
            env=env,
            start_new_session=True,
        )
        os.close(slave)

    def drain(self, seconds: float) -> None:
        deadline = time.monotonic() + seconds
        while True:
            remaining = deadline - time.monotonic()
            if remaining <= 0:
                return
            ready, _, _ = select.select([self.master], [], [], remaining)
            if not ready:
                return
            try:
                chunk = os.read(self.master, 65536)
            except OSError:
                return
            if not chunk:
                return
            self.output += chunk

    def wait(self, seconds: float) -> tuple[int | None, float]:
        """(codigo ou None se ainda roda, segundos desde o inicio)."""
        try:
            code = self.process.wait(timeout=seconds)
        except subprocess.TimeoutExpired:
            return None, time.monotonic() - self.started
        return code, time.monotonic() - self.started

    def stop(self) -> None:
        if self.process.poll() is None:
            os.killpg(self.process.pid, signal.SIGTERM)
            try:
                self.process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                os.killpg(self.process.pid, signal.SIGKILL)
                self.process.wait()
        os.close(self.master)

    def text(self) -> str:
        return self.output.decode("utf-8", errors="replace")


def detached_children(folder: Path) -> list[int]:
    """PIDs da UI cujo argumento e' esta pasta (unica por execucao)."""
    found = subprocess.run(
        ["pgrep", "-f", f"kinein-vectis {folder}$"],
        capture_output=True,
        text=True,
        check=False,
    )
    return [int(pid) for pid in found.stdout.split()]


def pid_alive(pid: int) -> bool:
    try:
        os.kill(pid, 0)
    except ProcessLookupError:
        return False
    return True


def isolated_env(sandbox: Path) -> dict[str, str]:
    env = dict(os.environ)
    xdg_names = ("XDG_CONFIG_HOME", "XDG_DATA_HOME", "XDG_CACHE_HOME", "XDG_STATE_HOME", "XDG_RUNTIME_DIR")
    for name in xdg_names:
        path = sandbox / name.lower()
        path.mkdir(mode=0o700)
        env[name] = str(path)
    env.update(
        {
            "QT_QPA_PLATFORM": "offscreen",
            "KINEIN_PERF_MARKER": "1",
            "KINEIN_PERF_MARKER_FILE": str(sandbox / FIRST_FRAME_FILE),
            "KINEIN_PERF_EXIT": "1",
        }
    )
    env.pop("KINEIN_DETACHED", None)
    return env


def check_quiet_open(binary: Path, env: dict[str, str], sandbox: Path) -> tuple[list[str], float]:
    failures: list[str] = []
    folder = sandbox / "project"
    folder.mkdir()
    run = PtyRun([str(binary), str(folder)], env)
    try:
        code, elapsed = run.wait(5.0)
        if code is None:
            failures.append("`kinein <pasta>` nao devolveu o terminal em 5 s (nao desacoplou)")
            return failures, elapsed
        if code != 0:
            failures.append(f"`kinein <pasta>` saiu com {code}, esperado 0")
        if elapsed > RETURN_BUDGET_SECONDS:
            failures.append(
                f"`kinein <pasta>` levou {elapsed * 1000:.0f} ms para devolver o terminal"
                f" (limite {RETURN_BUDGET_SECONDS * 1000:.0f} ms)"
            )

        # O pty fica ABERTO e drenado ate' o filho sair: um filho que ainda
        # escrevesse no terminal depois de o pai voltar seria exatamente o
        # defeito, e com o mestre fechado a escrita se perderia sem reprovar.
        children = detached_children(folder)
        if not children:
            failures.append("nenhuma IDE desacoplada ficou rodando para a pasta")
        deadline = time.monotonic() + CHILD_EXIT_TIMEOUT_SECONDS
        while time.monotonic() < deadline and any(pid_alive(pid) for pid in children):
            run.drain(0.1)
        run.drain(0.2)
        leftover = [pid for pid in children if pid_alive(pid)]
        if leftover:
            failures.append(
                f"a IDE desacoplada nao chegou ao primeiro frame em {CHILD_EXIT_TIMEOUT_SECONDS:.0f} s"
            )
            for pid in leftover:
                os.killpg(pid, signal.SIGKILL)
        elif children:
            # O PID sumiu: so' o arquivo distingue "saiu depois do primeiro
            # frame" de "morreu antes" (crash, abort, exit precoce).
            first_frame = Path(env["KINEIN_PERF_MARKER_FILE"])
            content = first_frame.read_text(errors="replace") if first_frame.exists() else ""
            if not content.startswith(FIRST_FRAME_PREFIX):
                failures.append(
                    "a IDE desacoplada saiu SEM primeiro frame (crash ou saida precoce):"
                    f" {first_frame.name} {'vazio' if first_frame.exists() else 'ausente'}"
                )
        if run.text().strip():
            failures.append(f"o terminal recebeu saida (do pai ou da IDE desacoplada):\n{run.text()}")
    finally:
        run.stop()

    log_file = Path(env["XDG_CACHE_HOME"]) / LOG_RELATIVE_PATH
    if log_file.exists() and log_file.read_text(errors="replace").strip():
        failures.append(
            "o log de diagnostico recebeu mensagens (a rede de seguranca nao deveria"
            f" receber nada em uso normal, 53 §0.1):\n{log_file.read_text(errors='replace')}"
        )
    return failures, elapsed


def check_stays_attached(binary: Path, env: dict[str, str], sandbox: Path, flag: str) -> list[str]:
    folder = sandbox / f"project{flag}"
    folder.mkdir()
    attached_env = dict(env)
    # Preso ao terminal, ele precisa continuar vivo durante a sonda.
    attached_env.pop("KINEIN_PERF_EXIT", None)
    run = PtyRun([str(binary), flag, str(folder)], attached_env)
    try:
        code, _ = run.wait(ATTACHED_PROBE_SECONDS)
        if code is not None:
            return [f"`kinein {flag} <pasta>` devolveu o terminal (saiu com {code}); devia ficar preso"]
        return []
    finally:
        run.stop()


def check_prints_and_exits(binary: Path, env: dict[str, str], args: list[str], expect_zero: bool) -> list[str]:
    run = PtyRun([str(binary), *args], env)
    try:
        code, _ = run.wait(5.0)
        run.drain(0.2)
        label = " ".join(args)
        if code is None:
            return [f"`kinein {label}` nao terminou em 5 s"]
        failures = []
        if expect_zero and code != 0:
            failures.append(f"`kinein {label}` saiu com {code}, esperado 0")
        if not expect_zero and code == 0:
            failures.append(f"`kinein {label}` saiu com 0, esperado erro")
        if not run.text().strip():
            failures.append(f"`kinein {label}` nao imprimiu nada; essa resposta pertence ao terminal")
        return failures
    finally:
        run.stop()


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--preset", default="dev-local")
    options = parser.parse_args()

    binary = preset_binary_dir(options.preset) / UI_BINARY
    if not binary.is_file():
        print(f"erro: binario ausente: {binary} (rode cmake --build --preset {options.preset})", file=sys.stderr)
        return 1

    failures: list[str] = []
    with tempfile.TemporaryDirectory(prefix="kinein-g05-") as raw_sandbox:
        sandbox = Path(raw_sandbox)
        env = isolated_env(sandbox)
        quiet_failures, elapsed = check_quiet_open(binary, env, sandbox)
        failures += quiet_failures
        failures += check_stays_attached(binary, env, sandbox, "--verbose")
        failures += check_stays_attached(binary, env, sandbox, "--wait")
        failures += check_prints_and_exits(binary, env, [str(sandbox / "missing")], expect_zero=False)
        failures += check_prints_and_exits(binary, env, ["--help"], expect_zero=True)

    if failures:
        print("✗ terminal mudo (G0.5) reprovou:", file=sys.stderr)
        for failure in failures:
            print(f"  - {failure}", file=sys.stderr)
        return 1
    print(f"✓ terminal mudo: volta em {elapsed * 1000:.0f} ms, sem imprimir, a IDE abre sozinha; --verbose/--wait/erro/--help no terminal")
    return 0


if __name__ == "__main__":
    sys.exit(main())

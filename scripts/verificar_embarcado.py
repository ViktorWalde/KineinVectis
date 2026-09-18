#!/usr/bin/env python3
"""O ciclo de embarcado, fim a fim, no QEMU — pelo core REAL, por stdio.

Ver scripts/verificar-embarcado.sh para o porque.

O que se prova, na ordem:
  1. `toolchain.setKit` grava adaptador `gdb`, `remoteTarget` e `debugServer`
  2. `debug.setBreakpoints` em main.c antes de subir (nasce PENDING no GDB)
  3. `debug.start` sobe o QEMU (servidor), o `gdb -i dap`, faz o `attach`
     — e a IDE recebe `event.debug.stopped { reason: "attach" }`
  4. `debug.continue` -> `event.debug.stopped { reason: "breakpoint",
     file: main.c, line: 4 }` — o breakpoint pendente VERIFICOU
  5. `debug.evaluate contador` le 0; continua; le 1
  6. `debug.stop` -> `event.debug.finished`, e o QEMU morreu junto
"""
from __future__ import annotations

import json
import os
import pathlib
import queue
import shutil
import subprocess
import sys
import tempfile
import threading
import time

RAIZ = pathlib.Path(__file__).resolve().parent.parent
FIXTURE = RAIZ / "scripts" / "fixtures" / "embarcado"
PORTA = 3389  # fora do 3333 (OpenOCD) e do 1234 (QEMU), para nao brigar com um aberto
MAQUINA = "lm3s6965evb"


class Core:
    """Conduz o core por stdio. Um thread leitor alimenta uma fila, porque
    `select` sobre o stream de TEXTO nao enxerga o que ja' esta no buffer do
    Python — foi esse o defeito que escondeu o `event.debug.finished` numa
    primeira versao deste script (2026-09-11)."""

    def __init__(self, binario: pathlib.Path) -> None:
        # O config global (workspaces recentes) e' ISOLADO: um gate que abre
        # pastas temporarias nao pode enche-las na tela inicial do autor
        # (pente-fino 2026-09-18: tres "caminho ausente" nos recentes).
        ambiente = dict(os.environ)
        ambiente["XDG_CONFIG_HOME"] = tempfile.mkdtemp(prefix="kinein-gate-config-")
        self.proc = subprocess.Popen(
            [str(binario)],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
            text=True,
            env=ambiente,
        )
        self.seq = 0
        self.respostas: dict[int, dict] = {}
        self.eventos: list[dict] = []
        self._fila: queue.Queue = queue.Queue()
        threading.Thread(target=self._ler, daemon=True).start()

    def _ler(self) -> None:
        for linha in self.proc.stdout:
            linha = linha.strip()
            if linha:
                self._fila.put(json.loads(linha))
        self._fila.put(None)

    def _bombear(self, timeout: float) -> bool:
        try:
            msg = self._fila.get(timeout=max(timeout, 0.0))
        except queue.Empty:
            return False
        if msg is None:
            return False
        if "id" in msg:
            self.respostas[msg["id"]] = msg
        elif "method" in msg:
            self.eventos.append(msg)
        return True

    def rpc(self, metodo: str, params: dict | None = None, timeout: float = 30.0) -> dict:
        self.seq += 1
        pedido = {"jsonrpc": "2.0", "id": self.seq, "method": metodo, "params": params or {}}
        self.proc.stdin.write(json.dumps(pedido) + "\n")
        self.proc.stdin.flush()
        fim = time.monotonic() + timeout
        while self.seq not in self.respostas:
            if not self._bombear(fim - time.monotonic()):
                raise RuntimeError(f"{metodo}: sem resposta em {timeout}s")
        msg = self.respostas.pop(self.seq)
        if "error" in msg:
            raise RuntimeError(f"{metodo}: {msg['error']}")
        return msg.get("result") or {}

    def evento(self, nome: str, timeout: float = 30.0) -> dict:
        fim = time.monotonic() + timeout
        while True:
            for i, msg in enumerate(self.eventos):
                if msg["method"] == nome:
                    return self.eventos.pop(i).get("params") or {}
            if not self._bombear(fim - time.monotonic()):
                raise RuntimeError(f"evento {nome} nao chegou em {timeout}s")


def compilar(destino: pathlib.Path) -> pathlib.Path:
    elf = destino / "fixture.elf"
    subprocess.run(
        ["arm-none-eabi-gcc", "-mcpu=cortex-m3", "-mthumb", "-g", "-O0", "-nostartfiles",
         "-ffreestanding", "-T", "link.ld", "startup.c", "main.c", "-o", str(elf)],
        cwd=destino, check=True,
    )
    return elf


def main() -> int:
    for ferramenta in ("arm-none-eabi-gcc", "qemu-system-arm", "gdb"):
        if shutil.which(ferramenta) is None:
            print(f"  - {ferramenta}: ausente nesta maquina (nao reprova; o ciclo fica NAO PROVADO aqui)")
            return 0
    core_bin = RAIZ / "target" / "debug" / "kinein-core"
    if not core_bin.exists():
        print(f"erro: {core_bin} ausente; rode cargo build -p kinein-core", file=sys.stderr)
        return 2

    with tempfile.TemporaryDirectory(prefix="kinein-embarcado-") as tmp:
        raiz = pathlib.Path(tmp)
        for nome in ("link.ld", "startup.c", "main.c"):
            shutil.copy(FIXTURE / nome, raiz / nome)
        elf = compilar(raiz)
        main_c = str(raiz / "main.c")

        core = Core(core_bin)
        try:
            core.rpc("workspace.open", {"path": str(raiz)})
            core.rpc("toolchain.set", {"role": "debugAdapter", "id": "gdb"})
            kit = core.rpc("toolchain.setKit", {
                "remoteTarget": f"localhost:{PORTA}",
                "debugServer": f"qemu-system-arm -machine {MAQUINA} -nographic -S -gdb tcp::{PORTA} -kernel {{program}}",
            })
            assert kit["remoteTarget"] == f"localhost:{PORTA}", kit
            assert "{program}" in kit["debugServer"], kit
            escolha = [s for s in kit["selections"] if s["role"] == "debugAdapter"][0]
            assert escolha.get("id") == "gdb", escolha

            core.rpc("debug.setBreakpoints", {"file": main_c, "breakpoints": [{"line": 4}]})
            core.rpc("debug.start", {"program": str(elf)})
            core.evento("event.debug.started")
            parado = core.evento("event.debug.stopped")
            assert parado.get("reason") == "attach", parado

            t0 = time.monotonic()
            core.rpc("debug.continue")
            parado = core.evento("event.debug.stopped")
            assert parado.get("reason") == "breakpoint", parado
            assert parado.get("line") == 4 and str(parado.get("file", "")).endswith("main.c"), parado
            fid = core.rpc("debug.stackTrace")["frames"][0]["id"]
            primeiro = core.rpc("debug.evaluate", {"expression": "contador", "frameId": fid})["result"]
            # O escopo que a UI ve NAO e' o de registradores (o GDB os lista
            # primeiro; o core prefere Globals/Locals).
            nomes = [v["name"] for v in core.rpc("debug.variables", {"frameId": fid})["variables"]]
            assert "contador" in nomes, nomes
            assert not any(n in ("r0", "pc", "sp", "lr") for n in nomes), nomes

            core.rpc("debug.continue")
            core.evento("event.debug.stopped")
            fid = core.rpc("debug.stackTrace")["frames"][0]["id"]
            segundo = core.rpc("debug.evaluate", {"expression": "contador", "frameId": fid})["result"]
            assert (primeiro, segundo) == ("0", "1"), (primeiro, segundo)

            core.rpc("debug.stop")
            code = core.evento("event.debug.finished").get("exitCode")
            ciclo_ms = int((time.monotonic() - t0) * 1000)

            # build.size: o `arm-none-eabi-size` do kit mede o ELF e o core
            # cruza com o linker script. Prova o mesmo par do 4.2 contra a
            # ferramenta REAL — o parse unitario nao ve mudanca de binutils.
            core.rpc("toolchain.set", {"role": "cCompiler", "id": "arm-none-eabi-gcc"})
            tam = core.rpc("build.size", {"program": str(elf)})
            assert tam["tool"] == "arm-none-eabi-size", tam["tool"]
            regioes = {r["name"]: r for r in tam["regions"]}
            assert "FLASH" in regioes and "SRAM" in regioes, tam["regions"]
            # 132 B de codigo+vetor na FLASH de 256 KiB; 4 B de .bss na SRAM de
            # 64 KiB. O `.comment`/`.ARM.attributes` (addr 0) NAO contam.
            assert regioes["FLASH"]["used"] == 132, regioes["FLASH"]
            assert regioes["FLASH"]["size"] == 256 * 1024, regioes["FLASH"]
            assert regioes["SRAM"]["used"] == 4, regioes["SRAM"]
        finally:
            core.proc.stdin.close()
            core.proc.wait(10)

    # O servidor tem de MORRER com a sessao: um QEMU orfao na porta e' o
    # proximo debug.start falhando. Espera curta, porque o kill nao e' instantaneo.
    fim = time.monotonic() + 3
    while time.monotonic() < fim:
        vivos = subprocess.run(
            ["pgrep", "-f", f"gdb tcp::{PORTA}"], capture_output=True, text=True
        ).stdout.split()
        if not vivos:
            break
        time.sleep(0.2)
    else:
        print(f"✗ o QEMU sobreviveu a sessao (pids {vivos})", file=sys.stderr)
        for pid in vivos:
            subprocess.run(["kill", pid], check=False)
        return 1

    print(f"embarcado no QEMU ({MAQUINA}, gdb -i dap): attach, breakpoint em main.c:4, "
          f"contador 0 -> 1, variaveis sem registradores, exitCode {code}, servidor morto "
          f"com a sessao; {ciclo_ms} ms do continue ao stop. "
          f"build.size: FLASH 132/262144, SRAM 4/65536 (arm-none-eabi-size)")
    return 0


if __name__ == "__main__":
    sys.exit(main())

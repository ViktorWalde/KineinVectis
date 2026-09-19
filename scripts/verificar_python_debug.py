#!/usr/bin/env python3
"""O ciclo de DEPURAR Python, pelo core de verdade e por stdio (fatia 4 da
cadeia Python do roadmaps/41, 2026-09-13).

O que prova:
  1. `workspace.open` num projeto Python cujo interpretador TEM o debugpy
     (o `.venv/bin/python` do workspace — ou o `$VIRTUAL_ENV` que este script
     passa ao core, que e' o primeiro degrau da precedencia do 29 §4.1)
  2. `debug.setBreakpoints` em app.py:2 antes de subir
  3. `debug.start {}` acha o app.py sozinho (ponto de entrada por evidencia),
     escolhe o debugpy DO interpretador (`-m debugpy.adapter`), faz o launch
     com `console: internalConsole` — e a IDE recebe `event.debug.started`
  4. `event.debug.stopped { reason: "breakpoint", line: 2 }`; stackTrace com
     `soma` no topo; variaveis a=2 e b=3 (Locals, nao Globals); evaluate `a+b`
  5. `debug.continue` -> a saida do programa chega por `event.debug.output`
     (stdout: "resultado 5") -> `event.debug.finished { exitCode: 0 }`
  6. o adaptador MORREU com a sessao (o debugpy nao sai sozinho no disconnect —
     medido no 1.8.21 — e' o kill da sessao que o encerra)
  7. o mesmo ciclo com um PACOTE como ponto de entrada (`pacote/__main__.py`,
     sem main.py): `debug.start {}` lanca `-m pacote` (o `module` do debugpy,
     2026-09-13) e para no breakpoint dentro do pacote
  8. attach TCP a debugpy.listen: breakpoint, locais, evaluate e reconexao;
     debug.stop (pausado/rodando) e workspace.close preservam o processo.

Uso: verificar_python_debug.py <interpretador-com-debugpy>
"""
from __future__ import annotations

import json
import os
import pathlib
import queue
import subprocess
import sys
import tempfile
import threading
import time

from verificar_python_attach import verify_attach

RAIZ = pathlib.Path(__file__).resolve().parents[1]


class Core:
    def __init__(self, binario: pathlib.Path, ambiente: dict[str, str], log: pathlib.Path) -> None:
        # O stderr do core vai para um arquivo. O stderr dos adaptadores
        # filhos ainda e' uma pendencia propria do roadmap. Um
        # "o adapter nao respondeu" sem o stderr e' um sintoma sem causa.
        self.log = open(log, "w", encoding="utf-8")
        self.proc = subprocess.Popen(
            [str(binario)],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=self.log,
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

    @staticmethod
    def _texto_do_render(params: dict) -> str:
        """O texto de um `event.terminal.render`, linhas desenroladas (uma
        linha logica mais larga que o grid continua na seguinte)."""
        cols = int(params.get("cols") or 80)
        linhas: list[str] = []
        continua = False
        for row in params.get("lines") or []:
            bruto = "".join(s.get("text", "") for s in row)
            if continua and linhas:
                linhas[-1] += bruto.rstrip()
            else:
                linhas.append(bruto.rstrip())
            continua = len(bruto) >= cols and not bruto.endswith(" ")
        return "\n".join(linhas).rstrip()

    def texto_da_execucao(self, terminal_id: str, timeout: float = 15.0) -> tuple[str, int | None]:
        """O que uma execucao (uma sessao de terminal desde 2026-09-18) mostrou
        ate' fechar: o texto do ULTIMO render da sessao e o `exitCode`."""
        fim = time.monotonic() + timeout
        texto = ""
        while time.monotonic() < fim:
            fechado = None
            restantes = []
            for msg in self.eventos:
                params = msg.get("params") or {}
                if params.get("id") == terminal_id and msg["method"] == "event.terminal.render":
                    texto = self._texto_do_render(params)
                elif params.get("id") == terminal_id and msg["method"] == "event.terminal.closed":
                    fechado = params
                else:
                    restantes.append(msg)
            self.eventos = restantes
            if fechado is not None:
                prazo = time.monotonic() + 0.4
                while time.monotonic() < prazo:
                    self._bombear(0.05)
                restantes = []
                for msg in self.eventos:
                    params = msg.get("params") or {}
                    if params.get("id") == terminal_id and msg["method"] == "event.terminal.render":
                        texto = self._texto_do_render(params)
                    else:
                        restantes.append(msg)
                self.eventos = restantes
                return texto, fechado.get("exitCode")
            self._bombear(0.1)
        raise RuntimeError(f"a execucao {terminal_id} nao fechou em {timeout}s; visto: {texto!r}")

    def evento(self, nome: str, timeout: float = 30.0, onde=None) -> dict:
        fim = time.monotonic() + timeout
        while True:
            for i, msg in enumerate(self.eventos):
                params = msg.get("params") or {}
                if msg["method"] == nome and (onde is None or onde(params)):
                    return self.eventos.pop(i).get("params") or {}
            if not self._bombear(fim - time.monotonic()):
                raise RuntimeError(f"evento {nome} nao chegou em {timeout}s; vistos: "
                                   f"{[e['method'] for e in self.eventos]}")


def main() -> int:
    if len(sys.argv) != 2:
        print(__doc__, file=sys.stderr)
        return 2
    # `absolute()`, NAO `resolve()`: o python de um venv e' um symlink para o
    # do sistema, e resolver o link jogaria fora o venv (e o debugpy com ele).
    interpretador = pathlib.Path(sys.argv[1]).absolute()
    core_bin = RAIZ / "target" / "debug" / "kinein-core"
    if not core_bin.exists():
        print(f"erro: {core_bin} ausente; rode cargo build -p kinein-core", file=sys.stderr)
        return 2

    with tempfile.TemporaryDirectory(prefix="kinein-python-debug-") as tmp:
        raiz = pathlib.Path(tmp).resolve()
        (raiz / "pyproject.toml").write_text('[project]\nname = "demo"\nversion = "0.1.0"\n')
        app = raiz / "app.py"
        app.write_text(
            "def soma(a, b):\n"
            "    total = a + b\n"
            "    return total\n"
            "\n"
            "\n"
            'print("inicio")\n'
            "resultado = soma(2, 3)\n"
            'print("resultado", resultado)\n'
        )
        # O interpretador entra pela precedencia do 29 §4.1: `$VIRTUAL_ENV`
        # (o ambiente onde o debugpy esta') — o core LE a variavel, nao a
        # escreve. Um interpretador fora de venv entra como "sistema" pelo PATH.
        ambiente = dict(os.environ)
        ambiente.pop("VIRTUAL_ENV", None)
        # Recentes isolados: o gate nao polui a tela inicial do autor.
        ambiente["XDG_CONFIG_HOME"] = tempfile.mkdtemp(prefix="kinein-gate-config-")
        venv = interpretador.parent.parent
        if (venv / "pyvenv.cfg").is_file():
            ambiente["VIRTUAL_ENV"] = str(venv)
        else:
            ambiente["PATH"] = f"{interpretador.parent}:{ambiente.get('PATH', '')}"

        log = raiz / "core-stderr.log"
        core = Core(core_bin, ambiente, log)
        try:
            core.rpc("workspace.open", {"path": str(raiz)})
            status = core.rpc("python.status")
            interp = (status.get("interpreter") or {}).get("interpreter", "")
            assert interp == str(interpretador), (interp, str(interpretador))

            core.rpc("debug.setBreakpoints", {"file": str(app), "breakpoints": [{"line": 2}]})
            t0 = time.monotonic()
            inicio = core.rpc("debug.start", {})
            assert inicio["program"] == str(app), inicio
            core.evento("event.debug.started")
            parado = core.evento("event.debug.stopped")
            assert parado.get("reason") == "breakpoint", parado
            assert parado.get("line") == 2 and str(parado.get("file", "")).endswith("app.py"), parado

            quadros = core.rpc("debug.stackTrace")["frames"]
            assert quadros[0]["name"] == "soma", quadros
            fid = quadros[0]["id"]
            variaveis = {v["name"]: v["value"] for v in core.rpc("debug.variables", {"frameId": fid})["variables"]}
            assert variaveis.get("a") == "2" and variaveis.get("b") == "3", variaveis
            assert "__name__" not in variaveis, ("o escopo da UI e' Locals, nao Globals", variaveis)
            soma = core.rpc("debug.evaluate", {"expression": "a + b", "frameId": fid})["result"]
            assert soma == "5", soma

            core.rpc("debug.continue")
            saida = core.evento("event.debug.output", onde=lambda p: "resultado" in p.get("line", ""))
            assert saida.get("line") == "resultado 5", saida
            fim = core.evento("event.debug.finished")
            assert fim.get("exitCode") == 0, fim
            ciclo_ms = int((time.monotonic() - t0) * 1000)

            # 7. O pacote como ponto de entrada: sem main.py, `-m pacote`.
            # (os breakpoints do app.py saem ANTES de apaga-lo: o setBreakpoints
            # exige que o arquivo exista)
            core.rpc("debug.setBreakpoints", {"file": str(app), "breakpoints": []})
            app.unlink()
            pacote = raiz / "pacote"
            pacote.mkdir()
            (pacote / "__init__.py").write_text("def dobro(x):\n    y = x * 2\n    return y\n")
            (pacote / "__main__.py").write_text('from pacote import dobro\nprint("dobro", dobro(21))\n')
            core.rpc("debug.setBreakpoints", {"file": str(pacote / "__init__.py"), "breakpoints": [{"line": 2}]})
            inicio = core.rpc("debug.start", {})
            assert inicio["program"] == "-m pacote", inicio
            core.evento("event.debug.started")
            parado = core.evento("event.debug.stopped")
            assert parado.get("reason") == "breakpoint", parado
            assert str(parado.get("file", "")).endswith("pacote/__init__.py") and parado.get("line") == 2, parado
            fid = core.rpc("debug.stackTrace")["frames"][0]["id"]
            assert core.rpc("debug.evaluate", {"expression": "x", "frameId": fid})["result"] == "21"
            core.rpc("debug.continue")
            saida = core.evento("event.debug.output", onde=lambda p: "dobro" in p.get("line", ""))
            assert saida.get("line") == "dobro 42", saida
            assert core.evento("event.debug.finished").get("exitCode") == 0
            verify_attach(core, raiz, interpretador)
        except Exception:
            core.log.flush()
            print("--- stderr do core (ultimas linhas) ---", file=sys.stderr)
            print("\n".join(log.read_text(errors="replace").splitlines()[-40:]), file=sys.stderr)
            raise
        finally:
            core.proc.stdin.close()
            core.proc.wait(10)
            core.log.close()

    # O adaptador tem de morrer com a sessao (o debugpy nao sai sozinho no
    # disconnect; e' o kill da sessao que o encerra).
    fim = time.monotonic() + 3
    while time.monotonic() < fim:
        vivos = subprocess.run(
            ["pgrep", "-f", f"{interpretador} -m debugpy.adapter"], capture_output=True, text=True
        ).stdout.split()
        if not vivos:
            break
        time.sleep(0.2)
    else:
        print(f"✗ o adaptador debugpy sobreviveu a sessao (pids {vivos})", file=sys.stderr)
        for pid in vivos:
            subprocess.run(["kill", pid], check=False)
        return 1

    print(f"depurar Python (debugpy do interpretador do projeto): breakpoint em app.py:2, "
          f"a=2 b=3, a+b=5, saida do programa, exitCode 0, adaptador morto — ciclo em {ciclo_ms} ms; "
          f"e o pacote como `-m pacote` (breakpoint em pacote/__init__.py:2, x=21, dobro 42)")
    return 0


if __name__ == "__main__":
    sys.exit(main())

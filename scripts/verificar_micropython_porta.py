"""A porta escolhida chega ao mpremote: prova REAL contra o kinein-core.

Uso: python3 scripts/verificar_micropython_porta.py

Reutiliza a classe `Core` do gate Python (`verificar_python_debug.py`): um
so' cliente JSON-RPC, o binario `target/debug/kinein-core` de verdade. O
mpremote e' FALSO (um shell script que ecoa os argv), porque a prova e' o que
o core PASSA a ele — `connect <porta> run <arquivo>`, sem shell no meio — e
isso se prova sem placa. O que acontece na placa depende do mpremote real e
esta' registrado como limitacao no roadmap 40 §7.39.
"""

from __future__ import annotations

import os
import pathlib
import shutil
import sys
import tempfile

RAIZ = pathlib.Path(__file__).resolve().parents[1]
sys.path.insert(0, str(RAIZ / "scripts"))
from verificar_python_debug import Core


def main() -> int:
    core_bin = RAIZ / "target" / "debug" / "kinein-core"
    if not core_bin.exists():
        print(
            f"erro: {core_bin} ausente; rode cargo build -p kinein-core",
            file=sys.stderr,
        )
        return 2
    raiz = pathlib.Path(tempfile.mkdtemp(prefix="kinein-mp-porta-"))
    try:
        return prova(core_bin, raiz)
    finally:
        shutil.rmtree(raiz, ignore_errors=True)


def prova(core_bin: pathlib.Path, raiz: pathlib.Path) -> int:
    bin_dir = raiz / "bin"
    bin_dir.mkdir()
    falso = bin_dir / "mpremote"
    falso.write_text('#!/bin/sh\necho "mpremote-falso $*"\n')
    falso.chmod(0o755)
    ws = raiz / "placa"
    ws.mkdir()
    (ws / "main.py").write_text("import machine\nprint('led')\n")
    (ws / "util.py").write_text("def f():\n    pass\n")

    ambiente = {
        k: v for k, v in os.environ.items() if k not in {"VIRTUAL_ENV", "PYTHONPATH"}
    }
    # So' o mpremote falso: o resto do PATH nao pode oferecer outro.
    ambiente["PATH"] = f"{bin_dir}:/usr/bin:/bin"
    # Recentes isolados: o gate nao polui a tela inicial do autor.
    ambiente["XDG_CONFIG_HOME"] = str(raiz / "config")
    core = Core(core_bin, ambiente, raiz / "core.log")
    falhas = 0

    def cabecalho(titulo: str) -> None:
        print(f"-- {titulo}")

    core.rpc("workspace.open", {"path": str(ws)})
    core.evento("event.index.finished", timeout=60)

    def roda(metodo: str, params: dict, esperado_saida: str) -> None:
        nonlocal falhas
        resultado = core.rpc(metodo, params)
        # O falso ecoa UMA linha: os argv como o core os passou. A execucao
        # e' uma sessao de terminal (2026-09-18): le-se o render dela.
        texto, codigo = core.texto_da_execucao(resultado["terminalId"], timeout=15)
        linha = texto.splitlines()[0] if texto else ""
        ok = linha == esperado_saida and codigo == 0
        print(f"   {metodo} {params} -> command={resultado['command']!r}")
        print(f"   {'ok ' if ok else 'ERRO'} mpremote recebeu: {linha}")
        if not ok:
            falhas += 1

    cabecalho("sem porta escolhida: o mpremote escolhe (campo ausente)")
    roda("run.start", {}, "mpremote-falso run main.py")
    roda(
        "run.script",
        {"path": str(ws / "util.py")},
        f"mpremote-falso run {ws / 'util.py'}",
    )

    cabecalho("porta escolhida na tela: connect <porta> run, nos DOIS gestos")
    roda(
        "run.start",
        {"device": "/dev/ttyACM7"},
        "mpremote-falso connect /dev/ttyACM7 run main.py",
    )
    roda(
        "run.script",
        {"path": str(ws / "util.py"), "device": "/dev/ttyUSB3"},
        f"mpremote-falso connect /dev/ttyUSB3 run {ws / 'util.py'}",
    )

    cabecalho("troca de porta entre execucoes: a ultima escolha vale")
    roda(
        "run.start",
        {"device": "/dev/ttyUSB0"},
        "mpremote-falso connect /dev/ttyUSB0 run main.py",
    )
    roda("run.start", {}, "mpremote-falso run main.py")

    cabecalho("contradicao e vazio sao recusados antes de rodar")
    for metodo, params, trecho in [
        (
            "run.start",
            {"command": "echo host", "device": "/dev/ttyUSB0"},
            "device so' vale sem command",
        ),
        ("run.start", {"device": ""}, "porta serial"),
        ("run.script", {"path": str(ws / "util.py"), "device": "  "}, "porta serial"),
    ]:
        try:
            core.rpc(metodo, params)
            print(f"   ERRO {metodo} {params}: aceitou")
            falhas += 1
        except RuntimeError as erro:
            ok = trecho in str(erro)
            print(f"   {'ok ' if ok else 'ERRO'} {metodo} {params}: {str(erro)[:110]}")
            if not ok:
                falhas += 1

    core.proc.stdin.close()
    core.proc.wait(timeout=10)
    print(
        "\n"
        + (
            "✓ porta do MicroPython: tudo verde"
            if falhas == 0
            else f"✗ {falhas} falha(s)"
        )
    )
    return 0 if falhas == 0 else 1


if __name__ == "__main__":
    sys.exit(main())

#!/usr/bin/env python3
"""O clangd do kit cross NAO marca os cabecalhos de libstdc++ ARM.

Ver scripts/verificar-clangd-cross.sh para o porque.

Mede contra o clangd e o arm-none-eabi-g++ REAIS desta maquina, nos dois lados:
  SEM `--query-driver` -> clangd nao acha <array>/<cstdint> do GCC ARM (erros)
  COM o driver que `Toolchain::clangd_args` emite -> zero erros
Se o "sem" NAO der erro nesta maquina (um clangd que acha tudo sozinho), o
teste se declara NAO CONCLUSIVO em vez de passar em falso — a defesa so' vale
onde o defeito existe.
"""
from __future__ import annotations

import json
import pathlib
import re
import shutil
import subprocess
import sys
import tempfile

FW = """#include <cstdint>
#include <array>
std::array<std::uint32_t, 4> buf{};
std::uint32_t soma() { return buf.size(); }
"""


def erros(clangd: str, arquivo: pathlib.Path, driver: str | None) -> int:
    cmd = [clangd, f"--check={arquivo}", "--log=error"]
    if driver:
        cmd.append(f"--query-driver={driver}")
    saida = subprocess.run(cmd, capture_output=True, text=True, timeout=60).stderr
    m = re.search(r"All checks completed, (\d+) errors", saida)
    if m is None:
        # Sem a linha de sumario, trata cada diagnostico de include nao achado.
        return saida.count("IncludeCleaner: Failed")
    return int(m.group(1))


def main() -> int:
    clangd = shutil.which("clangd") or shutil.which("clangd-qt6")
    cross = shutil.which("arm-none-eabi-g++")
    if clangd is None or cross is None:
        print("  - clangd/arm-none-eabi-g++ ausente nesta maquina (nao reprova)")
        return 0

    with tempfile.TemporaryDirectory(prefix="kinein-clangd-") as tmp:
        raiz = pathlib.Path(tmp)
        fonte = raiz / "fw.cpp"
        fonte.write_text(FW)
        (raiz / "compile_commands.json").write_text(json.dumps([{
            "directory": str(raiz),
            "file": str(fonte),
            "command": f"{cross} -mcpu=cortex-m3 -mthumb -std=c++17 -c fw.cpp",
        }]))

        sem = erros(clangd, fonte, None)
        com = erros(clangd, fonte, cross)

    if sem == 0:
        print(f"  ? este clangd acha os cabecalhos do cross sozinho (0 erros sem driver); "
              f"teste NAO CONCLUSIVO, com driver = {com}")
        return 0
    if com != 0:
        print(f"✗ com --query-driver={cross} o clangd ainda marca {com} erro(s) "
              f"(sem driver eram {sem})", file=sys.stderr)
        return 1
    print(f"clangd cross: {sem} erro(s) de cabecalho SEM --query-driver, 0 COM ele "
          f"(o que Toolchain::clangd_args passa para um kit arm-none-eabi)")
    return 0


if __name__ == "__main__":
    sys.exit(main())

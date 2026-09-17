#!/usr/bin/env bash
# O CICLO DE DEPURAR PYTHON, provado com o debugpy real — quando ha' um.
#
# POR QUE ESTE SCRIPT EXISTE (2026-09-13, fatia 4 da cadeia Python do
# roadmaps/41). O debugpy nao e' um binario: e' um MODULO do interpretador do
# projeto, e o adaptador sobe como `python -m debugpy.adapter`. Nenhum teste de
# unidade prova que o `launch` que o core monta (`console: internalConsole`,
# `program`, `cwd`) e' o que o debugpy aceita, nem que o `initialized` chega
# na ordem que a sessao espera — so' o debugpy REAL prova isso, como o QEMU
# prova o `gdb -i dap` (verificar-embarcado.sh).
#
# O interpretador com debugpy vem de `$KINEIN_PYTHON_DEBUGPY` (um venv seu),
# ou do `python3` do PATH quando ele importa o modulo. Sem nenhum dos dois o
# ciclo fica dito como NAO PROVADO — nao reprova: instalar debugpy no sistema
# nao e' decisao deste gate. O que reprova e' o ciclo QUEBRAR onde ha' debugpy.
#
# Medido em 2026-09-13 com um venv temporario (debugpy 1.8.21, Python 3.14.7):
# breakpoint, locais, evaluate, saida do programa, exitCode 0, adaptador morto
# com a sessao — 1,4 s.
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

# A porta do MicroPython (40 §7.39) vem ANTES da guarda do debugpy: so'
# precisa do core e de um mpremote falso — prova sem placa e sem interpretador.
binario="target/debug/kinein-core"
if [ ! -x "$binario" ]; then
    echo "-> compilando o core para o ciclo"
    cargo build -q -p kinein-core
fi
echo "== a porta escolhida chega ao mpremote (run.start/run.script { device }) =="
python3 scripts/verificar_micropython_porta.py

echo "== ciclo de depurar Python (debugpy do interpretador do projeto) =="

interpretador="${KINEIN_PYTHON_DEBUGPY:-}"
if [ -z "$interpretador" ]; then
    if command -v python3 >/dev/null 2>&1 && python3 -I -c "import debugpy" >/dev/null 2>&1; then
        interpretador="$(command -v python3)"
    fi
fi
if [ -z "$interpretador" ]; then
    echo "  - debugpy: ausente no python3 do PATH e KINEIN_PYTHON_DEBUGPY nao aponta um"
    echo "    interpretador (nao reprova; o ciclo fica NAO PROVADO aqui)"
    exit 0
fi
if ! "$interpretador" -I -c "import debugpy" >/dev/null 2>&1; then
    echo "erro: $interpretador nao importa debugpy" >&2
    exit 1
fi

python3 scripts/verificar_python_debug.py "$interpretador"

#!/usr/bin/env bash
# O clangd do kit CROSS enxerga os cabecalhos de libstdc++ do GCC ARM.
#
# POR QUE ESTE SCRIPT EXISTE (2026-09-11). O `Toolchain::clangd_args` passa
# `--query-driver=<cross>` quando o compilador C/C++ do kit e' um cross. Sem
# isso, medido nesta maquina com clangd 22 e arm-none-eabi-g++ 15.2, um `.cpp`
# de bare metal fica com 4 erros: o clangd acha `<stdint.h>` sozinho mas NAO os
# cabecalhos C++ do GCC ARM (`<array>`, `<cstdint>`). Teste de unidade cobre a
# LOGICA de montar o argumento; este cobre o EFEITO dele no clangd de verdade —
# a mesma licao do probe.rs e do fd: fixture nao ve mudanca de ferramenta.
#
# Ferramenta ausente nao reprova; e se o clangd desta maquina achar tudo
# sozinho, o teste se declara NAO CONCLUSIVO em vez de passar em falso.
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

echo "== clangd no kit cross (cabecalhos de libstdc++ ARM) =="

python3 scripts/verificar_clangd_cross.py

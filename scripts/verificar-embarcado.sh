#!/usr/bin/env bash
# O CICLO DE EMBARCADO, provado no QEMU — sem placa.
#
# POR QUE ESTE SCRIPT EXISTE (2026-09-11). A frente F entregou o fio (a tela
# alcanca sonda, chip e depurador), e depois a ponte: o `gdb -i dap` como
# adaptador e um servidor (QEMU/OpenOCD) que a IDE sobe e mata com a sessao.
# Nada disso e' exercitado por teste de unidade — igual ao `probe.rs` de
# 2026-09-03 e ao `fd` de 2026-09-04, a verdade so' aparece contra as
# ferramentas REAIS. E a frente de embarcados seria a unica do projeto sem
# rede de gate se dependesse de placa fisica.
#
# O QEMU resolve isso: `lm3s6965evb -S -gdb tcp::PORTA` e' um Cortex-M3 parado
# esperando o GDB, e a fixture em scripts/fixtures/embarcado/ e' o menor
# firmware que da' um breakpoint e uma variavel para ler. A fixture e' NOSSA,
# nao do usuario (`roadmaps/35` §5.6: a IDE nao adivinha linker script).
#
# O que o ciclo prova, pelo core de verdade e por stdio:
#   setKit(gdb, remoteTarget, debugServer) -> setBreakpoints -> start sobe o
#   QEMU e faz attach -> continue para no breakpoint -> evaluate le contador
#   0 e depois 1 -> variaveis NAO sao registradores -> stop -> finished, e o
#   QEMU morreu com a sessao.
#
# Ferramenta AUSENTE (arm-none-eabi-gcc, qemu-system-arm, gdb) nao reprova: nem
# toda maquina tem o ambiente de embarcado, e o ciclo fica dito como NAO
# PROVADO. O que reprova e' o ciclo QUEBRAR onde as tres existem.
#
# Provado por mutacao (2026-09-11): a sessao sem `exec` no servidor deixa um
# QEMU orfao e o gate pega pelo pgrep final; o escopo preferido errado mostra
# registradores e a assercao de variaveis cai; o adaptador `launch` em vez de
# `attach` nunca para no breakpoint.
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

echo "== ciclo de embarcado no QEMU (gdb -i dap, sem placa) =="

binario="target/debug/kinein-core"
if [ ! -x "$binario" ]; then
    echo "-> compilando o core para o ciclo"
    cargo build -q -p kinein-core
fi

python3 scripts/verificar_embarcado.py

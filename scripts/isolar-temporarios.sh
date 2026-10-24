#!/bin/sh
# Runner do cargo nos alvos unix (.cargo/config.toml): cada binario de TESTE
# roda com um TMPDIR proprio, apagado quando ele termina.
#
# POR QUE (2026-10-24, roadmaps/40.7 §7.357). Os testes criam pastas em
# `std::env::temp_dir()` com o pid no nome (`kinein-core-tests/<pid>-...`,
# `kinein-drift-<pid>-...`) e nenhum as apaga: sao 238 chamadas em 150
# arquivos. Num /tmp tmpfs isso e' RAM: medido 2,3 GB de restos de testes,
# 27 mil pastas so' no kinein-core-tests, que o kernel empurrou para o swap
# e que ajudaram a travar a maquina com o swap cheio. Apagar no fim de cada
# binario resolve todos de uma vez, sem tocar nos testes, e vale para o
# `cargo test` direto tanto quanto para o gate.
#
# So' o que mora em target/<perfil>/deps/ e' teste; o `cargo run` de um
# binario passa direto, com o TMPDIR de quem chamou.
# KINEIN_KEEP_TEST_TMP=1 desliga o isolamento para inspecionar o que um teste
# deixou (as pastas voltam a ficar no TMPDIR de quem chamou).
set -u

case "$1" in
    */deps/*) ;;
    *) exec "$@" ;;
esac
if [ -n "${KINEIN_KEEP_TEST_TMP:-}" ]; then
    exec "$@"
fi

dir="$(mktemp -d "${TMPDIR:-/tmp}/kinein-teste.XXXXXX")" || exec "$@"
trap 'rm -rf "$dir"' EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
TMPDIR="$dir" "$@"

#!/usr/bin/env bash
# cargo-deny: licencas, advisories, bans e origens das dependencias Rust.
#
# POR QUE ESTE SCRIPT EXISTE (2026-08-30). O `deny.toml` existia desde cedo, com
# uma allowlist estreita e correta de licencas — e NADA o executava. A unica
# politica de licenca do projeto era uma regra que morava num arquivo que
# ninguem rodava: a mesma forma da regra de split antes da catraca (§4 regra 10)
# e do shellcheck antes de 2026-08-29.
#
# O que a primeira execucao encontrou, e nenhum outro gate pegaria:
#
#   3 licencas rejeitadas  ISC e CC0-1.0, vindas do `notify` (ADR-0001).
#                          Ambas permissivas e FSF Free — a allowlist e' que
#                          estava estreita demais. Ampliada COM justificativa.
#   1 advisory             `serial` v0.4.0, ultima release em 2017, chegando
#                          por `portable-pty` 0.8.1 no caminho do TERMINAL.
#                          Resolvido subindo para portable-pty 0.9, que trocou
#                          `serial` por `serial2`.
#   2 wildcards            deps de path do proprio workspace. O cargo-deny so
#                          as aceita em crate NAO publicado — daí `publish =
#                          false`, que alem de destravar o lint e' a verdade.
#
# Um projeto que se propoe 100% open source precisa VERIFICAR isso, nao afirmar.
set -euo pipefail

cd "$(dirname "$0")/.." || exit 1

echo "== cargo-deny (licencas, advisories, bans, origens) =="

if ! command -v cargo-deny >/dev/null 2>&1; then
    echo "erro: cargo-deny nao encontrado." >&2
    echo "      Instale com:  cargo install cargo-deny --locked" >&2
    echo "      (ou rode scripts/instalar-ambiente.sh --extras)" >&2
    exit 1
fi

# `check` roda os quatro: advisories, bans, licenses, sources.
cargo deny check

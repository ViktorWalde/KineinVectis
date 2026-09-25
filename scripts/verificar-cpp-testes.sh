#!/usr/bin/env bash
# TESTES C++ da UI, pelo CTest.
#
# POR QUE ESTE SCRIPT EXISTE (2026-09-24). O repositorio media Rust (`cargo
# test`) e QML (harnesses headless), e o C++ da ponte tinha TRES coberturas —
# clang-format, clang-tidy e a catraca de fiacao IPC — mais o smoke de "o binario
# abre". Nenhuma das quatro olha o que uma FUNCAO decide: `clang-tidy` acha
# padrao suspeito, a fiacao acha elo sem dono, e "abre" nao passa da primeira
# tela.
#
# Nao havia decisao registrada justificando a ausencia; era omissao. A ironia
# ficou anotada no proprio repo: o `verificar-exercitacao.sh` roda o `ctest` REAL
# contra este projeto para exercitar o `test.discover` da IDE, e observava "zero
# testes declarados, sem erro".
#
# O primeiro assunto coberto e' o contrato da linha de comando (`ui/src/cli_args`),
# porque ele e' puro e porque o comportamento antigo — varrer argv e ignorar em
# silencio o que nao fosse pasta existente — e' exatamente o tipo de defeito que
# nenhum dos quatro gates anteriores pegaria.
#
# So' entra na biblioteca `kinein-ui-puro` o que e' testavel sem janela. As 34
# fontes do modulo QML ficam de fora por ora: seis cabecalhos registram tipo QML
# (`QML_ELEMENT`, inclusive o `CoreClient`), e tira-los do `qt_add_qml_module`
# quebraria o registro. A fronteira cresce por unidade.
#
# Uso: bash scripts/verificar-cpp-testes.sh [--preset <nome>]
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

PRESET=""
if [ "${1:-}" = "--preset" ]; then
    PRESET="${2:?--preset exige um nome}"
fi

CANDIDATOS="
build/linux-clang-debug-strict
build/dev-local
build/linux-clang-release-hardened
build/dev-local-release
"
BUILD_DIR=""
if [ -n "$PRESET" ]; then
    # O preset de BUILD e o de CONFIGURE tem nomes diferentes neste repo; aceitar
    # os dois evita a pegadinha que o contribuindo/02 descreve.
    for prefixo in "build/$PRESET" "build/linux-clang-$PRESET"; do
        if [ -f "$prefixo/CMakeCache.txt" ]; then
            BUILD_DIR="$prefixo"
            break
        fi
    done
else
    for candidato in $CANDIDATOS; do
        if [ -f "$candidato/CMakeCache.txt" ]; then
            BUILD_DIR="$candidato"
            break
        fi
    done
fi

if [ -z "$BUILD_DIR" ]; then
    echo "erro: nenhum build configurado. Rode: cmake --preset dev-local" >&2
    exit 1
fi

echo "== testes C++ da UI (ctest em $BUILD_DIR) =="

# Um alvo de teste que nao compila e' reprovacao, nao "nenhum teste".
#
# O alvo e' o AGREGADO `kinein-cpp-tests`, e nao um nome de teste: ate'
# 2026-09-25 este script compilava `tst_cli_args` pelo nome, entao o segundo
# teste do projeto simplesmente nao era construido — o CTest o reportava como
# "Not Run", acusando um binario ausente em vez do gate incompleto.
if ! cmake --build "$BUILD_DIR" --target kinein-cpp-tests >/dev/null; then
    echo "erro: os testes C++ nao compilaram." >&2
    exit 1
fi

# `ctest` com zero testes sai 0 e nao diz nada: seria um gate que passa vazio.
# Contar antes e exigir pelo menos um e' o que impede isso.
quantos="$(ctest --test-dir "$BUILD_DIR" -N 2>/dev/null | sed -n 's/^Total Tests: *//p')"
if [ -z "$quantos" ] || [ "$quantos" -lt 1 ]; then
    echo "erro: o CTest nao declarou nenhum teste C++ — o gate passaria vazio." >&2
    exit 1
fi

ctest --test-dir "$BUILD_DIR" --output-on-failure
echo "testes C++: $quantos declarado(s), todos verdes."

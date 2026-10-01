#!/bin/sh
# Verificacao rigorosa do C++ da UI: clang-format + clang-tidy.
# Usa o compile_commands.json do preset `linux-clang-debug-strict`.
#
# POR QUE ESTE PRESET, E NAO O `dev-local` (2026-10-01). O clang-tidy e' o
# frontend do clang: ele le' as flags de cada arquivo no compile_commands.json
# e as interpreta como clang. O `dev-local` que o instalar-ambiente.sh gera usa
# `c++` — o GCC no Arch e no Debian — e o KineinStrictOptions.cmake so' liga a
# opcao que o compilador aceita. Resultado medido: com o banco do GCC o
# clang-tidy reprova em `-Wlogical-op`, `-Wuseless-cast`, `-Wtrampolines`
# ("unknown warning option") e, mesmo silenciando isso, perderia as
# diagnosticas que so' o clang tem (`-Wcomma`, `-Wheader-hygiene`,
# `-Wloop-analysis`). O banco precisa ser de clang.
#
# POR QUE ELE SE CONFIGURA SOZINHO. Ate' esta data o gate exigia
# `build/linux-clang-debug-strict` ja' configurado. O instalar-ambiente.sh o
# configura, mas a documentacao manual (comandos-de-build-e-verificacao.md,
# 14-ambiente-de-desenvolvimento.md, como-executar.md) manda configurar so' os
# `dev-local*`, e a mensagem de erro daqui mandava configurar o `dev-local` —
# que nao cria esse diretorio. Quem seguia a documentacao reprovava sem defeito
# nenhum; numa maquina antiga o gate lia um banco de dias atras (arquivo .cpp
# novo fora dele, flag removida ainda nele).
# Configurar e' PREPARO, como a copia do QML no verificar-qml.sh: o gate o faz
# quando o banco falta ou e' mais velho que qualquer entrada do CMake, e DIZ que
# fez. Nao precisa compilar: o clang-tidy so' le' o banco e os fontes.

set -eu

REPO_ROOT="$(unset CDPATH; cd -- "$(dirname -- "$0")/.." && pwd)"
PRESET="linux-clang-debug-strict"
BUILD_DIR="$REPO_ROOT/build/$PRESET"
DATABASE="$BUILD_DIR/compile_commands.json"

for tool in clang++ clang-format clang-tidy; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "erro: $tool ausente; o gate C++ e' obrigatorio (scripts/instalar-ambiente.sh instala)." >&2
        exit 1
    fi
done

reason=""
if [ ! -f "$DATABASE" ]; then
    reason="banco de compilacao ausente"
else
    # Qualquer entrada do CMake mais nova que o banco o torna suspeito.
    newer="$(find "$REPO_ROOT" \
        \( -path "$REPO_ROOT/build" -o -path "$REPO_ROOT/target" -o -path "$REPO_ROOT/.git" \
           -o -path "$REPO_ROOT/KV0.3" -o -path "$REPO_ROOT/DocsPublic" -o -path "$REPO_ROOT/dist" \) -prune \
        -o \( -name CMakeLists.txt -o -name '*.cmake' -o -name CMakePresets.json -o -name CMakeUserPresets.json \) \
        -newer "$DATABASE" -print | head -n 1)"
    if [ -n "$newer" ]; then
        reason="banco mais velho que ${newer#"$REPO_ROOT"/}"
    fi
fi
if [ -n "$reason" ]; then
    echo "preparo: configurando o preset $PRESET ($reason)"
    log="$BUILD_DIR.configure.log"
    mkdir -p "$REPO_ROOT/build"
    if ! cmake --preset "$PRESET" >"$log" 2>&1; then
        tail -n 30 "$log" >&2
        echo "erro: cmake --preset $PRESET falhou (log completo: $log)" >&2
        exit 1
    fi
fi

# O QUE O MOC GERA TAMBEM E' FONTE (2026-10-01). Um `.cpp` com
# `#include "x.moc"` (o typing_perf_harness.cpp) so' compila depois do AUTOMOC,
# e o clang-tidy le' o mesmo banco que o compilador: numa arvore configurada e
# nunca compilada — exatamente a que o preparo acima acabou de criar — ele
# reprovava com "'typing_perf_harness.moc' file not found", sem defeito nenhum
# no codigo. Medido nesta data. Os alvos `*_autogen` geram so' isso, e nao
# fazem nada quando ja' estao em dia.
autogen_targets="$(cmake --build "$BUILD_DIR" --target help 2>/dev/null \
    | grep -oE '[A-Za-z0-9_.+-]+_autogen' | sort -u | tr '\n' ' ')"
if [ -n "$autogen_targets" ]; then
    # shellcheck disable=SC2086 # lista de alvos, separada por espaco de proposito
    if ! cmake --build "$BUILD_DIR" --target $autogen_targets >"$BUILD_DIR.autogen.log" 2>&1; then
        tail -n 30 "$BUILD_DIR.autogen.log" >&2
        echo "erro: o AUTOMOC de $PRESET falhou (log: $BUILD_DIR.autogen.log)" >&2
        exit 1
    fi
fi

echo "== clang-format =="
clang-format --dry-run --Werror "$REPO_ROOT"/ui/src/*.cpp "$REPO_ROOT"/ui/src/*.h

echo "== clang-tidy =="
clang-tidy -p "$BUILD_DIR" "$REPO_ROOT"/ui/src/*.cpp

echo "C++ verificado: tudo limpo."

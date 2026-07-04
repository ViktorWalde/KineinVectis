#!/bin/sh
# Verificacao rigorosa do C++ da UI: clang-format + clang-tidy.
# Usa o compile_commands.json do build debug estrito.

set -eu

REPO_ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
BUILD_DIR="$REPO_ROOT/build/linux-clang-debug-strict"

if [ ! -f "$BUILD_DIR/compile_commands.json" ]; then
    echo "erro: configure primeiro com: cmake --preset dev-local (ou linux-clang-debug-strict)" >&2
    exit 1
fi

echo "== clang-format =="
clang-format --dry-run --Werror "$REPO_ROOT"/ui/src/*.cpp "$REPO_ROOT"/ui/src/*.h

echo "== clang-tidy =="
clang-tidy -p "$BUILD_DIR" "$REPO_ROOT"/ui/src/*.cpp

echo "C++ verificado: tudo limpo."

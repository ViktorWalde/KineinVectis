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

# ui/tests entra desde 2026-07-17, junto do primeiro teste de C++ do projeto.
# Teste sob regra mais frouxa que o codigo testado e' como gate que nunca
# reprova: passa por nao olhar. O `ui/tests/.clang-tidy` desliga UM check
# (slot do QTest nao pode ser static) e herda todo o resto.
echo "== clang-format =="
clang-format --dry-run --Werror \
    "$REPO_ROOT"/ui/src/*.cpp "$REPO_ROOT"/ui/src/*.h \
    "$REPO_ROOT"/ui/tests/*.cpp

echo "== clang-tidy =="
clang-tidy -p "$BUILD_DIR" "$REPO_ROOT"/ui/src/*.cpp "$REPO_ROOT"/ui/tests/*.cpp

echo "C++ verificado: tudo limpo."

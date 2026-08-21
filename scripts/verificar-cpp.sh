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
# UMA invocacao, o .clang-tidy da raiz INTACTO, e nenhum arquivo isento.
#
# Ate 2026-08-21 este gate rodava duas vezes, com um check desligado para o
# typing_perf_harness.cpp: o clang-analyzer do Qt 6.8 acusava vazamento dentro
# do proprio qobjectdefs.h, na sobrecarga por functor do
# QMetaObject::invokeMethod. Aquilo era tapa-buraco — desligava um check real
# num arquivo real. A causa foi resolvida no CODIGO (o harness deixou de usar
# aquela sobrecarga; ver o comentario da conexao Queued la), entao a excecao
# saiu daqui em vez de ser maquiada. Gate com excecao e gate que ensina a
# proxima excecao.
#
# PARALELO por arquivo: cada unidade de traducao e independente, e a analise
# e o trecho mais caro do gate — em serie passava de dez minutos, o que pesa
# no loop de quem roda o gate a cada fatia. `xargs` devolve nao-zero se
# QUALQUER invocacao falhar, e o `set -e` do topo pega.
NUCLEOS="$(nproc 2>/dev/null || echo 4)"

find "$REPO_ROOT/ui/src" "$REPO_ROOT/ui/tests" -name '*.cpp' -print0 |
    xargs -0 -P "$NUCLEOS" -n 1 clang-tidy -p "$BUILD_DIR"

echo "C++ verificado: tudo limpo."

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
# typing_perf_harness.cpp sai da varredura geral e volta logo abaixo com UM
# check a menos. Motivo, medido em 2026-08-21 (Debian 13 / Qt 6.8.2):
#
#   qobjectdefs.h:624: Potential leak of memory pointed to by 'callable'
#     [clang-analyzer-cplusplus.NewDeleteLeaks]
#
# E falso positivo do analisador dentro do PROPRIO header do Qt: ele nao ve
# que `invokeMethodImpl` assume a posse do callable que a sobrecarga por
# functor aloca. O gate era verde no Qt 6.4 do Debian 12; quem mudou foi o
# ambiente, nao o codigo — o arquivo esta identico desde 2026-07-16.
#
# NOLINT nao resolve: o diagnostico e emitido na linha do header do Qt, e
# clang-tidy so honra NOLINT na linha do diagnostico. Reescrever a chamada
# exigiria Q_OBJECT + moc numa classe de namespace anonimo — plumbing caro
# para contornar bug de ferramenta em codigo de INSTRUMENTACAO, que so roda
# sob KINEIN_PERF_TYPING.
#
# O escopo e o mais estreito que a ferramenta permite: um check, um arquivo.
# Todo o resto do .clang-tidy continua valendo para ele. Ao subir de Qt,
# remova as duas invocacoes separadas e volte para a linha unica — se o
# falso positivo tiver sumido, o gate acusa nada e a divida morre sozinha.
HARNESS="$REPO_ROOT/ui/src/typing_perf_harness.cpp"

clang-tidy -p "$BUILD_DIR" \
    $(find "$REPO_ROOT/ui/src" -name '*.cpp' ! -name 'typing_perf_harness.cpp') \
    "$REPO_ROOT"/ui/tests/*.cpp

clang-tidy -p "$BUILD_DIR" \
    --checks=-clang-analyzer-cplusplus.NewDeleteLeaks \
    "$HARNESS"

echo "C++ verificado: tudo limpo."

#!/bin/sh
# Verificacao rigorosa do QML da UI: qmllint estrito (zero warnings).
#
# Usa o response file gerado pelo qt_add_qml_module no build debug local, que
# carrega import paths, qmldir e resources do modulo KineinVectis — o mesmo
# contexto do alvo `all_qmllint` do CMake, mas com `-W 0` para falhar em
# qualquer warning. Se um .qml novo nao aparecer no lint, reconfigure o build
# debug (o .rsp e regenerado na configuracao do CMake).

set -eu

REPO_ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"

RSP="${KINEIN_QML_RSP:-}"
if [ -z "$RSP" ]; then
    for candidato in \
        "$REPO_ROOT/build/linux-clang-debug-strict/ui/.rcc/qmllint/kinein-vectis.rsp" \
        "$REPO_ROOT/build/dev-local/ui/.rcc/qmllint/kinein-vectis.rsp"; do
        if [ -f "$candidato" ]; then
            RSP="$candidato"
            break
        fi
    done
fi
if [ -z "$RSP" ] || [ ! -f "$RSP" ]; then
    echo "erro: response file do qmllint nao encontrado." >&2
    echo "configure primeiro com: cmake --preset dev-local (ou linux-clang-debug-strict)" >&2
    exit 1
fi

QMLLINT="${KINEIN_QMLLINT:-}"
if [ -z "$QMLLINT" ]; then
    if command -v qmllint >/dev/null 2>&1; then
        QMLLINT="qmllint"
    elif command -v qmllint-qt6 >/dev/null 2>&1; then
        QMLLINT="qmllint-qt6"
    elif [ -x /usr/lib/qt6/bin/qmllint ]; then
        QMLLINT="/usr/lib/qt6/bin/qmllint"
    else
        echo "erro: qmllint nao encontrado (instale qt6-declarative)" >&2
        exit 1
    fi
fi

echo "== qmllint (estrito: zero warnings) =="
"$QMLLINT" -W 0 @"$RSP"
echo "QML verificado: tudo limpo."

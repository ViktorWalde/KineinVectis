#!/bin/sh
# Verificacao rigorosa do QML da UI: qmllint estrito (zero warnings).
#
# Usa o response file gerado pelo qt_add_qml_module no build debug local, que
# carrega import paths, qmldir e resources do modulo KineinVectis — o mesmo
# contexto do alvo `all_qmllint` do CMake, mas com `-W 0` para falhar em
# qualquer warning. Se um .qml novo nao aparecer no lint, reconfigure o build
# debug (o .rsp e regenerado na configuracao do CMake).

set -eu

REPO_ROOT="$(unset CDPATH; cd -- "$(dirname -- "$0")/.." && pwd)"

# O `.rsp` nasce na CONFIGURACAO; o `.qmltypes` do modulo so na COMPILACAO. Um
# diretorio configurado e nao compilado tem o primeiro e nao o segundo, e o
# qmllint responde com uma parede de "QML types file does not exist" em vez de
# uma mensagem util — parece defeito no QML e e' build faltando (medido em
# 2026-08-29). Por isso o candidato so vale se os DOIS existirem.
RSP="${KINEIN_QML_RSP:-}"
configurado_sem_build=""
if [ -z "$RSP" ]; then
    for base in \
        "$REPO_ROOT/build/linux-clang-debug-strict" \
        "$REPO_ROOT/build/dev-local"; do
        candidato="$base/ui/.rcc/qmllint/kinein-vectis.rsp"
        [ -f "$candidato" ] || continue
        if [ -f "$base/ui/KineinVectis/kinein-vectis.qmltypes" ]; then
            RSP="$candidato"
            break
        fi
        configurado_sem_build="$base"
    done
fi
if [ -z "$RSP" ] || [ ! -f "$RSP" ]; then
    if [ -n "$configurado_sem_build" ]; then
        echo "erro: $configurado_sem_build esta configurado mas NAO compilado." >&2
        echo "      O qmllint precisa do kinein-vectis.qmltypes, que sai da" >&2
        echo "      compilacao. Rode:  cmake --build $configurado_sem_build" >&2
    else
        echo "erro: response file do qmllint nao encontrado." >&2
        echo "      Configure com: cmake --preset dev-local" >&2
        echo "      (ou rode scripts/instalar-ambiente.sh, que configura tudo)" >&2
    fi
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

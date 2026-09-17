#!/bin/sh
# Verificacao rigorosa do QML da UI: qmllint estrito (zero warnings).
#
# Usa o contexto gerado pelo qt_add_qml_module no build debug local, que
# carrega import paths, qmldir e resources do modulo KineinVectis — o mesmo
# contexto do alvo `all_qmllint` do CMake. Qt recente usa .rsp e -W 0;
# Qt 6.4 usa o alvo JSON gerado e reprova qualquer warning do relatorio.
# Se um .qml novo nao aparecer no lint, reconfigure o build debug.

set -eu

REPO_ROOT="$(unset CDPATH; cd -- "$(dirname -- "$0")/.." && pwd)"

# O `.qmltypes` nasce na COMPILACAO. Sem ele o lint reclama dos tipos por
# falta de build, nao por defeito no QML. Qt recente tambem gera .rsp na
# configuracao; Qt 6.4 usa o alvo CMake/JSON, com o mesmo contexto de imports.
RSP="${KINEIN_QML_RSP:-}"
configurado_sem_build=""
native_build=""
if [ -z "$RSP" ]; then
    for base in \
        "$REPO_ROOT/build/linux-clang-debug-strict" \
        "$REPO_ROOT/build/dev-local"; do
        candidato="$base/ui/.rcc/qmllint/kinein-vectis.rsp"
        [ -f "$base/CMakeCache.txt" ] || continue
        if [ ! -f "$base/ui/KineinVectis/kinein-vectis.qmltypes" ]; then
            configurado_sem_build="$base"
            continue
        fi
        if [ -f "$candidato" ]; then
            RSP="$candidato"
        else
            native_build="$base"
        fi
        break
    done
fi
if [ -n "$native_build" ]; then
    if [ -n "${KINEIN_QMLLINT:-}" ]; then
        echo "erro: KINEIN_QMLLINT exige KINEIN_QML_RSP neste build sem .rsp." >&2
        echo "      O alvo CMake usa o qmllint da mesma instalacao Qt do build." >&2
        exit 1
    fi
    exec python3 "$REPO_ROOT/scripts/verificar_qml.py" "$native_build"
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

# O binario do Qt 6 PRIMEIRO, e cada candidato tem de RESPONDER `--version`:
# no Ubuntu 26.04 o `qmllint` do PATH e' o wrapper do qtchooser apontando para
# um Qt 5 que nao existe ("could not exec /usr/lib/qt5/bin/qmllint") — a mesma
# licao do `qml` no verificar-qml-logica.sh: estar no PATH nao e' prova de
# ferramenta funcional (medido em 2026-09-17).
QMLLINT="${KINEIN_QMLLINT:-}"
if [ -z "$QMLLINT" ]; then
    for candidato in /usr/lib/qt6/bin/qmllint qmllint-qt6 qmllint; do
        if "$candidato" --version >/dev/null 2>&1; then
            QMLLINT="$candidato"
            break
        fi
    done
    if [ -z "$QMLLINT" ]; then
        echo "erro: qmllint do Qt 6 nao encontrado (instale qt6-declarative-dev-tools)" >&2
        exit 1
    fi
fi

echo "== qmllint (estrito: zero warnings) =="
"$QMLLINT" -W 0 @"$RSP"
echo "QML verificado: tudo limpo."

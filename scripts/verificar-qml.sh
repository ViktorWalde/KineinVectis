#!/bin/sh
# Verificacao rigorosa do QML da UI: qmllint estrito (zero warnings).
#
# Usa o contexto gerado pelo qt_add_qml_module no build debug local, que
# carrega import paths, qmldir e resources do modulo KineinVectis — o mesmo
# contexto do alvo `all_qmllint` do CMake. Qt recente usa .rsp e -W 0;
# Qt 6.4 usa o alvo JSON gerado e reprova qualquer warning do relatorio.
#
# POR QUE ELE CONFERE A COPIA ANTES DE LINTAR (2026-09-24). O qmllint le' as
# copias do QML no diretorio de BUILD, nao a arvore. O `verificar.sh` roda este
# gate ANTES do build que atualiza essas copias, e o comentario que estava aqui
# apenas PEDIA que voce se lembrasse de reconfigurar. Isso reprova a toa quando
# uma propriedade nova ainda nao foi copiada — e, muito pior, PASSA lintando QML
# velho, que e' o gate mentindo que esta' verde. Medido nesta data: a fatia
# `remote.discover` reprovou com "Could not find property" em propriedades que
# existiam na arvore. Agora o gate RECUSA copia ausente ou mais velha que a
# fonte.
#
# A copia e' CACHE da fonte, nao artefato sob teste: atualiza-la e' preparo, e
# por isso este gate a refaz sozinho e DIZ que refez. O que ele recusa e' o
# defeito de verdade — um .qml na arvore que nao esta' no modulo QML, logo nao
# e' compilado nem lintado por ninguem.

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

# A copia que o lint vai ler tem de ser a arvore de agora. Arquivo com
# QT_RESOURCE_ALIAS e' achatado em KineinVectis/<nome>; sem alias, mantem o
# caminho relativo a ui/. Conferir os dois evita falso negativo E falso
# positivo.
#
# Compara CONTEUDO, nao mtime: o CMake copia so' o que difere, entao uma copia
# com bytes iguais e mtime mais velho e' correta — reprovar nela seria ruido.
# O .rsp vive em <raiz do build>/ui/.rcc/qmllint/: tres dirname dao a pasta
# `ui` (onde mora o modulo KineinVectis) e o quarto da' a raiz do build, que e'
# o que o `cmake --build` quer.
BUILD_DIR="$(dirname -- "$(dirname -- "$(dirname -- "$RSP")")")"
BUILD_ROOT="$(dirname -- "$BUILD_DIR")"
if command -v cmake >/dev/null 2>&1; then
    cmake --build "$BUILD_ROOT" --target kinein-vectis_copy_qml >/dev/null 2>&1 \
        || echo "aviso: nao pude atualizar a copia do QML; a conferencia abaixo decide." >&2
fi
REPO_ROOT="$REPO_ROOT" BUILD_DIR="$BUILD_DIR" python3 <<'PYCHECK'
import os
import pathlib
import sys

# DEBITO DECLARADO (medido em 2026-09-24). Este .qml esta' na arvore e no git,
# NAO esta' no ui/CMakeLists.txt, logo nao e' compilado nem lintado, e NINGUEM
# o referencia — conferido com grep no repo inteiro. Ou ele entra no modulo, ou
# sai da arvore: as duas coisas sao decisao do autor, nao deste gate. Enquanto
# isso, fica DITO aqui. Arquivo NOVO sem copia reprova.
SEM_COPIA_DECLARADO = {"ui/qml/editor/EditorUnsavedChangesDialog.qml"}

repo = pathlib.Path(os.environ["REPO_ROOT"])
modulo = pathlib.Path(os.environ["BUILD_DIR"]) / "KineinVectis"
ausentes, diferentes, declarados = [], [], []
for fonte in sorted((repo / "ui/qml").rglob("*.qml")):
    relativo = str(fonte.relative_to(repo))
    candidatos = [modulo / fonte.name, modulo / fonte.relative_to(repo / "ui")]
    copia = next((c for c in candidatos if c.is_file()), None)
    if copia is None:
        (declarados if relativo in SEM_COPIA_DECLARADO else ausentes).append(relativo)
    elif copia.read_bytes() != fonte.read_bytes():
        diferentes.append(relativo)

if ausentes or diferentes:
    print("erro: o qmllint leria QML DESATUALIZADO do diretorio de build.",
          file=sys.stderr)
    for f in diferentes:
        # Chegar aqui significa que o refresh automatico acima nao rodou ou
        # falhou: lintar assim passaria por engano sobre QML velho.
        print(f"  copia diverge da fonte: {f}", file=sys.stderr)
        print("      Atualize e repita:", file=sys.stderr)
        print("        cmake --build --preset debug-strict"
              " --target kinein-vectis_copy_qml", file=sys.stderr)
    for f in ausentes:
        print(f"  NAO esta' no modulo QML, logo nunca e' lintado: {f}",
              file=sys.stderr)
        print("      Registre em ui/CMakeLists.txt (qt_add_qml_module + alias)"
              " ou remova o arquivo.", file=sys.stderr)
    raise SystemExit(1)

if declarados:
    print(f"copia do QML: {len(declarados)} arquivo(s) fora do modulo, dito(s):"
          f" {', '.join(declarados)}")
PYCHECK

echo "== qmllint (estrito: zero warnings) =="
"$QMLLINT" -W 0 @"$RSP"
echo "QML verificado: tudo limpo."

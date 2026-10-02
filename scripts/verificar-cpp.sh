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
# UM FALSO POSITIVO CONHECIDO, PROVADO, E COM PRAZO (2026-10-01, decisao do
# autor: provar e decidir). O clang-analyzer 18 acusa
# `cplusplus.NewDelete` ("Use of memory after it is freed") DENTRO do
# `QWeakPointer` do Qt 6.4 (qsharedpointer_impl.h), a partir da atribuicao de
# um `QPointer` no window_chrome_controller.cpp. Provas, registradas no
# roadmaps/40.7 §7.152:
#   - reproduz num arquivo SO' com Qt (`QPointer<QObject> p; p = &obj;`), com
#     as flags do projeto: nao e' logica nossa;
#   - o caminho do analisador "assume" que a contagem atomica chegou a zero;
#   - o achado depende SO' de os headers do Qt serem de sistema: o mesmo
#     arquivo e o mesmo clang dao 1 achado com `-isystem` e 0 com `-I`. Essa
#     flag nao muda o que o programa faz com a memoria, so' como o analisador
#     trata o header; um use-after-free real apareceria nos dois;
#   - o mesmo padrao, 10.000 rodadas sob ASan/UBSan (trocar de alvo, destruir
#     o alvo, soltar), sem erro; e o mesmo ASan pega um use-after-free real.
# Supressao na NOSSA linha nao existe para isso: o NOLINT e o
# [[clang::suppress]] foram medidos e nao calam, porque o clang-tidy situa o
# diagnostico no header do Qt. Entao a excecao mora aqui, o mais estreita
# possivel: so' este arquivo, so' este check, so' este ponto do header, e o
# caminho tem de passar pela atribuicao do QPointer. Qualquer OUTRO achado no
# arquivo reprova. E ela EXPIRA: no dia em que o clang parar de acusar, o gate
# reprova pedindo para apagar esta excecao — a excecao nao sobrevive ao bug.
#
# So' nas versoes em que foi PROVADA (2026-10-01, a noite). O clang-tidy 21 nao
# acusa nada no mesmo caso minimo, nem com os headers do Qt 6.4.2 (copiados do
# container do AppImage) nem com os do 6.10: a variavel e' o analisador, nao o
# Qt. Sem esta condicao, o "expira" reprovava toda maquina com clang novo — a
# do autor inclusive —, confundindo "o bug foi corrigido" com "este ambiente
# nunca o teve". Em versao fora da lista o arquivo e' um arquivo comum: se o
# achado aparecer numa versao nao provada, reprova e pede a prova.
KNOWN_FALSE_POSITIVE_FILE="$REPO_ROOT/ui/src/window_chrome_controller.cpp"
KNOWN_FALSE_POSITIVE_TIDY_MAJORS="18"
tidy_major="$(clang-tidy --version | sed -n 's/.*LLVM version \([0-9][0-9]*\).*/\1/p' | head -n 1)"
known_applies=0
for major in $KNOWN_FALSE_POSITIVE_TIDY_MAJORS; do
    [ "$tidy_major" = "$major" ] && known_applies=1
done
regular_files=""
for source in "$REPO_ROOT"/ui/src/*.cpp; do
    [ "$known_applies" -eq 1 ] && [ "$source" = "$KNOWN_FALSE_POSITIVE_FILE" ] && continue
    regular_files="$regular_files $source"
done
# shellcheck disable=SC2086 # lista de arquivos, separada por espaco de proposito
clang-tidy -p "$BUILD_DIR" $regular_files

if [ "$known_applies" -eq 0 ]; then
    echo "clang-tidy ${tidy_major:-?}: sem excecao (o falso positivo do QPointer so' foi provado no clang-tidy $KNOWN_FALSE_POSITIVE_TIDY_MAJORS; roadmaps/40.7 §7.152)"
    echo "C++ verificado: tudo limpo."
    exit 0
fi

known_output="$(clang-tidy -p "$BUILD_DIR" "$KNOWN_FALSE_POSITIVE_FILE" 2>&1)" && known_status=0 || known_status=$?
findings="$(printf '%s\n' "$known_output" | grep -E ': (error|warning): ' || true)"
known_pattern='/QtCore/qsharedpointer_impl\.h:[0-9]+:[0-9]+: error: Use of memory after it is freed \[clang-analyzer-cplusplus\.NewDelete'
if [ "$known_status" -eq 0 ] && [ -z "$findings" ]; then
    echo "erro: o falso positivo conhecido do QPointer (NewDelete no QWeakPointer) SUMIU." >&2
    echo "      Bom sinal: apague a excecao deste script (bloco KNOWN_FALSE_POSITIVE_FILE)" >&2
    echo "      e o registro no roadmaps/40.7 §7.152 passa a ser historico." >&2
    exit 1
fi
if [ "$(printf '%s\n' "$findings" | grep -c .)" -ne 1 ] \
    || ! printf '%s\n' "$findings" | grep -Eq "$known_pattern" \
    || ! printf '%s\n' "$known_output" | grep -Eq "window_chrome_controller\.cpp:[0-9]+:[0-9]+: note: Calling 'QPointer::operator='"; then
    printf '%s\n' "$known_output" >&2
    echo "erro: achado do clang-tidy em window_chrome_controller.cpp alem do falso positivo conhecido." >&2
    exit 1
fi
echo "clang-tidy: 1 falso positivo conhecido e provado (NewDelete no QWeakPointer do Qt, via QPointer; roadmaps/40.7 §7.152)"

echo "C++ verificado: tudo limpo."

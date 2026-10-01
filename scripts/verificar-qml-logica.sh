#!/usr/bin/env bash
# Testes de LOGICA QML (headless, offscreen).
#
# Por que existe: o gate cobre Rust (fmt/clippy/testes), C++ (clang-format/
# clang-tidy) e qmllint — mas NADA executava a logica QML, que e justamente
# onde a IDE guarda o estado da UI. Foi por esse buraco que o bug do D1
# (autocomplete que nunca abria) sobreviveu a dois ciclos de "correcao":
# sonda verde no backend, GUI quebrada.
#
# Cada tst_*.qml carrega o controller REAL do projeto (nao uma copia) com
# bridges falsos e codifica as falhas no CODIGO DE SAIDA (bitmask). Um
# TIMEOUT tambem e falha — e como o teste de regex de largura zero pega o
# laco infinito no Find/Replace.
#
# Uso: bash scripts/verificar-qml-logica.sh
# Um único fixture: KINEIN_QML_TEST=tst_project_tree_drop bash scripts/verificar-qml-logica.sh
# Outro Qt: KINEIN_QML_RUNNER=/caminho/qml (o G0.3 usa o Qt 6.4 do AppImage).
set -uo pipefail

cd "$(dirname "$0")/.." || exit 1

qml_runner="${KINEIN_QML_RUNNER:-}"
if [ -z "$qml_runner" ]; then
    for candidate in qml6 qml-qt6 /usr/lib/qt6/bin/qml qml; do
        if command -v "$candidate" >/dev/null 2>&1; then
            qml_runner="$candidate"
            break
        fi
    done
fi

if [ -z "$qml_runner" ]; then
    echo "runner QML nao encontrado (pacote qt6-declarative)." >&2
    exit 1
fi

# ESPELHO PLANO DO MODULO KineinVectis (2026-09-04).
#
# Ate' aqui todo harness importava PASTA ("../../ui/qml/editor"), e por isso so
# dava para testar componente que nao usa o Theme. Componente VISUAL usa: ele
# faz `import KineinVectis`, e o modulo de verdade so' existe dentro do qrc do
# binario compilado. O resultado pratico era que geometria de tela nao tinha
# como ser testada — e foi por ai' que a caixa da previa do CMakeLists chegou a
# 10px numa tela 1366x768 sem nada acusar.
#
# O espelho monta o mesmo modulo a partir das FONTES: nomes de arquivo sao
# unicos no projeto (o qrc ja' exige isso, via QT_RESOURCE_ALIAS), entao o
# qmldir sai direto do `find`. Nao ha copia de codigo — sao os mesmos arquivos.
mirror="$(mktemp -d)"
run_log="$(mktemp)"
trap 'rm -rf "$mirror" "$run_log"' EXIT
mkdir -p "$mirror/KineinVectis"
printf 'module KineinVectis\n' > "$mirror/KineinVectis/qmldir"
while IFS= read -r source_file; do
    file_name="$(basename "$source_file")"
    cp "$source_file" "$mirror/KineinVectis/$file_name"
    type_name="${file_name%.qml}"
    if grep -q '^pragma Singleton' "$source_file"; then
        printf 'singleton %s 1.0 %s\n' "$type_name" "$file_name" >> "$mirror/KineinVectis/qmldir"
    else
        printf '%s 1.0 %s\n' "$type_name" "$file_name" >> "$mirror/KineinVectis/qmldir"
    fi
done < <(find ui/qml -name '*.qml')
# Os .js do modulo (KvIconGlyphs.js) viajam junto: um componente que os
# importa por caminho relativo (KvIcon) so' carrega se eles estiverem ao lado.
while IFS= read -r source_file; do
    cp "$source_file" "$mirror/KineinVectis/$(basename "$source_file")"
done < <(find ui/qml -name '*.js')

failed=0

for test_file in scripts/qml-harness/tst_*.qml; do
    test_name="$(basename "$test_file" .qml)"
    if [ -n "${KINEIN_QML_TEST:-}" ] && [ "$test_name" != "$KINEIN_QML_TEST" ]; then
        continue
    fi
    printf '== %s ==\n' "$test_name"
    # QT_ASSUME_STDERR_HAS_CONSOLE: sem isso o qml6 ENGOLE console.log/warn.
    # A saida passa por um arquivo para o grep dos avisos proibidos: um harness
    # verde que imprime "Unable to assign" ou "Component is not ready" esconde
    # exatamente o defeito que o usuario veria no terminal (roadmap 53 §0.1).
    timeout 60 env QT_QPA_PLATFORM=offscreen QT_ASSUME_STDERR_HAS_CONSOLE=1 \
        "$qml_runner" -I "$mirror" "$test_file" > "$run_log" 2>&1
    exit_code=$?
    cat "$run_log"
    if [ "$exit_code" -eq 0 ] && grep -F -q -f scripts/avisos-qml.txt "$run_log"; then
        echo "  ✗ AVISO QML PROIBIDO (lista em scripts/avisos-qml.txt):" >&2
        grep -F -f scripts/avisos-qml.txt "$run_log" | sed 's/^/    /' >&2
        failed=1
        continue
    fi
    if [ "$exit_code" -eq 0 ]; then
        echo "  ok"
    elif [ "$exit_code" -eq 124 ]; then
        echo "  ✗ TIMEOUT — provavel LACO INFINITO (ex.: regex de largura zero)" >&2
        failed=1
    else
        echo "  ✗ FALHOU (saida $exit_code; o bitmask das assercoes vem na linha FALHAS acima — $test_name.qml)" >&2
        failed=1
    fi
done

if [ "$failed" -ne 0 ]; then
    echo
    echo "✗ testes de logica QML FALHARAM" >&2
    exit 1
fi

echo
echo "✓ logica QML: tudo verde"

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
set -uo pipefail

cd "$(dirname "$0")/.."

# O runner PROPRIO vem primeiro. O binario `qml` da distro era a base ate o
# Debian 12; o Debian 13 parou de empacota-lo em qt6-declarative-dev-tools e
# este gate — a unica rede que EXECUTA logica de UI — deixou de rodar sem
# avisar. Gate que depende de ferramenta que a distro tira e poe some
# justamente quando o ambiente muda, que e quando ele seria mais necessario.
# O ui/tests/qml_harness_runner.cpp faz o minimo equivalente e vive no repo.
qml_runner="${KINEIN_QML_RUNNER:-}"
if [ -z "$qml_runner" ]; then
    for candidate in \
        build/linux-clang-debug-strict/ui/kinein-qml-harness \
        build/dev-local/ui/kinein-qml-harness; do
        if [ -x "$candidate" ]; then
            qml_runner="$candidate"
            break
        fi
    done
fi
if [ -z "$qml_runner" ]; then
    for candidate in qml6 qml-qt6 qml; do
        if command -v "$candidate" >/dev/null 2>&1; then
            qml_runner="$candidate"
            break
        fi
    done
fi

if [ -z "$qml_runner" ]; then
    echo "runner QML nao encontrado." >&2
    echo "compile o alvo do repo: cmake --build build/linux-clang-debug-strict \\" >&2
    echo "    --target kinein-qml-harness" >&2
    exit 1
fi

falhou=0

for teste in scripts/qml-harness/tst_*.qml; do
    nome="$(basename "$teste" .qml)"
    printf '== %s ==\n' "$nome"
    # QT_ASSUME_STDERR_HAS_CONSOLE: sem isso o qml6 ENGOLE console.log/warn.
    timeout 60 env QT_QPA_PLATFORM=offscreen QT_ASSUME_STDERR_HAS_CONSOLE=1 \
        "$qml_runner" "$teste"
    codigo=$?
    if [ "$codigo" -eq 0 ]; then
        echo "  ok"
    elif [ "$codigo" -eq 124 ]; then
        echo "  ✗ TIMEOUT — provavel LACO INFINITO (ex.: regex de largura zero)" >&2
        falhou=1
    else
        echo "  ✗ FALHOU (bitmask=$codigo — ver as assercoes no $nome.qml)" >&2
        falhou=1
    fi
done

if [ "$falhou" -ne 0 ]; then
    echo
    echo "✗ testes de logica QML FALHARAM" >&2
    exit 1
fi

echo
echo "✓ logica QML: tudo verde"

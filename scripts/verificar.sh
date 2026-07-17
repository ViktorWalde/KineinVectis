#!/usr/bin/env bash
# Gate unico de verificacao do Kinein Vectis.
#
# Roda toda a validacao em sequencia e PARA no primeiro erro (set -e), para
# evitar o caso em que um passo falha mas os seguintes continuam e dao falsa
# sensacao de "tudo passou". Ver docs/arquitetura/15-engineering-debt-and-refactor.md.
#
# Uso:
#   scripts/verificar.sh            # completo: lint + testes + C++ + builds debug/release
#   scripts/verificar.sh --rapido   # rapido:   lint + testes + C++ (sem builds)
#
# Antes de rodar, formate o codigo:  cargo fmt --all
# O gate apenas CHECA a formatacao (cargo fmt --check); ele nao altera arquivos.
#
# Presets CMake podem ser sobrescritos por ambiente (padrao: dev-local*):
#   KINEIN_PRESET_DEBUG   (padrao: dev-local)
#   KINEIN_PRESET_RELEASE (padrao: dev-local-release)
set -euo pipefail

modo="completo"
case "${1:-}" in
    --rapido | --rapida | -r) modo="rapido" ;;
    --completo | -c | "") modo="completo" ;;
    *)
        echo "uso: $0 [--rapido|--completo]" >&2
        exit 2
        ;;
esac

raiz="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$raiz"

preset_debug="${KINEIN_PRESET_DEBUG:-dev-local}"
preset_release="${KINEIN_PRESET_RELEASE:-dev-local-release}"

etapa=""
trap 'estado=$?; if [ "$estado" -ne 0 ]; then
    echo ""
    echo "✗ FALHOU em: ${etapa:-inicializacao} (exit $estado)"
fi' EXIT

passo() {
    etapa="$1"
    echo ""
    echo "== $1 =="
}

passo "cargo fmt --all --check"
cargo fmt --all --check

passo "cargo test --workspace --all-features"
cargo test --workspace --all-features

passo "cargo clippy --workspace --all-targets --all-features -- -D warnings"
cargo clippy --workspace --all-targets --all-features -- -D warnings

passo "scripts/verificar-cpp.sh (clang-format + clang-tidy)"
scripts/verificar-cpp.sh

passo "scripts/verificar-qml.sh (qmllint estrito)"
scripts/verificar-qml.sh

passo "scripts/verificar-qml-fiacao.sh (binding auto-referente)"
bash scripts/verificar-qml-fiacao.sh

passo "scripts/verificar-docs.sh (numero sem data que mente)"
bash scripts/verificar-docs.sh

passo "scripts/verificar-presets.sh (preset que sobrescreve o binario do atalho)"
bash scripts/verificar-presets.sh

passo "scripts/verificar-icone.sh (icone com fonte unica)"
bash scripts/verificar-icone.sh

passo "scripts/verificar-arquitetura.sh (catraca da regra de split)"
bash scripts/verificar-arquitetura.sh

passo "scripts/verificar-qml-logica.sh (controllers QML headless)"
scripts/verificar-qml-logica.sh

if [ "$modo" = "completo" ]; then
    passo "cmake --build --preset $preset_debug (UI debug sanitized)"
    cmake --build --preset "$preset_debug"

    passo "cargo build --release -p kinein-core"
    cargo build --release -p kinein-core

    passo "cmake --build --preset $preset_release (UI release)"
    cmake --build --preset "$preset_release"

    # Testes de C++ da UI (desde 2026-07-17). Rodam no preset DEBUG porque e' o
    # que tem sanitizers: teste que passa sem ASan/UBSan e' teste que nao viu
    # metade do que podia pegar.
    #
    # Via `--target test`, NAO `ctest --test-dir build/$preset_debug`: o nome do
    # preset nao e' o nome do diretorio. `debug-strict` e' preset de BUILD e
    # configura em `build/linux-clang-debug-strict`; `build/debug-strict` nao
    # existe. Deixar o CMake resolver o diretorio elimina a classe inteira de
    # erro — a mesma que fez `dev-local-release` gravar por cima do atalho.
    passo "ctest (testes C++ da UI, preset $preset_debug)"
    CTEST_OUTPUT_ON_FAILURE=1 cmake --build --preset "$preset_debug" --target test
fi

etapa=""
echo ""
if [ "$modo" = "completo" ]; then
    echo "✓ TUDO VERDE (completo — binarios release do icone atualizados)"
else
    echo "✓ TUDO VERDE (rapido — sem builds; rode --completo antes de release)"
fi

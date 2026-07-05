#!/usr/bin/env bash
# Gate unico de verificacao do Kinein Vectis.
#
# Roda toda a validacao em sequencia e PARA no primeiro erro (set -e), para
# evitar o caso em que um passo falha mas os seguintes continuam e dao falsa
# sensacao de "tudo passou". Ver docs/15-engineering-debt-and-refactor.md.
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

if [ "$modo" = "completo" ]; then
    passo "cmake --build --preset $preset_debug (UI debug sanitized)"
    cmake --build --preset "$preset_debug"

    passo "cargo build --release -p kinein-core"
    cargo build --release -p kinein-core

    passo "cmake --build --preset $preset_release (UI release)"
    cmake --build --preset "$preset_release"
fi

etapa=""
echo ""
if [ "$modo" = "completo" ]; then
    echo "✓ TUDO VERDE (completo — binarios release do icone atualizados)"
else
    echo "✓ TUDO VERDE (rapido — sem builds; rode --completo antes de release)"
fi

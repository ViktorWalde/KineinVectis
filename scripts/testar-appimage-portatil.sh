#!/usr/bin/env bash
# Runs the AppImage smoke test in a clean Debian runtime without Qt/Rust SDKs.

set -Eeuo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
IMAGE="localhost/kinein-vectis-appimage-smoke:bookworm"

if ! command -v podman >/dev/null 2>&1; then
    echo "erro: Podman é obrigatório para o smoke portátil." >&2
    exit 1
fi

echo "==> construindo runtime mínimo de validação"
podman build \
    --file "$REPO_ROOT/packaging/appimage/Containerfile.smoke" \
    --tag "$IMAGE" \
    "$REPO_ROOT"

echo "==> testando sem rede, Qt/Rust SDKs ou compiladores no host convidado"
podman run --rm --network none \
    --volume "$REPO_ROOT:/workspace:ro" \
    --workdir /workspace \
    "$IMAGE" \
    bash scripts/testar-appimage.sh

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
# Fedora/SELinux exige rótulo também no mount somente-leitura do smoke.
# Mesma regra do empacotador: uma saida fora do dist/ precisa estar no
# repositorio montado, e chega ao container pelo caminho de dentro.
env_args=()
if [[ -n "${KINEIN_APPIMAGE_DIST_DIR:-}" ]]; then
    dist_dir="$(realpath -m "$KINEIN_APPIMAGE_DIST_DIR")"
    if [[ "$dist_dir" != "$REPO_ROOT"/* ]]; then
        echo "erro: KINEIN_APPIMAGE_DIST_DIR precisa estar dentro de $REPO_ROOT" >&2
        exit 1
    fi
    env_args+=(--env "KINEIN_APPIMAGE_DIST_DIR=/workspace/${dist_dir#"$REPO_ROOT"/}")
fi

podman run --rm --network none \
    "${env_args[@]}" \
    --volume "$REPO_ROOT:/workspace:ro,Z" \
    --workdir /workspace \
    "$IMAGE" \
    bash scripts/testar-appimage.sh

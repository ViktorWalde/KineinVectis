#!/usr/bin/env bash
# Builds the x86_64 AppImage in the pinned Debian 12 container baseline.

set -Eeuo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BUILDER_FILE="$REPO_ROOT/packaging/appimage/Containerfile"
BUILDER_IMAGE="${KINEIN_APPIMAGE_BUILDER_IMAGE:-localhost/kinein-vectis-appimage-builder:bookworm}"

if [[ -n "${KINEIN_CONTAINER_ENGINE:-}" ]]; then
    CONTAINER_ENGINE="$KINEIN_CONTAINER_ENGINE"
elif command -v podman >/dev/null 2>&1; then
    CONTAINER_ENGINE="podman"
elif command -v docker >/dev/null 2>&1; then
    CONTAINER_ENGINE="docker"
else
    echo "erro: instale Podman ou Docker para o build portátil do AppImage." >&2
    exit 1
fi

echo "==> construindo builder AppImage fixado (Debian 12 / Rust 1.96.1)"
"$CONTAINER_ENGINE" build --file "$BUILDER_FILE" --tag "$BUILDER_IMAGE" "$REPO_ROOT"

RUN_ARGS=(
    --rm
    --env HOME=/workspace/build/appimage/cache/home
    --env CARGO_HOME=/workspace/build/appimage/cache/cargo
    --env KINEIN_APPIMAGE_TOOLS_DIR=/workspace/build/appimage/cache/tools
    --volume "$REPO_ROOT:/workspace:rw"
    --workdir /workspace
)

if [[ "$CONTAINER_ENGINE" == "podman" ]]; then
    RUN_ARGS+=(--userns=keep-id)
else
    RUN_ARGS+=(--user "$(id -u):$(id -g)")
fi

echo "==> gerando AppImage no baseline portátil"
"$CONTAINER_ENGINE" run "${RUN_ARGS[@]}" "$BUILDER_IMAGE" \
    bash scripts/empacotar-appimage.sh

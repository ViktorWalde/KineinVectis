#!/usr/bin/env bash
# Builds the x86_64 AppImage in the pinned Debian 12 container baseline.
#
# Este script roda no host. Toda manipulação do AppDir e dos plugins Qt
# ocorre em scripts/empacotar-appimage.sh, dentro do container.

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

if [[ ! -f "$BUILDER_FILE" ]]; then
    echo "erro: Containerfile do builder não encontrado:" >&2
    echo "  $BUILDER_FILE" >&2
    exit 1
fi

echo "==> construindo builder AppImage fixado (Debian 12 / Rust 1.96.1 / Qt 6.12.0 oficial)"
echo "==> a primeira rodada baixa o SDK Qt, inclusive WebEngine, para o container"

"$CONTAINER_ENGINE" build \
    --file "$BUILDER_FILE" \
    --tag "$BUILDER_IMAGE" \
    "$REPO_ROOT/packaging/appimage"

VOLUME_SPEC="$REPO_ROOT:/workspace:rw"

if [[ "$CONTAINER_ENGINE" == "podman" ]]; then
    # Fedora/SELinux bloqueia até a leitura de scripts em bind mounts sem um
    # rótulo de container. :Z cria um rótulo privado para este builder.
    VOLUME_SPEC="$REPO_ROOT:/workspace:rw,Z"
fi

RUN_ARGS=(
    --rm
    --env HOME=/workspace/build/appimage/cache/home
    --env CARGO_HOME=/workspace/build/appimage/cache/cargo
    --env KINEIN_APPIMAGE_TOOLS_DIR=/workspace/build/appimage/cache/tools
    --env KINEIN_APPIMAGE_NATIVE_BUILD_DIR=/workspace/build/appimage/native-portable
    --env "CMAKE_BUILD_PARALLEL_LEVEL=${CMAKE_BUILD_PARALLEL_LEVEL:-6}"
    --env "CARGO_BUILD_JOBS=${CARGO_BUILD_JOBS:-8}"
    --env "KINEIN_APPIMAGE_DEVELOPMENT=${KINEIN_APPIMAGE_DEVELOPMENT:-0}"
    --env "KINEIN_VERSION=${KINEIN_VERSION:-}"
    --env TMPDIR=/workspace/build/appimage/cache/tmp
    --volume "$VOLUME_SPEC"
    --workdir /workspace
)
mkdir -p "$REPO_ROOT/build/appimage/cache/tmp"

# A compilacao fica sem swap, como o gate; jobs conservadores fora dele.
available_mib="$(awk '/^MemAvailable:/ {print int($2 / 1024)}' /proc/meminfo)"
ceiling_mib=$((available_mib - 2048))
if (( ceiling_mib < 1024 )); then ceiling_mib=1024; fi
RUN_ARGS+=(--memory "${ceiling_mib}m" --memory-swap "${ceiling_mib}m")

# Saida fora do dist/ (build de desenvolvimento que nao deve sobrescrever o
# artefato publicado): a pasta precisa estar dentro do repositorio montado.
if [[ -n "${KINEIN_APPIMAGE_DIST_DIR:-}" ]]; then
    dist_dir="$(realpath -m "$KINEIN_APPIMAGE_DIST_DIR")"
    if [[ "$dist_dir" != "$REPO_ROOT"/* ]]; then
        echo "erro: KINEIN_APPIMAGE_DIST_DIR precisa estar dentro de $REPO_ROOT" >&2
        exit 1
    fi
    RUN_ARGS+=(--env "KINEIN_APPIMAGE_DIST_DIR=/workspace/${dist_dir#"$REPO_ROOT"/}")
fi

if [[ "$CONTAINER_ENGINE" == "podman" ]]; then
    RUN_ARGS+=(--userns=keep-id)
else
    RUN_ARGS+=(--user "$(id -u):$(id -g)")
fi

echo "==> gerando AppImage no baseline portátil"

"$CONTAINER_ENGINE" run \
    "${RUN_ARGS[@]}" \
    "$BUILDER_IMAGE" \
    bash scripts/empacotar-appimage.sh --baseline-worker

echo "==> build portátil concluído"
echo "==> artefatos disponíveis em: ${KINEIN_APPIMAGE_DIST_DIR:-$REPO_ROOT/dist}"

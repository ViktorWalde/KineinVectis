#!/usr/bin/env bash
# G0.3 (roadmap 53 §0.2): os harnesses QML no Qt 6.4.2 que o AppImage embarca.
#
# Por que existe: o Qt local (6.10) e o do AppImage (6.4.2, Debian 12) nao
# executam o mesmo QML. Em 2026-10-01 o usuario do AppImage via
# "Component is not ready" ao abrir o Git, e todo gate local estava verde —
# o 6.10 cria o componente que o 6.4 recusa. Este passo roda os MESMOS
# harnesses do verificar-qml-logica.sh, com o runner do 6.4, num container
# Debian 12 pinado (packaging/appimage/Containerfile.qml64).
#
# Uso: bash scripts/verificar-qml-logica-qt64.sh
# Um único fixture: KINEIN_QML_TEST=tst_list_parts_render bash scripts/verificar-qml-logica-qt64.sh
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
image="${KINEIN_QML64_IMAGE:-localhost/kinein-vectis-qml64:bookworm}"

if [[ -n "${KINEIN_CONTAINER_ENGINE:-}" ]]; then
    engine="$KINEIN_CONTAINER_ENGINE"
elif command -v podman >/dev/null 2>&1; then
    engine="podman"
elif command -v docker >/dev/null 2>&1; then
    engine="docker"
else
    # Sem container nao ha' Qt 6.4, e pular calado seria o verde falso que
    # este passo existe para impedir.
    echo "erro: o G0.3 precisa de Podman ou Docker (Qt 6.4 do AppImage)." >&2
    exit 1
fi

# O build e' cacheado pelo engine: so' refaz quando o Containerfile muda.
"$engine" build --quiet \
    --file "$repo_root/packaging/appimage/Containerfile.qml64" \
    --tag "$image" \
    "$repo_root/packaging/appimage" >/dev/null

volume="$repo_root:/workspace:ro"
run_args=(--rm --workdir /workspace)
if [[ "$engine" == "podman" ]]; then
    volume="$repo_root:/workspace:ro,Z"
    run_args+=(--userns=keep-id)
else
    run_args+=(--user "$(id -u):$(id -g)")
fi
run_args+=(--volume "$volume" --env KINEIN_QML_RUNNER=/usr/lib/qt6/bin/qml)
if [[ -n "${KINEIN_QML_TEST:-}" ]]; then
    run_args+=(--env "KINEIN_QML_TEST=$KINEIN_QML_TEST")
fi

# Diretorios de runtime e cache gravaveis e do proprio usuario: sem eles o Qt
# e o fontconfig avisam a cada harness, e o aviso de ambiente esconde o real.
# shellcheck disable=SC2016  # expandido dentro do container, nao aqui.
"$engine" run "${run_args[@]}" "$image" bash -c '
    XDG_RUNTIME_DIR="$(mktemp -d)"
    XDG_CACHE_HOME="$(mktemp -d)"
    export XDG_RUNTIME_DIR XDG_CACHE_HOME
    exec bash scripts/verificar-qml-logica.sh
'

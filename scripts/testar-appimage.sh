#!/usr/bin/env bash
# Structural and offscreen runtime smoke test for a generated AppImage.

set -Eeuo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST_DIR="${KINEIN_APPIMAGE_DIST_DIR:-$REPO_ROOT/dist}"

if [[ $# -gt 0 ]]; then
    APPIMAGE="$(realpath "$1")"
else
    APPIMAGE="$(find "$DIST_DIR" -maxdepth 1 -type f -name 'Kinein-Vectis-*-x86_64.AppImage' -print -quit)"
fi

if [[ -z "${APPIMAGE:-}" || ! -x "$APPIMAGE" ]]; then
    echo "erro: AppImage executável não encontrado. Gere-o primeiro." >&2
    exit 1
fi

if [[ -f "$(dirname "$APPIMAGE")/SHA256SUMS" ]]; then
    (
        cd "$(dirname "$APPIMAGE")"
        sha256sum --check SHA256SUMS
    )
fi

TEMP_DIR="$(mktemp -d)"
cleanup() {
    rm -rf "$TEMP_DIR"
}
trap cleanup EXIT

echo "==> extraindo e validando estrutura"
(
    cd "$TEMP_DIR"
    "$APPIMAGE" --appimage-extract >/dev/null
)

APPDIR="$TEMP_DIR/squashfs-root"
required_paths=(
    "$APPDIR/AppRun"
    "$APPDIR/usr/bin/kinein-vectis"
    "$APPDIR/usr/bin/kinein-core"
    "$APPDIR/usr/share/applications/io.github.viktorwalde.KineinVectis.desktop"
    "$APPDIR/usr/share/metainfo/io.github.viktorwalde.KineinVectis.appdata.xml"
    "$APPDIR/usr/share/doc/kinein-vectis/MANUAL.md"
    "$APPDIR/usr/share/doc/kinein-vectis/LICENSE-MIT.txt"
    "$APPDIR/usr/share/doc/kinein-vectis/LICENSE-APACHE-2.0.txt"
    "$APPDIR/usr/plugins/platforms/libqxcb.so"
    "$APPDIR/usr/plugins/platforms/libqoffscreen.so"
    "$APPDIR/usr/qml/QtQuick/qmldir"
    "$APPDIR/usr/qml/QtQuick/Window/qmldir"
    "$APPDIR/usr/qml/QtQml/WorkerScript/qmldir"
)

for required_path in "${required_paths[@]}"; do
    if [[ ! -e "$required_path" ]]; then
        echo "erro: item obrigatório ausente do AppImage: $required_path" >&2
        exit 1
    fi
done

if ! find "$APPDIR/usr/lib" -maxdepth 1 -type f -name 'libQt6Core.so*' -print -quit | grep -q .; then
    echo "erro: runtime Qt6 não foi empacotado." >&2
    exit 1
fi

echo "==> validando core empacotado"
PING_RESPONSE="$({ printf '%s\n' '{"jsonrpc":"2.0","id":1,"method":"core.ping","params":{}}'; } \
    | timeout 10 "$APPDIR/usr/bin/kinein-core")"
if [[ "$PING_RESPONSE" != *'"status":"ok"'* ]]; then
    echo "erro: kinein-core empacotado não respondeu ao ping." >&2
    exit 1
fi

echo "==> smoke offscreen da aplicação"
SMOKE_LOG="$TEMP_DIR/smoke.log"
if ! timeout 30 env \
    -u KINEIN_CORE_BIN \
    APPIMAGE_EXTRACT_AND_RUN=1 \
    QT_QPA_PLATFORM=offscreen \
    KINEIN_PERF_MARKER=1 \
    KINEIN_PERF_EXIT=1 \
    "$APPIMAGE" >"$SMOKE_LOG" 2>&1; then
    sed -n '1,240p' "$SMOKE_LOG" >&2
    echo "erro: AppImage falhou no smoke offscreen." >&2
    exit 1
fi

if ! grep -q 'KINEIN_PERF first_frame_ms=' "$SMOKE_LOG"; then
    sed -n '1,240p' "$SMOKE_LOG" >&2
    echo "erro: a UI não confirmou o primeiro frame." >&2
    exit 1
fi

echo "AppImage validado: $APPIMAGE"

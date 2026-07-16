#!/usr/bin/env bash
# Structural and offscreen runtime smoke test for a generated AppImage.

set -Eeuo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST_DIR="${KINEIN_APPIMAGE_DIST_DIR:-$REPO_ROOT/dist}"

if [[ $# -gt 0 ]]; then
    APPIMAGE="$(realpath "$1")"
else
    APPIMAGE="$(
        find "$DIST_DIR" -maxdepth 1 -type f -name 'Kinein-Vectis-*-x86_64.AppImage' -print |
            sort -V |
            tail -n 1
    )"
fi

if [[ -z "${APPIMAGE:-}" || ! -x "$APPIMAGE" ]]; then
    echo "erro: AppImage executável não encontrado. Gere-o primeiro." >&2
    exit 1
fi

DISTRIBUTION_DIR="$(dirname "$APPIMAGE")"

if [[ ! -x "$DISTRIBUTION_DIR/instalar-kinein-vectis.sh" ]]; then
    echo "erro: instalador de usuário ausente ao lado do AppImage." >&2
    exit 1
fi

if [[ ! -f "$APPIMAGE.sha256" ]]; then
    echo "erro: checksum específico ausente ao lado do AppImage." >&2
    exit 1
fi

if [[ ! -f "$DISTRIBUTION_DIR/Tutorial.md" ]]; then
    echo "erro: Tutorial.md ausente ao lado do AppImage." >&2
    exit 1
fi

if ! cmp -s "$REPO_ROOT/Tutorial.md" "$DISTRIBUTION_DIR/Tutorial.md"; then
    echo "erro: Tutorial.md distribuído está desatualizado." >&2
    exit 1
fi

if [[ -f "$DISTRIBUTION_DIR/SHA256SUMS" ]]; then
    (
        cd "$DISTRIBUTION_DIR"
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
    "$APPDIR/AppRun.wrapped"
    "$APPDIR/apprun-hooks/kinein-portable-graphics-hook.sh"
    "$APPDIR/usr/bin/kinein-vectis"
    "$APPDIR/usr/bin/kinein-core"
    "$APPDIR/usr/share/applications/io.github.viktorwalde.KineinVectis.desktop"
    "$APPDIR/usr/share/metainfo/io.github.viktorwalde.KineinVectis.appdata.xml"
    "$APPDIR/usr/share/doc/kinein-vectis/MANUAL.md"
    "$APPDIR/usr/share/doc/kinein-vectis/LICENSE-MIT.txt"
    "$APPDIR/usr/share/doc/kinein-vectis/LICENSE-APACHE-2.0.txt"
    "$APPDIR/usr/share/icons/hicolor/512x512/apps/kinein-vectis.png"
    "$APPDIR/usr/plugins/platforms/libqxcb.so"
    "$APPDIR/usr/plugins/platforms/libqoffscreen.so"
    "$APPDIR/usr/plugins/platforms/libqminimal.so"
    "$APPDIR/usr/plugins/platforms/libqwayland-egl.so"
    "$APPDIR/usr/plugins/platforms/libqwayland-generic.so"
    "$APPDIR/usr/plugins/wayland-graphics-integration-client/libqt-plugin-wayland-egl.so"
    "$APPDIR/usr/plugins/wayland-shell-integration/libxdg-shell.so"
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

if ! grep -Fq 'kinein-portable-graphics-hook.sh' "$APPDIR/AppRun"; then
    echo "erro: AppRun não carrega a política gráfica portátil." >&2
    exit 1
fi
if ! grep -Fq 'KINEIN_GRAPHICS_BACKEND' \
    "$APPDIR/apprun-hooks/kinein-portable-graphics-hook.sh"
then
    echo "erro: hook gráfico portátil está incompleto." >&2
    exit 1
fi

echo "==> validando instalador executado fora da pasta de entrega"
DELIVERY_DIR="$TEMP_DIR/delivery"
INSTALL_HOME="$TEMP_DIR/installer-home"
install -d "$DELIVERY_DIR" "$INSTALL_HOME"
cp "$APPIMAGE" "$DELIVERY_DIR/$(basename "$APPIMAGE")"
cp "$DISTRIBUTION_DIR/instalar-kinein-vectis.sh" \
    "$DELIVERY_DIR/instalar-kinein-vectis.sh"
chmod 0755 "$DELIVERY_DIR/$(basename "$APPIMAGE")" \
    "$DELIVERY_DIR/instalar-kinein-vectis.sh"
(
    cd "$REPO_ROOT"
    HOME="$INSTALL_HOME" \
        XDG_DATA_HOME="$INSTALL_HOME/data" \
        XDG_CACHE_HOME="$INSTALL_HOME/cache" \
        "$DELIVERY_DIR/instalar-kinein-vectis.sh" >/dev/null
)
INSTALLED_DESKTOP="$INSTALL_HOME/data/applications/kinein-vectis.desktop"
INSTALLED_ICON="$INSTALL_HOME/data/icons/hicolor/512x512/apps/kinein-vectis.png"
if [[ ! -s "$INSTALLED_DESKTOP" || ! -s "$INSTALLED_ICON" ]]; then
    echo "erro: instalador não gerou .desktop e ícone nos diretórios XDG." >&2
    exit 1
fi
if ! grep -Fq "$DELIVERY_DIR/$(basename "$APPIMAGE")" "$INSTALLED_DESKTOP"; then
    echo "erro: atalho instalado não aponta para o AppImage da entrega." >&2
    exit 1
fi

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
    -u QT_QUICK_BACKEND \
    -u QSG_RHI_BACKEND \
    APPIMAGE_EXTRACT_AND_RUN=1 \
    QT_QPA_PLATFORM=offscreen \
    QSG_INFO=1 \
    KINEIN_PERF_MARKER=1 \
    KINEIN_PERF_EXIT=1 \
    "$APPIMAGE" >"$SMOKE_LOG" 2>&1; then
    sed -n '1,240p' "$SMOKE_LOG" >&2
    echo "erro: AppImage falhou no smoke offscreen." >&2
    exit 1
fi

if ! grep -q 'Loading backend software' "$SMOKE_LOG"; then
    sed -n '1,240p' "$SMOKE_LOG" >&2
    echo "erro: AppImage não selecionou o renderer portátil por padrão." >&2
    exit 1
fi

if ! grep -q 'KINEIN_PERF first_frame_ms=' "$SMOKE_LOG"; then
    sed -n '1,240p' "$SMOKE_LOG" >&2
    echo "erro: a UI não confirmou o primeiro frame." >&2
    exit 1
fi

echo "AppImage validado: $APPIMAGE"

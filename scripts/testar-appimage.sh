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

if ! cmp -s "$REPO_ROOT/DocsPublic/tutorial.md" "$DISTRIBUTION_DIR/Tutorial.md"; then
    echo "erro: Tutorial.md distribuído está desatualizado." >&2
    exit 1
fi

if [[ -f "$DISTRIBUTION_DIR/SHA256SUMS" ]]; then
    (
        cd "$DISTRIBUTION_DIR"
        sha256sum --check SHA256SUMS
    )
fi

cache_root="${XDG_CACHE_HOME:-$HOME/.cache}"
mkdir -p "$cache_root"
TEMP_DIR="$(mktemp -d "$cache_root/kinein-appimage-smoke.XXXXXX")"
cleanup() {
    rm -rf "$TEMP_DIR"
}
trap cleanup EXIT
mkdir -p "$TEMP_DIR/tmp"
export TMPDIR="$TEMP_DIR/tmp"

echo "==> extraindo e validando estrutura"
(
    cd "$TEMP_DIR"
    "$APPIMAGE" --appimage-extract >/dev/null
)

APPDIR="$TEMP_DIR/squashfs-root"
package_id=io.github.viktorwalde.KineinVectis
icon_name=kinein-vectis
desktop_name=kinein-vectis
command_name=kinein
if [[ "$(basename "$APPIMAGE")" == *-dev* ]]; then
    package_id+=.Development
    icon_name="$package_id"
    desktop_name="$package_id"
    command_name=kinein-dev
fi
required_paths=(
    "$APPDIR/AppRun"
    "$APPDIR/AppRun.wrapped"
    "$APPDIR/apprun-hooks/kinein-portable-graphics-hook.sh"
    "$APPDIR/usr/bin/kinein-vectis"
    "$APPDIR/usr/bin/kinein-core"
    "$APPDIR/usr/share/applications/$package_id.desktop"
    "$APPDIR/usr/share/metainfo/$package_id.appdata.xml"
    "$APPDIR/usr/share/doc/kinein-vectis/MANUAL.md"
    "$APPDIR/usr/share/doc/kinein-vectis/LICENSE-MIT.txt"
    "$APPDIR/usr/share/doc/kinein-vectis/LICENSE-APACHE-2.0.txt"
    "$APPDIR/usr/share/icons/hicolor/512x512/apps/$icon_name.png"
    "$APPDIR/usr/plugins/platforms/libqxcb.so"
    "$APPDIR/usr/plugins/platforms/libqoffscreen.so"
    "$APPDIR/usr/plugins/platforms/libqminimal.so"
    # Os icones da arvore sao SVG; sem o plugin o Qt so' avisa no stderr.
    "$APPDIR/usr/plugins/imageformats/libqsvg.so"
    "$APPDIR/usr/plugins/platforms/libqwayland.so"
    "$APPDIR/usr/plugins/wayland-graphics-integration-client/libqt-plugin-wayland-egl.so"
    "$APPDIR/usr/plugins/wayland-shell-integration/libxdg-shell.so"
    "$APPDIR/usr/qml/QtQuick/qmldir"
    "$APPDIR/usr/qml/QtQuick/Window/qmldir"
    "$APPDIR/usr/qml/QtQml/WorkerScript/qmldir"
    "$APPDIR/usr/qml/QtWebEngine/qmldir"
    "$APPDIR/usr/lib/libQt6Network.so.6"
    "$APPDIR/usr/lib/libQt6WebEngineCore.so.6"
)

for required_path in "${required_paths[@]}"; do
    if [[ ! -e "$required_path" ]]; then
        echo "erro: item obrigatório ausente do AppImage: $required_path" >&2
        exit 1
    fi
done

if [[ "$package_id" == *.Development ]]; then
    grep -Fxq 'Name=Kinein Vectis (desenvolvimento)' \
        "$APPDIR/usr/share/applications/$package_id.desktop"
    grep -Fq "$package_id" "$APPDIR/usr/share/metainfo/$package_id.appdata.xml"
    version_output="$("$APPDIR/usr/bin/kinein-vectis" --version)"
    [[ "$version_output" == *-dev* ]] || { echo "erro: UI sem versao de desenvolvimento." >&2; exit 1; }
fi
if ! find "$APPDIR/usr" -name QtWebEngineProcess -type f -print -quit | grep -q .; then
    echo "erro: QtWebEngineProcess ausente." >&2
    exit 1
fi
if ! find "$APPDIR/usr" -name qtwebengine_resources.pak -type f -print -quit | grep -q .; then
    echo "erro: recursos WebEngine ausentes." >&2
    exit 1
fi

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
protected_files=(
    "$INSTALL_HOME/data/applications/kinein-vectis.desktop"
    "$INSTALL_HOME/data/applications/kinein-vectis-development.desktop"
    "$INSTALL_HOME/data/icons/hicolor/512x512/apps/kinein-vectis.png"
    "$INSTALL_HOME/.local/bin/kinein"
)
if [[ "$package_id" == *.Development ]]; then
    for protected_file in "${protected_files[@]}"; do
        mkdir -p "$(dirname "$protected_file")"
        printf 'stable-test-sentinel\n' > "$protected_file"
    done
fi
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
INSTALLED_DESKTOP="$INSTALL_HOME/data/applications/$desktop_name.desktop"
INSTALLED_ICON="$INSTALL_HOME/data/icons/hicolor/512x512/apps/$icon_name.png"
if [[ ! -s "$INSTALLED_DESKTOP" || ! -s "$INSTALLED_ICON" ]]; then
    echo "erro: instalador não gerou .desktop e ícone nos diretórios XDG." >&2
    exit 1
fi
if [[ ! -x "$INSTALL_HOME/.local/bin/$command_name" ]]; then
    echo "erro: comando do canal ausente: $command_name" >&2
    exit 1
fi
if [[ "$package_id" == *.Development ]]; then
    for protected_file in "${protected_files[@]}"; do
        grep -Fxq 'stable-test-sentinel' "$protected_file" || {
            echo "erro: instalador alterou arquivo do estavel/checkout: $protected_file" >&2
            exit 1
        }
    done
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

# Plugin sem rpath so' carrega quando alguem define LD_LIBRARY_PATH — e o AppImage
# nao define. Foi assim que o `wayland-egl` falhava em uso (53 §5.2.1).
if command -v readelf >/dev/null 2>&1; then
    missing_rpath=""
    while IFS= read -r -d '' plugin_file; do
        # shellcheck disable=SC2016  # $ORIGIN e' literal do carregador dinamico
        if ! readelf -d "$plugin_file" 2>/dev/null | grep -Eq 'R(UN)?PATH.*\$ORIGIN'; then
            missing_rpath+="  ${plugin_file#"$APPDIR"/}"$'\n'
        fi
    done < <(find "$APPDIR/usr/plugins" -type f -name '*.so' -print0)
    if [[ -n "$missing_rpath" ]]; then
        # shellcheck disable=SC2016  # $ORIGIN e' literal do carregador dinamico
        printf 'erro: plugin sem rpath $ORIGIN (nao carrega em execucao):\n%s' "$missing_rpath" >&2
        exit 1
    fi
fi

# A libxkbcommon vem do sistema, junto com os dados de teclado que ela le.
if find "$APPDIR/usr/lib" -maxdepth 1 -name 'libxkbcommon*.so*' -print -quit | grep -q .; then
    echo "erro: libxkbcommon empacotada (le os dados de teclado do sistema; 53 §5.2.1)." >&2
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
    XDG_CONFIG_HOME="$TEMP_DIR/smoke-config" \
    XDG_DATA_HOME="$TEMP_DIR/smoke-data" \
    XDG_CACHE_HOME="$TEMP_DIR/smoke-cache" \
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

# O primeiro frame pode sair com a fiacao QML quebrada: a mesma lista de avisos
# que `verificar-binario-abre.sh` aplica ao binario do checkout reprova aqui o
# Qt do ARTEFATO (2026-10-01: o Qt 6.4 do pacote recusava os paineis de
# ambiente e o checkout, com Qt mais novo, nao via).
if grep -F -f "$REPO_ROOT/scripts/avisos-qml.txt" "$SMOKE_LOG" >"$TEMP_DIR/avisos.log"; then
    sed -n '1,40p' "$TEMP_DIR/avisos.log" >&2
    echo "erro: o AppImage abriu com aviso do motor QML (lista em scripts/avisos-qml.txt)." >&2
    exit 1
fi

# G0.4: o passeio por superficies no Qt do pacote. A primeira tela nao
# mostra a lista do Git com secoes, os paineis de ambiente nem os overlays.
echo "==> passeio por superfícies no AppImage"
(
    cd "$TEMP_DIR"
    env -u KINEIN_CORE_BIN -u QT_QUICK_BACKEND -u QSG_RHI_BACKEND APPIMAGE_EXTRACT_AND_RUN=1 \
        bash "$REPO_ROOT/scripts/run-surface-tour.sh" "$APPIMAGE"
)

echo "AppImage validado: $APPIMAGE"

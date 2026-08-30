#!/usr/bin/env bash
# Creates the AppDir and AppImage using audited, pinned linuxdeploy releases.
#
# A chamada pública delega ao builder portátil. O trabalho abaixo só roda com
# --baseline-worker, argumento interno passado pelo wrapper Debian 12.
# Toda operação sobre APPDIR e plugins Qt deve permanecer neste arquivo.

set -Eeuo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

if [[ "${1:-}" != "--baseline-worker" ]]; then
    if (( $# > 0 )); then
        echo "erro: argumento desconhecido: $1" >&2
        echo "uso: bash scripts/empacotar-appimage.sh" >&2
        exit 2
    fi

    echo "==> empacotamento direto usa o baseline portátil auditado"
    exec bash "$REPO_ROOT/scripts/empacotar-appimage-portatil.sh"
fi

shift

if (( $# > 0 )); then
    echo "erro: --baseline-worker é um argumento interno e não aceita opções." >&2
    exit 2
fi

BUILD_ROOT="${KINEIN_APPIMAGE_BUILD_ROOT:-$REPO_ROOT/build/appimage}"
# O checkout pode ser visto como /workspace dentro do container e por seu
# caminho real no host. Caches CMake não são relocáveis entre essas duas
# raízes, portanto o wrapper portátil escolhe explicitamente outro diretório.
NATIVE_BUILD_DIR="${KINEIN_APPIMAGE_NATIVE_BUILD_DIR:-$BUILD_ROOT/native-host}"
APPDIR="$BUILD_ROOT/AppDir"
DIST_DIR="${KINEIN_APPIMAGE_DIST_DIR:-$REPO_ROOT/dist}"
TOOLS_DIR="${KINEIN_APPIMAGE_TOOLS_DIR:-$BUILD_ROOT/cache/tools}"
CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$BUILD_ROOT/cargo-target}"

LINUXDEPLOY_VERSION="1-alpha-20251107-1"
LINUXDEPLOY_COMMIT="cc7b86472c3caa3fd729b9dc502fd2aa78394257"
LINUXDEPLOY_SHA256="c20cd71e3a4e3b80c3483cef793cda3f4e990aca14014d23c544ca3ce1270b4d"
LINUXDEPLOY_URL="https://github.com/linuxdeploy/linuxdeploy/releases/download/$LINUXDEPLOY_VERSION/linuxdeploy-x86_64.AppImage"

QT_PLUGIN_VERSION="1-alpha-20250213-1"
QT_PLUGIN_COMMIT="ce5291e25979d7d6417551d787f308b88c1b8d76"
QT_PLUGIN_SHA256="15106be885c1c48a021198e7e1e9a48ce9d02a86dd0a1848f00bdbf3c1c92724"
QT_PLUGIN_URL="https://github.com/linuxdeploy/linuxdeploy-plugin-qt/releases/download/$QT_PLUGIN_VERSION/linuxdeploy-plugin-qt-x86_64.AppImage"

TYPE2_RUNTIME_COMMIT="75849dce7cc37e4319b633df1f116ca895c71a12"
TYPE2_RUNTIME_SHA256="1cc49bcf1e2ccd593c379adb17c9f85a36d619088296504de95b1d06215aebbf"
TYPE2_RUNTIME_URL="https://github.com/AppImage/type2-runtime/releases/download/continuous/runtime-x86_64"

LINUXDEPLOY="$TOOLS_DIR/linuxdeploy-x86_64.AppImage"
QT_PLUGIN="$TOOLS_DIR/linuxdeploy-plugin-qt-x86_64.AppImage"
TYPE2_RUNTIME="$TOOLS_DIR/appimage-runtime-x86_64"

require_command() {
    local command_name="$1"

    if ! command -v "$command_name" >/dev/null 2>&1; then
        echo "erro: comando obrigatório não encontrado: $command_name" >&2
        exit 1
    fi
}

download_checked() {
    local url="$1"
    local expected="$2"
    local destination="$3"
    local temporary="${destination}.download"

    if [[ -f "$destination" ]]; then
        local current

        current="$(
            sha256sum "$destination" |
                cut -d ' ' -f 1
        )"

        if [[ "$current" == "$expected" ]]; then
            chmod 0755 "$destination"
            return
        fi

        echo "aviso: cache divergente removido: $destination" >&2
        rm -f "$destination"
    fi

    echo "==> baixando $(basename "$destination")"

    curl \
        --fail \
        --location \
        --retry 3 \
        --retry-all-errors \
        --output "$temporary" \
        "$url"

    local actual

    actual="$(
        sha256sum "$temporary" |
            cut -d ' ' -f 1
    )"

    if [[ "$actual" != "$expected" ]]; then
        rm -f "$temporary"

        echo "erro: SHA256 inválido para $url" >&2
        echo "esperado: $expected" >&2
        echo "recebido: $actual" >&2
        exit 1
    fi

    mv "$temporary" "$destination"
    chmod 0755 "$destination"
}

copy_wayland_plugin_group() {
    local plugin_group="$1"
    local source_dir="$QT_PLUGIN_DIR/$plugin_group"
    local destination_dir="$APPDIR/usr/plugins/$plugin_group"

    if [[ ! -d "$source_dir" ]]; then
        echo "erro: grupo de plugins Wayland ausente no builder:" >&2
        echo "  $source_dir" >&2
        echo "confirme que qt6-wayland está instalado no Containerfile." >&2
        exit 1
    fi

    install -d "$destination_dir"
    cp -a "$source_dir/." "$destination_dir/"

    echo "  incluído: $plugin_group"
}

require_appdir_file() {
    local required_file="$1"

    if [[ ! -f "$required_file" ]]; then
        echo "erro: arquivo obrigatório ausente do AppDir:" >&2
        echo "  $required_file" >&2
        exit 1
    fi
}

check_dynamic_dependencies() {
    local plugin_file="$1"
    local missing

    missing="$(
        LD_LIBRARY_PATH="$APPDIR/usr/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}" \
            ldd "$plugin_file" 2>/dev/null |
            grep 'not found' ||
            true
    )"

    if [[ -n "$missing" ]]; then
        echo "erro: dependências dinâmicas ausentes no plugin:" >&2
        echo "  $plugin_file" >&2
        echo "$missing" >&2
        exit 1
    fi
}

for command_name in \
    cargo \
    clang++ \
    cmake \
    curl \
    desktop-file-validate \
    find \
    grep \
    ldd \
    mktemp \
    ninja \
    qmake6 \
    sha256sum
do
    require_command "$command_name"
done

if [[ "$(uname -m)" != "x86_64" ]]; then
    echo "erro: esta receita auditada gera apenas AppImage x86_64." >&2
    exit 1
fi

mkdir -p \
    "$BUILD_ROOT" \
    "$DIST_DIR" \
    "$TOOLS_DIR" \
    "$CARGO_TARGET_DIR"

# A entrega é construída integralmente fora de dist/. Assim um erro de
# compilação, plugin ou linuxdeploy preserva o último AppImage válido.
DELIVERY_STAGING_DIR="$(mktemp -d "$BUILD_ROOT/delivery-staging.XXXXXX")"
PUBLISH_TEMP_FILES=()

cleanup_delivery() {
    local temporary_file

    for temporary_file in "${PUBLISH_TEMP_FILES[@]}"; do
        rm -f -- "$temporary_file"
    done

    cmake -E remove_directory "$DELIVERY_STAGING_DIR"
}

trap cleanup_delivery EXIT

download_checked \
    "$LINUXDEPLOY_URL" \
    "$LINUXDEPLOY_SHA256" \
    "$LINUXDEPLOY"

download_checked \
    "$QT_PLUGIN_URL" \
    "$QT_PLUGIN_SHA256" \
    "$QT_PLUGIN"

download_checked \
    "$TYPE2_RUNTIME_URL" \
    "$TYPE2_RUNTIME_SHA256" \
    "$TYPE2_RUNTIME"

echo "==> ferramentas de packaging auditadas"
echo "linuxdeploy $LINUXDEPLOY_VERSION ($LINUXDEPLOY_COMMIT)"
echo "linuxdeploy-plugin-qt $QT_PLUGIN_VERSION ($QT_PLUGIN_COMMIT)"
echo "AppImage type-2 runtime ($TYPE2_RUNTIME_COMMIT)"

echo "==> compilando kinein-core release"

CARGO_TARGET_DIR="$CARGO_TARGET_DIR" \
cargo build \
    --locked \
    --release \
    --package kinein-core \
    --manifest-path "$REPO_ROOT/Cargo.toml"

echo "==> configurando frontend Qt release"

cmake \
    --fresh \
    -S "$REPO_ROOT" \
    -B "$NATIVE_BUILD_DIR" \
    -G Ninja \
    -DCMAKE_BUILD_TYPE=Release \
    -DCMAKE_CXX_COMPILER=clang++ \
    -DCMAKE_INSTALL_PREFIX=/usr \
    -DKINEIN_ENABLE_HARDENING=ON \
    -DKINEIN_ENABLE_SANITIZERS=OFF \
    -DKINEIN_WARNINGS_AS_ERRORS=ON

cmake --build \
    "$NATIVE_BUILD_DIR" \
    --parallel "$(nproc)"

echo "==> montando AppDir"

cmake -E remove_directory "$APPDIR"
cmake -E make_directory "$APPDIR"

DESTDIR="$APPDIR" \
cmake --install \
    "$NATIVE_BUILD_DIR" \
    --component Unspecified

install \
    -D \
    -m 0755 \
    "$CARGO_TARGET_DIR/release/kinein-core" \
    "$APPDIR/usr/bin/kinein-core"

desktop-file-validate \
    "$APPDIR/usr/share/applications/io.github.viktorwalde.KineinVectis.desktop"

VERSION="$(
    sed -n \
        's/^project(kinein-vectis VERSION \([^ ]*\).*/\1/p' \
        "$REPO_ROOT/CMakeLists.txt"
)"

VERSION="${KINEIN_VERSION:-$VERSION}"

if [[ -z "$VERSION" ]]; then
    echo "erro: não foi possível determinar a versão do produto." >&2
    exit 1
fi

OUTPUT_BASENAME="Kinein-Vectis-$VERSION-x86_64.AppImage"
OUTPUT_FILE="$DELIVERY_STAGING_DIR/$OUTPUT_BASENAME"
FINAL_OUTPUT_FILE="$DIST_DIR/$OUTPUT_BASENAME"

export APPIMAGE_EXTRACT_AND_RUN=1

QMAKE="$(
    command -v qmake6 ||
        command -v qmake
)"
export QMAKE

export QML_SOURCES_PATHS="$REPO_ROOT/ui/qml"
export PATH="$TOOLS_DIR:$PATH"
export LDAI_OUTPUT="$OUTPUT_FILE"
export LDAI_RUNTIME_FILE="$TYPE2_RUNTIME"

QT_PLUGIN_DIR="$("$QMAKE" -query QT_INSTALL_PLUGINS)"

if [[ ! -d "$QT_PLUGIN_DIR" ]]; then
    echo "erro: diretório de plugins Qt não encontrado:" >&2
    echo "  $QT_PLUGIN_DIR" >&2
    exit 1
fi

echo "==> diretório de plugins Qt"
echo "$QT_PLUGIN_DIR"

PLATFORM_PLUGINS=(
    libqxcb.so
    libqoffscreen.so
    libqminimal.so
    libqwayland-egl.so
    libqwayland-generic.so
)

echo "==> validando plugins de plataforma no builder"

for plugin_name in "${PLATFORM_PLUGINS[@]}"; do
    plugin_path="$QT_PLUGIN_DIR/platforms/$plugin_name"

    if [[ ! -f "$plugin_path" ]]; then
        echo "erro: plugin Qt obrigatório ausente no builder:" >&2
        echo "  $plugin_path" >&2
        echo "confirme que qt6-wayland está instalado no Containerfile." >&2
        exit 1
    fi

    echo "  ok: platforms/$plugin_name"
done

EXTRA_PLATFORM_PLUGINS="$(
    IFS=';'
    echo "${PLATFORM_PLUGINS[*]}"
)"

export EXTRA_PLATFORM_PLUGINS

echo "==> primeira etapa: empacotando Qt, QML e plugins de plataforma"

"$LINUXDEPLOY" \
    --appdir "$APPDIR" \
    --plugin qt

WAYLAND_PLUGIN_GROUPS=(
    wayland-graphics-integration-client
    wayland-shell-integration
    wayland-decoration-client
)

echo "==> copiando integrações auxiliares do cliente Wayland"

for plugin_group in "${WAYLAND_PLUGIN_GROUPS[@]}"; do
    copy_wayland_plugin_group "$plugin_group"
done

echo "==> segunda etapa: empacotando dependências dos plugins Wayland"

"$LINUXDEPLOY" \
    --appdir "$APPDIR"

REQUIRED_WAYLAND_FILES=(
    "$APPDIR/usr/plugins/platforms/libqwayland-egl.so"
    "$APPDIR/usr/plugins/platforms/libqwayland-generic.so"
    "$APPDIR/usr/plugins/wayland-graphics-integration-client/libqt-plugin-wayland-egl.so"
    "$APPDIR/usr/plugins/wayland-shell-integration/libxdg-shell.so"
)

REQUIRED_PLATFORM_FILES=(
    "$APPDIR/usr/plugins/platforms/libqxcb.so"
    "$APPDIR/usr/plugins/platforms/libqoffscreen.so"
    "$APPDIR/usr/plugins/platforms/libqminimal.so"
)

echo "==> validando plugins de plataforma básicos"

for required_file in "${REQUIRED_PLATFORM_FILES[@]}"; do
    require_appdir_file "$required_file"
    check_dynamic_dependencies "$required_file"
    echo "  ok: ${required_file#"$APPDIR/"}"
done

echo "==> validando arquivos obrigatórios do Wayland"

for required_file in "${REQUIRED_WAYLAND_FILES[@]}"; do
    require_appdir_file "$required_file"
    echo "  ok: ${required_file#"$APPDIR/"}"
done

if ! find \
    "$APPDIR/usr/lib" \
    -maxdepth 1 \
    -type f \
    -name 'libQt6WaylandClient.so*' \
    -print -quit |
    grep -q .
then
    echo "erro: libQt6WaylandClient não foi empacotada." >&2
    exit 1
fi

echo "  ok: libQt6WaylandClient"

if ! find \
    "$APPDIR/usr/lib" \
    -maxdepth 1 \
    -type f \
    -name 'libQt6WaylandEglClientHwIntegration.so*' \
    -print -quit |
    grep -q .
then
    echo "erro: libQt6WaylandEglClientHwIntegration não foi empacotada." >&2
    exit 1
fi

echo "  ok: libQt6WaylandEglClientHwIntegration"

echo "==> validando dependências dinâmicas dos plugins Wayland"

for plugin_file in "${REQUIRED_WAYLAND_FILES[@]}"; do
    check_dynamic_dependencies "$plugin_file"
    echo "  ok: ${plugin_file#"$APPDIR/"}"
done

echo "==> instalando launcher gráfico portátil"
install \
    -D \
    -m 0755 \
    "$REPO_ROOT/packaging/appimage/kinein-portable-graphics-hook.sh" \
    "$APPDIR/apprun-hooks/kinein-portable-graphics-hook.sh"

echo "==> gerando AppImage"

"$LINUXDEPLOY" \
    --appdir "$APPDIR" \
    --output appimage

if [[ ! -x "$OUTPUT_FILE" ]]; then
    echo "erro: linuxdeploy não produziu o AppImage esperado:" >&2
    echo "  $OUTPUT_FILE" >&2
    exit 1
fi

(
    cd "$DELIVERY_STAGING_DIR"

    sha256sum \
        "$OUTPUT_BASENAME" \
        > SHA256SUMS
)

cp \
    "$DELIVERY_STAGING_DIR/SHA256SUMS" \
    "$OUTPUT_FILE.sha256"

install \
    -m 0755 \
    "$REPO_ROOT/scripts/instalar-appimage.sh" \
    "$DELIVERY_STAGING_DIR/instalar-kinein-vectis.sh"

install \
    -m 0644 \
    "$REPO_ROOT/Tutorial.md" \
    "$DELIVERY_STAGING_DIR/Tutorial.md"

publish_delivery_file() {
    local source_file="$1"
    local destination_file="$2"
    local mode="$3"
    local temporary_file="${destination_file}.kinein-new-$$"

    PUBLISH_TEMP_FILES+=("$temporary_file")
    install -m "$mode" "$source_file" "$temporary_file"
    mv -f -- "$temporary_file" "$destination_file"
}

echo "==> publicando entrega validada em dist"

publish_delivery_file \
    "$OUTPUT_FILE" \
    "$FINAL_OUTPUT_FILE" \
    0755
publish_delivery_file \
    "$OUTPUT_FILE.sha256" \
    "$FINAL_OUTPUT_FILE.sha256" \
    0644
publish_delivery_file \
    "$DELIVERY_STAGING_DIR/SHA256SUMS" \
    "$DIST_DIR/SHA256SUMS" \
    0644
publish_delivery_file \
    "$DELIVERY_STAGING_DIR/instalar-kinein-vectis.sh" \
    "$DIST_DIR/instalar-kinein-vectis.sh" \
    0755
publish_delivery_file \
    "$DELIVERY_STAGING_DIR/Tutorial.md" \
    "$DIST_DIR/Tutorial.md" \
    0644

echo "==> AppImage pronto"
echo "$FINAL_OUTPUT_FILE"
echo "$FINAL_OUTPUT_FILE.sha256"
echo "$DIST_DIR/SHA256SUMS"
echo "$DIST_DIR/instalar-kinein-vectis.sh"
echo "$DIST_DIR/Tutorial.md"

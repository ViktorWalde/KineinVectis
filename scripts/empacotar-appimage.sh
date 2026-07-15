#!/usr/bin/env bash
# Creates the AppDir and AppImage using audited, pinned linuxdeploy releases.

set -Eeuo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BUILD_ROOT="${KINEIN_APPIMAGE_BUILD_ROOT:-$REPO_ROOT/build/appimage}"
NATIVE_BUILD_DIR="$BUILD_ROOT/native"
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
    if ! command -v "$1" >/dev/null 2>&1; then
        echo "erro: comando obrigatório não encontrado: $1" >&2
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
        current="$(sha256sum "$destination" | cut -d ' ' -f 1)"
        if [[ "$current" == "$expected" ]]; then
            chmod 0755 "$destination"
            return
        fi
        echo "aviso: cache divergente removido: $destination" >&2
        rm -f "$destination"
    fi

    echo "==> baixando $(basename "$destination")"
    curl --fail --location --retry 3 --output "$temporary" "$url"

    local actual
    actual="$(sha256sum "$temporary" | cut -d ' ' -f 1)"
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

for command_name in cargo cmake curl desktop-file-validate ninja qmake6 sha256sum; do
    require_command "$command_name"
done

if [[ "$(uname -m)" != "x86_64" ]]; then
    echo "erro: esta receita auditada gera apenas AppImage x86_64." >&2
    exit 1
fi

mkdir -p "$BUILD_ROOT" "$DIST_DIR" "$TOOLS_DIR" "$CARGO_TARGET_DIR"

download_checked "$LINUXDEPLOY_URL" "$LINUXDEPLOY_SHA256" "$LINUXDEPLOY"
download_checked "$QT_PLUGIN_URL" "$QT_PLUGIN_SHA256" "$QT_PLUGIN"
download_checked "$TYPE2_RUNTIME_URL" "$TYPE2_RUNTIME_SHA256" "$TYPE2_RUNTIME"

echo "==> ferramentas de packaging auditadas"
echo "linuxdeploy $LINUXDEPLOY_VERSION ($LINUXDEPLOY_COMMIT)"
echo "linuxdeploy-plugin-qt $QT_PLUGIN_VERSION ($QT_PLUGIN_COMMIT)"
echo "AppImage type-2 runtime ($TYPE2_RUNTIME_COMMIT)"

echo "==> compilando kinein-core release"
CARGO_TARGET_DIR="$CARGO_TARGET_DIR" cargo build \
    --locked \
    --release \
    --package kinein-core \
    --manifest-path "$REPO_ROOT/Cargo.toml"

echo "==> configurando frontend Qt release"
cmake -S "$REPO_ROOT" -B "$NATIVE_BUILD_DIR" -G Ninja \
    -DCMAKE_BUILD_TYPE=Release \
    -DCMAKE_C_COMPILER=clang \
    -DCMAKE_CXX_COMPILER=clang++ \
    -DCMAKE_INSTALL_PREFIX=/usr \
    -DKINEIN_ENABLE_HARDENING=ON \
    -DKINEIN_ENABLE_SANITIZERS=OFF \
    -DKINEIN_WARNINGS_AS_ERRORS=ON

cmake --build "$NATIVE_BUILD_DIR" --parallel "$(nproc)"

echo "==> montando AppDir"
cmake -E remove_directory "$APPDIR"
cmake -E make_directory "$APPDIR"
DESTDIR="$APPDIR" cmake --install "$NATIVE_BUILD_DIR" --component Unspecified
install -D -m 0755 "$CARGO_TARGET_DIR/release/kinein-core" \
    "$APPDIR/usr/bin/kinein-core"

desktop-file-validate "$APPDIR/usr/share/applications/io.github.viktorwalde.KineinVectis.desktop"

VERSION="${KINEIN_VERSION:-$(sed -n 's/^project(kinein-vectis VERSION \([^ ]*\).*/\1/p' "$REPO_ROOT/CMakeLists.txt")}"
if [[ -z "$VERSION" ]]; then
    echo "erro: não foi possível determinar a versão do produto." >&2
    exit 1
fi

OUTPUT_FILE="$DIST_DIR/Kinein-Vectis-$VERSION-x86_64.AppImage"
rm -f "$OUTPUT_FILE" "$OUTPUT_FILE.sha256" "$DIST_DIR/SHA256SUMS"

export APPIMAGE_EXTRACT_AND_RUN=1
export QMAKE="${QMAKE:-$(command -v qmake6 || command -v qmake)}"
export QML_SOURCES_PATHS="$REPO_ROOT/ui/qml"
export PATH="$TOOLS_DIR:$PATH"
export LDAI_OUTPUT="$OUTPUT_FILE"
export LDAI_RUNTIME_FILE="$TYPE2_RUNTIME"

QT_PLUGIN_DIR="$($QMAKE -query QT_INSTALL_PLUGINS)"
PLATFORM_PLUGINS=(libqoffscreen.so libqminimal.so)
if [[ -f "$QT_PLUGIN_DIR/platforms/libqwayland-egl.so" ]]; then
    PLATFORM_PLUGINS+=(libqwayland-egl.so)
fi
if [[ -f "$QT_PLUGIN_DIR/platforms/libqwayland-generic.so" ]]; then
    PLATFORM_PLUGINS+=(libqwayland-generic.so)
fi
EXTRA_PLATFORM_PLUGINS="$(IFS=';'; echo "${PLATFORM_PLUGINS[*]}")"
export EXTRA_PLATFORM_PLUGINS

echo "==> empacotando runtime Qt/QML e gerando AppImage"
"$LINUXDEPLOY" --appdir "$APPDIR" --plugin qt --output appimage

if [[ ! -x "$OUTPUT_FILE" ]]; then
    echo "erro: linuxdeploy não produziu $OUTPUT_FILE" >&2
    exit 1
fi

(
    cd "$DIST_DIR"
    sha256sum "$(basename "$OUTPUT_FILE")" > SHA256SUMS
)
cp "$DIST_DIR/SHA256SUMS" "$OUTPUT_FILE.sha256"

echo "==> AppImage pronto"
echo "$OUTPUT_FILE"
echo "$DIST_DIR/SHA256SUMS"

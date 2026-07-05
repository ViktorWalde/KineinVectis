#!/bin/sh
# Instala o atalho do Kinein Vectis no menu de aplicativos do usuario.
# Nao precisa de sudo: escreve apenas em ~/.local/share/applications.

set -eu

REPO_ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
APPS_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/applications"
DESKTOP_FILE="$APPS_DIR/kinein-vectis.desktop"

mkdir -p "$APPS_DIR"

cat > "$DESKTOP_FILE" <<EOF
[Desktop Entry]
Type=Application
Name=Kinein Vectis
Comment=IDE open source, Linux-first, offline
Exec=$REPO_ROOT/scripts/kinein-vectis
Icon=$REPO_ROOT/imagens/app-icon.png
Terminal=false
Categories=Development;IDE;
StartupNotify=true
EOF

chmod 644 "$DESKTOP_FILE"
update-desktop-database "$APPS_DIR" 2>/dev/null || true

echo "atalho instalado em: $DESKTOP_FILE"
echo "procure por 'Kinein Vectis' no menu de aplicativos."

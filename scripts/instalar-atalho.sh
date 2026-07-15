#!/bin/sh
# Instala o atalho do checkout de desenvolvimento no menu de aplicativos.
# Nao precisa de sudo: escreve apenas em ~/.local/share/applications.

set -eu

REPO_ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
APPS_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/applications"
DESKTOP_FILE="$APPS_DIR/kinein-vectis-development.desktop"
LEGACY_DESKTOP_FILE="$APPS_DIR/kinein-vectis.desktop"
LAUNCHER="$REPO_ROOT/scripts/kinein-vectis"

mkdir -p "$APPS_DIR"

# Antes do AppImage, este script usava o mesmo desktop id da distribuição.
# Remova apenas a entrada que comprovadamente pertence a este checkout; um
# atalho do AppImage no mesmo caminho deve ser preservado.
if [ -f "$LEGACY_DESKTOP_FILE" ] \
    && grep -Fqx "Exec=$LAUNCHER" "$LEGACY_DESKTOP_FILE"; then
    rm -f "$LEGACY_DESKTOP_FILE"
fi

cat > "$DESKTOP_FILE" <<EOF
[Desktop Entry]
Type=Application
Name=Kinein Vectis (Desenvolvimento)
GenericName=Ambiente de desenvolvimento da Kinein Vectis
Comment=Executa a Kinein Vectis diretamente deste checkout
Exec=$LAUNCHER
Icon=$REPO_ROOT/imagens/app-icon.png
Terminal=false
Categories=Development;IDE;
StartupNotify=true
StartupWMClass=kinein-vectis
EOF

chmod 644 "$DESKTOP_FILE"
update-desktop-database "$APPS_DIR" 2>/dev/null || true

echo "atalho instalado em: $DESKTOP_FILE"
echo "procure por 'Kinein Vectis (Desenvolvimento)' no menu de aplicativos."
echo "o atalho 'Kinein Vectis' do AppImage, se existir, foi preservado."

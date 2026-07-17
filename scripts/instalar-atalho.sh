#!/bin/sh
# Instala o atalho do checkout de desenvolvimento no menu de aplicativos.
# Nao precisa de sudo: escreve apenas em ~/.local/share.
#
# POR QUE O ICONE VAI PELO TEMA hicolor, e nao por caminho absoluto (2026-07-17).
# `Icon=/caminho/para/app-icon.png` FUNCIONA, mas o gnome-shell no Wayland guarda
# o icone renderizado e nao o rele so' porque o arquivo mudou no mesmo caminho —
# o autor trocava o icone, reconstruia, e o atalho seguia com o antigo. O padrao
# XDG resolve de forma definitiva: instala o PNG no tema hicolor sob um NOME, e
# `gtk-update-icon-cache` invalida o cache do tema. FONTE UNICA do icone:
# ui/assets/app-icon.png (a mesma que o AppImage empacota via ui/CMakeLists.txt).

set -eu

REPO_ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
DATA_DIR="${XDG_DATA_HOME:-$HOME/.local/share}"
APPS_DIR="$DATA_DIR/applications"
ICON_DIR="$DATA_DIR/icons/hicolor/512x512/apps"
DESKTOP_FILE="$APPS_DIR/kinein-vectis-development.desktop"
LEGACY_DESKTOP_FILE="$APPS_DIR/kinein-vectis.desktop"
LAUNCHER="$REPO_ROOT/scripts/kinein-vectis"
ICON_SOURCE="$REPO_ROOT/ui/assets/app-icon.png"
ICON_NAME="kinein-vectis-development"

if [ ! -f "$ICON_SOURCE" ]; then
    echo "erro: fonte do icone ausente: $ICON_SOURCE" >&2
    exit 1
fi

mkdir -p "$APPS_DIR" "$ICON_DIR"

# Antes do AppImage, este script usava o mesmo desktop id da distribuicao.
# Remova apenas a entrada que comprovadamente pertence a este checkout; um
# atalho do AppImage no mesmo caminho deve ser preservado.
if [ -f "$LEGACY_DESKTOP_FILE" ] \
    && grep -Fqx "Exec=$LAUNCHER" "$LEGACY_DESKTOP_FILE"; then
    rm -f "$LEGACY_DESKTOP_FILE"
fi

# Copia a fonte da verdade para o tema, sob o nome do atalho de desenvolvimento
# (distinto do `kinein-vectis` do AppImage, para os dois coexistirem).
cp -f -- "$ICON_SOURCE" "$ICON_DIR/$ICON_NAME.png"

cat > "$DESKTOP_FILE" <<EOF
[Desktop Entry]
Type=Application
Name=Kinein Vectis (Desenvolvimento)
GenericName=Ambiente de desenvolvimento da Kinein Vectis
Comment=Executa a Kinein Vectis diretamente deste checkout
Exec=$LAUNCHER
Icon=$ICON_NAME
Terminal=false
Categories=Development;IDE;
StartupNotify=true
StartupWMClass=kinein-vectis
EOF

chmod 644 "$DESKTOP_FILE"

# Invalida o cache do tema (o passo que faltava): sem isto o gnome-shell serve
# o icone antigo ate' um re-login. `-f` forca mesmo sem index.theme na pasta.
gtk-update-icon-cache -f -t "$DATA_DIR/icons/hicolor" 2>/dev/null || true
update-desktop-database "$APPS_DIR" 2>/dev/null || true

echo "atalho instalado em: $DESKTOP_FILE"
echo "icone instalado em:  $ICON_DIR/$ICON_NAME.png (fonte: ui/assets/app-icon.png)"
echo "procure por 'Kinein Vectis (Desenvolvimento)' no menu de aplicativos."
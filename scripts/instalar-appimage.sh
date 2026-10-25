#!/usr/bin/env bash
# Integra o AppImage mais recente ao menu do usuário, sem sudo.

set -Eeuo pipefail

APPIMAGE_PATTERN='Kinein-Vectis-*-x86_64.AppImage'
APPIMAGE_CHANNEL="${KINEIN_APPIMAGE_CHANNEL:-stable}"
if [[ "$APPIMAGE_CHANNEL" == development ]]; then
    APPIMAGE_PATTERN='Kinein-Vectis-*-dev*-x86_64.AppImage'
fi
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SEARCH_DIR="${1:-$SCRIPT_DIR}"
ARTIFACTS=()

if [[ -f "$SEARCH_DIR" ]]; then
    ARTIFACTS=("$(realpath "$SEARCH_DIR")")
    SEARCH_DIR="$(dirname "${ARTIFACTS[0]}")"
elif [[ -d "$SEARCH_DIR" ]]; then
    SEARCH_DIR="$(realpath "$SEARCH_DIR")"
else
    echo "erro: arquivo ou pasta não encontrado: $SEARCH_DIR" >&2
    exit 1
fi

if [[ "${#ARTIFACTS[@]}" -eq 0 ]]; then
    ARTIFACTS=()
    while IFS= read -r -d '' candidate; do
        if [[ "$APPIMAGE_CHANNEL" == stable && "$(basename "$candidate")" == *-dev* ]]; then continue; fi
        ARTIFACTS+=("$candidate")
    done < <(find "$SEARCH_DIR" -maxdepth 1 -type f -name "$APPIMAGE_PATTERN" -print0 | sort -zV)
fi

if (( ${#ARTIFACTS[@]} == 0 )); then
    echo "erro: nenhum $APPIMAGE_PATTERN encontrado em:" >&2
    echo "  $SEARCH_DIR" >&2
    exit 1
fi

LATEST="${ARTIFACTS[${#ARTIFACTS[@]} - 1]}"
chmod u+x "$LATEST"

DATA_HOME="${XDG_DATA_HOME:-$HOME/.local/share}"
APPLICATIONS_DIR="$DATA_HOME/applications"
ICON_DIR="$DATA_HOME/icons/hicolor/512x512/apps"
TEMP_DIR="$(mktemp -d)"

cleanup() {
    rm -rf "$TEMP_DIR"
}
trap cleanup EXIT

echo "==> usando a versão mais recente"
echo "$LATEST"

(
    cd "$TEMP_DIR"
    "$LATEST" --appimage-extract \
        usr/share \
        >/dev/null
)

desktop_id=io.github.viktorwalde.KineinVectis
icon_name=kinein-vectis
command_name=kinein
desktop_name=kinein-vectis
if [[ -f "$TEMP_DIR/squashfs-root/usr/share/applications/$desktop_id.Development.desktop" ]]; then
    desktop_id+=.Development
    icon_name="$desktop_id"
    command_name=kinein-dev
    desktop_name="$desktop_id"
fi
EXTRACTED_DESKTOP="$TEMP_DIR/squashfs-root/usr/share/applications/$desktop_id.desktop"
if [[ ! -f "$EXTRACTED_DESKTOP" ]]; then
    echo "erro: identidade desktop ausente do AppImage." >&2
    exit 1
fi
DESKTOP_FILE="$APPLICATIONS_DIR/$desktop_name.desktop"
ICON_FILE="$ICON_DIR/$icon_name.png"
EXTRACTED_ICON="$TEMP_DIR/squashfs-root/usr/share/icons/hicolor/512x512/apps/$icon_name.png"
if [[ ! -f "$EXTRACTED_ICON" ]]; then
    echo "erro: o AppImage não contém o ícone esperado." >&2
    exit 1
fi

install -d "$APPLICATIONS_DIR" "$ICON_DIR"
install -m 0644 "$EXTRACTED_ICON" "$ICON_FILE"

ESCAPED_EXEC="${LATEST//\\/\\\\}"
ESCAPED_EXEC="${ESCAPED_EXEC//\"/\\\"}"

# O modelo e o nome sao os do artefato; so' os caminhos locais mudam.
while IFS= read -r desktop_line; do
    case "$desktop_line" in
        Exec=*) printf 'Exec="%s" %%F\n' "$ESCAPED_EXEC" ;;
        Icon=*) printf 'Icon=%s\n' "$ICON_FILE" ;;
        *) printf '%s\n' "$desktop_line" ;;
    esac
done < "$EXTRACTED_DESKTOP" > "$DESKTOP_FILE"

chmod 0644 "$DESKTOP_FILE"
update-desktop-database "$APPLICATIONS_DIR" >/dev/null 2>&1 || true

# O COMANDO CURTO `kinein` (P0, §3 da especificacao de projetos).
#
# UM DONO SO' PARA O MOLDE: no checkout ele esta' em `scripts/kinein.in`, ao
# lado deste arquivo. No artefato entregue, o empacotador o EMBUTE entre os
# marcadores abaixo, para que um instalador baixado sozinho continue completo.
# Escrever o corpo do comando duas vezes seria a mesma derivacao em dois
# lugares, divergindo em silencio na primeira correcao que so' uma receba.
modelo_do_comando() {
    if [[ -f "$SCRIPT_DIR/kinein.in" ]]; then
        cat "$SCRIPT_DIR/kinein.in"
        return
    fi
    cat <<'FIM_DO_MODELO'
@MODELO_EMBUTIDO@
FIM_DO_MODELO
}

BIN_DIR="${XDG_BIN_HOME:-$HOME/.local/bin}"
COMANDO="$BIN_DIR/$command_name"
mkdir -p "$BIN_DIR"
# O TESTE NAO PODE CITAR O MARCADOR: ele apareceria duas vezes no arquivo, e o
# empacotador — que exige UMA ocorrencia — recusaria embutir. Modelo de
# verdade comeca por shebang; marcador nao substituido, nao.
if ! modelo_do_comando | head -n 1 | grep -q '^#!'; then
    echo "AVISO: sem o modelo do comando curto; o comando kinein nao foi instalado." >&2
else
    modelo_do_comando | sed "s|@ALVO@|$LATEST|" > "$COMANDO"
    chmod 0755 "$COMANDO"
fi

echo "==> atalho instalado/atualizado"
echo "$DESKTOP_FILE"
echo "O menu agora abre: $(basename "$LATEST")"
if [[ -x "$COMANDO" ]]; then
    echo "==> comando curto: $COMANDO"
    echo "    $command_name          abre a pasta atual"
    echo "    $command_name <pasta>  abre outra pasta"
    # PATH NAO E' GARANTIA: em varias distros `~/.local/bin` so' entra no PATH
    # se ja' existir na hora do login.
    case ":$PATH:" in
        *":$BIN_DIR:"*) ;;
        *)
            echo "    AVISO: $BIN_DIR nao esta' no seu PATH. Abra um terminal novo;" >&2
            # shellcheck disable=SC2016  # e' a linha que a pessoa vai copiar.
            echo '           se continuar fora, acrescente: export PATH="$HOME/.local/bin:$PATH"' >&2
            ;;
    esac
fi

if (( ${#ARTIFACTS[@]} > 1 )); then
    printf 'Apagar as %d versão(ões) anterior(es) desta pasta? [s/N] ' "$(( ${#ARTIFACTS[@]} - 1 ))"
    read -r REMOVE_OLD

    if [[ "$REMOVE_OLD" == "s" || "$REMOVE_OLD" == "S" ]]; then
        for old_appimage in "${ARTIFACTS[@]:0:${#ARTIFACTS[@]}-1}"; do
            rm -f -- "$old_appimage" "$old_appimage.sha256"
            echo "removido: $(basename "$old_appimage")"
        done
    else
        echo "Versões anteriores preservadas. O atalho continua apontando para a mais recente."
    fi
fi

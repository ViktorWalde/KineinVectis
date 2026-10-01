# shellcheck shell=bash
# Protocolo NAO PROVADO dos gates (2026-10-01). Par do scripts/unproven.py,
# onde o contrato inteiro esta' escrito; aqui, so' a funcao para quem e' shell.
#
# Uso:  . "$(dirname "${BASH_SOURCE[0]}")/unproven.sh"
#       record_unproven "<gate>" "<o que nao foi provado>" "<por que>"

record_unproven() {
    local gate what why
    gate="$(printf '%s' "$1" | tr '\t\n' '  ')"
    what="$(printf '%s' "$2" | tr '\t\n' '  ')"
    why="$(printf '%s' "$3" | tr '\t\n' '  ')"
    echo "  - NAO PROVADO: $what ($why)"
    if [ -n "${KINEIN_UNPROVEN_FILE:-}" ]; then
        printf '%s\t%s\t%s\n' "$gate" "$what" "$why" >>"$KINEIN_UNPROVEN_FILE"
    fi
}

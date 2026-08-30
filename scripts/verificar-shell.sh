#!/usr/bin/env bash
# ShellCheck em todos os scripts do repositorio.
#
# POR QUE ESTE SCRIPT EXISTE (2026-08-29). Os `scripts/` sao 3.708 linhas e sao
# o que EXECUTA todos os outros gates: um defeito aqui derruba a verificacao
# inteira sem que nada reclame. E ate esta data nada os verificava — o
# `instalar-ambiente.sh --extras` instalava o shellcheck e o `verificar.sh`
# nunca o chamava.
#
# A classe de erro, medida nesta mesma sessao: eu escrevi `${BASH_SOURCE[0]}`
# num script com shebang `#!/bin/sh`. No Fedora isso PASSA, porque `/bin/sh` e'
# o bash; no Debian, onde `/bin/sh` e' o dash, e' erro de sintaxe. O script
# rodava verde na maquina do autor e quebraria na de qualquer outro. Nenhum
# outro gate deste repositorio pega isso.
#
# Sem catraca de proposito: quando este gate nasceu havia 7 achados em 20
# scripts — pequeno o bastante para corrigir de uma vez. Baseline aqui seria
# maquinaria para congelar quase nada. Supressao e' permitida e SEMPRE com o
# motivo escrito ao lado (`# shellcheck disable=SCxxxx  # porque...`).
set -euo pipefail

cd "$(dirname "$0")/.." || exit 1

echo "== shellcheck (scripts do repositorio) =="

if ! command -v shellcheck >/dev/null 2>&1; then
    echo "erro: shellcheck nao encontrado." >&2
    echo "      Fedora: sudo dnf install ShellCheck" >&2
    echo "      Arch:   sudo pacman -S shellcheck" >&2
    echo "      Debian: sudo apt-get install shellcheck" >&2
    echo "      (ou rode scripts/instalar-ambiente.sh --extras)" >&2
    exit 1
fi

# Todo arquivo rastreado que E' shell: por extensao .sh ou por shebang.
alvos=""
while IFS= read -r arquivo; do
    case "$arquivo" in
        *.sh) alvos="$alvos $arquivo"; continue ;;
    esac
    [ -f "$arquivo" ] || continue
    # Pula binario antes de olhar o shebang: `head` num PNG cospe byte nulo e
    # enche a saida do gate de aviso do proprio shell.
    case "$arquivo" in
        *.png | *.jpg | *.svg | *.zip | *.pyc | *.qmltypes | *.lock) continue ;;
    esac
    case "$(head -n 1 "$arquivo" 2>/dev/null | tr -d '\0')" in
        '#!/bin/sh'* | '#!/usr/bin/env sh'* | '#!/bin/bash'* | '#!/usr/bin/env bash'*)
            alvos="$alvos $arquivo"
            ;;
    esac
done <<EOF
$(git ls-files)
EOF

if [ -z "$alvos" ]; then
    echo "erro: nenhum script encontrado — o gate nao pode passar por vazio." >&2
    exit 1
fi

# shellcheck disable=SC2086  # `alvos` e' lista separada por espaco, de propósito.
if shellcheck $alvos; then
    total=$(echo $alvos | wc -w)
    echo "shellcheck: $total scripts, nenhum achado."
else
    echo "" >&2
    echo "✗ shellcheck FALHOU" >&2
    echo "  Suprimir e' permitido, e SEMPRE com o motivo ao lado:" >&2
    echo "    # shellcheck disable=SC2086  # porque ..." >&2
    exit 1
fi

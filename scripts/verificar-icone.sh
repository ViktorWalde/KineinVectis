#!/usr/bin/env bash
# O icone da aplicacao tem UMA fonte da verdade: ui/assets/app-icon.png.
#
# POR QUE ESTE GATE EXISTE (2026-07-17, ARCHITECTURE §4 regra 11: gate nasce de
# falha SILENCIOSA). Havia DUAS copias do icone — imagens/app-icon.png e
# ui/assets/app-icon.png — e nenhum script as mantinha iguais. O atalho de
# desenvolvimento lia imagens/, o autor editava ui/assets/, e o AppImage
# empacotava ui/assets/. Trocar o icone e reconstruir nao mudava o atalho, sem
# um erro sequer: o atalho apontava para um arquivo que o autor nao estava
# tocando. Consolidada a fonte em ui/assets/, este gate impede a divergencia de
# renascer — se alguem recriar imagens/app-icon.png ou apontar um consumidor
# para fora da fonte, ele reprova.
set -euo pipefail

cd "$(dirname "$0")/.."

echo "== icone: uma fonte da verdade (ui/assets/app-icon.png) =="

FONTE="ui/assets/app-icon.png"
falhou=0

if [[ ! -f "$FONTE" ]]; then
    echo "✗ fonte da verdade ausente: $FONTE" >&2
    exit 1
fi

# A falha silenciosa e' ter DUAS COPIAS FISICAS do icone — nao uma mencao em
# texto. Um script que aponta para caminho errado quebra barulhento (arquivo nao
# encontrado); o que enganou o autor por horas foi um segundo app-icon.png real
# num caminho que ninguem lembrava. Entao a checagem e' de filesystem: so' pode
# existir UM app-icon.png rastreado, e ele e' a fonte. (build/ e' derivado; a
# copia no tema hicolor do usuario, ~/.local, nao esta no repo.)
while IFS= read -r encontrado; do
    [[ -z "$encontrado" ]] && continue
    echo "✗ segunda copia do icone: $encontrado" >&2
    echo "    A fonte e' $FONTE; qualquer outra copia rastreada vai divergir." >&2
    echo "    (imagens/ e' so' para imagens de trabalho — o icone da app nao mora la.)" >&2
    falhou=1
done < <(git ls-files '*app-icon.png' | grep -vFx "$FONTE" || true)

if ((falhou != 0)); then
    echo >&2
    echo "✗ icone FALHOU: mais de uma fonte para o mesmo icone" >&2
    exit 1
fi

echo "icone: fonte unica ui/assets/app-icon.png, sem copias concorrentes."
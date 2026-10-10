#!/usr/bin/env bash
# A CATRACA dos testes adiados no Windows (DocsPublic/roadmaps/60 §3.3, W2b).
#
# POR QUE ESTE SCRIPT EXISTE (2026-10-10). Na W2b, o teste do core que e' de um
# dominio de fatia posterior (Python na W7, Banco na W8...) ficou com
# `#[cfg_attr(windows, ignore = "W<n>: <motivo>")]`: no Windows ele aparece
# como ignorado, com o motivo e a fatia que o devolve. Sem catraca, adiar vira
# o caminho facil, e o porte "verde" seria verde por omissao. Esta catraca:
#
#   - conta os adiados por fatia e compara com a linha de base, que so' desce;
#   - reprova o adiamento sem fatia (`ignore` dentro de `cfg_attr(windows` que
#     nao comeca com `W<n>:`);
#   - conta os `#[cfg(unix)]` do codigo Rust, que tambem so' descem: um teste
#     escondido do Windows por `cfg(unix)` fugiria da contagem acima. O que e'
#     de Unix por natureza ja' esta' na linha de base, com o motivo no codigo.
#
# Quando uma fatia devolve testes, a contagem desce e a linha de base tem de
# descer junto, no mesmo commit (o script diz qual linha mudar).
set -euo pipefail

cd "$(dirname "$0")/.."
baseline="scripts/adiados-windows-baseline.txt"

echo "== adiados no Windows (catraca: so' descem) =="

failures=0

# 1. Adiamento sem fatia.
unnamed="$(grep -rn --include=*.rs -A2 'cfg_attr($' crates |
    awk '/windows,/{w=1; next} w && /ignore = "/{ if ($0 !~ /ignore = "W[0-9]+: /) print; w=0; next } {w=0}')"
if [ -n "$unnamed" ]; then
    echo "✗ adiamento sem fatia (o motivo comeca com W<n>: e diz o que falta):"
    echo "$unnamed"
    failures=1
fi

# 2. Contagem por fatia e dos cfg(unix), contra a linha de base.
measured="$(
    grep -rhoE --include=*.rs 'ignore = "W[0-9]+:' crates |
        sed 's/ignore = "//; s/://' | sort | uniq -c | awk '{print $2, $1}'
    printf 'cfg(unix) %s\n' "$(grep -rh --include=*.rs '#\[cfg(unix)\]' crates | wc -l | tr -d ' ')"
)"
expected="$(grep -v '^#' "$baseline" | grep -v '^$')"

while read -r key count; do
    allowed="$(awk -v k="$key" '$1 == k {print $2}' <<<"$expected")"
    allowed="${allowed:-0}"
    if [ "$count" -gt "$allowed" ] && [ "$key" = "cfg(unix)" ]; then
        echo "✗ $key: $count, a linha de base permite $allowed (so' o que e' de Unix por natureza, com o motivo no codigo; o resto roda no Windows ou e' adiado com fatia)"
        failures=1
    elif [ "$count" -gt "$allowed" ]; then
        echo "✗ $key: $count, a linha de base permite $allowed (adiar e' o caminho dificil: o teste roda, ou o motivo vai para o 60 §3.3)"
        failures=1
    elif [ "$count" -lt "$allowed" ]; then
        echo "✗ $key: $count, abaixo da linha de base ($allowed); desca a linha '$key' em $baseline"
        failures=1
    fi
done <<<"$measured"

while read -r key allowed; do
    if ! awk -v k="$key" '$1 == k {found=1} END {exit !found}' <<<"$measured" && [ "$allowed" -gt 0 ]; then
        echo "✗ $key: 0, abaixo da linha de base ($allowed); desca a linha '$key' em $baseline"
        failures=1
    fi
done <<<"$expected"

if [ "$failures" -ne 0 ]; then
    exit 1
fi
echo "✓ adiados no Windows: $(awk '$1 ~ /^W/ {s += $2} END {print s+0}' <<<"$measured") testes, por fatia, e os cfg(unix), iguais a linha de base"

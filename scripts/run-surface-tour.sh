#!/usr/bin/env bash
# G0.4 (roadmap 53 §0.2 e §5.2): o passeio por superficies, com o dono de cada aviso.
#
# Abre o LANCADOR num projeto de teste (com mudancas git em mais de uma pasta,
# para a lista do Git ter secoes — foi ai' que o "Component is not ready" do
# Qt 6.4 se escondia), roda scripts/surface-tour.txt e reprova:
#   - qualquer linha de scripts/avisos-qml.txt, nomeando o passo que a produziu;
#   - id do roteiro que ninguem trata (erro de digitacao vira passeio menor);
#   - passeio que nao chega ao `@quit` ou IDE que nao sai com 0.
#
# Em shell de proposito: o testar-appimage.sh roda no Debian minimo, sem Python.
# O ambiente do lancador (APPIMAGE_EXTRACT_AND_RUN, KINEIN_CORE_BIN) e' de quem
# chama; aqui so' entram o offscreen e os XDG isolados.
#
# Uso: bash scripts/run-surface-tour.sh LANCADOR [LOG]
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
launcher="${1:?uso: run-surface-tour.sh LANCADOR [LOG]}"
work_dir="$(mktemp -d)"
trap 'rm -rf "$work_dir"' EXIT
log_file="${2:-$work_dir/tour.log}"

tour_list="$(grep -v '^[[:space:]]*#' "$repo_root/scripts/surface-tour.txt" | sed '/^[[:space:]]*$/d')"
step_count="$(printf '%s\n' "$tour_list" | grep -vc '^@passo=')"
startup_commands="$(printf '%s\n' "$tour_list" | paste -sd, -)"
step_ms="$(printf '%s\n' "$tour_list" | sed -n 's/^@passo=//p')"
deadline_seconds=$(( step_count * step_ms / 1000 + 90 ))

# O projeto do passeio: tres pastas com mudanca (raiz, src, docs) quando ha'
# git; sem git (Debian minimo) o painel do Git mostra o proprio estado de
# ausencia, que tambem e' superficie.
project="$work_dir/project"
mkdir -p "$project/src" "$project/docs"
printf 'int main() { return 0; }\n' > "$project/src/main.cpp"
printf '#pragma once\n' > "$project/src/util.h"
printf '# Projeto do passeio\n' > "$project/docs/readme.md"
printf 'build/\n' > "$project/.gitignore"
if command -v git >/dev/null 2>&1; then
    git -C "$project" init -q
    git -C "$project" -c user.name=tour -c user.email=tour@localhost add -A
    git -C "$project" -c user.name=tour -c user.email=tour@localhost commit -q -m inicial
    printf 'int main() { return 1; }\n' > "$project/src/main.cpp"
    printf 'novo\n' > "$project/docs/new.md"
    printf 'build/\ndist/\n' > "$project/.gitignore"
fi

# XDG_RUNTIME_DIR tambem (0700, como o logind cria): sem ele o Qt avisa
# "XDG_RUNTIME_DIR not set" — num container, ssh sem sessao ou CI, onde a IDE
# nao tem defeito. O aviso nao esta' em avisos-qml.txt hoje, mas o passeio nao
# pode depender da sessao de quem o roda (mesma correcao do G0.5, 2026-10-01).
for name in config data cache state runtime; do
    mkdir -p "$work_dir/xdg-$name"
done
chmod 700 "$work_dir/xdg-runtime"

echo "== passeio por superficies: $step_count passos de ${step_ms} ms ($(basename "$launcher")) =="
status=0
timeout "$deadline_seconds" env \
    QT_QPA_PLATFORM=offscreen \
    QT_FORCE_STDERR_LOGGING=1 \
    XDG_CONFIG_HOME="$work_dir/xdg-config" \
    XDG_DATA_HOME="$work_dir/xdg-data" \
    XDG_CACHE_HOME="$work_dir/xdg-cache" \
    XDG_STATE_HOME="$work_dir/xdg-state" \
    XDG_RUNTIME_DIR="$work_dir/xdg-runtime" \
    KINEIN_STARTUP_COMMANDS="$startup_commands" \
    "$launcher" "$project" >"$log_file" 2>&1 || status=$?

failed=0
if [[ "$status" -ne 0 ]]; then
    tail -n 40 "$log_file" >&2
    echo "✗ a IDE nao saiu com 0 no fim do passeio (saida $status; 124 = estourou ${deadline_seconds} s)" >&2
    failed=1
fi

ran_steps="$(grep -c 'KINEIN_PASSEIO passo=' "$log_file" || true)"
if [[ "$ran_steps" -ne "$step_count" ]]; then
    echo "✗ o passeio rodou $ran_steps de $step_count passos" >&2
    failed=1
fi

if grep -F 'KINEIN_STARTUP_COMMANDS: ninguem trata' "$log_file" >&2; then
    echo "✗ o roteiro tem id que ninguem trata (scripts/surface-tour.txt)" >&2
    failed=1
fi

# Cada aviso com o passo corrente: "antes do passeio" e' a abertura.
if grep -F -q -f "$repo_root/scripts/avisos-qml.txt" "$log_file"; then
    echo "✗ aviso do motor QML durante o passeio (lista em scripts/avisos-qml.txt):" >&2
    awk -v patterns="$repo_root/scripts/avisos-qml.txt" '
        BEGIN { step = "antes do passeio"; while ((getline line < patterns) > 0) if (line != "") list[line] = 1 }
        /KINEIN_PASSEIO passo=/ { sub(/.*passo=/, ""); step = $0; next }
        { for (p in list) if (index($0, p) > 0) { print "  [" step "] " $0; break } }
    ' "$log_file" | sort | uniq -c | sort -rn | head -n 40 >&2
    failed=1
fi

if [[ "$failed" -ne 0 ]]; then
    exit 1
fi
echo "✓ passeio: $step_count passos, nenhum aviso, saida limpa"

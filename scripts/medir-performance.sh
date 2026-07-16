#!/usr/bin/env bash
# Medicao de performance LOCAL da Kinein Vectis (fatia M4.2 de docs/diario/18).
#
# ZERO telemetria/rede: mede via stdio (core) e /proc, imprime uma tabela e
# sai. Roda quando o dev quiser; o orcamento (numeros-alvo) vive no docs/roadmaps/21.
# Cada metrica roda N vezes (KINEIN_PERF_N, default 5) e reporta a MEDIANA.
#
# Metricas:
#   A. UI: time-to-first-frame (offscreen, marker KINEIN_PERF da main.cpp)
#   B. Core: workspace.open no proprio repo (o "workspace grande")
#   C. Core: fs.read de um .txt de 10k linhas (proxy de abrir arquivo grande)
#   D. RSS: UI em boot vazio; core em regime + LSP Rust vivo (best-effort)
#
# Uso: scripts/medir-performance.sh   (nada de argumentos)

set -eu

REPO_ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
N="${KINEIN_PERF_N:-5}"

UI_BIN="${KINEIN_UI_BIN:-}"
if [ -z "$UI_BIN" ]; then
    for c in "$REPO_ROOT/build/dev-local/ui/kinein-vectis" \
             "$REPO_ROOT/build/linux-clang-debug-strict/ui/kinein-vectis"; do
        [ -x "$c" ] && UI_BIN="$c" && break
    done
fi
CORE_BIN="${KINEIN_CORE_BIN:-$REPO_ROOT/target/debug/kinein-core}"

if [ -z "$UI_BIN" ] || [ ! -x "$UI_BIN" ]; then
    echo "erro: binario da UI nao encontrado. Compile: cmake --build build/dev-local" >&2
    exit 1
fi
if [ ! -x "$CORE_BIN" ]; then
    echo "erro: kinein-core nao encontrado. Compile: cargo build -p kinein-core" >&2
    exit 1
fi

# Mediana de uma lista de inteiros passada como argumentos.
mediana() {
    printf '%s\n' "$@" | sort -n | awk '{ v[NR]=$1 } END {
        if (NR==0) { print "0"; exit }
        if (NR%2) print v[(NR+1)/2]; else printf "%d\n", (v[NR/2]+v[NR/2+1])/2
    }'
}

echo "== Kinein Vectis — medicao de performance (N=$N, local, sem rede) =="
echo "   maquina: $(grep -m1 'model name' /proc/cpuinfo | sed 's/.*: //') / $(nproc) threads"
echo

# --- A. UI: time-to-first-frame (offscreen) ---
echo "-- A. UI startup (time-to-first-frame, offscreen) --"
ttf=()
for _ in $(seq 1 "$N"); do
    ms="$(QT_QPA_PLATFORM=offscreen QT_FORCE_STDERR_LOGGING=1 \
          KINEIN_PERF_MARKER=1 KINEIN_PERF_EXIT=1 \
          timeout 30 "$UI_BIN" 2>&1 |
          sed -n 's/.*first_frame_ms=\([0-9]\{1,\}\).*/\1/p' | head -n1)"
    [ -n "$ms" ] && ttf+=("$ms") && printf '   run: %s ms\n' "$ms"
done
ttf_med="$([ ${#ttf[@]} -gt 0 ] && mediana "${ttf[@]}" || echo n/d)"
echo "   -> mediana TTF: ${ttf_med} ms"
echo

# --- D (UI). RSS em boot vazio ---
echo "-- D1. UI RSS (boot vazio, offscreen, apos settle) --"
QT_QPA_PLATFORM=offscreen KINEIN_PERF_MARKER=1 "$UI_BIN" >/dev/null 2>&1 &
ui_pid=$!
sleep 3
ui_rss_kb="$(awk '/VmRSS/{print $2}' "/proc/$ui_pid/status" 2>/dev/null || echo 0)"
kill "$ui_pid" 2>/dev/null || true
wait "$ui_pid" 2>/dev/null || true
echo "   -> UI VmRSS: $((ui_rss_kb / 1024)) MB"
echo

# --- B, C, D(core+LSP) via stdio ---
# Qual binario foi medido NAO pode ficar implicito: o default de CORE_BIN e o
# build DEBUG, e Rust sem otimizacao e ~34x mais lento no Tree-sitter (11 s vs
# 323 ms na mesma fixture). Numero de perf sem o binario ao lado nao significa
# nada — e o release que o usuario roda.
echo "-- binarios medidos --"
echo "   UI  : $UI_BIN"
echo "   CORE: $CORE_BIN"
case "$CORE_BIN" in
    *release*) ;;
    *) echo "   AVISO: core NAO-release. Os numeros nao sao comparaveis ao orcamento." >&2 ;;
esac
echo
echo "-- B/C/D2/A3.1/A3.3. Core (stdio: workspace.open, fs.read, RSS+LSP, sintaxe, rajada) --"
core_out="$(python3 "$REPO_ROOT/scripts/medir-core.py" "$CORE_BIN" "$REPO_ROOT" "$N")"
echo "$core_out" | sed 's/^/   /'
echo

echo "== fim. Compare com o ORCAMENTO em docs/roadmaps/21 (M4.2). =="

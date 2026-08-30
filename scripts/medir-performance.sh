#!/usr/bin/env bash
# Medicao de performance LOCAL da Kinein Vectis (fatia M4.2 de docs-privada/diario/18).
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
#   A3.1. Tree-sitter: primeiro snapshot frio e update incremental
#   A3.2. LSP: primeira resposta util por servidor (rust-analyzer, clangd)
#   A3.3. Terminal: rajada do PTY, input -> frame com marcador
#   A3.3-1. Editor: digitacao tecla -> frame, no editor real com arquivo grande
#
# A3.4: a saida comeca por um CARIMBO de ambiente. Numero de performance sem
# maquina, distro, Qt, binario e N ao lado nao e comparavel com nada — nao da
# para dizer se um numero maior amanha e regressao ou so outra maquina.
#
# Uso: KINEIN_CORE_BIN=.../target/release/kinein-core scripts/medir-performance.sh

set -eu

REPO_ROOT="$(unset CDPATH; cd -- "$(dirname -- "$0")/.." && pwd)"
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

# Percentil de uma lista que aceita DECIMAIS (as amostras de digitacao vem com
# casas). Mesma definicao do `percentil()` do medir-core.py, de proposito: dois
# p95 calculados de formas diferentes no mesmo relatorio nao se comparam.
# Uso: percentil <p> <amostra>...
percentil() {
    local p="$1"
    shift
    printf '%s\n' "$@" | sort -g | awk -v p="$p" '{ v[NR]=$1 } END {
        if (NR==0) { print "n/d"; exit }
        idx = int((p / 100.0) * (NR - 1) + 0.5) + 1
        if (idx < 1) idx = 1
        if (idx > NR) idx = NR
        printf "%.1f\n", v[idx]
    }'
}

echo "== Kinein Vectis — medicao de performance (N=$N, local, sem rede) =="

# --- A3.4. Carimbo de ambiente: o que torna a serie comparavel ---
echo "== CARIMBO (A3.4) — colar junto com qualquer numero reportado =="
echo "   data      : $(date -Iseconds)"
echo "   cpu       : $(awk -F: '/model name/{gsub(/^ +/,"",$2); print $2; exit}' /proc/cpuinfo)"
echo "   nucleos   : $(nproc)"
echo "   ram_gb    : $(awk '/MemTotal/{printf "%.1f", $2/1048576}' /proc/meminfo)"
# /etc/os-release e' do sistema, nao do repositorio: nao ha o que seguir.
# shellcheck disable=SC1091
echo "   distro    : $(. /etc/os-release 2>/dev/null && echo "$PRETTY_NAME" || echo desconhecida)"
echo "   kernel    : $(uname -r)"
echo "   qt        : $( (qmake6 -query QT_VERSION 2>/dev/null || qmake -query QT_VERSION 2>/dev/null) || echo n/d)"
echo "   sessao    : ${XDG_SESSION_TYPE:-n/d}"
echo "   N         : $N"
echo "   git       : $(git -C "$REPO_ROOT" rev-parse --short HEAD 2>/dev/null || echo n/d)$( git -C "$REPO_ROOT" diff --quiet 2>/dev/null || echo '+sujo' )"
echo "   UI  : $UI_BIN"
echo "   CORE: $CORE_BIN"
case "$CORE_BIN" in
    *release*) ;;
    *) echo "   AVISO: core NAO-release. Os numeros nao sao comparaveis ao orcamento." >&2 ;;
esac
echo
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

# --- A3.3 item 1. UI: digitacao tecla -> frame no editor real ---
# Roda no processo REAL (harness atras de KINEIN_PERF_TYPING), porque o runner
# `qml` nao carrega o modulo KineinVectis: o qmldir gerado aponta para caminhos
# qrc:, que so existem dentro do binario. Evidencia em docs/roadmaps/21.
#
# Offscreen NAO tem vsync. O numero e o custo PROPRIO da Kinein da tecla ao
# frame — piso do que o usuario sente num compositor a 60 Hz, nao o total.
# E o que precisa ser: comparavel entre runs e sem depender de monitor.
echo "-- A3.3-1. UI: digitacao tecla->frame (editor real, offscreen) --"
typing_ws="$(mktemp -d -t kinein-typing-XXXXXX)"
python3 "$REPO_ROOT/scripts/medir-core.py" --emit-fixture "$typing_ws" | sed 's/^/   /'
typing_out="$(QT_QPA_PLATFORM=offscreen QT_FORCE_STDERR_LOGGING=1 \
    KINEIN_CORE_BIN="$CORE_BIN" \
    KINEIN_PERF_TYPING=1 \
    KINEIN_PERF_TYPING_WORKSPACE="$typing_ws" \
    KINEIN_PERF_TYPING_FILE="$typing_ws/fixture.rs" \
    timeout 180 "$UI_BIN" 2>&1 || true)"
rm -rf "$typing_ws"

typing_err="$(printf '%s\n' "$typing_out" | sed -n 's/.*typing_error=\(.*\)/\1/p' | head -n1)"
if [ -n "$typing_err" ]; then
    echo "   -> n/d: $typing_err"
else
    mapfile -t typing_amostras < <(printf '%s\n' "$typing_out" |
        sed -n 's/.*typing_sample_ms=\([0-9.]\{1,\}\).*/\1/p')
    typing_linhas="$(printf '%s\n' "$typing_out" | sed -n 's/.*typing_file_lines=\([0-9]\{1,\}\).*/\1/p' | head -n1)"
    typing_inseridos="$(printf '%s\n' "$typing_out" | sed -n 's/.*typing_chars_inserted=\([0-9]\{1,\}\).*/\1/p' | head -n1)"
    if [ "${#typing_amostras[@]}" -eq 0 ]; then
        echo "   -> n/d: nenhuma amostra"
    else
        echo "   fixture  : ${typing_linhas} linhas (a MESMA do A3.1)"
        # Prova de que as teclas entraram: sem isto o harness poderia cronometrar
        # frames que tecla nenhuma causou e reportar um numero lindo.
        echo "   teclas   : ${#typing_amostras[@]} enviadas, ${typing_inseridos} caracteres inseridos"
        echo "   -> mediana tecla->frame: $(percentil 50 "${typing_amostras[@]}") ms"
        echo "   -> p95    tecla->frame: $(percentil 95 "${typing_amostras[@]}") ms"
        # O PIOR caso vai junto de proposito. Com 40 amostras a p95 cai na 38a e
        # descarta as duas piores — exatamente a travada que o roadmap manda
        # nao perder. A primeira tecla depois do arquivo carregar e sempre a mais
        # cara (caminho frio), e e uma tecla que o usuario de fato digita.
        echo "   -> pior   tecla->frame: $(percentil 100 "${typing_amostras[@]}") ms"
    fi
fi
echo

# --- B, C, D(core+LSP) via stdio ---
# Qual binario foi medido NAO pode ficar implicito: o default de CORE_BIN e o
# build DEBUG, e Rust sem otimizacao e ~34x mais lento no Tree-sitter (11 s vs
# 323 ms na mesma fixture). Numero de perf sem o binario ao lado nao significa
# nada — e o release que o usuario roda.
echo "-- B/C/D2/A3.1/A3.3. Core (stdio: workspace.open, fs.read, RSS+LSP, sintaxe, rajada) --"
core_out="$(python3 "$REPO_ROOT/scripts/medir-core.py" "$CORE_BIN" "$REPO_ROOT" "$N")"
# Indenta uma saida MULTI-LINHA; `${var//x/y}` nao ancora por linha.
# shellcheck disable=SC2001
echo "$core_out" | sed 's/^/   /'
echo

echo "== fim. Compare com o ORCAMENTO em docs/roadmaps/21 (M4.2 e A3.1-A3.4). =="
echo
echo "A3.4 — reacao a regressao (nao e sugestao, e o gate):"
echo "  1. numero acima do orcamento ABRE fatia de causa-raiz. Nao se aprofunda"
echo "     semantica nem se abre nivel novo com orcamento estourado."
echo "  2. antes de otimizar, PERFILAR: so mexe depois de perfil local apontar o"
echo "     dono do custo. A3.1 ja mostrou por que — o gargalo do Tree-sitter nao"
echo "     era o parser, era o payload de 1138 KB por tecla."
echo "  3. conferir o carimbo acima antes de gritar regressao: maquina, binario"
echo "     (release?) e N diferentes explicam mais desvio que codigo."

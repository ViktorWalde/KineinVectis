#!/usr/bin/env bash
# F0 da 0.3.6 (roadmap 53 §5.3 e §10): as telas da IDE nas tres larguras, de
# forma reproduzivel — o "antes" com que cada fatia da 0.3.6 a 0.3.9 se compara.
#
# Cada cena abre o binario num Xvfb do tamanho exato (sem gerenciador de janela
# nem notificacao), num projeto de teste montado aqui e com XDG isolados, roda
# os comandos da cena pelo KINEIN_STARTUP_COMMANDS e fotografa a janela pelo
# hook KINEIN_SCREENSHOT. Mesmo binario, mesmo projeto, mesma cena: mesma foto.
#
# Cenas (nome = comandos da paleta, em ordem):
#   editor     projeto aberto, duas abas restauradas pela sessao, main.cpp ativa
#   inicio     tela inicial (workspace.close)
#   git        painel do Git com mudancas em tres pastas
#   terminal   painel de baixo no Terminal
#   busca      busca no projeto
#   paleta     paleta de comandos
#   remoto     painel de ambiente do Remote (overlay do trilho)
#   criar      "Criar Projeto": o seletor no modo de criar, linguagem por escolher
#   abrir      "Abrir projeto": o seletor no modo de abrir
#
# Uso: bash scripts/capturar-telas.sh [BINARIO] [PASTA_DE_SAIDA]
#   BINARIO         padrao build/dev-local/ui/kinein-vectis
#   KINEIN_CORE_BIN padrao target/release/kinein-core, compilado aqui antes: a UI
#                   procura o core em target/debug pelo diretorio corrente, e um
#                   core velho ali fotografa a versao errada (achado em 2026-10-01)
#   PASTA_DE_SAIDA  padrao build/telas; grava <cena>-<largura>x<altura>.png
#   KINEIN_TELAS_CENAS="editor git"  restringe as cenas
#   KINEIN_TELAS_TAMANHOS="1366x768" restringe os tamanhos
#   KINEIN_TELAS_ESPERA_MS=8000      espera depois do primeiro quadro
#
# A FOTO DEPENDE DA CARGA DA MAQUINA: com o gate rodando ao lado (o clang-tidy
# ocupa todos os nucleos), a deteccao de ferramentas chegou depois da foto e o
# trilho saiu sem o Containers (2026-10-02). Por isso a espera e' de 8 s e o
# script avisa quando a carga passa da metade dos nucleos.
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
binary="${1:-$repo_root/build/dev-local/ui/kinein-vectis}"
out_dir="${2:-$repo_root/build/telas}"
sizes="${KINEIN_TELAS_TAMANHOS:-1024x700 1366x768 1920x1080}"
delay_ms="${KINEIN_TELAS_ESPERA_MS:-8000}"
scenes="${KINEIN_TELAS_CENAS:-editor inicio git terminal busca paleta remoto criar abrir}"

if [[ ! -x "$binary" ]]; then
    echo "erro: binario nao encontrado: $binary (compile: cmake --build build/dev-local)" >&2
    exit 1
fi
core_binary="${KINEIN_CORE_BIN:-}"
if [[ -z "$core_binary" ]]; then
    cargo build -q --release -p kinein-core --manifest-path "$repo_root/Cargo.toml"
    core_binary="$repo_root/target/release/kinein-core"
fi
if ! command -v xvfb-run >/dev/null 2>&1; then
    echo "erro: xvfb-run ausente (Ubuntu: sudo apt install xvfb)" >&2
    exit 1
fi

scene_commands() {
    case "$1" in
        editor) echo "" ;;
        inicio) echo "workspace.close" ;;
        git) echo "git.commit" ;;
        terminal) echo "terminal.open" ;;
        busca) echo "fs.search" ;;
        paleta) echo "command.list" ;;
        remoto) echo "remote.list" ;;
        criar) echo "workspace.createProject" ;;
        abrir) echo "workspace.open" ;;
        *) return 1 ;;
    esac
}

work_dir="$(mktemp -d)"
trap 'rm -rf "$work_dir"' EXIT
mkdir -p "$out_dir"

# O projeto mora num caminho FIXO: o caminho aparece na barra de status, e um
# temporario aleatorio faria duas fotos da mesma cena diferirem. E FORA do
# repositorio: dentro dele, o clangd herdava o .clangd da propria IDE e as
# fotos mostravam diagnosticos que nao sao do projeto (achado em 2026-10-01).
project="${TMPDIR:-/tmp}/kinein-telas/projeto"
fake_home="${TMPDIR:-/tmp}/kinein-telas/home"

# O mesmo projeto do passeio (run-surface-tour.sh): mudancas git em tres pastas,
# para o painel do Git ter secoes. A sessao abre duas abas, como no uso real, e
# o CMakeLists.txt faz dele um projeto C++ detectado, o caso comum.
make_project() {
    rm -rf "$project"
    mkdir -p "$project/src" "$project/docs" "$project/.kinein"
    cat >"$project/CMakeLists.txt" <<'EOF'
cmake_minimum_required(VERSION 3.20)
project(telas CXX)
add_executable(telas src/main.cpp)
EOF
    cat >"$project/src/main.cpp" <<'EOF'
#include "util.h"

#include <cstdio>

int main()
{
    const int answer = twice(21);
    std::printf("%d\n", answer);
    return 0;
}
EOF
    cat >"$project/src/util.h" <<'EOF'
#pragma once

inline int twice(int value)
{
    return value * 2;
}
EOF
    printf '# Projeto das telas\n' >"$project/docs/readme.md"
    printf 'build/\n' >"$project/.gitignore"
    git -C "$project" init -q
    git -C "$project" -c user.name=telas -c user.email=telas@localhost add -A
    git -C "$project" -c user.name=telas -c user.email=telas@localhost commit -q -m inicial
    printf 'build/\ndist/\n' >"$project/.gitignore"
    printf 'novo\n' >"$project/docs/new.md"
    sed -i 's/twice(21)/twice(22)/' "$project/src/main.cpp"
    cat >"$project/.kinein/session.json" <<'EOF'
{"schemaVersion": 1, "openFiles": ["src/util.h", "src/main.cpp"], "activeFile": "src/main.cpp"}
EOF
}

load="$(cut -d' ' -f1 /proc/loadavg)"
if awk -v load="$load" -v cores="$(nproc)" 'BEGIN { exit !(load > cores / 2) }'; then
    echo "aviso: carga $load com $(nproc) nucleos — as fotos podem sair antes de a IDE assentar" >&2
fi
rustup_home="${RUSTUP_HOME:-$HOME/.rustup}"
cargo_home="${CARGO_HOME:-$HOME/.cargo}"
failed=0
for scene in $scenes; do
    if ! commands="$(scene_commands "$scene")"; then
        echo "erro: cena desconhecida: $scene" >&2
        exit 1
    fi
    for size in $sizes; do
        run_dir="$work_dir/$scene-$size"
        mkdir -p "$run_dir"
        make_project
        for name in config data cache state runtime; do
            mkdir -p "$run_dir/xdg-$name"
        done
        chmod 700 "$run_dir/xdg-runtime"
        # HOME de teste: o seletor de pasta abre na home, e a foto mostrava a
        # home REAL de quem rodou (achado em 2026-10-01). O rustup e o cargo
        # continuam onde estao.
        # Caminho fixo pelo mesmo motivo do projeto: ele aparece na foto.
        rm -rf "$fake_home"
        mkdir -p "$fake_home/Projetos" "$fake_home/Documentos"
        shot="$out_dir/$scene-$size.png"
        rm -f "$shot"
        startup=""
        if [[ -n "$commands" ]]; then
            startup="@passo=700,$commands"
        fi
        status=0
        timeout 60 env \
            HOME="$fake_home" \
            RUSTUP_HOME="$rustup_home" \
            CARGO_HOME="$cargo_home" \
            XDG_CONFIG_HOME="$run_dir/xdg-config" \
            XDG_DATA_HOME="$run_dir/xdg-data" \
            XDG_CACHE_HOME="$run_dir/xdg-cache" \
            XDG_STATE_HOME="$run_dir/xdg-state" \
            XDG_RUNTIME_DIR="$run_dir/xdg-runtime" \
            QT_QPA_PLATFORM=xcb \
            KINEIN_CORE_BIN="$core_binary" \
            KINEIN_STARTUP_COMMANDS="$startup" \
            KINEIN_SCREENSHOT="$shot" \
            KINEIN_SCREENSHOT_SIZE="$size" \
            KINEIN_SCREENSHOT_DELAY_MS="$delay_ms" \
            KINEIN_PERF_EXIT=1 \
            xvfb-run -a -s "-screen 0 ${size}x24 -dpi 96 -nolisten tcp" \
            "$binary" --wait "$project" >"$run_dir/log" 2>&1 || status=$?
        if [[ "$status" -ne 0 || ! -s "$shot" ]]; then
            tail -n 20 "$run_dir/log" >&2
            echo "✗ $scene $size: sem foto (saida $status)" >&2
            failed=1
            continue
        fi
        echo "✓ $shot"
    done
done
rm -rf "$(dirname "$project")"
exit "$failed"

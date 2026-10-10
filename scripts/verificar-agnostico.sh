#!/usr/bin/env bash
# O LADO AGNOSTICO do gate do Windows (DocsPublic/roadmaps/60 §3.1, D8).
#
# POR QUE ESTE SCRIPT EXISTE (2026-10-09). O desenvolvimento passou para o
# Windows 11 Pro, e o `scripts/verificar-windows.ps1` roda nativo la' tudo o
# que depende do sistema: build, testes, clippy, C++, smoke. O que NAO depende
# do sistema — documentacao, links, mapa de modulos, catraca de arquitetura,
# shellcheck, as verificacoes de texto do QML e da fiacao — continua em bash, e
# roda aqui, no WSL, sobre o PROPRIO checkout do Windows (`/mnt/c/...`): nao ha'
# copia para sincronizar, entao nao ha' copia velha para enganar.
#
# Fora daqui, de proposito: o `check_identifier_language.py`. O veredito dele
# depende do dicionario do sistema — o do Fedora reprova o mesmo codigo que o
# do Arch aprova (medido em 2026-10-09, roadmaps/40.7 §7.262). O gate completo
# do Linux continua sendo o `verificar.sh` (`verificar-windows.ps1 -Linux`).
set -uo pipefail

cd "$(dirname "$0")/.." || exit 1

# O git do WSL recusa um repositorio de outro dono, e o checkout e' do usuario
# do Windows. A excecao vale so' para este caminho.
git config --global --get-all safe.directory 2>/dev/null | grep -qxF "$PWD" ||
    git config --global --add safe.directory "$PWD"

# O ambiente do rustup (o mapa de modulos chama `cargo metadata`).
if [ -f "$HOME/.cargo/env" ]; then
    # shellcheck disable=SC1091  # arquivo do rustup, fora do repositorio
    . "$HOME/.cargo/env"
fi

falhas=()

passo() {
    echo "== $* =="
    if "$@"; then
        echo "ok"
    else
        falhas+=("$*")
        echo "FALHOU: $*"
    fi
}

passo bash scripts/verificar-docs.sh
passo bash scripts/verificar-links-docs.sh
passo python3 scripts/module_map.py --check
passo bash scripts/verificar-arquitetura.sh
passo bash scripts/verificar-shell.sh
passo bash scripts/verificar-qml-fiacao.sh
passo bash scripts/verificar-qml-propriedades.sh
passo bash scripts/verificar-qml-mortas.sh
passo bash scripts/verificar-qml-duplicacao.sh
passo bash scripts/verificar-qml-tokens.sh
passo bash scripts/verificar-qml-alcance.sh
passo bash scripts/verificar-fiacao-ipc.sh
passo bash scripts/verificar-atalhos.sh

if [ "${#falhas[@]}" -gt 0 ]; then
    printf '✗ agnostico FALHOU em: %s\n' "${falhas[@]}"
    exit 1
fi
echo "✓ agnostico: tudo verde"

#!/usr/bin/env bash
# O LADO AGNOSTICO do gate do Windows (DocsPublic/roadmaps/60 §3.1, D8).
#
# POR QUE ESTE SCRIPT EXISTE (2026-10-09). O desenvolvimento passou para o
# Windows 11 Pro, e o `scripts/verificar-windows.ps1` roda nativo la' tudo o
# que depende do sistema: build, testes, clippy, C++, smoke. O que NAO depende
# do sistema — documentacao, links, mapa de modulos, catraca de arquitetura,
# o lint dos scripts, as verificacoes de texto do QML e da fiacao — continua em
# bash, e roda aqui, no WSL, sobre o PROPRIO checkout do Windows (`/mnt/c/...`):
# nao ha' copia para sincronizar, entao nao ha' copia velha para enganar.
#
# O `check_identifier_language.py` ficou de fora ate' 2026-10-09 (roadmaps/40.7
# §7.262): o veredito dele dependia do dicionario do sistema, e o do Fedora nao
# traz termos que o do Arch traz (`sqlite`, `stdio`, `linux`...). A allowlist
# passou a listar esses termos (40.7 §7.269), e o veredito ficou o mesmo nas
# duas distros. O gate completo do Linux continua sendo o `verificar.sh`
# (`verificar-windows.ps1 -Linux`).
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

failures=()

step() {
    echo "== $* =="
    if "$@"; then
        echo "ok"
    else
        failures+=("$*")
        echo "FALHOU: $*"
    fi
}

step bash scripts/verificar-docs.sh
step bash scripts/verificar-links-docs.sh
step python3 scripts/module_map.py --check
step bash scripts/verificar-arquitetura.sh
step bash scripts/verificar-adiados-windows.sh
step python3 scripts/check_identifier_language.py
step bash scripts/verificar-shell.sh
step bash scripts/verificar-qml-fiacao.sh
step bash scripts/verificar-qml-propriedades.sh
step bash scripts/verificar-qml-mortas.sh
step bash scripts/verificar-qml-duplicacao.sh
step bash scripts/verificar-qml-tokens.sh
step bash scripts/verificar-qml-alcance.sh
step bash scripts/verificar-fiacao-ipc.sh
step bash scripts/verificar-atalhos.sh

if [ "${#failures[@]}" -gt 0 ]; then
    printf '✗ agnostico FALHOU em: %s\n' "${failures[@]}"
    exit 1
fi
echo "✓ agnostico: tudo verde"

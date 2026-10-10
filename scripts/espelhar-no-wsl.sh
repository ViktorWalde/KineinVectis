#!/usr/bin/env bash
# O ESPELHO do checkout do Windows dentro do WSL, para o gate completo do
# Linux (DocsPublic/roadmaps/60 §3.1, D2 e D8).
#
# POR QUE ESTE SCRIPT EXISTE (2026-10-09). O Linux continua sendo a
# referencia: o `verificar.sh` inteiro (build, testes, clippy, C++, QML) tem de
# passar nele a cada fatia. Rodar o build Linux sobre `/mnt/c` seria lento e
# misturaria os `build/` dos dois sistemas; por isso existe um espelho, numa
# pasta SO' DELE, que recebe o estado do Windows: o branch, o diff ainda nao
# commitado e os arquivos novos.
#
# A PASTA E' DEDICADA DE PROPOSITO. A sincronizacao descarta o que estiver no
# espelho (`reset --hard`), e por isso ele nunca e' um checkout onde alguem
# trabalha: o padrao fica em `~/.local/share/kinein-vectis/espelho-windows`.
#
# Uso (dentro do WSL):
#   scripts/espelhar-no-wsl.sh            # sincroniza e roda verificar.sh --rapido
#   scripts/espelhar-no-wsl.sh --so-espelhar
set -euo pipefail

checkout="$(cd "$(dirname "$0")/.." && pwd)"
mirror="${KINEIN_MIRROR:-$HOME/.local/share/kinein-vectis/espelho-windows}"

git config --global --get-all safe.directory 2>/dev/null | grep -qxF "$checkout" ||
    git config --global --add safe.directory "$checkout"

branch="$(git -C "$checkout" branch --show-current)"
first_run=0
if [ ! -d "$mirror/.git" ]; then
    mkdir -p "$(dirname "$mirror")"
    git clone -q "$checkout" "$mirror"
    first_run=1
fi

cd "$mirror"
git reset -q --hard
git clean -q -fd -- crates ui scripts DocsPublic
git fetch -q "$checkout" "$branch"
git switch -q -C "$branch" FETCH_HEAD
# O diff nao commitado; `filemode=false` porque no `/mnt/c` todo arquivo
# aparece com permissao 777.
git -c core.filemode=false -C "$checkout" diff --binary HEAD | git apply --allow-empty
git -c core.filemode=false -C "$checkout" ls-files --others --exclude-standard -z |
    while IFS= read -r -d '' file; do
        mkdir -p "$(dirname "$file")"
        cp "$checkout/$file" "$file"
    done
echo "espelho: $mirror no $branch @ $(git rev-parse --short HEAD)"

if [ "${1:-}" = "--so-espelhar" ]; then
    exit 0
fi
if [ -f "$HOME/.cargo/env" ]; then
    # shellcheck disable=SC1091  # arquivo do rustup, fora do repositorio
    . "$HOME/.cargo/env"
fi
# Na primeira vez o espelho ainda nao tem os presets locais nem os build dirs.
if [ "$first_run" -eq 1 ]; then
    ./scripts/instalar-ambiente.sh
fi
exec ./scripts/verificar.sh --rapido

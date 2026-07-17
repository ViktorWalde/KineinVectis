#!/usr/bin/env bash
# Dois presets nao podem escrever no MESMO diretorio de build.
#
# POR QUE ESTE GATE EXISTE (2026-07-17). ARCHITECTURE §4 regra 11: todo gate
# nasceu de uma falha SILENCIOSA, e o criterio para criar o proximo e' "o que
# pode quebrar sem nada reclamar?". Esta e' a resposta medida neste repositorio:
#
#   `dev-local-release` nao tinha `binaryDir` proprio. Herdou o do pai —
#   `build/linux-clang-release-hardened` — que e' o diretorio do binario que
#   `scripts/kinein-vectis` executa. Configurar/compilar o dev-local-release
#   passou a sobrescrever o executavel do atalho de Desenvolvimento.
#
# Nada falhou. O CMake nao reclama, a build fica verde, o app ABRE. O autor
# passou horas depurando uma IDE que estava quatro commits atras. E a correcao
# (dar binaryDir proprio) vive no `CMakeUserPresets.json`, que e' GITIGNORED:
# nao ha commit, revisao nem CI que a proteja. Se o arquivo for recriado sem
# `binaryDir`, a mina volta em silencio — e so este gate a pega.
#
# Ele roda em qualquer maquina: o CMakeUserPresets.json e' local de cada
# desenvolvedor, e e' exatamente por isso que a checagem tem que ser local.
set -euo pipefail

cd "$(dirname "$0")/.."

echo "== presets: dois nao escrevem no mesmo diretorio de build =="

mapfile -t linhas < <(python3 scripts/presets-binarydir.py "$PWD")

if ((${#linhas[@]} == 0)); then
    echo "presets: nenhum configure preset usavel declara binaryDir."
    exit 0
fi

# Agrupa nomes por diretorio. Diretorio com 2+ presets = colisao.
declare -A donos=()
for linha in "${linhas[@]}"; do
    nome="${linha%%$'\t'*}"
    dir="${linha#*$'\t'}"
    if [[ -n "${donos[$dir]:-}" ]]; then
        donos[$dir]="${donos[$dir]} $nome"
    else
        donos[$dir]="$nome"
    fi
done

falhas=0
for dir in "${!donos[@]}"; do
    read -r -a nomes <<<"${donos[$dir]}"
    if ((${#nomes[@]} > 1)); then
        falhas=$((falhas + 1))
        echo "✗ ${#nomes[@]} presets escrevem em ${dir#"$PWD"/}" >&2
        for nome in "${nomes[@]}"; do
            echo "    - $nome" >&2
        done
        echo "    Um deles herdou o binaryDir do pai sem declarar o proprio." >&2
        echo "    De' um binaryDir proprio a cada preset. Se o diretorio for o de" >&2
        echo "    um preset OFICIAL, o binario do atalho e' sobrescrito em silencio." >&2
    fi
done

if ((falhas > 0)); then
    echo >&2
    echo "✗ presets FALHOU ($falhas diretorio(s) com dono duplicado)" >&2
    exit 1
fi

echo "presets: ${#linhas[@]} configure presets usaveis, cada um no seu diretorio."

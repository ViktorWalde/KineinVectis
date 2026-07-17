#!/usr/bin/env bash
# Reconstrucao integral e transacional da IDE de desenvolvimento e/ou do
# AppImage portatil, sempre a partir de cache limpo.
#
# Objetivo: impedir que scripts/kinein-vectis combine UI/core de revisoes
# diferentes ou que caches CMake antigos sobrevivam a uma troca de distro,
# toolchain ou fonte. Nao faz git pull, cargo update ou push: ele sincroniza e
# valida somente o codigo que ja esta neste checkout.
#
# POR QUE A LIMPEZA E' PADRAO. Cache velho aqui nao falha barulhento: ele
# entrega uma IDE que parece atual e nao e'. Dois casos ja medidos neste
# repositorio — build de outra distro apontando para /usr/lib/x86_64-linux-gnu
# inexistente, e o binario do atalho quatro commits atras. Reconstruir custa
# minutos; depurar um binario fantasma custou horas.

set -Eeuo pipefail

alvo_ide=0
alvo_appimage=0
limpar=1

mostrar_uso() {
    cat <<'EOF'
uso: scripts/atualizar-tudo.sh [ALVO]... [OPCAO]...

ALVOS (sem alvo = --ide):
  --ide         reconstroi a IDE de Desenvolvimento: CMake sem cache, UI/core,
                gate completo, smoke do launcher e manifesto.
  --appimage    gera o AppImage portatil auditado em dist/, no container
                Debian 12 fixado, e roda os dois smokes de entrega.
  --tudo        os dois, IDE primeiro. So empacota se a IDE ficar verde.

OPCOES:
  --sem-limpeza reaproveita os caches de build (mais rapido, menos seguro).
  --help, -h    esta ajuda.

Nao faz git pull, cargo update nem push.
EOF
}

while (($# > 0)); do
    case "$1" in
        --ide) alvo_ide=1 ;;
        --appimage) alvo_appimage=1 ;;
        --tudo)
            alvo_ide=1
            alvo_appimage=1
            ;;
        --sem-limpeza) limpar=0 ;;
        --help | -h)
            mostrar_uso
            exit 0
            ;;
        *)
            echo "erro: argumento desconhecido: $1" >&2
            echo >&2
            mostrar_uso >&2
            exit 2
            ;;
    esac
    shift
done

if ((alvo_ide == 0 && alvo_appimage == 0)); then
    alvo_ide=1
fi

REPO_ROOT="$(CDPATH= cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

UI_DEBUG="$REPO_ROOT/build/linux-clang-debug-strict/ui/kinein-vectis"
UI_RELEASE="$REPO_ROOT/build/linux-clang-release-hardened/ui/kinein-vectis"
CORE_DEBUG="$REPO_ROOT/target/debug/kinein-core"
CORE_RELEASE="$REPO_ROOT/target/release/kinein-core"
MANIFEST="$REPO_ROOT/build/kinein-build-manifest.env"
SMOKE_SECONDS="${KINEIN_SUPER_SMOKE_SECONDS:-8}"

step="inicializacao"
complete=0
backup_dir=""
lock_file="${TMPDIR:-/tmp}/kinein-vectis-atualizar-tudo-${UID}.lock"

if ! command -v flock >/dev/null 2>&1; then
    echo "erro: comando obrigatorio ausente: flock" >&2
    exit 127
fi
exec 9>"$lock_file"
if ! flock -n 9; then
    echo "erro: outra atualizacao integral da Kinein ja esta em andamento" >&2
    exit 75
fi
backup_dir="$(mktemp -d "${TMPDIR:-/tmp}/kinein-atualizar-tudo.XXXXXX")"

log_step() {
    step="$1"
    echo
    echo "== $step =="
}

require_command() {
    if ! command -v "$1" >/dev/null 2>&1; then
        echo "erro: comando obrigatorio ausente: $1" >&2
        exit 127
    fi
}

backup_artifact() {
    local source="$1"
    local name="$2"
    if [[ -f "$source" ]]; then
        cp -p -- "$source" "$backup_dir/$name"
    else
        : >"$backup_dir/$name.absent"
    fi
}

restore_artifact() {
    local destination="$1"
    local name="$2"
    if [[ -f "$backup_dir/$name" ]]; then
        mkdir -p -- "$(dirname -- "$destination")"
        cp -p -- "$backup_dir/$name" "$destination"
    elif [[ -f "$backup_dir/$name.absent" ]]; then
        rm -f -- "$destination"
    fi
}

# Diretorios de build que ALGUM preset usavel reivindica hoje. A resolucao do
# `inherits` vive em scripts/presets-binarydir.py porque o gate de presets
# precisa da MESMA resposta: se as duas copias divergirem, uma passa a limpar o
# que a outra considera legitimo e a mina volta por baixo. Sai um nome por linha.
dirs_reivindicados() {
    python3 "$REPO_ROOT/scripts/presets-binarydir.py" "$REPO_ROOT" \
        | cut -d$'\t' -f2 \
        | while read -r caminho; do
            case "$caminho" in
                "$REPO_ROOT"/build/*) basename -- "$caminho" ;;
            esac
        done
}

# Um diretorio em build/ que preset nenhum reivindica e' resto de um fluxo que
# nao existe mais. Ele nao e' inofensivo: `build/dev-local` e `build/debug`
# ficaram meses no disco parecendo builds validas.
limpar_orfaos() {
    local -a reivindicados=()
    mapfile -t reivindicados < <(dirs_reivindicados)
    reivindicados+=(appimage)

    local existente nome encontrado
    for existente in "$REPO_ROOT"/build/*/; do
        [[ -d "$existente" ]] || continue
        nome="$(basename -- "$existente")"
        encontrado=0
        for reivindicado in "${reivindicados[@]}"; do
            if [[ "$nome" == "$reivindicado" ]]; then
                encontrado=1
                break
            fi
        done
        if ((encontrado == 0)); then
            echo "  orfao removido (preset nenhum o reivindica): build/$nome"
            rm -rf -- "$existente"
        fi
    done
}

limpar_cache_ide() {
    echo "  build/linux-clang-debug-strict"
    rm -rf -- "$REPO_ROOT/build/linux-clang-debug-strict"
    echo "  build/linux-clang-release-hardened"
    rm -rf -- "$REPO_ROOT/build/linux-clang-release-hardened"
    limpar_orfaos
}

# O cache de FERRAMENTAS (linuxdeploy, plugin Qt, runtime type-2) fica. Ele e'
# verificado por SHA256 fixado a cada uso, entao nao tem como estar "velho": ou
# casa com o pin, ou o proprio empacotador o rebaixa e rebaixa. Apagar so
# forcaria download — e o smoke portatil roda SEM REDE de proposito.
limpar_cache_appimage() {
    local alvo
    for alvo in AppDir cargo-target native native-host native-portable; do
        if [[ -d "$REPO_ROOT/build/appimage/$alvo" ]]; then
            echo "  build/appimage/$alvo"
            rm -rf -- "$REPO_ROOT/build/appimage/$alvo"
        fi
    done
    echo "  preservado: build/appimage/cache/tools (pinado por SHA256)"
}

source_fingerprint() {
    local -a files=()
    mapfile -d '' -t files < <(
        git ls-files -co --exclude-standard -z -- \
            Cargo.toml Cargo.lock CMakeLists.txt CMakePresets.json \
            crates ui cmake scripts schemas \
            | sort -z
    )
    {
        local file
        for file in "${files[@]}"; do
            if [[ -f "$file" ]]; then
                sha256sum -- "$file"
            fi
        done
    } | sha256sum | awk '{ print $1 }'
}

on_exit() {
    local status=$?
    set +e
    if [[ "$status" -ne 0 && "$complete" -ne 1 ]]; then
        echo >&2
        echo "✗ FALHOU em: $step (exit $status)" >&2
        echo "  restaurando os binarios de desenvolvimento anteriores..." >&2
        restore_artifact "$UI_DEBUG" ui-debug
        restore_artifact "$UI_RELEASE" ui-release
        restore_artifact "$CORE_DEBUG" core-debug
        restore_artifact "$CORE_RELEASE" core-release
    fi
    if [[ -n "$backup_dir" ]]; then
        rm -rf -- "$backup_dir"
    fi
    exit "$status"
}
trap on_exit EXIT

for command in bash cargo cmake flock git ninja python3 sha256sum timeout; do
    require_command "$command"
done

# A ajuda vem para uma VARIAVEL antes do grep. Com `cmake --help | grep -q`,
# o grep sai no primeiro acerto, fecha o cano, o cmake morre de SIGPIPE (141)
# e o `pipefail` la em cima transforma isso no resultado do pipeline: a
# checagem reprovava um CMake que TEM --fresh. Reproduzido 8/8 no CMake 4.3.0,
# cuja ajuda e' longa o bastante para o cmake ainda estar escrevendo.
cmake_help="$(cmake --help)"
if ! grep -q -- '--fresh' <<<"$cmake_help"; then
    echo "erro: este fluxo exige CMake com suporte a --fresh" >&2
    exit 1
fi

fluxo_ide() {

log_step "snapshot das fontes e backup transacional"
source_before="$(source_fingerprint)"
backup_artifact "$UI_DEBUG" ui-debug
backup_artifact "$UI_RELEASE" ui-release
backup_artifact "$CORE_DEBUG" core-debug
backup_artifact "$CORE_RELEASE" core-release
echo "fonte: $source_before"

if ((limpar == 1)); then
    log_step "apagando o cache de build da IDE"
    limpar_cache_ide
fi

log_step "CMake debug --fresh"
cmake --fresh --preset linux-clang-debug-strict

log_step "CMake release --fresh"
cmake --fresh --preset linux-clang-release-hardened

log_step "rebuild limpo da UI debug"
cmake --build --preset debug-strict --clean-first

log_step "rebuild limpo da UI release"
cmake --build --preset release-hardened --clean-first

log_step "limpeza dirigida dos crates executaveis"
cargo clean -p kinein-protocol
cargo clean -p kinein-core

log_step "gate integral nos presets usados pelo launcher"
KINEIN_PRESET_DEBUG=debug-strict \
KINEIN_PRESET_RELEASE=release-hardened \
    "$REPO_ROOT/scripts/verificar.sh" --completo

log_step "materializacao do core debug"
# `cargo test`/Clippy validam o bin target, mas podem deixar somente os
# executaveis de harness em target/debug/deps. O launcher aceita fallback
# debug; produzi-lo explicitamente impede um manifesto de quatro artefatos com
# um deles herdado de uma build antiga.
cargo build -p kinein-core

log_step "consistencia das fontes durante o build"
source_after="$(source_fingerprint)"
if [[ "$source_before" != "$source_after" ]]; then
    echo "erro: arquivos-fonte mudaram durante a atualizacao" >&2
    echo "antes:  $source_before" >&2
    echo "depois: $source_after" >&2
    echo "rode o script novamente com o checkout estavel" >&2
    exit 1
fi

for artifact in "$UI_DEBUG" "$UI_RELEASE" "$CORE_DEBUG" "$CORE_RELEASE"; do
    if [[ ! -x "$artifact" ]]; then
        echo "erro: executavel esperado nao foi produzido: $artifact" >&2
        exit 1
    fi
done

log_step "smoke do launcher real (${SMOKE_SECONDS}s)"
set +e
timeout "$SMOKE_SECONDS" env \
    QT_QPA_PLATFORM=offscreen \
    QT_FORCE_STDERR_LOGGING=1 \
    "$REPO_ROOT/scripts/kinein-vectis"
smoke_status=$?
set -e
if [[ "$smoke_status" -ne 124 ]]; then
    echo "erro: smoke deveria permanecer vivo ate o timeout (124), recebeu $smoke_status" >&2
    exit 1
fi

log_step "manifesto da build sincronizada"
mkdir -p -- "$(dirname -- "$MANIFEST")"
manifest_tmp="$backup_dir/kinein-build-manifest.env"
worktree_state=clean
if [[ -n "$(git status --porcelain --untracked-files=normal)" ]]; then
    worktree_state=dirty
fi
{
    echo "schema_version=1"
    echo "generated_at=$(date --iso-8601=seconds)"
    echo "git_head=$(git rev-parse HEAD)"
    echo "worktree=$worktree_state"
    echo "source_sha256=$source_after"
    echo "ui_debug_path=$UI_DEBUG"
    echo "ui_debug_sha256=$(sha256sum -- "$UI_DEBUG" | awk '{ print $1 }')"
    echo "ui_release_path=$UI_RELEASE"
    echo "ui_release_sha256=$(sha256sum -- "$UI_RELEASE" | awk '{ print $1 }')"
    echo "core_debug_path=$CORE_DEBUG"
    echo "core_debug_sha256=$(sha256sum -- "$CORE_DEBUG" | awk '{ print $1 }')"
    echo "core_release_path=$CORE_RELEASE"
    echo "core_release_sha256=$(sha256sum -- "$CORE_RELEASE" | awk '{ print $1 }')"
    echo "gate=green"
    echo "smoke_exit=124"
} >"$manifest_tmp"
mv -f -- "$manifest_tmp" "$MANIFEST"
cat "$MANIFEST"

# Reinstala o atalho de Desenvolvimento no fim: o proposito do script e' que
# o atalho rode o estado ATUAL, e isso inclui o icone. Sem este passo, trocar o
# icone e rodar o script nao mudava o atalho — a fonte da verdade
# (ui/assets/app-icon.png) so' chegava ao gnome-shell por instalacao explicita.
log_step "reinstalando o atalho de Desenvolvimento (binario + icone)"
bash "$REPO_ROOT/scripts/instalar-atalho.sh"

}

fluxo_appimage() {

if ((limpar == 1)); then
    log_step "apagando o cache de build do AppImage"
    limpar_cache_appimage
fi

# O empacotador monta a entrega em staging e so publica em dist/ quando o
# conjunto esta completo: uma falha aqui preserva o ultimo AppImage valido.
log_step "AppImage portatil (container Debian 12 fixado)"
bash "$REPO_ROOT/scripts/empacotar-appimage-portatil.sh"

log_step "smoke do AppImage no host"
bash "$REPO_ROOT/scripts/testar-appimage.sh"

# O que interessa ao testador: abrir numa maquina que NAO tem Qt, Rust, CMake
# nem compilador — e sem rede.
log_step "smoke do AppImage em Debian minimo, sem rede"
bash "$REPO_ROOT/scripts/testar-appimage-portatil.sh"

}

if ((alvo_ide == 1)); then
    fluxo_ide
fi

if ((alvo_appimage == 1)); then
    fluxo_appimage
fi

complete=1
echo
echo "✓ ATUALIZACAO INTEGRAL VERDE"
if ((alvo_ide == 1)); then
    echo "  IDE:       $REPO_ROOT/scripts/kinein-vectis"
    echo "  manifesto: $MANIFEST"
fi
if ((alvo_appimage == 1)); then
    echo "  AppImage:  $REPO_ROOT/dist/"
fi

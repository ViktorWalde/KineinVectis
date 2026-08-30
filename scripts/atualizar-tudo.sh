#!/usr/bin/env bash
# Reconstrucao integral e transacional do checkout de desenvolvimento.
#
# Objetivo: impedir que scripts/kinein-vectis combine UI/core de revisoes
# diferentes ou que caches CMake antigos sobrevivam a uma troca de distro,
# toolchain ou fonte. Nao faz git pull, cargo update, packaging ou rede: ele
# sincroniza e valida somente o codigo que ja esta neste checkout.

set -Eeuo pipefail

case "${1:-}" in
    "") ;;
    --help | -h)
        cat <<'EOF'
uso: scripts/atualizar-tudo.sh

Reconfigura CMake sem cache, reconstrói UI/core, executa o gate completo,
valida o launcher e grava build/kinein-build-manifest.env.

Não faz git pull, cargo update, AppImage ou push.
EOF
        exit 0
        ;;
    *)
        echo "erro: argumento desconhecido: $1" >&2
        echo "uso: scripts/atualizar-tudo.sh" >&2
        exit 2
        ;;
esac

REPO_ROOT="$(unset CDPATH; cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
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

for command in bash cargo cmake flock git ninja sha256sum timeout; do
    require_command "$command"
done

if ! cmake --help | grep -q -- '--fresh'; then
    echo "erro: este fluxo exige CMake com suporte a --fresh" >&2
    exit 1
fi

log_step "snapshot das fontes e backup transacional"
source_before="$(source_fingerprint)"
backup_artifact "$UI_DEBUG" ui-debug
backup_artifact "$UI_RELEASE" ui-release
backup_artifact "$CORE_DEBUG" core-debug
backup_artifact "$CORE_RELEASE" core-release
echo "fonte: $source_before"

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

complete=1
echo
echo "✓ ATUALIZACAO INTEGRAL VERDE"
echo "  execute: $REPO_ROOT/scripts/kinein-vectis"
echo "  manifesto: $MANIFEST"

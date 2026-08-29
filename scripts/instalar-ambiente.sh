#!/bin/sh
# Instala as dependencias de desenvolvimento do Kinein Vectis apos trocar de
# distro/maquina: toolchain C++/Qt6, Rust e ferramentas orquestradas pela IDE.
#
# Uso:
#   scripts/instalar-ambiente.sh              # instala o conjunto base
#   scripts/instalar-ambiente.sh --extras     # + shellcheck e cargo-deny
#                                             #   (degraus da escada de rigor,
#                                             #   docs/diario/18-daily-driver-plan.md)
#   scripts/instalar-ambiente.sh --dry-run    # so mostra o que seria executado
#
# Distros suportadas: Arch/CachyOS (pacman, alvo principal), Debian/Ubuntu/
# Pop!_OS (apt) e Fedora (dnf, melhor esforco). Em outra distro o script
# imprime a lista de ferramentas para instalacao manual.
#
# O script e idempotente (pacman --needed / apt-get install so instala o que
# falta) e usa sudo por comando — nao rode como root. Nada aqui mexe no
# repositorio; depois de instalar, configure os presets e rode o gate
# (docs/build/COMANDOS_BUILD_VERIFICACAO.md).

set -eu

DRY_RUN=0
EXTRAS=0
for arg in "$@"; do
    case "$arg" in
        --dry-run) DRY_RUN=1 ;;
        --extras) EXTRAS=1 ;;
        -h | --help)
            sed -n '2,19p' "$0" | sed 's/^# \{0,1\}//'
            exit 0
            ;;
        *)
            echo "uso: $0 [--extras] [--dry-run]" >&2
            exit 2
            ;;
    esac
done

executar() {
    echo "+ $*"
    if [ "$DRY_RUN" -eq 0 ]; then
        "$@"
    fi
}

# ---------------------------------------------------------------------------
# Deteccao de distro via /etc/os-release (ID e ID_LIKE).
# ---------------------------------------------------------------------------
DISTRO="desconhecida"
if [ -r /etc/os-release ]; then
    # shellcheck disable=SC1091
    . /etc/os-release
    ids="${ID:-} ${ID_LIKE:-}"
    case "$ids" in
        *arch*) DISTRO="arch" ;;
        *debian* | *ubuntu*) DISTRO="debian" ;;
        *fedora* | *rhel*) DISTRO="fedora" ;;
    esac
fi
echo "distro detectada: $DISTRO"

# ---------------------------------------------------------------------------
# Pacotes por distro. Conjunto base = o que docs/build/14-development-environment.md
# e o ToolDetector do core esperam: cmake, ninja, clang (format/tidy/clangd),
# gcc, gdb, lldb, Qt6 (base/declarative/tools), git, ripgrep, fd e rustup.
# ---------------------------------------------------------------------------
instalar_arch() {
    executar sudo pacman -S --needed --noconfirm \
        base-devel git cmake ninja \
        clang lldb gdb \
        qt6-base qt6-declarative qt6-tools \
        rustup rust-analyzer \
        ripgrep fd
    if [ "$EXTRAS" -eq 1 ]; then
        executar sudo pacman -S --needed --noconfirm shellcheck cargo-deny
    fi
}

instalar_debian() {
    executar sudo apt-get update
    executar sudo apt-get install -y \
        build-essential git cmake ninja-build \
        clang clangd clang-format clang-tidy lldb gdb \
        qt6-base-dev qt6-declarative-dev qt6-tools-dev \
        qml6-module-qtqml qml6-module-qtqml-workerscript \
        qml6-module-qtqml-models qml6-module-qtquick \
        qml6-module-qtquick-controls qml6-module-qtquick-layouts \
        qml6-module-qtquick-window \
        ripgrep fd-find
    if [ "$EXTRAS" -eq 1 ]; then
        executar sudo apt-get install -y shellcheck
        echo "aviso: cargo-deny no Debian/Ubuntu vem via 'cargo install cargo-deny'"
    fi
    echo "aviso: no Debian/Ubuntu o rustup oficial vem de https://rustup.rs"
    echo "       (rust-analyzer entra como componente: rustup component add rust-analyzer)"
}

instalar_fedora() {
    executar sudo dnf install -y \
        @development-tools git cmake ninja-build \
        clang clang-tools-extra lldb gdb \
        qt6-qtbase-devel qt6-qtdeclarative-devel qt6-qttools-devel \
        rustup rust-analyzer \
        ripgrep fd-find
    if [ "$EXTRAS" -eq 1 ]; then
        executar sudo dnf install -y ShellCheck
        echo "aviso: cargo-deny no Fedora vem via 'cargo install cargo-deny'"
    fi
}

case "$DISTRO" in
    arch) instalar_arch ;;
    debian) instalar_debian ;;
    fedora) instalar_fedora ;;
    *)
        echo "distro nao reconhecida. Instale manualmente:" >&2
        echo "  git cmake ninja clang(+clangd/format/tidy) gcc gdb lldb" >&2
        echo "  Qt6 (base, declarative, tools) rustup rust-analyzer rg fd" >&2
        exit 1
        ;;
esac

# ---------------------------------------------------------------------------
# Toolchain Rust: o canal vem do rust-toolchain.toml, nao de um literal aqui.
# Instalar agora evita surpresa offline — mas so evita se for a MESMA toolchain
# que a build vai usar. Este bloco dizia 'stable' enquanto o repositorio fixava
# uma versao exata: instalava uma toolchain que ninguem usa e deixava a de
# verdade para o primeiro `cargo build` baixar, justamente o que ele promete
# evitar.
# ---------------------------------------------------------------------------
RAIZ_REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CANAL_RUST="$(sed -n 's/^[[:space:]]*channel[[:space:]]*=[[:space:]]*"\([^"]*\)".*/\1/p' \
    "$RAIZ_REPO/rust-toolchain.toml" | head -n1)"
if [ -z "$CANAL_RUST" ]; then
    echo "erro: nao consegui ler 'channel' de rust-toolchain.toml" >&2
    exit 1
fi

if command -v rustup >/dev/null 2>&1 || [ "$DRY_RUN" -eq 1 ]; then
    executar rustup toolchain install "$CANAL_RUST"
    executar rustup component add --toolchain "$CANAL_RUST" rustfmt clippy
else
    echo "aviso: rustup ainda nao esta no PATH; instale-o e rode:" >&2
    echo "       rustup toolchain install $CANAL_RUST" >&2
    echo "       rustup component add --toolchain $CANAL_RUST rustfmt clippy" >&2
fi

# ---------------------------------------------------------------------------
# Verificacao final: mesmo conjunto que o ToolDetector do core reporta.
# ---------------------------------------------------------------------------
echo ""
echo "== verificacao =="
faltando=0
for ferramenta in git cmake ninja gcc g++ clang clang++ clangd clang-format \
    clang-tidy gdb lldb lldb-dap rustup cargo rustc rustfmt rust-analyzer rg; do
    if command -v "$ferramenta" >/dev/null 2>&1; then
        echo "  ok      $ferramenta"
    else
        echo "  FALTA   $ferramenta"
        faltando=1
    fi
done
# fd chama-se fdfind no Debian/Ubuntu; o core aceita os dois nomes.
if command -v fd >/dev/null 2>&1 || command -v fdfind >/dev/null 2>&1; then
    echo "  ok      fd/fdfind"
else
    echo "  FALTA   fd/fdfind"
    faltando=1
fi
if command -v qmake6 >/dev/null 2>&1 || [ -x /usr/lib/qt6/bin/qmllint ]; then
    echo "  ok      qt6"
else
    echo "  FALTA   qt6"
    faltando=1
fi

echo ""
if [ "$faltando" -eq 0 ]; then
    echo "ambiente completo. Proximos passos:"
else
    echo "ambiente INCOMPLETO (itens FALTA acima). Apos resolver:"
fi
echo "  cmake --preset dev-local && cmake --preset dev-local-release"
echo "  scripts/verificar.sh"
echo "  scripts/instalar-atalho.sh   # atalho de desenvolvimento no menu"

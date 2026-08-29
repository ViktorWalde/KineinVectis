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

# O rustup instala em ~/.cargo/bin e so mexe no PERFIL do shell — o shell atual
# nao ve nada ate reabrir. Sem isto a verificacao final diz "FALTA cargo" numa
# maquina onde o cargo esta instalado, que e' pior que nao verificar.
if [ -d "$HOME/.cargo/bin" ]; then
    PATH="$HOME/.cargo/bin:$PATH"
    export PATH
fi

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
    # `rustup` da distro so entra se ainda nao houver um no PATH: quem ja
    # instalou pelo rustup.rs (em ~/.cargo/bin) acabaria com dois gerenciadores
    # disputando o mesmo ~/.rustup.
    pacotes_rust="rustup rust-analyzer"
    if command -v rustup >/dev/null 2>&1; then
        pacotes_rust="rust-analyzer"
    fi
    # shellcheck disable=SC2086
    executar sudo dnf install -y \
        @development-tools git cmake ninja-build \
        clang clang-tools-extra lldb gdb \
        qt6-qtbase-devel qt6-qtdeclarative-devel qt6-qttools-devel \
        $pacotes_rust \
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
RAIZ_REPO="$(unset CDPATH; cd -- "$(dirname -- "$0")/.." && pwd)"
CANAL_RUST="$(sed -n 's/^[[:space:]]*channel[[:space:]]*=[[:space:]]*"\([^"]*\)".*/\1/p' \
    "$RAIZ_REPO/rust-toolchain.toml" | head -n1)"
if [ -z "$CANAL_RUST" ]; then
    echo "erro: nao consegui ler 'channel' de rust-toolchain.toml" >&2
    exit 1
fi

if command -v rustup >/dev/null 2>&1 || [ "$DRY_RUN" -eq 1 ]; then
    executar rustup toolchain install "$CANAL_RUST"
    # `rust-analyzer` entra como COMPONENTE, e nao so como pacote de distro: o
    # do rustup e' casado com a toolchain fixada, e o proxy `~/.cargo/bin/
    # rust-analyzer` existe mesmo sem o componente — um `command -v` sozinho
    # diria "ok" para um binario que nao roda.
    executar rustup component add --toolchain "$CANAL_RUST" rustfmt clippy rust-analyzer
else
    echo "aviso: rustup ainda nao esta no PATH; instale-o e rode:" >&2
    echo "       rustup toolchain install $CANAL_RUST" >&2
    echo "       rustup component add --toolchain $CANAL_RUST rustfmt clippy rust-analyzer" >&2
fi

# ---------------------------------------------------------------------------
# Verificacao final: mesmo conjunto que o ToolDetector do core reporta.
# ---------------------------------------------------------------------------
echo ""
echo "== verificacao =="
faltando=0
for ferramenta in git cmake ninja gcc g++ clang clang++ clangd clang-format \
    clang-tidy gdb lldb lldb-dap rustup cargo rustc rustfmt rust-analyzer rg; do
    # `rust-analyzer` e' proxy do rustup: existe no PATH mesmo sem o componente.
    # Achar o arquivo nao prova nada; rodar prova.
    if [ "$ferramenta" = "rust-analyzer" ]; then
        if rust-analyzer --version >/dev/null 2>&1; then
            echo "  ok      rust-analyzer"
        else
            echo "  FALTA   rust-analyzer (proxy existe, componente nao)"
            faltando=1
        fi
        continue
    fi
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

# ---------------------------------------------------------------------------
# Presets locais (`dev-local*`).
#
# POR QUE ISTO ESTA AQUI (2026-08-29). `CMakeUserPresets.json` e' gitignorado —
# certo, porque ele descreve ESTA maquina. O problema e' que o gate
# (`scripts/verificar.sh`) usa `dev-local` por padrao e a documentacao mandava
# "depois deste script, rode `cmake --preset dev-local`" — um preset que num
# checkout novo NAO EXISTE. Resultado: `scripts/verificar.sh` completo falha
# num ambiente recem-instalado, e nada explica por que. O bootstrap gera o
# arquivo se ele faltar; se ja existir, nao encosta (e' arquivo do usuario).
#
# Os presets herdam os oficiais e desligam sanitizers/clang, seguindo o que o
# COMO_EXECUTAR.md ja documentava para a maquina do autor. Quem quiser o rigor
# maximo usa os presets oficiais direto:
#   cmake --preset linux-clang-debug-strict
# ---------------------------------------------------------------------------
PRESETS_USUARIO="$RAIZ_REPO/CMakeUserPresets.json"
echo ""
echo "== presets locais =="
if [ -e "$PRESETS_USUARIO" ]; then
    echo "  ok      CMakeUserPresets.json ja existe (nao foi tocado)"
elif [ "$DRY_RUN" -eq 1 ]; then
    echo "+ gerar $PRESETS_USUARIO (dev-local, dev-local-release)"
else
    cat > "$PRESETS_USUARIO" <<'PRESETS'
{
  "version": 6,
  "cmakeMinimumRequired": { "major": 3, "minor": 25, "patch": 0 },
  "configurePresets": [
    {
      "name": "dev-local",
      "displayName": "Dev local (debug, sem sanitizers)",
      "inherits": "linux-clang-debug-strict",
      "binaryDir": "${sourceDir}/build/dev-local",
      "cacheVariables": {
        "CMAKE_C_COMPILER": "cc",
        "CMAKE_CXX_COMPILER": "c++",
        "KINEIN_ENABLE_SANITIZERS": "OFF"
      }
    },
    {
      "name": "dev-local-release",
      "displayName": "Dev local (release)",
      "inherits": "linux-clang-release-hardened",
      "binaryDir": "${sourceDir}/build/dev-local-release",
      "cacheVariables": {
        "CMAKE_C_COMPILER": "cc",
        "CMAKE_CXX_COMPILER": "c++"
      }
    }
  ],
  "buildPresets": [
    { "name": "dev-local", "configurePreset": "dev-local" },
    { "name": "dev-local-release", "configurePreset": "dev-local-release" }
  ]
}
PRESETS
    echo "  criado  CMakeUserPresets.json (dev-local, dev-local-release)"
fi

# `verificar-cpp.sh` le o compile_commands.json de build/linux-clang-debug-strict;
# `verificar.sh` compila dev-local*. Sao TRES diretorios, e todos precisam existir
# antes do gate completo passar.
echo ""
echo "== configuracao dos build dirs =="
if [ "$faltando" -ne 0 ]; then
    echo "  pulado (ambiente incompleto)"
elif [ "$DRY_RUN" -eq 1 ]; then
    echo "+ cmake --preset linux-clang-debug-strict"
    echo "+ cmake --preset dev-local"
    echo "+ cmake --preset dev-local-release"
else
    for preset in linux-clang-debug-strict dev-local dev-local-release; do
        executar cmake -S "$RAIZ_REPO" --preset "$preset"
    done
fi

echo ""
if [ "$faltando" -eq 0 ]; then
    echo "ambiente completo. Proximos passos:"
else
    echo "ambiente INCOMPLETO (itens FALTA acima). Apos resolver:"
fi
echo "  scripts/verificar.sh          # gate completo"
echo "  scripts/instalar-atalho.sh    # atalho de desenvolvimento no menu"

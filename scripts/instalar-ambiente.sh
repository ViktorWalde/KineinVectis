#!/bin/sh
# Instala as dependencias de desenvolvimento do Kinein Vectis apos trocar de
# distro/maquina: toolchain C++/Qt6, Rust e ferramentas orquestradas pela IDE.
#
# Uso:
#   scripts/instalar-ambiente.sh              # instala o conjunto base
#   scripts/instalar-ambiente.sh --extras     # + shellcheck e cargo-deny
#                                             #   (degraus da escada de rigor,
#                                             #   DocsPrivate/diario/18-daily-driver-plan.md)
#   scripts/instalar-ambiente.sh --dry-run    # so mostra o que seria executado
#
# Distros suportadas: Arch/CachyOS (pacman, alvo principal), Debian/Ubuntu/
# Pop!_OS (apt) e Fedora (dnf, melhor esforco). Em outra distro o script
# imprime a lista de ferramentas para instalacao manual.
#
# O script e idempotente (pacman --needed / apt-get install so instala o que
# falta) e usa sudo por comando — nao rode como root. Nada aqui mexe no
# repositorio; depois de instalar, configure os presets e rode o gate
# (DocsPublic/build/comandos-de-build-e-verificacao.md).

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

# O rustup instala em ~/.cargo/bin. Prepor aqui deixa ESTE script funcionar,
# mas NAO e' o que o usuario tem — ver `verificar_no_shell_do_usuario` no fim.
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
# Pacotes por distro. Conjunto base = o que DocsPublic/build/14-development-environment.md
# e o ToolDetector do core esperam: cmake, ninja, clang (format/tidy/clangd),
# gcc, gdb, lldb, Qt6 (base/declarative/tools), git, ripgrep, fd e rustup.
# ---------------------------------------------------------------------------
instalar_arch() {
    executar sudo pacman -S --needed --noconfirm \
        base-devel git cmake ninja \
        clang lldb gdb \
        qt6-base qt6-declarative qt6-tools \
        rustup \
        ripgrep fd
    if [ "$EXTRAS" -eq 1 ]; then
        executar sudo pacman -S --needed --noconfirm shellcheck
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
    # `rust-analyzer` NAO entra pelo dnf: ele depende do pacote `rust` e
    # arrasta a toolchain inteira da distro — medido em 2026-08-29 nesta
    # maquina: rust + rust-std-static + rust-src, ~330 MiB, na versao 1.98.0,
    # que NAO e' a que o rust-toolchain.toml fixa. O rustup instala um
    # rust-analyzer casado com a toolchain do projeto (ver o bloco de
    # componentes abaixo), que e' o que se quer.
    pacotes_rust="rustup"
    if command -v rustup >/dev/null 2>&1; then
        pacotes_rust=""
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

# ---------------------------------------------------------------------------
# rustup: INSTALAR, nao avisar.
#
# POR QUE ISTO MUDOU (2026-08-29). Este bloco antes so imprimia "instale-o e
# rode..." quando o rustup nao estava no PATH. Num Debian/Ubuntu limpo — onde
# nao existe pacote `rustup` — o script terminava dizendo "ambiente completo"
# e o usuario ficava sem `cargo`. Um bootstrap que manda voce fazer o passo
# central a mao nao e' bootstrap.
#
# Usa o instalador OFICIAL (https://rustup.rs), que e' o que a documentacao do
# proprio projeto ja mandava usar no Debian. E' o UNICO ponto deste script que
# baixa algo fora do gerenciador de pacotes; por isso ele anuncia a URL antes.
#
# SEM `--no-modify-path`: e' o padrao do rustup, e e' ele que acrescenta
# `~/.cargo/bin` ao perfil do shell. Instalar com --no-modify-path e depois
# esquecer de gravar a linha foi exatamente o defeito medido nesta data.
if ! command -v rustup >/dev/null 2>&1 && [ "$DRY_RUN" -eq 0 ]; then
    echo ""
    echo "== rustup =="
    echo "  nao encontrado. Instalando pelo instalador oficial:"
    echo "    https://sh.rustup.rs"
    instalador="$(mktemp)"
    if curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -o "$instalador"; then
        sh "$instalador" -y --profile minimal --default-toolchain none
        rm -f "$instalador"
        if [ -f "$HOME/.cargo/env" ]; then
            # shellcheck disable=SC1091
            . "$HOME/.cargo/env"
        fi
    else
        rm -f "$instalador"
        echo "erro: falha ao baixar o instalador do rustup." >&2
        echo "      Sem rede? Instale manualmente e rode este script de novo:" >&2
        echo "      https://rustup.rs" >&2
        exit 1
    fi
elif ! command -v rustup >/dev/null 2>&1; then
    echo ""
    echo "== rustup =="
    echo "+ curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y"
fi

if command -v rustup >/dev/null 2>&1 || [ "$DRY_RUN" -eq 1 ]; then
    executar rustup toolchain install "$CANAL_RUST"
    # `rust-analyzer` entra como COMPONENTE, e nao so como pacote de distro: o
    # do rustup e' casado com a toolchain fixada, e o proxy `~/.cargo/bin/
    # rust-analyzer` existe mesmo sem o componente — um `command -v` sozinho
    # diria "ok" para um binario que nao roda.
    executar rustup component add --toolchain "$CANAL_RUST" rustfmt clippy rust-analyzer
    # `cargo-deny` NAO e' extra: o gate depende dele desde 2026-08-30
    # (scripts/verificar-deny.sh). Vem por `cargo install` porque nenhuma das
    # distros-alvo o empacota. `--locked` para reproduzir o build do autor dele.
    if ! command -v cargo-deny >/dev/null 2>&1; then
        executar cargo install cargo-deny --locked
    fi
else
    # So chega aqui se a instalacao acima falhou de um jeito que nao abortou.
    echo "aviso: rustup ainda nao esta no PATH; instale-o e rode:" >&2
    echo "       rustup toolchain install $CANAL_RUST" >&2
    echo "       rustup component add --toolchain $CANAL_RUST rustfmt clippy rust-analyzer" >&2
fi

# ---------------------------------------------------------------------------
# Verificacao final: mesmo conjunto que o ToolDetector do core reporta.
# ---------------------------------------------------------------------------
# ---------------------------------------------------------------------------
# PATH persistente do rustup.
#
# POR QUE ISTO ESTA AQUI (2026-08-29). O rustup instala em `~/.cargo/bin` e
# grava `~/.cargo/env`, mas so acrescenta a linha ao PERFIL do shell se tiver
# sido instalado SEM `--no-modify-path`. Quando nao ha essa linha, o shell do
# usuario simplesmente nao tem `cargo` — e este script nao percebia, porque
# prepoe `~/.cargo/bin` ao proprio PATH la em cima.
#
# O sintoma medido nesta data: o script imprimiu "ok cargo" e "ambiente
# completo", e o comando SEGUINTE do usuario morreu em "cargo: comando nao
# encontrado". Verificacao que passa num contexto que o usuario nao tem e' pior
# que verificacao nenhuma: ela mente com confianca.
# ---------------------------------------------------------------------------
if [ -f "$HOME/.cargo/env" ]; then
    ja_no_perfil=0
    for perfil in "$HOME/.bashrc" "$HOME/.bash_profile" "$HOME/.profile" \
        "$HOME/.zshrc"; do
        if [ -f "$perfil" ] && grep -q '\.cargo/env' "$perfil"; then
            ja_no_perfil=1
        fi
    done
    if [ "$ja_no_perfil" -eq 0 ]; then
        alvo="$HOME/.bashrc"
        echo ""
        echo "== PATH do rustup =="
        if [ "$DRY_RUN" -eq 1 ]; then
            echo "+ acrescentar '. \"\$HOME/.cargo/env\"' em $alvo"
        else
            # O `$HOME` fica LITERAL de proposito: quem expande e' o perfil,
            # a cada sessao, e nao este script uma vez so.
            # shellcheck disable=SC2016
            printf '\n# Kinein Vectis: toolchain Rust do rustup.\n. "$HOME/.cargo/env"\n' \
                >>"$alvo"
            echo "  acrescentado  . \"\$HOME/.cargo/env\"  em $alvo"
            echo "  (vale no PROXIMO shell; neste, rode: . \"\$HOME/.cargo/env\")"
        fi
    fi
fi

echo ""
echo "== verificacao =="
faltando=0
# A lista abaixo = o que o `KNOWN_TOOLS` do core detecta (o painel Ferramentas
# da IDE reporta exatamente isto) MAIS o que os gates exigem.
#
#   do core, e nao do gate:  claude, codex — CLIs de IA, opcionais. O core os
#                            detecta como qualquer outra ferramenta; a IDE nao
#                            depende deles (a linha de IA embutida saiu em
#                            2026-07-17). Ficam fora daqui de proposito: um
#                            bootstrap nao deve instalar CLI de IA de ninguem.
#   do gate, e nao do core:  clang-format, clang-tidy — usados pelo
#                            verificar-cpp.sh; o core nao os orquestra.
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
# DocsPublic/build/como-executar.md ja documentava para a maquina do autor. Quem quiser o rigor
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

# A verificacao acima roda com `~/.cargo/bin` preposto por este script. O que
# decide se o ambiente serve, porem, e' o shell de LOGIN do usuario: e' nele
# que ele vai rodar `cargo` e `scripts/verificar.sh`.
echo ""
echo "== verificacao no shell de login (o PATH que voce realmente tem) =="
# `env -i` e' obrigatorio: sem ele o `bash -l` HERDA o PATH deste script (que
# tem `~/.cargo/bin` preposto la em cima) e responde "ok" para um shell que na
# verdade nao ve nada. Foi assim que a primeira versao desta checagem mentiu.
fora_do_path=""
for ferramenta in cargo rustfmt; do
    if ! env -i HOME="$HOME" USER="${USER:-}" TERM=dumb \
        bash -lc "command -v $ferramenta" >/dev/null 2>&1; then
        fora_do_path="$fora_do_path $ferramenta"
    fi
done
if [ -n "$fora_do_path" ]; then
    echo "  AVISO  fora do seu PATH:$fora_do_path"
    echo "         Este shell ainda nao ve a toolchain. Rode UMA vez:"
    echo "           . \"\$HOME/.cargo/env\""
    echo "         (o proximo shell ja vem certo, pelo perfil)"
else
    echo "  ok      cargo e rustfmt visiveis no seu shell"
fi

# Um `rustc` de distro no PATH nao quebra o build (o cargo do rustup resolve o
# rustc pela toolchain fixada), mas engana na hora de depurar: `rustc --version`
# no terminal responde a versao da DISTRO, nao a do projeto.
rustc_do_shell="$(env -i HOME="$HOME" USER="${USER:-}" TERM=dumb \
    bash -lc 'command -v rustc' 2>/dev/null || true)"
case "$rustc_do_shell" in
    "$HOME"/.cargo/bin/* | "") ;;
    *)
        echo "  nota    o \`rustc\` do seu shell e' $rustc_do_shell (da distro)."
        echo "          O build usa a toolchain fixada em rust-toolchain.toml;"
        echo "          so \`rustc --version\` no terminal e' que vai divergir."
        ;;
esac

echo ""
if [ "$faltando" -ne 0 ]; then
    echo "ambiente INCOMPLETO (itens FALTA acima). Apos resolver:"
elif [ -n "$fora_do_path" ]; then
    # NAO dizer "completo" aqui: o passo seguinte e' `scripts/verificar.sh`, que
    # comeca com `cargo fmt` e morreria em "comando nao encontrado".
    echo "ambiente instalado, mas ESTE shell ainda nao ve a toolchain."
    echo "Rode a linha do aviso acima e entao:"
else
    echo "ambiente completo. Proximos passos:"
fi
echo "  scripts/verificar.sh          # gate completo"
echo "  scripts/instalar-atalho.sh    # atalho de desenvolvimento no menu"

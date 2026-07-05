# Como executar o Kinein Vectis

A IDE é **100% offline e local**. Você não precisa ligar nada antes: ao abrir a
UI, ela mesma inicia o `kinein-core` (Rust) como processo filho e conversa
com ele por JSON-RPC. Fechou a UI, o core morre junto.

## Jeito mais simples: clicar no ícone

O atalho **Kinein Vectis** já está instalado no menu de aplicativos
(ícone KW). Ele executa `scripts/kinein-vectis`, que sobe a UI release e
aponta para o core release.

Se precisar reinstalar o atalho (por exemplo, se mover a pasta do projeto):

```bash
./scripts/instalar-atalho.sh
```

## Pelo terminal

```bash
./scripts/kinein-vectis
```

O launcher usa os binários release e cai para os de debug se os release não
existirem.

## Compilar do zero (quando mudar o código)

Sequência curta para copiar/colar: veja
[`docs/COMANDOS_BUILD_VERIFICACAO.md`](docs/COMANDOS_BUILD_VERIFICACAO.md).

### 1. Core Rust

```bash
cargo build --release -p kinein-core     # uso diário
cargo build -p kinein-core               # debug, para desenvolvimento
```

### 2. UI Qt/QML

```bash
cmake --preset dev-local-release            # configura (só na primeira vez)
cmake --build --preset dev-local-release    # compila
```

Para desenvolvimento com sanitizers (ASan/UBSan):

```bash
cmake --preset dev-local
cmake --build --preset dev-local
./build/linux-clang-debug-strict/ui/kinein-vectis
```

> Os presets `dev-local*` estão em `CMakeUserPresets.json` (arquivo local,
> fora do git). Eles herdam os presets estritos oficiais e apontam o clang
> para o toolchain GCC 13 desta máquina (o Pop!_OS tem um diretório GCC 14
> incompleto que quebra o link). Em outra máquina, use os presets oficiais
> `linux-clang-debug-strict` / `linux-clang-release-hardened` direto.

## Verificação rigorosa (antes de considerar algo pronto)

```bash
rustup run stable cargo kw-fmt      # formatação Rust
rustup run stable cargo kw-clippy   # lints Rust (zero warnings)
rustup run stable cargo kw-test     # testes Rust
clang-format --dry-run --Werror ui/src/*.cpp ui/src/*.h   # formatação C++
```

## Dependências do sistema (já instaladas nesta máquina)

```bash
# Pop!_OS / Ubuntu / Debian
sudo apt install cmake ninja-build clang \
    qt6-base-dev qt6-declarative-dev \
    qml6-module-qtqml qml6-module-qtqml-workerscript qml6-module-qtqml-models \
    qml6-module-qtquick qml6-module-qtquick-controls \
    qml6-module-qtquick-layouts qml6-module-qtquick-window

# Arch / CachyOS (alvo principal do projeto)
sudo pacman -S cmake ninja clang qt6-base qt6-declarative qt6-tools rustup
```

## Logs de erro da IDE

Erros e mau funcionamento da própria IDE (crash do core, falha de processo,
respostas IPC inválidas) são gravados automaticamente com timestamp em:

```text
~/.cache/kinein-vectis/logs/kinein-ui-erros.txt
```

Anexe esse arquivo ao investigar problemas. A aba "IDE" do painel inferior
mostra o fluxo interno ao vivo (ferramenta de desenvolvimento da IDE); os
erros do *seu projeto* aparecem nas abas Build/Problemas via eventos do core.

## Como a UI encontra o core

Ordem de busca do binário `kinein-core`:

1. variável de ambiente `KINEIN_CORE_BIN` (o launcher já define);
2. diretório do executável da UI;
3. `target/debug/kinein-core` relativo ao diretório atual;
4. `PATH`.

Se a UI mostrar "kinein-core nao encontrado", compile o core
(`cargo build --release -p kinein-core`) ou use o launcher.

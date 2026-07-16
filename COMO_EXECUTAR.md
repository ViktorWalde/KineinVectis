# Como executar o Kinein Vectis

> Vai **usar** a IDE (atalhos, funções, troubleshooting)? O guia completo é
> o [MANUAL.md](MANUAL.md). Este arquivo cobre só build/execução.

O core da IDE é **offline e local**. Você não precisa ligar nada antes: ao
abrir a UI, ela mesma inicia o `kinein-core` (Rust) como processo filho e
conversa com ele por JSON-RPC. Fechou a UI, o core morre junto. A exceção é
uma CLI externa iniciada explicitamente no KV Context, que pode usar rede de
acordo com a política da própria ferramenta.

## Jeito mais simples: clicar no ícone certo

As duas formas de execução podem coexistir no menu sem se sobrescrever:

- **Kinein Vectis** abre o AppImage instalado para validar o mesmo artefato
  entregue aos testadores;
- **Kinein Vectis (Desenvolvimento)** executa `scripts/kinein-vectis` deste
  checkout, usando os binários locais recompilados.

Instale ou atualize somente o atalho de desenvolvimento com:

```bash
./scripts/instalar-atalho.sh
```

O script usa o desktop id próprio `kinein-vectis-development.desktop`. Uma
entrada antiga só é migrada quando aponta comprovadamente para este checkout;
o atalho `kinein-vectis.desktop` do AppImage é preservado.

## Pelo terminal

```bash
./scripts/kinein-vectis
```

Esse launcher pertence ao checkout: usa os binários release locais e cai para
os de debug se os release não existirem. Ele nunca executa o AppImage.

## Compilar do zero (quando mudar o código)

Sequência curta para copiar/colar: veja
[`docs/build/COMANDOS_BUILD_VERIFICACAO.md`](docs/build/COMANDOS_BUILD_VERIFICACAO.md).

Se existir qualquer dúvida sobre cache antigo ou UI/core de momentos
diferentes, prefira o comando integral:

```bash
./scripts/atualizar-tudo.sh
```

Ele reconfigura, reconstrói, testa e valida o mesmo launcher usado pelo atalho
de desenvolvimento. Ao terminar, confira
`build/kinein-build-manifest.env` para os hashes das UIs e cores Debug/Release
produzidos na mesma transação.

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

Para desenvolvimento (build debug local):

```bash
cmake --preset dev-local
cmake --build --preset dev-local
./build/dev-local/ui/kinein-vectis
```

> Os presets `dev-local*` estão em `CMakeUserPresets.json` (arquivo local,
> fora do git). Eles herdam os presets estritos oficiais, mas nesta máquina
> usam GCC nativo com warnings-as-errors e sanitizers desligados, porque o
> toolchain recente dispara warnings em código gerado pelo Qt (detalhe em
> `ContextoIA.md`, seção "Toolchain local"). Em outra máquina, prefira os
> presets oficiais `linux-clang-debug-strict` / `linux-clang-release-hardened`
> (debug oficial liga ASan/UBSan). Se o build reclamar de caminhos de outra
> distro (ex.: `/usr/lib/x86_64-linux-gnu/...`), apague o diretório em
> `build/` e reconfigure do zero — reconfigurar por cima não limpa o cache.

## Verificação rigorosa (antes de considerar algo pronto)

```bash
rustup run stable cargo kw-fmt      # formatação Rust
rustup run stable cargo kw-clippy   # lints Rust (zero warnings)
rustup run stable cargo kw-test     # testes Rust
clang-format --dry-run --Werror ui/src/*.cpp ui/src/*.h   # formatação C++
```

## Dependências do sistema

Esta seção é para quem compila a IDE a partir do repositório. Uma distribuição
binária futura deverá incluir a UI, o core e o runtime Qt, de modo que o usuário
não precise instalar Rust ou Qt só para abrir a Kinein. As ferramentas dos
projetos — compilador C/C++ ou Rust, CMake/Ninja, LSP e debugger — permanecem
externas e são instaladas conforme a linguagem usada.

Atalho para tudo isso (detecta a distro, instala o que falta e verifica):

```bash
./scripts/instalar-ambiente.sh            # use --dry-run para só ver os comandos
```

Ou manualmente:

```bash
# Arch / CachyOS (alvo principal do projeto e máquina atual)
sudo pacman -S cmake ninja clang qt6-base qt6-declarative qt6-tools rustup

# Debian / Ubuntu / Pop!_OS (referência)
sudo apt install cmake ninja-build clang \
    qt6-base-dev qt6-declarative-dev \
    qml6-module-qtqml qml6-module-qtqml-workerscript qml6-module-qtqml-models \
    qml6-module-qtquick qml6-module-qtquick-controls \
    qml6-module-qtquick-layouts qml6-module-qtquick-window
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

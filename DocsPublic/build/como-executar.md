# Como executar o Kinein Vectis

> Vai **usar** a IDE (atalhos, funções, troubleshooting)? O guia completo é
> o [MANUAL.md](../manual.md). Este arquivo cobre só build/execução.

O core da IDE é **offline e local**. Você não precisa ligar nada antes: ao
abrir a UI, ela mesma inicia o `kinein-core` (Rust) como processo filho e
conversa com ele por JSON-RPC. Fechou a UI, o core morre junto. A exceção é uma
CLI externa que você mesmo iniciar no terminal da IDE (um agente de IA, por
exemplo), que usa rede de acordo com a política da própria ferramenta.

## Jeito mais simples: clicar no ícone certo

As duas formas de execução podem coexistir no menu sem se sobrescrever:

- **Kinein Vectis** abre o AppImage instalado para validar o mesmo artefato
  entregue aos testadores;
- **Kinein Vectis (Desenvolvimento)** executa `scripts/kinein-vectis` deste
  checkout, usando os binários locais recompilados.

> **Qual binário o atalho sobe (corrigido em 2026-08-29).** O launcher escolhe,
> nesta ordem: `linux-clang-release-hardened` → `dev-local-release` →
> `dev-local` → `linux-clang-debug-strict`. Antes ele pulava os dois
> `dev-local*` — justamente os que o `scripts/verificar.sh` compila por padrão —
> e caía no build **sanitized** (ASan/UBSan), que é vários vezes mais lento, sem
> avisar. Agora ele **imprime** qual binário subiu e de quando ele é, e alerta
> se for o sanitized. Era a metade silenciosa da armadilha registrada no
> `PONTO_ATUAL` ("horas com uma IDE quebrada porque esse binário estava 4
> commits atrás").

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
[`comandos-de-build-e-verificacao.md`](comandos-de-build-e-verificacao.md).

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
> `DocsPrivate/ContextoIA.md`, seção "Toolchain local"). Em outra máquina, prefira os
> presets oficiais `linux-clang-debug-strict` / `linux-clang-release-hardened`
> (debug oficial liga ASan/UBSan). Se o build reclamar de caminhos de outra
> distro (ex.: `/usr/lib/x86_64-linux-gnu/...`), apague o diretório em
> `build/` e reconfigure do zero — reconfigurar por cima não limpa o cache.

## Verificação rigorosa (antes de considerar algo pronto)

```bash
cargo kw-fmt      # formatação Rust      (nao use `rustup run stable`: ele
cargo kw-clippy   # lints Rust (0 warnings)  ignora o pin do rust-toolchain.toml)
cargo kw-test     # testes Rust
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
# Arch / CachyOS (alvo principal do projeto)
sudo pacman -S cmake ninja clang qt6-base qt6-declarative qt6-tools rustup

# Debian / Ubuntu / Pop!_OS (referência)
sudo apt install cmake ninja-build clang \
    qt6-base-dev qt6-declarative-dev \
    qml6-module-qtqml qml6-module-qtqml-workerscript qml6-module-qtqml-models \
    qml6-module-qtquick qml6-module-qtquick-controls \
    qml6-module-qtquick-layouts qml6-module-qtquick-window
```

## AppImage: gerar ou baixar

Em 2026-09-16, `ViktorWalde/KineinVectis` tinha **zero releases** na API do
GitHub. Portanto, não há AppImage publicado para baixar nesta data.
Com Podman ou Docker instalado, o fluxo existente para gerar localmente é:

```bash
bash scripts/empacotar-appimage.sh
bash scripts/testar-appimage.sh
(cd dist && sha256sum -c Kinein-Vectis-0.1.0-x86_64.AppImage.sha256)
./dist/Kinein-Vectis-0.1.0-x86_64.AppImage
```

O primeiro build baixa o builder Debian 12, dependências e ferramentas
fixadas pelo projeto. A validação acima deve passar antes de distribuir.
Para acrescentar o artefato ao menu: `bash scripts/instalar-appimage.sh dist`.
Sem FUSE, executar com `APPIMAGE_EXTRACT_AND_RUN=1` antes do caminho do
AppImage. A versão `0.1.0` corresponde ao projeto nesta data.

**Somente depois de uma release ser publicada**, este comando baixa o
AppImage e seu checksum usando o GitHub CLI:

```bash
gh release download --repo ViktorWalde/KineinVectis \
  --pattern '*.AppImage' --pattern '*.AppImage.sha256' \
  --dir "$HOME/Downloads/KineinVectis"
cd "$HOME/Downloads/KineinVectis"
sha256sum --check ./*.AppImage.sha256
chmod u+x ./*.AppImage
# Execute o nome exato do arquivo baixado, por exemplo:
./Kinein-Vectis-0.1.0-x86_64.AppImage
```

Para o estado das ferramentas no notebook e os gates ainda pendentes, ler
[ambiente de desenvolvimento](14-development-environment.md).

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

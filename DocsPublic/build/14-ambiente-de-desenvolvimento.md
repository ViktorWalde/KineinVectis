# 14 — Ambiente de desenvolvimento

O ambiente local deve refletir o modo de segurança máximo do projeto.

**Ambiente vigente em 2026-10-24:** Ubuntu 26.04.1 e Qt 6.12.0 oficial em
`~/Qt/6.12.0/gcc_64`. O Qt do sistema (6.10.2) não atende ao projeto.
Decisão e desenho no [52 §14.49](../roadmaps/52-arquitetura-executavel-da-0.4.md#1449-041h4b2b4l--o-ambiente-no-qt-do-projeto-o-612-desenho-2026-10-24).
As observações datadas de outras distribuições abaixo são históricas.

## Bootstrap ao trocar de distro/máquina

`scripts/instalar-ambiente.sh` instala todas as dependências abaixo com
detecção de distro (apt no Debian/Ubuntu, pacman no Arch/CachyOS,
dnf no Fedora), aceita `--dry-run` (só mostra os comandos) e
`--extras` (o Podman, o motor de containers rootless; o `cargo-deny` saiu de
`--extras` em 2026-08-30 e o `shellcheck` em 2026-10-15, porque o gate depende
dos dois), e termina verificando ferramenta por ferramenta o mesmo conjunto que
o ToolDetector do core reporta, mais o que os gates usam (o runtime `qml`, o
`xvfb-run`, o `xdotool`, o `shellcheck`) e um LINK de verdade com o `clang++`:
o clang escolhe o GCC mais novo de `/usr/lib/gcc/<alvo>/`, e no Ubuntu 26.04 o
16 vem sem a `libstdc++` dele — a verificação diz o pacote que falta
([59](../roadmaps/59-fechamento-da-0.3.9.md) §2.6). O `rust-analyzer` vem do
`rust-toolchain.toml`: o rustup instala os componentes que ele lista no
primeiro `cargo` dentro do repositório (medido no rustup 1.29.1). Depois dele:
`cmake --preset dev-local && cmake --preset dev-local-release` e
`scripts/verificar.sh`. Lembrete: build dir de outra distro é lixo — apagar e
reconfigurar: o CMake guarda caminhos absolutos da distro no cache.

## Rust

Toolchain definido em `rust-toolchain.toml`:

O mínimo do workspace é Rust **1.88** (corrigido em 2026-10-06): o código
já usa let chains e dependências como MongoDB/ICU/time já exigiam 1.88.
O antigo `rust-version = 1.85` não refletia o código compilável.

```bash
cargo --version
cargo kw-fmt
cargo kw-check
cargo kw-clippy
cargo kw-test
```

Os aliases `kw-*` estão em `.cargo/config.toml`.

> **Não use `rustup run stable`** (corrigido em 2026-08-29). O
> `rust-toolchain.toml` fixa uma versão exata, e `rustup run stable` **força** o
> canal `stable`, ignorando o pin: você rodaria o gate num compilador diferente
> do que o projeto declara. Numa máquina onde só a toolchain fixada está
> instalada — o caso depois do `instalar-ambiente.sh` — o comando simplesmente
> falha com *"toolchain 'stable' is not installed"*. `cargo` puro já respeita o
> arquivo: é o rustup que resolve o pin por você.

## C++/Qt/QML

O core também liga dinamicamente o gerenciador unixODBC do sistema.
Para compilar, instale `unixodbc-dev` (Debian/Ubuntu), `unixODBC-devel`
(Fedora) ou `unixodbc` (Arch). A IDE lista DSN existentes; instalar e
configurar o driver do banco é responsabilidade da pessoa. Não existe
download automático de driver. A decisão e a licença estão no
[ADR-0007](../decisoes-adr/ADR-0007-odbc-com-consentimento.md).

Dependências de compilação no Ubuntu (instalação pelo autor):

```bash
sudo apt install cmake ninja-build clang lldb gdb libgl1-mesa-dev libegl1-mesa-dev libxcb-cursor0 unixodbc-dev
```

O Qt vem do instalador oficial ou do `aqtinstall` no usuário. Confira a
lista de módulos antes do download; não combine bibliotecas do sistema com
o prefixo oficial. Para completar uma instalação existente:

```bash
pipx install aqtinstall
aqt list-qt linux desktop --modules 6.12.0 linux_gcc_64
aqt install-qt linux desktop 6.12.0 linux_gcc_64 -m qtwebengine qtwebchannel qtpositioning -O "$HOME/Qt"
```

`cmake/KineinQt.cmake` é o dono da versão 6.12 e da escolha do prefixo:
`-DKINEIN_QT_PREFIX=/caminho` ou o mais recente `~/Qt/6.12.*/gcc_64`.
Cada build grava `kinein-qt-bins.txt`; os gates usam as ferramentas dali.
Reconfigure os três builds com `cmake --fresh --preset dev-local`,
`dev-local-release` e `linux-clang-debug-strict`. Não basta conservar um
cache CMake que ainda aponta para o Qt do sistema.

Presets previstos:

```bash
cmake --preset linux-clang-debug-strict
cmake --build --preset debug-strict
ctest --preset debug-strict

cmake --preset linux-clang-release-hardened
cmake --build --preset release-hardened
```

Todo alvo C++/Qt deve chamar:

```cmake
include(cmake/KineinStrictOptions.cmake)
kinein_enable_strict_compiler_options(nome_do_target)
```

## Medição de performance (`scripts/medir-performance.sh`)

Mede, 100% local (offscreen + stdio + `/proc`, **zero telemetria/rede**),
startup da UI, `workspace.open`, `fs.read` de arquivo grande e RSS
(UI + core + LSP). Cada métrica roda `KINEIN_PERF_N` vezes (default 5) e
reporta a mediana; imprime a tabela e sai. Precisa dos binários compilados
(`cmake --build build/dev-local` + `cargo build -p kinein-core`). O
ORÇAMENTO (números-alvo) e as métricas manuais (latência de digitação)
vivem em `DocsPublic/roadmaps/21`, seção "M4.2". Detalhe do gancho `KINEIN_PERF_MARKER`
(marker env-gated na `main.cpp`, sem efeito no uso normal): o comentário de
`installStartupPerfMarker` em `ui/src/main.cpp`.

## Estado observado no notebook (Ubuntu 24.04.5, 2026-09-16)

**Atualização de 2026-09-17 — o sistema mudou:** a máquina está em **Ubuntu
26.04.1**, Qt **6.10.2**, Clang **21.1.8** (clang-18 ainda instalado), GCC 15.2,
CMake 4.2, Python 3.14.4. Tudo abaixo que fala de Qt 6.4.2/Clang 18 é registro
da época. Dois efeitos medidos: (1) os presets `linux-clang-*` **não
configuram** — `clang++` escolhe a instalação GCC 16 (existe `libgcc-16-dev`)
e não há `libstdc++-16-dev`, logo `ld: cannot find -lstdc++` no try-compile;
remédio `sudo apt-get install libstdc++-16-dev` e reconfigurar os dois presets;
(2) o `dev-local` (g++) compila a UI com zero avisos, o qmllint 6.10 fica
**limpo** (os avisos StandardKey do 6.4 não existem nele) e o binário abre.
Shims do pipx (`mpremote`) apontam para o Python antigo: `pipx reinstall-all`.
Detalhe no roadmap 40 §7.39. **À tarde**, com `libstdc++-16-dev` instalado, os
dois presets clang configuraram e compilaram; a única correção de código foi a
isenção do `-Wctad-maybe-unsupported` no `.moc` inline do
`typing_perf_harness.cpp` (moc do Qt 6.10). Clang-Tidy e qmllint nativo
voltaram a rodar (40 §7.40). O `verificar-qml.sh` agora testa o `qmllint`
candidato com `--version`: o do PATH é um wrapper Qt 5 sem alvo.

**Atualização posterior — debugpy attach, protocolo 0.109.0:** core e UI debug/release
recompilados; atalho usa a release nova. Attach real (breakpoint, locais, evaluate,
reconexões e desconexão preservando o processo) passou com debugpy 1.8.0. Foram
aprovados 723 testes Rust, 35 harnesses QML e Clang-Tidy dos arquivos C++ desta
fatia. Primeiro frame offscreen: 3151 ms debug / 988 ms release, amostras pontuais.
O lint QML ainda reprova nos dois avisos StandardKey conhecidos; os demais
impedimentos anteriores permanecem descritos abaixo. Detalhes no roadmap 40 §7.38.

Ambiente atual: Ubuntu x86_64, Rust 1.96.1, Clang 18.1.3 e Qt 6.4.2.
CMake/Ninja, Qt dev, libfmt-dev, OpenSSL dev, cargo-deny, ShellCheck,
rustfmt, Clippy e rust-analyzer estão disponíveis; rust-src foi acrescentado.

O bootstrap acima continua sendo a entrada para uma máquina limpa. No
Ubuntu, conferir também as dependências abaixo; algumas já estavam
instaladas neste notebook antes da preparação:

```bash
sudo apt-get install --yes libfmt-dev libssl-dev pkg-config \
  python3-venv pipx python3-pytest python3-debugpy qml-qt6 \
  podman uidmap slirp4netns fuse-overlayfs \
  qemu-system-arm gcc-arm-none-eabi gdb-multiarch \
  patchelf libfuse2t64 libxcb-cursor0 \
  qml6-module-qt-labs-platform qml6-module-qtquick-dialogs
pipx install 'ruff==0.16.7'
pipx install 'basedpyright==1.40.1'
pipx install 'uv==0.12.15'
pipx install 'mpremote==1.29.0'
pipx install 'esptool==5.4.0'
rustup component add --toolchain 1.96.1 rust-src
```

As ferramentas Python usam ambientes pipx separados. `~/.local/bin` e
`~/.cargo/bin` precisam estar no PATH; já estavam neste notebook. Foram
criados os nomes `fd` (para `/usr/bin/fdfind`) e `lldb-dap` (para
`/usr/lib/llvm-18/bin/lldb-dap`) em `~/.local/bin`, sem substituir binários
do sistema. Podman 4.9.3 executou um contêiner Alpine sem root.

**Validação medida:** UI debug compilada e primeiro frame offscreen em
2874 ms; teste real de debugpy passou (breakpoint, variáveis, evaluate,
stdout e execução de módulo). Ruff/basedpyright passaram na prova LSP real.
UI e core release também foram compilados; a UI chegou ao primeiro frame
em 1071 ms e o launcher real permaneceu vivo por oito segundos offscreen.
**Retomada de compatibilidade, 2026-09-16:** o pacote `qml-qt6` foi
instalado e incluído no bootstrap apt. O runner de lógica prefere
`/usr/lib/qt6/bin/qml` ao wrapper `qml` do Qt 5; os **34 harnesses QML
passaram**. O lint agora aceita builds sem `.rsp`, usando o alvo CMake/JSON
e rejeitando avisos, relatório vazio/inválido ou resultado antigo. Dependências
QtQuick/QtQuick.Window explícitas, conversão de URL e animação com alvo
explícito resolveram cinco das sete advertências reveladas pelo lint 6.4.

O harness de digitação entrega amostras por sinal tipado queued, mantendo o
carimbo na render thread; seu Clang-Tidy passou. A limpeza redundante do
QPointer em `destroyed` foi removida; troca/destruição de janelas passaram em
prova com ASan/UBSan. Os builds atualizados chegaram ao primeiro frame em
2891 ms (debug) e 1070 ms (release), medições pontuais offscreen.

A prova do harness revelou e corrigiu a busca do editor no contexto errado
(agora usa `domains.editorController`) e a saída por erro antes de `app.exec()`.
Uma tecla inseriu um caractere e gerou amostra; a sequência de cinco teclas
continua expirando após a primeira amostra. A medição completa ainda não é
um benchmark validado neste notebook.

O **gate completo continua pendente**:

- Clang-Tidy 18/Qt 6.4.2: `NewDelete` na atribuição de QPointer em
  `window_chrome_controller.cpp:47`; a prova de runtime não encerra a
  investigação desse diagnóstico;
- qmllint 6.4: dois avisos de acesso não qualificado em
  `GlobalShortcuts.qml:25/31`, ambos `StandardKey`. Um exemplo mínimo
  executa com sucesso no runtime 6.4, mas reprova no lint da mesma versão;
- o teste QEMU não para com GDB nativo; com gdb-multiarch 15.1 chega ao
  breakpoint, mas retorna registradores em vez da global `contador`
  (medição anterior, não repetida nesta fatia).

Não confundir dependência instalada, build que abre e gate completo verde.
Os comandos de execução estão em [como executar](como-executar.md).
O que um Ubuntu limpo exige dos gates, medido em 2026-10-01, está na tabela
"o que o gate assume da máquina" de
[`contribuindo/04`](../contribuindo/04-os-gates-que-dizem-nao.md).

## Histórico observado (Fedora 44, 2026-08-29 a 2026-09-04)

Este bloco preserva a medição do ambiente anterior. Não descreve o notebook
Ubuntu atual nem comprova os gates nele.

- Fedora Linux 44 (Workstation), kernel 7.1.
- Rust 1.96.1 (a toolchain fixada), rustfmt e clippy via `cargo` direto.
- CMake e Ninja disponíveis; Clang 22 (`clang++`, `clang-format`, `clang-tidy`,
  `clangd`), GCC, GDB, LLDB e `lldb-dap` no PATH.
- `ripgrep` e `fd` no PATH; ShellCheck instalado.
- Qt 6.11.1 **runtime** presente; os pacotes **`-devel` não estavam
  instalados** — sem eles não há `qmllint` nem `qmake6`, e os gates
  `verificar-cpp.sh`, `verificar-qml.sh` e `verificar-qml-logica.sh` não rodam.
  É o que `scripts/instalar-ambiente.sh` resolve.
- **`scripts/verificar.sh` completo: VERDE em 2026-09-04**, depois de
  `scripts/instalar-ambiente.sh` — inclui `verificar-cpp.sh` (clang-format +
  clang-tidy), `verificar-qml.sh` (qmllint estrito) e `verificar-qml-logica.sh`
  (24 harnesses headless, medido em 2026-09-04; `ls scripts/qml-harness/ | wc -l`).
  Desde 2026-09-04 o runner monta um **espelho plano** do módulo
  `KineinVectis` a partir das fontes, então o harness também alcança
  componente **visual** — o que antes era impossível, porque o módulo só
  existia dentro do `qrc` do binário compilado.

> **Armadilha do `qmllint`, medida em 2026-08-29.** O response file do lint
> nasce na **configuração**; o `kinein-vectis.qmltypes` do módulo só na
> **compilação**. Um build dir configurado e não compilado faz o qmllint
> despejar uma parede de *"QML types file does not exist"*, que parece defeito
> no QML e é build faltando. O `verificar-qml.sh` passou a exigir os dois e a
> dizer qual `cmake --build` rodar.

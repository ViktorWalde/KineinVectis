# `toolchain.*` — o domínio no protocolo IPC

> Parte do [03](../03-protocolo-ipc.md), que tem o formato base, as listas
> canônicas de métodos e eventos e as regras. Movido sem mudar o texto em
> 2026-10-16 ([61](../../roadmaps/61-organizacao-da-documentacao.md) §3.7, O6).

### Toolchain (`toolchain.get` / `toolchain.set` / `toolchain.setKit` / `toolchain.installable` / `toolchain.install` / `toolchain.inspectSysroot` / `toolchain.importKit`)

Implementado no protocolo `0.64.0` (etapa 5 de
`DocsPublic/roadmaps/30-caminho-para-o-mvp.md`; B2 do TR2 e §5d do `roadmaps/29`).
Ambos exigem workspace aberto. Persistência em `.kinein/toolchain.json` com
`schemaVersion` — arquivo inválido ou de schema desconhecido é tratado como
vazio, e toolchain quebrada nunca impede a IDE de abrir o projeto.

**A pergunta que o domínio responde:** *qual executável cumpre cada papel?*
Até 2026-09-02 todo processo externo do core nascia de `Command::new("<nome>")`
— 28 chamadas, todas resolvidas pelo `PATH` do processo. Trocar de compilador
significava editar `CMakeLists.txt` à mão ou exportar `CC`/`CXX` antes de abrir
a IDE.

Os **papéis** são vocabulário fechado (`cCompiler`, `cxxCompiler`, `generator`,
`cmake`, `cargo`). Papel novo é entrada nova no protocolo e no catálogo do
core, nunca string livre vinda da UI: um papel que o core não sabe usar daria
ao usuário um seletor sem efeito.

- `toolchain.get {}` e `toolchain.set { role, id? }` respondem o **mesmo**
  shape — como as run configs, a UI nunca calcula estado derivado:

```text
{ selections: [{ role, id?, resolvedPath? }],
  candidates: [{ role, id, label, path?, version? }] }
```

- `id` **ausente** em `selections` é AUTOMÁTICO, e é o padrão: nada é fixado e
  o `PATH` continua decidindo — exatamente o comportamento histórico. Quem
  nunca abrir o seletor não vê diferença nenhuma.
- `resolvedPath` no automático é **informação** (o primeiro candidato
  detectado), não fixação. É o que a UI mostra para dizer o que vai acontecer.
  **Desde o `0.172.0`**, nos papéis que o CMake decide (`cCompiler`,
  `cxxCompiler`, `generator`), o automático é o que o `CMakeCache.txt` do
  projeto diz (`.kinein/build`): `resolvedPath` é o compilador (no gerador, o
  `CMAKE_MAKE_PROGRAM`) e `effectiveId` o candidato detectado que é o mesmo
  arquivo (no gerador, o `CMAKE_GENERATOR` quando é candidato); antes do
  primeiro configure, os dois ficam ausentes. O primeiro detectado não é o
  que o CMake usa: sem nada fixado, nada vai para a linha do `cmake`
  (decisão do autor, 2026-10-16; `roadmaps/59` §7.1.7.1).
- `toolchain.set` com `id` ausente volta para automático. Com um `id` que não
  existe no catálogo, ou que existe mas **não foi detectado nesta máquina**,
  responde `INVALID_PARAMS`: oferecer um compilador ausente é oferecer um
  configure que vai falhar.

**A escolha passou a ser do KIT, não do workspace (protocolo `0.67.0`).** Um kit
é um preset do CMake mais o que ele precisa para compilar: os executáveis por
papel, o `sysroot` e o triple do alvo.

```text
toolchain.get  { preset? }                         -> ToolchainResult
toolchain.set  { role, id?, preset? }              -> ToolchainResult
toolchain.setKit { preset?, sysroot?, targetTriple?, chip?,               NOVO
                   remoteTarget?, debugServer?, toolchainFile?, svdFile? } -> ToolchainResult
```

**Desde o `0.97.0` (2026-09-12, [`integracoes/39`](../../integracoes/39-toolchains-por-alvo.md))
o `ToolchainResult` carrega dois campos que só um processo responde:**
`rustTargets` — os alvos Rust INSTALADOS (`rustup target list --installed`,
pelo `rustup` detectado; **ausente** sem rustup, e a UI então não avisa nada;
com o `targetTriple` do kit fora da lista a UI diz o `rustup target add`) — e
`sysrootHint` — quando o compilador C efetivo é `*-linux-gnu*`, o kit não tem
`sysroot` e o sysroot que o próprio compilador declara (`-print-sysroot`) não
tem `usr/include`: é como as distros empacotam o `gcc-aarch64-linux-gnu`
(medido no Fedora 44), compila e não linka programa de usuário nenhum, e a
dica nomeia as três saídas (rsync da placa, Bootlin, SDK Yocto/Buildroot).
Bare metal não entra. Os candidatos de `cCompiler`/`cxxCompiler`/`debugAdapter`
ganharam no mesmo dia os triples da indústria (`riscv-none-elf`,
`xtensa-esp-elf`, `riscv32-esp-elf`, `aarch64-linux-gnu`,
`arm-linux-gnueabihf`, `riscv64-linux-gnu`; `gdb-multiarch` e os
`<triple>-gdb`), e o detector procura além do `PATH` (o store do xpm, o
`~/.espressif/tools`, a pasta da IDE, `/opt/*/bin`).

`preset` ausente é **o kit padrão do workspace** — que é exatamente o que o
schema 1 do `.kinein/toolchain.json` guardava, e por isso a migração é direta:
as seleções antigas viram o kit padrão **na leitura**, não na escrita. Migrar na
leitura é o que impede a IDE de perder a escolha de quem abriu o projeto e não
mexeu na toolchain.

**Em `toolchain.setKit`, campo ausente NÃO é campo vazio.** Ausente preserva o
valor atual; string vazia (ou só de espaços) limpa. Sem essa distinção, mexer no
`sysroot` apagaria o `targetTriple` e o usuário só descobriria no próximo build.

**O kit chega aos language servers** (`handlers/lsp/toolchain.rs`, chamado ao
abrir o workspace e a cada `toolchain.set`/`setKit`): o compilador cross ao
clangd por `--query-driver` (desde 2026-09-11) e, **desde `0.115.0` (P0)**, o
`targetTriple` ao **rust-analyzer** — `rust-analyzer.cargo.target = <triple>`
("Compilation target override (target tuple)", manual do rust-analyzer,
conferido em 2026-09-17), entregue como o basedpyright recebe a dele: secção
`rust-analyzer` respondida no `workspace/configuration` e empurrada no
`workspace/didChangeConfiguration`. O `cargo build` do kit já levava
`--target`; o servidor checando para o host daria diagnóstico do host, que
parece certo e não é. Vale na próxima subida; um servidor de Rust vivo é
**reiniciado** quando o kit muda (é a UI que o sobe de novo ao re-sincronizar
o arquivo ativo, como em qualquer `event.lsp.restarted`); sem triple, o
servidor sobe sem configuração, como sempre.

**O papel `debugAdapter` entrou no protocolo `0.69.0`** (etapa 22 do
`roadmaps/35`). Antes, o adaptador DAP era uma **constante** no core
(`ADAPTER_BINARY = "lldb-dap"`), e por isso não havia debug de embarcado
nenhum: embarcado não debuga com `lldb-dap`.

```text
lldb-dap    desktop; sem argumento          (o PADRAO — nada muda sem escolha)
probe-rs    embarcado; `probe-rs dap-server` fala DAP por stdin/stdout
gdb         `gdb -q -iex ... -i dap`; todo GDB (`gdb-multiarch`, `<triple>-gdb`)
debugpy     Python (0.101.0); NAO vem do kit: o programa e' o INTERPRETADOR do
            projeto e o adaptador e' `-m debugpy.adapter` (DAP por stdin/stdout)
```

**O catálogo da toolchain NÃO sabe os argumentos, e a fronteira é deliberada.**
O cabeçalho dele diz que responde *"quais binários interessam a cada papel"* —
não *"com quais argumentos"*. Que o `probe-rs` precisa do subcomando
`dap-server` é conhecimento de quem **sobe o processo**, e isso mora no domínio
`dap/`.

**O kit ganhou `chip` no protocolo `0.70.0`** — o alvo do adaptador de
embarcado. Ele vai no **`launch` do DAP**, não na linha de comando: verificado
na documentação do probe-rs, onde `chip` é campo da configuração de launch e não
flag do `dap-server`. Supor o contrário daria um processo que sobe e falha no
primeiro request, com a causa longe do sintoma. **Campo ausente não é campo
nulo** — sem chip escolhido, o `launch` não carrega a chave, mesma regra da
condição de breakpoint (`0.66.0`).

**O `launch` do probe-rs tem a forma DELE (`0.118.0`, P3; medido no
`dap-server` 0.32.0 desta máquina e lido em `server/configuration.rs`):**
`program`+`chip` no topo — a forma que a IDE mandava até aqui — falha com
`Serialization error "missing field coreConfigs"`. A forma certa é `{ cwd,
chip?, coreConfigs: [{ coreIndex: 0, programBinary: <ELF>, svdFile?,
rttEnabled: true }] }`; os outros adaptadores continuam com `program`/`cwd`.
**O kit ganhou `svdFile` (`0.118.0`)** — o CMSIS-SVD do chip (do pack do
fabricante ou do `cmsis-svd`, cada um com a sua licença, `35` §5.7), que o
probe-rs transforma no escopo `Peripherals` (`debug.scopes`). Como o
`toolchainFile`: `toolchain.setKit { svdFile? }` ("" limpa, ausente
preserva), `ToolchainResult.svdFile?`, campo **SVD** no painel de Embarcados.

**O kit ganhou `remoteTarget` e `debugServer` no `0.89.0`** — o caminho para
depurar o que não está na máquina. Quando o kit escolhe o adaptador `gdb` (que
fala DAP nativamente desde a v14, fonte `/usr/share/doc/gdb/NEWS`) e declara um
`remoteTarget` (`host:porta`), o `debug.start` faz **`attach`** com esse alvo —
*"passed to the `target remote` command"* (manual do GDB, capítulo Debugger
Adapter Protocol) — em vez de `launch`. O `debugServer` é o comando do servidor
(QEMU, OpenOCD) que a IDE sobe antes de conectar, com `{program}` trocado pelo
ELF; ela o mata com a sessão. **Ambos são declarados, nunca deduzidos** (a IDE
não sabe qual máquina do QEMU é a placa): `roadmaps/35` §5.1. Isso derruba o que
o `integracoes/36` §3 dizia — não há protocolo GDB-remote novo a escrever, é um
segundo candidato do papel `debugAdapter`.

**Cross-compilador NÃO virou papel novo**, e a medição corrigiu o esboço do
`roadmaps/35` §5.3 que dizia que sim: `arm-none-eabi-gcc` escreve a **mesma**
variável que o `gcc` — `CMAKE_C_COMPILER`. Dois papéis apontando para a mesma
variável seria duplicação. O cross é **candidato** dos papéis que já existem.

**Adaptador desconhecido não é recusado:** roda sem argumento, que é a forma da
maioria dos adaptadores DAP. Recusar o que não está na lista impediria o usuário
de apontar um adaptador que a IDE não conhece — e a lista é curta por ser nova,
não por ser completa.

O que o kit vira, no build:

```text
sysroot        -> -DCMAKE_SYSROOT=<caminho>
targetTriple   -> --target <triple>          (cargo: check, clippy e build)
               -> -DCMAKE_SYSTEM_NAME=<...>  quando o triple é inequívoco
               -> -DCMAKE_SYSTEM_PROCESSOR=<arch>
```

**Bare metal ganha `CMAKE_TRY_COMPILE_TARGET_TYPE=STATIC_LIBRARY`, e sem isso o
configure falha SEMPRE.** Medido em 2026-09-03 exercitando
`thumbv7em-none-eabihf` com o `arm-none-eabi-gcc` real: `CMAKE_SYSTEM_NAME=
Generic` sozinho não basta — o CMake ainda tenta **linkar** um executável no
teste de compilador, e bare metal não tem os stubs do newlib
(`undefined reference to _exit`). O projeto nem chega a ser configurado. A doc
do CMake diz que `STATIC_LIBRARY` existe exatamente para *"cross-compiling
toolchains that cannot link without custom flags or linker scripts"*.

Só entra em `Generic`. Em cross para Linux/Windows o link funciona, e forçar o
teste a virar biblioteca **esconderia** um toolchain de verdade quebrado.

**Com sysroot, o host não vaza (`0.178.0`, a 0.4.1a do `roadmaps/52` §14.7).**
Quando o kit tem sysroot e põe o CMake em cross, e nem o preset nem o kit
trazem toolchain file, o configure passa também
`CMAKE_FIND_ROOT_PATH_MODE_PROGRAM=NEVER` e `..._LIBRARY`/`..._INCLUDE`/
`..._PACKAGE=ONLY`. Medido em 2026-10-17 (CMake 4.2.3): com o padrão, o
`find_package(ZLIB)` de um build com sysroot achou o `/usr/include` do host;
com os modos, nada — o certo, porque o sysroot não tinha a zlib. Sem sysroot
nada muda: no multiarch do Debian os cabeçalhos dos pacotes `:arm64` moram no
`/usr/include` mesmo.

`CMAKE_SYSTEM_NAME` é o que faz o CMake entrar em modo cross; o sysroot sozinho
não muda a decisão de compilador. **Triple que o mapa não conhece não vira
palpite:** fica sem `CMAKE_SYSTEM_NAME`, e o caminho oficial para alvos exóticos
continua sendo um `toolchainFile` do preset. O `--target` do cargo vale para
check e clippy também — compilar para o alvo e checar para o host daria
diagnóstico do host, que é pior que não ter porque parece certo.

O resultado carrega `presetToolchainFile` quando o preset declara um
`toolchainFile` (campo do schema de presets desde a versão 3 / CMake 3.21, com
precedência sobre `CMAKE_TOOLCHAIN_FILE`). **É informação, não escolha:** quando
ele existe, o compilador efetivo pode não ser o que o usuário escolheu aqui, e a
IDE diz isso em vez de deixar procurar no lugar errado.
- `candidates` traz só o que existe aqui. `Unix Makefiles` só aparece se o
  `make` existir — por isso ele entrou no `tools.detect` na mesma fatia.

**O que a escolha muda de fato**, e é o que faz dela uma entidade e não um
enfeite:

```text
cmake.configure   -DCMAKE_C_COMPILER / -DCMAKE_CXX_COMPILER / -G, e o
                  EXECUTAVEL do proprio cmake
build.run         o cmake (configure implicito + --build) e o cargo
quality.run       o cargo do clippy
cargo.check       o cargo
cargo.metadata    o cargo
```

Só o que o usuário **fixou** entra na linha de comando. Emitir
`-DCMAKE_CXX_COMPILER` com o que o `PATH` resolveria hoje congelaria no cache do
`CMake` uma escolha que o usuário não fez.

**O que esta fatia NÃO entrega, e está registrado:** sysroot, cross-compilação e
kit por preset. São o resto do B2 do TR2, e ficam para uma fatia própria — o
que existe hoje é a entidade e a rota até o comando.

### O provedor de instalação de toolchain (`toolchain.installable` / `toolchain.install`, `0.103.0`)

A fatia do [`integracoes/39`](../../integracoes/39-toolchains-por-alvo.md) §5,
entregue em 2026-09-13 com a regra que a decisão registrada e o pedido do
autor juntos permitem: *"zero-config = DETECTAR + UM CLIQUE com o comando
visível, NUNCA download calado"* (`roadmaps/42` §8).

```text
toolchain.installable {}       -> { installRoot, toolchains: [InstallableToolchain],
                                    projectFamily? }
toolchain.install { id }       -> { jobId }        (job; event.toolchain.installed no fim)

InstallableToolchain   id, label, version, family (cortex-m | riscv | aarch64-linux |
                       arm-linux | riscv64-linux | espressif | rp2040), url, sizeBytes,
                       sha256, license, source, installDir, installed, recommended,
                       kind (toolchain | firmware; 0.116.0),
                       firmware? { board, engine, offset?, chip?, file }
event.toolchain.installed { jobId, id, version, path, success, error? }
```

**Firmware MicroPython no mesmo catálogo (`0.116.0`, C5 do `roadmaps/41`
bloco C).** Cinco entradas `kind: firmware`, v1.29.0 (2026-08-24) do
micropython.org: `ESP32_GENERIC` (esptool, offset `0x1000`, chip `esp32`),
`ESP32_GENERIC_C3` e `ESP32_GENERIC_S3` (esptool, offset `0x0`),
`RPI_PICO` e `RPI_PICO_W` (picotool, `.uf2`). O mesmo provedor baixa
(`.part`, SHA-256 conferido antes de qualquer coisa) e guarda o ARQUIVO
inteiro em `<installRoot>/<id>/<versão>/<arquivo>` (`firmware.file`) — sem
`tar`, sem `bin/`; `installed` é o arquivo existir. **A fonte não publica
checksum** (conferido nas cinco páginas em 2026-09-17): o SHA-256 pinado
foi medido no download desta sessão e o `source` diz isso — protege contra
um download corrompido ou trocado depois da medição, não contra a fonte
ter sido trocada antes. `recommended` só num projeto MicroPython da mesma
família (`espressif`/`rp2040`); a uma app C do ESP-IDF, nunca. Quem grava é
`runConfig.flashProposal { firmware }`.

**O catálogo é PINADO e medido.** Nove entradas (Linux x86_64): Arm GNU
Toolchain 15.2.rel1 (`arm-none-eabi`, `aarch64-none-linux-gnu`,
`arm-none-linux-gnueabihf`), xPack `arm-none-eabi-gcc` 15.2.1-1.1 e
`riscv-none-elf-gcc` 15.2.0-1, Arm Toolchain for Embedded 23.1.0, Bootlin
glibc stable 2026.08-1 (`aarch64`, `armv7-eabihf`, `riscv64-lp64d`). Cada
SHA-256 foi **lido no arquivo que a fonte publica** em 2026-09-13
(`.sha256asc` da Arm, `.sha` do GitHub release da xPack, `.sha256` da ATfE e
da Bootlin) e cada tamanho veio do `Content-Length` de um HEAD no tarball —
`toolchain/install/catalog.rs` guarda a fonte e a data ao lado de cada
número. Não entram: Espressif (o `idf_tools.py` é o instalador oficial e a
IDE já lê `~/.espressif/tools`), Zephyr SDK (precisa do `setup.sh` —
importar kit, P1), e "latest" de qualquer fonte.

**`toolchain.installable` não exige workspace** (catálogo e pasta são desta
máquina); com um, a `family` do `project.model` marca `recommended` —
`stm32`/`rp2040`/`nrf`/`cortex-m` → `cortex-m`, `riscv` → `riscv`;
`espressif` e `linux` não recomendam nada. `installed` é
`<installRoot>/<id>/<version>/bin` existir.

**`toolchain.install` recusa antes de gastar rede:** id fora do catálogo →
`INVALID_PARAMS` apontando o método da lista; já instalada →
`INVALID_REQUEST` dizendo onde está; `tar` ausente → `TOOL_NOT_FOUND`. O job
então: baixa por `ureq` (TLS por rustls; prazo de 60 s **entre bytes**, não
total — um tarball de 400 MB numa rede lenta leva o que levar) para
`<id>/<version>.part/`, calculando o SHA-256 no caminho e reportando
`event.job.progress` por ponto percentual; **confere o digest ANTES de
desempacotar** — divergiu, o erro mostra os dois digests e nada é
desempacotado; `tar -xf … --strip-components=1 -C` (o `tar` do sistema, GPL,
como processo; todos os tarballs do catálogo trazem uma pasta de topo, e é
ela que sai) numa pasta dentro do `.part`; exige `bin/` no resultado; só então
renomeia para `<version>` e apaga o `.part`. Cancelamento, HTTP ≠ 200, disco
e `tar` com erro deixam no máximo um `.part`, que a próxima tentativa apaga.
Provado com as peças REAIS num servidor local: ureq de verdade, SHA-256 de
verdade, `tar` de verdade (`toolchain/install/mod.rs`, testes).

**Depois do sucesso o detector já enxerga** (`tools/search_dirs.rs`,
`installed_bin_dirs`): a pasta da IDE é enumerada a cada busca, não na
construção do detector; o core refaz o registro de ferramentas ao ver
`event.toolchain.installed { success: true }`, e o `toolchain.get` seguinte
lista o compilador novo como candidato — o kit pode fixá-lo. O `PATH` continua
vencendo a pasta da IDE: o que o usuário escolheu vence o que a IDE
encontrou. A UI (`EmbeddedInstallView`) mostra label, versão, tamanho, URL,
sha256 e licença de cada entrada, recomendadas primeiro, e um botão por linha;
uma instalação por vez.

### O gerenciador que lê o disco (`toolchain.inspectSysroot` / `toolchain.importKit`, `0.104.0`)

Os itens (b) e (d) do `roadmaps/42` §8 e o §3 do `integracoes/39`, entregues
em 2026-09-13. Nenhum dos dois exige workspace nem grava nada: leem caminhos
desta máquina e respondem; aplicar é o `toolchain.setKit` de sempre.

```text
toolchain.inspectSysroot { path }  -> SysrootReport { path, exists,
                                      folders { usrInclude, usrLib, lib },
                                      tripleLibDirs[], pkgconfigFiles, libc?, verdict }
toolchain.importKit { path }       -> KitImport { kind (yocto | buildroot | zephyr-sdk | toolchain-dir),
                                      path, evidence[], cCompiler?, cxxCompiler?, gdb?,
                                      sysroot?, targetTriple?, toolchainFile?, hint? }
toolchain.setKit { …, toolchainFile? }   ("" limpa; ausente preserva)
ToolchainResult.toolchainFile?           (o do KIT; presetToolchainFile e' o do preset)
```

**`inspectSysroot` responde "o `--sysroot` aqui vai achar headers e
bibliotecas?"** Lê `usr/include`, `usr/lib`, `lib`, os `usr/lib/<triple>` e
`lib/<triple>` do multiarch (Debian/Raspberry Pi OS), conta os `.pc` onde o
pkg-config procura (`usr/lib/pkgconfig`, `usr/lib64/pkgconfig`,
`usr/share/pkgconfig`, `<triple>/pkgconfig`), e identifica a libc — glibc pela
`__GLIBC__`/`__GLIBC_MINOR__` de `usr/include/features.h`, musl pela
`libc.so`. O veredito é uma frase: *utilizável* (com libc e `.pc`, ou o aviso
"sem .pc — o pkg-config não vai achar bibliotecas de terceiros"), *só
headers*, *só bibliotecas*, ou *vazia para o compilador* — que é o sysroot de
distro do Fedora medido em 2026-09-12 — com o remédio. Caminho relativo →
`INVALID_PARAMS`; pasta inexistente não é erro, é um relatório com
`exists: false`.

**`importKit` propõe, não grava.** Por evidência no caminho:
- **Yocto** — um `environment-setup-*` (o arquivo, ou a pasta do SDK que
  contém UM). O script é **carregado pelo `sh`** — é o que o manual do SDK
  manda fazer (`. environment-setup-…`, docs.yoctoproject.org sdk-manual,
  lido em 2026-09-13) — e as variáveis que ele exporta são lidas: `CC`/`CXX`/
  `GDB` (o `CC` do SDK já traz `--sysroot=$SDKTARGETSYSROOT`; o binário é
  resolvido por `command -v` no `PATH` que o script montou), `SDKTARGETSYSROOT`,
  `OECORE_NATIVE_SYSROOT` (o toolchain file do SDK mora em
  `usr/share/cmake/OEToolchainConfig.cmake`), `TARGET_PREFIX` (o triple). Um
  script que não exporta `CC`, ou que falha ao carregar, é recusa que diz isso;
  dois scripts na pasta = a IDE não escolhe (aponte o arquivo).
- **Buildroot** — `output/`, `output/host` ou a raiz da árvore: a marca é
  `host/share/buildroot/` (onde fica o `toolchainfile.cmake`), mais
  `host/bin/<triple>-gcc` e `host/<triple>/sysroot` (manual do Buildroot,
  "Using the generated toolchain outside Buildroot", 2026-09-13). Os tarballs
  da **Bootlin** entram por aqui — são SDKs do Buildroot, com o arquivo de
  CMake.
- **zephyr-sdk** (2026-09-17) — a raiz de um Zephyr SDK do sdk-ng: a prova
  é `sdk_version` **e** `cmake/Zephyr-sdkConfig.cmake` (o pacote CMake que o
  `setup.sh` registra em `~/.cmake/packages/Zephyr-sdk`). As toolchains GNU
  moram em `gnu/<toolchain>/bin/<toolchain>-gcc` (v1.x — o `setup.sh` extrai
  em `gnu/` e `sdk_gnu_toolchains` lista os nomes) ou na raiz
  (`<toolchain>/`, o layout 0.16/0.17 e os links de "bisectability" que o
  `setup.sh -c` cria); caminho canônico conta uma vez. Lido no
  `scripts/template_setup_posix` e no `cmake/Zephyr-sdkConfig.cmake` do
  sdk-ng em 2026-09-17. A proposta é **uma** toolchain: `arm-zephyr-eabi`
  quando há várias (o Cortex-M é o caso comum), **dita** na `evidence` com a
  lista das outras e a instrução "para outra, importe a pasta dela
  (`gnu/<toolchain>`)" — que cai no `toolchain-dir`. `targetTriple` é o nome
  da toolchain (`arm-zephyr-eabi`, `riscv64-zephyr-elf`,
  `xtensa-espressif_esp32_zephyr-elf`); `sysroot` é o que o gcc declara (a
  libc da toolchain — picolibc/newlib —, não um sistema de destino); `gdb` o
  da toolchain; sem `toolchainFile` (um projeto Zephyr compila pelo `west
  build -b <board>`, que acha o SDK por `ZEPHYR_SDK_INSTALL_DIR` ou pelo
  registro CMake — o `hint` diz isso; nenhum board é inventado). SDK sem
  toolchain nenhuma → proposta vazia com o passo (`./setup.sh -t
  arm-zephyr-eabi`). **Não medido contra um SDK real** (não há nesta máquina;
  a fixture é a forma do `setup.sh`).
- **toolchain-dir** — uma pasta com `bin/<triple>-gcc` (o tarball da Arm, o
  que a IDE instalou): o sysroot é o que o próprio gcc declara
  (`-print-sysroot`, se ele roda aqui e a pasta existe com conteúdo), senão as
  convenções `<triple>/libc` (Arm) e `<triple>/sysroot`; sem arquivo de CMake —
  o `CMAKE_SYSTEM_NAME` sai do triple, como sempre.

Caminho relativo → `INVALID_PARAMS`; nada reconhecido → `INVALID_REQUEST`
dizendo o que se procurou. **Não medido contra um SDK Yocto ou uma árvore
Buildroot reais** — não há nenhum nesta máquina (42 §8 item 5): o que está
provado é o contrato (um `environment-setup` que exporta as variáveis
documentadas; uma `output/host` com a forma documentada), e a exercitação do
gate lê a raiz do projeto como sysroot "vazia".

**O `toolchainFile` do kit** vira `-DCMAKE_TOOLCHAIN_FILE=<arquivo>` no
configure **só quando o preset não declara o seu** — o do `CMakePresets.json`
vence (dois arquivos brigariam), e a tela já dizia isso. Com o arquivo do
Buildroot/Yocto no kit, o configure é o do SDK: compiladores e sysroot vêm
dele. A UI (`EmbeddedKitImportView`): um campo de caminho, "Ler sysroot",
"Importar kit", a proposta em linhas e "Aplicar proposta ao kit" — um `setKit`
com sysroot, alvo e arquivo; o chip fica. O painel, ao aplicar chip/alvo/
sysroot, **preserva** o arquivo (um sinal não carrega `undefined`: o
controller substitui pelo atual).

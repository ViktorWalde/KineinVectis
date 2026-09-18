# 42 — A trilha PROFUNDA de embarcados: MCU bare metal e Linux embarcado, com C/C++, Rust e Python

> **Classe: PLANO**, com o ESTADO medido em **2026-09-12** na §1. Sucede o
> [`41`](41-ecossistema-embarcados-e-python.md) na parte de embarcados: o 41
> inventariou e ordenou em blocos; este documento **reorganiza por
> profundidade**, a pedido do autor em 2026-09-12 — *"quero profundidade e não
> um monte de corte vertical raso"* — e amplia o alvo para **Linux/software
> embarcado**. A fila continua sendo o [`40`](40-estado-e-continuidade.md) §4;
> este é o mapa que a ordena daqui em diante. **Em 2026-09-12 à tarde ganhou
> a §8** (o "efeito JetBrains": zero-config, indexação visível, intention
> actions, project model antes do LSP, toolchain manager com sysroot, remote
> deploy & debug, SVD com escrita, sondas visuais — o que já existe medido, o
> que falta e em que pilar) **e a §9** (a trilha completa em Python com
> C/C++/Rust: bare metal → edge → backend → banco), a pedido do autor.
>
> **O que "profundo" significa aqui, dito de forma verificável** — sem isto a
> palavra é sentimento:
>
> ```text
> 1  um PILAR so' fecha quando a capacidade funciona de ponta a ponta para TODAS
>    as familias do escopo (§0), nao para a primeira que apareceu
> 2  o que o pilar entrega e' MODELO + SERVICO + TELA, nos tres — nunca so' o fio
> 3  cada pilar continua com gate, mutacao e exercitacao real por familia; o que
>    muda e' o CRITERIO DE PRONTO, nao o metodo
> 4  fundacao antes de superficie: o pilar 0 (o MODELO do projeto) vem antes de
>    qualquer botao novo, porque hoje a IDE nao sabe o que e' um projeto
>    embarcado (§1.1) — e sem isso todo botao adivinha
> ```

> **Atualização 2026-09-16:** attach do debugpy por `connect {host, port}`
> entregue e validado no fluxo DAP existente (40 §7.38). Isso não fecha P6:
> contexto SSH, deploy, execução remota e mapeamento de caminhos continuam
> pendentes. A sequência vigente de implementação continua em 40 §4.1.

## 0. O escopo, em famílias — e as placas que definem "pronto"

```text
MCU bare metal   ESP32 classico (Xtensa, so' serial)           NA MESA
                 ESP32-C3/C6 (RISC-V, USB-JTAG embutido)        vai para a mesa (decisao 2026-09-11)
                 STM32 via ST-Link (Cortex-M)                   sonda: o autor tem regra udev para ST-Link
                 RP2040/RP2350 (Pico; UF2, Debug Probe)         idem, 2e8a na regra do autor
                 QEMU (Cortex-M) e Renode                       sem placa; o GATE
Linux embarcado  Raspberry Pi 4/5 (aarch64, Linux)              alvo do SSH remoto
                 imagem propria (Yocto/Buildroot)               reconhecer o projeto e o SDK
Linguagens       C/C++ (ESP-IDF, Zephyr, pico-sdk, CMSIS/Cube, CMake puro)
                 Rust (embassy/esp-hal/rp-hal; probe-rs; defmt)
                 Python (host: ferramentas e testes; alvo: MicroPython/CircuitPython no
                 MCU, CPython no Linux embarcado)
```

**Decidido pelo autor em 2026-09-12 (as três perguntas da §7):**

```text
a mesa           SO' o ESP32 classico, por ora. C3/C6, STM32 e Pico: pronto = gate
                 no QEMU/Renode + motores falsos, e a exercitacao real fica DITA
                 como pendente ate' a placa chegar — nunca marcada como provada
Linux embarcado  Raspberry Pi 4/5 com Raspberry Pi OS: Linux pronto do outro lado.
                 Yocto/Buildroot entram como RECONHECIMENTO de projeto (P0), sem
                 imagem propria agora
P0 primeiro      confirmado: a proxima fatia de codigo e' o MODELO do projeto
                 embarcado, sem botao novo ate' ele existir
```

## 1. O estado medido em 2026-09-12 — o que a IDE lê de um projeto hoje

Pergunta do autor: *"a IDE atualmente já lê todo o projeto, correto?"*
Resposta medida **na manhã de 2026-09-12** — **lia o que precisava, de forma
preguiçosa e delegada**. **A resposta do autor, no mesmo dia, virou
exigência:** *"a IDE deve ler o projeto inteiro que for aberto, ter integração
profunda de leitura do contexto do código/compilador, deve ser lido todas
funções/arquivos/pastas — tudo de Python/C/C++/Rust"*. A primeira forma dessa
exigência é o domínio `index` ([`40`](40-estado-e-continuidade.md) §7.17);
o que segue abaixo é o retrato de ANTES dele, mantido porque é a linha de
base que a exigência mede:

```text
deteccao do projeto   `workspace/detect.rs`: marcadores SO' NA RAIZ
                      (Cargo.toml, CMakeLists.txt, pyproject/setup.py/
                      requirements.txt, pom.xml, gradle). CMakeLists numa
                      subpasta = "Unknown". NENHUM marcador de embarcado:
                      ESP-IDF, Zephyr/west, pico-sdk, PlatformIO, Cube
arvore                sob demanda: raiz listada, pasta expandida e' listada
                      (fs.list por diretorio); nao ha' varredura inteira
watcher               `notify` NAO recursivo, por pasta observada (as abertas/
                      expandidas) — escolha de escala, registrada no ADR-0001
busca                 rg/fd na arvore inteira, quando pedida
semantica             DELEGADA: clangd com --background-index sobre o
                      compile_commands.json (que so' existe apos cmake
                      configure), rust-analyzer sobre o workspace cargo inteiro.
                      A IDE NAO tem indice proprio do projeto (o KSWE e' so'
                      especificacao); Tree-sitter so' nos buffers abertos
build                 targets do CMake pela file-api (codemodel-v2), presets,
                      kits com sysroot/triple/chip/remoteTarget/debugServer;
                      cargo por comando
artefatos             o ELF e' resolvido por convencao (dap::resolve_program);
                      build.size le o linker script `.ld`; nada sabe de .bin/
                      .hex/.uf2, map file, flasher_args.json, tabela de particoes
```

**Consequência:** para desktop isso bastava como *ponto de partida*; o autor
decidiu que não basta como *destino* — a IDE tem de ler o projeto inteiro, e
o LSP passa a ser a camada de semântica profunda **por cima** de um índice
próprio, não o único leitor. Para embarcado havia ainda a **lacuna do
modelo**: que framework é, que SDK precisa, que alvo gera, que artefatos saem,
como se grava e se depura. O pilar 0 responde às duas coisas.

**O que mudou depois desta medição (2026-09-12, `40` §7.16–§7.17):** o
domínio `project` (9 frameworks por evidência, SDKs, artefatos, alvo;
receita e partições do ESP-IDF lidas; `build.size` consumindo a partição
`app`) e o domínio `index` (todas as pastas, arquivos e declarações de
C/C++/Rust/Python, com as gramáticas do editor, em job, com incremento pelo
watcher e `#nome` sem LSP). Continuam DELEGADOS ao LSP: tipos, referências,
rename. O contexto de compilador por arquivo entrou à tarde (`40` §7.18:
`index.context`, a CDB envelhecida por subpasta detectada) e a gramática
Python também (`40` §7.19), o índice passou a seguir o disco inteiro (`40`
§7.20) e o modelo por alvo do CMake entrou pelo file-api (`40` §7.21).
Continua **por fazer** no P0: o modelo por preset, o map file (§3, P0) — e o que a §8 acrescenta (o
preset no configure automático, file-api `compileGroups`, Bear para
Makefile).

**O que os últimos dois dias entregaram e ENTRA nos pilares** (não se refaz):
`serial.list` (E1), `serial.monitor` (E3), `container.*` (Docker/Podman
nativo), o papel `serialMonitor`, `build.size`, `probe.list`, os três
adaptadores DAP, o QEMU no gate, o clangd com `--query-driver`.

## 2. A análise do que foi levantado — o que se repete, e o que isso ordena

Lendo o 28 (plataforma), o 35 §5 (frente F), o 36 e o 38 (levantamentos), o 40
§8 (varredura) e o 41 (inventário) juntos, **quatro coisas se repetem em todas
as áreas**, e são elas que viram fundação em vez de feature:

```text
A  IDENTIDADE E MODELO       quem e' o projeto, o alvo, a placa, o artefato.
                             Aparece como "deducao do alvo" (35 §5.7), "identidade
                             pelo canal" (38 §2), "flasher_args" (38 §3.3), "IDF_PATH/
                             ZEPHYR_BASE/PICO_SDK_PATH" (41 §3.7). E' UMA coisa.
B  PROCESSO ORQUESTRADO      toda ferramenta entra como processo com saida
                             estruturada e job cancelavel: esptool, probe-rs, openocd,
                             picotool, west, pio, mpremote, gdbserver, rsync, ssh.
                             A infraestrutura (jobs, eventos, aba de terminal com
                             titulo, open_command) ja' existe e foi provada esta semana.
C  CANAL ATE' O ALVO          serial (feito), USB/sonda (feito para ARM), SSH (decidido),
                             container (feito o local; falta o "dentro"). O 28 §4 ja'
                             dizia: container e SSH sao o MESMO contrato (RemoteContext).
D  PERMISSAO E AMBIENTE      udev/dialout/uaccess/ModemManager, grupo docker, chaves
                             SSH, SDKs e toolchains instalados por distro. Sempre
                             "diagnostica e imprime o passo oficial, nunca sudo".
```

**O que o 41 tinha de raso, dito com franqueza:** os blocos A–F eram cortes
por *ferramenta* (uma linha por CLI). Isso faz a IDE ganhar botões rápido e
deixa a inteligência na cabeça do usuário — que é exatamente o que "plug and
play" (35 §5.1) proíbe. A trilha abaixo inverte: primeiro o modelo, depois os
serviços sobre o modelo, depois as telas que só *mostram* o modelo.

## 3. Os pilares, e a ordem que os dependências impõem

```text
P0  MODELO DO PROJETO EMBARCADO        fundacao de tudo (A)
P1  AMBIENTE: toolchains, SDKs,        o que o modelo EXIGE, instalado e sao (D)
    interpretadores, permissoes
P2  CICLO MCU: build -> gravar ->      o loop diario, completo por familia (B, C)
    rodar -> monitorar
P3  DEPURACAO PROFUNDA                 o que Cortex-Debug/probe-rs mostram, inteiro
P4  PYTHON NO EMBARCADO                MicroPython/CircuitPython no MCU + CPython
                                       host; entra aqui porque precisa de P0-P2
P5  TESTE E QUALIDADE                  host, alvo, simulador; estatica; cobertura
P6  LINUX EMBARCADO                    SSH remoto como RemoteContext; deploy; gdbserver;
                                       Yocto/Buildroot; o container por dentro
P7  RUST EMBARCADO PROFUNDO            atravessa P0-P6; o que e' so' dele fica aqui
```

Cada pilar abaixo diz: **o que é**, **o que já existe**, **o que falta para
ser profundo**, **como se prova**, e **o que NÃO entra**.

### P0 — O modelo do projeto embarcado, e o projeto INTEIRO lido

**O que é.** Duas coisas, por decisão do autor em 2026-09-12: (a) o
**modelo** do projeto embarcado — o domínio `project`; (b) a **leitura do
projeto inteiro** — o domínio `index` (todas as pastas, arquivos, funções e
tipos, C/C++/Rust/Python) **mais o contexto de código/compilador por arquivo**.
A parte (b) é o "entender o projeto inteiro" da especificação do KSWE
([`KINEIN_VECTIS_DEEP_SEMANTIC_ENGINE…`](motor-semantico-profundo-cpp-rust.md)
§2), reaberta pelo autor **nesta forma** — índice estrutural próprio +
contexto de compilador, com o LSP por cima — sem adotar a especificação
inteira (scheduler, brokers, RAM budget) até que a dor a peça.

```text
(b) o projeto inteiro                     hoje (2026-09-12)
    pastas, arquivos, linguagem, linhas   FEITO — index/, job ao abrir, 4 MiB/arquivo
    funcoes e tipos (C/C++/Rust)          FEITO — Tree-sitter tags, dedup, container
    funcoes e tipos (Python)              FEITO (2026-09-12, 40 §7.19) — a gramatica
                                          oficial (tree-sitter-python 0.25.0, MIT)
                                          na mesma fundacao: indice, realce, outline
    busca por nome sem LSP (#nome)        FEITO — indice primeiro, LSP substitui
    incremento                            FEITO (2026-09-12 tarde, 40 §7.20) — o
                                          indice registra no watcher TODAS as
                                          pastas que caminhou (uma a uma, nao
                                          recursivo: ADR-0001 de pe'); pasta
                                          nova caminhada, pasta apagada limpa;
                                          148 watches neste repositorio
    contexto de compilador por arquivo    FEITO (2026-09-12 tarde, 40 §7.18) —
                                          index.context: unidade da CDB (nas
                                          duas formas; chave canonica; -I/-D/
                                          -std), crate/target/features do
                                          cargo metadata, interpretador Python
                                          com origem e aviso; a CDB envelhecida
                                          por CMakeLists.txt de SUBPASTA; recarga
                                          no configure e no Cargo.toml.
                                          O inverso (arquivo -> targets) e a
                                          unidade sem CDB vieram pelo file-api
                                          (40 §7.21, 0.96.0). FALTA: sys.path do
                                          Python; modelo por PRESET
    referencias/tipos/rename              DELEGADO ao clangd/rust-analyzer/
                                          basedpyright, por decisao — o indice
                                          nao os reimplementa
    o que o indice NAO le                 pastas de saida (a lista do watcher),
                                          links simbolicos, >4 MiB, nao-UTF-8
                                          — contados e DITOS em skipped
```

O domínio `project` responde, para o workspace aberto e para cada *alvo* dele:

```text
framework      cmake-puro | esp-idf | zephyr | pico-sdk | stm32cube | platformio |
               cargo-embedded | micropython | yocto | buildroot | desconhecido
               (detectado por marcadores EM QUALQUER NIVEL razoavel: project.cmake do
               IDF, west.yml/zephyr/, pico_sdk_import.cmake, platformio.ini,
               .ioc do Cube, Embed.toml/memory.x/.cargo/config com target
               thumbv*/riscv32*, conf/local.conf + bitbake, .config do buildroot)
alvos          por preset/kit: chip, familia, triple, sysroot, servidor de debug
sdk            que SDK/toolchain o framework exige e ONDE esta' (IDF_PATH,
               ZEPHYR_BASE + SDK, PICO_SDK_PATH, xtensa/riscv/arm gcc, espup)
artefatos      ELF por alvo (cmake file-api / cargo metadata), .bin/.hex/.uf2/
               .map gerados, flasher_args.json (IDF), tabela de particoes,
               memory map (linker script OU tabela de particoes OU SVD)
gravacao       motor por familia deduzido (esptool | probe-rs | picotool |
               dfu-util | openocd program | west flash | pio run -t upload) —
               SUGERIDO, confirmado pelo usuario (35 §5.7)
monitor/debug  porta serial provavel (serial.list + identidade), sonda (probe.list),
               adaptador DAP e servidor deduzidos
```

**O que já existe.** Detecção por marcador na raiz; kits; file-api do CMake;
`build.size` lendo `.ld`; `serial.list`; `probe.list`; catálogo de toolchain.
**E, desde 2026-09-12 (`40` §7.16), a primeira fatia deste pilar:** o domínio
`project` com os 9 frameworks reconhecidos por evidência, SDKs, artefatos e
alvo deduzido, o evento `event.project.changed` e a vista no painel.

**O que falta para ser profundo.** O modelo em si (tipos no protocolo, um
serviço que o computa e RE-computa quando o build muda), a detecção por
framework, a leitura dos artefatos (`flasher_args.json`, `.map`, tabela de
partições do IDF, `memory.x` do Rust), e o **evento** `event.project.changed`
para as telas seguirem o modelo em vez de perguntar.

**Como se prova.** Fixtures **reais e mínimas** de cada framework no
repositório (`scripts/fixtures/projetos/`: ESP-IDF, Zephyr, pico-sdk,
PlatformIO, STM32Cube, cargo embarcado, MicroPython, Yocto, Buildroot —
FEITAS) e testes que dizem framework/alvo/artefato de cada uma; mutação:
trocar o marcador e ver a detecção mentir. O índice se prova contra um
projeto de quatro linguagens com `build/` que não conta, e contra este
próprio repositório pelo core real (1.022 arquivos, 3.939 declarações em
2026-09-12 à tarde — o "4.658" da manhã foi medido antes de o dedup do `fn`
em `impl` entrar no mesmo commit; corrigido no `40` §7.17).
Exercitação: o ESP32 da mesa com um projeto ESP-IDF real (instalar o IDF é
passo do autor — P1 diz como).

**O que NÃO entra.** Semântica própria (tipos, referências, rename) — continua
delegada ao clangd/rust-analyzer/basedpyright. O que P0 acrescenta é o modelo
de *build e alvo* e o *mapa estrutural* do projeto com o *contexto de
compilador* por arquivo — o que o LSP não tem, e o que ele precisa para
funcionar bem (a CDB certa, o crate certo, o interpretador certo).

### P1 — Ambiente: toolchains, SDKs, interpretadores, permissões

**O que é.** Para cada exigência do modelo, três respostas: **está?**, **onde?**,
**como instalar/ativar nesta distro?** — e nunca rodar `sudo`.

```text
toolchains      arm-none-eabi (ja'), riscv32-esp-elf, xtensa-esp-elf (do IDF ou
                crosstool-NG), aarch64 (Linux embarcado), o SDK do Zephyr
SDKs            ESP-IDF (install.sh + export.sh -> o core aprende o ambiente do
                export SEM exigir shell interativo), Zephyr (west init/update +
                SDK), pico-sdk (git), PlatformIO (pipx), STM32Cube (arquivos do
                usuario)
Rust            rustup targets (thumbv7em-none-eabihf, riscv32imc/imac-unknown-
                none-elf), espup para Xtensa, cargo-embed/flash, flip-link,
                cargo-binutils
Python          interpretador por projeto (29 §4.1), uv, basedpyright, ruff,
                debugpy, pytest, mpremote, micropython-stubs por placa
permissoes      E2 do 38: por CANAL — dialout, uaccess/plugdev, ID_MM_DEVICE_IGNORE,
                grupo docker, chave SSH no alvo (P6)
setup           o dominio `setup` (hoje: grafana/postgres/timescale) ganha TODAS
                as entradas acima com o comando oficial por distro, e o estado
                "ja' ativo"
```

**Como se prova.** Detecção por `PATH` falso (o padrão de `tools.rs`); o
`export.sh` do IDF exercitado de verdade quando instalado; a permissão medida
com o ESP32 na mesa (já medida a mão em 2026-09-11 — vira código).

### P2 — O ciclo MCU: build → gravar → rodar → monitorar, completo por família

**O que é.** O loop diário de quem escreve firmware, sem editar arquivo à mão
(35 §5.1 item 3), para **cada** família do §0.

```text
build           o de hoje + o modelo: alvo e artefato por preset; diagnosticos
                do compilador cross no painel de problemas (ja' vem do build
                para o nativo — medir no cross); `idf.py`/`west`/`pio` como
                MOTORES de build reconhecidos (processo, saida estruturada)
gravar          configuracao de execucao "Gravar" (decisao 2026-09-11) com motor
                por familia; le flasher_args.json (IDF), UF2 (Pico: picotool ou
                BOOTSEL por copia), DFU (STM32), probe-rs download (ARM/RISC-V
                via sonda ou USB-JTAG). Reset/bootloader: DTR/RTS (esptool faz),
                `picotool reboot -f`, BOOT0. Progresso e erro NO PAINEL
rodar           `probe-rs run` (grava+reseta+RTT), `west flash && monitor`,
                `pio run -t upload -t monitor` — como configuracao de execucao
monitorar       o E3 de hoje + os filtros que o pio device monitor ensina:
                carimbo de tempo, log em arquivo, hexlify, decodificador de
                excecao (espflash --elf), reconectar quando a porta some e volta
                (a placa reseta!), baud escolhido, DTR/RTS explicitos
tamanho         build.size + tabela de particoes do IDF (a "flash" do ESP32 e' a
                particao, nao o .ld) + map file -> "o que cresceu" (bloaty)
```

**Como se prova.** Gate: QEMU (já) + fixture por família compilada no gate
(onde a toolchain existir) + motores FALSOS que gravam em arquivo; exercitação:
ESP32 clássico (esptool), C3/C6 (probe-rs e esptool), Pico (picotool/UF2),
STM32 (probe-rs), cada um com um firmware real mínimo. **Pronto = o loop
inteiro em cada placa da mesa.**

### P3 — Depuração profunda

**O que é.** Tudo que o Cortex-Debug e o probe-rs mostram (41 §3.5), no `dap/`
que já tem três adaptadores e o caminho `attach`:

```text
launch/attach com gravacao   probe-rs flashingEnabled; gdb `load` via OpenOCD/QEMU
RTT / defmt                  canais do probe-rs -> painel de console proprio (nao
                             so' a aba), com carimbo e filtro; defmt no canal 0
registradores de periferico  SVD: probe-rs svdFile; para o caminho gdb, cmsis-svd
                             (Apache-2.0) parseado pelo core -> arvore de
                             perifericos/registradores/campos com leitura ao vivo
memoria e disassembly        DAP readMemory/disassemble -> duas vistas; breakpoint
                             por instrucao
registradores do core        mostrar o escopo hoje escondido, sob pedido
RTOS                         threads via OpenOCD `rtos` (FreeRTOS, Zephyr) no
                             gdb -i dap; Zephyr thread awareness
multi-core                   ESP32 (2 cores), RP2040/2350 — o probe-rs ainda nao
                             (limite documentado), OpenOCD sim
semihosting, SWO/ITM         OpenOCD; orbuculum (BSD-3) para SWO quando houver placa
core dump / post-mortem      ESP-IDF espcoredump; gdb com core do Linux embarcado (P6)
Renode                       2o servidor sem placa, com placas reais que o QEMU nao
                             tem (nRF52840, STM32F4), como `debugServer`
```

**Como se prova.** QEMU no gate para tudo que é DAP genérico (memória,
disassembly, registradores); Renode no gate quando instalado; SVD com um
arquivo Apache-2.0 (CMSIS) na fixture; RTT/defmt exercitado no C3/C6 da mesa.

### P4 — Python no embarcado

**O que é.** Três Pythons, um domínio de cada vez, sem anunciar "Python" antes
da cadeia inteira (41 §1):

```text
host           basedpyright + ruff + debugpy + pytest + interpretador/venv/uv —
               e' o Python das FERRAMENTAS de embarcado (esptool, west, pio,
               idf.py sao Python) e dos scripts de teste
MicroPython    mpremote como cidadao: REPL na aba (porta do serial.list), painel
               "arquivos no dispositivo" (fs ls/cp/rm/mkdir/tree), "rodar este
               arquivo na placa", `mip install`, soft-reset; stubs por placa
               (micropython-<port>-stubs em typings/, typingsPath do basedpyright)
               = completar `machine`, `network`; firmware .bin/.uf2 pelo P2
CircuitPython  drive CIRCUITPY + circup; o mesmo painel de arquivos
Linux embarcado CPython no alvo via SSH (P6): interpretador remoto, debugpy
               remoto (attach por porta), pytest remoto
```

**Como se prova.** `mpremote` falso no gate (o padrão do `fd` falso — E a
exercitação real que a lição do `fd` exige); o ESP32 clássico com firmware
MicroPython oficial gravado pelo P2 (é o primeiro firmware REAL que a mesa
roda sem toolchain nenhum).

### P5 — Teste e qualidade

```text
host        Unity/Ceedling (C), GoogleTest/Catch2/doctest (C++) por descoberta,
            pytest, cargo test/nextest -> UM painel de testes com a saida que
            hoje se perde (event.test.output sem ouvinte, 40 §8)
alvo        ESP-IDF unit test app, Zephyr twister, probe-rs run com defmt-test,
            Renode + Robot Framework — como configuracao de teste
estatica    clang-tidy e cppcheck (C/C++; hoje so' clippy), ruff/mypy (Python),
            clippy (ja') -> painel de problemas
cobertura   gcov/lcov (cross tambem, com gcov no alvo via semihosting/QEMU),
            llvm-cov, cargo-llvm-cov -> gutters no editor
recursos    -fstack-usage + puncover (pilha por funcao), bloaty ("o que cresceu"),
            map file
sanitizers  ja' como acao de configuracao; entram no painel de problemas
```

### P6 — Linux embarcado: o RemoteContext

**O que é.** O 28 §4 já decidiu a forma: **SSH e container são o mesmo
contrato** — filesystem próprio, path mapping, ciclo de vida, execução do
outro lado. É o item "SSH remoto" do 40 §4, agora com o alvo nomeado: a
Raspberry Pi e a imagem própria.

```text
transporte     `ssh` do sistema como PROCESSO (ControlMaster para uma conexao
               so'), `rsync`/`scp` para deploy, `sftp` para a arvore — licenca:
               OpenSSH BSD, rsync GPL-3 (processo). Crate: NAO (decidir com
               fonte, como o 40 §4 pede)
workspace      abrir uma pasta REMOTA: arvore, editor, busca (rg do outro lado
               ou local com espelho), watcher (inotify remoto via `inotifywait`
               ou polling), LSP do outro lado (clangd/basedpyright/rust-analyzer
               rodando no alvo ou no host com sysroot + path mapping)
build/run      cross no host + deploy + run remoto; OU build no alvo. As duas
               formas como configuracao de execucao
debug          gdbserver no alvo + gdb multiarch `-i dap` no host (a ponte da
               fatia 2 ja' faz attach a servidor); debugpy attach; core dumps
sistema        journalctl/systemctl/dmesg do alvo num painel; serial console
               (o monitor do E3 na UART da Pi)
imagem propria Yocto (bitbake, `oe-init-build-env`, SDK `environment-setup-*`
               -> kit cross automatico) e Buildroot (`make menuconfig`, o
               `output/host` como sysroot) RECONHECIDOS pelo P0; o build como
               job longo com log
container      o "dentro" do dominio container: workspace dentro do container,
               devcontainer.json como formato de entrada (MODE-D), o mesmo
               RemoteContext
```

**Como se prova.** Um `sshd` local em porta alta no gate (OpenSSH está em toda
máquina de desenvolvimento) faz o papel do alvo — o mesmo truque do QEMU;
exercitação na Raspberry Pi do autor.

**Desenho da fatia 1 (2026-09-17, escrito antes do código — o que o 40 §4.1
pedia).** O que entra é o **alvo SSH como recurso do projeto**, no molde do
`datasource.*`: um perfil sem segredo em disco, um "testar" que MEDE o alvo,
e o ciclo deploy → rodar → depurar como CONFIGURAÇÃO DE EXECUÇÃO (a decisão
do E4). Nada de workspace remoto, LSP do outro lado ou mapeamento de caminhos
nesta fatia — é o que fica dito como não feito.

```text
remote.list    {}                          -> { targets: [RemoteTarget] }
remote.save    { target }                  -> { targets }        (cria/substitui pelo nome)
remote.remove  { name }                    -> { targets }
remote.probe   { name }                    -> { jobId }          (job; event.remote.probed)
remote.deploy  { name, source?, dest? }    -> { jobId }          (job; event.remote.deployed)
remote.command { name, kind: run|debugServer|shell, dest? } -> { command, remoteTarget? }

RemoteTarget   name, host, user?, port? (22), identityFile?, deployDir? (~/kinein/<projeto>)
               — SEM senha: SSH e' por chave (o `ssh` do sistema pergunta o que faltar
               no terminal da IDE; a IDE nunca guarda nem passa senha)
event.remote.probed   { jobId, name, success, arch?, kernel?, tools: [{ id, found, path? }],
                        error?, raw }        (uname -m; uname -sr; command -v gdbserver
                                              python3 rsync — o que o alvo TEM)
event.remote.deployed { jobId, name, success, source, dest, command, error? }
```

Transporte: o `ssh`/`rsync`/`scp` do sistema como PROCESSO (OpenSSH BSD,
rsync GPL-3 — nunca crate), `-o BatchMode=yes -o ConnectTimeout=5` no
probe (falha rápida e dita: "sem chave para <user@host>: `ssh-copy-id`").
Deploy: `rsync -az --delete <artefato> <alvo>:<deployDir>/` (ou `scp -r`
quando o `rsync` falta de um dos lados — o probe diz). Rodar:
`ssh <alvo> '<deployDir>/<binario>'` como configuração de execução ("Rodar
em <nome>"); depurar: o kit ganha `debugServer = ssh <alvo> gdbserver :2345
<binario>` e `remoteTarget = <host>:2345` (`remote.command { kind:
debugServer }` compõe, `toolchain.setKit` grava — o attach pelo `gdb -i
dap` existe desde 0.89.0); Python: `debugpy --listen 0.0.0.0:5678 --wait-
for-client` do outro lado e o attach TCP de 0.109.0. O binário vem do
modelo (`artifacts.elf` mais novo, ou o `program` do debug.start). Provas:
`ssh`/`rsync` falsos ecoando argv (linha certa, probe lido, recusas: alvo
sem nome, deploy sem artefato); exercitação real na Pi do autor — não há
Pi nesta máquina, e um `sshd` local fica para o gate quando houver chave.

**Fatia 1 FEITA (2026-09-17, noite, 0.120.0 — `40` §7.51).** O código seguiu
o desenho acima com três ajustes medidos no caminho: `remote.command` ganhou
o tipo `debugpy` (a linha `python3 -m debugpy --listen 0.0.0.0:5678
--wait-for-client <script>`, attach TCP em `host:5678`) e os campos
`program?`/`port?` no lugar de `dest?`; a resposta dele traz `name` (o nome
sugerido da configuração) e `source[]` (de onde veio cada pedaço); e a linha
de comando usa `ssh -tt` — o `debugServer` do kit roda por `sh -c` sem
terminal, e sem pty forçado matar o `ssh` local deixaria o `gdbserver` órfão
na placa. O `toolchain.setKit` já aceitava `remoteTarget`/`debugServer`, mas
a ponte C++ não os mandava — ganhou `toolchainSetKitRemote`. Painel **Alvo
remoto (SSH)** no menu Ambiente; `remote.list` na paleta.

**O que falta do P6 (a fatia 2 em diante):** o `RemoteContext` de verdade —
abrir a pasta REMOTA como workspace (árvore por `sftp`, editor, busca com
`rg` do outro lado, watcher), LSP do outro lado com mapeamento de caminhos,
`journalctl`/`dmesg`/`systemctl` num painel, Yocto/Buildroot reconhecidos
pelo P0, o `sshd` local no gate e a exercitação na Pi do autor.

**Desenho da fatia 2 (2026-09-18, escrito antes do código): o workspace
ESPELHADO.** A pergunta era como "abrir a pasta da Pi". O VS Code
(Remote-SSH, proprietário) responde subindo um servidor Node no alvo e
falando com ele — pesado numa Pi e inviável para nós sem reescrever
`fs.*`, o índice, o git e o LSP para um segundo filesystem, exatamente o que
o 28 §4 chama de duplicar o `RemoteContext`. A resposta desta fatia é a
outra escola (o "deployment" do PyCharm, o fluxo Zephyr/Yocto de todo
mundo): **a pasta remota vira um espelho local por `rsync`, e a IDE abre o
espelho como workspace comum**. Zero mudança em `fs.*`, índice, busca, git
e LSP — todos trabalham no espelho; o que muda é a SINCRONIA, que é o
`rsync` do sistema nos dois sentidos.

```text
remote.open   { name, path }               -> { jobId, mirror }   (job: rsync pull)
              -> event.remote.synced { jobId, name, direction: "pull", success,
                                       command, changed: [caminho], error?, mirror }
              a UI abre o espelho com o workspace.open de sempre
remote.sync   { direction: pull | push, paths? } -> { jobId, command }   (job)
              -> event.remote.synced (o mesmo); `paths` relativos ao espelho,
              ausente = a árvore inteira; nunca --delete (apagar e' gesto explicito,
              fica fora desta fatia)
remote.status {}                            -> { mirror?: RemoteMirror }
RemoteMirror  name · host · path (no alvo) · mirrorRoot (local)

espelho       ~/.cache/kinein-vectis/remote/<alvo>/<hash do path>/<basename>
marcador      <espelho>/.kinein/remote-mirror.json { schemaVersion, name, host, path }
              — `.kinein/` NUNCA sincroniza (e' estado da IDE, dos dois lados)
salvar        o fs.write num espelho EMPURRA o arquivo sozinho (job `Empurrar
              <arquivo>`), com ControlMaster do ssh para nao reabrir a conexao a
              cada gravacao (-o ControlMaster=auto -o ControlPath=<cache>/ssh-%C
              -o ControlPersist=60)
workspace.open do espelho responde `remote: RemoteMirror` — a UI sabe que e'
              espelho e o painel Remoto seleciona o alvo e mostra Puxar/Empurrar
deployDir     de um espelho e' o `path` remoto: "Rodar em pi" roda o que acabou de
              ser empurrado
```

**O que fica dito como não feito nesta fatia:** watcher do lado remoto (o
que muda na Pi só aparece ao Puxar); apagar/renomear não propaga (Empurrar
não usa `--delete`); LSP resolve contra ESTA máquina — para C/C++ cross o
sysroot do kit já cobre; Python/Rust do alvo não; conflito de edição dos
dois lados = o `rsync` mais recente vence (dito ao autor no painel).
**Prova:** `rsync`/`ssh` falsos ecoando argv (a linha do pull com os
excludes, o marcador escrito, o `workspace.open` do espelho devolvendo
`remote`, o push automático ao salvar, `remote.sync` com e sem `paths`,
`remote.status` fora de um espelho); exercitação real numa Pi quando houver.

**Fatia 2 FEITA (2026-09-18, 0.122.0 — `40` §7.53).** Como desenhado, com um
acréscimo que o teste pediu antes da Pi: o espelho aberto precisava do alvo
(usuário, porta, chave), que só existia no catálogo do workspace de origem —
o `remote.open` copia o alvo para o `.kinein/remotes.json` do espelho, e ele
fica autossuficiente. **Resta do P6:** watcher remoto, renomear/apagar
propagados, LSP/interpretador do alvo, journalctl/dmesg, Yocto/Buildroot,
`sshd` local no gate, exercitação numa Pi.

### P7 — Rust embarcado profundo (o que é só dele)

```text
alvos rustup por kit, espup para Xtensa, cargo-embed/flash como motores de
gravar/rodar, defmt no console RTT, `memory.x` no modelo de artefatos, svd2rust
para PACs no catalogo de templates (cortex-m-quickstart, embassy, esp-hal,
rp-hal — todos MIT/Apache, decisao 35 §5.7), flip-link, cargo-binutils
(size/objdump do alvo), probe-rs como depurador de primeira (ja').
```

## 4. A ordem, e por que ela

```text
1  P0  modelo do projeto        sem ele, P2/P3/P6 adivinham; e' a resposta a
   + projeto inteiro lido       "a IDE le o projeto?" — e, por decisao do
                                autor, a leitura de TUDO (indice + contexto
                                de compilador por arquivo) vem antes de
                                qualquer botao novo
2  P1  ambiente                 o que P0 descobre que falta, com o passo oficial;
                                inclui o E2 (permissao) que estava na fila
3  P2  ciclo MCU                o loop diario nas placas da mesa — inclui E4/E5
                                da fila (gravar, identidade Espressif)
4  P4  MicroPython              o primeiro firmware REAL da mesa e' o MicroPython;
   (parte MCU)                  e o Python host entra junto porque as ferramentas
                                de P1/P2 sao Python
5  P3  depuracao profunda       com C3/C6 e STM32 na mesa; QEMU/Renode no gate
6  P5  teste e qualidade        fecha o loop com a saida do teste na tela
7  P6  Linux embarcado          o RemoteContext — SSH e container por dentro
8  P7  Rust profundo            atravessa; o que sobrou fica por ultimo
```

**Por que P4 antes de P3:** com o ESP32 clássico (sem JTAG) e sem ESP-Prog, a
depuração do que está na mesa hoje é impossível; MicroPython na mesma placa é
possível amanhã e prova P0–P2 de ponta a ponta. Quando o C3/C6 chegar, P3 tem
onde ser exercitado.

## 5. O que muda no método — e o que não muda

**Muda o critério de pronto:** um pilar não fecha com a primeira família; fecha
quando *cada* família do §0 passa pela exercitação. Um item da fila do 40 §4
passa a ser um pilar ou uma parte nomeada de um pilar (`P2.gravar.esp32`), não
uma CLI.

**Não muda:** gate nascido de falha silenciosa, mutação que o compilador não
pega, exercitação contra a ferramenta real, corte por responsabilidade, limite
de arquivo, licença lida na fonte, processo para copyleft, nunca `sudo`,
documento no mesmo gesto. Profundidade não é licença para pular a prova; é a
obrigação de provar em mais lugares.

## 6. O que NÃO entra, para não reabrir

Tudo do 41 §4 (Pylance, Serial Monitor fechado, cpptools proprietário, nRF
Connect, Wokwi, Dependi, MPL como crate, symbolica), mais: **AVR/Arduino**
(decisão 35 §5.7 — reabrir é do autor), **Jetson/CUDA** (proprietário),
**STM32CubeIDE/MX/Programmer** (proprietários; o `.ioc` é lido como marcador,
nunca gerado), **J-Link Software** (a sonda funciona via probe-rs/OpenOCD),
**IA na IDE**.

## 7. As decisões que este documento pediu ao autor — respondidas em 2026-09-12

```text
1  a MESA: so' o ESP32 classico hoje (§0). As outras familias fecham no gate e
   ficam DITAS como nao exercitadas ate' a placa chegar
2  Linux embarcado: Raspberry Pi 4/5 com Raspberry Pi OS. Yocto/Buildroot so'
   como reconhecimento de projeto; sem imagem propria agora
3  P0 primeiro: confirmado
```

## 8. O "efeito JetBrains" — a experiência como critério de pronto (autor, 2026-09-12)

**O pedido do autor, na tarde de 2026-09-12**, depois da quinta fatia do P0:
o que faz alguém amar o CLion/PyCharm/RustRover é a sensação de *baterias
inclusas* — **zero-config ao abrir**, **indexação agressiva** visível,
**intention actions** (Alt+Enter) proativas, um **Project Model** construído
pelo backend **antes** de alimentar o LSP, um **Toolchain Manager visual**
que lê o sysroot do alvo, **Remote Deploy & Debug** de um clique (SSH +
gdbserver), **SVD ao vivo** com escrita de bits no breakpoint, e **sondas
visuais** (J-Link/ST-Link) com probe-rs/OpenOCD invisíveis por baixo.
*"Parte já deve ter sido desenvolvida; o resto dá um bom norte."*

Esta seção é o mapa **medido**: para cada item, o que a IDE **já faz** (com
onde), o que **falta**, em **que pilar** entra e **com que ferramenta aberta**
(licença lida na fonte, versão medida nesta máquina em 2026-09-12). A ordem
dos pilares (§4) **não muda** — o que muda é o critério de pronto de cada um.

```text
1  ZERO-CONFIG AO ABRIR
   ja' faz      workspace.open detecta os build systems; o kit e' automatico
                (o primeiro detectado por papel) e fica por projeto; o clangd
                ja' sobe com --query-driver do compilador do kit; project.model
                e o indice nascem no proprio open; setup.list diz o que falta
                COM o comando da distro
   CORRIGIDO    a primeira versao desta linha dizia "hoje o configure e'
   2026-09-12   explicito" — MEDIDO depois, e' FALSO: o ProjectHealthController
                ja' pede cmake.configure sozinho quando o cmake.status diz
                "nao configurado" (uma vez por workspace; falha vira o aviso
                "CMake sem configure — Configurar"), e salvar CMakeLists.txt/
                CMakePresets.json pela IDE reconfigura (WorkspaceEventRouter).
                Sempre em .kinein/build, sempre sem preset, sempre com a CDB
                exportada. Medido neste repositorio: o configure sem preset
                funciona (47 s, Qt). A regra zero valeu para quem escreveu
   falta        (a) o PRESET: cmake.presets.list e' roteado e nenhuma tela o
                consome; o configure automatico ignora os presets do projeto
                (com um so' nao-oculto, usa-lo nao e' adivinhar); e o core
                nao prova o automatico na exercitacao (e' a UI que pede)
                (b) FEITO 2026-09-12 (40 §7.24): pyproject/requirements sem
                ambiente -> "criar .venv" de UM clique pela faixa de saude
                (uv se houver, senao `python -m venv`), comando mostrado
                (c) FEITO: o painel de instalacao manda o comando oficial para
                o TERMINAL da IDE, visivel, e nada roda escondido (o
                SetupPanelHost.commandRequested -> submitShellInput)
   NAO entra    baixar toolchain CALADA. O JetBrains faz isso porque distribui
                os binarios; aqui a decisao registrada (40 §5, "comando de
                instalacao") e' fonte oficial citada, nunca sudo, nunca em
                silencio. A ausencia de JSON vem da DETECCAO + UM CLIQUE, nao
                do download escondido
   pilar        P0 (a), P1 (b, c)

2  INDEXACAO AGRESSIVA, VISIVEL
   ja' faz      "indice: 1.022 arquivos · 72.000 linhas · 3.939 simbolos" e
                "indexando… N arquivos" na barra (40 §7.17); o contexto de
                compilador do arquivo ativo ao lado (§7.18): "contexto: c++ ·
                gnu++23 · 12 -I · 9 -D"
   falta        um painel do indice ("o que li, o que pulei e por que"). A
                gramatica Python (40 §7.19) e o indice seguindo o disco inteiro
                (40 §7.20) entraram em 2026-09-12 a tarde
   pilar        P0

3  INTENTION ACTIONS (Alt+Enter)
   ja' faz      lsp.codeActions + lsp.applyCodeAction, atalho Alt+Return
                (shell/GlobalShortcuts.qml), no menu "Acoes de codigo" —
                SOB DEMANDA. As assists do rust-analyzer e os fix-its do
                clangd ja' chegam por esse caminho; "o borrow checker avisar
                antes de compilar" ja' e' o flycheck do rust-analyzer chegando
                como diagnostico. Ruff como companheiro do basedpyright
                VALIDADO em 2026-09-15 (40 §7.36): diagnosticos fundidos,
                acoes na mesma lista e preview com versao do servidor de origem
   falta        o PROATIVO: a lampada na margem quando a linha do cursor tem
                acao (codeAction com o diagnostico da linha); clang-tidy
                DENTRO do clangd (--clang-tidy com o .clang-tidy do projeto: e'
                dai que sai "este loop pode ser otimizado", como
                performance-* e modernize-*)
   pilar        P5 (qualidade) para as fontes; a lampada e' UMA fatia de
                editor — respeitando o EditorController congelado (40 §5):
                mora no EditorLanguageController, que e' quem ja' pede as acoes

4  PROJECT MODEL ANTES DO LSP
   ja' faz      a ORDEM ja' e' essa, medida em handlers/workspace.rs
                activate_workspace: project.model -> indice (job) -> args do
                clangd (query-driver do kit); o clangd so' sobe no primeiro
                .c/.cpp aberto, ja' com tudo isso
   FEITO        a "CDB em memoria" na forma que o CMake oferece de verdade
   2026-09-12   (40 §7.21): o codemodel-v2 lido por target (fontes, grupos
                de compilacao com includes/defines/fragments/sysroot,
                artefatos, dependencias) + a toolchains-v1 (CMake >= 3.20)
                com o compilador por linguagem; cmake.targets.list carrega o
                modelo; index.context da' os targets do arquivo e, sem CDB, a
                unidade do file-api. Escrever a compile_commands.json a
                partir dele para gerador que nao exporta nao foi preciso no
                Linux (a IDE ja' configura com CMAKE_EXPORT_COMPILE_COMMANDS)
   falta        Makefile puro: Bear como PROCESSO (`bear -- make`;
                GPL-3.0, COPYING; 3.1.6 instalado aqui) ou compiledb
                (GPL-3.0). Cargo: o rust-analyzer monta o proprio modelo — o
                que falta e' a IDE passar o alvo do kit
                (rust-analyzer.cargo.target). ESP-IDF, Zephyr, pico-sdk sao
                CMake por baixo (a CDB esta' no build/ deles); PlatformIO:
                `pio run -t compiledb`
   pilar        P0

5  TOOLCHAIN MANAGER VISUAL (sysroot do alvo)
   ja' faz      kit por projeto com sysroot, triple, chip, remoteTarget e
                debugServer (toolchain.setKit; EmbeddedKitField no painel de
                embarcados); o clangd aprende o compilador cross. E desde a
                noite de 2026-09-12 (integracoes/39, 40 §7.23): o catalogo
                reconhece os triples da industria (riscv-none-elf, xtensa-
                esp-elf, riscv32-esp-elf, aarch64-linux-gnu, arm-linux-
                gnueabihf, riscv64-linux-gnu, e os <triple>-gdb) na grafia de
                cada distribuidor; procura ALEM do PATH (~/.local/xPacks,
                ~/.espressif/tools, a pasta da IDE, /opt/*/bin); o
                toolchain.get diz os alvos Rust INSTALADOS e denuncia o
                compilador cross de distro SEM sysroot (medido no Fedora).
                E desde 2026-09-13 (40 §7.29 e §7.30): (0) o PROVEDOR DE
                INSTALACAO existe — nove releases pinados (Arm GNU, xPack,
                ATfE, Bootlin) com SHA-256 lido na fonte, download em job,
                checksum ANTES do tar, pasta da IDE lida pelo detector, botao
                no painel de embarcados; (b) `toolchain.inspectSysroot` LE a
                pasta (usr/include, usr/lib, lib, usr/lib/<triple>, os .pc, a
                glibc/musl) e da' um VEREDITO; (c) o kit injeta
                -DCMAKE_SYSROOT e -DCMAKE_TOOLCHAIN_FILE (quando o preset nao
                declara um) e o --sysroot no clangd; (d) `toolchain.importKit`
                de Yocto (environment-setup-* por `sh -c`), Buildroot
                (share/buildroot/) e pasta de toolchain (-print-sysroot)
   falta        (a) seletor de PASTA nativo no lugar do campo de texto; o SDK
                do Zephyr (setup.sh + ZEPHYR_SDK_INSTALL_DIR) no importKit;
                medir o importKit contra um SDK Yocto e uma arvore Buildroot
                REAIS (nao ha' nenhum nesta maquina — hoje e' fixture minima
                do padrao dos dois); o toolchain file GERADO pela IDE em
                .kinein/ (hoje o kit usa o do SDK ou o que o autor apontar)
   pilar        P1 (o manager), P6 (Yocto/Buildroot)

6  REMOTE DEPLOY & DEBUG (SSH + gdbserver) DE UM CLIQUE
   ja' faz      remoteTarget no kit vira `attach` com `target remote` (dap/
                adapter.rs, 40 §7.9); o servidor de debug (OpenOCD/QEMU) e'
                subido pela IDE; configuracoes de execucao existem
   falta        o transporte SSH (ssh/scp/rsync como PROCESSO: OpenSSH BSD,
                rsync GPL-3 — decisao do P6) e UMA configuracao "Remoto (SSH)":
                build cross com o kit -> rsync do binario -> `ssh alvo
                gdbserver :porta ./bin` -> attach com `gdb -i dap` (17.2 aqui;
                gdbserver 17.2 e OpenOCD instalados). debugpy attach para o
                Python do alvo pelo mesmo caminho
   prova        sshd local em porta alta + gdbserver local no gate (o truque
                do QEMU); exercitacao na Raspberry Pi do autor
   pilar        P6

7  SVD AO VIVO (ler E ESCREVER bits no breakpoint)
   ja' faz      nada de SVD. O DAP ja' e' o caminho de tudo
   como         dois caminhos, o mesmo painel:
                - probe-rs: `svdFile` por core na configuracao e os perifericos
                  aparecem como escopo de Variables (probe.rs/docs/tools/
                  debugger, lido em 2026-09-12; leitura confirmada na doc,
                  escrita A CONFIRMAR no adaptador)
                - gdb -i dap (OpenOCD, QEMU, gdbserver): a IDE LE o SVD com o
                  crate svd-parser (0.14.10, MIT OR Apache-2.0, rust-embedded/
                  svd, atualizado 2026-08-11) e usa DAP readMemory/
                  writeMemory. MEDIDO: o gdb 17.2 desta maquina implementa os
                  dois (/usr/share/gdb/python/gdb/dap/memory.py:
                  supportsReadMemoryRequest e supportsWriteMemoryRequest)
                painel Qt proprio: periferico -> registrador -> campo, valor
                lido no breakpoint, campo EDITAVEL (o "ligar o clock do GPIO
                pela UI" do autor) com o write confirmado por releitura; os
                perifericos escolhidos ficam no projeto
                de onde vem o SVD: o fabricante (CMSIS-Pack .pdsc -> .svd;
                espressif/svd e' Apache-2.0, LICENSE lido em 2026-09-12; o
                cmsis-svd-data tem licenca POR FABRICANTE — ler o arquivo, nao
                o repositorio); o P0 ja' preve o SVD no modelo de artefatos
                referencia de comportamento (nao de codigo — sao extensoes
                JS): CLion Peripheral view (Embedded GDB Server e OpenOCD
                Download & Run), mcu-debug/peripheral-viewer e
                eclipse-cdt-cloud/vscode-peripheral-inspector (ambos MIT)
   pilar        P3

8  SONDAS VISUAIS (J-Link, ST-Link, CMSIS-DAP)
   ja' faz      probe.list (probe-rs list) e a sonda no painel de embarcados;
                `probe-rs info` SUGERE o chip e o usuario confirma (decisao
                2026-09-11); debugServer no kit
   falta        escolher a sonda na LISTA (VID:PID:serial) e grava-la no kit;
                a config do OpenOCD deduzida do VID:PID (interface/stlink.cfg
                0483:3748/374b/374e/3752; interface/jlink.cfg 1366:*;
                interface/cmsis-dap.cfg — o Debug Probe da Pico 2e8a:000c);
                J-Link pelo probe-rs direto por USB (o software da SEGGER
                continua fora, §6)
   pilar        P2 (gravar) e P3 (depurar)
```

**O que isto muda no §4:** nada na ordem; muda o que cada pilar precisa
entregar para fechar. E deixa **duas coisas ditas**: (1) o zero-config aqui é
*detectar e propor com um clique*, nunca *baixar calado* — é a mesma
experiência de "não editei JSON nenhum", sem quebrar a regra de instalação;
(2) o SVD com escrita é a peça de maior valor que **não existe em nenhuma
forma** hoje, e o caminho pelo `gdb -i dap` já está medido do nosso lado.

## 9. A trilha completa em Python (com C/C++ e Rust): bare metal → edge → backend → banco (autor, 2026-09-12)

**O pedido do autor:** *"esquece a parte de simulação física/matemática;
vamos refinar ao máximo para sistemas embarcados e também para
desenvolvimento de software — no Python dá para fazer, com C/C++ ou Rust, um
projeto desde bare metal → edge computing → backend e banco de dados de
forma completa."* A decisão sobre a simulação está registrada no
[`40`](40-estado-e-continuidade.md) §5. Esta seção é a trilha.

**Sobre a versão do Python, com fonte** (o autor citou o 3.16): nesta máquina
o Python é **3.14.7** (medido em 2026-09-12). O **3.15.0 final sai em
2026-10-01** (PEP 790; o rc2 já está publicado). O **3.16** segue o ciclo
anual — outubro de **2027** — e em 2026-09-12 não tem calendário publicado.
A IDE não depende de versão: o interpretador é o do projeto (`.venv`, uv,
poetry, sistema — `roadmaps/29` §4.1, hoje lido pelo `index.context`).

```text
estagio        ferramenta aberta (licenca lida na fonte, 2026-09-12)      dominio da IDE
bare metal     MicroPython (MIT) no ESP32; mpremote 1.29.0 (instalado     P2 gravar (esptool
               aqui via pipx; vive no repositorio do MicroPython, MIT);   5.3.1), P4 REPL/
               umqtt para publicar; ou C/C++ (ESP-IDF) e Rust (esp-hal)   arquivos, serial.*
               quando o Python nao cabe — o mesmo projeto, dois alvos
transporte     MQTT: Mosquitto como CONTAINER no Podman (EPL-2.0 OR       container.* (ja':
               EDL-1.0, LICENSE.txt); alternativa Zenoh (Apache-2.0 OR    listar, logs, shell,
               EPL-2.0). Como processo/imagem, nunca como crate            compose)
edge           Raspberry Pi OS: CPython no alvo, servico Python           P6 (SSH: deploy,
               (systemd), Podman no Pi, SQLite local (dominio publico);   run, debugpy attach,
               modulo nativo quando precisar de velocidade: pybind11 e     journalctl)
               nanobind (BSD-3), PyO3 + maturin (MIT OR Apache-2.0) — o
               build do modulo e' CMake ou cargo: a MESMA CDB e o MESMO
               cargo metadata que o index.context ja' le
backend        FastAPI (MIT) + uvicorn; uv (MIT OR Apache-2.0) para o      41 bloco A (ruff,
               ambiente; ruff, basedpyright, pytest, debugpy               basedpyright, pytest,
                                                                          debugpy) + run/test
banco          PostgreSQL e TimescaleDB (series temporais), SQLite,        datasource.* (ja':
               MongoDB — conectar, introspecionar, ler; executar e         postgres, timescaledb,
               escrever ainda sao fatia propria (40 §4 item 27)            sqlite, mongo)
observar       Grafana pela HTTP API (embutir e' PROIBIDO, AGPL)           grafana.*
```

**O que a IDE precisa fazer de NOVO para essa trilha existir de ponta a
ponta — e onde já está planejado:**

```text
1  o ESP32 da mesa com MicroPython oficial gravado e o REPL na aba          P2, P4
   (2026-09-13: o REPL ja' e' o serial.monitor de um projeto MicroPython e o
   .py roda na placa pelo Executar — 40 §7.28; falta GRAVAR o firmware)
2  "criar .venv com uv" de um clique + ruff/basedpyright/pytest ligados      P1, 41 A
   (FEITO 2026-09-12/13: a cadeia Python inteira, 40 §7.24-§7.28 — ambiente,
   basedpyright com o interpretador, ruff, run, pytest, debugpy)
3  a Pi como alvo: SSH, deploy, rodar, debugpy attach, journalctl            P6
4  o Mosquitto de um clique: uma RECEITA de container (imagem oficial,       container
   porta, volume) no painel de containers — a primeira receita do dominio
5  o modulo nativo: o projeto Python com CMake/cargo dentro reconhecido        P0 (project)
   pelo project.model (scikit-build-core, maturin) e a CDB/cargo do modulo
   no index.context (FEITO em 2026-09-13 no python.status.nativeModule —
   40 §7.28; o project.model ficou para o embarcado; a CDB/cargo do modulo
   o index.context ja' lia)
6  o banco: executar consulta e escrever (a fatia propria do item 27)         datasource
```

Nada aqui é ferramenta nova para a IDE **adotar como dependência**: são
ferramentas que o **projeto do usuário** usa e que a IDE **reconhece, sobe,
observa e depura**. A regra de licença vale para o que entra no binário
(`deny.toml`) e para o que a IDE executa (processo com licença lida); as
bibliotecas do projeto são escolha do autor do projeto.

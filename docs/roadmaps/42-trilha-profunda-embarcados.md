# 42 — A trilha PROFUNDA de embarcados: MCU bare metal e Linux embarcado, com C/C++, Rust e Python

> **Classe: PLANO**, com o ESTADO medido em **2026-09-12** na §1. Sucede o
> [`41`](41-ecossistema-embarcados-e-python.md) na parte de embarcados: o 41
> inventariou e ordenou em blocos; este documento **reorganiza por
> profundidade**, a pedido do autor em 2026-09-12 — *"quero profundidade e não
> um monte de corte vertical raso"* — e amplia o alvo para **Linux/software
> embarcado**. A fila continua sendo o [`40`](40-estado-e-continuidade.md) §4;
> este é o mapa que a ordena daqui em diante.
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
Resposta medida — **lê o que precisa, de forma preguiçosa e delegada**:

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

**Consequência:** para desktop isso basta e é a arquitetura certa (delegar ao
LSP). Para embarcado é a **primeira lacuna profunda**: a IDE não tem um
**modelo do projeto embarcado** — que framework é, que SDK precisa, que alvo
gera, que artefatos saem, como se grava e se depura. Cada botão que se
acrescentasse agora teria de adivinhar isso de novo. É por isso que o pilar 0
vem primeiro.

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

### P0 — O modelo do projeto embarcado

**O que é.** Um domínio `project` (ou a evolução do `workspace`) que responde,
para o workspace aberto e para cada *alvo* dele:

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

**O que falta para ser profundo.** O modelo em si (tipos no protocolo, um
serviço que o computa e RE-computa quando o build muda), a detecção por
framework, a leitura dos artefatos (`flasher_args.json`, `.map`, tabela de
partições do IDF, `memory.x` do Rust), e o **evento** `event.project.changed`
para as telas seguirem o modelo em vez de perguntar.

**Como se prova.** Fixtures **reais e mínimas** de cada framework no
repositório (o `hello_world` do ESP-IDF sem o SDK, um `west` workspace
esqueleto, o `blink` do pico-sdk, um `platformio.ini`, um `Embed.toml`) e
testes que dizem framework/alvo/artefato de cada uma; mutação: trocar o
marcador e ver a detecção mentir. Exercitação: o ESP32 da mesa com um
projeto ESP-IDF real (instalar o IDF é passo do autor — P1 diz como).

**O que NÃO entra.** Índice semântico próprio (KSWE) — continua delegado ao
clangd/rust-analyzer; o que P0 acrescenta é o modelo de *build e alvo*, que o
LSP não tem.

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
                                "a IDE le o projeto?" para embarcado
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

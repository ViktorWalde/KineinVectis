# 39 — Toolchains por alvo: o catálogo credível, e como a IDE as encontra

> **Classe: INTEGRAÇÃO, com o ESTADO medido em 2026-09-12.** Nasce de um
> levantamento que o autor recebeu de outra IA e pediu para conferir:
> *"veja se algo dessas informações pode ser usado e qual a credibilidade real
> delas; pesquise e, se tiver credibilidade real, implemente; procure o que
> falta."* Cada linha abaixo tem **fonte lida no dia** (release, README,
> LICENSE do repositório, `rpm -q` desta máquina) — nunca a afirmação de
> partida. Onde a afirmação de partida estava errada, está dito.
>
> O que este documento decide entra no código no mesmo gesto: o catálogo de
> candidatos por papel (`toolchain/catalog.rs`), a tabela de ferramentas
> (`tools/known.rs`), os diretórios onde a IDE procura além do `PATH`
> (`tools/search_dirs.rs`), os alvos Rust instalados e a dica de sysroot no
> `toolchain.get` (protocolo 0.97.0). O **provedor de download** (§5) é a
> fatia seguinte.

## 1. O veredito sobre o levantamento recebido

```text
afirmacao                                        credibilidade   o que a fonte diz
Arm GNU Toolchain e' o padrao Cortex-M/R,         CONFIRMADA      15.2.rel1 em developer.arm.com, tarball
mantida pela Arm, substituiu gcc-arm-embedded                    arm-gnu-toolchain-15.2.rel1-x86_64-arm-none-eabi.tar.xz
                                                                 com .sha256asc ao lado (baixado e lido em
                                                                 2026-09-12); tambem aarch64-none-linux-gnu.
                                                                 O `arm-none-eabi-gdb` do tarball exige
                                                                 libpython no host (README do release) —
                                                                 o gdb do sistema com `-i dap` cobre ARM
xPack empacota binarios portateis para IDEs       CONFIRMADA      xpack-dev-tools/arm-none-eabi-gcc-xpack
                                                                 v15.2.1-1.1 (2026-03-03), riscv-none-elf-gcc
                                                                 v15.2.0-1 (2025-10-23), openocd v0.12.0-7;
                                                                 assets *-linux-x64.tar.gz + .sha; o xpack
                                                                 (scripts) e' MIT, os binarios seguem a
                                                                 licenca de cada programa (GPL). O store do
                                                                 xpm e' ~/.local/xPacks (docs do xpm;
                                                                 XPACKS_STORE_FOLDER sobrescreve) — a
                                                                 afirmacao dizia ~/.local/share/xPacks: ERRADA
riscv-none-elf-gcc para MCUs RISC-V                CONFIRMADA      o xPack acima; upstream riscv-collab/
                                                                 riscv-gnu-toolchain nao publica binario
                                                                 estavel — o xPack e' a fonte pratica
Espressif: xtensa-esp32-elf-gcc e                  DESATUALIZADA   desde o ESP-IDF 5.x a toolchain e' UNIFICADA:
xtensa-esp32s3-elf-gcc por chip                                  `xtensa-esp-elf-gcc` (ESP32/S2/S3, -mcpu) e
                                                                 `riscv32-esp-elf-gcc` (C2/C3/C5/C6/H2/P4).
                                                                 espressif/crosstool-NG esp-16.1.0_20260609
                                                                 (2026-06-10): xtensa-esp-elf-16.1.0_20260609
                                                                 -x86_64-linux-gnu.tar.xz e riscv32-esp-elf-...
                                                                 O nome antigo fica como binario alternativo
LLVM Embedded Toolchain for Arm (clang)           SUPERADA        o ultimo release foi 19.1.5 (2024-12-16). O
                                                                 sucessor e' o Arm Toolchain for Embedded
                                                                 (ATfE), no repositorio arm/arm-toolchain:
                                                                 release-23.1.0-ATfE em 2026-09-10, asset
                                                                 ATfE-23.1.0-Linux-x86_64.tar.xz + .sha256,
                                                                 Apache-2.0 WITH LLVM-exception; linka picolibc
arm-linux-gnueabihf / aarch64-linux-gnu /          CONFIRMADA      triples corretos; nesta maquina (Fedora 44)
riscv64-linux-gnu para Linux embarcado                           `gcc-aarch64-linux-gnu` 16.1.1 e `gcc-arm-
                                                                 linux-gnu` 16.1.1 (grafia da Fedora:
                                                                 arm-linux-gnu-gcc) INSTALADOS, gcc-riscv64-
                                                                 linux-gnu disponivel. Falta o que a afirmacao
                                                                 nao disse: o pacote da distro NAO TRAZ o
                                                                 sistema alvo (medido: `-print-sysroot` ->
                                                                 /usr/aarch64-linux-gnu/sys-root, sem
                                                                 usr/include). Sem sysroot nao se compila
                                                                 programa de usuario nenhum — ver §3
Bootlin: referencia de toolchains prontas          CONFIRMADA      toolchains.bootlin.com, tarballs
para Linux embarcado (glibc/musl/uclibc)                         <arch>--<libc>--<stable|bleeding-edge>-
                                                                 2026.08-1.tar.xz, com <nome>.sha256 ao lado
                                                                 (listagem lida em 2026-09-12); construidas
                                                                 com Buildroot
Yocto/Buildroot: apontar para o                    CONFIRMADA      o script environment-setup-<arch> exporta CC
environment-setup do SDK                                         ja' com --sysroot=$SDKTARGETSYSROOT
                                                                 (docs.yoctoproject.org, sdk-manual); e' a
                                                                 importacao de kit do 42 §8 item 5
Rust: rustc ja' e' cross; basta                    CONFIRMADA      platform-support do rustc: thumbv6m/7m/7em
`rustup target add <triple>`                                     (hf)/8m.main-none-eabi(hf), riscv32imc/imac-
                                                                 unknown-none-elf, aarch64-unknown-linux-gnu/
                                                                 musl, armv7-unknown-linux-gnueabihf,
                                                                 riscv64gc-unknown-linux-gnu. Xtensa (ESP32
                                                                 classico) e' tier 3 e vem pelo espup
                                                                 (esp-rs/espup v0.17.1, MIT OR Apache-2.0).
                                                                 Nesta maquina: rustup 1.29.0, rustc 1.96.1,
                                                                 so' x86_64-unknown-linux-gnu instalado
Autodeteccao em /usr/bin, /opt, xPacks,            CONFIRMADA,     a IDE ja' lia o PATH; passa a ler tambem os
STM32CubeIDE/plugins                               COM CORTE      diretorios dos distribuidores (§4). O
                                                                 CubeIDE fica de fora: produto proprietario
                                                                 (decisao do 42 §6) — quem tem o GCC dele
                                                                 aponta a pasta como qualquer outra
"download silencioso" da toolchain                 REFINADA        a decisao registrada (40 §5, "comando de
                                                                 instalacao") proibe instalar no sistema e
                                                                 baixar calado. O que o autor pede agora e'
                                                                 outra coisa: baixar, DEPOIS DE UM CLIQUE,
                                                                 para a pasta da IDE, sem tocar no sistema.
                                                                 Entra assim (§5): URL e sha256 mostrados
                                                                 antes, verificados depois, nunca sem clique
```

**O que faltava no levantamento, e entra:**

```text
Zephyr SDK              zephyrproject-rtos/sdk-ng v1.0.1 (2026-03-25), Apache-2.0:
                        UM bundle com todas as toolchains cross + host tools;
                        zephyr-sdk-1.0.1_linux-x86_64_minimal.tar.xz. E' a forma
                        oficial do Zephyr, nao a colecao de tarballs avulsos
servidores de debug     OpenOCD (xPack v0.12.0-7; Fedora 0.12+dev instalado),
                        probe-rs 0.32.0 (instalado), pyOCD — o levantamento fala
                        de compilar e nao de depurar; a IDE ja' depura por gdb -i dap
gdb                     Fedora nao tem gdb-multiarch: o `gdb` 17.2 do sistema ja' e'
                        multi-arquitetura (medido: falta so' xtensa); Debian/Ubuntu
                        chamam gdb-multiarch. Os tarballs da Arm e da Espressif
                        trazem o proprio <triple>-gdb
o sysroot               o problema REAL do Linux embarcado com compilador de distro
                        (§3): a toolchain compila, o programa nao linka. Nenhuma
                        lista de tarballs resolve isso; o gerenciador precisa dizer
picolibc / newlib       a libc do bare metal vem DENTRO do tarball (newlib e
                        newlib-nano na Arm; picolibc no ATfE) — nao e' pacote a parte
mpy-cross / mpremote    o Python no MCU (P4): mpremote 1.29.0 esta' instalado aqui
```

## 2. O catálogo por universo (fonte e forma, 2026-09-12)

```text
universo          triple / binario             fonte credivel (versao lida)         forma na IDE
BARE METAL
Cortex-M/R        arm-none-eabi-gcc            Arm GNU Toolchain 15.2.rel1;         processo; candidato
                                               xPack 15.2.1-1.1; Fedora            de compilador (ja')
                                               arm-none-eabi-gcc-cs 15.2.0
                  clang (ATfE)                 arm/arm-toolchain 23.1.0-ATfE        processo; `clang`
                                               (Apache-2.0 w/ LLVM exc.)           com --target
RISC-V MCU        riscv-none-elf-gcc           xPack 15.2.0-1                       candidato (novo)
Espressif         xtensa-esp-elf-gcc,          crosstool-NG esp-16.1.0_20260609     candidatos (novos);
                  riscv32-esp-elf-gcc          (o idf_tools.py os instala em        ~/.espressif/tools
                                               ~/.espressif/tools)                  lido (§4)
Zephyr            zephyr-sdk (todas)           sdk-ng v1.0.1 (Apache-2.0)           importKit FEITO 2026-09-17 (40 §7.45)
LINUX EMBARCADO
AArch64           aarch64-linux-gnu-gcc        distro (Fedora 16.1.1, SEM sysroot); candidato (novo) +
                  aarch64-none-linux-gnu-gcc   Arm 15.2.rel1; Bootlin 2026.08-1     dica de sysroot
ARM 32 hard-float arm-linux-gnueabihf-gcc      distro (Fedora: arm-linux-gnu-gcc);  candidato (novo)
                                               Arm arm-none-linux-gnueabihf;
                                               Bootlin armv7-eabihf
RISC-V 64         riscv64-linux-gnu-gcc        distro (gcc-riscv64-linux-gnu        candidato (novo)
                                               16.1.1 disponivel); Bootlin
imagem propria    SDK Yocto / Buildroot        environment-setup-*; output/host     P1/P6: importar kit
RUST
todos os acima    rustup target add <triple>   platform-support (tier 2 sem std;   toolchain.get diz os
                                               tier 3 xtensa via espup 0.17.1)      instalados (novo)
DEPURACAO
                  gdb / gdb-multiarch /        sistema; tarballs                    candidatos do
                  <triple>-gdb                                                      debugAdapter (novos)
                  openocd, probe-rs, pyOCD     xPack / cargo / pip                  servidor no kit (ja')
```

## 3. O sysroot: a peça que nenhum tarball resolve sozinho

Compilador cross de Linux é dois problemas: o **compilador** (qualquer linha
acima) e o **sistema alvo** — headers e bibliotecas da placa (`/usr/include`,
`/usr/lib`, a glibc ou musl certa). O tarball da Arm e o da Bootlin trazem um
sysroot **genérico** (funciona para `hello`, não para linkar com a `libgpiod`
que está na placa). O pacote da distro **não traz nenhum** — medido no Fedora
44 em 2026-09-12: `aarch64-linux-gnu-gcc -print-sysroot` responde
`/usr/aarch64-linux-gnu/sys-root` e ele está **vazio**.

A IDE passou a **dizer** isso (`toolchain.get.sysrootHint`, 0.97.0): quando
o compilador C efetivo é `*-linux-gnu*`, o kit não tem `sysroot`, e o sysroot
que o próprio compilador declara não tem `usr/include`. A dica nomeia as três
saídas honestas, em ordem de fidelidade:

```text
1. a PLACA          rsync -a pi:/usr/include pi:/usr/lib pi:/lib  ~/sysroots/pi/
                    (o sistema exato, com as bibliotecas que o programa vai achar)
2. Bootlin          <arch>--glibc--stable-2026.08-1.tar.xz: compilador + sysroot
                    genericos, checksum ao lado
3. SDK Yocto/       o sysroot EXATO da imagem que roda na placa; a IDE importa o
   Buildroot        kit do environment-setup (42 §8 item 5)
```

> **Estado (2026-09-13, 0.104.0):** a IDE **lê** a pasta de sysroot e diz o
> que há (`toolchain.inspectSysroot`: headers, bibliotecas, multiarch, `.pc`,
> libc, veredito) e **importa** o kit de um SDK Yocto, de uma árvore Buildroot
> (e dos tarballs da Bootlin, que são SDKs do Buildroot) ou de uma pasta de
> toolchain (`toolchain.importKit`), com o arquivo de toolchain do SDK no kit
> (`roadmaps/40` §7.30). Não medido contra um SDK real ainda.

Bare metal não entra: a libc vem dentro do tarball.

## 4. Onde a IDE procura (além do `PATH`) — implementado

`tools/search_dirs.rs`, lido pelo `ToolDetector::from_environment()` DEPOIS
do `PATH` (o que o usuário escolheu vence o que a IDE encontrou):

```text
~/.local/share/kinein-vectis/toolchains/<id>/<versao>/bin   o que a IDE instalar (§5)
~/.local/xPacks/@xpack-dev-tools/<nome>/<versao>/.content/bin   xpm (XPACKS_STORE_FOLDER)
~/.espressif/tools/<nome>/<versao>/<nome>/bin               idf_tools.py (IDF_TOOLS_PATH)
~/.cargo/bin, ~/.local/bin                                  rustup/cargo install, pipx
/opt/<algo>/bin                                             tarballs da Arm, Bootlin, SDKs
```

Só diretórios que **existem** entram; a lista é determinística (ordem de
nome); as duas variáveis de ambiente são lidas por quem chama e passadas por
parâmetro — a função é pura e o teste a prova com uma `$HOME` falsa.

## 5. O provedor de instalação — FEITO em 2026-09-13 (0.103.0), com a regra decidida

> **Estado (2026-09-13):** entregue como `toolchain.installable` /
> `toolchain.install` (`arquitetura/03`, `roadmaps/40` §7.29). O catálogo tem
> nove releases pinados com o SHA-256 lido na fonte nesse dia; o botão está no
> painel de embarcados. O que o texto abaixo descrevia é o que existe — as
> diferenças: o gatilho é a família do `project.model` (recomendada, não
> obrigatória: o catálogo inteiro aparece), e Espressif/Zephyr ficaram de fora
> pelas razões do §6.

*"Se o usuário abrir um projeto de STM32 e a máquina estiver limpa, a IDE
oferece um botão: Instalar toolchain Arm Cortex-M recomendada."* Entra, com a
forma que a decisão registrada e o pedido do autor juntos permitem:

```text
gatilho     o project.model diz a familia (ESP-IDF, Zephyr, pico-sdk, Cube,
            cargo embarcado) e o toolchain.get diz que o compilador do alvo
            NAO esta' na maquina
botao       "Instalar <toolchain> <versao> na pasta da IDE" — com a URL, o
            tamanho e o sha256 esperado VISIVEIS antes do clique
download    ureq (ja' no core, TLS por rustls) para
            ~/.local/share/kinein-vectis/toolchains/<id>/<versao>/, como JOB
            com progresso e cancelamento; sha256 verificado ANTES de
            desempacotar; `tar` do sistema (GPL, processo) desempacota
fontes      SO' as da §2 com checksum publicado: Arm (.sha256asc), xPack (.sha),
            ATfE (.sha256), Bootlin (.sha256), Espressif (release do
            crosstool-NG), Zephyr SDK (.sha256). Versao PINADA no catalogo,
            com data — nunca "latest"
depois      o detector ja' le a pasta (§4): a toolchain aparece como candidato
            e o kit pode fixa-la; nada e' escrito fora da pasta da IDE, nada
            de sudo, nada de PATH editado
nunca       download sem clique; instalacao no sistema; binario sem checksum
```

## 6. O que NÃO entra, e por quê

- **STM32CubeIDE/plugins como fonte de toolchain**: produto proprietário
  (42 §6); quem tem o GCC dele aponta a pasta.
- **Wokwi, Keil, IAR, SEGGER Embedded Studio**: proprietários (41 §4).
- **`gdb` do tarball da Arm como padrão**: exige libpython específica no
  host (README do release); o `gdb` do sistema com `-i dap` é o padrão, e o
  do tarball fica como candidato para quem o tem funcionando.
- **Baixar "latest"**: sem checksum publicado e sem versão pinada não entra
  (ADR-0003 já decidiu isso para o AppImage; vale igual aqui).

## 7. Fontes (lidas em 2026-09-12)

- Arm GNU Toolchain: https://developer.arm.com/downloads/-/arm-gnu-toolchain-downloads ;
  https://learn.arm.com/install-guides/gcc/arm-gnu/ (padrão de URL e nome dos tarballs;
  o `.sha256asc` de 15.2.rel1 foi baixado e lido)
- xPack: https://github.com/xpack-dev-tools/arm-none-eabi-gcc-xpack/releases ,
  https://github.com/xpack-dev-tools/riscv-none-elf-gcc-xpack/releases ,
  https://github.com/xpack-dev-tools/openocd-xpack/releases ; LICENSE (MIT) lido;
  pastas do xpm: https://xpack.github.io/xpm/docs/user/folders/
- Espressif: https://github.com/espressif/crosstool-NG/releases (esp-16.1.0_20260609)
- Arm Toolchain for Embedded: https://github.com/arm/arm-toolchain/releases
  (release-23.1.0-ATfE, 2026-09-10); LICENSE.TXT (Apache-2.0 WITH LLVM-exception);
  LLVM Embedded Toolchain for Arm: https://github.com/ARM-software/LLVM-embedded-toolchain-for-Arm/releases
  (19.1.5 é o último)
- Bootlin: https://toolchains.bootlin.com/ (listagem de aarch64, 2026.08-1, `.sha256`)
- Yocto SDK: https://docs.yoctoproject.org/sdk-manual/working-projects.html
- Zephyr SDK: https://github.com/zephyrproject-rtos/sdk-ng/releases (v1.0.1)
- Rust: https://doc.rust-lang.org/nightly/rustc/platform-support.html ;
  espup: https://github.com/esp-rs/espup (LICENSE-MIT lido)
- Esta máquina: `rpm -qa` (gcc-aarch64-linux-gnu 16.1.1, gcc-arm-linux-gnu 16.1.1,
  arm-none-eabi-gcc-cs 15.2.0), `aarch64-linux-gnu-gcc -print-sysroot`,
  `rustup target list --installed`, `gdb --version` (17.2)

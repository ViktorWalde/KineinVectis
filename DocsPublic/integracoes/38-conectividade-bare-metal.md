# 38 — Conectividade bare metal: o mapa, medido com um ESP32 no USB

> **Classe: ESTADO.** Tem que ser verdade hoje. Tudo aqui foi **medido nesta
> máquina em 2026-09-11** ou **lido na fonte oficial na mesma data** — e os
> dois estão separados no texto. Licença de cada ferramenta foi lida no arquivo
> de licença do repositório, não no campo `license` da API do GitHub, que errou
> de novo (o `openocd-esp32`, o `debugprobe` e o `openocd` vêm como
> `NOASSERTION`; o `espflash` vem como só `Apache-2.0` e é dual).
>
> **Por que existe.** A decisão do autor de 2026-09-11
> ([`roadmaps/35`](../roadmaps/35-ambiente-cpp-e-embarcados.md) §5.7)
> dizia *"ESP32-C3/C6/S3 existe mas NÃO pode ser plugado agora"* e o escopo era
> *"ARM Cortex-M via probe-rs; RISC-V/ESP32 depois"*. **Na tarde do mesmo dia o
> autor plugou um ESP32** — e pediu o mapa de como um microcontrolador bare
> metal se conecta, o que existe de open source para cada elo, e se o que vale
> para o ESP32 vale para STM32 e Raspberry Pi. Este documento responde as três
> perguntas. Ele **não** é código nem decisão: é o levantamento que vem antes,
> como o [`36`](36-ferramentas-de-embarcados.md) veio antes da etapa 22.

## 0. A resposta curta, para quem não vai ler o resto

```text
O QUE ESTA NO USB     ESP32-D0WD-V3 rev v3.1 (ESP32 CLASSICO, Xtensa LX6),
                      atras de uma ponte CP2102 -> /dev/ttyUSB0. Nao e' um
                      S3/C3/C6: NAO tem USB-JTAG embutido.

O QUE DA PARA FAZER   gravar (esptool/espflash pela serial) e monitorar (UART
JA', SEM COMPRAR NADA 115200) — a permissao ja' esta' certa (dialout).

O QUE NAO DA'         depurar. O ESP32 classico so' depura por JTAG EXTERNO
                      (4 GPIOs + um adaptador FT2232H tipo ESP-Prog), e o
                      software e' o FORK da Espressif do OpenOCD + um GDB
                      xtensa que o Fedora nao tem. O upstream que esta' nesta
                      maquina conhece o alvo mas nao grava nele.

VALE PARA STM32/PI?   Em parte, e a parte que vale e' a maior: o CANAL SERIAL
                      (USB-UART/CDC, DTR/RTS, dialout, ModemManager), a camada
                      de IDENTIDADE (VID:PID -> familia -> chip lido pelo
                      canal), o mecanismo de PERMISSAO (uaccess/plugdev) e o
                      canal de DEPURACAO por sonda (probe-rs/OpenOCD, ja'
                      feito). O que NAO vale e' o protocolo do bootloader: cada
                      fabricante tem o seu (SLIP da Espressif, AN3155/DFU da
                      ST, picoboot/UF2 da Raspberry Pi). A Raspberry Pi SBC
                      (4/5) nem e' este dominio: e' Linux do outro lado — e'
                      o item "SSH remoto" da fila.
```

## 1. O que foi MEDIDO nesta máquina (2026-09-11, ~19:43 → 20:30)

```text
lsusb                Bus 008 Device 002: ID 10c4:ea60 Silicon Labs CP210x UART Bridge
kernel               cp210x converter now attached to ttyUSB0  (19:43:48)
/dev/ttyUSB0         crw-rw---- root:dialout          (0660)
/dev/serial/by-id/   usb-Silicon_Labs_CP2102_USB_to_UART_Bridge_Controller_0001-if00-port0
id                   hugh: hugh, wheel, dialout        (NAO esta' em plugdev; plugdev nao existe)
esptool chip-id      ESP32-D0WD-V3 (revision v3.1), Wi-Fi/BT, dual core, 240 MHz,
                     cristal 40 MHz, MAC 70:4b:ca:04:54:54
esptool flash-id     flash 4 MB (fabricante 5e, device 4016), 3.3 V por strapping
auto-reset           "Connecting....." e "Hard resetting via RTS pin..." — o
                     circuito DTR/RTS da placa FUNCIONA; ninguem apertou BOOT
```

**Quem mais olhou para a porta, e isto importa para "plug and play":**

```text
ModemManager 1.24.2  ATIVO, sem --filter-policy no ExecStart. A porta saiu com
                     ID_MM_CANDIDATE=1 e o journal mostra o MM examinando o
                     dispositivo 4 s depois do plug:
                       19:43:52 [base-manager] couldn't check support for device
                       '.../usb8/8-1': not supported by any plugin
                     Ele desistiu, mas durante a sondagem a porta e' dele. Um
                     `esptool` nesses segundos pode falhar ou ler lixo — e a
                     doc do esptool avisa exatamente isso ("other software such
                     as modem-manager on Linux is not trying to interact with
                     it"). A saida documentada pelo proprio MM e' uma regra
                     udev com ENV{ID_MM_DEVICE_IGNORE}="1" para o VID:PID.
brltty 6.8           INSTALADO, servico INATIVO, e sem regra udev no disco que
                     cite 10c4 — o sequestro classico do CP210x/CH340 pelo
                     brltty NAO acontece aqui hoje. Continua sendo o segundo
                     suspeito quando a porta "some" em outra maquina.
```

**Permissão USB, o que há no disco** (é o insumo da fatia 4.3 da frente F):

```text
/etc/udev/rules.d/99-openocd.rules   do autor (26 ago): ST-Link V2/V2-1/V3,
                                     J-Link, CMSIS-DAP, ESP32 USB-JTAG
                                     303a:1001 e Raspberry Pi 2e8a, todos
                                     MODE="0666", GROUP="plugdev"
/usr/lib/udev/rules.d/60-openocd.rules  do pacote openocd do Fedora. E' o
                                     contrib/60-openocd.rules do upstream com
                                     UMA diferenca sistematica: o Fedora
                                     REMOVE o GROUP="plugdev" (zero
                                     ocorrencias) e deixa so' TAG+="uaccess"
                                     (120 regras no Fedora, 127 no upstream de
                                     hoje). Cobre 303a:1001 e 303a:1002.
69-probe-rs.rules                    NAO INSTALADO. O oficial
                                     (probe.rs/files/69-probe-rs.rules, sha256
                                     94ebdfa6…75fdc, 156 linhas em 2026-09-11)
                                     usa MODE="660", GROUP="plugdev",
                                     TAG+="uaccess" em cada regra — e a doc
                                     manda criar `plugdev` como grupo de
                                     SISTEMA no systemd >= 258.
```

O que isso ensina, e é a nuance que a fatia 4.3 tem de carregar: no Fedora, o
`TAG+="uaccess"` sozinho já dá acesso ao usuário logado na sessão gráfica (ACL
por seat), sem grupo nenhum. Copiar o `69-probe-rs.rules` oficial **funciona**
pelo mesmo `uaccess`, e o `GROUP="plugdev"` dele só tem efeito se o grupo
existir — hoje **não existe** nesta máquina. O passo oficial e o passo que o
Fedora já deu não são o mesmo, e a IDE tem de dizer qual dos dois está em
vigor, não mandar rodar `sudo` para instalar o que o `uaccess` já resolve.

**Ferramentas nesta máquina:**

```text
PRESENTE   esptool 5.3.1 (pipx, ~/.local/bin; o upstream esta' em 5.4.0 de 2026-09-02)
           probe-rs 0.32.0 — `chip list` traz esp32, esp32c2/c3/c5/c6/c61/c6_lp,
             esp32h2, esp32p4, esp32s2/s3/s31
           openocd 0.12.0+dev-snapshot (2026-07-11), UPSTREAM: target/esp32*.cfg,
             xtensa-core-esp32*.cfg, interface/esp_usb_jtag.cfg,
             interface/ftdi/esp32_devkitj_v1.cfg, board/esp32-wrover-kit-*.cfg,
             board/esp32s3-builtin.cfg, board/pico-debug.cfg, board/rpi4b.cfg,
             target/rp2040.cfg, rp2350.cfg, bcm2711.cfg
           gdb 17.2 multiarch: `set architecture riscv:rv32` OK,
             `set architecture xtensa` -> Undefined item
           arm-none-eabi-gcc 15.2, cargo-embed/cargo-flash 0.32,
           minicom, picocom, screen
AUSENTE    espflash, cargo-espflash, espup, idf.py, xtensa-esp-elf-gcc,
           riscv32-esp-elf-gcc, xtensa-esp-elf-gdb, picotool, pyocd,
           stm32flash, dfu-util, st-flash, tio, alvos rustup
           riscv32*/thumb*/xtensa
           (o Fedora EMPACOTA dfu-util 0.11, stlink 1.8.0 e tio 3.9 — um `dnf
           install` de distancia; NAO empacota esptool, picotool, stm32flash,
           pyocd nem toolchain xtensa — medido com `dnf` em 2026-09-11)
```

**E o upstream OpenOCD desta máquina NÃO grava ESP32.** Os `.cfg` não
declaram `flash bank` nenhum para ESP, o binário não tem `program_esp`, e o
`help` não lista comando `esp`. Ele **conhece o alvo** (target `esp32`,
`xtensa xtdef LX`), o que serve para halt/step/breakpoint por JTAG — não para
escrever a flash. Quem grava por JTAG é o fork `openocd-esp32` (§3.1).

## 2. O mapa: três canais e uma camada de identidade

Todo microcontrolador bare metal chega ao PC por **até três canais**, e o que
muda entre famílias é *quais* deles existem e *que protocolo* corre em cada um.
É este o mapa que generaliza; o resto é tabela.

```text
CANAL A — SERIAL        um tty. /dev/ttyUSB* (ponte USB-UART externa: CP210x,
                        FTDI, CH340/CH9102) ou /dev/ttyACM* (CDC nativo do chip
                        ou da sonda). Carrega TRES coisas:
                          A1 o protocolo do BOOTLOADER DE ROM (gravar sem sonda)
                          A2 o CONSOLE/monitor do firmware
                          A3 as linhas DTR/RTS, que as placas ligam ao RESET e ao
                             pino de boot para entrar no bootloader SEM apertar
                             botao
                        Permissao: grupo `dialout` (Fedora/Debian) ou `uucp` (Arch).

CANAL B — DEPURACAO     SWD ou JTAG, atravessado por uma SONDA (ST-Link, J-Link,
                        CMSIS-DAP, FTDI) ou EMBUTIDO no chip (USB-JTAG da
                        Espressif). Quem fala: probe-rs (DAP nativo) ou OpenOCD
                        (GDB remote -> `gdb -i dap`). Grava, para, le memoria,
                        RTT. Permissao: USB cru -> udev (uaccess / plugdev).

CANAL C — BOOTLOADER    o chip se apresenta como OUTRA COISA no USB: um disco
DE MASSA / DFU          (UF2: RP2040/RP2350, muitas placas Adafruit) ou um
                        dispositivo DFU (STM32 0483:df11; ESP32-S2/S3 pela ROM
                        via USB-OTG). Gravar = copiar arquivo (UF2) ou
                        `dfu-util`. Permissao: udev.

IDENTIDADE              1) VID:PID do USB diz a FAMILIA do elo (10c4:ea60 =
                        CP210x = "ha' uma ponte, nao sei o chip"; 303a:1001 =
                        Espressif USB-JTAG = "e' um S3/C3/C6/H2/C5/P4";
                        0483:374b = ST-Link V2-1; 2e8a:000c = Pi Debug Probe;
                        2e8a:0003 = RP2040 em BOOTSEL).
                        2) O CHIP so' se le PELO CANAL: `esptool chip-id`
                        (eFuse, pela serial), `probe-rs info` (DP IDR / part
                        number, pela sonda), `picotool info` (bootrom, pelo
                        picoboot). A IDE nunca adivinha o chip pelo VID da ponte.
```

**O ESP32 que está plugado tem só o canal A.** Não tem B (sem sonda; JTAG
embutido não existe no clássico) nem C (o clássico não tem USB nativo). É o
caso mais pobre da tabela — e por isso é o melhor para provar o canal A, que é
o que todas as famílias compartilham.

## 3. ESP32 em detalhe

### 3.1 O clássico (o que está no USB): Xtensa LX6, ponte externa

**Como entra em modo de gravação** (fonte: esptool, *Boot Mode Selection*,
lido em 2026-09-11): *"The ESP32 will enter the serial bootloader when GPIO0 is
held low on reset"*; GPIO2 *"must also be either left unconnected/floating, or
driven Low"*; o reset é pelo pino EN (CHIP_PU). As placas de desenvolvimento
ligam **DTR → GPIO0** e **RTS → EN** através de dois transistores, com uma
tabela-verdade que impede reset quando os dois são acionados juntos, e um
capacitor de 1–10 µF no EN para o reset ser confiável. É isso que o `esptool`
aciona quando imprime *"Connecting....."* — e foi medido funcionando aqui.

**O protocolo da ROM** (fonte: esptool, *Serial Protocol*): quadros **SLIP**
(`0xC0` nas pontas, `0xDB 0xDC`/`0xDB 0xDD` como escape), comandos `SYNC`
(0x08), `FLASH_BEGIN/DATA/END` (0x02/0x03/0x04), `MEM_BEGIN/END`,
`READ_REG`/`WRITE_REG`, `SPI_ATTACH` (0x0d), `SPI_SET_PARAMS` (0x0b). *"ESP32
always initialises at 115200bps"*; o `SYNC` leva um payload grande que o chip
usa para detectar o baud. Depois do sync o esptool sobe um **stub** para a IRAM
(a menos de `--no-stub`) e é o stub que grava, comprimido e mais rápido. **Este
protocolo é da Espressif** — não tem nada em comum com o da ST nem com o da
Raspberry Pi (§5).

**Gravar — duas ferramentas, duas licenças, uma forma:**

| ferramenta | licença (arquivo lido) | versão | forma na IDE |
| --- | --- | --- | --- |
| `esptool` | **GPL-2.0** (`LICENSE`) | v5.4.0 (2026-09-02); aqui 5.3.1 | **processo**, nunca linkado — regra do `deny.toml` |
| `espflash` | **MIT OR Apache-2.0** (`LICENSE-MIT`, `LICENSE-APACHE`) | v4.6.0 (2026-09-10), MSRV 1.95 | processo; **como crate, BARRADO hoje** — ver abaixo |

O `espflash` como biblioteca parecia o caminho natural para um core em Rust.
**Medido em 2026-09-11, ele não passa no `deny.toml`:** a conexão serial dele
vem do crate `serialport` 4.10.1, que é **MPL-2.0** — copyleft fraco, e a
política do projeto é *"nada de copyleft, nem mesmo LGPL"*. A feature
`serialport` é a que liga `dep:serialport`, `slip-codec`, `reed-solomon`; sem
ela o crate só monta imagem, não fala com chip. Admitir MPL-2.0 é **decisão do
autor**, com o mesmo peso da BSD-3 de 2026-09-04 — e não é decisão deste
documento. Até lá, `espflash` e `esptool` entram do mesmo jeito: processo
filho, saída em evento.

O que o esptool v5 mudou e a IDE tem de saber (fonte: esptool, *Migration
Guide*): o executável é `esptool` (o `esptool.py` avisa deprecação), os
comandos trocaram `_` por `-` (`write_flash` → `write-flash`, `chip_id` →
`chip-id`, `default_reset` → `default-reset`), **todo erro vai para STDERR**, a
saída é colorida e **`NO_COLOR` desliga** — injetada no `Command` do filho,
nunca no processo da IDE (regra do `unsafe`). E ele redesenha linhas com
`ESC[1A ESC[2K` (medido): a lição do `probe.rs` de tirar escape ANSI antes da
tela vale de novo.

**Monitorar:** UART a 115200 no mesmo `/dev/ttyUSB0`. O ESP-IDF e o `espflash
monitor` decodificam o backtrace de um *Guru Meditation* (endereços `0x400d…`)
com o **ELF** do firmware e um `addr2line` do alvo — o `espflash` traz
`addr2line` embutido (feature `cli`), o ESP-IDF usa o do toolchain xtensa. Sem
o ELF, o monitor é só texto — que já é a fatia 4.4 da frente F. Abrir a porta
para monitorar **reseta o chip** se DTR/RTS forem acionados na abertura: o
monitor tem de abrir com as linhas em nível conhecido (o `espflash` tem
`hold-in-reset` e `reset` como subcomandos separados por isso).

**Depurar — e aqui a placa plugada é a mais cara de todas:**

```text
hardware    JTAG por 4 GPIOs — MTDO/GPIO15=TDO, MTDI/GPIO12=TDI, MTCK/GPIO13=TCK,
            MTMS/GPIO14=TMS (fonte: ESP-IDF, JTAG Debugging, "Configure Other
            JTAG Interfaces") — mais um adaptador FT2232H (ESP-Prog, ou o
            ESP-WROVER-KIT que o traz na placa, a 20 MHz). Sem isso, nada.
            ARMADILHA: GPIO12 (MTDI) e' strapping da tensao da flash — "On power
            up ESP32 is sampling binary level on MTDI to set its internal
            voltage regulator" (3.3 V vs 1.8 V). Um adaptador que segure TDI
            alto no boot pode fazer a placa nao subir.
limites     "2 hardware implemented breakpoints" e "2 watchpoints"; breakpoints
            de software em flash e IRAM (ate' 32 + 32) so' existem com o driver
            de flash do OpenOCD da Espressif, que le a tabela de particoes em
            0x8000 para mapear enderecos.
software    openocd-esp32 v0.12.0-esp32-20260831 (2026-09-03), GPL-2.0-or-later
            (COPYING com SPDX) — processo. Traz `program_esp` (grava por JTAG).
            esp-gdb v17.1_20260402 (espressif/binutils-gdb), GPL-3.0 — processo.
            E' o `xtensa-esp-elf-gdb`; o gdb do Fedora NAO tem xtensa (medido).
            Para RISC-V (C3/C6/H2) o gdb do Fedora SERVE.
probe-rs    tem o alvo `esp32` e fala com FTDI (FT2232 esta' na lista de sondas
            da doc), mas a propria issue de rastreio do Xtensa diz que o suporte
            "is pretty fresh and was checked out on classical esp32" — funciona,
            nao e' maduro. E' a UNICA saida que nao precisa do fork do GDB.
```

**Conclusão para esta placa:** o ciclo **compilar → gravar → monitorar** é
inteiramente exercitável hoje, sem comprar nada. **Depurar não é** — e não é
por software: falta o adaptador. Quando houver um ESP-Prog, o caminho de menor
custo é `probe-rs` (já é candidato do papel `debugAdapter`) com `chip=esp32`;
o de maior fidelidade é `openocd-esp32` + `esp-gdb -i dap` pela ponte
`debugServer`/`remoteTarget` que a fatia 2 já construiu — os dois entram sem
protocolo novo no core.

### 3.2 S3/C3/C6/H2/C5/P4: o USB Serial/JTAG muda tudo

A partir do ESP32-C3 (rev 0.3+), Espressif embutiu no chip um controlador
**USB Serial/JTAG** de função fixa — *"cannot be reconfigured to perform any
function other than to provide a serial channel and JTAG debugging
functionality"* — que aparece como **`303a:1001`** e dá **os canais A e B no
mesmo cabo**, sem ponte e sem sonda:

| chip | arquitetura | USB Serial/JTAG | USB-OTG | como aparece |
| --- | --- | --- | --- | --- |
| ESP32 | Xtensa LX6 | não | não | só pela ponte externa (`/dev/ttyUSB*`) |
| ESP32-S2 | Xtensa LX7 | **não** | FS | CDC pelo firmware (TinyUSB) ou DFU da ROM pelo OTG; JTAG só externo |
| ESP32-S3 | Xtensa LX7 | **sim** | FS | `/dev/ttyACM*` + JTAG (`303a:1001`) |
| ESP32-C2 | RISC-V | não | não | ponte externa |
| ESP32-C3 | RISC-V | **sim** (rev ≥ 0.3) | não | `/dev/ttyACM*` + JTAG |
| ESP32-C6 / H2 / C5 | RISC-V | **sim** | não | `/dev/ttyACM*` + JTAG |
| ESP32-P4 / S31 | RISC-V / (S31: arquitetura não verificada hoje) | **sim** | HS+FS / HS | `/dev/ttyACM*` + JTAG |

(Fonte da matriz: ESP-IoT-Solution, *ESP USB Peripheral Introduction*, e
ESP-IDF, *USB Serial/JTAG Controller Console*, lidos em 2026-09-11. A lista de
chips com USB-JTAG do Rust on ESP Book coincide: *"ESP32-C6, ESP32-H2,
ESP32-S3 and ESP32-C3 (revision 0.3 or later)"*.)

O que isso significa na prática, com fonte:

- **Gravar:** o controlador *"is able to put the ESP32-S3 into download mode
  automatically"* — não há DTR/RTS; o reset é uma sequência própria pelo CDC
  (o `espflash` implementa: *"Custom reset sequence, which is required when
  the device is connecting via its USB-JTAG-Serial peripheral"*,
  `connection/reset.rs`). `esptool` e `espflash` fazem os dois casos.
- **Depurar:** `probe-rs` sem hardware extra (*"Espressif devices equipped with
  the USB-JTAG-SERIAL peripheral can use probe-rs without any external
  hardware"*), com flash, breakpoints e RTT/defmt — é exatamente o caminho que
  a frente F já escolheu para Cortex-M. Para os RISC-V, o `gdb` do Fedora
  também serve via `openocd -f board/esp32c3-builtin.cfg` (upstream tem o
  cfg; **gravar** continua sendo do fork ou do esptool).
- **Limites do próprio controlador** (ESP-IDF): some em *deep sleep* (o PHY
  desliga), trava em *light sleep* com clock gated, e um firmware que
  reconfigure os pinos USB **derruba o dispositivo** — a recuperação é GPIO0
  baixo + reset, à mão. A IDE tem de dizer isso quando a porta sumir, em vez de
  "nenhuma sonda".
- **Permissão:** `303a:1001` já está coberto pelo `60-openocd.rules` do Fedora
  (`uaccess`) e pelo `69-probe-rs.rules` oficial. O `/dev/ttyACM*` do mesmo
  dispositivo continua exigindo `dialout`.

### 3.3 Compilar: ESP-IDF é CMake, e a IDE já fala CMake

**ESP-IDF** (Apache-2.0, `LICENSE`; v6.1 de 2026-08-27; push em 2026-09-11):
o `idf.py` é um *front-end* — *"manages several tools, for example: CMake,
which configures the project to be built; Ninja, which builds the project;
esptool, which flashes the target"*. A doc do build system diz como fazer
**sem ele**: `cmake .. -G Ninja` (CMake ≥ 3.22), `-DIDF_TARGET=esp32`,
`IDF_PATH` apontando para o ESP-IDF e o toolchain no `PATH`; o
`CMakeLists.txt` do projeto inclui `$ENV{IDF_PATH}/tools/cmake/project.cmake`.
**Isso é o `cmake.configure` que a IDE já tem**, com um kit cujo compilador é
`xtensa-esp-elf-gcc` ou `riscv32-esp-elf-gcc` (do `crosstool-NG` da
Espressif, esp-16.1.0_20260609) — e o clangd os enxerga pelo `--query-driver`
da fatia 4.1, por exclusão, sem tocar em código.

**E o build entrega a receita de gravação pronta:** o diretório `build/` sai
com `flasher_args.json` (*"Project flash information in JSON format"*),
`flash_project_args`, `flash_app_args` e `flash_bootloader_args`, invocáveis
como `esptool --chip esp32 write-flash @build/flash_project_args`. **É isto que
uma integração nativa lê** — os offsets (bootloader, tabela de partições, app)
e as flags de flash vêm do build, e a IDE não adivinha nenhum deles. O mesmo
princípio do `compile_commands.json`.

**Rust** (fonte: Rust on ESP Book, *Toolchain*): os RISC-V compilam com o
`rustc` **upstream** — `riscv32imc-unknown-none-elf` (C2, C3) e
`riscv32imac-unknown-none-elf` (C6, H2), *"currently Tier 2"*. Os Xtensa (ESP32,
S2, S3) exigem *"a fork of the Rust compiler for now"*, instalado pelo `espup`
(0.17.1). `esp-hal` está em 1.2.1 (2026-09-08). **Para a placa plugada, Rust é
o caminho caro** (toolchain fork); para um C3/C6 é o barato.

**Arduino-ESP32** é ecossistema do usuário, não da IDE — um projeto Arduino
compila pelo `arduino-cli`, não por CMake, e o autor já tirou AVR/Arduino do
escopo (§5.7). Não entra aqui.

## 4. As ferramentas, medidas (licença lida no arquivo, 2026-09-11)

| ferramenta | para quê | licença (arquivo) | versão / data | forma |
| --- | --- | --- | --- | --- |
| esptool | gravar/identificar Espressif pela serial | GPL-2.0 (`LICENSE`) | v5.4.0 · 2026-09-02 | processo |
| espflash | idem + monitor com defmt e addr2line | MIT OR Apache-2.0 | v4.6.0 · 2026-09-10 | processo (crate barrado: `serialport` MPL-2.0) |
| openocd-esp32 | depurar E gravar Espressif por JTAG | GPL-2.0-or-later (`COPYING`) | v0.12.0-esp32-20260831 · 2026-09-03 | processo (`debugServer`) |
| esp-gdb | GDB xtensa/riscv da Espressif | GPL-3.0 (`gdb/COPYING`) | v17.1_20260402 · 2026-04-08 | processo (`gdb -i dap`) |
| ESP-IDF | framework + build CMake | Apache-2.0 (`LICENSE`) | v6.1 · 2026-08-27 | do projeto do usuário; a IDE configura |
| crosstool-NG (esp) | `xtensa-esp-elf-gcc`, `riscv32-esp-elf-gcc` | GCC (GPL-3 + runtime exception) | esp-16.1.0_20260609 | candidato de compilador |
| probe-rs | flash/debug/RTT por sonda ou USB-JTAG | MIT OR Apache-2.0 | v0.32.0 · 2026-07-22 | já adotado (`debugAdapter`) |
| OpenOCD upstream | alvo esp32/c3/s3, rp2040/2350, bcm2711 | GPL-2.0-or-later | 0.12.0+dev (Fedora 2026-07-11) | processo; **não grava ESP** |
| picotool | RP2040/RP2350 em BOOTSEL: info, load, reboot | BSD-3-Clause (`LICENSE.TXT`) | 2.3.1 · 2026-09-05 | processo |
| debugprobe | firmware da Pi Debug Probe (CMSIS-DAP) | MIT (`LICENSE`) | v2.3.1 · 2026-05-29 | não é da IDE; é o que a sonda roda |
| stm32flash | bootloader USART da ST (AN3155) | GPL-2.0 (`gpl-2.0.txt`) | GitLab, atividade 2026-03-06 | processo |
| dfu-util | DFU USB (STM32 `0483:df11`, ESP32-S2/S3 ROM) | GPL-2.0 | 0.11 (Fedora) | processo |
| stlink (open) | `st-flash`/`st-util` para ST-Link | BSD-3-Clause | v1.8.0 · 2024-01-31 (Fedora tem) | redundante com probe-rs |
| serialport (crate) | abrir tty com termios | **MPL-2.0** | 4.10.1 · 2026-09-08 | **barrado** pelo `deny.toml` |
| tokio-serial (crate) | idem, async | MIT — mas depende de `serialport` | 5.5.0 | barrado pela transitiva |
| rustix (crate) | termios sem MPL | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | 1.1.4, já transitivo no `Cargo.lock` | candidato para o monitor |
| nusb (crate) | enumerar USB sem libusb | Apache-2.0 OR MIT | 0.2.7 · 2026-08-03 | desnecessário: `/sys` e `/dev/serial/by-id` bastam |

**A regra que ordena a tabela:** copyleft roda como processo (o GDB já entra
assim); crate só entra permissivo. O único ponto em que a regra **dói** é o
monitor serial: os dois crates óbvios são MPL na raiz ou na transitiva. Abrir
um tty e configurar termios com `rustix` (ou `libc`, já no lock) são poucas
linhas e sem licença nova — o mesmo custo que o `dap/server.rs` pagou para
esperar uma porta abrir.

## 5. Isso vale para STM32 e Raspberry Pi? Sim — por canal, não por família

| | ESP32 clássico | ESP32-S3/C3/C6 | STM32 | RP2040 / RP2350 | Raspberry Pi 4/5 (SBC) |
| --- | --- | --- | --- | --- | --- |
| **A1 bootloader serial** | ROM, SLIP (esptool) | ROM, pelo CDC | ROM (AN2606) por USART, `stm32flash`; BOOT0 alto no reset | **não** — é o canal C | não se aplica |
| **A2 console** | UART pela ponte, 115200 | CDC `/dev/ttyACM*` | VCP do ST-Link (`/dev/ttyACM*`) ou ponte | CDC do firmware (`2e8a:000a`) ou UART pela Debug Probe | **UART GPIO14/15** com a MESMA ponte CP2102; Pi 5 tem conector de debug UART |
| **A3 reset por DTR/RTS** | sim (DTR→GPIO0, RTS→EN) | não; sequência própria | depende da placa (raro) | não (BOOTSEL é botão ou `picotool reboot -f`) | não |
| **B depuração** | JTAG externo (ESP-Prog) + fork | **embutido** (`303a:1001`) | SWD por ST-Link/CMSIS-DAP (`0483:37xx`), probe-rs, **feito** | SWD por Debug Probe (`2e8a:000c`, CMSIS-DAP), probe-rs `rp2040`/`rp235x`; upstream tem `pico-debug.cfg` | JTAG por GPIO com `board/rpi4b.cfg` (upstream), nicho |
| **C massa / DFU** | não | S2/S3: DFU da ROM pelo OTG | `0483:df11` (`dfu-util`), ROM em F2/F4/F7… | **UF2**: `2e8a:0003` (RP2040) / `2e8a:000f` (RP2350), `picotool` | não |
| **permissão** | `dialout` | `dialout` + udev `303a` | udev `0483` (já no 99 do autor) | udev `2e8a` (já no 99 do autor) | `dialout` |
| **compilador** | `xtensa-esp-elf-gcc` (C); Rust só pelo fork (`espup`) | `riscv32-esp-elf-gcc` (C); Rust com `rustc` upstream (C3/C6); S3 é xtensa | `arm-none-eabi-gcc` (tem) | `arm-none-eabi-gcc` (tem); RP2350 também RISC-V | `aarch64-none-elf` (bare) — ou é Linux |

(VID:PID lidos em: `69-probe-rs.rules` oficial e `60-openocd.rules` upstream
para ST-Link/Espressif/FTDI; `picoboot_connection.h` do picotool para os
`2e8a`; `/usr/share/hwdata/usb.ids` para `0483:df11`, `10c4:ea60`,
`0403:6010`; doc da Debug Probe para `2e8a:000c`.)

**O que generaliza, e por isso o ESP32 clássico é um bom primeiro alvo:**

```text
1  O CANAL SERIAL e' o denominador comum das CINCO colunas. Enumerar
   /dev/serial/by-id, ler VID:PID pelo sysfs, abrir com termios, distinguir
   ttyUSB de ttyACM, e dizer quem mais esta' segurando a porta (ModemManager,
   brltty) — escreve-se UMA vez e serve para todos. E' a fundacao da fatia 4.4
   (monitor UART) e da 4.3 (permissao).
2  A camada de IDENTIDADE e' um mecanismo so': VID:PID -> familia do ELO ->
   chip lido PELO CANAL, e a IDE diz o que leu e onde. E' a decisao "deducao do
   alvo" da §5.7, estendida: `esptool chip-id` para Espressif faz o papel que
   `probe-rs info` faz para ARM.
3  PERMISSAO e' um mecanismo so' com tres formas (dialout para tty; uaccess
   por seat ou plugdev para USB cru; ID_MM_DEVICE_IGNORE para o MM). A fatia
   4.3 que era "estado da regra do probe-rs" vira "estado do canal": qual dos
   tres falta para ESTE dispositivo.
4  O canal B ja' esta' FEITO no core (probe-rs, `gdb -i dap`, debugServer).
   Cada familia nova e' uma linha no catalogo (chip, servidor, gdb do alvo),
   nao um motor.
```

**O que NÃO generaliza, e fingir que sim seria o erro:**

```text
1  O PROTOCOLO DO BOOTLOADER e' do fabricante: SLIP+stub (Espressif), AN3155
   USART / AN5362 DFU (ST), picoboot+UF2 (Raspberry Pi). A IDE nao implementa
   nenhum — ela orquestra a ferramenta oficial de cada um como processo e le a
   saida. "Gravar" e' um JOB com MOTOR por familia, e o motor e' escolhido pelo
   que a identidade leu — nunca por um menu que o usuario preenche as cegas.
2  O DEBUG do ESP32 classico nao e' "probe-rs com outro chip": e' hardware que
   nao esta' na mesa + GDB fork + OpenOCD fork. Dizer isso na tela e' o item 4
   da §5.1 ("quando NAO consegue, diz o que faltou").
3  A RASPBERRY PI 4/5 como computador NAO e' este dominio. E' Linux com SSH do
   outro lado, com o gdbserver dela e o compilador dela — e' exatamente o item
   "SSH REMOTO nativo" que o autor pos na fila em 2026-09-11 (`roadmaps/40`
   §4). O que a Pi compartilha com o ESP32 e' so' o console UART (a mesma
   ponte CP2102 num GPIO) e, em bare metal de verdade (kernel8.img no cartao),
   o JTAG por GPIO — nicho que nao paga fatia propria hoje.
```

## 6. O que isto muda no desenho da IDE — proposta, não decisão

Medido contra o core em 2026-09-11: **nada** nele sabe o que é uma porta serial
(o único `ttyUSB0` nos fontes é um teste negativo do `dap/server.rs`); o
catálogo conhece `arm-none-eabi-gcc/g++`, `probe-rs`, `gdb`, `lldb-dap`; o kit
guarda `chip`, `targetTriple`, `sysroot`, `remoteTarget`, `debugServer`;
`probe.list` roda `probe-rs list`; `build.size` lê o `MEMORY` do `.ld` — e o
ESP-IDF **não tem `MEMORY` legível** para a flash: a capacidade vem da
partição (`0x10000` + tamanho da partição `factory`/`ota_0`) e do `flash-id`.

A ordem abaixo segue a regra da frente F — fio, prova sem hardware, tela,
polimento — e cada fatia tem gate que roda **sem** a placa e exercitação
**com** ela:

```text
E1  serial.list (core)        FEITA em 2026-09-11 (`roadmaps/40` §7.13,
                              protocolo 0.91.0): sysfs + udevadm, sem abrir a
                              porta; permissao por access(2) via rustix (ja'
                              era transitiva); familia do elo; painel com
                              EmbeddedSerialView. Exercitada contra o ESP32
                              real, provada por 3 mutacoes Rust + 3 QML.
E2  permissao por canal       FEITA em 2026-09-17 (`roadmaps/40` §7.43,
    (core + painel)           protocolo 0.114.0): `serial.access` mede, por
                              canal, qual das tres formas falta — grupo/ACL
                              uaccess, ModemManager, regra de sonda — e da' o
                              passo OFICIAL datado (ESP-IDF para o grupo;
                              probe.rs/probe-setup para B; a forma das regras
                              do proprio MM para o ID_MM_DEVICE_IGNORE) e o
                              que a distro JA' fez (60-openocd/49-stlink no
                              Ubuntu, medido). Nunca roda sudo; escreve o
                              comando no terminal da IDE. Medido no ESP32 real.
E3  monitor UART              FEITA em 2026-09-12 (`roadmaps/40` §7.15) na
                              forma DECIDIDA abaixo (processo, nao termios):
                              papel `serialMonitor` no kit, `serial.monitor`
                              abre tio/picocom/minicom/espflash numa aba de
                              terminal; exercitada no ESP32 com o picocom.
E4  gravar como JOB com       `flash.run { engine, args }` com motor por familia:
    motor por familia         esptool/espflash (le flasher_args.json quando ha'),
                              probe-rs download (ja' decidido na §5.7), picotool,
                              dfu-util. Saida em evento (a mesma forma de
                              build.output). Motor SUGERIDO pela identidade, o
                              usuario confirma. Gate: motor falso que grava num
                              arquivo; mutacao: offset errado.
E5  identidade Espressif      `esptool chip-id`/`flash-id` como o `probe-rs
                              info` da §5.7: a IDE le o chip e a flash e SUGERE
                              o kit (IDF_TARGET, compilador, capacidade da
                              flash para o build.size). Exercitavel HOJE.
E6  debug do ESP32 classico   BLOQUEADO por hardware (ESP-Prog) e por toolchain
                              (esp-gdb). Fica como a fatia 3: pausado ate'
                              haver o adaptador. O S3/C3/C6 entra em probe-rs
                              sem fatia nova — e' `chip` no kit.
```

**As decisões do autor — 2026-09-11, à noite, depois de ler este documento.**
A regra que ele deu antes de responder, e que passa a ordenar a frente:
*"seguir o padrão estabelecido no mercado e as soluções open source já
adotadas; apenas incluir algo pronto na IDE"* — não escrever o que já existe.

```text
monitor serial   PROCESSO PRONTO no painel de terminal: `espflash monitor
                 --elf <firmware>` para Espressif (decodifica backtrace) e
                 `tio` para STM32/Pico/Pi (GPL-2.0, empacotado no Fedora). E' a
                 forma da extensao oficial da Espressif para o VS Code (`idf.py
                 monitor` num terminal). Zero codigo serial, zero licenca nova:
                 MPL-2.0 NAO entra no deny.toml, e `serialport`/`espflash`
                 como crate ficam FORA. A IDE escolhe a porta e passa o ELF
gravar           CONFIGURACAO DE EXECUCAO, nao dominio novo: um tipo de Run,
                 como o "Upload" do PlatformIO e o "OpenOCD Download & Run" do
                 CLion. Reaproveita run/runconfig/jobs e o evento de saida. O
                 motor (esptool, probe-rs download, picotool, dfu-util) e'
                 sugerido pela identidade da porta e confirmado pelo usuario
ordem            E1 serial.list -> E3 monitor (processo) -> E5 identidade
                 Espressif (`esptool chip-id`/`flash-id` sugerindo o kit) ->
                 E4 gravar -> E2 permissao por canal. E6 (debug do classico)
                 continua bloqueado por hardware. Na mesma noite esta ordem
                 virou o BLOCO A do roadmaps/41, que poe o ecossistema
                 inteiro (Python, MicroPython, profundidade, frameworks)
                 em fila linear atras dela
placa            um ESP32-C3 ou C6 vai para a mesa: RISC-V com USB-JTAG
                 embutido (303a:1001, ja' coberto pelo udev do Fedora) — flash,
                 debug, RTT/defmt via probe-rs sem hardware extra, gdb do
                 Fedora serve, Rust com rustc upstream. Destrava o E6 sem
                 nenhuma ferramenta fork. Na placa, a porta marcada USB, nao a
                 UART. O classico fica para provar o canal serial
```

## 7. O que NÃO foi provado (a fronteira, dita)

```text
gravar de verdade      nenhum firmware foi escrito no ESP32 hoje — so' leituras
                       (chip-id, flash-id). O ciclo compilar->gravar->monitorar
                       fica para a fatia que o implementar, com o ESP-IDF ou o
                       esp-hal instalados, que hoje NAO estao.
probe-rs + ESP32       sem adaptador FTDI, o alvo `esp32` do probe-rs nao foi
                       exercitado; a maturidade e' a que a issue de rastreio diz.
S3/C3/C6               nao ha' nenhum nesta mesa hoje. Tudo da §3.2 e' fonte,
                       nao medicao.
ModemManager           medido que ele EXAMINA a porta por ~4 s; NAO medido que
                       isso derrube um flash — e' plausivel e documentado pelo
                       esptool, nao observado.
STM32 / Pico           nenhum plugado; a coluna e' fonte + os cfg/rules que
                       estao no disco. O que ja' e' FEITO (probe-rs, ST-Link)
                       foi provado em 2026-09-03 no QEMU, nao em placa.
```

## 8. Fontes (lidas em 2026-09-11)

```text
esptool  Serial Protocol       docs.espressif.com/projects/esptool/en/latest/esp32/advanced-topics/serial-protocol.html
esptool  Boot Mode Selection   docs.espressif.com/projects/esptool/en/latest/esp32/advanced-topics/boot-mode-selection.html
esptool  Migration Guide (v5)  docs.espressif.com/projects/esptool/en/latest/esp32/migration-guide.html
esptool  Troubleshooting       docs.espressif.com/projects/esptool/en/latest/esp32/troubleshooting.html
ESP-IDF  USB Serial/JTAG       docs.espressif.com/projects/esp-idf/en/latest/esp32s3/api-guides/usb-serial-jtag-console.html
ESP-IDF  JTAG Debugging        docs.espressif.com/projects/esp-idf/en/latest/esp32/api-guides/jtag-debugging/
         (index, configure-other-jtag, tips-and-quirks)
ESP-IDF  Build System          docs.espressif.com/projects/esp-idf/en/latest/esp32/api-guides/build-system.html
ESP-IDF  idf.py                docs.espressif.com/projects/esp-idf/en/latest/esp32/api-guides/tools/idf-py.html
ESP-IoT-Solution USB overview  docs.espressif.com/projects/esp-iot-solution/en/latest/usb/usb_overview/usb_overview.html
Rust on ESP Book               docs.espressif.com/projects/rust/book/ (toolchain, tooling/probe-rs)
probe-rs probe-setup           probe.rs/docs/getting-started/probe-setup/  +  probe.rs/files/69-probe-rs.rules
probe-rs Xtensa tracking       github.com/probe-rs/probe-rs/issues/2001
espflash                       github.com/esp-rs/espflash (README, espflash/Cargo.toml, src/connection/reset.rs)
openocd upstream rules         github.com/openocd-org/openocd/blob/master/contrib/60-openocd.rules
openocd-esp32                  github.com/espressif/openocd-esp32 (COPYING, releases)
esp-gdb                        github.com/espressif/binutils-gdb (gdb/COPYING, releases)
picotool                       github.com/raspberrypi/picotool (README, LICENSE.TXT, picoboot_connection.h)
Debug Probe                    raspberrypi.com/documentation/microcontrollers/debug-probe.html
ST AN2606 / AN3155 / AN5362    st.com (system memory boot mode; USART; USB DFU)
dfu-util DfuSe                 dfu-util.sourceforge.net/dfuse.html
ModemManager                   `man ModemManager` 1.24.2 (--filter-policy) e /usr/lib/udev/rules.d/*mm*
crates.io                      espflash 4.6.0, serialport 4.10.1, tokio-serial 5.5.0, rustix 1.1.4,
                               nusb 0.2.7, esp-hal 1.2.1, espup 0.17.1 (licenca no campo `license` da versao)
```

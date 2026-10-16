# `serial.*` — o domínio no protocolo IPC

> Parte do [03](../03-protocolo-ipc.md), que tem o formato base, as listas
> canônicas de métodos e eventos e as regras. Movido sem mudar o texto em
> 2026-10-16 ([61](../../roadmaps/61-organizacao-da-documentacao.md) §3.7, O6).

## `serial.*` — as portas seriais USB

Domínio novo no protocolo `0.91.0` (E1 do `integracoes/38` §6, 2026-09-11). A
porta serial é o **canal que toda família bare metal compartilha** — bootloader
de ROM, console e as linhas DTR/RTS de reset — e por isso é a fundação do
monitor UART e do "Gravar".

```text
serial.list     {}                   -> { ports: [SerialPortInfo], hint? }
serial.monitor  { device, baud? }    -> { id, command, tool }   (aba de terminal; 0.92.0)
serial.identify { device, tool? }    -> { jobId, command }      (job; 0.112.0)
serial.access   { device? }          -> { channels: [AccessChannel] }   (0.114.0)
serial.files    { device, action, path?, local?, tool? } -> { jobId, command }   (job; 0.116.0)
                action: list | get | put | rm | mkdir

event.serial.files { jobId, device, action, path, command, success, error?,
                     entries?: [{ name, size, directory }], local?, raw }

AccessChannel   kind (serial|probe|modemManager), device?, ok, detail, problem?,
                fix? { steps: [{ explanation, command }], sourceUrl, checkedOn },
                distroDidIt?

event.serial.identified { jobId, device, command, success, error?,
                          identity?: SerialIdentity, target?: TargetModel, raw }
SerialIdentity  chip? (esp32c3), chipDescription?, features[], crystal?, usbMode?,
                mac?, flashManufacturer?, flashDevice?, flashSize? (4MB), flashSizeBytes?

SerialPortInfo  device, byId?, kind (usbUartBridge|usbCdc), vid, pid,
                manufacturer?, product?, serial?, interface?, driver?, family?,
                access { readableWritable, mode, group?, hint? },
                modemManager? { candidate, ignored, running }
```

**Não exige workspace**, ao contrário do `probe.list`: não há ferramenta vinda
do kit — é o sysfs desta máquina, o mesmo com ou sem projeto aberto.

**Nunca abre a porta.** Abrir um tty aciona DTR/RTS na maioria das pontes
(CP210x, CH340, FTDI) e isso **reseta a placa**; um `serial.list` que
resetasse o firmware a cada abertura do painel seria um defeito. A permissão é
medida com `access(2)`, que honra ACL — é assim que `TAG+="uaccess"` dá acesso
ao usuário da sessão sem grupo nenhum; um `stat` sozinho mentiria nesse caso.

**De onde vem cada campo:** VID:PID, `manufacturer`/`product`/`serial` e
`bInterfaceNumber` subindo de `/sys/class/tty/<n>/device` até o diretório USB
com `idVendor` (ABI documentada do kernel); `driver` do link `device/driver`;
`byId` de `/dev/serial/by-id`; `modemManager` de `udevadm info -q property`
(`ID_MM_CANDIDATE`, `ID_MM_DEVICE_IGNORE`) mais `/proc/*/comm` — e é `null`,
não `false`, quando o `udevadm` não existe. Medido em 2026-09-11: o
ModemManager examinou o ESP32 4 s depois do plug.

**`family` fala do ELO, nunca do chip.** `10c4:ea60` é *"ponte USB-UART
CP210x — o chip do outro lado não se lê pelo USB"*; `303a:1001` é *"Espressif
USB Serial/JTAG — o próprio chip"*. A identidade do chip vem **pelo canal**
(`esptool chip-id`, `probe-rs info`), que é a fatia E5. Fontes da tabela no
`integracoes/38` §5.

**`serial.monitor` é o monitor como PROCESSO numa aba de terminal** (E3 do
`integracoes/38` §6, decisão do autor em 2026-09-11: nunca código serial
nosso). A ferramenta vem do papel novo do kit, `serialMonitor` — candidatos
`tio`, `picocom`, `minicom`, `espflash`, `mpremote` nessa ordem —, com duas
regras a mais que o catálogo não conhece: **projeto MicroPython e `mpremote`
detectado → `mpremote connect <dev> repl`** (`0.102.0`: num firmware
MicroPython o monitor É o REPL — um tio a 115200 mostraria o mesmo texto sem
o raw-paste nem o Ctrl-] de sair; o mpremote é o último do catálogo e nunca
vence sozinho fora de MicroPython); chip Espressif no kit **e** `espflash`
detectado → `espflash monitor --elf <ELF>`, que decodifica o backtrace;
escolha **fixada** pelo autor vence tudo. **Bloco E (`0.117.0`):** antes de tudo que não foi fixado, o wrapper do
framework — num projeto ESP-IDF com ativação nesta máquina, o **IDF
Monitor** (`bash -c '<wrapper>' idf <ativação> -p <dev> monitor`; decodifica
o backtrace com o ELF do build); num PlatformIO com `pio`, `pio device
monitor -p <dev> -b <baud>`. Nenhum dos dois exige monitor no catálogo.
Linhas de comando lidas na fonte: `tio -b`,
`picocom -b`, `minicom -D … -b`, e no espflash é `--monitor-baud` (o `--baud`
dele é o de **gravação**). Baud ausente = 115200. Sem nenhum monitor
instalado, `TOOL_NOT_FOUND` com o que instalar. Exige workspace (a aba nasce
no cwd do projeto) e volta como sessão de terminal, com o comando no título.

**A `hint` de acesso é o passo oficial, nunca um `sudo` que a IDE rodaria:**
*"`/dev/ttyUSB0` é `crw-rw----` do grupo `dialout` e você não está nele …
`sudo usermod -aG dialout $USER` e sair/entrar da sessão. A IDE não roda isso."*

**`serial.identify` é a identidade PELO CANAL (E5 do `integracoes/38` §6,
`0.112.0`)** — o único `serial.*` que ABRE a porta, e por isso um job pedido
por clique: o esptool puxa DTR/RTS e a placa reseta para o bootloader de ROM.
Linha: `esptool --port <device> --chip auto --before default-reset --after
hard-reset flash-id` (`--after hard-reset` devolve a placa ao firmware ao
terminar). A v4 (`esptool.py`, argparse) não conhece `flash-id`: o job vê
"invalid choice"/"No such command" e repete com `flash_id`; o `command` do
evento é o que rodou por último. O parser é **tolerante** como o do
`probe.list` — o formato foi lido no código do esptool 5.4.0 desta máquina
(`Chip type:`, `Features:`, `Crystal frequency:`, `USB mode:`, `MAC:`,
`Manufacturer:`, `Device:`, `Detected flash size:`; `BASE MAC:`/`MAC_EXT:`
não confundem o `MAC:`; `Unknown` não vira tamanho) — linha que não casa é
ignorada e `raw` volta sempre; `success` exige `Chip type:` lido E exit 0.
`chip` é a chave do IDF (`ESP32-C3 (QFN32) (revision v0.4)` → `esp32c3`;
`ESP8685/8686` → `esp32c3`, `ESP8684` → `esp32c2`). **`target`** é o que o
chip sugere pelas MESMAS tabelas de família e motores do `project.model`
(`espressif`; `esptool`/`espflash`/`probe-rs` nos chips com USB-JTAG, sem
depurador no ESP32 clássico), com `evidence` dizendo que veio do
`flash-id`; nada vira kit sem `toolchain.setKit`. Não exige workspace.
Recusas ANTES de abrir a porta: `device` inexistente → `INVALID_PARAMS`;
sem leitura/escrita → `INVALID_REQUEST` com a `hint` de `serial.list`; sem
`esptool`/`esptool.py` no PATH (ou `tool` dado) → `TOOL_NOT_FOUND` com `pipx
install esptool`. Prazo de 30 s no job (o esptool tenta sincronizar várias
vezes): estourou → `success: false` com "não respondeu em 30 s"; cancelado
(`job.cancel`) → mata o esptool e diz "cancelada". `JobRisk::Medium` porque
reseta a placa; nada é escrito nela.

**`serial.files` são os ARQUIVOS NA PLACA de um MicroPython (C2 do
`roadmaps/41` bloco C, `0.116.0`; referência de UX: o "Files on device" do
Thonny)** — `mpremote connect <device> fs <ls|cp|rm|mkdir>` como job, um por
vez (o mpremote prende a porta). O `:` marca o lado da placa e é a IDE que
o põe: `path` vai sem ele (`main.py`, `lib/wifi.py`; ausente só em `list`
= a raiz); `get` é `cp :<path> <local>`, `put` é `cp <local> :<path>`. Um
`local` relativo é sob o workspace (sem workspace, só absoluto); a tela
baixa para `placa/<caminho>` — o espelho da placa, que nunca colide com o
código do projeto e pode ser baixado de novo (o core cria a pasta). O
formato do `ls` foi medido no ESP32 do autor com o mpremote 1.29.0
(`{tamanho:12} {nome}[/]`, uma linha verbosa `ls :<path>` antes) e lido
no código dele (`commands.py::do_filesystem`). **Todo `fs` interrompe o
programa da placa** (raw REPL, soft reset ao sair) — por isso até o `ls`
é gesto explícito e `JobRisk::Medium`; `put`/`rm`/`mkdir` escrevem a
flash e são `High` (a tela pede o segundo clique). **A primeira conexão
pode falhar:** medido com o firmware do autor inundando a UART (1,3 MB de
binário em segundos), o mpremote morreu com `could not enter raw repl`;
a segunda entrou — o job repete UMA vez nesse caso, e as duas saídas
viajam em `raw`. `error` é a linha `mpremote: <cmd>: <caminho>: <motivo>.`
que ele escreve no stderr (sem o prefixo). Recusas antes de tocar a
porta: pedido inválido (`:` na frente, `rm` sem `path`, `get` sem `local`,
`put` de arquivo inexistente) → `INVALID_PARAMS`; porta inexistente ou
sem acesso → como `serial.identify`; sem `mpremote` → `TOOL_NOT_FOUND`.
Prazo de 120 s por execução; cancelar mata o processo. O `put` do mesmo
conteúdo não regrava: o mpremote confere o hash e diz `Up to date`
(medido: o `main.py` do autor reenviado, SHA-256 igual antes e depois).

**`serial.access` é a permissão POR CANAL (E2 do `integracoes/38` §6,
`0.114.0`)** — a fatia 4.3 redesenhada: para CADA canal, o que falta e o
passo oficial. Não exige workspace; não roda nada. Canais: **`serial`**, por
porta — `ok` é o `access(2)` do `serial.list`; `detail` diz o modo, o grupo
dono e se o processo está nele (`/proc/self/status` `Groups:` × `/etc/group`);
quando o acesso vem sem grupo e as propriedades udev trazem `TAGS=…:uaccess:…`,
`distroDidIt` credita a ACL do udev; sem acesso, `fix` é `sudo usermod -a -G
<grupo> $USER` + re-login (fonte: ESP-IDF *Establish Serial Connection*,
2026-09-17) para `dialout|uucp|plugdev|tty`, ou uma regra `TAG+="uaccess"`
para a VID:PID quando o grupo é outro. **`modemManager`**, por porta, só
quando o `udevadm` respondeu — `ok` = não (rodando ∧ candidata ∧ sem regra);
`fix` escreve `ATTRS{idVendor}=="…", ATTRS{idProduct}=="…",
ENV{ID_MM_DEVICE_IGNORE}="1"` em `/etc/udev/rules.d/77-mm-kinein-<vid>-<pid>.rules`
(o 77 corre ANTES do `80-mm-candidate.rules` que marca toda tty; a forma é a
das regras que o pacote instala) e recarrega o udev. **`probe`**, da máquina
— procura `*probe-rs*|*openocd*|*stlink*|*jlink*|*cmsis*.rules` em
`/etc/udev/rules.d`, `/usr/lib/udev/rules.d` e `/lib/udev/rules.d` (pastas
canônicas iguais contam uma vez); achado fora de `/etc` é `distroDidIt`;
nada achado → os três passos do probe.rs *Probe Setup* (baixar
`69-probe-rs.rules`, `udevadm control --reload`, `udevadm trigger`,
citados). Cada `fix` traz `sourceUrl` e `checkedOn`; a UI escreve o
`command` no terminal da IDE pelo mesmo caminho do painel de instalação.

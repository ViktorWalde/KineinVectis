# `project.*` — o domínio no protocolo IPC

> Parte do [03](../03-protocolo-ipc.md), que tem o formato base, as listas
> canônicas de métodos e eventos e as regras. Movido sem mudar o texto em
> 2026-10-16 ([61](../../roadmaps/61-organizacao-da-documentacao.md) §3.7, O6).

## `project.*` — o modelo do projeto embarcado

Domínio novo no protocolo `0.93.0` (pilar 0 do `roadmaps/42`, 2026-09-12).
`workspace.open` diz *que build system* a raiz tem; este domínio diz *o que o
projeto é*.

```text
project.model {} -> ProjectModel        (exige workspace)
event.project.changed  ProjectModel     (ao abrir o workspace; ao fim de
                                         event.cmake.finished e event.build.finished)

ProjectModel   root, embedded, frameworks[], sdks[], artifacts, target, hints[]
FrameworkInfo  framework (espIdf|zephyr|picoSdk|platformIo|stm32Cube|cargoEmbedded|
               microPython|yocto|buildroot), evidence (caminho relativo do
               marcador), detail? (IDF_TARGET, PICO_BOARD, DeviceId do Cube,
               triple do cargo, MACHINE do Yocto, ambientes do PlatformIO)
SdkRequirement id, label, env?, path?, found, hint?
ProjectArtifacts elf[], bin[], hex[], uf2[], map[], flasherArgs?, partitionTable?,
               memoryX?, linkerScripts[], flashRecipe?, partitions?
FlashRecipe    chip?, flashMode?, flashSize?, flashSizeBytes?, flashFreq?, before?,
               after?, stub, files[] { offset, file (absoluto), name?, encrypted }
               — LIDO do flasher_args.json (forma do template
               components/esptool_py/flasher_args.json.in do ESP-IDF)
PartitionTable tableOffset, entries[] { name, kind, subtype, offset, size, flags? },
               end, unreadable[] — LIDA do partitions.csv com os offsets em
               branco resolvidos como o gen_esp32part.py (4 KB; app a 64 KB)
TargetModel    chip?, family?, triple?, flashEngine?, monitor?, debugAdapter?,
               evidence[]  — uma linha por dedução
```

**A detecção desce até 3 níveis e LÊ o marcador**: um `CMakeLists.txt` é
ESP-IDF se inclui o `project.cmake` do `IDF_PATH`, Zephyr se faz
`find_package(Zephyr)`, pico-sdk se chama `pico_sdk_init()`; um `main.py` é
MicroPython se importa `machine`/`board`. Pastas de saída (`build/`, `target/`,
`.pio/`, `.kinein/`) não contam. Entre dois achados do mesmo framework vence o
marcador que **decide** (`.cargo/config.toml` com o triple, não um `memory.x`
solto) e, empatando, o mais raso. Um `CMakeLists.txt` comum **não** é
embarcado; um `main.py` que não importa hardware **não** é MicroPython.

**O alvo tem evidência ou não tem alvo.** O chip do **kit** vence o do
framework (é a palavra do usuário); a família vem do chip ou do triple; os
motores vêm da família — e o ESP32 clássico (sem USB-JTAG) recebe `debugAdapter`
**ausente** com a evidência dizendo que depurar exige ESP-Prog, em vez de um
`probe-rs` que não funcionaria.

**"Achado" só vem de variável, pasta padrão ou binário.** `IDF_PATH` definido
mas apontando para pasta inexistente é `found: false`; a toolchain xtensa fora
do `PATH` mas em `~/.espressif/tools` é `found: true` com o caminho. O alvo
rustup não é medido aqui (é processo) e diz isso no `hint`. Nenhum comando roda.

**O que era só localizado passou a ser LIDO** (segunda fatia, 2026-09-12): a
receita de gravação com os caminhos resolvidos contra o `build/`, e a tabela
de partições — porque a "flash" de um ESP32 não é o `.ld`, é a partição
`app`, e é ela que o tamanho e o "Gravar" vão consumir. Linha de CSV que não
se lê vai em `unreadable`, nunca some.

**Fixtures reais e mínimas** de cada framework moram em
`scripts/fixtures/projetos/`; são elas que os testes leem.

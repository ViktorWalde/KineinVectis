# `build.run`, `build.size` — o domínio no protocolo IPC

> Parte do [03](../03-protocolo-ipc.md), que tem o formato base, as listas
> canônicas de métodos e eventos e as regras. Movido sem mudar o texto em
> 2026-10-16 ([61](../../roadmaps/61-organizacao-da-documentacao.md) §3.7, O6).

### `build.size` — o tamanho do ELF (síncrono)

**Desde 2026-09-12 (pilar 0 do `roadmaps/42`)**, num projeto ESP-IDF a lista
de regiões ganha a partição `app` que o `flasher_args.json` aponta — usado =
tamanho da imagem, capacidade = a partição que **começa** naquele offset. O
`.ld` do IDF não declara a flash; a partição é a flash.

Implementado no protocolo `0.90.0`. **Síncrono**, ao contrário do `build.run`:
o `size` lê um arquivo e volta em milissegundos. `{ "program"? }` — ausente, o
core resolve o ELF como o `debug.start`. Roda `<prefix>size -A` (o prefixo vem
do compilador cross do kit: `arm-none-eabi-gcc` → `arm-none-eabi-size`; sem
cross, o `size` do sistema) e responde `SizeReport { toolAvailable, tool,
program, sections: [{ name, size, addr }], regions: [{ name, used, size }],
rawOutput }`. As `regions` saem do bloco `MEMORY` do único `.ld` do workspace,
com `used` = soma das seções **alocadas** cujo endereço cai na região —
`.comment`/`.ARM.attributes` (metadados do ELF, endereço 0) não contam. Sem
linker script legível, `regions` vem vazio e a UI mostra os totais por seção.

### Build (`build.run` — job assíncrono)

Implementado no protocolo `0.6.0`; migrado para **job assíncrono** (ver seção
Jobs e `DocsPublic/arquitetura/ARCHITECTURE.md` §7). Requer workspace aberto. O core valida de
forma síncrona e responde **na hora** com `{ "jobId": "job_N" }`; o build roda
em background (`cargo build --message-format=json` para Rust/Cargo;
`cmake -S/-B` + `cmake --build` em `.kinein/build` para CMake) e é **cancelável**
via `job.cancel` (mata o processo de build).

Desde `0.55.0`, aceita `{ "buildSystem"?: "cargo"|"cmake"|... }`. Em workspace
híbrido, a seleção explícita precisa existir em
`workspace.capabilities.buildSystems`; sem o campo, o `workspace.kind` primário
preserva o comportamento anterior. Sistema ausente retorna `INVALID_PARAMS`
com a lista disponível, antes de criar job.

Enquanto roda, emite os eventos ricos que a UI consome, agora com `jobId`:

```text
event.build.started     { "jobId", "command": "cargo build" }
event.build.output      { "jobId", "stream": "stdout|stderr", "line": "..." }
event.build.diagnostic  { "jobId", "source": "build", "category": "compiler", "severity": "error|warning|note", "message", "file"?, "line"?, "column"? }
event.build.finished    { "jobId", "success", "exitCode", "diagnostics" }   // ou { "jobId", "success": false, "error" }
```

**Os motores de framework (`0.117.0`, bloco E do `roadmaps/41`).** Antes do
tipo do projeto, o `build.run` pergunta ao `project.model` (`build/engine.rs`)
e, se o framework tem wrapper, é ele quem compila — nada de campo novo:

```text
PlatformIO   pio run                                     (platformio.ini; vence tudo)
ESP-IDF      bash -c '. "$1" >/dev/null || …; shift; exec idf.py "$@"' idf <ativação> build
             ativação = <esp-idf>/export.sh (IDF_PATH ou ~/esp/esp-idf) ou o mais novo
             ~/.espressif/tools/activate_idf_*.sh (EIM, v6) — as duas formas do Get Started
Zephyr       west build -d build [-b <placa>]            placa: CACHED_BOARD do build/CMakeCache.txt,
                                                          senão `west config build.board`
pico-sdk     cmake … -DPICO_SDK_PATH=<sdk>                (o CMake de sempre; o SDK de PICO_SDK_PATH
                                                          ou ~/pico/pico-sdk)
```

Diagnósticos: os três wrappers chamam gcc/clang por baixo e o parser
`arquivo:linha:coluna` casa. O título do job diz o motor (`Build (idf.py)`).
Framework reconhecido e ferramenta ausente (sem `pio`, sem `west`, sem
ativação do IDF) é `TOOL_NOT_FOUND` **antes** do job, com o passo que o
`project.model.sdks` já dava (e, no IDF, o `eim install`). O banner da
ativação vai para `/dev/null`; o erro dela, não. Os builds do `idf.py` e do
`west` ficam em `build/` (a pasta de cada um — é onde o `flasher_args.json` e
o `CMakeCache.txt` são lidos); o do pico-sdk continua em `.kinein/build`.

Além destes, o Job System emite `event.job.created` (ao iniciar),
`event.job.output` (fan-out da saida bruta) e `event.job.finished` (ao
encerrar) para a status bar / lista de jobs. O
resultado do build chega por `event.build.finished`, **não** mais na resposta.
Diagnósticos vêm do JSON do cargo (span primário) ou do formato
`arquivo:linha:coluna: nivel: mensagem` de compiladores/CMake e alimentam o
Problems. Validação síncrona antes de iniciar o job: tipos sem integração de
build retornam `INVALID_REQUEST`; sem workspace, `INVALID_REQUEST`. Falhas do
build (ferramenta ausente, erro de compilação) chegam por
`event.build.finished`/`event.job.finished`, não como erro da resposta.

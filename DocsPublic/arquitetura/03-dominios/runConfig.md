# `runConfig.*` — o domínio no protocolo IPC

> Parte do [03](../03-protocolo-ipc.md), que tem o formato base, as listas
> canônicas de métodos e eventos e as regras. Movido sem mudar o texto em
> 2026-10-16 ([61](../../roadmaps/61-organizacao-da-documentacao.md) §3.7, O6).

### Run configurations (`runConfig.list` / `runConfig.save` / `runConfig.delete` / `runConfig.setActive` / `runConfig.flashProposal`)

**0.187.0 — intenção nativa (contrato anterior ao produtor, 2026-10-21):**
`flashProposal` aceita `native?: bool` (padrão falso) e `buildDirectory?: string`.
No modo nativo, retorna também `flash: {version: 1, workspace, selection,
program, args, deviceKey, fingerprint}`. selection tem o mesmo tipo dos parâmetros,
com native=true e escolhas resolvidas; fingerprint é opaco, 64 dígitos hex.
Programa/argv são a prévia estruturada, nunca autorização para executar valores
do cliente sem regeneração. Nesta primeira parte, esptool com receita ESP-IDF
ou firmware instalado é o provider nativo; outros motores recusam esse modo
explicitamente (INVALID_REQUEST), sem fallback. O modo manual mantém seu contrato.
Native requer porta existente/acessível, sem abri-la na prévia; metadados do build
devem comprovar projeto/configuração/chip/app. Sem prova → recusa orientada.

**0.188.0 — probe-rs nativo com alvo CMake e sonda explícita (contrato
anterior ao produtor, 2026-10-22):** flashProposal ganha `target?: string`,
o nome de um alvo do codemodel CMake, e `probe?: string`, o seletor probe-rs
`vvvv:pppp[-IF]:SERIAL` em hexadecimal. Os dois só valem com native=true e
motor resolvido probe-rs; fora disso, INVALID_PARAMS. Nesse motor os dois são
obrigatórios, e device, firmware, buildDirectory e flashSizeBytes são
INVALID_PARAMS. O core consulta a ferramenta do papel debugAdapter (a mesma do
probe.list), fora do laço, com prazo e limite de saída: `mi meta` deve dizer
0.32.x; `chip info` deve reconhecer o chip do kit, com um núcleo da arquitetura
do ELF; `list` precisa ter todas as linhas no formato 0.32.0 e exatamente uma
entrada igual ao seletor, com serial e acessível. O alvo vem do reply do
`.kinein/build`: fonte = raiz, uma configuração, tipo EXECUTABLE e um único
artefato ELF dentro do build dir. Chip só deduzido de framework, versão
diferente, sonda ausente/duplicada/inacessível ou `.probe-rs.{toml,json,yaml,yml}`
na raiz, na pasta do executável ou no home → INVALID_REQUEST com a causa.
Ferramenta ausente → TOOL_NOT_FOUND. Programa/argv:
`download --chip <chip> --probe <seletor> --binary-format elf --verify <ELF>`,
sem reset, apagar tudo, ciclo de energia ou host remoto. `source` diz
ferramenta/versão, alvo, artefato, núcleos, sonda e as variáveis PROBE_RS_*
que não chegam ao processo. A intenção mantém versão 1; deviceKey é o digest
opaco da entrada listada. A listagem não prova acesso exclusivo. Desenho e
aceites no 52 §14.34.

**Ambiente e configuração do esptool nativo (0.4.1h4b2b4d1, 2026-10-23, sem
mudança de forma):**
- **Ambiente.** Na gravação esptool nativa, inclusive a de firmware de
  catálogo, o core remove toda variável `ESPTOOL_*` herdada, tanto da
  consulta de versão quanto da gravação. `source` diz quais foram removidas,
  ou que não havia nenhuma.
- **Configuração.** O core procura `esptool.cfg`, `setup.cfg` ou `tox.ini`
  com cabeçalho `[esptool]` em três lugares: na raiz, que é o diretório da
  gravação, em `~/.config/esptool` (no Windows, `~/AppData/Local/esptool`)
  e no home. Se encontra um, responde INVALID_REQUEST com o caminho, antes
  de qualquer processo. Um candidato que não pode ser conferido também é
  recusado. `source` diz que nenhum arquivo foi encontrado.
- **Revalidação.** O `run.start` repete as duas checagens. Uma intenção
  salva antes desta versão difere no fingerprint e pede outra prévia.

Desenho no 52 §14.38.

**Build ESP-IDF que queima eFuses (0.4.1h4b2b4d2, 2026-10-23, sem mudança de
forma):** a origem nativa ESP-IDF lê também `<build>/config/sdkconfig.json`,
a configuração efetiva que o build gerou. A gravação responde INVALID_REQUEST
quando:
- `SECURE_BOOT` ou `SECURE_FLASH_ENC_ENABLED` é `true`, porque o bootloader
  queima eFuses no primeiro boot;
- a receita marca uma imagem como cifrada;
- o arquivo falta, não é um objeto JSON ou traz uma dessas chaves sem valor
  booleano.

Uma chave ausente vale como desabilitada. O arquivo entra no snapshot, e o
`run.start` recusa quando ele muda. Desenho no 52 §14.39.

**`idf.py flash` nativo (0.4.1h4b2b4d3, 2026-10-23, sem mudança de forma).**
`flashProposal` com `native: true` e `engine: "idf.py"` (ou `idf`) passa a
valer num projeto ESP-IDF com build configurado:
- **Seleção.** `device` é obrigatório; `target`, `probe` e `firmware` são
  recusados.
- **Linha.** Programa: o `bash` do detector. Argv: o `IDF_WRAPPER` com o
  ativador do ESP-IDF do modelo, seguido de
  `-B <build> -p <porta> -b 460800 flash`.
- **Consulta de versão.** `idf.py --version` deve informar ESP-IDF 6.x.
  Desde a 0.4.1h4b2b4d4, ela roda na pasta do ativador, fora do projeto,
  porque o `idf.py` importa o `idf_ext.py` do diretório corrente.
- **Recusas herdadas.** A origem e o eFuse da §14.39 valem para o build atual.
  Um arquivo `[esptool]` no `build/`, na raiz, na pasta do usuário ou no home
  recusa.
- **Extensões do `idf.py` (desde a 0.4.1h4b2b4d4).** Um `idf_ext.py` na raiz,
  ou na pasta de um componente do build, recusa: ele pode redefinir a ação
  `flash`.
- **Ambiente.** O processo e a consulta não recebem `ESPTOOL_*`, `ESPPORT`,
  `ESPBAUD`, `IDF_FLASH_FULL`, `IDF_TRUST_FLASH_CONTENT`,
  `SERIAL_TOOL_EXTRA_*`, nem (desde a 0.4.1h4b2b4d4) `IDF_EXTRA_ACTIONS_PATH`
  e `IDF_EXTENSION_ALLOW_UNTRUSTED`.
- **O que a intenção vincula.** Por conteúdo: descrição, receita,
  `sdkconfig`, `sdkconfig.defaults*`, `config/sdkconfig.json`, o `bash` e o
  ativador. Também a versão e a porta.
- **O que ela não vincula.** As imagens: mudar o código e reconstruir não
  invalida a configuração salva, e mudar a configuração invalida.
- **Avisos.** O comando compila antes de gravar e faz o fast reflash
  conferido por MD5. Desenho no 52 §14.40.

**`pio run -t upload` nativo para ESP32 (0.4.1h4b2b4d4, 2026-10-23, sem
mudança de forma).** `flashProposal` com `native: true` e
`engine: "platformio"` (ou `pio`) passa a valer num projeto PlatformIO cujo
env é espressif32 com upload por esptool:
- **Seleção.** `device` é obrigatório. `target`, `probe`, `firmware` e
  `buildDirectory` são recusados.
- **Env.** É o único que o `pio run` processaria sem `-e`: o único
  `default_envs` ou o único `[env:*]`. Mais de um candidato recusa.
- **Linha.** Programa: o `pio` do modelo. Argv:
  `run -d <raiz> -e <env> -t upload --upload-port <porta>`.
- **Consultas.**
  - `pio system info --json-output` deve informar o Core 6.x.
  - O interpretador do PlatformIO, com `-I`, lê a configuração e a placa pela
    API do PlatformIO, sem instalar nada, sem SCons e sem script do projeto.
    Uma plataforma cuja pasta fica dentro do projeto recusa antes de ser
    carregada.
- **Recusas.**
  - Plataforma diferente de espressif32 ou não instalada.
  - Protocolo diferente de `esptool`, ou `tool-esptool` ausente.
  - `extra_scripts`, `upload_command`, `platform_packages`, `extra_configs`
    ou framework `espidf`.
  - Placa com toque de 1200 bps ou espera de porta nova.
  - Arquivo `[esptool]` na raiz, na pasta do usuário ou no home.
- **Ambiente.** O processo e as consultas não recebem `ESPTOOL_*` nem as
  `PLATFORMIO_*` das opções do projeto, exceto as que localizam a instalação
  e os caches (`CORE_DIR`, `PLATFORMS_DIR`, `PACKAGES_DIR`, `CACHE_DIR`,
  `BUILD_CACHE_DIR`).
- **O que a intenção vincula.**
  - Por conteúdo: o `platformio.ini`, os arquivos da plataforma
    (`platform.json`, `platform.py`, `builder/main.py`), o manifesto da placa
    e o lançador do `pio`.
  - Também as opções calculadas do env e de `[platformio]`, as versões e a
    porta.
- **O que ela não vincula.** Fontes e imagens.
- **Avisos.** O comando compila antes, grava as imagens inteiras, pode
  instalar pacotes e `lib_deps` ausentes e limpa o `.pio/build` quando a estrutura
  muda. Desenho no 52 §14.44.

`runConfig.save {id?, name, command, flash?}` e configs ganham flash opcional.
Com flash, command deve ser exatamente a prévia derivada de programa/argv;
workspace/versão/seleção/fingerprint devem ser válidos. Schema 1 continua legível
e não é reescrito na abertura. Salvar a primeira nativa promove para schema 2;
schema 2 aceita nativas e manuais. Schema 1 contendo flash ou intenção malformada
é armazenamento inválido, preservado. Editar uma nativa sem flash é INVALID_PARAMS,
sem descarte silencioso por cliente antigo. active_command não converte nativa
em shell. Desde a 0.4.1h4b2b3 a UI preserva a intenção em prévia/save/run e
renomeação; a prévia nativa fica somente leitura. Resposta nativa sem objeto
de intenção não vazio é recusada, sem cair no lançador padrão. O modo manual
é explícito; uma linha antiga mencionando gravação não ganha suporte nativo.

run.start regenera contexto/plano e confere fingerprint/porta/arquivos antes do
PTY, com reserva até a saída real. Arquivos de imagem/binário limitados a 1 GiB;
metadados/configuração usam o limite de leitura existente. Mudança, falta de prova,
cancelamento ou incompatibilidade não executam command nem heurística.
Desenho, limites e aceites no 52 §14.30. Métodos/eventos permanecem os mesmos.

Implementado no protocolo `0.27.0` (fatia M2.4). Todos exigem
workspace aberto (qualquer kind). Persistência em `.kinein/runconfigs.json`
com `schemaVersion` (ausência real = vazio; arquivo inválido ou ilegível é
recusado sem sobrescrever dados, desde 0.185.0). Uma config v1 é `{ id, name, command }` — comando shell executado
na raiz pelo `run.start`.

- Todas as quatro operações respondem o mesmo shape:
  `{ configs: [{ id, name, command }], activeId? }` — a UI nunca calcula
  estado derivado.
- `runConfig.save { id?, name, command }`: cria quando `id` está ausente
  (ids `cfg-N`); criar/editar torna a config a ATIVA. Nome/comando vazios →
  `INVALID_PARAMS`.
- `runConfig.delete { id }`: remove; se era a ativa, volta a "automático".
- `runConfig.setActive { id? }`: `id` ausente = automático; id inexistente →
  `INVALID_PARAMS`.
- `run.start {}` (sem `command`) resolve: comando explícito > config ativa >
  heurística (`cargo run` / executável único do CMake / o ponto de entrada
  Python — na placa, com o `device?` escolhido, quando o projeto é
  MicroPython). Só a heurística recebe `device`.
- **`runConfig.flashProposal { device?, engine?, flashSizeBytes?, firmware? }` →
  `{ name, command, engine, source, warnings }` (`0.113.0`, E4 do
  `integracoes/38` §6; `firmware` em `0.116.0`).** "Gravar" como configuração de execução, na forma
  decidida pelo autor em 2026-09-11 (o Upload do PlatformIO, o Download &
  Run do CLion): **sem gravar hardware ou salvar**; desde 0.186.0 pode
  consultar a versão do esptool, fora do laço e com prazo. A resposta é uma
  proposta que a UI mostra como prévia e que vira `runConfig.save` (nome
  `Gravar (<motor>)`) ou `run.start { command }`. `engine` ausente = o
  `target.flashEngine` do modelo; sem ele, uma receita `flasher_args.json`
  no build sugere `esptool` sozinha; sem nada, `INVALID_REQUEST`. As linhas,
  cada uma com a evidência em `source`: **esptool** `--chip <receita|kit>
  --port <device> --baud 460800 --before/--after <receita> [--no-stub]
  write-flash --flash-mode/size/freq <receita> <offset> '<imagem>'…` — tudo
  do `flasher_args.json` que o ESP-IDF escreve (nada adivinhado; `device`
  obrigatório: gravar na placa errada é pior que um clique; sem receita,
  "compile o projeto"); **probe-rs** `download --chip <chip> '<ELF mais
  novo>'` (chip do kit/modelo obrigatório); **picotool** `load -f -x
  '<UF2>'`; **dfu-util** `-a 0 -s 0x08000000:leave -D '<BIN>'` **só para
  STM32** — outra família não tem tabela, e sem tabela não há comando.
  Caminhos entre aspas simples POSIX (a linha roda por `sh -c`).
  `warnings`: imagens marcadas como cifradas (`write-flash` grava em claro;
  acrescente `--encrypt`), e `flashSizeBytes` (a flash que `serial.identify`
  leu) menor que a da receita. Erros: ferramenta ausente → `TOOL_NOT_FOUND`
  com o passo; motor desconhecido → `INVALID_PARAMS`; o resto (sem build,
  sem porta, sem chip, família sem tabela) → `INVALID_REQUEST`. A linha
  salva é editável como qualquer configuração manual. Desde 0.186.0, a
  nova prévia seleciona automaticamente a sintaxe conhecida v4/v5; comandos
  shell já salvos não são interpretados ou reescritos.
- **Os wrappers dos frameworks (`0.117.0`, bloco E):** `engine: "idf.py"`
  → `bash -c '<wrapper>' idf '<ativação>' -p '<porta>' flash` (a linha do
  "Start a Project": `idf.py -p PORT flash`; a porta é exigida, como no
  esptool; sem ativação nesta máquina, `TOOL_NOT_FOUND` com o passo);
  `engine: "west"` → `'<west>' flash -d build` (o **padrão** de um projeto
  Zephyr, pelo `target.flashEngine` do modelo; uma porta escolhida vira
  aviso — o runner do board grava pela sonda/USB); `engine: "platformio"`
  (ou `pio`) → `'<pio>' run -t upload [--upload-port '<porta>']` (o padrão
  de um PlatformIO). Num ESP-IDF o padrão continua sendo o esptool com a
  receita do build; o `idf.py` é escolha explícita.
- **`firmware` (`0.116.0`, C5 do `roadmaps/41` bloco C):** o id de um
  firmware do catálogo (`toolchain.installable`, `kind: firmware`) já
  BAIXADO substitui os artefatos do build. A linha é a da página da placa
  no micropython.org (lida em 2026-09-17): **esptool** `--chip <chip da
  página> --port <device> --baud 460800 --before default-reset --after
  hard-reset write-flash <offset> '<arquivo>'` — `0x1000` no ESP32 clássico,
  `0x0` em C3/S3 —, com `warnings` para a primeira instalação (a página
  manda `erase-flash` antes; a linha pronta vai no aviso, nunca na de
  gravar), para um chip do kit diferente do da página, e para a flash
  identificada menor que o arquivo; **picotool** `load -f -x '<uf2>'` (o
  mesmo que copiar para o drive `RPI-RP2`, como a página manda). O motor é
  o da página: `engine` diferente → `INVALID_PARAMS`; firmware não
  baixado, id de toolchain ou desconhecido → `INVALID_REQUEST` com o passo.
  `name` é `Gravar firmware (<rótulo>)`.

**Sintaxe por versão do esptool (0.186.0, contrato anterior ao produtor):**
o próprio binário selecionado responde `version`, sem parâmetro de porta ou
operação serial. Consulta adiada com prazo de 5 s; falha, timeout, saída
inválida/discordante ou principal diferente de 4/5 → INVALID_REQUEST com
causa e orientação. Ferramenta ausente continua TOOL_NOT_FOUND. Não adivinhar
versão pelo nome esptool/esptool.py nem tentar gravar para descobrir sintaxe.
Na v4 usar `write_flash`, `--flash_mode/size/freq`, resets com underscore e
`erase_flash` no aviso; na v5, nomes correspondentes com hífen. Normalizar
somente argumentos estruturados da sintaxe, preservando caminhos/valores de
imagem. `source` registra binário e versão consultados. Vale para receita
de projeto e firmware do catálogo; outros motores não consultam esptool.
Schema 1, métodos, eventos e shape da resposta permanecem.
Na ponte Qt, o id já existente do JSON-RPC acompanha sucesso/falha da
prévia. Workspace/porta alterados invalidam o resultado; resposta antiga
ou duplicada não conclui outro pedido nem habilita salvar/executar. Uma
falha genérica sem id não substitui a resposta correlacionada.

**Armazenamento rigoroso (0.185.0, contrato anterior ao produtor):** erro de
leitura, JSON inválido, schema desconhecido, arquivo não regular, identidades
duplicadas, ativa inexistente, entrada vazia ou campo não reconhecido são INVALID_REQUEST com caminho
e orientação. Os quatro métodos preservam o conteúdo; não há reparo automático.
Parâmetro inválido continua INVALID_PARAMS. Mutações são serializadas entre
instâncias atualizadas, com recusa em conflito; publicação usa substituição
atômica existente. Schema 1 permanece. run.start sem comando não cai na
heurística se não pôde ler a configuração ativa; comando explícito continua
independente do arquivo. ConfigAction delega ao mesmo armazenamento.

O armazenamento usa o mesmo limite de leitura do editor (1 MiB em 2026-10-21).
Leitura acima do limite é INVALID_REQUEST; atualização que o excederia é
INVALID_PARAMS antes de publicar, preservando a configuração anterior.


### dfu-util nativo — arquivo DfuSe explícito (0.194.0, 2026-10-26)

`runConfig.flashProposal` aceita `dfu?: {image, destination:{vid,pid,serial,
path,configuration,interface,alternate}}`, exclusivamente com `native=true`
e motor resolvido `dfu-util`. Os demais campos de destino (device, probe,
target, firmware, buildDirectory, flashSizeBytes) são incompatíveis. Outros
motores e o modo manual recusam dfu. O arquivo explícito dentro do workspace
é a origem contratada; não se deduz relação com o build ativo. Prévia mostra
conteúdo, elementos/endereço/tamanho e apagamento por página.

Recorte inicial: DfuSe v1/1.1a, um Target e até 128 elementos ordenados,
sufixo e CRC válidos, até 32 MiB; VID/PID exatos 0483:df11, configuração 1,
interface 0, dispositivo já em DFU. Serial obrigatório sem vírgula/controle,
caminho USB canônico e alternativa do arquivo igual à selecionada. Exige
lista completa e uma única instância VID/PID/serial. O mapa Internal Flash
precisa cobrir os elementos em regiões apagáveis e graváveis, dentro de
0x08000000–0x09000000. BIN sem o vínculo da 0.195.0 abaixo, múltiplos targets, Runtime, memória externa e
Option Bytes ficam recusados. Não pede reset, leave, unprotect, mass-erase,
force ou espera por reconexão. Contrato e fontes no [52 §14.52](../../roadmaps/52-arquitetura-executavel-da-0.4.md#1452-041h4b2b4n2--dfuse-escolhido-e-destino-usb-revalidado-desenho-2026-10-26).

A intenção versão 1/schema 2 transporta a seleção normalizada e o fingerprint
com contexto, arquivo, ferramenta e interface observada. `run.start` regenera
tudo, compara, verifica revisões e usa o PTY existente com argv literal.
Reserva USB por VID/PID/serial, comum a todas as alternativas da mesma
instância, acompanha o processo e o guardião após crash. Conflito responde
SERIAL_PORT_BUSY com scope=usb. Nenhum campo de descoberta é prova de
identidade imutável ou exclusividade de ferramentas externas à IDE.


### BIN DFU conferido com ELF CMake (0.195.0, 2026-10-26)

`dfu.binaryTarget?: string` escolhe explicitamente BIN quando presente;
nome não vazio/sem espaços externos, executável único de uma configuração do
file-api da raiz/build canônicos em `.kinein/build`. `dfu.image` deve estar
nesse build. Ausente mantém DfuSe; não deduzir formato por extensão.

BIN regular não vazio até 32 MiB; ELF até 64 MiB, ELF32 LE ARM ET_EXEC v1,
1–128 program headers. Bytes file-backed de PT_LOAD em flash interna, sem
sobreposição/overflow, filesz <= memsz e alinhamento coerente; entrada em
segmento executável. BSS sem bytes não é gravado. BIN cobre exatamente os
segmentos por p_paddr, com gaps somente zero, sem padding extra. Prefixos
TI/LPC/DfuSe ou assinatura de sufixo UFD recusados: dfu-util deve transferir
os mesmos bytes conferidos. Unsupported/ambíguo recusa antes da execução.

Endereço deriva do primeiro segmento; argv inclui `--dfuse-address
0xENDERECO`, sem modificadores. Intervalo inteiro validado no mapa da n2.
Prévia mostra alvo/id/ELF/endereço/tamanho e hashes. Reply CMake, ELF e BIN
integram o snapshot/fingerprint, revalidado na execução e antes do PTY.
Seletores USB, reservas e recusas de campos externos da n2 preservados.
Não executa build/objcopy nem afirma atualização com as fontes; preserva a
escolha explícita e os limites de compatibilidade física da n2.

### picotool nativo — UF2 conferido com ELF CMake (0.197.0, 2026-10-27)

`pico?: {image, destination:{chip,serialSelector,flashSizeBytes}}` exige
native=true, motor picotool e target CMake explícito. Recusa device, probe,
dfu, firmware, buildDirectory e flashSizeBytes do nível superior; outros
motores/modo manual recusam pico. UF2 regular até 32 MiB dentro de `.kinein/build`,
ELF até 64 MiB do executável único confirmado no file-api desse workspace.
Aliases de UF2 para fora do workspace são recusados.

ELF32 LE ARM ET_EXEC, até 128 headers, entrada executável, sem dinâmica,
overflow/sobreposição; bytes carregáveis em 0x10000000–0x11000000. UF2 de 512
bytes/bloco, payload 256, flags 0x2000, família 0xe48bff56, índices/contagem
exatos e endereços ordenados/alinhados. Magics e cauda zero conferidos; páginas
iguais ao ELF com padding zero segundo elf2uf2 2.2.0/2.3.1, sem páginas extras.
Outras famílias, flags, RAM, arquivo parcial e origem ambígua recusados.

Destino RP2040 2e8a:0003 já em BOOTSEL; --ser confirmado com um único mesmo
chip/ID/capacidade e versão 2.2.0/2.3.1. Reserva USB por família
`picotool:2e8a:0003`, pois o upstream faz claim antes de comparar flash ID;
serial/sondas de outros recursos permanecem livres. Consultas protegidas pelo
guardião, 5 s/64 KiB conjuntos. Conflito usa SERIAL_PORT_BUSY, scope=usb.

Argv: load --verify --family 0xe48bff56 IMAGE --type uf2 --vid 0x2e8a --pid
0x0003 --ser ID. Apaga setores de 4 KiB, pode zerar bytes fora das páginas UF2
no mesmo setor, verifica os dados e não inicia a aplicação. Fim do setor deve
caber na capacidade confirmada. Sem force, reboot, offset, update ou OTP.
Intenção versão 1/schema 2 e PTY existentes; snapshot inclui CMake/ELF/UF2,
ferramenta, destino/versão e contexto, revalidados em run.start e antes do PTY.
Não compila/converte na prévia, não comprova atualização das fontes ou
compatibilidade física. Desenho/fontes: [52 §14.55](../../roadmaps/52-arquitetura-executavel-da-0.4.md#1455-041h4b2b4o2--uf2build-e-picotool-nativo-desenho-2026-10-27).

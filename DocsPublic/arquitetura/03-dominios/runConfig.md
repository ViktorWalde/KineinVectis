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

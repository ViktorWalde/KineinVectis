# `runConfig.*` — o domínio no protocolo IPC

> Parte do [03](../03-protocolo-ipc.md), que tem o formato base, as listas
> canônicas de métodos e eventos e as regras. Movido sem mudar o texto em
> 2026-10-16 ([61](../../roadmaps/61-organizacao-da-documentacao.md) §3.7, O6).

### Run configurations (`runConfig.list` / `runConfig.save` / `runConfig.delete` / `runConfig.setActive` / `runConfig.flashProposal`)

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
  Run do CLion): **puro** — nada roda, nada é salvo; a resposta é uma
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
  salva é editável como qualquer configuração — é assim que um `esptool` v4
  troca `write-flash` por `write_flash`.
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

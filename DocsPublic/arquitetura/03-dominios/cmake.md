# `cmake.*` — o domínio no protocolo IPC

> Parte do [03](../03-protocolo-ipc.md), que tem o formato base, as listas
> canônicas de métodos e eventos e as regras. Movido sem mudar o texto em
> 2026-10-16 ([61](../../roadmaps/61-organizacao-da-documentacao.md) §3.7, O6).

### CMake service (`cmake.configure` / `cmake.presets.list` / `cmake.targets.list` / `cmake.status`)

Implementado no protocolo `0.25.0` (fatia M2.2). Todos exigem
workspace aberto com kind `cmake` (`INVALID_PARAMS` para outros kinds). O
diretório de build é único e imutável: `<root>/.kinein/build` — o mesmo de
`build.run` e `run.start`; presets **não** mudam o diretório (o `-B`
explícito tem precedência).

- `cmake.configure { preset? }` → `{ jobId }` (job cancelável). Escreve a
  query `codemodel-v2` do file-api antes de rodar e sempre passa
  `-DCMAKE_EXPORT_COMPILE_COMMANDS=ON`. Eventos: `event.cmake.started
  { jobId, command, preset?, presetSource? }` e `event.cmake.finished { jobId,
  success, exitCode, hasCompileCommands }`; saída linha a linha vai por
  `event.job.output`. **O preset (`0.115.0`, P0):** o do pedido (a ponte C++
  manda o do kit ativo — o último `toolchain.get` que a tela pediu;
  `presetSource: "kit"`, e as ESCOLHAS de ferramenta são as desse kit) ou o
  **padrão do projeto** — `cmake::default_preset`: o primeiro
  `configurePresets` não `hidden` de `CMakeUserPresets.json` (a escolha do
  usuário para esta máquina vence), senão de `CMakePresets.json`, pulando o
  que uma `condition` (`const false`, ou `equals`/`notEquals` sobre
  `${hostSystemName}`) exclui no Linux; `presetSource` é o arquivo; as
  escolhas de ferramenta são as do kit padrão. Sem presets, configure sem
  preset como sempre e `presetSource` ausente. Antes, o configure automático
  de um projeto com presets rodava SEM preset. No sucesso o core anota o
  preset em `<buildDir>/.kinein-preset` (o `CMakeCache.txt` não o guarda) e
  o `cmake.status` o devolve. **Desde o `0.174.0` (a 0.4.0b, `roadmaps/52`
  §14.2), sem preset pedido a ordem é:** o anotado no build dir, se ainda
  existe entre os presets do projeto (`presetSource: "buildDir"`) → o ÚNICO
  preset que vale do primeiro arquivo que tem algum → **vários: o configure é
  recusado com `INVALID_PARAMS`**, dizendo o arquivo e os nomes, e nenhum job
  roda. O "primeiro da lista" do P0 deixou de ser escolhido pela pessoa.
- `cmake.presets.list {}` → `{ presets: [{ name, displayName? }] }` —
  configure presets não ocultos de `CMakePresets.json` +
  `CMakeUserPresets.json`, na ordem dos arquivos; JSON inválido → erro
  humano.
- `cmake.targets.list {}` → `{ origin, targets: [{ name, kind, artifacts[],
  sources, generatedSources, languages[], standard?, includes, defines,
  sysroot?, dependencies[], sourceDir? }] }` — lidos da resposta codemodel-v2
  do file-api do último configure (`kind`: `executable`, `staticLibrary`,
  ...; utilitários ficam de fora); vazio antes do primeiro configure. **Desde
  o `0.96.0` (2026-09-12) cada target carrega o seu MODELO** (`cmake/model.rs`):
  `artifacts` absolutos ao build dir (o ELF que o P2 grava), `sources` do
  autor e `generatedSources` (moc/rcc) separados, `languages` na grafia do
  file-api (`C`, `CXX`), `standard` do `languageStandard` do primeiro grupo,
  `includes`/`defines` distintos entre os grupos, `sysroot` quando há
  `CMAKE_SYSROOT`, `dependencies` com os ids resolvidos para nome (um alvo
  importado, fora do codemodel, não entra). Medido neste repositório:
  `kinein-vectis` com 301 fontes + 521 geradas, `CXX` 23, 12 includes, 8
  defines, artefato `.kinein/build/ui/kinein-vectis`. A query passou a pedir
  também a `toolchains-v1` (CMake ≥ 3.20, cmake-file-api(7)): compilador,
  id, versão e includes implícitos por linguagem — vale a partir do próximo
  configure.
- `cmake.status {}` → `{ configured, hasCompileCommands, buildDir, preset? (0.115.0),
  cdbDirectory?, cdbStale?, cdbStaleBecause?, presetChoices[]?,
  presetChoicesFile? (0.174.0) }` — `presetChoices` são os nomes quando um
  configure sem preset seria recusado por haver vários, e a UI pergunta
  pela faixa de saúde em vez de configurar sozinha; stat de
  `CMakeCache.txt`/`compile_commands.json`, mais o **diagnóstico da compilation
  database** (protocolo `0.62.0`).
- **`hasCompileCommands` e `cdbDirectory` não são a mesma pergunta**, e confundi-
  las é a origem de "erro de include sem causa":

  ```text
  hasCompileCommands   existe CDB no build dir DA IDE (.kinein/build)?
  cdbDirectory         existe CDB que o clangd ALCANCA, e onde? Relativa ao
                       root; "." e' a propria raiz. AUSENTE quando nao ha.
  ```

  O clangd procura `compile_commands.json` **nos diretórios pai e em
  subdiretórios `build/`** por conta própria
  (<https://clangd.llvm.org/installation>), então um projeto **Meson ou `bear`
  funciona com `hasCompileCommands: false`**. Só `cdbDirectory` responde se o
  usuário tem inteligência de código ou não.
- `cdbStale` + `cdbStaleBecause`: a CDB alcançável é mais velha que um arquivo
  de build que a define (`CMakeLists.txt`, `CMakePresets.json`, `meson.build`,
  `Makefile`). O clangd então usa flags de um projeto que mudou. Os três campos
  são **omitidos quando não há o que reportar** — projeto sadio não carrega
  ruído, e a UI não precisa distinguir `false` de ausente.
- clangd: quando `compile_commands.json` existe no build dir da IDE, novos
  servidores cpp sobem com `--compile-commands-dir` apontando para ele — o
  `.kinein/build/` **não** é `$SRC/build/`, então o clangd não o acharia sozinho.
- **Recarga de flags, corrigido em 2026-08-30.** O clangd **tem** hot-reload da
  `compile_commands.json` desde a v12 (reconfere a cada ~5 s,
  [D92663](https://reviews.llvm.org/D92663)). O que não atualiza é o **documento
  já aberto**, que fica com a compilação em cache
  ([vscode-clangd #42](https://github.com/clangd/vscode-clangd/issues/42)).
  Logo a ação certa após um configure é **reabrir os documentos abertos**, e não
  reiniciar o servidor — reiniciar joga o índice fora.


**Limite de leitura do file-api (0.195.0, 2026-10-26).** O modelo lê somente
arquivos regulares, sem seguir links na abertura, com até 16 MiB por JSON.
FIFO, arquivo ilegível/excessivo ou conteúdo inválido não confirmam alvos.
A origem nativa exige o modelo válido; não cai no parser de CMakeLists.

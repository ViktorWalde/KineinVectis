# `configAction.*` — o domínio no protocolo IPC

> Parte do [03](../03-protocolo-ipc.md), que tem o formato base, as listas
> canônicas de métodos e eventos e as regras. Movido sem mudar o texto em
> 2026-10-16 ([61](../../roadmaps/61-organizacao-da-documentacao.md) §3.7, O6).

### Configuration Actions (`configAction.list` / `configAction.preview` / `configAction.apply`)

**Instalação do compilador C (`0.204.0`, 2026-11-08).** A ação
`cmake.selectCCompiler`, título "Escolher a instalação do compilador C", escopo
`cmake`, categoria "CMake Basic", risco `low` e efeito `edit`, afeta somente
`.kinein/toolchain.json`. Parâmetros string opcionais: `compiler`, caminho
absoluto de um driver C local (vazio volta ao automático), e `kit`, nome do
preset cujo kit recebe a escolha (vazio usa o kit ativo do domínio `cmake`: o
da tentativa pendente ou o usado pelo último configure; sem configure, o
padrão). As sugestões de `compiler` são
executáveis C encontrados nas pastas do detector, sem lista fechada; as de
`kit`, os configure presets. Prévia e aplicação conferem o executável fora do
laço com a mesma consulta de versão e sonda C da criação, recusando caminho
relativo/inexistente, ferramenta que não é GCC/Clang e driver C++ com
`INVALID_PARAMS`. Se o workspace mudar durante a conferência, a resposta é
recusada. As notas da prévia mostram instalações detectadas com versão,
escolha atual, compilador do último configure, origem sem fixação e
divergências (toolchain file que pode prevalecer, compilador do preset
sobreposto pela linha de comando e cache que será recriado). `apply` usa
`expected`; presets, toolchain files, fontes, flags e `.clangd` não mudam.
Desenho na [consolidação C §15](../../especificacoes/consolidacao-nativa-de-c.md#15-c1-instalação-do-compilador-em-projetos-existentes--desenho-de-2026-11-08).

**Padrão C por alvo (`0.199.0`, 2026-11-03).** A ação
`cmake.setCStandard`, título "Fixar o padrão C do alvo", escopo `cmake`,
categoria "CMake Rigor", risco `medium` e efeito `edit`, afeta somente
`CMakeLists.txt`. Parâmetros string: `target` obrigatório, `standard`
opcional (identificador decimal positivo para `C_STANDARD`; ausência/vazio escolhe `23`) e
`extensions` opcional (`OFF`, `ON`; ausência/vazio escolhe `OFF`). A listagem
oferece sugestões e descrições próprias de C; `standard` em `setCxxStandard`
conserva seus valores C++.

**Desde `0.200.0` (2026-11-04)**, os exemplos `90`, `99`, `11`, `17`, `23`
não limitam os valores numéricos de `standard`. A prévia/aplicação aceitam
valores futuros sem uma nova lista na IDE; configurar/build validam sua
disponibilidade no CMake/toolchain. Blocos exatos da receita com esses valores
também podem ser atualizados. Sintaxe inválida/injeção continua `INVALID_PARAMS`.
Aliases de driver permanecem no projeto/CDB; não se convertem em propriedades
CMake por aproximação. Ler um projeto C não depende desta receita ou sugestões.

A prévia escreve propriedades do alvo: `C_STANDARD`,
`C_STANDARD_REQUIRED ON`, `C_EXTENSIONS`. Não muda variáveis globais/presets
nem outros alvos. O alvo deve ser literal, declarado na raiz, com fonte `.c`;
fluxo dinâmico, scripts externos ou configuração manual conflitante são
`INVALID_PARAMS`, com motivo. O bloco exato gerado pela ação pode ser trocado
por outra seleção; alterações manuais impedem a substituição automática.
`apply` usa a barreira `expected` existente (`FILE_CHANGED` quando o snapshot
mudou). O resultado confirma a edição, sem afirmar que a toolchain aceitou
o padrão: configurar/build e consultar a CDB/contexto continuam necessários.
Sem método, evento ou campo novo nos envelopes. Desenho e limites na
[consolidação C §8](../../especificacoes/consolidacao-nativa-de-c.md#8-configuração-do-padrão-c-por-alvo--desenho-de-2026-11-03).

A UI invalida a prévia ao alterar parâmetros ou receber uma falha. Respostas
de consultas anteriores são descartadas pela identidade da requisição na
ponte; a aplicação usa somente a prévia vigente e seu snapshot.

Implementado no protocolo `0.63.0` (etapa 2 de `DocsPublic/roadmaps/30-caminho-para-o-mvp.md`).
Os três exigem workspace aberto. O escopo de cada ação é o **build system
ativo** do workspace (spec 9.2 §1): projeto Cargo não vê ação `CMake`, e pedir
uma fora do escopo é `INVALID_PARAMS`, não silêncio.

**São três métodos porque o ciclo tem três passos**, e a spec 9.1 §10 exige os
três: *explicar, mostrar preview, aplicar com consentimento e validar*. Um
método só seria um botão que edita o `CMakeLists.txt` do usuário sem mostrar o
quê.

- `configAction.list { includeHiddenByScope? }` →
  `{ actions: [...], activeBuildSystems: [...] }`. Cada ação traz
  `{ id, title, description, scope, category, risk, affects, effect, state,
  reason?, params, docs }`. O `state` é medido no workspace agora
  (`available`, `recommended`, `partiallyAvailable`, `unavailable`,
  `hiddenByScope`) e o `reason` diz por quê — "o preset debug já existe",
  "nenhum target declarado". `includeHiddenByScope: true` traz também as ações
  do build system inativo, marcadas (spec 9.2 §25).
- `configAction.preview { id, params? }` →
  `{ id, title, summary, files: [{ path, before?, after }], report, notes }`.
  **Não escreve nada.** `before` ausente significa que o arquivo será CRIADO.
  `report` é o resultado das ações de leitura (o cache do `CMake`); `notes` são
  avisos que o usuário precisa ler antes de aplicar.
- `configAction.apply { id, params?, expected? }` →
  `{ id, message, files, jobId? }`. O `expected` é a lista
  `[{ path, content? }]` que veio do `before` do preview: é a **mesma barreira
  do `fs.write`** (`ARCHITECTURE.md` §7.1) aplicada ao consentimento — se o
  disco mudou entre o preview e o Apply, a resposta é `FILE_CHANGED` e nada é
  escrito. Lista vazia dispensa a comparação.

O `effect` de cada ação diz o que o `apply` faz, e por isso está no contrato:

```text
edit      reescreve um arquivo de texto; o preview mostra o diff exato
inspect   so le: o preview E o resultado, e o apply nao escreve
job       devolve `jobId` de um job: cargo.check, ou, desde 0.177.0, o
          compiledb do PlatformIO
delegate  efeito nao textual dentro do core (remover build dir, salvar run config)
```

As 16 ações do MVP são as da §12 da spec de fechamento: 10 de `CMake`
(`enableCompileCommands`, `createDebugPreset`, `createReleasePreset`,
`addExecutable`, `addStaticLibrary`, `addSourceToTarget`, `addIncludeDirectory`,
`addTargetLinkLibraries`, `inspectCache`, `repairBuildDir`) e 6 de Cargo
(`addDependency`, `addDevDependency`, `addFeature`, `setEdition`, `check`,
`createRunConfig`). **Em 2026-10-17 são 27**, em uma tabela por escopo
(`configaction/catalog/`): 19 de `CMake` (as 10, mais `findPackage`,
`fetchContent`, `removeTargetLinkLibraries`, as cinco de regime —
`setCxxStandard`, `strictWarnings`, `enableSanitizers`, `enableOpenMP`,
`generateHexBin` — e `exportToolchainFile`), 7 de Cargo (as 6 e
`embeddedTarget`) e 1 de `PlatformIO` (`compiledb`). **Em 2026-10-23, 28:** entra
`espidf.clangdFreeRtos` (escopo `CMake`, categoria ESP-IDF, efeito `edit`).
Ela é recomendada só num projeto ESP-IDF, pela regra do modelo do projeto, e
acrescenta ao `.clangd` (ou cria) um fragmento separado por `---` com
`Diagnostics.Includes.IgnoreHeader: ['freertos/.*']`. Com isso, o
include-cleaner do clangd deixa de sugerir trocar `freertos/FreeRTOS.h`, o que
quebra o build. Fica "já aplicada" quando o arquivo já tem esse
`IgnoreHeader`, e repeti-la é INVALID_PARAMS (52 §14.41). Em 2026-11-03 são 29,
com `cmake.setCStandard`; em 2026-11-08, 30, com `cmake.selectCCompiler`.

**PlatformIO (`0.177.0`, a 0.4.0e do `roadmaps/52` §14.5).** O escopo
`platformio` (o build system `platformIo`) e a ação `platformio.compiledb`,
"Gerar compile_commands.json": efeito `job`, afeta `compile_commands.json`,
risco baixo; `recommended` quando não há CDB alcançável. O `apply` roda
`pio run -t compiledb` na raiz como job, só em projeto confiável (o `pio`
roda os scripts do projeto) e com o `pio` detectado (sem ele, recusa
`ToolNotFound` com o passo, como o `build.run`); no sucesso, o contexto do
índice é relido e o clangd passa a ver a CDB nova, como depois de um
`cmake.configure`: os documentos C/C++ fechados para a UI reabri-los, ou o
servidor reiniciado quando subiu sem CDB ([lsp](lsp.md)).

**O kit exportável (`0.178.0`, a 0.4.1a do `roadmaps/52` §14.7).** A ação
`cmake.exportToolchainFile`, "Exportar o kit como toolchain file": efeito
`edit`, afeta `.kinein/toolchains/<kit>.cmake`, risco baixo. Grava os MESMOS
valores que o configure da IDE passa pela linha (o gerador é um só, no dono
`toolchain`), para o terminal e o CI (`cmake --toolchain <arquivo>`); **a IDE
não lê o arquivo** — o kit continua a fonte, e um toolchain file vence o `-D`
da linha (medido), então lê-lo deixaria o kit mudado perder em silêncio. O
kit é o do configure (o do preset do último configure, se houver, senão o
padrão). `partiallyAvailable` quando o kit não é cross ou já tem toolchain file;
`alreadyApplied` quando o arquivo já diz o mesmo.

O `riskExplanation` do risco `low` diz, desde `0.177.0`, "não edita fonte nem
configuração de build; no máximo gera arquivos que se recriam (o build, a
compile_commands.json, o .kinein)" — o "não altera nenhum arquivo" de antes
era falso para o `compiledb`, e já era frouxo para o `cargo.check` (o
`target/`) e o `createRunConfig` (o `.kinein`).

**O que o core RECUSA em vez de adivinhar**, e é o comportamento certo: nome de
target fora do padrão da CMP0037, target inexistente, fonte com `..`,
dependência que já está no manifest, `Cargo.toml` com string multilinha ou
`dependencies` como tabela inline, `edition.workspace` herdada. Corromper o
manifest do usuário não tem desfazer; a recusa vem com o motivo, na lista e no
diálogo.

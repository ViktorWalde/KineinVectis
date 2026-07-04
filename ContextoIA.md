# ContextoIA - Continuidade do Kernwerk Studio

Este arquivo registra decisoes de produto/arquitetura para IAs que continuarem
o desenvolvimento do repositorio. Use junto de `AGENTS.md` e dos documentos em
`docs/`.

## Direcao do produto

- Produto: Kernwerk Studio.
- Objetivo: IDE open source, Linux-first, rigida por padrao, visualmente
  familiar para usuarios de IDEs JetBrains, mas com identidade propria.
- Uso inicial: projeto de uso proprio do autor, com foco em qualidade alta e
  ergonomia diaria.
- Filosofia: a IDE orquestra ferramentas maduras; nao reimplementa compilador,
  LSP, debugger, build system ou analisadores quando ja houver ferramenta aberta
  consolidada.

## Decisoes recentes de UX/workspace

- O fluxo de projeto/workspace deve seguir a memoria muscular de IDEs JetBrains.
- O seletor de pasta deve ser proprio da IDE, nao o dialogo nativo do desktop.
- A UI nao deve listar o filesystem diretamente. A navegacao do seletor passa
  pelo core via `workspace.browse`.
- A arvore de projeto deve ser limpa, leve e confortavel, mais proxima do
  Project View das IDEs JetBrains do que do explorer pesado/fechado do VS Code.
- O painel Project deve parecer uma arvore de projeto da IDE, nao um gerenciador
  de arquivos. Usar hover/selecao sutis, indentacao clara, poucas bordas
  internas e acoes compactas por icone.
- Clique em diretorio no explorer do projeto deve expandir/recolher, nao trocar
  workspace implicitamente.
- Abrir outro workspace:
  - se for o mesmo root, pode preservar abas abertas;
  - se for outro root, a UI deve limpar abas e estado visual do workspace
    anterior.
- Fechar projeto deve fechar automaticamente abas abertas e limpar explorer.

## Estado tecnico atual

- Arquitetura: Qt/QML UI <-> JSON-RPC local/stdin-stdout <-> Rust core.
- Core Rust:
  - `core.ping`
  - `core.shutdown`
  - `command.list`
  - `tools.detect`
  - `tools.status`
  - `workspace.browse`
  - `workspace.open`
  - `workspace.status`
  - `workspace.close`
  - `fs.list`
  - `fs.read`
  - `fs.createFile`
  - `fs.createDirectory`
  - `fs.write`
  - `fs.rename`
  - `fs.delete`
  - `fs.findFiles`
  - `build.run`
  - `test.run`
  - `quality.run`
  - `lsp.didChange`
  - `lsp.definition`
  - `lsp.hover`
  - `lsp.completion`
  - `lsp.references`
  - `lsp.rename`
  - `fs.search`
  - `run.start`
  - `run.stdin`
  - `run.stop`
  - `terminal.open`
  - `terminal.input`
  - `terminal.close`
  - `lsp.semanticTokens`
- Protocolo IPC atual: `0.19.0`.
- `workspace.browse` lista subdiretorios para o seletor proprio da UI. Ele
  canonicaliza caminhos e retorna `{ path, parent, entries }`.
- `fs.*` continua confinado ao workspace aberto e nao deve ser usado para
  navegar fora do workspace. `fs.createFile` cria arquivo novo e
  `fs.createDirectory` cria diretorio novo, ambos sem sobrescrever caminho
  existente; `fs.write` continua salvando apenas arquivos ja existentes.
  `fs.rename` renomeia/move e `fs.delete` remove (recursivo para diretorios),
  ambos confinados a raiz e recusando a propria raiz do workspace.
- `fs.findFiles` usa `fd` para busca de arquivos por nome, sem indexador
  proprio. Falta de `fd` retorna `TOOL_NOT_FOUND`.
- `build.run` executa a ferramenta de build pelo core e emite
  `event.build.started/output/diagnostic/finished`.
- `lsp.didChange` sincroniza o buffer aberto com o LSP gerenciado pelo core.
  Eventos `event.lsp.status` e `event.lsp.diagnostics` alimentam a UI.
- `lsp.definition` e `lsp.hover` fazem navegacao semantica minima via LSP,
  sempre passando pelo core e usando posicao 1-based do editor.
- A UI sobe `kernwerk-core` como processo filho via `CoreClient`.
- O usuario testa normalmente pelo icone "Kernwerk Studio" ja instalado no
  menu de aplicativos. Esse icone executa `scripts/kernwerk-studio`.
- O launcher `scripts/kernwerk-studio` prefere:
  - UI release: `build/linux-clang-release-hardened/ui/kernwerk-studio`;
  - core release: `target/release/kernwerk-core`;
  - e so cai para debug se os binarios release nao existirem.
- Depois de alterar core/UI, atualizar os binarios usados pelo icone com:
  `cargo build --release -p kernwerk-core` e
  `cmake --build --preset dev-local-release`.
- CORRECAO (2026-07-03): nesta maquina (Pop!_OS), o CONFIGURE do CMake deve
  usar os presets locais `dev-local` / `dev-local-release` de
  `CMakeUserPresets.json` (fora do git). Eles herdam os presets oficiais e
  apontam o clang para `--gcc-install-dir=.../gcc/x86_64-linux-gnu/13`,
  porque o diretorio GCC 14 do sistema esta incompleto (sem libstdc++) e
  quebra o link. Em outras maquinas, usar os presets oficiais.

## Strict mode

- Rust deve continuar com o maximo rigor:
  - `unsafe_code = "forbid"`;
  - warnings como erro;
  - docs/debug impls obrigatorios onde configurado;
  - clippy pedantic/nursery;
  - sem `unwrap`, `expect`, `panic`, `todo`, `dbg!` fora de casos aceitos por
    testes existentes.
- C++/Qt usa C++23, warnings-as-errors, sanitizers em Debug e hardening/LTO em
  Release via `cmake/KernwerkStrictOptions.cmake`.
- Futuramente deve existir seletor de nivel de rigidez:
  - Strict como padrao;
  - Balanced;
  - Relaxed apenas por escolha explicita.

## Prioridade imediata

Estabilizar UX/UI basica antes de avancar para features grandes:

1. Fluxo de abrir/fechar workspace. [feito]
2. Explorer previsivel (arvore expande/recolhe). [feito]
3. Abas e salvamento. [feito]
4. Layout arredondado, denso e tecnico inspirado no mockup em
   `imagens/layout-mockup.png`. [feito]
5. Top bar, sidebar de icones, paineis inferior e direito. [em andamento]
   - Sidebar de icones (esquerda): Projeto (mostra/oculta explorer),
     Ferramentas e Logs (abrem o painel inferior). [feito 2026-07-03]
   - Painel inferior com abas "Logs" e "Ferramentas". A aba Ferramentas usa
     `tools.detect` (status, versao e sugestao pacman por ferramenta) com
     botao de redeteccao. [feito 2026-07-03]
   - Painel direito (Assistente KW): casca visual pronta (icone ✦ na sidebar,
     header com badge "offline", historico de mensagens e input). SEM provider
     de IA ligado; resposta e um aviso honesto sobre a Fase 7 e a politica de
     confirmacao. Nao fingir IA funcionando. [feito 2026-07-03]
6. Polir a arvore Project para a sensacao limpa/confortavel JetBrains-like.
   - Reduzir peso visual do painel e das linhas.
   - Evitar visual de explorer pesado/fechado estilo VS Code.
   - Manter diretorio como expandir/recolher, nao trocar workspace.
   - Preferir icones pequenos e tooltips futuros para acoes.

## DECISAO DE PRIORIDADE (usuario, 2026-07-03)

O usuario decidiu: UI/UX fica no nivel "basico/funcional" atual; a prioridade
agora e integrar ferramentas para usar a IDE como ambiente de desenvolvimento
real. Ordem acordada:

1. Syntax highlighting no editor. [FEITO 2026-07-03]
2. Fase 4 - Build. [FEITO 2026-07-03 — ver sessao abaixo; cancelamento e
   run/execucao do binario ainda pendentes]
3. Fase 5 - LSP: diagnosticos MVP, go to definition e hover implementados
   para clangd e rust-analyzer. O core sobe servidores como processos
   gerenciados e traduz LSP para IPC. Nesta sessao `clangd` foi encontrado em
   `/usr/bin/clangd`; ainda nao assumir que `rust-analyzer` esta instalado.
   Completion (Ctrl+Space), find usages (Alt+F7) e rename (Shift+F6)
   entregues na Fase 5.2 [FEITO 2026-07-03]; semantic tokens e code actions
   continuam pendentes. [MVP EM ANDAMENTO]
4. Git (Fase 6) na sequencia. A Fase 5 passa a ser tratada como base LSP
   suficiente para C/C++ e Rust, sem tentar fechar todos os recursos avançados
   de IDE profissional antes de avançar.
5. Decisao atualizada (2026-07-04): reformular Fases 7-9 para foco de curto
   prazo em C/C++ e Rust. Java/Python saem do curto prazo e ficam para pos-V1
   ou retomada futura. IA nao deve virar sistema complexo de providers agora:
   o usuario consegue usar Claude/Codex/GPT pelo terminal; a necessidade da
   IDE e uma aba/terminal visualmente separado para chat de IA, separado do
   terminal geral usado para comandos do projeto.
6. Decisao de produto pos-V1 (2026-07-04): adicionar no futuro "modos de
   compilador" e uma "loja de funcoes" para C/C++ e Rust. Isso deve permitir
   ativar perfis mais rigidos ou menos rigidos de compilador/quality sem
   decorar flags. A loja deve ordenar funcoes por confianca: ISO/Standard
   primeiro em C/C++; Rust oficial primeiro em Rust; depois diagnosticos
   oficiais, ferramentas maduras, presets Kernwerk, regras locais e opcoes
   experimentais. A experiencia deve ser visual e JetBrains-like: janela de
   opcoes do ambiente do projeto com nome da funcao, explicacao simples,
   impacto, risco, fonte, previa de alteracoes e reversao. Nao bloquear a
   V1.0 com isso. Ver
   `docs/16-compiler-modes-and-function-store.md`.

## Visao pos-V1.0 (usuario, 2026-07-03)

Depois da V1.0 o foco e polir a IDE continuamente para chegar o mais perto
possivel das IDEs JetBrains em analise, navegacao e refatoracao — mesmo
orquestrando ferramentas abertas (CMake, LSPs etc.). O objetivo explicito e
NAO terminar como "um VS Code": a régua de qualidade de navegacao/refactoring
e JetBrains. Isso reforca a Fase 5 (LSP) como investimento central: semantic
tokens, go-to-definition, find usages, rename via LSP, e futuramente acoes de
refatoracao proprias por cima do que os LSPs oferecem.

## Sessao 2026-07-04 (Opus, rename/delete de arquivos + gate no agente)

- ENTREGA: rename/delete/move de arquivos no explorer (a "PROXIMA ETAPA"
  acordada na sessao de quality abaixo). PROTOCOLO `0.19.0`:
  `fs.rename { from, to }` → `{ from, to }` e `fs.delete { path }` → `{ path }`.
  Ambos confinados a raiz (canonicalizam, `INVALID_PARAMS` se escaparem) e
  recusam a propria raiz do workspace (`FsError::WorkspaceRoot`). `fs.rename`
  cobre mover (o `to` pode ter outro diretorio pai, que precisa existir e nao
  pode ja existir); `fs.delete` remove recursivo para diretorios.
- CORE (reuso, sem duplicar): `fsops::rename`/`fsops::delete` reusam
  `confine`/`new_child_path` existentes. Roteados em `fs_request_response`
  (`fs.delete` reusa `FsPathParams`); descriptors `fs.rename`/`fs.delete`
  (context-menu, sem atalho global). CLI: `fs rename <from> <to>` e
  `fs delete <path>`.
- UI: `CoreClient.renamePath/deletePath` + sinais `pathRenamed/pathDeleted`.
  `Main.qml`: menu de contexto (botao direito) no Project panel com Renomear/
  Excluir, dialogo de rename e dialogo de confirmacao de exclusao (overlays
  top-level, mesmo estilo do createDialog). Abas abertas sao reapontadas no
  rename (inclusive quando a pasta pai e renomeada, via prefixo) e fechadas no
  delete (inclusive arquivos sob a pasta removida). Erros voltam para o proprio
  dialogo via `onRequestFailed`. Nomes `entry*` para nao colidir com o rename
  de simbolo LSP (`renameDialogVisible`/`renameError`, Shift+F6).
- DIVIDA TECNICA PAGA NO CAMINHO: (a) corrigido o `clang-format` que quebrava
  `verificar-cpp.sh` em `core_client.cpp` (bloco `event.quality.output/finished`
  do quality) — era so estilo, o build/ninja compilava. (b) `dispatchResult`
  passou de complexidade cognitiva 26 (limite 25, exposto assim que o
  clang-format parou de falhar antes do tidy): extrai `dispatchLspResult`
  espelhando o `dispatchFileResult` ja existente. NAO criar outro dispatcher.
- MUDANCA DE PROCESSO (instrucao do usuario, 2026-07-04): a sessao de
  DESENVOLVIMENTO agora roda o gate completo E recompila os binarios release do
  icone ela mesma — nao deixar mais "PENDENTE PARA O USUARIO". Rodei a sequencia
  inteira encadeada com `&&` (para no primeiro erro), que e exatamente a direcao
  de "gate unico" do `docs/15`. Com isso, a pendencia de rebuild da sessao de
  quality abaixo ficou resolvida nesta sessao.
- VERIFICADO (gate unico, ALL-GREEN): `cargo fmt --all` (aplicado por ultimo)
  + fmt --check limpos; testes 14 CLI / 1 config / 111 core / 21 protocolo
  (novos: fsops rename/delete x8, dispatch fs.rename/delete, protocolo x2, CLI
  x3); clippy limpo; `verificar-cpp.sh` limpo; build debug (sanitized) + release
  core + release UI; smoke offscreen sem erro de QML/ASan; E2E via pipe no core
  release: rename e delete reais funcionaram e `fs.delete /etc/hostname` foi
  recusado como fora do workspace. `core.ping` responde `0.19.0`.
- PROXIMA ETAPA (decisao do usuario): antes de novas features, fazer a
  reorganizacao de engenharia do `docs/15` (reduzir `lib.rs`/`lsp.rs`/
  `CoreClient`/`Main.qml`, separar roteamento/handlers/serviços por dominio,
  transformar a verificacao no gate unico oficial, enxugar docs). `docs/16`
  (modos de compilador + loja de funcoes) e visao pos-V1, nao entra agora.

## Sessao 2026-07-04 (Opus, quality/lint na aba Problemas)

- ORDEM ACORDADA COM O USUARIO: quality primeiro, depois rename/delete de
  arquivos. Ele quer a IDE funcional e profissional o quanto antes.
- PROTOCOLO `0.18.0`: `quality.run {}` → `QualityRunResult
  { success, exitCode?, diagnostics }`. Eventos
  `event.quality.started/output/diagnostic/finished` (mesmos formatos dos
  `event.build.*`).
- CORE (reuso maximo, sem parser novo): `build::run_quality` roda
  `cargo clippy --all-targets --message-format=json` — JSON identico ao do
  `cargo build`, entao passa pelo `stream_command`/`parse_cargo_json_line`
  existentes. Extrai `project_kind_name` (antes triplicado em build/test)
  para um helper pub(crate); test.rs tambem passou a usa-lo. `quality.run`
  so suporta RustCargo por ora; Cmake/clang-tidy e o proximo passo
  (retorna INVALID_REQUEST com mensagem). Wiring `quality_run_response`
  espelha `build_run_response` mas mapeia para `event.quality.*`. Descriptor
  Ctrl+Shift+L; `kernwerk-cli quality`.
- UI: `CoreClient.runQuality()` + propriedade `analyzing` + sinais
  `qualityStarted/qualityDiagnostic/qualityFinished`. `Main.qml`: botao
  "Análise" no header (ao lado de Testes), Ctrl+Shift+L, diagnosticos vao
  para a aba PROBLEMAS existente com `source:"quality"` (limpos a cada run,
  distintos de build/lsp; clicaveis via openDiagnostic). Empty-state dos
  Problemas atualizado. clearWorkspaceUiState limpa origem quality.
- VERIFICADO: kw-fmt/clippy limpos, 132 testes verdes (novo
  `run_quality_rejects_kinds_without_linter`). E2E via pipe (binario DEBUG,
  nao release): projeto com `items.len() == 0` gerou event.quality.diagnostic
  warning "length comparison to zero" em src/main.rs:3, finished
  diagnostics:2 success:true. Build UI debug (sanitized) compilou/linkou e
  smoke offscreen subiu sem erro de QML nem ASan com o botao Análise.
- PENDENTE PARA O USUARIO: recompilar binarios release (core+UI) pelo
  COMANDOS_BUILD_VERIFICACAO.md. So depois o icone tera o botao Análise.
- PROXIMA ETAPA (acordada): rename/delete/move de arquivos no explorer — o
  "e afins" que o usuario pediu e ainda falta (hoje so tem create).
  Provavel `fs.rename`/`fs.delete` no core (confinados ao workspace, como
  os outros fs.*), menu de contexto no Project panel, e fechar/atualizar
  abas abertas do arquivo afetado. Depois: quality para C++ (clang-tidy),
  format (rustfmt/clang-format) e debug (DAP/GDB/LLDB). Python/Ruff saiu do
  curto prazo pela decisao de 2026-07-04 de focar C/C++ e Rust. SQLite
  pos-V1.0.

## Sessao 2026-07-04 (Fable/Opus, retomada apos Codex: Test Runner)

- CONTEXTO DA RETOMADA: o usuario passou do Fable para o Codex quando os
  tokens acabaram e depois voltou. Ao retomar, auditei o que o Codex fez
  (criacao de arquivo/pasta 0.15.0, `fs.findFiles`+Search Everywhere 0.16.0,
  indentacao do editor, deteccao de rg/fd) — clippy limpo e 127 testes
  verdes; binarios release estavam atualizados. Estado saudavel.
- DIRECAO SEGUIDA (registrada pelo Codex): foco em integracao real das
  tecnologias dos .md — build/debug/**test**/quality. Build e run ja
  existiam; a proxima lacuna de maior valor era o runner de testes.
- REFATORACAO ANTI-DUPLICACAO: extrai o plumbing de streaming (threads +
  mpsc + wait) do `build.rs` para novo modulo `process.rs`
  (`stream_command_lines`, com `ProcessError::Spawn/Wait` preservando a
  distincao "ferramenta ausente" vs erro interno). `build.rs` passou a
  usa-lo sem mudar comportamento (seus testes continuam verdes). NAO criar
  outro executor de processo streaming: build e test compartilham este.
- PROTOCOLO `0.17.0`: `test.run { filter? }` → `TestRunResult
  { success, exitCode?, passed, failed, ignored }`. Eventos
  `event.test.started/output/case/finished`; `event.test.case`
  = `{ name, status: passed|failed|ignored }`.
- CORE: novo modulo `test.rs` (`run_tests`): RustCargo → `cargo test`
  (+filtro posicional); Cmake → `ctest --test-dir .kernwerk/build
  --output-on-failure` (+`-R`). Parser de casos a partir da saida do runner:
  libtest `test <nome> ... ok|FAILED|ignored` (a linha `test result:` e
  ignorada porque nao tem ` ... `) e ctest `Test #N: <nome> ... Passed|
  ***Failed`. NAO reimplementa framework — orquestra os runners maduros.
  Wiring em `handle_request_streaming` espelhando `build.run`; descriptor
  `test.run` no command.list (Ctrl+Shift+F9). CLI: `kernwerk-cli test [f]`.
- UI: `CoreClient.runTests(filter)` + propriedade `testing` + sinais
  `testStarted/testOutput/testCase/testFinished`. `Main.qml`: botao "Testes"
  no header (entre Compilar e Iniciar), atalho Ctrl+Shift+F9, aba "Testes" no
  painel inferior com um ponto verde/vermelho/cinza por caso + linha de
  resumo (passou/falhou/ignorado). Estado limpo em clearWorkspaceUiState.
- VERIFICADO: kw-clippy limpo, 131 testes verdes (process.rs 2, test.rs 3
  novos). E2E via pipe: `test.run {filter:"cargo_case_lines"}` emitiu
  event.test.case com nome completo e status, finished passed:1. Build UI
  debug (sanitized) compilou/linkou e smoke offscreen subiu sem erro de QML
  nem relatorio ASan/UBSan com a aba/botao de Testes.
- CORRECOES QUE A VERIFICACAO DO USUARIO REVELOU (ele rodou o
  COMANDOS_BUILD_VERIFICACAO.md): (a) `cargo fmt --all --check` acusou drift
  em build/lib/process/test.rs — eu tinha rodado kw-clippy/kw-test mas nao
  kw-fmt depois das ultimas edicoes; resolvido com `cargo fmt --all`.
  LICAO: sempre rodar kw-fmt por ultimo. (b) clippy nursery
  `option_if_let_else` no CLI (`test` subcommand) — troquei o match por
  `args.get(1).map_or_else(...)`. clang-tidy do C++ estava limpo.
- BUG REAL CORRIGIDO (crash no fechamento): ao rodar `./scripts/kernwerk-
  studio` o usuario viu `QProcess: Destroyed while process is still running`
  + `corrupted double-linked list / Aborted`. Causa: `CoreClient` NAO tinha
  destrutor nem shutdown — o QProcess morria com o core ainda vivo. Adicionei
  `~CoreClient()`: desconecta sinais, fecha stdin (core sai no EOF do loop),
  `waitForFinished(2s)`, com fallback terminate()/kill(). Bug era
  pre-existente (nao do test runner). PRECISA REBUILD DA UI para valer.
- PENDENTE PARA O USUARIO RODAR: recompilar os binarios release do icone
  (core+UI) pelo doc de comandos — mudei CLI (fmt/clippy) e o CoreClient
  (destrutor). Depois, testar fechar a IDE: nao deve mais dar core dump.
- PROXIMOS PASSOS na direcao build/debug/test/quality: (1) quality — rodar
  clippy/clang-tidy/ruff e jogar na aba Problemas; (2) format — rustfmt/
  clang-format no arquivo atual; (3) debug — DAP/GDB/LLDB (mais complexo);
  (4) file rename/delete/move no explorer ("e afins" que o usuario pediu e
  ainda falta). SQLite (indice) fica pos-V1.0.

## Sessao 2026-07-04 (IntelliSense: completion automatico + cores semanticas)

- FEEDBACK DO USUARIO: ao programar na IDE "nao teve o intellisense
  ajudando" e faltam "cores para auxiliar, para saber o que e ou nao uma
  variavel". Tambem pediu para remover o placeholder confuso do input do
  Terminal ("digite no seu shell e pressione Enter") — removido.
- COMPLETION AUTOMATICO (Main.qml): novo Timer `completionDebounce` (250ms)
  disparado em `onTextChanged` quando o popup NAO esta visivel; se o
  caractere antes do cursor e identificador, `.` ou `:`, chama o
  `requestCompletion()` existente. Com popup aberto, o refiltro por prefixo
  continua como antes. Ctrl+Space segue funcionando manualmente.
- PROTOCOLO `0.14.0`: `lsp.semanticTokens { path, content }` →
  `{ tokens: [{ line (1-based), start, length (UTF-16 0-based), kind }] }`.
  UTF-16 e proposital: coincide com indices de QString, o highlighter aplica
  sem conversao.
- CORE (lsp.rs):
  - Capability `textDocument.semanticTokens` declarada no initialize
    (requests.full, tokenTypes padrao, formats [relative]).
  - `wait_for_initialize` agora extrai a legend
    (`capabilities.semanticTokensProvider.legend.tokenTypes`) e guarda em
    `ServerHandle.semantic_token_types`.
  - `semantic_tokens()`: sincroniza o documento, chama
    `textDocument/semanticTokens/full` e decodifica os grupos de 5 deltas
    (`decode_semantic_tokens`, com testes: deltas de linha/coluna, kinds
    fora da legend sao pulados mas ainda avancam a posicao).
- UI:
  - `EditorHighlighter.setSemanticTokens()/clearSemanticTokens()`: spans por
    linha aplicados APOS as regras regex em `highlightBlock` (semantico
    vence). Cores novas em editor_highlighter.cpp: tipos/classes teal
    #5fb3ac, funcoes/metodos #d8a657, variaveis #cdd6e4 (parametro em
    italico), campos/properties #b48ead; macro/keyword/comment/string/number
    reusam a paleta existente. Trocar de arquivo limpa os spans.
  - `CoreClient.requestSemanticTokens()` + sinal; resposta fora do log IDE
    (como lsp.didChange) para nao poluir.
  - Requests: apos o debounce de didChange (600ms), ao carregar arquivo e ao
    trocar de aba. Tokens ficam ~1 ciclo desatualizados durante digitacao
    rapida (drift breve aceitavel; reconcilia no proximo debounce).
- DECISAO DO USUARIO (2026-07-04): SQLite para indexacao/cache e viavel
  APOS a V1.0 — o plano dele e abrir projetos medios/grandes e desenvolver a
  propria IDE nela (dogfooding pesado). Registrar como direcao pos-V1.0
  (indice persistente de simbolos/busca), NAO implementar agora.
- VERIFICACAO: fmt/clippy limpos, 119 testes Rust verdes, verificar-cpp
  limpo, builds debug+release, smoke offscreen, E2E via pipe com clangd real
  (14 tokens em main.cpp: function/parameter/variable/operator com posicoes
  corretas). Binarios do icone atualizados.
- Anti-duplicacao: completion automatico reusa `requestCompletion`/popup
  existentes; semantic tokens reusam `send_request`/`sync_document` do
  LspManager e o `EditorHighlighter` existente (NAO criar outro highlighter
  nem outro caminho de request LSP).

## Sessao 2026-07-04 (continuidade: criacao de arquivos pelo core)

- O pedido de continuidade mencionava que "ainda falta autocomplete"; ao
  reler o projeto, completion automatico, Ctrl+Space, semantic tokens,
  find usages e rename ja estavam implementados. Para evitar duplicacao, a
  proxima lacuna escolhida foi criacao de arquivos/pastas e afins.
- PROTOCOLO `0.15.0`: `fs.createFile { path, content? }` →
  `{ path, bytesWritten }`; `fs.createDirectory { path }` → `{ path }`.
- CORE: `fsops::create_file` confina o diretorio pai ao workspace, exige pai
  existente e falha se o arquivo ja existir. `fsops::create_directory` segue
  a mesma regra para diretorios. `fs.write` permanece restrito a arquivo
  existente.
- UI/CoreClient: expostos `createFile(path, content)`/`fileCreated` e
  `createDirectory(path)`/`directoryCreated`.
- UI/Project panel: header ganhou acoes compactas para novo arquivo e nova
  pasta. O alvo e a pasta selecionada, o pai do arquivo selecionado ou a raiz
  do workspace quando nao ha selecao. O dialogo valida nome vazio/barra, chama
  apenas o core, recarrega o diretorio pai e abre o arquivo recem-criado.

## Sessao 2026-07-04 (Search Everywhere inicial com fd)

- Direcao do usuario: focar no fluxo de IDE JetBrains-like e nas ferramentas
  abertas citadas nos docs, tratando a comparacao com Neovim apenas como
  exemplo de "super ferramenta" e nao como objetivo visual.
- PROTOCOLO `0.16.0`: `fs.findFiles { query }` →
  `{ matches: [{ path, name }], truncated }`.
- CORE: `fs.findFiles` usa `fd` (ou `fdfind` quando esse for o nome
  disponivel na distro) com `--type f --fixed-strings --hidden --color never
  --strip-cwd-prefix`, respeita ignores do projeto e exclui `.git`,
  `.kernwerk`, `.idea`, `.cache`, `target`, `build`, `node_modules`. Limite:
  100 arquivos. `fd`/`fdfind` ausente retorna `TOOL_NOT_FOUND`.
- Tool detection agora cobre `ripgrep` (`rg`) e `fd`/`fdfind`, alem da lista
  anterior.
- UI: Search Everywhere inicial abre por `Ctrl+Shift+N` e `Ctrl+Shift+A`,
  busca arquivos pelo core com debounce, permite setas/Enter/mouse e abre o
  arquivo selecionado.

## Sessao 2026-07-04 (Search Everywhere com comandos + direcao de toolchains)

- UI: Search Everywhere agora mistura comandos de `command.list` com arquivos
  vindos de `fs.findFiles`. Os comandos sao carregados apos conexao com o core,
  filtrados no QML e executam acoes ja existentes quando ha mapeamento local.
  Ainda nao foi criado um protocolo generico de `command.execute`.
- Fontes ainda pendentes para o Search Everywhere: simbolos via LSP, targets
  via CMake/Cargo, settings e recentes.
- Decisao do usuario para a proxima etapa: depois desta etapa, pode emendar o
  desenvolvimento de um icone/entrada propria para abrir a fase de ferramentas.
  O foco principal passa a ser integracao real das tecnologias mencionadas nos
  `.md` (toolchains, LSPs, build/debug/test/quality etc.) e so depois lapidar a
  UI em ordem. A UI deve expor e organizar essas integracoes, mas nao substituir
  nem reimplementar ferramentas maduras.

## Sessao 2026-07-04 (build curto + indentacao do editor)

- Criado `docs/COMANDOS_BUILD_VERIFICACAO.md` com a sequencia padrao de
  comandos para o usuario copiar/colar e atualizar binarios/verificar sem
  gastar contexto com logs longos.
- Editor QML: Tab agora indenta linha ou selecao com 4 espacos; Shift+Tab
  desindenta linha ou selecao; Enter replica a indentacao da linha atual e
  aumenta um nivel depois de `{`, `(`, `[` ou `:`. Ainda e MVP e nao substitui
  formatadores externos/LSP, mas remove o problema grosseiro de indentacao
  manual.

## Sessao 2026-07-04 (feedback do usuario: terminal REAL + botao Iniciar)

- FEEDBACK DO USUARIO sobre a entrega anterior: o terminal deve ser o
  terminal que ele usa no notebook (shell local, aliases, prompt), nao um
  executor proprio; e o botao deve alternar "Iniciar" ↔ "Parar".
- PROTOCOLO `0.13.0`: `terminal.open {}` → `{ shell }`,
  `terminal.input { data }`, `terminal.close {}` + eventos
  `event.terminal.data { data }` (chunks sanitizados) e
  `event.terminal.closed { exitCode }`.
- CORE: novo modulo `crates/kernwerk-core/src/terminal.rs`
  (`TerminalManager`):
  - Orquestra `script(1)` (util-linux) para alocar PTY REAL e rodar
    `$SHELL -i` na raiz do workspace — perfil/aliases/prompt do usuario
    carregados; sessao persistente (cd/vars sobrevivem entre comandos).
  - Roda com `TERM=dumb`; `AnsiSanitizer` (maquina de estados CSI/OSC/CR
    que sobrevive a chunks divididos, com testes) limpa a saida para a UI.
  - Leitura em CHUNKS (nao linhas) para o prompt aparecer sem newline.
  - `run.rs` ganhou helpers compartilhados `wait_for_exit` +
    `drain_readers` (contador atomico + deadline de 2s) usados por run e
    terminal: o `finished/closed` nao atrasa se um processo neto herdar o
    pipe (corrigiu flakiness real do teste de stop sob carga).
  - Fechar workspace fecha a sessao. `terminal.*` registrado em
    `command.list` (Alt+F12).
- UI:
  - Aba "Terminal" agora e o SHELL DO USUARIO: TextEdit read-only com a
    saida corrida (padrao Flickable do editor, autoscroll), input unico que
    manda tudo para o PTY (eco vem do proprio shell). Abrir a aba abre a
    sessao; sessao encerrada mostra aviso e Enter reabre. Alt+F12 foca.
  - O executor antigo virou aba "Executar" (ids run*/runModel/runInput no
    QML) — continua sendo a saida do botao Run com stdin por linha.
  - Botao do header: "▶ Iniciar" ↔ "■ Parar" (pedido explicito).
  - `CoreClient`: `terminalOpen/terminalInput/terminalClose`, propriedade
    `terminalActive`, sinais `terminalData/terminalClosed`.
- VERIFICACAO: clippy/fmt limpos, 3 rodadas completas de kw-test sem falha
  (91 testes; terminal.rs testa sanitizador e sessao real com sh),
  verificar-cpp limpo, builds debug+release, smoke offscreen, E2E via pipe
  (bash real abriu, prompt apareceu sanitizado, `echo`/`pwd` executaram na
  raiz do workspace, close reportado). Binarios do icone atualizados.
- LIMITACAO REGISTRADA: a UI renderiza texto sanitizado; full-screen TUIs
  (vim/htop) nao desenham mesmo com PTY real. Caminho futuro: renderizar
  ANSI/cores na UI (nao trocar a arquitetura, so o render).

## Sessao 2026-07-04 (dogfooding: botao Run + Terminal)

- PEDIDO DO USUARIO: "botao magico" de run e terminal na IDE, priorizando
  deixar a IDE funcional para projetos reais.
- PROTOCOLO `0.12.0`: `run.start { command? }`, `run.stdin { data }`,
  `run.stop {}` + eventos `event.run.started/output/finished`, documentados
  em `docs/03-ipc-protocol.md`.
- CORE: novo modulo `crates/kernwerk-core/src/run.rs` (`RunManager`):
  - Execucao NAO-BLOQUEANTE: `sh -c` na raiz do workspace, threads leitoras
    de stdout/stderr e thread waiter que emitem eventos pelo MESMO canal
    assincrono das notificacoes LSP (`lsp::EventSender`, habilitado em
    `Core::enable_lsp`, que agora tambem cria o RunManager). O core continua
    respondendo IPC durante a execucao (comprovado no E2E: run.stdin
    respondido no meio do run).
  - Um processo por vez; stdin encaminhado cru; stop via kill; fechar
    workspace mata o processo automaticamente.
  - Comando padrao do botao magico: RustCargo → `cargo run`; Cmake → unico
    executavel em `.kernwerk/build` (erros claros para zero/multiplos);
    demais tipos → erro orientando digitar o comando no Terminal.
  - Roteador `run_request_response` no lib.rs (padrao dos roteadores fs/lsp)
    e descritores em `run_command_descriptors()` (Shift+F10 / Ctrl+F2).
- UI: `CoreClient` com `runStart/runStdin/runStop`, propriedade `running` e
  sinais `runStarted/runOutput/runFinished` (eventos roteados em
  `handleNotification`; crash do core reseta `running`). `Main.qml`:
  - Botao "▶ Executar"/"■ Parar" no header ao lado de Compilar.
  - Aba "Terminal" no painel inferior: saida ao vivo (stdout cinza, stderr
    vermelho, comando em destaque), input unico que roda comando novo quando
    ocioso (prompt `$`) ou alimenta o stdin do processo (prompt `>`).
  - Status bar mostra "executando..."; `clearWorkspaceUiState` limpa tudo.
- LIMITACAO REGISTRADA (honesta, tambem no docs/03): sem TTY/PTY. Programas
  full-screen (vim/htop) nao funcionam; testes, servidores e prompts por
  linha funcionam. Terminal real com PTY e um passo futuro consciente.
- VERIFICACAO: fmt/clippy/test verdes (92 testes Rust; run.rs tem testes de
  output, stdin, stop, cap e default por kind), builds debug+release,
  verificar-cpp limpo, smoke offscreen ok, E2E via pipe com stdin
  interativo (`read nome` respondeu `eco: kernwerk` e finished success).
  Binarios do icone atualizados.
- Anti-duplicacao: run reusa o canal de eventos do LSP e o painel inferior;
  NAO criar outro canal de notificacao, outro executor de processos nem
  outro painel de saida. O parser de linha do Terminal e o mesmo padrao do
  build (threads + BufReader).

## Sessao 2026-07-04 (dogfooding: Find in Files + .clangd)

- DECISAO DO USUARIO: antes de Git (Fase 6), priorizar o que permite
  desenvolver o proprio Kernwerk dentro do Kernwerk Studio (dogfooding).
  Primeira entrega escolhida: busca no projeto. Proximas na fila (ainda nao
  implementadas): rodar comandos/testes pela IDE e build misto cargo+CMake
  pelo Ctrl+F9 (hoje o repo e detectado como RustCargo e o Ctrl+F9 so roda
  cargo).
- PROTOCOLO `0.11.0`: novo metodo `fs.search { query, caseSensitive? }` →
  `{ matches: [{ path, line, column, preview }], truncated }`, documentado em
  `docs/03-ipc-protocol.md`. Query literal (nao regex), case-insensitive
  ASCII por padrao, 1 match por linha, cap de 500 matches, preview de 200
  chars, paths relativos a raiz, 1-based.
- CORE: `fsops::search` — walk deterministico (profundidade, nome
  case-insensitive), pula symlinks, arquivos >1MiB, nao-UTF8 e dirs `.git`,
  `.kernwerk`, `.idea`, `.cache`, `target`, `build`, `node_modules`. Falhas
  de IO em entradas individuais sao ignoradas sem abortar a busca. O
  `handle_request` foi refatorado com roteador `fs_request_response`
  (mesmo padrao do `lsp_request_response`) para respeitar o limite clippy de
  100 linhas. Registrado em `command.list` com atalho Ctrl+Shift+F.
- CLI: `kernwerk-cli fs search <query>`.
- UI: `CoreClient::searchInFiles()` + sinal `searchResults`. `Main.qml` tem
  nova aba "Busca" no painel inferior (input + chip "Aa" de case + status de
  contagem/truncado + resultados clicaveis). Ctrl+Shift+F abre/foca a busca.
  Clique no resultado reusa `openDiagnostic`/pending jump. Estado limpo em
  `clearWorkspaceUiState`.
- `.clangd` novo na raiz aponta `CompilationDatabase` para
  `build/linux-clang-debug-strict`, para o clangd achar os headers Qt ao
  editar os `.cpp` da UI dentro da IDE (dogfooding C++ sem erros falsos).
- VERIFICACAO: cargo fmt/clippy/test verdes (novos testes de fsops::search,
  protocolo e dispatch fs.search), builds debug+release da UI,
  `verificar-cpp.sh` limpo, smoke offscreen ok, E2E via pipe no proprio repo
  (core release respondeu 0.11.0 e achou ocorrencias reais). Binarios do
  icone atualizados.
- Anti-duplicacao: busca usa `fsops`, `CoreClient`, o painel inferior e o
  fluxo `openDiagnostic`/pending jump existentes. Nao criar outro walker de
  arquivos, outro painel de resultados nem outro mecanismo de abrir arquivo
  em linha/coluna.

## Sessao 2026-07-03/04 (Fase 5.2 completion/references/rename + quebra do Qt)

- PROTOCOLO `0.10.0`: novos metodos `lsp.completion`, `lsp.references` e
  `lsp.rename`, documentados em `docs/03-ipc-protocol.md`. Completion e
  references usam os mesmos parametros posicionais de `lsp.definition`;
  rename recebe `newName` adicional.
- CORE (`lsp.rs` + `lib.rs`):
  - `completion`: ordena por `sortText`, limita a 50 itens, achata
    `CompletionItemKind` numerico em texto (`function`, `variable`, ...).
  - `references`: `includeDeclaration: true`, 1-based, limite de 200 usos.
  - `rename` e fim a fim: consulta o servidor, valida que TODOS os arquivos
    afetados estao no workspace, aplica edits em memoria (LSP 0-based/UTF-16
    -> offsets UTF-8) e so entao escreve no disco; `sync_if_open` re-sincroniza
    documentos abertos no servidor. Resource operations (criar/renomear/apagar
    arquivo) retornam erro estruturado sem tocar em nada.
- UI: `CoreClient` ganhou `requestCompletion()/requestReferences()/
  requestRename()` + sinais correspondentes. `Main.qml` tem popup de
  completion com filtro por prefixo (Ctrl+Space), painel de usos (Alt+F7) e
  dialogo de rename (Shift+F6) — atalhos JetBrains registrados tambem em
  `command.list`.
- VERIFICACAO (sessao de continuacao, madrugada 04):
  - `cargo kw-fmt`, `cargo kw-clippy`, `cargo kw-test` verdes.
  - `core.ping` responde `0.10.0`; `command.list` lista os 3 metodos novos.
  - E2E via pipe com clangd 18.1.3 real: `lsp.completion` retornou a funcao,
    `lsp.references` achou declaracao + 2 usos (1-based) e `lsp.rename`
    aplicou 3 edits corretamente no disco.
  - Binario release do core atualizado (`cargo build --release`).
- PENDENTE / BLOQUEIO DE AMBIENTE (IMPORTANTE):
  - Um `apt install qt6-base-dev qt6-tools-dev ... qtcreator` das 21:31 de
    2026-07-03 substituiu a arvore CMake do Qt6 do sistema e os configs de
    QML sumiram (`Qt6Qml`, `Qt6Quick`, `Qt6QmlIntegration`, `Qt6QmlModels`).
    `qt6-declarative-dev` NUNCA foi instalado via apt e nao existe
    `Qt6QmlConfig.cmake` em lugar nenhum do sistema. Consequencia: a UI NAO
    compila (configure falha em `find_package(Qt6 ... Qml)`).
  - Correcao aplicada: `qt6-declarative-dev` instalado pelo usuario em
    2026-07-03 ~23:20. Depois disso: corrigido erro real de compilacao
    deixado pela Fase 5.2 (`core_client.cpp:473`, range-loop sobre
    `QJsonArray` com referencia a temporario — a UI nunca tinha compilado
    apos as mudancas). Build debug + `verificar-cpp.sh` limpos; release da
    UI recompilado (23:28).
  - RESOLVIDO (23:32): o runtime QML tambem tinha ficado incompleto na troca
    de Qt; o usuario instalou `qml6-module-qtquick-window` e
    `qml6-module-qtqml`. Smoke offscreen via `scripts/kernwerk-studio`
    passou sem erro de componente e sem entradas no log de erros da IDE.
    A Fase 5.2 esta 100% verificada; os binarios release do icone (core e
    UI) estao atualizados.
  - LICAO PARA PROXIMAS IAs: nesta maquina o Qt6 do sistema (6.4.2) foi
    montado por partes via apt em 2026-07-03. Se a UI parar de compilar em
    `find_package(Qt6 ...)` ou falhar com "module ... is not installed",
    verificar pacotes `qt6-*-dev` e `qml6-module-*` antes de suspeitar do
    codigo.
- Anti-duplicacao: completion/references/rename reutilizam
  `position_request_with` do `LspManager`, `CoreClient` e o editor de
  `Main.qml`. Nao criar outro caminho de request LSP nem outro popup/painel
  paralelo.

## Sessao 2026-07-03 (Fase 5.1 LSP navigation)

- PROTOCOLO `0.8.0`: novos metodos `lsp.definition` e `lsp.hover`.
  Ambos recebem `{ path, content, line, column }`, com `line`/`column`
  1-based, e o core sincroniza o buffer antes de consultar o servidor.
- CORE:
  - `LspManager` ganhou roteamento de requests LSP com resposta correlacionada
    por `id`, timeout de 4s e erros estruturados.
  - O reader thread continua tratando `publishDiagnostics`, mas agora tambem
    entrega respostas de `textDocument/definition` e `textDocument/hover` para
    o request pendente correto.
  - `definition` aceita tanto `Location`/`Location[]` quanto `LocationLink[]`.
  - `hover` achata `MarkupContent`, `MarkedString` e arrays em texto simples
    para a UI.
  - `fsops::confine_file` virou helper reutilizavel para comandos que exigem
    arquivo regular dentro do workspace.
- UI:
  - `CoreClient` ganhou `requestDefinition()` e `requestHover()`, mais sinais
    `lspDefinitionResolved` e `lspHoverResolved`.
  - `Main.qml` calcula linha/coluna do cursor no editor, usa Ctrl+B para go to
    definition e Ctrl+Q para quick documentation/hover.
  - Go to definition reaproveita o fluxo existente de abrir arquivo e pular
    para linha/coluna. Hover aparece em popup pequeno dentro da ilha do editor
    e some ao editar/trocar contexto.
- LIMITACOES REGISTRADAS: sem completion, semantic tokens, find usages,
  rename/refactor, restart LSP, cancelamento de request LSP ou UI de status
  dedicada por servidor.
- Anti-duplicacao: a navegacao semantica usa `CoreClient`, `lsp.rs`,
  `Main.qml`, `fsops`, e o fluxo de abertura/pending jump existente. Nao criar
  outro cliente LSP, outro editor ou outro painel para esses resultados.
- Verificacao desta entrega: `cargo kw-fmt`, `cargo kw-check`,
  `cargo kw-clippy`, `cargo kw-test`, `scripts/verificar-cpp.sh`, builds
  `dev-local` e `dev-local-release`, `cargo build --release -p kernwerk-core`,
  smoke offscreen via `scripts/kernwerk-studio`, checagem JSON-RPC de
  `core.ping`/`command.list` e smoke real com `clangd` em `/tmp` validando
  `lsp.definition` (`main.cpp:1:5`) e `lsp.hover` (assinatura de funcao).
  Binarios release usados pelo icone foram atualizados.

## Sessao 2026-07-03 (Fase 5 LSP diagnostics - estabilizacao)

- PROTOCOLO `0.7.0`: novo metodo `lsp.didChange` e eventos
  `event.lsp.status` / `event.lsp.diagnostics`, documentados em
  `docs/03-ipc-protocol.md`.
- CORE: `crates/kernwerk-core/src/lsp.rs` gerencia servidores LSP por
  workspace. Suporte atual:
  - C/C++ via `clangd --background-index`;
  - Rust via `rust-analyzer`.
- O core faz handshake `initialize`/`initialized`, responde requests
  servidor->cliente com `null` quando ainda nao suportados, sincroniza
  documentos por texto completo (`didOpen`/`didChange`/`didSave`) e converte
  `textDocument/publishDiagnostics` para diagnosticos Kernwerk 1-based.
- `Core::run_stdio` multiplexa linhas JSON-RPC da UI e notificacoes
  assincronas de LSP no mesmo stdout, sem intercalar mensagens no meio da
  linha.
- `fs.read` envia `didOpen`; `fs.write` envia `didSave`; `lsp.didChange`
  envia alteracoes apos debounce do editor. O comando agora valida caminho
  como os demais comandos de arquivo: canonicaliza, exige workspace aberto e
  rejeita arquivo fora da raiz.
- UI: `CoreClient::notifyFileChanged()` envia `lsp.didChange`; `Main.qml`
  reinicia `changeDebounce` em edicoes reais do `TextEdit` e integra
  `event.lsp.diagnostics` na aba Problemas com origem `lsp`.
- UX: fechar/trocar workspace limpa abas, explorer, saida de build,
  problemas de build/LSP, pending jump e highlighter do editor.
- LIMITACOES DA EPOCA: sincronizacao full-document; sem hover, completion,
  go-to-definition, semantic tokens, rename/refactor, comando de restart LSP,
  cancelamento de requests LSP ou UI dedicada de status por servidor. Hover e
  go-to-definition foram implementados depois na Fase 5.1 acima.
- Anti-duplicacao: proximas IAs devem estender `CoreClient`, `lsp.rs`,
  `Main.qml`, `docs/03-ipc-protocol.md` e os comandos existentes. Nao criar
  outro cliente IPC, outro gerenciador LSP paralelo, outro painel de Problemas
  ou outro protocolo para diagnosticos.
- Verificacao desta entrega: `cargo kw-fmt`, `cargo kw-check`,
  `cargo kw-clippy`, `cargo kw-test`, `scripts/verificar-cpp.sh`, builds
  `dev-local` e `dev-local-release`, `cargo build --release -p kernwerk-core`,
  smoke offscreen via `scripts/kernwerk-studio` e checagem
  JSON-RPC de `core.ping`/`command.list`. Binarios release usados pelo icone
  foram atualizados.

## Checkpoint para a proxima etapa

Estado pronto para continuar:

- A base de workspace/editor/build/problemas esta funcional e validada.
- Binarios release do icone (core e UI) atualizados e smoke offscreen ok
  (2026-07-03 23:32). O bloqueio de Qt da sessao Fase 5.2 foi resolvido.
- `command.list` inclui `lsp.didChange`, `lsp.definition`, `lsp.hover`,
  `lsp.completion`, `lsp.references` e `lsp.rename`; `core.ping` responde
  protocolo `0.10.0`.
- Diagnosticos LSP ja chegam pelo core e sao exibidos na aba Problemas.
- Go to definition (Ctrl+B) e hover/quick documentation (Ctrl+Q) usam requests
  LSP com resposta correlacionada pelo core.
- Fechar/trocar workspace limpa corretamente estado visual, abas, problemas e
  buffers associados.
- `clangd` existe nesta maquina em `/usr/bin/clangd`. Nao assumir que
  `rust-analyzer` existe sem detectar.
- `git status` nao funcionou neste ambiente mesmo existindo diretorio `.git`;
  nao depender de Git para descobrir mudancas locais nesta sessao.

Proxima etapa recomendada (dogfooding, decisao do usuario em 2026-07-04):

1. [FEITO] Rodar comandos/testes pela IDE — aba Executar + botao Iniciar.
2. [FEITO] Terminal com PTY real — aba Terminal roda o $SHELL do usuario.
3. [FEITO] IntelliSense: completion automatico ao digitar + cores
   semanticas via LSP (variavel/funcao/tipo/parametro/campo).
4. Build misto cargo+CMake pelo Ctrl+F9 para repos com os dois marcadores.
5. Polimento continuo: cores ANSI no Terminal, historico de comandos com
   setas, run configurations, popup de completion com docs/detalhe.
6. Pos-V1.0 (decisao do usuario): SQLite como indice persistente de
   simbolos/busca para projetos medios/grandes.
7. Git (Fase 6) e code actions LSP na sequencia.
3. Reaproveitar o que ja existe:
   - `CoreClient` para novos metodos IPC;
   - `lsp.rs` para gerenciar servidores;
   - `Main.qml` para cursor, abertura de arquivo e painel existente;
   - aba Problemas existente para diagnosticos;
   - `docs/03-ipc-protocol.md` para versionar qualquer contrato novo.
4. Nao criar cliente LSP paralelo, painel de problemas duplicado, outro IPC de
   arquivos ou outro seletor de workspace.
5. Ao terminar qualquer mudanca de core/UI, repetir validacao rigorosa e
   recompilar os binarios release usados pelo icone.

## Sessao 2026-07-03 (Fase 4 Build + Problemas + ajuste de Ferramentas)

- FERRAMENTAS: a sugestao `sudo pacman -S ...` agora so aparece quando o
  proprio `pacman` existe no PATH (pedido do usuario: nesta maquina Pop!_OS a
  sugestao era ruido). Em Arch/CachyOS ela volta automaticamente. Implementado
  em `ToolDetector::suggested_install_for`; testes atualizados.
- PROTOCOLO `0.6.0`: novos tipos `BuildDiagnosticSeverity`, `BuildDiagnostic`,
  `BuildRunResult`. Novo metodo `build.run` + notificacoes
  `event.build.started/output/diagnostic/finished` (docs/03 atualizado).
- CORE: novo modulo `crates/kernwerk-core/src/build.rs`:
  - `run_build(root, kind, sink)`: RustCargo → `cargo build
    --message-format=json` (diagnosticos do JSON, texto `rendered` vai para a
    saida bruta, JSON cru nao); Cmake → configure em `.kernwerk/build` (so na
    primeira vez) + `cmake --build`, com parser `arquivo:linha:col: nivel:`
    sem regex. Outros kinds → erro "nao suportado".
  - Streaming: threads leitoras de stdout/stderr + mpsc; eventos via sink.
  - `Core::handle_request_streaming`/`handle_json_line_streaming` emitem
    notificacoes JSON-RPC (sem id) intercaladas antes da resposta;
    `run_json_lines` escreve e da flush em cada notificacao.
  - LIMITACOES REGISTRADAS: sem cancelamento; o core fica bloqueado durante o
    build (a UI continua responsiva); sem `build.configure` separado; sem
    execucao do binario compilado (run) ainda.
- CLI: `kernwerk-cli build` emite `build.run` (util para testar via pipe).
- UI: `CoreClient` ganhou `runBuild()`, propriedade `building`, roteamento de
  notificacoes (eventos NAO poluem mais o log "IDE" linha a linha) e sinais
  `buildStarted/buildOutput/buildDiagnostic/buildFinished`. `Main.qml`:
  botao "Compilar" no header (Ctrl+F9, memoria muscular JetBrains), abas
  "Build" (saida ao vivo) e "Problemas (N)" no painel inferior; clique num
  problema abre o arquivo e posiciona o cursor na linha/coluna; build com
  falha troca automaticamente para a aba Problemas; status bar mostra
  "compilando...".
- E2E validado via pipe: projeto cargo com erro proposital gerou
  `event.build.diagnostic {file: src/main.rs, line: 1, column: 26}` e
  resposta final com `success:false, exitCode:101, diagnostics:1`.
- Verificacao: 58 testes Rust, clippy/tidy/format limpos, smoke offscreen ok,
  binarios do icone atualizados.

## Sessao 2026-07-03 (syntax highlighting + logs de erro da IDE)

- SYNTAX HIGHLIGHTING implementado (prioridade 1 da decisao do usuario):
  - Arquivos novos: `ui/src/editor_highlighter.h` e
    `ui/src/editor_highlighter.cpp` (registrados no qt_add_qml_module).
  - Tipo QML `EditorHighlighter` (herda QSyntaxHighlighter): propriedades
    `document` (recebe `editor.textDocument`) e `filePath` (deriva a
    linguagem pela extensao; `CMakeLists.txt` e caso especial).
  - Linguagens: rust, cpp, python, cmake, toml, js/qml, json, shell; demais
    caem em "plain" (sem cores).
  - Regras: keywords (accent #ffbb00, semibold), strings (#7fbf7f),
    comentarios (#8f8a7c, italico), numeros (#7aa2d8), metadados/macros/
    atributos/secoes (#d16d6d). Paleta espelha docs/05.
  - Spans multi-linha via estado de bloco: /* */ em rust/cpp/js e """ em
    python. Limitacao conhecida (MVP): delimitadores dentro de strings podem
    confundir o span; cores semanticas reais virao do LSP na Fase 5.
  - Wiring em `Main.qml`: instancia `editorHighlighter` dentro do TextEdit;
    `filePath` e atualizado em selectTab/closeTab/onFileLoaded.
  - Achado de tidy corrigido: QRegularExpression globais violavam
    cert-err58-cpp; viraram funcoes com static local.
- LOGS DA IDE:
  - Aba "Logs" do painel inferior renomeada para "IDE" (tooltip da sidebar
    "Log da IDE", chip da status bar "IDE") — e log interno de
    desenvolvimento, nao log do projeto do usuario.
  - `CoreClient::appendErrorLog` grava falhas da IDE em
    `~/.cache/kernwerk-studio/logs/kernwerk-ui-erros.txt` (ver "Decisoes de
    produto" acima).
- Verificacao: build debug/release verdes, clang-tidy limpo, smoke offscreen
  ok, binarios do icone atualizados. Rust nao foi tocado nesta sessao.

## Sessao 2026-07-03 (painel direito Assistente KW)

- `Main.qml`: nova ilha `assistantPanel` (300px, direita), botao ✦ na sidebar,
  `showAssistant`, `assistantModel` e `sendAssistantMessage()`. Mensagem de
  boas-vindas e resposta automatica explicavam originalmente que providers
  chegariam na Fase 7 com confirmacao explicita — nada de IA simulada.
  Decisao atualizada em 2026-07-04: Fase 7 vira terminal dedicado para
  Claude/Codex/GPT CLI, sem provider proprio no curto prazo.
- Nenhuma mudanca em protocolo/core; nenhum arquivo novo. Builds debug e
  release verdes; binarios do icone atualizados.

## Sessao 2026-07-03 (auditoria + sidebar/painel inferior)

- Auditoria completa do estado herdado: Rust (49 testes, clippy
  pedantic/nursery), C++ (build estrito, clang-tidy), QML (smoke offscreen),
  `workspace.browse` E2E via pipe CLI->core. NENHUMA falha encontrada; nenhuma
  referencia orfa ao overlay antigo ou a QtQuick.Dialogs.
- UI: `CoreClient` ganhou `detectTools()` + sinal `toolsListed` (usa os
  metodos `tools.detect`/`tools.status` ja existentes no core; nada novo no
  protocolo). `Main.qml` ganhou sidebar de icones, painel inferior com abas e
  a propriedade `showExplorer`. O chip "Logs" da status bar abre a aba Logs
  do painel inferior.
- Binarios release do icone recompilados e revalidados.

## Instrucoes por agente (confirmado pelo usuario em 2026-07-03)

O usuario trabalha com dois agentes de IA em paralelo e vai dividir
responsabilidades entre eles. Este arquivo e o ponto de sincronizacao: quando
os tokens de um acabarem, o outro continua o desenvolvimento lendo este
arquivo.

- FABLE (Claude Code): sessao de DESENVOLVIMENTO. Autorizado a alterar codigo
  quando o usuario pedir "prossiga". Responsavel por manter este arquivo
  sincronizado ao fim de cada entrega.
- CODEX: a instrucao "nao alterar codigo, apenas documentacao" registrada
  anteriormente valia para a sessao do Codex, NAO para o Fable (o usuario
  confirmou isso explicitamente). Se o usuario mandar o Codex desenvolver,
  ele deve ler este arquivo inteiro, seguir a politica anti-duplicacao e as
  decisoes registradas, e continuar do estado descrito na secao "Estado
  tecnico atual" + sessoes datadas.
- Regras comuns aos dois:
  - O icone/atalho do aplicativo ja existe. Nao recriar, reinstalar ou
    modificar icone/atalho sem pedido explicito. Recompilar os binarios
    release que o atalho executa e obrigatorio apos mudancas de codigo.
  - Ler o codigo existente antes de editar; nada de codigo duplicado.
  - Verificacao antes de dar por pronto: `cargo kw-fmt/kw-clippy/kw-test`,
    build debug + release da UI, `scripts/verificar-cpp.sh`, smoke offscreen.

## Decisoes de produto registradas em 2026-07-03

- ASSISTENTE KW (Fase 7): por baixo dos panos ele vai executar os CLIs
  `claude` (Claude Pro) e `codex` no terminal, como o usuario ja faz
  manualmente. O painel e um "atalho para IA": visualmente separa terminais
  por provider, evitando digitar `claude`/`codex` toda vez. NAO e integracao
  via API num primeiro momento. Implementacao fica para a Fase 7 (o usuario
  disse "deixa mais pra frente" os detalhes).
- LOGS: o log de tráfego IPC/acoes da IDE e ferramenta de desenvolvimento DA
  IDE, nao do usuario final. Por isso a aba do painel inferior foi renomeada
  para "IDE". No fluxo real de projeto, "Logs/Problemas" devem mostrar erros
  DO PROJETO (build, diagnosticos) — essas abas nascem nas Fases 4/5.
- ERROS DA IDE EM .TXT: qualquer erro/mau funcionamento da IDE (crash do
  core, stderr do core, falha de processo, resposta IPC invalida, core nao
  encontrado) e gravado com timestamp ISO em:
  `~/.cache/kernwerk-studio/logs/kernwerk-ui-erros.txt`
  (implementado em `CoreClient::appendErrorLog`; caminho exposto ao QML pela
  propriedade `errorLogFile`). Segue o diretorio de logs previsto em
  docs/07-tooling-lifecycle.md.

## Politica anti-duplicacao de codigo

Regra obrigatoria: nao duplicar codigo, componentes, comandos IPC ou estado de
UI que ja existem.

Antes de implementar qualquer coisa, uma IA deve:

1. Buscar implementacoes existentes com `rg`.
2. Ler os arquivos relevantes antes de propor ou editar.
3. Reaproveitar, estender ou refatorar o que ja existe.
4. Criar novo arquivo/componente apenas quando houver necessidade real e sem
   sobrepor responsabilidade de componente existente.
5. Atualizar docs quando mudar contrato, arquitetura ou comportamento.

Elementos que ja existem e devem ser reaproveitados:

- `ui/src/core_client.h` e `ui/src/core_client.cpp`: cliente IPC da UI. Nao
  criar outro cliente IPC paralelo.
- `ui/qml/FolderPickerDialog.qml`: seletor proprio de workspace. Nao voltar
  para `QtQuick.Dialogs` e nao criar outro seletor de pasta duplicado.
- `ui/qml/Main.qml`: layout principal, sidebar, explorer, editor, painel
  inferior e estado visual (`showExplorer`, abas, logs/ferramentas).
- `ui/qml/Theme.qml`: tokens visuais. Nao espalhar cores soltas quando um token
  existente resolver.
- `workspace.browse`: navegacao de diretorios para abrir workspace. Nao criar
  outro metodo IPC para a mesma finalidade.
- `fs.list`, `fs.read`, `fs.write`: acesso a arquivos dentro do workspace. Nao
  acessar filesystem diretamente pela UI.
- `tools.detect` e `tools.status`: deteccao/status de ferramentas. Nao criar
  outro fluxo paralelo de deteccao na UI.

Exemplos do que nao fazer:

- Nao criar `CoreClient2`, `WorkspaceClient`, `FileSystemClient` ou similar se
  a responsabilidade pertence ao `CoreClient`.
- Nao criar outro `FolderPicker`/`OpenProjectDialog` com a mesma funcao do
  `FolderPickerDialog.qml`.
- Nao duplicar listas de arquivos no QML quando `treeModel`/explorer ja cobre o
  caso.
- Nao criar novo comando `workspace.list`, `workspace.folders` ou equivalente
  para navegar pastas se `workspace.browse` ja atende.
- Nao copiar blocos grandes de QML para variar layout; extrair componente ou
  refatorar quando a duplicacao ficar real.

Se uma IA achar que precisa duplicar algo, deve primeiro registrar o motivo no
plano da tarefa e pedir confirmacao do usuario.

## Cuidados para proximas IAs

- Nao adicionar acesso direto a filesystem na UI para contornar problemas de UX.
- Nao usar `QtQuick.Dialogs` para abrir workspace; o seletor proprio substitui
  essa dependencia.
- Nao duplicar protocolo de navegacao de pasta; use `workspace.browse`.
- Nao duplicar codigo existente; aplicar a Politica anti-duplicacao acima.
- Atualizar `docs/03-ipc-protocol.md` e schemas quando mudar contrato IPC.
- Manter docs sincronizadas quando comportamento de workspace/editor mudar.

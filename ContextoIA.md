# ContextoIA - Continuidade do Kinein Vectis

Este arquivo registra decisoes de produto/arquitetura para IAs que continuarem
o desenvolvimento do repositorio. Use junto de `AGENTS.md` e dos documentos em
`docs/`. Este arquivo e enxuto de proposito: contrato/estado detalhado vive em
`docs/03-ipc-protocol.md`, `docs/ARCHITECTURE.md` e
`docs/BACKEND_TO_UI_UX_ROADMAP.md`; aqui so ficam decisoes vigentes e
prioridade atual (ver `docs/15-engineering-debt-and-refactor.md` sobre por
que este arquivo foi enxugado em 2026-07-05).

## ATENCAO — trabalho real ainda nao commitado (2026-07-05)

O ultimo commit e `32523cb` ("run quality.run e test.run as async cancelable
jobs"). Desde entao, ha uma quantidade grande de trabalho **real, testado e
funcionando** que so existe no working tree, nunca commitado — nao apagar/
resetar sem revisar antes (`git status`, nunca `git checkout -- .` ou
`git clean` sem olhar primeiro):

- Feature de scan de ambiente inteira: `crates/kinein-core/src/lib.rs`
  (+161), `crates/kinein-core/src/tools.rs` (+86),
  `crates/kinein-core/src/tests/tools.rs` (+86, testes novos).
- Feature de diagnostics unificados inteira: `crates/kinein-protocol/src/
  diagnostic.rs` (**arquivo novo, nunca commitado, sem o qual o build quebra
  num clone limpo**), mais `crates/kinein-protocol/src/build.rs` (+72),
  `crates/kinein-protocol/src/lib.rs` (+4) e `crates/kinein-core/src/
  handlers/build.rs` (+89) que a usam.
- Trabalho adicional em `crates/kinein-core/src/jobs/manager.rs` (+138) e
  `crates/kinein-protocol/src/job.rs` (+33) alem do que ja foi commitado na
  "fundacao" do job system.
- `crates/kinein-core/src/lsp/parse.rs` (+32/-diff) e `crates/kinein-core/
  src/commands.rs` (+8), sem contexto claro do motivo — revisar antes de
  commitar.
- `docs/03-ipc-protocol.md` (+193/-) ja documenta boa parte disso.
- Mais as mudancas desta sessao (limpeza de docs, `ContextoIA.md`,
  `core_client.h/.cpp`, `Main.qml`, `AGENTS.md`, `docs/README.md`,
  `docs/15`).

Tudo isso passa no gate completo (`cargo fmt/clippy -D warnings/test`, 170+
testes) e nao e trabalho quebrado — so nunca foi commitado. Recomendacao:
commitar em pedacos logicos (scan de ambiente; diagnostics unificados; job
manager extra; a limpeza de docs + job.cancel desta sessao) antes de seguir
com o proximo refactor grande de UI, para nao acumular ainda mais coisa
uncommitted em cima de uncommitted.

**Ruido separado, nao mexer:** quase todo arquivo rastreado do repositorio
mudou de modo `100644` para `100755` (executavel) — isso NAO e conteudo
mudando, e so bit de permissao, provavelmente de uma copia/extracao do
projeto entre maquinas. Polui `git status`/`git diff --stat` do repositorio
inteiro. Vale normalizar (`chmod 644` de volta em arquivos que nao sao
scripts) numa limpeza separada, sem misturar com os commits de feature acima.

## Direcao do produto

- Produto: Kinein Vectis.
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
- Abrir outro workspace: mesmo root pode preservar abas; root diferente limpa
  abas e estado visual do workspace anterior.
- Fechar projeto deve fechar automaticamente abas abertas e limpar explorer.

## Estado tecnico atual

- Arquitetura: Qt/QML UI <-> JSON-RPC local/stdin-stdout <-> Rust core.
- Protocolo IPC atual: `0.20.0`. Lista completa de comandos/eventos e contrato:
  `docs/03-ipc-protocol.md` (nao duplicar essa lista aqui).
- Job system assincrono/cancelavel (`job.list`/`job.cancel`, `event.job.*`)
  cobre `build.run`, `quality.run`, `test.run` e `environment.scan`: cada um
  responde `{ jobId }` na hora e emite `event.<dominio>.finished` com o
  resultado real. Arquitetura: `docs/ARCHITECTURE.md` §7. Impacto/estado de UI:
  `docs/BACKEND_TO_UI_UX_ROADMAP.md` (P0).
- Diagnostics parcialmente unificados: modelo comum `Diagnostic` em
  `kinein-protocol` usado por `event.build.diagnostic`/`event.quality.diagnostic`
  (faltam ids estaveis, actions e `logRef` — ver roadmap P1).
- `workspace.browse`/`fs.*`/`fs.findFiles` seguem confinados ao workspace
  aberto; `fs.*` nunca navega fora dele. Detalhe de cada metodo:
  `docs/03-ipc-protocol.md`.
- A UI sobe `kinein-core` como processo filho via `CoreClient`. O usuario testa
  pelo icone "Kinein Vectis" do menu de aplicativos (`scripts/kinein-vectis`),
  que prefere binarios release (`build/linux-clang-release-hardened/ui/
  kinein-vectis`, `target/release/kinein-core`) e cai para debug se release
  nao existir. Depois de alterar core/UI, rebuild obrigatorio:
  `cargo build --release -p kinein-core` +
  `cmake --build --preset dev-local-release`.
- **Toolchain local (2026-07-05, maquina atual):** o Clang do sistema virou
  trunk (Clang 21 experimental) e o GCC trunk foi para 15/16. Com Clang trunk,
  o proprio codigo gerado pelo moc do Qt dispara `-Wctad-maybe-unsupported`;
  com GCC nativo, o codigo QML gerado (`qmlcache_loader`,
  `qmltyperegistrations`, JS AOT de `Main.qml`/`FolderPickerDialog.qml`)
  dispara `-Wfloat-equal`/`-Wuseless-cast`/`-Wmissing-declarations`. Nenhum dos
  dois e bug nosso — e codigo gerado pelo Qt que este toolchain de ponta
  considera suspeito. Workaround **local** (`CMakeUserPresets.json`,
  gitignored, nao afeta CI/outras maquinas): `dev-local`/`dev-local-release`
  usam `g++`/`gcc` nativos e `KINEIN_WARNINGS_AS_ERRORS=OFF`.
  `cmake/KineinStrictOptions.cmake` (rastreado no git) permanece inalterado.
  Se o proximo agente ve warnings so em `.rcc/qmlcache/*`, `moc_*` ou
  `*_qmltyperegistrations.cpp`, e esse problema conhecido, nao regressao.
  Em outra maquina, ajustar `CMakeUserPresets.json` conforme o toolchain local
  (nao editar `cmake/KineinStrictOptions.cmake` sem motivo registrado).

## Modularizacao pos-V1

Fase de "monolito modular" concluida em 2026-07-05: core, protocolo e CLI
quebrados por responsabilidade; rename kernwerk -> Kinein Vectis concluido.
Detalhe completo (arquivos, pastas, verificacao): ver
`docs/15-engineering-debt-and-refactor.md`.

## Strict mode

- Rust: `unsafe_code = "forbid"`, warnings como erro, clippy pedantic/nursery,
  sem `unwrap`/`expect`/`panic`/`todo`/`dbg!` fora de casos aceitos por testes.
- C++/Qt: C++23, warnings-as-errors, sanitizers em Debug, hardening/LTO em
  Release via `cmake/KineinStrictOptions.cmake`.
- Futuramente: seletor de nivel de rigidez (Strict padrao / Balanced /
  Relaxed so por escolha explicita).

## Prioridade imediata (2026-07-05, decisao do usuario)

UX/UI basica (workspace, explorer, abas, layout, sidebar, paineis, syntax
highlighting, build/test/quality, LSP MVP) esta **feita** — ver histórico no
git e em `docs/BACKEND_TO_UI_UX_ROADMAP.md`. A partir de 2026-07-05 a ordem
mudou:

1. **Nao empilhar mais fases de backend antes de voltar ao Qt.** O usuario
   pediu explicitamente para nao atrasar o frontend: o projeto ja e
   medio/medio-grande, entao cada entrega de backend "invisivel" tem que vir
   acompanhada de um ganho de UI logo em seguida, nao de mais uma fase de
   contrato.
2. [feito 2026-07-05] Cancelar build/test/quality/environment-scan pela UI:
   `CoreClient` guarda o `jobId` aceito por dominio e ganhou
   `cancelBuild()/cancelTests()/cancelQuality()/cancelEnvironmentScan()`
   chamando `job.cancel`; `Main.qml` ganhou um "×" na status bar ao lado de
   cada indicador (compilando/testando/analisando/scan de ambiente).
3. Itens de backend ainda pendentes no roadmap (Process Runner unificado,
   CMake/Cargo service, Project Health, Settings/Storage, Risk Engine, Run
   Configs, Git, AI Bridge) ficam **explicitamente adiados**: nenhum deles
   bloqueia o cancelamento de jobs nem a proxima fatia de UI. Retomar um deles
   so quando uma fatia de UI concreta precisar dele.
4. **TAREFA CONCLUIDA EM 2026-07-06:** quebrar `Main.qml` (~4600 linhas
   originalmente) em componentes por dominio, controllers/stores nao visuais e
   roteadores IPC por dominio. Historico da sequencia abaixo; o estado atual e
   o bloco "Main deixou de ser god file/god controller".

   **Atualizacao CODEX 2026-07-06:** primeira fatia executada e validada.
   `Main.qml` caiu primeiro para ~4130 linhas. Foram extraidos e registrados em
   `ui/CMakeLists.txt`: `BottomTabBar.qml`, `BuildPanel.qml`,
   `TestsPanel.qml`, `ProblemsPanel.qml`, `IdeLogPanel.qml`,
   `ToolsPanel.qml` e `WorkspaceStatusBar.qml`. `Main.qml` continua dono dos
   `ListModel`s e dos handlers `Connections { target: coreClient }`; os
   componentes novos recebem dados por `property` e devolvem acoes por
   `signal` quando necessario (`ProblemsPanel.openRequested`,
   `WorkspaceStatusBar.cancel*Requested`, etc.). Tambem foi corrigido um lint
   C++ pequeno em `ui/src/editor_highlighter.cpp` (`Rule` com designated
   initializer). Verificacao feita: `cmake --build --preset dev-local`,
   `cmake --build --preset dev-local-release`, smoke offscreen release
   (encerrou por `timeout`, so mostrou warning antigo de `Shortcut`) e
   `scripts/verificar.sh --rapido` verde.

   **Atualizacao CODEX 2026-07-06 (segunda fatia):** tambem foram extraidos
   `TerminalPanel.qml`, `RunPanel.qml` e `SearchPanel.qml`; `Main.qml` caiu
   para ~3810 linhas. O terminal agora envia input via `coreClient.terminalInput`
   (antes o QML chamava `runInput`, metodo inexistente no `CoreClient`).
   Verificacao repetida: `cmake --build --preset dev-local`,
   `cmake --build --preset dev-local-release`, smoke offscreen release
   (mesmo warning antigo de `Shortcut`) e `scripts/verificar.sh --rapido`
   verde.

   **Atualizacao CODEX 2026-07-06 (arquitetura QML por responsabilidade):**
   os componentes QML agora ficam em pastas por dominio, nao soltos em
   `ui/qml/`. `Main.qml` caiu para **2745 linhas** e continua sendo o
   orquestrador de estado/modelos/IPC. Estrutura atual:
   `shell/` (`BottomTabBar`, `WorkspaceStatusBar`), `panels/bottom/`
   (`BuildPanel`, `TestsPanel`, `ProblemsPanel`, `TerminalPanel`,
   `RunPanel`, `SearchPanel`, `IdeLogPanel`, `ToolsPanel`), `workspace/`
   (`FolderPickerDialog`), `assistant/` (`AssistantPanel`), `project/`
   (`ProjectCreateDialog`, `ProjectEntryContextMenu`,
   `ProjectEntryRenameDialog`, `ProjectEntryDeleteDialog`), `command/`
   (`SearchEverywhereDialog`) e `editor/` (`SymbolRenameDialog`,
   `EditorTabsBar`, `EditorTextSurface`, `EditorHoverPopup`,
   `EditorCompletionPopup`, `EditorUsagesPopup`).
   `ui/CMakeLists.txt` usa `QT_RESOURCE_ALIAS` para manter os nomes dos tipos
   QML estaveis apesar dos arquivos fisicamente organizados em subpastas.
   Componentes recebem dados por `property` e retornam acoes por `signal`; nao
   chamam ferramentas externas nem falam diretamente com filesystem/core, salvo
   quando o pai (`Main.qml`) ja encaminha via `CoreClient`.

   Verificacao desta terceira fatia: `cmake --build --preset dev-local`,
   smoke offscreen debug, `cmake --build --preset dev-local-release`, smoke
   offscreen release (ambos encerraram por `timeout`; unico aviso novo/visivel
   segue sendo o warning antigo de `Shortcut`) e
   `scripts/verificar.sh --rapido` verde. Durante a extracao foi preservado o
   padrao arquitetural: `Main.qml` e dono dos `ListModel`s e dos handlers
   `Connections { target: coreClient }`; componentes novos so recebem modelos,
   texto, flags e emitem sinais.

   **Atualizacao CODEX 2026-07-06 (warning de Shortcut resolvido):** o aviso
   runtime `QML Shortcut: Only binding to one of multiple key bindings...`
   vinha de `Shortcut { sequence: StandardKey.Save }`. Corrigido sem camada
   nova, usando `sequences: [StandardKey.Save]`. Smoke offscreen debug e
   release ficaram sem saida QML; `scripts/verificar.sh` completo terminou
   verde e atualizou os binarios release usados pelo icone.

   **Atualizacao CODEX 2026-07-06 (Main deixou de ser god file/god
   controller):** `Main.qml` foi reduzido para **336 linhas** e virou
   composition root. Ele nao possui mais `ListModel`, nao possui
   `Connections { target: coreClient }`, `Shortcut`, `Timer`, helper de
   dominio ou componente visual pesado embutido, e nao concentra os
   estados/modelos de editor, project tree, jobs, busca, run/terminal,
   assistente ou workspace.
   Estado e orquestracao foram movidos para controllers/stores nao visuais:
   `editor/EditorController.qml`, `project/ProjectTreeController.qml`,
   `jobs/JobsController.qml`, `runtime/RuntimeController.qml`,
   `search/SearchController.qml`, `workspace/WorkspaceController.qml`,
   `assistant/AssistantController.qml` e `command/CommandDispatcher.qml`.
   Eventos do `CoreClient` foram separados em roteadores IPC por dominio:
   `ipc/WorkspaceEventRouter.qml`, `ipc/EditorEventRouter.qml`,
   `ipc/JobsEventRouter.qml`, `ipc/SearchEventRouter.qml` e
   `ipc/RuntimeEventRouter.qml`. A superficie visual central foi consolidada em
   `shell/ShellWorkspaceHost.qml` (sem acesso direto a `CoreClient`), o editor
   foi quebrado em subcontrollers e a implementacao C++ do `CoreClient` foi
   fatiada por responsabilidade interna.
   Validacao desta etapa: `cmake --build --preset dev-local`, smoke offscreen
   debug, `cmake --build --preset dev-local-release`, smoke offscreen release,
   smoke offscreen pelo launcher `scripts/kinein-vectis` e
   `scripts/verificar.sh --rapido` verde.

   **Atualizacao CODEX 2026-07-06 (ProjectExplorer extraido):** a arvore de
   projeto saiu de `Main.qml` e virou `ui/qml/project/ProjectExplorer.qml`,
   registrada em `ui/CMakeLists.txt` com `QT_RESOURCE_ALIAS`. `Main.qml` caiu
   para **2533 linhas**; `ProjectExplorer.qml` ficou com **253 linhas**.
   `treeModel`, selecao e IPC continuam no pai. O componente so recebe
   `workspaceName`, `workspaceKindLabel`, `selectedPath` e `entriesModel`, e
   emite sinais para criar arquivo/pasta, atualizar, fechar workspace,
   selecionar entrada, expandir/recolher diretorio, abrir arquivo e pedir menu
   de contexto. Validado com `cmake --build --preset dev-local`, smoke
   offscreen debug, `cmake --build --preset dev-local-release`, smoke offscreen
   release e `scripts/verificar.sh --rapido` verde.

   **Atualizacao CODEX 2026-07-06 (editor visual extraido):** a barra de abas
   e a superficie visual do editor sairam de `Main.qml` e viraram
   `ui/qml/editor/EditorTabsBar.qml` e
   `ui/qml/editor/EditorTextSurface.qml`, ambas registradas em
   `ui/CMakeLists.txt` com `QT_RESOURCE_ALIAS`. `Main.qml` caiu para
   **2340 linhas**; `EditorTabsBar.qml` ficou com **110 linhas** e
   `EditorTextSurface.qml` com **176 linhas**. `Main.qml` continua dono de
   `openFiles`, `currentTab`, `loadingEditorText`, dirty state, timers,
   requests LSP, save/read/write e IPC. `EditorTabsBar` so recebe modelo/indice
   e emite selecao, fechamento e salvar. `EditorTextSurface` encapsula
   `Flickable + TextEdit + EditorHighlighter`, expoe texto/cursor/selecao e
   emite sinais para texto editado, completion, hover/usages, indent/unindent
   e newline. Validado com `cmake --build --preset dev-local`, smoke offscreen
   debug, `cmake --build --preset dev-local-release`, smoke offscreen release e
   `scripts/verificar.sh --rapido` verde.

   **Atualizacao CODEX 2026-07-06 (Main deixou de ser god file/god
   controller):** a estrategia aprovada foi executada. O shell visual saiu
   para `ui/qml/shell/TopHeaderBar.qml`, `SideRail.qml` e `ShellLayout.qml`.
   O estado/orquestracao saiu para controllers/stores QML nao visuais:
   `workspace/WorkspaceController.qml`, `editor/EditorController.qml`,
   `project/ProjectTreeController.qml`, `jobs/JobsController.qml`,
   `runtime/RuntimeController.qml`, `search/SearchController.qml`,
   `assistant/AssistantController.qml` e `command/CommandDispatcher.qml`.
   Os handlers de `CoreClient` foram divididos em roteadores IPC por dominio:
   `ipc/WorkspaceEventRouter.qml`, `EditorEventRouter.qml`,
   `JobsEventRouter.qml`, `SearchEventRouter.qml` e
   `RuntimeEventRouter.qml`.

   `Main.qml` caiu para **336 linhas**. Ele nao tem mais `ListModel`, nao tem
   mais bloco `Connections { target: coreClient }`, `Shortcut`, `Timer`, helper
   de dominio ou componente visual pesado embutido; tambem nao guarda
   `openFiles`, `treeModel`, modelos de build/test/problems/search/run, timers
   de LSP, estado de terminal, command palette ou mensagens do assistente. O
   arquivo ficou como composition root: cria a janela, instancia `CoreClient`,
   controllers, roteadores e hosts de shell, e conecta sinais de alto nivel.

   **Decisao do usuario em 2026-07-06:** nao aceitar sobras pequenas como
   divida tecnica "adiavel". A fase de higiene arquitetural sem divida nova foi
   executada e ficou documentada em `docs/17-architecture-hygiene-plan.md`.
   Daqui para frente, novas features grandes so devem entrar mantendo os
   guardrails: `Main.qml` composition root, controllers/stores por dominio,
   roteadores IPC por dominio e `CoreClient` como fachada QML unica com
   implementacao interna fatiada quando crescer.

   **Onde olhar primeiro agora:**
   - `ui/qml/Main.qml`: composition root; deve permanecer pequeno e nao voltar
     a concentrar modelos/handlers de dominio.
   - `ui/qml/editor/EditorController.qml`: fachada do editor; documentos,
     texto e completion ficam nos subcontrollers do mesmo diretorio.
   - `ui/qml/project/ProjectTreeController.qml`: `treeModel`,
     expand/collapse, selecao, create/rename/delete e sincronizacao com abas.
   - `ui/qml/jobs/JobsController.qml`: saida de build, problemas
     build/quality/LSP, testes, summary e historico generico de jobs.
   - `ui/qml/runtime/RuntimeController.qml`: terminal integrado, saida de run,
     stdin e start/stop de processo.
   - `ui/qml/search/SearchController.qml`: busca em arquivos e Search
     Everywhere.
   - `ui/qml/ipc/*.qml`: roteadores dos eventos `CoreClient` por dominio.

   **Padrao de extracao daqui para frente:** componente visual recebe dados por
   `property` e emite `signal`; controller/store guarda modelo, estado, timers
   e pequenas decisoes de UI; roteador IPC recebe evento do `CoreClient` e chama
   o controller certo. Nenhum componente visual deve acessar filesystem ou
   chamar ferramenta externa. Todo novo `.qml` em subpasta precisa entrar em
   `ui/CMakeLists.txt` com `QT_RESOURCE_ALIAS`.

   **Verificacao apos cada aba extraida:** `cmake --build --preset
   dev-local` e `--preset dev-local-release` (ambos devem linkar),
   `clang-format --dry-run --Werror` no `.qml` novo se o projeto passar a
   formatar QML (hoje so C++ tem gate automatico — QML e revisado a olho),
   smoke offscreen (`QT_QPA_PLATFORM=offscreen`), e o comportamento visual
   idempotente (build/test/quality continuam iniciando, mostrando saida e
   finalizando via eventos; Problems continua recebendo as tres origens).

   **Specs de UI/UX sao inegociaveis mesmo neste refactor estrutural.** Esta
   tarefa e reorganizacao de arquivo (extrair QML para componentes), NAO
   redesign — mas qualquer decisao de layout, espacamento, cor, icone,
   copy ou comportamento visual que aparecer no caminho tem que seguir
   estritamente os `.md` de `docs/specs/`, nunca inventar por conta propria
   nem "aproveitar para melhorar visualmente". Specs relevantes para as abas
   desta tarefa: `docs/specs/KINEIN_VECTIS_LAYOUT_SYSTEM.md`,
   `docs/specs/KINEIN_VECTIS_UI_COMPONENTS_SYSTEM.md`,
   `docs/specs/KINEIN_VECTIS_VISUAL_SYSTEM_ICONS.md` e
   `docs/specs/KINEIN_VECTIS_PRODUCT_FLOWS_BUILD_RUN_DEBUG.md` (indice geral:
   `docs/specs/KINEIN_VECTIS_SPEC_INDEX.md`). Se o comportamento atual de
   `Main.qml` divergir do spec, extrair mantendo o comportamento atual (bug
   de UX vira tarefa separada, com o usuario ciente) — nao corrigir de
   passagem escondido dentro do refactor estrutural.

   **Nao fazer:** nao mudar o layout/visual escondido dentro de refactor
   estrutural; nao recolocar `ListModel`, timers, `Connections` ou switchs de
   dominio em `Main.qml`; nao criar acesso direto a filesystem/ferramentas em
   componente visual. Modelos de dominio agora vivem nos controllers
   (`JobsController`, `EditorController`, `ProjectTreeController`,
   `RuntimeController`, `SearchController`, etc.) e eventos IPC vivem nos
   roteadores `ui/qml/ipc/`.

   **Atualizacao CODEX 2026-07-06 (higiene arquitetural finalizada):** a fase
   sem divida nova foi executada e documentada em
   `docs/17-architecture-hygiene-plan.md`. Estado final validado:
   `Main.qml` ficou com **336 linhas**, sem `ListModel`, `Connections`,
   `Shortcut`, `Timer`, helper de dominio ou componente visual pesado
   embutido. O host visual central do workspace virou
   `ui/qml/shell/ShellWorkspaceHost.qml` (**248 linhas**, sem acesso direto a
   `CoreClient`). O `EditorController.qml` ficou com **318 linhas** e delega
   documentos, texto
   e completion para `EditorDocumentController.qml`,
   `EditorTextController.qml` e `EditorCompletionController.qml`. O
   `CoreClient` segue sendo a fachada QML unica, mas sua implementacao C++ foi
   dividida em `core_client_process.cpp`, `core_client_requests.cpp`,
   `core_client_dispatch.cpp`, `core_client_state.cpp` e
   `core_client_log.cpp`. `ui/CMakeLists.txt` foi sincronizado com os novos
   QML/C++ e tambem declara o prefixo QML usado pelo runtime, alem de tratar a
   politica Qt de `qmldir` extra quando disponivel. Validacao final:
   `cmake --build --preset dev-local`, smoke offscreen debug,
   `cmake --build --preset dev-local-release`, smoke offscreen release, smoke
   offscreen via `scripts/kinein-vectis` e `scripts/verificar.sh --rapido`
   verde.

   **Atualizacao CODEX 2026-07-06 (aba Jobs generica):** a UI passou a consumir
   `event.job.created/progress/output/finished` em uma aba dedicada "Jobs" no
   painel inferior, sem mudanca de contrato IPC nem backend novo. O
   `JobsController.qml` agora mantem `jobsModel`, `JobsEventRouter.qml`
   encaminha os sinais genericos de job, e o componente visual novo
   `ui/qml/panels/bottom/JobsPanel.qml` mostra titulo, status, risco nao baixo
   e ultima linha/progresso textual do job. `ui/CMakeLists.txt` foi atualizado
   com `QT_RESOURCE_ALIAS` para o novo QML. Validacao da fatia: build debug,
   smoke offscreen debug, build release, smoke offscreen release e smoke pelo
   launcher local verdes; `scripts/verificar.sh --rapido` tambem verde. O
   ruido restante segue restrito a qmlcache gerado pelo Qt/toolchain local.

   **Regra daqui para frente:** a fase nao deixa divida tecnica conhecida
   nessa frente. Nova feature deve manter o fluxo `QML visual ->
   controller/store -> CoreClient facade -> handler IPC interno -> Rust core`.
   Se algum arquivo passar dos limites de `docs/17-architecture-hygiene-plan.md`
   ou misturar renderizacao, estado e IPC, a feature so esta pronta depois do
   split.

   **Proxima sequencia recomendada apos este checkpoint/commit (2026-07-06):**
   1. Depois de trocar/reinstalar a distro, revalidar o ambiente local antes de
      codar: instalar dependencias de `docs/14-development-environment.md`,
      configurar presets locais se necessario (`cmake --preset dev-local` e
      `cmake --preset dev-local-release`), rodar `scripts/verificar.sh
      --rapido`, `cmake --build --preset dev-local`,
      `cargo build --release -p kinein-core`,
      `cmake --build --preset dev-local-release` e smoke offscreen pelo
      launcher `scripts/kinein-vectis`.
   2. Nao iniciar Git/AI/debug avancado antes de uma fatia UI concreta. A
      proxima entrega recomendada e **Project Health minimo e visivel**:
      primeiro usar dados ja existentes (`workspace.kind`, `tools.status`,
      `environment.scan`, estado de jobs/LSP) para um banner/painel discreto;
      so criar contrato novo (`project.health`) se a UI realmente precisar de
      dado que o core ainda nao expõe.
   3. Se `project.health` virar necessario, seguir o fluxo de
      `docs/ARCHITECTURE.md`: tipos em `kinein-protocol`, handler fino,
      servico de dominio no core, testes, docs/03 atualizado e UI por
      controller/roteador/componente visual. Operacao longa deve ser job.
   4. Depois do Project Health minimo, escolher a proxima fatia visivel entre:
      Toolchain/Environment Settings usando `environment.scan`; CMake/Cargo
      toolbar basica so quando houver contrato suficiente; ou Settings/Storage
      com schema se a UI precisar persistir escolhas.
5. `docs/BACKEND_TO_UI_UX_ROADMAP.md` continua sendo a ponte backend->UI: nao
   substitui `docs/specs/`, so evita que o backend avance sem mapear a
   experiencia visual futura. Atualizar os dois ao fim de cada entrega.

Decisoes anteriores (2026-07-03/04) ja cumpridas e resumidas: UI/UX
basica antes de features grandes; syntax highlighting; Fase 4 (build); Fase 5
(LSP: diagnosticos, go to definition, hover, completion, find usages, rename —
semantic tokens e code actions ainda pendentes); Java/Python fora do curto
prazo (pos-V1); IA usada via terminal (Claude/Codex/GPT), sem provider embutido
no MVP. Detalhe: git log e `docs/BACKEND_TO_UI_UX_ROADMAP.md`.

Ideia de produto pos-V1 (2026-07-04, nao bloqueia V1.0): "modos de compilador"
e "loja de funcoes" para C/C++ e Rust, com janela de opcoes visual e
JetBrains-like (nome, explicacao, impacto, risco, fonte, previa, reversao),
ordenando por confianca (ISO/Rust oficial primeiro). Ainda sem doc dedicado
apos a limpeza de `docs/archive/`; se for retomada, criar
`docs/17-compiler-modes-and-function-store.md` antes de implementar.

## Visao pos-V1.0

Depois da V1.0 o foco e polir a IDE continuamente para chegar o mais perto
possivel das IDEs JetBrains em analise, navegacao e refatoracao — mesmo
orquestrando ferramentas abertas (CMake, LSPs etc.). O objetivo explicito e
NAO terminar como "um VS Code": a régua de qualidade de navegacao/refactoring
e JetBrains. Isso reforca a Fase 5 (LSP) como investimento central: semantic
tokens, go-to-definition, find usages, rename via LSP, e futuramente acoes de
refatoracao proprias por cima do que os LSPs oferecem.

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
  tecnico atual" + "Prioridade imediata".
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
  via API num primeiro momento.
- LOGS: o log de tráfego IPC/acoes da IDE e ferramenta de desenvolvimento DA
  IDE, nao do usuario final — por isso a aba do painel inferior e "IDE", nao
  "Logs". "Problemas" mostra erros DO PROJETO (build, diagnosticos).
- ERROS DA IDE EM .TXT: qualquer erro/mau funcionamento da IDE e gravado com
  timestamp ISO em `~/.cache/kinein-vectis/logs/kinein-ui-erros.txt`
  (`CoreClient::appendErrorLog`, propriedade QML `errorLogFile`).

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

- `ui/src/core_client.h` e `ui/src/core_client*.cpp`: cliente IPC da UI. Nao
  criar outro cliente IPC paralelo; manter `CoreClient` como fachada QML unica
  e dividir so a implementacao interna por responsabilidade.
- `ui/qml/workspace/FolderPickerDialog.qml`: seletor proprio de workspace.
  Nao voltar para `QtQuick.Dialogs` e nao criar outro seletor de pasta
  duplicado.
- `ui/qml/Main.qml`: composition root da UI. Nao recolocar nele modelos,
  timers, handlers IPC ou blocos grandes de dominio se ja houver componente,
  controller ou roteador em `shell/`, `panels/bottom/`, `workspace/`,
  `assistant/`, `project/`, `command/`, `editor/`, `jobs/`, `runtime/`,
  `search/` ou `ipc/`.
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
- Nao recriar `docs/archive/`: foi removido de proposito em 2026-07-05 (ver
  `docs/15-engineering-debt-and-refactor.md`). Documento descontinuado vira
  resumo no doc numerado relevante e depois e apagado, nao guardado numa pasta
  de arquivo morto.

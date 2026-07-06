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
4. **PROXIMA TAREFA (handoff detalhado para quem pegar a sessao seguinte —
   FABLE ficou sem tokens em 2026-07-05; limite semanal so volta
   quarta-feira a noite; CODEX deve poder seguir so com o que esta escrito
   aqui, sem depender de memoria de conversa):** quebrar `Main.qml`
   (~4600 linhas) em componentes por dominio. Trabalho maior, fazer **uma aba
   por vez**, com gate verde entre cada extracao — nunca big-bang.

   **Onde olhar primeiro (linhas aproximadas nesta versao; podem ter mudado
   um pouco, procure pelos ids/comentarios citados, nao confie so no numero):**
   - Estado (`property ...`) do `root` (`id: root`, comeca linha 8): flags
     como `showBottomPanel`, `bottomTab` (linha 23) controlam qual aba do
     painel inferior esta visivel ("build", "tests", "problems", "tools",
     "logs", "terminal", "run", "search").
   - `ListModel`s compartilhados (linhas ~1024-1066): `buildOutputModel`,
     `problemsModel` (**compartilhado** entre build/quality/lsp diagnostics —
     nao dividir por dominio sem cuidado), `testModel`, `runModel`,
     `searchModel`, `everywhereModel`, `treeModel`, `openFiles`.
   - `Connections { target: coreClient ... }` (comeca linha ~1102): tem
     `onBuildStarted/onBuildOutput/onBuildDiagnostic/onBuildFinished`,
     `onTestCase/onTestFinished`, `onQualityDiagnostic/onQualityFinished` —
     e aqui que os eventos do `CoreClient` viram itens nos `ListModel`s
     acima via `root.appendBuildLine(...)` etc.
   - O painel inferior inteiro (tab bar + conteudo de todas as abas) fica
     num bloco unico dentro do layout principal, comeca por volta da linha
     3100 (`id: bottomPanel`) e vai ate perto da status bar (~linha 4060).
     Dentro dele, cada aba tem seu `ListView`/conteudo com
     `visible: root.bottomTab === "<nome>"` (ex.: `"build"`, `"tests"`,
     `"problems"`, `"tools"`, `"logs"`).
   - A status bar (`id: statusBar`, por volta da linha 4090) ja tem os
     controles de cancelar (× ao lado de "compilando.../testando...") —
     nao duplicar isso ao extrair os paineis.

   **Padrao de extracao a seguir (ja existe um precedente no arquivo —
   `ui/qml/FolderPickerDialog.qml`, instanciado em `Main.qml` por volta da
   linha 1072):** um componente QML em arquivo separado **nao ve os `id`s**
   de `Main.qml` automaticamente (escopo de id e por documento). O jeito
   certo, exatamente como `FolderPickerDialog` faz:
   1. O novo `.qml` declara `property`s para tudo que precisa vir de fora
      (ex.: `property CoreClient coreClient`, `property ListModel model`,
      `property bool active`) e `signal`s para o que precisa avisar o pai.
   2. `Main.qml` instancia o componente passando essas propriedades
      (`BuildPanel { coreClient: coreClient; model: buildOutputModel;
      visible: root.bottomTab === "build" }`) e conectando os sinais, do
      mesmo jeito que `onBrowseRequested`/`onOpenRequested` etc. sao
      conectados no `FolderPickerDialog` hoje.
   3. Todo novo arquivo `.qml` precisa ser adicionado em `QML_FILES` no
      `ui/CMakeLists.txt` (mesma lista onde `qml/FolderPickerDialog.qml` ja
      esta) — sem isso o tipo nao fica disponivel via `import KineinVectis`.

   **Ordem recomendada (do mais isolado ao mais acoplado):**
   1. Aba "logs"/IDE — so lista `coreClient.logLines`, sem modelo proprio nem
      logica de dominio. Bom primeiro corte para validar o padrao.
   2. Aba "tools"/Ferramentas — usa `root.toolsList` (populado por
      `onToolsListed`) e `coreClient.scanEnvironment()`/`detectTools()`;
      pouco acoplamento com outras abas.
   3. Abas "build"/"tests" — cada uma usa seu proprio `ListModel`
      (`buildOutputModel`/`testModel`) + os handlers de `Connections`
      correspondentes; extrair os dois juntos ou um de cada vez.
   4. Aba "problems" — cuidado: e alimentada por build, quality **e** LSP;
      so extrair depois que build/quality ja estiverem em componentes
      separados, para nao quebrar nenhuma das tres origens.
   5. NAO mexer em "terminal"/"run"/"search" nesta rodada a menos que sobre
      tempo — nao fazem parte do pedido atual (jobs/build/quality) e tem
      logica propria (PTY, stdin) que merece atencao dedicada.

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

   **Nao fazer:** nao mudar o layout/visual (isso e refactor de estrutura,
   nao redesign); nao duplicar
   `problemsModel`/`buildOutputModel`/`testModel` — eles continuam vivendo
   em `Main.qml` e sendo passados por propriedade; nao mover a logica de
   IPC para os componentes novos (isso e do `CoreClient`, so o binding fica
   no QML).
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
- Nao recriar `docs/archive/`: foi removido de proposito em 2026-07-05 (ver
  `docs/15-engineering-debt-and-refactor.md`). Documento descontinuado vira
  resumo no doc numerado relevante e depois e apagado, nao guardado numa pasta
  de arquivo morto.

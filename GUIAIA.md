# GUIAIA — mapa operacional do Kinein Vectis

> **Status:** ativo e interno — atualizar junto com mudanças de arquitetura.
> **Função:** dizer rapidamente **onde buscar conhecimento**, **quais módulos se
> conectam** e **quais arquivos normalmente mudam juntos**.
> **Não substitui:** `ContextoIA.md`, `docs/ARCHITECTURE.md`, specs ou contrato
> IPC. Este arquivo é o roteador prático entre essas fontes e o código real.
> **Publicação:** não entra em nenhuma cópia entregue a terceiros; entre
> Markdown, essa cópia leva somente `README.md`, `MANUAL.md` e `Tutorial.md`.

## 1. Entrada rápida para uma IA ou pessoa nova

Leia nesta ordem antes de alterar código:

1. `ContextoIA.md` — estado real, decisões vigentes e restrições do produto.
2. Este `GUIAIA.md` — descubra o domínio e os arquivos conectados.
3. `docs/ARCHITECTURE.md` — camadas, fronteiras e regra de split obrigatória.
4. `PONTO_ATUAL.md` — próxima tarefa executável e ordem vigente.
5. O documento específico indicado nas tabelas deste guia.
6. O código real do fluxo completo antes de propor arquivo ou subsistema novo.

Para qualquer mudança, responda primeiro:

```text
É UI, estado visual, contrato IPC, core, serviço, job, integração externa,
persistência, tooling, documentação ou packaging?
Já existe um domínio que é dono dessa responsabilidade?
Quais consumidores e testes precisam mudar junto?
```

Se houver conflito, a precedência é:

```text
ContextoIA.md + código real
        ↓
docs/specs/ (visão-alvo e UI/UX não negociável)
        ↓
docs numerados (contrato/estado implementado)
        ↓
PONTO_ATUAL.md (ordem de execução)
        ↓
GUIAIA.md (mapa; nunca sobrepõe as fontes acima)
```

## 2. Meta operacional e sequência explícita

### TR0 — aceitar a rodada visual atual

- Validar menus, manual, terminal, autocomplete, KV Context e painéis.
- Corrigir regressões sem redesenhar a UI aprovada.
- Fonte: `PONTO_ATUAL.md`, seção de validação ao vivo.

### TR1 — substituir o VS Code no uso diário

Critério objetivo: desenvolver projetos C/C++ e Rust por uma semana sem abrir
VS Code, Sublime ou outro editor para compensar uma ausência da Kinein.

Prioridades:

1. self-hosting: abrir e desenvolver a própria Kinein dentro da Kinein;
2. projeto híbrido Cargo + CMake + Qt/QML sem perder um dos lados;
3. medir abertura, primeira sugestão, indexação, memória e latência de digitação;
4. corrigir apenas os motivos concretos que forçarem saída para outra IDE.

Workspaces recentes e recuperação de sessão previsível foram entregues na
fatia A1 do protocolo 0.53.0.

### TR2 — ficar imediatamente abaixo do CLion

Critério objetivo: a Kinein conhece targets, contextos e toolchains do projeto,
oferece build/run/debug coerentes e não exige terminal ou CLion para descobrir
como um arquivo é compilado.

Ordem arquitetural:

1. CMake File API completa + Cargo Metadata;
2. Unified Project Graph + Context Matrix do KSWE;
3. targets, perfis e toolchains como entidades de primeira classe;
4. scheduler LSP por documento/contexto, cancelamento e backpressure;
5. Symbol Broker e Diagnostic Broker, sem duplicar clangd/rust-analyzer;
6. painel explicável de Effective Compile Context;
7. debugger com watches, variáveis, pilha e pretty-printers confiáveis;
8. split editor, multicursor e EditorConfig;
9. testes prolongados em projetos reais, inclusive a própria Kinein.

### TR3 — buscar paridade de profundidade com CLion

- múltiplos contextos C/C++ e feature sets Rust;
- refatorações e navegação global mais profundas;
- análise estática em lote e diagnósticos correlacionados;
- flash, serial, QEMU, OpenOCD/pyOCD e debug remoto;
- cache/indexação persistente, observável e adaptado a recursos;
- integração equivalente pela qualidade da orquestração, nunca por criar um
  compilador, LSP, build system ou debugger próprio.

Quando o usuário disser **“estou no Kinein”**, iniciar o protocolo de
self-hosting: registrar cada saída para outra ferramenta, o motivo exato, o
projeto/arquivo, a ação que faltou, o impacto e uma reprodução mínima. Esses
dados passam a ordenar o backlog antes de confortos hipotéticos. A frase pode
ser o primeiro e único conteúdo de uma nova sessão: nesse caso, confirmar a
ativação, ler o handoff da seção 0 de `PONTO_ATUAL.md` e aguardar o primeiro
feedback real. Se não houver bloqueio e o usuário pedir continuidade do
roadmap, seguir a fila viva de `PONTO_ATUAL.md`; após A1, a próxima fatia
planejada é A2 — projeto híbrido e self-hosting.

Feedback de amigos/testadores usa o mesmo funil, sempre identificado pela
origem, distro e versão do artefato. Prioridade: perda de dados/segurança/crash
→ bloqueio diário → regressão funcional → conforto/feature. Reproduzir antes de
alterar arquitetura ou transformar uma impressão isolada em funcionalidade.

## 3. Onde buscar conhecimento por pergunta

| Pergunta | Leia primeiro | Complemento |
| --- | --- | --- |
| Qual é o estado real agora? | `ContextoIA.md` | `PONTO_ATUAL.md` |
| Qual é a próxima tarefa? | `PONTO_ATUAL.md` | `docs/21-long-horizon-roadmap.md` |
| Em qual camada colocar código? | `docs/ARCHITECTURE.md` | `docs/02-repository-structure.md` |
| Como UI e core conversam? | `docs/03-ipc-protocol.md` | `crates/kinein-protocol/src/` |
| Qual é a UX/layout obrigatória? | `docs/specs/KINEIN_VECTIS_SPEC_INDEX.md` | specs de Layout, UI Components e Visual System |
| Editor, completion e navegação | spec `EDITOR_LANGUAGE_INTELLIGENCE` | `docs/25-syntax-tree-semantic-foundation.md` |
| Tree-sitter e fallback local | spec `TREE_SITTER_EDITOR_LAYER` | `docs/25-syntax-tree-semantic-foundation.md` + ADR-0002 |
| Projeto profundo/KSWE | `KINEIN_VECTIS_DEEP_SEMANTIC_ENGINE_CPP_RUST_WORKFLOW.md` | seção “simbiose” de `docs/21-long-horizon-roadmap.md` |
| Build, Run, Test e Debug | spec `PRODUCT_FLOWS_BUILD_RUN_DEBUG` | `docs/22-compilacao-c-cpp-rust.md` |
| Terminal e KV Context | spec `AI_CLI_BRIDGE_EXTERNAL_TERMINAL` | `docs/24-paridade-e-fundacao.md` |
| Dados, drafts e escrita segura | `docs/23-rede-de-seguranca.md` | ADR-0001 + `docs/16-hidden-risks-checklist.md` |
| Adotar ferramenta open source | `KINEIN_VECTIS_OPEN_PLUGIN_ADAPTATION_ROADMAP.md` | `docs/tooling/OPEN_COMPONENT_REGISTRY.json` |
| Strict mode e gates | `docs/06-strict-mode.md` | `docs/COMANDOS_BUILD_VERIFICACAO.md` |
| Daily driver e dogfooding | `docs/18-daily-driver-plan.md` | `docs/21-long-horizon-roadmap.md` |
| Convergência visual | `docs/20-ui-spec-convergence-plan.md` | specs visuais |
| Instalação para contribuir | `docs/14-development-environment.md` | `COMO_EXECUTAR.md` |
| Uso da IDE por testador | `MANUAL.md` | `README.md` |
| AppImage, instalação e atualização | `Tutorial.md` | seção 8 deste guia |
| Entrega sanitizada do código | M7 de `docs/21-long-horizon-roadmap.md` | `Tutorial.md` + item correspondente de `PONTO_ATUAL.md` |

### 3.1 Catálogo dos documentos de estado/engenharia

| Documento | Conhecimento que ele possui |
| --- | --- |
| `README.md` | apresentação pública e estado resumido |
| `MANUAL.md` | operação da IDE, recursos e atalhos do usuário/testador |
| `Tutorial.md` | distribuição, checksum, instalação, atualização, packaging e entrega externa do código |
| `COMO_EXECUTAR.md` | build, launcher e dependências para executar pelo checkout |
| `AGENTS.md` | regras obrigatórias para agentes de terminal |
| `ContextoIA.md` | decisões vigentes, histórico operacional útil e estado real |
| `PONTO_ATUAL.md` | fila explícita e critérios imediatos de aceite |
| `GUIAIA.md` | este mapa entre conhecimento, módulo, arquivo e gate |
| `KINEIN_VECTIS_DEEP_SEMANTIC_ENGINE_CPP_RUST_WORKFLOW.md` | desenho profundo do KSWE, C++/Rust, scheduler, brokers e contextos |
| `KINEIN_VECTIS_OPEN_PLUGIN_ADAPTATION_ROADMAP.md` | como avaliar/adotar componentes externos sem extensão improvisada |
| `docs/ARCHITECTURE.md` | camadas, ownership, limites de arquivo e crescimento modular |
| `docs/02-repository-structure.md` | estrutura de crates/pastas e convenções de nomes |
| `docs/03-ipc-protocol.md` | requests, responses e eventos realmente implementados |
| `docs/06-strict-mode.md` | rigor Rust, C++, QML e linguagens suportadas |
| `docs/14-development-environment.md` | preparação da máquina de desenvolvimento |
| `docs/15-engineering-debt-and-refactor.md` | dívidas conhecidas e regras para não recriar monólitos |
| `docs/16-hidden-risks-checklist.md` | segurança, dados, segredos, a11y, observabilidade e packaging |
| `docs/17-architecture-hygiene-plan.md` | higiene arquitetural e concentrações que devem permanecer fechadas |
| `docs/18-daily-driver-plan.md` | marcos M/E/T, dogfooding e paridade diária |
| `docs/19-architecture-tradeoffs.md` | razões e trade-offs por trás das decisões |
| `docs/20-ui-spec-convergence-plan.md` | execução vinculante da UI/UX C0–C6 |
| `docs/21-long-horizon-roadmap.md` | KSWE, M4–M7, distribuição e continuidade longa |
| `docs/22-compilacao-c-cpp-rust.md` | comandos/ferramentas reais orquestrados para C/C++/Rust |
| `docs/23-rede-de-seguranca.md` | atomic save, drafts SQLite e recuperação contra perda |
| `docs/24-paridade-e-fundacao.md` | fases D1–D4: completion, terminal, Tree-sitter e remake |
| `docs/25-syntax-tree-semantic-foundation.md` | contrato da camada sintática, versões e workspace edits |
| `docs/BACKEND_TO_UI_UX_ROADMAP.md` | ponte entre capacidade de backend e experiência visual |
| `docs/COMANDOS_BUILD_VERIFICACAO.md` | comandos oficiais do gate |
| `docs/adr/*` | por que uma decisão externa/estrutural foi adotada |
| `docs/tooling/OPEN_COMPONENT_REGISTRY.json` | pins, licenças e auditoria de componentes adotados |

### 3.2 Roteador das specs de visão-alvo

Comece sempre por `docs/specs/KINEIN_VECTIS_SPEC_INDEX.md`. Depois leia apenas
o grupo da tarefa:

| Grupo | Specs |
| --- | --- |
| Visual e shell | `VISUAL_SYSTEM_ICONS`, `LAYOUT_SYSTEM`, `UI_COMPONENTS_SYSTEM` |
| Editor | `EDITOR_LANGUAGE_INTELLIGENCE`, `TREE_SITTER_EDITOR_LAYER` |
| Build/execução | `PRODUCT_FLOWS_BUILD_RUN_DEBUG` |
| Embarcados | `EMBEDDED_TARGETS_FLASH_SERIAL_QEMU` |
| IA externa | `AI_CLI_BRIDGE_EXTERNAL_TERMINAL`; `KV_CONTEXT_AI_ASSISTANCE` é histórico/superado onde propõe chat embutido |
| Setup | `ONBOARDING_PROJECT_WIZARD_SETTINGS`, `ONBOARDING_SETUP_OPTIMIZATION_LAYER` |
| Arquitetura | `INTERNAL_ARCHITECTURE_CORE_IPC_JOBS` |
| Performance | `RESOURCE_ON_DEMAND_PERFORMANCE_STRATEGY`, `RESOURCE_ON_DEMAND_INTELLIGENCE_REFACTORING_STRATEGY` |
| Configuration Actions | `SCOPED_CONFIGURATION_ACTIONS_DOC_LINKS`, `DUAL_WORKFLOW_CONFIGURATION_ACTIONS` |
| Planejamento macro | `FINALIZATION_MVP_ROADMAP_POLISH_CHECKLIST`, `IMPLEMENTATION_PLAN_UI_UX_ARCH_PERFORMANCE`, `IMPLEMENTATION_TASKS` |

O pacote `docs/KINEIN_VECTIS_ICONS_COMPLETE/` só entra em tarefa de iconografia
ou contrato de ícones QML; não é leitura padrão de core/IPC.

## 4. Arquitetura e direção das dependências

```text
Qt/QML visual
    ↓ signal/property
Controller/store QML
    ↓ método da fachada
CoreClient C++
    ↓ JSON-RPC tipado
kinein-protocol
    ↓ parse/dispatch
handler fino do kinein-core
    ↓ API interna
serviço de domínio
    ↓ Job quando for longo
ferramenta externa madura
    ↓ evento tipado
router IPC QML → controller → visual
```

Regra curta:

> A UI apresenta; o Core decide; o serviço executa; Jobs acompanham; Events
> notificam; ferramentas externas continuam sendo a autoridade do seu domínio.

### Composition roots e fachadas que não devem virar depósitos

- `ui/qml/Main.qml`: somente composição e ligações de alto nível.
- `ui/src/core_client*.cpp`: uma fachada QML, dividida internamente; não criar
  um segundo cliente.
- `crates/kinein-core/src/lib.rs`: estado e dispatch; sem lógica de domínio.
- `crates/kinein-core/src/handlers/*.rs`: parse + validação + delegação.
- `crates/kinein-protocol/src/lib.rs`: reexports; tipos ficam por domínio.

## 5. Mapa de módulos que se comunicam

### 5.1 Workspace, Start Screen e projeto

```text
ui/qml/workspace/* + ui/qml/project/*
    ↕ ui/qml/ipc/WorkspaceEventRouter.qml
ui/src/core_client_requests.cpp + core_client_dispatch.cpp
    ↕ crates/kinein-protocol/src/workspace.rs
crates/kinein-core/src/handlers/workspace.rs
    → crates/kinein-core/src/workspace/*
    → crates/kinein-core/src/fswatch.rs
```

- Testes: `crates/kinein-core/src/tests/workspace.rs` e harnesses QML.
- Contrato: workspace em `docs/03-ipc-protocol.md`.
- UX-alvo: specs de Onboarding/Project Wizard/Setup Optimization.
- Workspaces recentes vivem em `workspace/recent.rs`, passam pelos métodos
  `workspace.recent.*`, `RecentWorkspacesController.qml` e
  `RecentWorkspacesCard.qml`; o core fornece o snapshot ordenado e a UI abre
  pelo `workspace.open`, sem explorador ou acesso ao filesystem paralelo.

### 5.2 Arquivos, árvore, buffers, drafts e save seguro

```text
ui/qml/project/ProjectTreeController.qml
ui/qml/editor/EditorController.qml
    ├─ EditorDocumentController.qml
    ├─ EditorTextController.qml
    └─ EditorExternalChangeBanner.qml
ui/qml/ipc/EditorEventRouter.qml + WorkspaceEventRouter.qml
    ↕ crates/kinein-protocol/src/{fs,draft,format}.rs
crates/kinein-core/src/handlers/{fs,draft,format}.rs
    → crates/kinein-core/src/{fsops,db}/* + fswatch.rs + format.rs
```

- Testes: `crates/kinein-core/src/tests/{fs,format}.rs` e
  `scripts/qml-harness/tst_external_change.qml`.
- Segurança: `docs/23-rede-de-seguranca.md`, ADR-0001 e
  `docs/16-hidden-risks-checklist.md`.
- Nunca acessar filesystem do workspace diretamente pela UI.

### 5.3 Editor, Tree-sitter, LSP e diagnósticos

```text
ui/qml/editor/EditorController.qml
    ├─ EditorCompletionController.qml
    ├─ EditorOutlineController.qml / EditorOutlinePanel.qml
    ├─ EditorHoverPopup.qml / EditorUsagesPopup.qml
    └─ EditorWorkspaceEditPreviewDialog.qml
ui/qml/diagnostics/DiagnosticsController.qml
ui/qml/ipc/EditorEventRouter.qml
    ↕ crates/kinein-protocol/src/{lsp,syntax,diagnostic}.rs
crates/kinein-core/src/handlers/{lsp,syntax}.rs
    ├─ crates/kinein-core/src/lsp/{manager,server,framing,parse,edit,transaction,uri}.rs
    └─ crates/kinein-core/src/lang/{registry,service,positions,outline,folding}.rs
```

- Testes: `crates/kinein-core/src/tests/{lsp,syntax}.rs` e
  `scripts/qml-harness/{tst_completion,tst_outline}.qml`.
- Fontes: spec Editor/Language Intelligence, spec Tree-sitter,
  `docs/25-syntax-tree-semantic-foundation.md` e especificação KSWE.
- Tree-sitter entrega estrutura/fallback; clangd e rust-analyzer são a
  autoridade semântica. Não misturar ou duplicar esses papéis.

### 5.4 CMake, Cargo, build, qualidade, testes, tools e jobs

```text
ui/qml/shell/{RunConfigMenu,RunConfigDialog}.qml
ui/qml/panels/bottom/{Build,Tests,Tools,Jobs,Problems}Panel.qml
ui/qml/jobs/JobsController.qml + routers IPC
    ↕ crates/kinein-protocol/src/{cmake,cargo,build,job,tools,runconfig}.rs
crates/kinein-core/src/handlers/{cmake,cargo,build,jobs,runconfig}.rs
    → crates/kinein-core/src/{cmake,cargo,build,test,tools,runconfig}.rs
    → crates/kinein-core/src/jobs/* + process.rs
    → CMake, Ninja, Cargo, compilador, tidy ou Clippy externos
```

- Exceção real a não ampliar: `tools.detect`, `tools.status` e
  `environment.scan` ainda entram pelo dispatch de
  `crates/kinein-core/src/lib.rs`;
  a lógica de detecção vive em `tools.rs` e o scan usa Jobs. Se esse fluxo
  crescer, extrair `handlers/tools.rs` antes de adicionar mais casos ao `lib.rs`.
- Testes: `crates/kinein-core/src/tests/{cmake,cargo,build,runners,jobs,tools,runconfig}.rs`.
- Fontes: spec Build/Run/Debug, `docs/22-compilacao-c-cpp-rust.md`,
  `docs/18-daily-driver-plan.md` e `docs/21-long-horizon-roadmap.md`.
- KSWE deve reutilizar estes serviços; não criar outro executor de build.

### 5.5 Run e Debug

```text
ui/qml/runtime/RuntimeController.qml
ui/qml/debug/DebugController.qml
ui/qml/panels/bottom/{Run,Debug}Panel.qml
ui/qml/ipc/{Runtime,Debug}EventRouter.qml
    ↕ crates/kinein-protocol/src/{run,debug}.rs
crates/kinein-core/src/handlers/{run,debug}.rs
    → crates/kinein-core/src/run.rs
    → crates/kinein-core/src/dap/{target,session}.rs + jobs/* + process.rs
    → executável real + lldb-dap/GDB adapter
```

- Testes: `crates/kinein-core/src/tests/{run,debug}.rs`.
- Fontes: spec Build/Run/Debug e partes Debug da especificação KSWE.

### 5.6 Terminal e KV Context

```text
ui/qml/panels/bottom/TerminalPanel.qml
    ├─ TerminalViewport.qml
    ├─ TerminalSelectionController.qml
    ├─ TerminalInputController.qml
    └─ TerminalScrollController.qml
ui/qml/assistant/{AssistantPanel,AssistantController}.qml
    └─ AssistantTerminalHeader.qml
ui/qml/ipc/AiBridgeEventRouter.qml
    ↕ crates/kinein-protocol/src/{terminal,ai}.rs
crates/kinein-core/src/handlers/{terminal,ai}.rs
    → crates/kinein-core/src/terminal.rs (TerminalManager, PTY e grid VT100)
    → claude/codex instalados e escolhidos pelo usuário
```

- Testes: testes de terminal/AI, `tst_multi_terminal.qml`,
  `tst_assistant_layout.qml`, `tst_terminal_input.qml`,
  `tst_terminal_scroll.qml`, `tst_terminal_selection.qml` e
  `scripts/sonda_scrollback.py`.
- Fonte: spec AI CLI Bridge + `docs/24-paridade-e-fundacao.md`.
- KV Context reutiliza o TerminalManager; não criar terminal ou chat paralelo.
- Codex usa o modo inline oficial; o bridge preserva o transcript contra o
  erase-scrollback ainda emitido pela CLI. Seletor compacto e sessão terminal
  livre/persistida/maximizável são estados diferentes da mesma superfície,
  nunca um renderer de conversa da IDE. `Project` só é ocultado na maximização
  explícita, não pela mera existência da sessão.

### 5.7 Git

```text
ui/qml/git/* + ui/qml/panels/bottom/GitPanel.qml
ui/qml/ipc/GitEventRouter.qml
    ↕ crates/kinein-protocol/src/git.rs
crates/kinein-core/src/handlers/git.rs
    → crates/kinein-core/src/git/{operations,parse}.rs → binário git
```

- Testes: `crates/kinein-core/src/tests/git.rs` com repositórios temporários.
- Operações longas/remotas são Jobs; a UI não executa Git diretamente.

### 5.8 Settings, configuração e persistência

```text
ui/qml/settings/* + ui/qml/ipc/SettingsEventRouter.qml
    ↕ crates/kinein-protocol/src/settings.rs
crates/kinein-core/src/handlers/settings.rs
    → crates/kinein-core/src/settings.rs → crates/kinein-config
                                      ↘ storage XDG/workspace
crates/kinein-core/src/db/* → drafts SQLite e persistência apropriada
```

- Testes: `crates/kinein-core/src/tests/settings.rs` e testes de
  `crates/kinein-config`.
- Schema: `schemas/settings.schema.json`.
- Não criar formato novo sem schema, migração e documentação.

### 5.9 Shell, menus, overlays, componentes e documentação interna

```text
ui/qml/Main.qml
    ├─ shell/* (layout, header, menus, overlays, status)
    ├─ components/* (ícones, botões, tooltips)
    └─ ui/src/{documentation,clipboard,editor_highlighter}.*
```

- UX-alvo: specs Layout, UI Components e Visual System.
- Plano de convergência: `docs/20-ui-spec-convergence-plan.md`.
- O visualizador do manual é recurso read-only da própria aplicação e não
  acessa workspace; mudanças de negócio continuam proibidas na UI.

### 5.10 CLI, schemas, templates e tooling

- `crates/kinein-cli`: cliente fino do protocolo; não replica core.
- `schemas/`: formatos persistidos/contratos documentados.
- `templates/`: scaffolds controlados pelo core.
- `cmake/`: políticas estritas da própria UI.
- `scripts/`: gates, sondas, ambiente, launcher e futuro packaging.
- `docs/tooling/` + `docs/adr/`: auditoria de dependências e decisões.

## 6. Receitas: o que normalmente muda junto

### Novo comando IPC

1. `crates/kinein-protocol/src/<dominio>.rs`;
2. `crates/kinein-core/src/handlers/<dominio>.rs`;
3. serviço real em `crates/kinein-core/src/<dominio>`;
4. unitários + `crates/kinein-core/src/tests/<dominio>.rs`;
5. método/dispatch no `CoreClient` se a UI consumir;
6. router/controller/visual QML;
7. `docs/03-ipc-protocol.md`, schema aplicável e `ContextoIA.md`.

### Mudança puramente visual

1. localizar a spec visual;
2. editar componente burro em `ui/qml`;
3. preservar signals/properties existentes;
4. adicionar harness se houver lógica de estado;
5. qmllint + build + inspeção em tela real.

Não criar RPC para buscar dentro do buffer já aberto e não mover lógica de
negócio para QML só para evitar um contrato IPC.

### Operação longa

1. contrato tipado;
2. handler inicia Job e responde imediatamente com `jobId`;
3. serviço usa `JobContext`, cancelamento e eventos;
4. painéis Jobs/Problems/Build recebem eventos;
5. teste cancelamento, shutdown e ausência de processo órfão.

### Nova ferramenta externa

1. provar que é madura, aberta e necessária;
2. seguir o roadmap de adaptação open source;
3. registrar pin, licença e modo de integração no registry;
4. ADR quando a decisão atravessar arquitetura ou segurança;
5. core orquestra; UI nunca chama o binário diretamente.

### Nova setting ou persistência

1. tipo/configuração no domínio correto;
2. schema e migração/compatibilidade;
3. protocolo e storage no core;
4. controller/Settings UI;
5. testes de default, override, roundtrip e schema desconhecido.

### Mudança no KSWE/projeto profundo

1. desenhar uma fatia pequena na especificação KSWE/roadmap;
2. escolher uma fonte autoritativa (File API, Cargo Metadata etc.);
3. normalizar no Project Graph sem duplicar executor existente;
4. orçamento de CPU/RAM/latência e cancelamento antes do código;
5. protocolo de snapshot/eventos versionado;
6. teste com projeto fixture e depois com projeto real híbrido.

## 7. Gates por tipo de mudança

Gate integral antes de release/checkpoint:

```bash
bash scripts/verificar.sh
```

Alvos complementares:

| Mudança | Prova mínima adicional |
| --- | --- |
| Controller QML | `bash scripts/verificar-qml-logica.sh` + novo caso de regressão |
| Terminal | `cargo build -p kinein-core` + `python3 scripts/sonda_scrollback.py` |
| LSP/KSWE | teste de core + fixture real + latência/cancelamento |
| Filesystem/save | conflito externo + rollback + crash/draft |
| Build/Run/Debug | processo real, cancelamento, output e ausência de órfão |
| UI/layout | build Debug/Release + teste offscreen + aceite em tela real |
| Packaging | execução numa máquina/VM limpa sem Rust/Qt de desenvolvimento |

## 8. Distribuição prática para amigos e colegas Linux

### Caminho implementado: AppImage portátil de teste

O método mais simples para um testador é receber **um AppImage + SHA256**. Ele
não precisa clonar o repositório nem instalar Rust, CMake de desenvolvimento ou
Qt para abrir a IDE. O AppImage inclui:

- `kinein-vectis` (UI);
- `kinein-core`;
- bibliotecas/runtime Qt e plugins QML necessários;
- ícone, `.desktop`, licença e `MANUAL.md` empacotado;
- launcher que aponta a UI para o core do próprio bundle.

Compiladores, LSPs, CMake/Ninja e debugadores dos projetos continuam externos.
Na primeira abertura, Project Health deve explicar o que falta conforme a
linguagem usada.

Fluxo reproduzível atual:

```bash
bash scripts/empacotar-appimage-portatil.sh
bash scripts/testar-appimage.sh
bash scripts/testar-appimage-portatil.sh
```

O resultado fica em `dist/Kinein-Vectis-<versão>-x86_64.AppImage`, acompanhado
de `SHA256SUMS` e de um `.AppImage.sha256` específico. Para um testador, enviar
o AppImage e o checksum específico juntos; o procedimento completo fica em
`Tutorial.md`. A receita usa `linuxdeploy` + `linuxdeploy-plugin-qt`, builder
Debian 12 fixado e ferramentas auditadas no registry. O smoke portátil executa
sem rede em um Debian mínimo que não contém Qt, Rust, CMake ou compiladores.

Baseline honesto: Linux x86_64 com glibc 2.36 ou posterior e a pilha gráfica/
fontes normal de uma instalação desktop. Suportar distribuições anteriores ao
baseline exige reconstruir Qt 6.4+ sobre uma base mais antiga e repetir a
matriz. Decisão, pins e rollback: `docs/adr/ADR-0003-linuxdeploy-appimage-packaging.md`.

### Alternativa imediata para grupo 100% Arch/CachyOS

Gerar um pacote local `.pkg.tar.zst` por PKGBUILD é mais natural e menor, mas
serve apenas à família Arch. Não é necessário publicar no AUR para compartilhar
o arquivo entre testadores confiáveis. O AUR público só entra depois do espelho
público e da política de fontes estar pronta.

### Para colaboradores que vão compilar a IDE

Conceder acesso ao repositório privado e usar:

```bash
bash scripts/instalar-ambiente.sh
bash scripts/verificar.sh
bash scripts/instalar-atalho.sh
```

Esse fluxo é adequado para contribuidores, não para testadores de produto.

## 9. Política de entrega externa do código

- O repositório privado continua sendo a fonte completa.
- Qualquer cópia para terceiros é gerada por allowlist em outra árvore; um
  eventual espelho é separado e nunca muda a visibilidade da fonte.
- A cópia leva o código do projeto e, entre Markdown, somente `README.md`,
  `MANUAL.md` e `Tutorial.md`.
- `GUIAIA.md`, `ContextoIA.md`, `PONTO_ATUAL.md`, `AGENTS.md`, `docs/`,
  `prompts/`, specs, roadmaps e notas internas não entram.
- `.git/` e o histórico privado não entram; eventual espelho começa com
  histórico próprio da árvore sanitizada.
- O exportador precisa de dry-run, recusa de Markdown extra, auditoria de
  segredos e lista final verificável antes do push.

## 10. Como manter este mapa vivo

Atualize `GUIAIA.md` quando ocorrer qualquer um destes eventos:

- módulo, crate, domínio, router ou controller criado/renomeado/removido;
- contrato IPC ou direção de dependência alterada;
- ferramenta externa ou ADR adicionada;
- gate/sonda novo;
- etapa de produto concluída ou reordenada;
- método de packaging/publicação alterado.

Checklist de fechamento de uma fatia:

```text
[ ] ContextoIA.md representa o estado real?
[ ] PONTO_ATUAL.md aponta a próxima ação, não trabalho já encerrado?
[ ] GUIAIA.md ainda leva ao domínio e aos arquivos corretos?
[ ] docs/03/schema mudaram se o contrato mudou?
[ ] MANUAL.md mudou se o comportamento do usuário mudou?
[ ] Tutorial.md mudou se distribuição, instalação ou packaging mudaram?
[ ] O gate relevante e uma prova do gesto real ficaram verdes?
```

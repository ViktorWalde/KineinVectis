# GUIAIA — mapa operacional do Kinein Vectis

> **Status:** ativo e interno — atualizar junto com mudanças de arquitetura.
> **Função:** dizer rapidamente **onde buscar conhecimento**, **quais módulos se
> conectam** e **quais arquivos normalmente mudam juntos**.
> **Não substitui:** `ContextoIA.md`, `docs/arquitetura/ARCHITECTURE.md`, specs ou contrato
> IPC. Este arquivo é o roteador prático entre essas fontes e o código real.
> **Publicação:** não entra em nenhuma cópia entregue a terceiros; entre
> Markdown, essa cópia leva somente `README.md`, `MANUAL.md` e `Tutorial.md`.

## 1. Entrada rápida para uma IA ou pessoa nova

Leia nesta ordem antes de alterar código:

1. `ContextoIA.md` — estado real, decisões vigentes e restrições do produto.
2. Este `GUIAIA.md` — descubra o domínio e os arquivos conectados.
3. `docs/arquitetura/ARCHITECTURE.md` — **LEITURA OBRIGATÓRIA, inteiro**: camadas,
   fronteiras, regra de split e caminho de crescimento. É contrato, não consulta.
   Verificado por catraca (`scripts/verificar-arquitetura.sh`). Antes de propor
   arquitetura nova, MEDIR: duas vezes seguidas a resposta certa foi "o projeto
   já tem arquitetura, é boa, e não era aplicada" (ver §1.1 de lá).

   **Três âncoras, sempre juntas** (ARCHITECTURE.md §1.3): o documento + código
   medido (contra alucinação de arquitetura); IDEs open source consolidadas com
   revisão citada (contra dogmatismo); e a **documentação oficial da
   linguagem/tecnologia** — Rust, Qt/QML, C++, CMake, POSIX — contra API
   imaginada. Comportamento de API se consulta na fonte, com versão; não se
   deduz do nome nem se lembra de cor.
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

### 1.1 Referência profissional antes de funcionalidade de IDE

É obrigatório seguir a seção 2.1 de
`docs/roadmaps/KINEIN_VECTIS_OPEN_PLUGIN_ADAPTATION_ROADMAP.md` antes de criar ou alterar
uma funcionalidade de IDE. Use a fonte oficial e atual pertinente — Code OSS,
IntelliJ IDEA Community, Zed, Lapce e/ou Apache NetBeans — para estudar
comportamento, invariantes, falhas, segurança, concorrência e testes. Registre
repositório/revisão, subsistema estudado, licença/modo, lições e adaptação no
documento do domínio.

```text
referência profissional atual
        ↓ extrair invariantes e modos de falha, sem copiar código
design próprio nas specs e camadas da Kinein
        ↓
Qt/QML → CoreClient → protocolo tipado → Rust Core → serviço/job/tool
        ↓
testes próprios + comparação comportamental
```

“VS Code” nesse contexto significa o repositório público Code OSS
`microsoft/vscode`, não a distribuição proprietária, Marketplace ou extensões
fechadas. Não copiar função pronta, não fazer tradução mecânica e não adotar
uma revisão antiga apenas por conveniência. Para o terminal, o Code OSS é
referência do backend e do ciclo de sessão; a implementação continua nativa em
Rust + IPC + Qt/QML, conforme a seção 5.6.

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
fatia A1 do protocolo 0.53.0. A2 foi entregue no protocolo 0.55.0: um único
snapshot expõe Cargo e CMake simultaneamente e as ações de build/teste podem
selecionar o sistema sem perder o `workspace.kind` compatível.

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
roadmap, seguir a fila viva de `PONTO_ATUAL.md`; depois de A1/A2, a próxima
fatia funcional é A3 — responsividade medida, salvo regressão concreta do
dogfooding.

Feedback de amigos/testadores usa o mesmo funil, sempre identificado pela
origem, distro e versão do artefato. Prioridade: perda de dados/segurança/crash
→ bloqueio diário → regressão funcional → conforto/feature. Reproduzir antes de
alterar arquitetura ou transformar uma impressão isolada em funcionalidade.

## 3. Roteador por TIPO DE TAREFA (comece por aqui)

Identifique a tarefa e leia **somente** a coluna correspondente, na ordem. Cada
linha termina num gate. Se a tarefa não se encaixar em nenhuma, ela provavelmente
está mal definida — recorte antes de codar.

### 3.A — Adotar uma integração / plugin / tecnologia nova

```text
1. docs/integracoes/README.md          ← ENTRADA OBRIGATÓRIA (modos, gate,
                                          níveis L0–L10, checklist de 10 passos)
2. docs/roadmaps/KINEIN_VECTIS_OPEN_PLUGIN_ADAPTATION_ROADMAP.md   norte A–D
3. PONTO_ATUAL.md A5.1–A5.3            nível aberto? decisão do candidato?
4. docs/tooling/OPEN_COMPONENT_REGISTRY.json   o que já existe (não duplicar)
5. docs/adr/                           precedentes; ADR-0004 é o exemplo canônico
6. docs/arquitetura/03-ipc-protocol.md contrato tipado do domínio
```

Escreve em: `kinein-protocol/src/<dominio>.rs` → `kinein-core/src/<dominio>/` →
`handlers/` → testes → UI burra. **Nunca** a UI chamando a ferramenta.
Fecha com: ADR + entrada no registry + `bash scripts/verificar.sh`.

### 3.B — Melhoria / polimento de algo que já existe

```text
1. PONTO_ATUAL.md §0 (dogfooding) e A4  o atrito já foi registrado?
2. a spec da área em docs/specs/        o alvo visual/UX é inegociável
3. docs/roadmaps/20-ui-spec-convergence-plan.md   convergência C0–C6
4. o doc do domínio (tabela 3.1)        contrato e estado atual
5. GUIAIA §5                            quais arquivos mudam juntos
```

Regra: polimento **não** redesenha a UI aprovada nem inventa fatia nova. Se o
polimento for de terminal, entra por `docs/roadmaps/26` — e depois do ADR-0004
o emulador é o `alacritty_terminal`, não invente parser.

### 3.C — Bug / regressão vinda do uso real

```text
1. PONTO_ATUAL.md §0                   registrar ação/esperado/observado/ambiente
2. reproduzir ANTES de alterar arquitetura
3. o doc do domínio + GUIAIA §5        localizar a camada dona
4. teste/harness de regressão          antes ou junto da correção
```

Prioridade: P0 perda de dados/segurança/crash → P1 bloqueio de fluxo → P2
comportamento incorreto repetível → P3 conforto. Bug tem precedência sobre
qualquer fatia do roadmap.

### 3.D — Funcionalidade de IDE nova

```text
1. seção 2.1 do roadmap de adaptação    REFERÊNCIA PROFISSIONAL OBRIGATÓRIA
   (Code OSS · IntelliJ Community · Zed · Lapce · NetBeans)
2. docs/specs/KINEIN_VECTIS_SPEC_INDEX.md   a visão-alvo da área
3. docs/arquitetura/ARCHITECTURE.md     camadas e regra de split
4. PONTO_ATUAL.md                       a fatia está na fila? em que nível?
```

Registrar no doc do domínio: revisão estudada, subsistema, licença/modo, lições
e a adaptação. Referência autoriza **estudo**, nunca cópia ou tradução mecânica.

### 3.E — Contrato, arquitetura ou persistência

```text
1. docs/arquitetura/ARCHITECTURE.md     OBRIGATORIO, inteiro — contrato, nao consulta
2. docs/arquitetura/03-ipc-protocol.md  contrato implementado (+ bump de versão)
3. docs/arquitetura/02-repository-structure.md   onde o arquivo nasce
4. schemas/                             formato persistido precisa de schema
5. docs/arquitetura/06-strict-mode.md   rigor não é sugestão
```

### 3.F — Documentação

```text
1. docs/README.md                       índice; todo doc técnico entra nele
2. PLANO_ORGANIZACAO_E_HANDOFF.md       faixas P/T/X e o que já foi executado
3. docs/CONTRIBUINDO.md                 se a mudança afeta quem colabora
```

Faixa P (público) = só `README.md`, `MANUAL.md`, `Tutorial.md`. Faixa X
(pessoal) nunca entra em cópia entregue. Detalhe na seção 9.

### 3.G — Onde as coisas vivem agora (pós-reorganização de 2026-07-16)

```text
docs/arquitetura/   contrato de engenharia, IPC, strict mode, dívida, higiene
docs/build/         ambiente, comandos de compilação, gate
docs/seguranca/     rede de segurança de dados (atomic save + drafts)
docs/roadmaps/      planos de execução, longo prazo, adaptação open-source, KSWE
docs/specs/         especificação canônica (visão-alvo) + diagramas
docs/integracoes/   como adotar/escalar uma integração ("Plugins")
docs/adr/           decisões arquiteturais registradas
docs/tooling/       registro auditável de componentes
docs/iconografia/   sistema visual, ícones de arquivo e da árvore
raiz                README · MANUAL · Tutorial · COMO_EXECUTAR (+ faixa X)
```

### 3.H — Consulta rápida por pergunta

| Pergunta | Leia primeiro | Complemento |
| --- | --- | --- |
| Qual é o estado real agora? | `ContextoIA.md` | `PONTO_ATUAL.md` |
| Qual é a próxima tarefa? | `PONTO_ATUAL.md` | `docs/roadmaps/21-long-horizon-roadmap.md` |
| Em qual camada colocar código? | `docs/arquitetura/ARCHITECTURE.md` | `docs/arquitetura/02-repository-structure.md` |
| Como UI e core conversam? | `docs/arquitetura/03-ipc-protocol.md` | `crates/kinein-protocol/src/` |
| Qual é a UX/layout obrigatória? | `docs/specs/KINEIN_VECTIS_SPEC_INDEX.md` | specs de Layout, UI Components e Visual System |
| Editor, completion e navegação | spec `EDITOR_LANGUAGE_INTELLIGENCE` | `docs/roadmaps/25-syntax-tree-semantic-foundation.md` |
| Tree-sitter e fallback local | spec `TREE_SITTER_EDITOR_LAYER` | `docs/roadmaps/25-syntax-tree-semantic-foundation.md` + ADR-0002 |
| Projeto profundo/KSWE | `docs/roadmaps/KINEIN_VECTIS_DEEP_SEMANTIC_ENGINE_CPP_RUST_WORKFLOW.md` | seção “simbiose” de `docs/roadmaps/21-long-horizon-roadmap.md` |
| Build, Run, Test e Debug | spec `PRODUCT_FLOWS_BUILD_RUN_DEBUG` | `docs/build/22-compilacao-c-cpp-rust.md` |
| Terminal e KV Context | spec `AI_CLI_BRIDGE_EXTERNAL_TERMINAL` | `docs/roadmaps/24-paridade-e-fundacao.md` + `docs/roadmaps/26-terminal-rendering-parity-roadmap.md` |
| Dados, drafts e escrita segura | `docs/seguranca/23-rede-de-seguranca.md` | ADR-0001 + `docs/arquitetura/16-hidden-risks-checklist.md` |
| Adotar ferramenta open source | `docs/roadmaps/KINEIN_VECTIS_OPEN_PLUGIN_ADAPTATION_ROADMAP.md` | `docs/tooling/OPEN_COMPONENT_REGISTRY.json` |
| Referenciar implementação de IDE profissional | seção 2.1 do roadmap de adaptação | documento do domínio + `docs/arquitetura/ARCHITECTURE.md` |
| Strict mode e gates | `docs/arquitetura/06-strict-mode.md` | `docs/build/COMANDOS_BUILD_VERIFICACAO.md` |
| Daily driver e dogfooding | `docs/diario/18-daily-driver-plan.md` | `docs/roadmaps/21-long-horizon-roadmap.md` |
| Convergência visual | `docs/roadmaps/20-ui-spec-convergence-plan.md` | specs visuais |
| Instalação para contribuir | `docs/build/14-development-environment.md` | `COMO_EXECUTAR.md` |
| Uso da IDE por testador | `MANUAL.md` | `README.md` |
| AppImage, instalação e atualização | `Tutorial.md` | seção 8 deste guia |
| Entrega sanitizada do código | M7 de `docs/roadmaps/21-long-horizon-roadmap.md` | `Tutorial.md` + item correspondente de `PONTO_ATUAL.md` |

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
| `docs/roadmaps/KINEIN_VECTIS_DEEP_SEMANTIC_ENGINE_CPP_RUST_WORKFLOW.md` | desenho profundo do KSWE, C++/Rust, scheduler, brokers e contextos |
| `docs/roadmaps/KINEIN_VECTIS_OPEN_PLUGIN_ADAPTATION_ROADMAP.md` | como avaliar/adotar componentes externos sem extensão improvisada |
| `docs/arquitetura/ARCHITECTURE.md` | camadas, ownership, limites de arquivo e crescimento modular |
| `docs/arquitetura/02-repository-structure.md` | estrutura de crates/pastas e convenções de nomes |
| `docs/arquitetura/03-ipc-protocol.md` | requests, responses e eventos realmente implementados |
| `docs/arquitetura/06-strict-mode.md` | rigor Rust, C++, QML e linguagens suportadas |
| `docs/build/14-development-environment.md` | preparação da máquina de desenvolvimento |
| `docs/arquitetura/15-engineering-debt-and-refactor.md` | dívidas conhecidas e regras para não recriar monólitos |
| `docs/arquitetura/16-hidden-risks-checklist.md` | segurança, dados, segredos, a11y, observabilidade e packaging |
| `docs/arquitetura/17-architecture-hygiene-plan.md` | higiene arquitetural e concentrações que devem permanecer fechadas |
| `docs/diario/18-daily-driver-plan.md` | marcos M/E/T, dogfooding e paridade diária |
| `docs/arquitetura/19-architecture-tradeoffs.md` | razões e trade-offs por trás das decisões |
| `docs/roadmaps/20-ui-spec-convergence-plan.md` | execução vinculante da UI/UX C0–C6 |
| `docs/roadmaps/21-long-horizon-roadmap.md` | KSWE, M4–M7, distribuição e continuidade longa |
| `docs/build/22-compilacao-c-cpp-rust.md` | comandos/ferramentas reais orquestrados para C/C++/Rust |
| `docs/seguranca/23-rede-de-seguranca.md` | atomic save, drafts SQLite e recuperação contra perda |
| `docs/roadmaps/24-paridade-e-fundacao.md` | fases D1–D4: completion, terminal, Tree-sitter e remake |
| `docs/roadmaps/25-syntax-tree-semantic-foundation.md` | contrato da camada sintática, versões e workspace edits |
| `docs/roadmaps/26-terminal-rendering-parity-roadmap.md` | cursor/TUI aberto, referência Code OSS e etapas R0–R7 da paridade nativa |
| `docs/roadmaps/BACKEND_TO_UI_UX_ROADMAP.md` | ponte entre capacidade de backend e experiência visual |
| `docs/build/COMANDOS_BUILD_VERIFICACAO.md` | comandos oficiais do gate |
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

O pacote `docs/iconografia/sistema-visual/` só entra em tarefa de iconografia
ou contrato de ícones QML; não é leitura padrão de core/IPC. O pacote
complementar `docs/iconografia/icones-de-arquivo/` (raiz do repo) cobre os ícones
de tipos de arquivo especiais da árvore de projetos — `cmake-lists`,
`project-config`, `sql` e `docker-yaml` — em light/dark 16/20/24 px, com fontes
64 px. O contrato de resolução (nome exato → padrão → caminho → extensão) está em
`FILE_ICON_MAPPINGS.json`; a semântica, o sizing ótico, a implementação Qt/QML e
o checklist ficam nos `docs/01–04` da própria pasta. Assets originais
MIT OR Apache-2.0, sem logos oficiais; complementa a spec `VISUAL_SYSTEM_ICONS`.

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
- Contrato: workspace em `docs/arquitetura/03-ipc-protocol.md`.
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
- Segurança: `docs/seguranca/23-rede-de-seguranca.md`, ADR-0001 e
  `docs/arquitetura/16-hidden-risks-checklist.md`.
- Nunca acessar filesystem do workspace diretamente pela UI.
- Ícones de tipo de arquivo na árvore: pacote
  `docs/iconografia/icones-de-arquivo/`, contrato `FILE_ICON_MAPPINGS.json`
  (precedência nome exato → padrão → caminho → extensão composta → extensão →
  genérico). Ao ligar no delegate, honrar a regra de segurança do `.env` (não
  vazar valores/segredos) e não dar o ícone Docker a YAML genérico.

### 5.3 Editor, Tree-sitter, LSP e diagnósticos

```text
ui/qml/editor/EditorController.qml
    ├─ EditorCompletionController.qml
    ├─ EditorOutlineController.qml / EditorOutlinePanel.qml
    ├─ EditorTextSurface.qml / EditorGutter.qml
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
  `docs/roadmaps/25-syntax-tree-semantic-foundation.md` e especificação KSWE.
- Tree-sitter entrega estrutura/fallback; clangd e rust-analyzer são a
  autoridade semântica. Não misturar ou duplicar esses papéis.
- Toda edição avança imediatamente as versões sintática e semântica e limpa os
  semantic tokens anteriores. `lsp.semanticTokens` ecoa `path` e `version`;
  nunca aplicar resposta cujo par difira do documento/buffer ativo. Essa é a
  barreira contra a corrida Tree-sitter/LSP, não uma regra de precedência por
  tempo de chegada.
- `EditorGutter.qml` é a única dona do layout de números, folding, breakpoint,
  diagnóstico, blame e diff. As faixas são independentes e a coluna numérica
  usa a métrica real da fonte; não voltar a posicionar marcadores sobre uma
  estimativa fixa por dígito dentro de `EditorTextSurface.qml`.

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
- Fontes: spec Build/Run/Debug, `docs/build/22-compilacao-c-cpp-rust.md`,
  `docs/diario/18-daily-driver-plan.md` e `docs/roadmaps/21-long-horizon-roadmap.md`.
- KSWE deve reutilizar estes serviços; não criar outro executor de build.
- `workspace.capabilities.buildSystems` é a fonte única das ações disponíveis
  em projeto híbrido. `workspace.kind` continua apenas como primário compatível;
  não voltar a inferir capacidade isoladamente na UI.

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
- `run.script { path }` é o caminho de execução rápida da árvore para scripts
  `.sh/.bash/.zsh`: o core confina o arquivo e usa argv explícito. Não enviar
  conteúdo, comando shell ou caminho não validado pela UI.

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
- Fonte: spec AI CLI Bridge + `docs/roadmaps/24-paridade-e-fundacao.md`.
- KV Context reutiliza o TerminalManager; não criar terminal ou chat paralelo.
- Codex usa o modo inline oficial; o bridge preserva o transcript contra o
  erase-scrollback ainda emitido pela CLI. Seletor compacto e sessão terminal
  livre/persistida/maximizável são estados diferentes da mesma superfície,
  nunca um renderer de conversa da IDE. `Project` só é ocultado na maximização
  explícita, não pela mera existência da sessão.
- A grade VT e o cursor são a única representação da entrada. Não recriar
  faixa, `TextInput`, composer ou borda inferida por parsing da tela; a
  referência comportamental é terminal-first (Code OSS/xterm.js), adaptada ao
  renderer Qt/QML e ao contrato tipado existentes.
- Cada span de `event.terminal.render` informa `cells`; essa largura VT, a
  métrica monoespaçada usada no resize e a coluna do cursor formam uma única
  grade. Não derivar geometria de `text.length` ou de `implicitWidth`.
- O teste humano de 2026-07-15 não percebeu correção do alinhamento vertical da
  TUI mesmo após DECSCUSR e remoção de offsets. O problema permanece aberto e
  sua retomada obrigatória está em `docs/roadmaps/26-terminal-rendering-parity-roadmap.md`:
  começar por fixture/overlay/métricas, nunca por outro ajuste manual de `y`.
- Por decisão do usuário, a experiência funcional do terminal do Code OSS é a
  base de paridade para shell, Claude e Codex. Adaptar fielmente os contratos
  e invariantes para o renderer Qt; não incorporar xterm.js/Node nem criar um
  caminho exclusivo para o KV Context.

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
    │   ├─ AppMenuBar.qml → WindowControls.qml (min/max-restore/fechar)
    │   └─ WindowResizeHandles.qml (oito bordas)
    ├─ components/* (ícones, botões, tooltips)
    └─ ui/src/{documentation,clipboard,editor_highlighter,window_chrome_controller}.*
```

- UX-alvo: specs Layout, UI Components e Visual System.
- Plano de convergência: `docs/roadmaps/20-ui-spec-convergence-plan.md`.
- O visualizador do manual é recurso read-only da própria aplicação e não
  acessa workspace; mudanças de negócio continuam proibidas na UI.
- Decoração client-side (frameless): `Main.qml` usa `FramelessWindowHint` e
  `WindowChromeController` (`ui/src/window_chrome_controller.*`, `QML_ELEMENT`)
  é a única ponte para operações de janela — `showMinimized`/`showMaximized`/
  `showNormal`/`close` e `startSystemMove`/`startSystemResize` do compositor. O
  estado autoritativo é o do `QWindow`, sem booleano visual paralelo em QML;
  janela é responsabilidade do shell Qt, não do core/IPC. Aceite é visual em
  Wayland/X11 (`docs/roadmaps/20`).

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
7. `docs/arquitetura/03-ipc-protocol.md`, schema aplicável e `ContextoIA.md`.

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

### Nova funcionalidade de IDE

1. definir o problema, invariantes e falhas antes da solução;
2. consultar fonte oficial atual pertinente conforme a seção 1.1;
3. registrar revisão, subsistema, licença/modo e lições no documento do domínio;
4. desenhar a adaptação nas camadas nativas da Kinein, sem copiar ou traduzir
   mecanicamente a implementação de referência;
5. cobrir comportamento, cancelamento/erro, segurança e regressões com testes
   próprios.

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

O método mais simples para um testador é receber **um AppImage + SHA256 + o
instalador de atalho + o tutorial vigente**. Ele
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

`scripts/empacotar-appimage.sh`, quando chamado diretamente e sem argumentos,
encaminha para o mesmo fluxo portátil. Seu modo `--baseline-worker` é interno ao
container e não deve ser usado como build nativo no host.

O resultado fica em `dist/Kinein-Vectis-<versão>-x86_64.AppImage`, acompanhado
de `SHA256SUMS`, de um `.AppImage.sha256` específico e de
`instalar-kinein-vectis.sh`, além de uma cópia byte a byte do `Tutorial.md`
vigente. Esse instalador seleciona semanticamente o
AppImage mais recente da pasta, sobrescreve um único `.desktop` no escopo do
usuário e oferece apagar versões anteriores. Para um testador, enviar os quatro
arquivos; o procedimento completo fica em `Tutorial.md`. A receita usa
`linuxdeploy` + `linuxdeploy-plugin-qt`, builder
Debian 12 fixado e ferramentas auditadas no registry. O smoke portátil executa
sem rede em um Debian mínimo que não contém Qt, Rust, CMake ou compiladores. A
entrega é preparada em staging e publicada em `dist/` somente quando o conjunto
está completo; falhas preservam a última versão válida já entregue.

No computador de desenvolvimento, os atalhos têm identidades separadas:
`kinein-vectis.desktop` pertence ao AppImage distribuído e
`kinein-vectis-development.desktop` ao checkout. O segundo aparece como
**Kinein Vectis (Desenvolvimento)** e é criado somente por
`scripts/instalar-atalho.sh`; nenhum dos dois deve sobrescrever o outro.

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
[ ] docs/arquitetura/03/schema mudaram se o contrato mudou?
[ ] MANUAL.md mudou se o comportamento do usuário mudou?
[ ] Tutorial.md mudou se distribuição, instalação ou packaging mudaram?
[ ] O gate relevante e uma prova do gesto real ficaram verdes?
```

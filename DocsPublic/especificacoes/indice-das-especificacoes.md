# Kinein Vectis — SPEC_INDEX

> **Documento:** índice consolidado das especificações da Kinein Vectis.  
> **Função:** organizar todas as partes, corrigir contradições, definir fonte de verdade, separar MVP/Pós-MVP/Futuro e preparar a fase de implementação.  
> **Estado:** fase macro de definição concluída. Próxima fase: polimento, corte de escopo e implementação incremental.

> **Revisão vinculante de 2026-09-22:** a arquitetura de frontend consolidada
> está em `arquitetura-de-frontend-0.3-em-diante.md` e tem precedência sobre
> este inventário quando houver conflito. A IDE não terá Assistente/Chat de IA
> embutido nem telemetria de produto/usuário. Referências antigas a esses itens
> são material superado, não backlog. Wizards determinísticos de projeto/setup,
> logs locais, métricas locais e dados do alvo não dependem de IA nem implicam
> coleta pela Kinein.
>
> **Complemento de 2026-09-22:** Remote SSH precisa ser mais prático que o
> terminal tanto com SSH existente quanto na configuração inicial; o desenho
> está em `remote-ssh-ui-hud.md`. Grafana é obrigatório na 0.3.5 com UI/UX de
> uso diário, conforme `grafana-ui-ux-0.3.5.md`. Esses dois documentos vencem
> descrições antigas das respectivas interfaces.
> A ergonomia da série também inclui preview renderizado de Markdown, definido
> em `markdown-preview-0.3.md`.
>
> **Complemento de 2026-09-23:** Selecionar Tudo completo e rótulos de sessão
> reutilizáveis são básicos do terminal, ainda pendentes. Launcher e interação
> completa de pastas entram antes de Grafana, conforme
> [projetos, arquivos e desktop](projetos-arquivos-e-integracao-desktop-0.3.md).
> O refinamento de bordas/cabeçalho entra na 0.3.x, definido no
> [layout §6.5](sistema-de-layout.md#65-bordas-e-cabeçalho-mais-naturais--compromisso-da-03x).

---

## 1. Identidade do projeto

```text
Nome completo: Kinein Vectis
Nome curto: Kinein
Sigla visual: KV
Tipo: IDE Linux-first para C, C++ e Rust
Foco: CMake, Cargo, toolchains, build, run, debug, targets e setup visual
```

A Kinein Vectis deve ser uma IDE que:

```text
facilita sem invadir;
mostra opções reais;
respeita usuários avançados;
ajuda iniciantes/intermediários;
não esconde CMake/Cargo;
não força IA;
não modifica projeto sem preview;
mantém o usuário no controle.
```

Frase-guia:

```text
Kinein Vectis deve ser leve na presença,
forte na estrutura
e cuidadosa na forma como ajuda.
```

---

## 2. Decisões congeladas

Estas decisões devem ser consideradas estáveis até a implementação inicial.

```text
UI: Qt/QML
Core: Rust
IPC: JSON-RPC local
Modelo: UI separada do Core
Jobs: operações longas sempre assíncronas
Events: UI atualizada por eventos
Sem IA embutida
Sem chat lateral interno
IA externa: apenas ferramentas que o usuário execute por conta própria no terminal
Telemetria de produto/usuário: inexistente
Tree-sitter: estrutura local rápida
clangd/rust-analyzer: inteligência semântica sob demanda
Configuration Actions: camada visual para CMake/Cargo
Preview obrigatório para alterações
Experience Modes: Guided / Balanced / Expert
CMake/Cargo filtrados por Build System ativo
No public plugin system no MVP
```

---

## 3. Documentos oficiais

### 3.1 Parte 1 — Iconografia

Arquivo:

```text
sistema-visual-e-icones.md
```

Função:

```text
define identidade visual, app icon, ícones, paleta, tokens visuais,
estilo industrial/confortável e linguagem gráfica da Kinein.
```

Status:

```text
válido
```

---

### 3.2 Parte 2 — Layout principal

Arquivo:

```text
sistema-de-layout.md
```

Função:

```text
define app shell, top bar, toolbar, activity bar, editor,
painéis, terminal, status bar e separação entre UI real e marketing.
```

Status:

```text
válido
```

---

### 3.3 Parte 3 — Sistema de componentes UI

Arquivo:

```text
sistema-de-componentes-de-ui.md
```

Função:

```text
define botões, inputs, selects, badges, panels, tree view,
editor tabs, tool windows, dialogs, command palette e componentes QML.
```

Status:

```text
válido
```

---

### 3.4 Parte 4 — Build, Run e Debug

Arquivo:

```text
fluxos-de-produto-build-run-debug.md
```

Função:

```text
define fluxos de toolchain, CMake, Cargo, build, run,
debug, jobs, logs, parsing de erros e estados operacionais.
```

Status:

```text
válido
```

---

### 3.5 Parte 5 — Editor e Language Intelligence

Arquivo:

```text
editor-e-inteligencia-de-linguagem.md
```

Função:

```text
define editor, buffers, LSP, clangd, rust-analyzer,
diagnósticos, navegação, autocomplete, hover, rename e integração semântica.
```

Status:

```text
válido
```

Observação:

```text
deve ser lido junto com o addendum de Tree-sitter.
```

---

### 3.6 Parte 5.1 — Tree-sitter

Arquivo:

```text
camada-de-editor-tree-sitter.md
```

Função:

```text
define Tree-sitter como camada estrutural local para syntax highlighting,
folding, outline, breadcrumbs, escopo e fallback antes do LSP.
```

Status:

```text
válido
```

Regra:

```text
Tree-sitter não substitui clangd/rust-analyzer.
```

---

### 3.7 Parte 6 — Embedded Targets, Flash, Serial e QEMU

Arquivo:

```text
embarcados-targets-flash-serial-qemu.md
```

Função:

```text
define targets locais/remotos/embarcados, serial monitor,
flash, remote SSH, QEMU e integração futura de sistemas embarcados.
```

Status:

```text
válido, mas majoritariamente pós-MVP
```

---

### 3.8 Parte 7 — Assistente original

Arquivo:

```text
KINEIN_VECTIS_ASSISTANT_AI_ASSISTANCE.md
```

Função original:

```text
definia Assistente como painel de assistência/IA.
```

Status:

```text
cancelado; não é backlog
```

Correção:

```text
não implementar IA embutida;
não implementar chat lateral;
manter apenas ideias úteis de contexto determinístico,
preview, evidência e sanitização.
```

---

### 3.9 Parte 7.1 — AI CLI Bridge e Terminal IA Externo (cancelada)

Arquivo:

```text
KINEIN_VECTIS_AI_CLI_BRIDGE_EXTERNAL_TERMINAL.md
```

Função:

```text
define IA como ferramenta externa acionada por atalho,
com AI Terminal separado, Context Builder, sanitização e perfis de CLI.
```

Status:

```text
cancelado; não implementar bridge, context builder, perfis ou terminal de IA
```

Ferramentas externas que o usuário decidir executar continuam sendo apenas
processos comuns no terminal, sem integração especial, promessa de produto ou
acesso privilegiado ao contexto da IDE.

---

### 3.10 Parte 8 — Onboarding, Project Wizard e Settings

Arquivo:

```text
onboarding-assistente-de-projeto-e-configuracoes.md
```

Função:

```text
define primeira abertura, Project Wizard, Toolchain Manager,
CMake Setup visual, Run/Debug Wizard, Target Manager e Settings.
```

Status:

```text
válido
```

---

### 3.11 Parte 8.1 — Setup Intelligence Optimization Layer

Arquivo:

```text
onboarding-camada-de-setup-e-otimizacao.md
```

Função:

```text
define Environment Fingerprint, Project Fingerprint,
Capability Matrix, Configuration Graph, Setup Plan e Repair Engine.
```

Status:

```text
válido, implementar incrementalmente
```

---

### 3.12 Parte 9 — Arquitetura Interna

Arquivo:

```text
arquitetura-interna-core-ipc-jobs.md
```

Função:

```text
define Qt/QML Frontend, Rust Core, JSON-RPC, Jobs, Events,
Storage, Services, Risk Engine, Logging, CLI futura e Plugin Boundaries.
```

Status:

```text
fonte de verdade arquitetural
```

---

### 3.13 Parte 9.1 — Dual Workflow e Configuration Actions

Arquivo:

```text
fluxo-duplo-e-acoes-de-configuracao.md
```

Função:

```text
define Guided/Balanced/Expert Modes, Configuration Actions,
lista densa mas confortável, preview, filtros e modo expert não invasivo.
```

Status:

```text
válido
```

---

### 3.14 Parte 9.2 — Scoped Configuration Actions e Documentation Links

Arquivo:

```text
acoes-de-configuracao-com-escopo-e-links-de-documentacao.md
```

Função:

```text
define Configuration Actions filtradas por Build System ativo:
CMake-only, Cargo-only ou Mixed, além de links de documentação por ação.
```

Status:

```text
fonte de verdade para Configuration Actions
```

---

### 3.15 Parte 10 — Fechamento, Performance, MVP e Polimento

Arquivo:

```text
finalizacao-mvp-e-checklist-de-polimento.md
```

Função:

```text
define impacto de clangd/rust-analyzer, estratégia de performance,
MVP final, não-MVP, decisões congeladas e próximos passos.
```

Status:

```text
fonte de verdade para encerramento macro
```

---

### 3.x Specs que estavam FORA deste índice (adicionadas em 2026-08-29)

Medição de 2026-08-29: quatro specs de `DocsPublic/especificacoes/` não apareciam neste índice.
Três delas não eram citadas por **nenhum** documento do repositório — cerca de
3.500 linhas invisíveis para quem navega pelo índice, que é o caminho que o
`AGENTS.md` manda seguir. Não são canceladas (isso teria destino em
`DocsPrivate/legado/`); eram só inalcançáveis, o que na prática dá no mesmo.

```text
recursos-sob-demanda-estrategia-de-performance.md          (Parte 10)
  arquitetura orientada a demanda, LSP sob demanda, servicos ociosos,
  modos de performance. Derruba a fronteira rigida de "2 GB" de RAM.

recursos-sob-demanda-estrategia-de-refatoracao-da-inteligencia.md (10.1)
  correcao/evolucao da Parte 10: inspecoes em tempo real, contexto
  profundo de projeto e refatoracao progressiva.

plano-de-implementacao-ui-ux-arquitetura-performance.md
  plano de implementacao: milestones e criterios de aceite de UI/UX,
  arquitetura, performance e memoria.

```

O `tarefas-de-implementacao-diagrama.md` **saiu de `DocsPublic/especificacoes/`** em
2026-08-29, por decisão do autor: fala em "Kernwerk Studio" (nome anterior do
projeto), e as tarefas M1.1–M1.3 que ele propunha — modularizar `lsp.rs`,
dividir o `CoreClient`, quebrar o `Main.qml` — **já foram executadas**. A
sequência dele foi substituída pelos roadmaps 21, 24 e 28. Está em
`DocsPrivate/legado/`; a regra durável que ele carregava (reutilizar o sistema
existente) foi extraída para `DocsPublic/arquitetura/ARCHITECTURE.md` §8.1.

---

## 4. Contradições corrigidas

### 4.1 IA embutida

Antes:

```text
Assistente poderia ser interpretado como painel de IA dentro da IDE.
```

Agora:

```text
Não existe IA embutida nem AI CLI Bridge como recurso do produto.
Ferramentas externas continuam sendo responsabilidade do usuário no terminal.
```

---

### 4.2 Configuration Actions globais

Antes:

```text
poderia parecer que CMake e Cargo aparecem sempre juntos.
```

Agora:

```text
CMake project → CMake actions
Cargo project → Cargo actions
Mixed project → CMake + Cargo
```

Decisão final:

```text
Configuration Actions são filtradas pelo Build System ativo.
```

---

### 4.3 LSP como base de tudo

Antes:

```text
poderia parecer que clangd/rust-analyzer sustentam todo o produto.
```

Agora:

```text
clangd/rust-analyzer sustentam inteligência semântica,
mas setup visual, toolchain, Project Health e Configuration Actions
são determinísticos e independentes do LSP.
```

---

### 4.4 Usuário avançado vs usuário iniciante

Antes:

```text
poderia parecer que a UX guiada seria obrigatória.
```

Agora:

```text
Guided/Balanced/Expert Modes definem presença da ajuda visual.
Expert Mode reduz sugestões sem remover recursos.
```

---

## 5. Fonte de verdade por área

```text
Visual:
Parte 1, 2, 3

Editor/Linguagens:
Parte 5, 5.1

Build/Run/Debug:
Parte 4

Embedded/Targets:
Parte 6

IA externa:
Parte 7.1

Onboarding/Setup:
Parte 8, 8.1

Arquitetura interna:
Parte 9

Configuration Actions:
Parte 9.1, 9.2

Performance/MVP:
Parte 10
```

---

## 6. MVP recomendado

O MVP deve provar a proposta com escopo controlado.

### 6.1 Core mínimo

```text
Rust Core
JSON-RPC stdio
kinein-protocol
workspace.open
workspace.close
core.ping
event stream
job system básico
storage básico
logging básico
```

### 6.2 UI mínima

```text
Qt/QML AppShell
Top bar
Activity bar
Project panel
Editor area
Bottom tool window
Status bar
Settings básicas
Command Palette básica
```

### 6.3 Projeto e setup

```text
Project detection CMake/Cargo
Environment scan
Toolchain scan
Project Wizard C++ CMake
Project Wizard Rust Cargo
CMakePresets generation
Cargo project initialization
Project Health básico
```

### 6.4 Build e linguagem

```text
cmake configure
cmake build
cargo metadata
cargo check
cargo build
Tree-sitter syntax highlighting
clangd status
rust-analyzer status
diagnostics básicos
```

### 6.5 Configuration Actions MVP

CMake:

```text
Enable compile_commands.json
Create Debug preset
Create Release preset
Add executable
Add static library
Add source file to target
Add include directory
Add target_link_libraries
Inspect CMake cache
Repair stale build directory
```

Cargo:

```text
Add dependency
Add dev-dependency
Add feature
Set edition
Run cargo check
Create run config
```

### 6.6 Terminal

```text
Terminal comum
AI Terminal externo
AI CLI profile simples
Context Builder básico
Preview de contexto
Sanitização básica
```

---

## 7. Pós-MVP

```text
debug visual mais completo
breakpoints avançados
variables/call stack refinado
remote SSH target
serial monitor refinado
QEMU targets
OpenOCD/probe-rs
CMake actions avançadas
Cargo workspace actions
docs cacheadas
support bundle sanitizado
kinein doctor CLI
fuzzy search nas Configuration Actions
favorites/recent actions
```

---

## 8. Futuro distante

```text
plugin system público
marketplace
QEMU visual workbench
remote development completo
embedded flashing completo
multi-root workspace avançado
refatorações profundas próprias
integração de documentação oficial offline completa
team profiles
cloud sync opcional
```

---

## 9. Não implementar agora

```text
IA embutida
chat lateral
plugin marketplace
instalador automático de toolchain
simulador OpenGL
QEMU completo
remote dev completo
CMake parser universal
suporte completo a todos build systems
agente autônomo
aplicar patch vindo de IA automaticamente
modificar projeto existente sem preview
```

---

## 10. Roadmap de implementação

### 10.1 Milestone 0 — Fundação

```text
criar repositório
definir estrutura de pastas
criar crates básicos
criar app Qt/QML mínimo
criar core process
criar JSON-RPC ping
criar logging
```

### 10.2 Milestone 1 — Workspace

```text
abrir pasta
listar arquivos
detectar CMake/Cargo
criar Project Fingerprint
mostrar Project Health inicial
```

### 10.3 Milestone 2 — Toolchains

```text
scan de CMake/Ninja/GCC/Clang/Cargo/Rust
Toolchain Manager básico
health check básico
settings locais
```

### 10.4 Milestone 3 — CMake/Cargo

```text
CMake configure/build como jobs
Cargo metadata/check/build como jobs
logs
events
Problems básico
```

### 10.5 Milestone 4 — Editor

```text
editor básico
Tree-sitter
abrir/salvar arquivo
diagnostics básicos
clangd/rust-analyzer status
```

### 10.6 Milestone 5 — Onboarding

```text
first-run mode choice
Project Wizard C++ CMake
Project Wizard Rust Cargo
CMakePresets geração
```

### 10.7 Milestone 6 — Configuration Actions MVP

```text
registry de actions
filtro CMake/Cargo/Mixed
preview
apply
diff
validação
docs metadata placeholder
```

### 10.8 Milestone 7 — AI Terminal Bridge

```text
AI Terminal separado
profile simples
context markdown
sanitização
preview
abrir CLI externa
```

### 10.9 Milestone 8 — Polimento MVP

```text
UX densa confortável
mensagens melhores
atalhos
Command Palette
performance
testes
README
docs
```

---

## 11. Estrutura inicial sugerida

```text
kinein/
├── apps/
│   ├── kinein-ui/
│   └── kinein-core-daemon/
├── crates/
│   ├── kinein-protocol/
│   ├── kinein-core/
│   ├── kinein-ipc/
│   ├── kinein-events/
│   ├── kinein-jobs/
│   ├── kinein-storage/
│   ├── kinein-workspace/
│   ├── kinein-projects/
│   ├── kinein-toolchains/
│   ├── kinein-cmake/
│   ├── kinein-cargo/
│   ├── kinein-language/
│   ├── kinein-terminal/
│   ├── kinein-ai-bridge/
│   ├── kinein-config-actions/
│   ├── kinein-setup/
│   └── kinein-cli/
├── ui/
│   └── qml/
├── DocsPublic/
│   ├── specs/
│   ├── guides/
│   └── registry/
└── README.md
```

---

## 12. Polimento posterior

Quando entrar na fase de polimento, revisar nesta ordem:

```text
1. Contradições entre docs.
2. Escopo MVP vs Pós-MVP.
3. Nome de componentes.
4. Texto de UX.
5. Densidade visual.
6. Performance.
7. Testabilidade.
8. Segurança.
9. README público.
10. Issues e milestones.
```

---

## 13. Checklist final da fase macro

```text
[ ] Nome definido.
[ ] Identidade visual definida.
[ ] Layout definido.
[ ] Componentes definidos.
[ ] Editor/LSP/Tree-sitter definidos.
[ ] Build/Run/Debug definidos.
[ ] Onboarding definido.
[ ] Setup Intelligence definido.
[ ] Partes 7 e 7.1 marcadas como canceladas.
[ ] Arquitetura interna definida.
[ ] Configuration Actions definidas.
[ ] Escopo CMake/Cargo/Mixed definido.
[ ] Performance definida.
[ ] MVP definido.
[ ] Pós-MVP definido.
[ ] Não-MVP definido.
[ ] Roadmap definido.
```

---

## 14. Próximo documento recomendado

Depois deste SPEC_INDEX, o próximo documento útil é:

```text
KINEIN_VECTIS_IMPLEMENTATION_PLAN.md
```

Ele deve transformar o roadmap em tarefas implementáveis:

```text
Milestone
Epic
Task
Critério de aceite
Ordem
Dependência
Risco
```

---

## 15. Resumo executivo

A Kinein Vectis está conceitualmente definida.

Ela será:

```text
uma IDE para C/C++/Rust;
Linux-first;
determinística;
visualmente confortável;
não invasiva;
com setup visual;
com Configuration Actions;
com fluxo expert;
com IA externa opcional;
com Rust Core;
com Qt/QML Frontend;
com Jobs/Events/JSON-RPC;
com MVP controlado.
```

A fase de ideias macro está fechada.

Agora o trabalho deve sair de:

```text
imaginar o produto
```

para:

```text
organizar o MVP
criar tarefas
implementar base
testar fluxo real
polir com evidência
```

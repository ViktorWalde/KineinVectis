# Documentação da Kinein Vectis

Este índice organiza a documentação do projeto e define a ordem de precedência
quando houver conflito entre documentos.

## Ordem de precedência

```text
1. ContextoIA.md (raiz)   → estado real e decisões vigentes do repositório
2. docs/specs/            → especificação canônica da Kinein Vectis (visão-alvo)
3. docs/ (numerados)      → contrato e estado do que já está implementado
```

Regra: `docs/specs/` descreve o **alvo** (produto, UX, visual, arquitetura
completa). Os docs numerados mantidos em `docs/` descrevem o que **já existe**
no repositório. Onde a visão divergir da implementação, vale o estado real —
`ContextoIA.md` + docs numerados + código.

Para localizar rapidamente quais documentos e arquivos se conectam em cada
domínio, use primeiro o [GUIAIA.md](../GUIAIA.md). Ele é um índice operacional
mantido junto com a arquitetura, não uma nova fonte de verdade.

## docs/specs/ — especificação canônica (Kinein Vectis)

Fonte de verdade de produto, UX, sistema visual e arquitetura-alvo: 20
especificações com diagramas `.svg` pareados e imagens de referência.

Comece pelo índice:
[specs/KINEIN_VECTIS_SPEC_INDEX.md](specs/KINEIN_VECTIS_SPEC_INDEX.md) — mapa de
todas as partes, fonte de verdade por área, escopo MVP/Pós-MVP/Não-fazer e
roadmap de milestones.

Âncoras principais (ver o índice para o conjunto completo):

| Área | Spec |
| --- | --- |
| Arquitetura interna (Core/IPC/Jobs) | `KINEIN_VECTIS_INTERNAL_ARCHITECTURE_CORE_IPC_JOBS.md` |
| Layout principal | `KINEIN_VECTIS_LAYOUT_SYSTEM.md` |
| Componentes UI | `KINEIN_VECTIS_UI_COMPONENTS_SYSTEM.md` |
| Sistema visual / iconografia | `KINEIN_VECTIS_VISUAL_SYSTEM_ICONS.md` |
| Build / Run / Debug | `KINEIN_VECTIS_PRODUCT_FLOWS_BUILD_RUN_DEBUG.md` |
| Editor / Language Intelligence | `KINEIN_VECTIS_EDITOR_LANGUAGE_INTELLIGENCE.md` |
| IA externa (AI CLI Bridge) | `KINEIN_VECTIS_AI_CLI_BRIDGE_EXTERNAL_TERMINAL.md` |
| Configuration Actions | `KINEIN_VECTIS_SCOPED_CONFIGURATION_ACTIONS_DOC_LINKS.md` |
| Fechamento / MVP / Performance | `KINEIN_VECTIS_FINALIZATION_MVP_ROADMAP_POLISH_CHECKLIST.md` |

## docs/ — contrato e estado implementado

Documentos que descrevem o repositório como ele **é hoje**:

| Doc | Assunto |
| --- | --- |
| [ARCHITECTURE.md](ARCHITECTURE.md) | **Arquitetura e convenções de crescimento — ler antes de codar** |
| [BACKEND_TO_UI_UX_ROADMAP.md](BACKEND_TO_UI_UX_ROADMAP.md) | Ponte operacional backend → UI/UX para implementar backend primeiro sem perder os specs visuais |
| [02-repository-structure.md](02-repository-structure.md) | Estrutura real do repositório e crates |
| [03-ipc-protocol.md](03-ipc-protocol.md) | Protocolo IPC JSON-RPC implementado (0.55.0) |
| [06-strict-mode.md](06-strict-mode.md) | Strict mode (Rust e C++/Qt) |
| [14-development-environment.md](14-development-environment.md) | Ambiente de desenvolvimento |
| [15-engineering-debt-and-refactor.md](15-engineering-debt-and-refactor.md) | Dívida técnica e modularização pós-V1 |
| [16-hidden-risks-checklist.md](16-hidden-risks-checklist.md) | Riscos ocultos (perda de dados, migração de config, segurança de comandos, segredos, testes de regressão, a11y, observabilidade, packaging) |
| [17-architecture-hygiene-plan.md](17-architecture-hygiene-plan.md) | Fase ativa para eliminar concentração arquitetural antes de novas features grandes |
| [18-daily-driver-plan.md](18-daily-driver-plan.md) | Plano de daily driver (marcos M0–M4 de dogfooding) e escada de rigor |
| [19-architecture-tradeoffs.md](19-architecture-tradeoffs.md) | Requisitos funcionais/não funcionais e trade-offs de arquitetura (o porquê das decisões) |
| [20-ui-spec-convergence-plan.md](20-ui-spec-convergence-plan.md) | Plano vinculante de convergência da UI atual para docs/specs (fatias C0–C6, regras anti-vagueza) |
| [21-long-horizon-roadmap.md](21-long-horizon-roadmap.md) | M4–M7 detalhados, KSWE, distribuição e playbook de continuidade |
| [22-compilacao-c-cpp-rust.md](22-compilacao-c-cpp-rust.md) | Referência prática de comandos de compilação C/C++ (gcc/clang/CMake) e Rust (cargo), mapeada para a IDE |
| [23-rede-de-seguranca.md](23-rede-de-seguranca.md) | Rede de segurança contra perda de dado (escrita atômica + autosave em SQLite): problemas, design e status vivo da fatia S1 |
| [24-paridade-e-fundacao.md](24-paridade-e-fundacao.md) | Fase pós-rede-de-segurança (D1–D4, ordem do usuário): autocomplete LSP ao vivo → terminal paridade VS Code/JetBrains → tree-sitter/plugins/views → remake. Status vivo. |
| [25-syntax-tree-semantic-foundation.md](25-syntax-tree-semantic-foundation.md) | Contrato D3: Tree-sitter incremental, composição com LSP, folding/outline e vínculo com workspace edits transacionais. |
| [../KINEIN_VECTIS_OPEN_PLUGIN_ADAPTATION_ROADMAP.md](../KINEIN_VECTIS_OPEN_PLUGIN_ADAPTATION_ROADMAP.md) | **Norte autoritativo de adoção de ferramentas open-source** (modos A–D, gate de auditoria, licenças, P0–P3). Ler antes de adotar qualquer tech externa. |
| [COMANDOS_BUILD_VERIFICACAO.md](COMANDOS_BUILD_VERIFICACAO.md) | Gate único de build e verificação |
| [tooling/OPEN_COMPONENT_REGISTRY.json](tooling/OPEN_COMPONENT_REGISTRY.json) | Registro auditável de componentes open-source adotados |
| [adr/ADR-0001-notify-filesystem-watcher.md](adr/ADR-0001-notify-filesystem-watcher.md) | Decisão de adoção do watcher `notify` e barreira compare-before-save |
| [adr/ADR-0002-tree-sitter-syntax-foundation.md](adr/ADR-0002-tree-sitter-syntax-foundation.md) | Decisão de adoção do Tree-sitter e fronteira com LSP |
| [adr/ADR-0003-linuxdeploy-appimage-packaging.md](adr/ADR-0003-linuxdeploy-appimage-packaging.md) | Decisão de empacotamento AppImage, pins, baseline Linux e auditoria |

## Guias na raiz do repositório

| Arquivo | Assunto |
| --- | --- |
| [../README.md](../README.md) | Apresentação e estado atual do projeto |
| [../MANUAL.md](../MANUAL.md) | **Manual do usuário** — operação, funções e atalhos dentro da IDE |
| [../Tutorial.md](../Tutorial.md) | Distribuição, checksum, instalação, atualização e geração do AppImage |
| [../COMO_EXECUTAR.md](../COMO_EXECUTAR.md) | Como executar a IDE (ícone/launcher) |
| [../AGENTS.md](../AGENTS.md) | Instruções e política de leitura para agentes de IA |
| [../ContextoIA.md](../ContextoIA.md) | Estado operacional e decisões vigentes |
| [../GUIAIA.md](../GUIAIA.md) | **Mapa operacional:** documentos por necessidade, módulos conectados, arquivos que mudam juntos e gates |
| [../PONTO_ATUAL.md](../PONTO_ATUAL.md) | Ordem explícita do trabalho pendente |
| [../KINEIN_VECTIS_DEEP_SEMANTIC_ENGINE_CPP_RUST_WORKFLOW.md](../KINEIN_VECTIS_DEEP_SEMANTIC_ENGINE_CPP_RUST_WORKFLOW.md) | Complemento de arquitetura semântica C++/Rust; aplicar por fatias conforme o roadmap vigente |

## Sem pasta de arquivo morto

`docs/archive/` (docs numerados era-Kernwerk superados, planning antigo, logs
de sessão) foi removido deliberadamente em 2026-07-05: era material histórico
que nenhum documento ativo referenciava mais como fonte, e mantê-lo só
custava tokens de leitura para humanos e agentes sem guiar trabalho atual.
Não recriar uma pasta de arquivo "só para guardar"; se algo for descontinuado,
prefira apagar depois de extrair o que ainda tiver valor para o doc numerado
relevante (mesmo espírito do item anterior).

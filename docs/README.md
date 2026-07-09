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
| [03-ipc-protocol.md](03-ipc-protocol.md) | Protocolo IPC JSON-RPC implementado (0.20.0) |
| [06-strict-mode.md](06-strict-mode.md) | Strict mode (Rust e C++/Qt) |
| [14-development-environment.md](14-development-environment.md) | Ambiente de desenvolvimento |
| [15-engineering-debt-and-refactor.md](15-engineering-debt-and-refactor.md) | Dívida técnica e modularização pós-V1 |
| [16-hidden-risks-checklist.md](16-hidden-risks-checklist.md) | Riscos ocultos (perda de dados, migração de config, segurança de comandos, segredos, testes de regressão, a11y, observabilidade, packaging) |
| [17-architecture-hygiene-plan.md](17-architecture-hygiene-plan.md) | Fase ativa para eliminar concentração arquitetural antes de novas features grandes |
| [18-daily-driver-plan.md](18-daily-driver-plan.md) | Plano de daily driver (marcos M0–M4 de dogfooding) e escada de rigor |
| [COMANDOS_BUILD_VERIFICACAO.md](COMANDOS_BUILD_VERIFICACAO.md) | Gate único de build e verificação |

## Guias na raiz do repositório

| Arquivo | Assunto |
| --- | --- |
| [../README.md](../README.md) | Apresentação e estado atual do projeto |
| [../COMO_EXECUTAR.md](../COMO_EXECUTAR.md) | Como executar a IDE (ícone/launcher) |
| [../AGENTS.md](../AGENTS.md) | Instruções e política de leitura para agentes de IA |
| [../ContextoIA.md](../ContextoIA.md) | Estado operacional e decisões vigentes |

## Sem pasta de arquivo morto

`docs/archive/` (docs numerados era-Kernwerk superados, planning antigo, logs
de sessão) foi removido deliberadamente em 2026-07-05: era material histórico
que nenhum documento ativo referenciava mais como fonte, e mantê-lo só
custava tokens de leitura para humanos e agentes sem guiar trabalho atual.
Não recriar uma pasta de arquivo "só para guardar"; se algo for descontinuado,
prefira apagar depois de extrair o que ainda tiver valor para o doc numerado
relevante (mesmo espírito do item anterior).

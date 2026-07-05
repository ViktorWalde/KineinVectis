# Documentação da Kinein Vectis

Este índice organiza a documentação do projeto e define a ordem de precedência
quando houver conflito entre documentos.

## Ordem de precedência

```text
1. ContextoIA.md (raiz)   → estado real e decisões vigentes do repositório
2. docs/specs/            → especificação canônica da Kinein Vectis (visão-alvo)
3. docs/ (numerados)      → contrato e estado do que já está implementado
4. docs/archive/          → material histórico; NÃO guia trabalho atual
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
| [02-repository-structure.md](02-repository-structure.md) | Estrutura real do repositório e crates |
| [03-ipc-protocol.md](03-ipc-protocol.md) | Protocolo IPC JSON-RPC implementado (0.19.0) |
| [06-strict-mode.md](06-strict-mode.md) | Strict mode (Rust e C++/Qt) |
| [14-development-environment.md](14-development-environment.md) | Ambiente de desenvolvimento |
| [15-engineering-debt-and-refactor.md](15-engineering-debt-and-refactor.md) | Dívida técnica e modularização pós-V1 |
| [COMANDOS_BUILD_VERIFICACAO.md](COMANDOS_BUILD_VERIFICACAO.md) | Gate único de build e verificação |

## Guias na raiz do repositório

| Arquivo | Assunto |
| --- | --- |
| [../README.md](../README.md) | Apresentação e estado atual do projeto |
| [../COMO_EXECUTAR.md](../COMO_EXECUTAR.md) | Como executar a IDE (ícone/launcher) |
| [../AGENTS.md](../AGENTS.md) | Instruções e política de leitura para agentes de IA |
| [../ContextoIA.md](../ContextoIA.md) | Estado operacional e decisões vigentes |

## docs/archive/ — histórico (não guia trabalho atual)

- **[archive/legacy/](archive/legacy/)** — docs numerados de visão (00, 01,
  04, 05, 07–13, 16), `subsystems/` e `quality/` da fase Kernwerk, **superados
  pelas `docs/specs/`** da Kinein Vectis. Mantidos apenas como histórico.
- **[archive/planning/](archive/planning/)** — visão e blueprint de concepção
  (a estrutura real de crates evoluiu diferente — ver doc 02 e o código).
- **archive/contextoia-session-logs-2026-07.md** — logs de sessão antigos,
  movidos do `ContextoIA.md`.

Índices agregadores duplicados (`*_ALL_DOCS`, `*_MASTER`) foram removidos; o
conteúdo único vive nas `docs/specs/` e nos docs numerados mantidos.

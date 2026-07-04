# Documentação do Kernwerk Studio

Este índice organiza toda a documentação do projeto e define a ordem de
precedência quando houver conflito entre documentos.

## Ordem de precedência

```text
1. ContextoIA.md (raiz)      → estado real e decisões vigentes do repositório
2. docs/00–16 (numerados)    → contratos e especificações canônicas em vigor
3. docs/quality/             → políticas de tooling/qualidade (referência)
4. docs/subsystems/          → especificação de subsistemas futuros (referência)
5. docs/planning/            → visão e planejamento histórico (referência)
```

Os documentos numerados e o `ContextoIA.md` descrevem o repositório como ele
é. Os pacotes em `planning/`, `quality/` e `subsystems/` são material de
visão/planejamento gerado nas fases de concepção: valem como guia de rumo e
padrões, mas onde divergirem da implementação real (crates, protocolo,
fases), vale o que está em `ContextoIA.md` + docs numerados + código.

## Documentos canônicos (numerados)

| Doc | Assunto |
| --- | --- |
| [00-product-vision.md](00-product-vision.md) | Visão de produto |
| [01-architecture.md](01-architecture.md) | Arquitetura UI Qt/QML ↔ core Rust |
| [02-repository-structure.md](02-repository-structure.md) | Estrutura do repositório |
| [03-ipc-protocol.md](03-ipc-protocol.md) | Protocolo IPC JSON-RPC (contrato vigente) |
| [04-command-system.md](04-command-system.md) | Sistema de comandos |
| [05-design-system.md](05-design-system.md) | Design system visual |
| [06-strict-mode.md](06-strict-mode.md) | Strict mode (Rust e C++/Qt) |
| [07-tooling-lifecycle.md](07-tooling-lifecycle.md) | Ciclo de vida de ferramentas externas |
| [08-performance-budget.md](08-performance-budget.md) | Orçamento de performance |
| [09-roadmap.md](09-roadmap.md) | Roadmap por fases |
| [10-mvp-plan.md](10-mvp-plan.md) | Plano do MVP |
| [11-layout-interactions.md](11-layout-interactions.md) | Layout e interações da UI |
| [12-ai-policy.md](12-ai-policy.md) | Política de IA |
| [13-open-source-references.md](13-open-source-references.md) | Referências open source |
| [14-development-environment.md](14-development-environment.md) | Ambiente de desenvolvimento |
| [15-engineering-debt-and-refactor.md](15-engineering-debt-and-refactor.md) | Dívida técnica, modularidade e refatoração pós-V1 |
| [16-compiler-modes-and-function-store.md](16-compiler-modes-and-function-store.md) | Modos de compilador e loja de funções pós-V1 |

## Guias na raiz do repositório

| Arquivo | Assunto |
| --- | --- |
| [../README.md](../README.md) | Apresentação e estado atual do projeto |
| [../COMO_EXECUTAR.md](../COMO_EXECUTAR.md) | Como executar a IDE (ícone/launcher) |
| [../AGENTS.md](../AGENTS.md) | Instruções para agentes de IA |
| [../ContextoIA.md](../ContextoIA.md) | Sincronização de estado entre agentes de IA |

## Pacotes de referência

- **[planning/](planning/)** — visão e planejamento histórico:
  `KERNWERK_STUDIO_MASTER.md` (consolidação inicial), `KERNWERK_STUDIO_ALL_DOCS.md`
  (concatenação das docs 00–14 de uma época), `KERNWERK_STUDIO_ARCHITECTURE_OVERVIEW.md`
  e `IMPLEMENTATION_BLUEPRINT.md` (plano de execução inicial; a estrutura real de
  crates evoluiu diferente — ver docs 02 e o código).
- **[quality/](quality/)** — políticas de toolchain, tooling nativo C/C++/Rust,
  Quality Center/strict modes e padrões profissionais de IDE. Ver
  [quality/README.md](quality/README.md).
- **[subsystems/](subsystems/)** — especificações de subsistemas profissionais
  (task manager, editor engine, busca/indexação, refatoração, UX de erros,
  first-run, segredos, dependências C++, crash recovery, design system da UI).
  Ver [subsystems/README.md](subsystems/README.md).

## Prompts

- [../prompts/GPT_TERMINAL_BOOTSTRAP.md](../prompts/GPT_TERMINAL_BOOTSTRAP.md) —
  prompt inicial para agentes GPT/Claude no terminal.

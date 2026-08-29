# Documentação da Kinein Vectis

Este índice organiza a documentação técnica do projeto por assunto e define a
ordem de precedência quando houver conflito entre documentos.

## Ordem de precedência

```text
0. O CÓDIGO + os gates    → a ÚNICA fonte do que existe. Mede-se, não se lê.
1. CONTRATO               → as regras. Só mudam por decisão explícita registrada.
2. ESTADO                 → tem que ser verdade HOJE. Se divergir do código,
                            o código vence e o documento se corrige no mesmo gesto.
3. PLANO / ALVO           → aspiracional. Diverge por natureza; reconciliar.
4. LOG                    → registro datado. Nunca reescrever. NÃO é estado.
```

**Em conflito, o código vence sempre. Nenhum documento derruba uma medição.**

### As quatro classes de volatilidade

Este eixo é **ortogonal** às faixas P/T/X (audiência) de
`PLANO_ORGANIZACAO_E_HANDOFF.md`. Ele responde outra pergunta: *"se este arquivo
envelhecer, o que acontece?"*

| Classe | Regra | O que acontece se envelhecer | Onde |
| --- | --- | --- | --- |
| **CONTRATO** | Não muda sem decisão explícita e registrada. **Não contém número medido nem inventário** — número é o que apodrece. | Nada: é regra, não estado. | `AGENTS.md`, `arquitetura/ARCHITECTURE.md` §2/§4/§5, `adr/` |
| **ESTADO** | Tem que ser verdade **agora**. Todo número é verificável contra o disco. | **Mente.** Manda a próxima sessão reimplementar o que existe. | `PONTO_ATUAL.md`, `GUIAIA.md` (os mapas), `arquitetura/02`, `arquitetura/03` |
| **PLANO** | Descreve o alvo. Pode divergir da implementação — é para isso que existe. | Aceitável, mas reconciliar ao retomar. | `docs/specs/`, `docs/roadmaps/` |
| **LOG** | Registro datado do que foi decidido **naquele dia**. Nunca reescrever. | Nada: envelhecer é a função dele. | `ContextoIA.md`, `diario/`, `adr/` |

### Por que o `ContextoIA.md` saiu de "estado real" (2026-07-17)

Ele estava em **primeiro** nesta lista, descrito como *"estado real e decisões
vigentes"*. Ele é um **log append-only** com 44 entradas datadas, e o próprio
cabeçalho dele afirma ser "enxuto de propósito" — sendo o maior documento do
repositório. Log em primeiro na precedência é o mecanismo que faz uma sessão nova
confiar num registro velho: foi assim que, em 2026-07-17, uma IA reimplementou um
seletor que o autor mandou remover no mesmo dia e listou como pendente um harness
entregue havia 24 horas.

**Log é ótimo para responder "por que isto é assim?". É péssimo para responder "o
que existe hoje?" — essa pergunta se responde no código.**

### A regra que separa registro de mentira

> **Número com data é registro. Número sem data é afirmação sobre AGORA — e tem
> que ser verdade.**

`scripts/verificar-docs.sh` (no gate) verifica isso mecanicamente: todo `arquivo
… N linhas` sem data por perto é conferido contra o disco. Se você precisa citar
um número antigo, **date-o**; custa quatro palavras.

O limite dessa trava, para ninguém achar que ela resolve tudo: ela pega número
**sem** data. Um número datado que envelheceu continua enganando quem lê — foi o
caso do `arquitetura/17`, que passava no script e mentia por 3,4x. Esse caso pede
julgamento, e o remédio é o mesmo da §1.1: marcar "não vale mais" e apontar a
fonte viva.

**Vai alterar ou implementar algo?** Comece por
[CONTRIBUINDO.md](CONTRIBUINDO.md): arquitetura em uma tela, tabela de "quero
mudar X → olhe aqui", ritual de uma mudança e o gate.

Para localizar rapidamente quais documentos e arquivos se conectam em cada
domínio, use também o [GUIAIA.md](../GUIAIA.md), cuja seção 3 é um roteador por
tipo de tarefa (integração nova, polimento, bug, funcionalidade, contrato).

## Estrutura

```text
docs/
├── arquitetura/   contrato de engenharia, protocolo IPC, strict mode, dívida e higiene
├── build/         ambiente, comandos de compilação e gate de verificação
├── seguranca/     rede de segurança de dados (escrita atômica + drafts)
├── roadmaps/      planos de execução, roadmap de longo prazo, adaptação e KSWE
├── specs/         especificação canônica (visão-alvo) + diagramas
├── integracoes/   como adicionar/escalar uma integração ("Plugins")
├── adr/           decisões arquiteturais registradas
├── tooling/       registro auditável de componentes open-source
├── iconografia/   sistema visual, ícones de arquivo e da árvore
└── diario/        registro de sessões (material interno; não publicado)
```

## Público × interno

Uma cópia entregue a terceiros leva o código e **somente três Markdown**:
`README.md`, `MANUAL.md` e `Tutorial.md`. Todo o resto — inclusive esta pasta
`docs/` inteira — é interno.

Isso não depende de disciplina: `scripts/exportar-copia-limpa.sh` gera a cópia
por allowlist numa árvore separada, recusa Markdown extra, verifica que nenhum
caminho interno aparece nem como nome, audita segredos e imprime a lista final.
Roda em dry-run por padrão e **não cria repositório nem publica nada**.

## Entrada para quem vai mexer no código

| Documento | Assunto |
| --- | --- |
| [CONTRIBUINDO.md](CONTRIBUINDO.md) | **Onde olhar para alterar/implementar**: arquitetura, mapa por área, ambiente, ritual da mudança, gate e convenções |
| [integracoes/README.md](integracoes/README.md) | **Entrada obrigatória para adotar qualquer ferramenta**: modos A–D, gate de auditoria, níveis L0–L10, checklist de 10 passos e o índice do que já está adotado |

## arquitetura/ — contrato e estado implementado

| Documento | Assunto |
| --- | --- |
| [arquitetura/ARCHITECTURE.md](arquitetura/ARCHITECTURE.md) | **LEITURA OBRIGATÓRIA — contrato de arquitetura (camadas, regra de split, crescimento). Verificado por catraca. Antes de propor arquitetura nova: MEDIR — o problema costuma ser regra não cumprida, não regra ausente (§1.1)** |
| [arquitetura/02-repository-structure.md](arquitetura/02-repository-structure.md) | Estrutura real do repositório e crates |
| [arquitetura/03-ipc-protocol.md](arquitetura/03-ipc-protocol.md) | Protocolo IPC JSON-RPC implementado |
| [arquitetura/06-strict-mode.md](arquitetura/06-strict-mode.md) | Strict mode (Rust e C++/Qt) |
| [arquitetura/15-engineering-debt-and-refactor.md](arquitetura/15-engineering-debt-and-refactor.md) | Dívida técnica e modularização |
| [arquitetura/16-hidden-risks-checklist.md](arquitetura/16-hidden-risks-checklist.md) | Riscos ocultos (dados, config, segurança de comandos, segredos, a11y, observabilidade, packaging) |
| [arquitetura/17-architecture-hygiene-plan.md](arquitetura/17-architecture-hygiene-plan.md) | Higiene arquitetural e concentrações a eliminar |
| [arquitetura/19-architecture-tradeoffs.md](arquitetura/19-architecture-tradeoffs.md) | Requisitos e trade-offs de arquitetura (o porquê das decisões) |
| [arquitetura/27-modulos-por-dominio.md](arquitetura/27-modulos-por-dominio.md) | **PROPOSTA** — módulos por domínio (front + back), catraca no core e fronteira do subsistema opcional (simulador OpenGL) |

## build/ — ambiente, compilação e verificação

| Documento | Assunto |
| --- | --- |
| [build/14-development-environment.md](build/14-development-environment.md) | Ambiente de desenvolvimento |
| [build/22-compilacao-c-cpp-rust.md](build/22-compilacao-c-cpp-rust.md) | Referência prática de comandos de compilação C/C++ e Rust mapeados para a IDE |
| [build/COMANDOS_BUILD_VERIFICACAO.md](build/COMANDOS_BUILD_VERIFICACAO.md) | Gate único de build e verificação |

## seguranca/ — segurança de dados

| Documento | Assunto |
| --- | --- |
| [seguranca/23-rede-de-seguranca.md](seguranca/23-rede-de-seguranca.md) | Rede de segurança contra perda de dado (escrita atômica + autosave em SQLite) |

## roadmaps/ — planos de execução e visão de longo prazo

| Documento | Assunto |
| --- | --- |
| [roadmaps/BACKEND_TO_UI_UX_ROADMAP.md](roadmaps/BACKEND_TO_UI_UX_ROADMAP.md) | Ponte operacional backend → UI/UX |
| [roadmaps/20-ui-spec-convergence-plan.md](roadmaps/20-ui-spec-convergence-plan.md) | Convergência vinculante da UI atual para as specs (fatias C0–C6) |
| [roadmaps/21-long-horizon-roadmap.md](roadmaps/21-long-horizon-roadmap.md) | M4–M7, KSWE, distribuição e continuidade longa |
| [roadmaps/24-paridade-e-fundacao.md](roadmaps/24-paridade-e-fundacao.md) | Fases D1–D4: completion, terminal, Tree-sitter e remake |
| [roadmaps/25-syntax-tree-semantic-foundation.md](roadmaps/25-syntax-tree-semantic-foundation.md) | Contrato da camada sintática (Tree-sitter incremental, composição com LSP) |
| [roadmaps/28-plataforma-de-plugins-e-verticais.md](roadmaps/28-plataforma-de-plugins-e-verticais.md) | **Plataforma de plugins (`integration` v1) e as verticais**: C/C++/Rust sólidos, Docker e banco como domínios NATIVOS, embarcados — e a dívida contínua de UI/UX com o IntelliJ Community como referência adaptada |
| [roadmaps/26-terminal-rendering-parity-roadmap.md](roadmaps/26-terminal-rendering-parity-roadmap.md) | Paridade de renderização/scroll do terminal: reprodução instrumentada, métricas de célula/DPR e gates (R0–R7) |
| [roadmaps/KINEIN_VECTIS_OPEN_PLUGIN_ADAPTATION_ROADMAP.md](roadmaps/KINEIN_VECTIS_OPEN_PLUGIN_ADAPTATION_ROADMAP.md) | **Norte de adoção e referência open-source**: modos A–D, gate/licenças e política de estudo de Code OSS, IntelliJ, Zed, Lapce e NetBeans |
| [roadmaps/KINEIN_VECTIS_DEEP_SEMANTIC_ENGINE_CPP_RUST_WORKFLOW.md](roadmaps/KINEIN_VECTIS_DEEP_SEMANTIC_ENGINE_CPP_RUST_WORKFLOW.md) | Desenho profundo do KSWE (C++/Rust, scheduler, brokers e contextos) |

## specs/ — especificação canônica (visão-alvo)

Fonte de verdade de produto, UX, sistema visual e arquitetura-alvo. Comece pelo
índice: [specs/KINEIN_VECTIS_SPEC_INDEX.md](specs/KINEIN_VECTIS_SPEC_INDEX.md).

Âncoras principais:

| Área | Spec |
| --- | --- |
| Arquitetura interna (Core/IPC/Jobs) | [specs/KINEIN_VECTIS_INTERNAL_ARCHITECTURE_CORE_IPC_JOBS.md](specs/KINEIN_VECTIS_INTERNAL_ARCHITECTURE_CORE_IPC_JOBS.md) |
| Layout principal | [specs/KINEIN_VECTIS_LAYOUT_SYSTEM.md](specs/KINEIN_VECTIS_LAYOUT_SYSTEM.md) |
| Componentes UI | [specs/KINEIN_VECTIS_UI_COMPONENTS_SYSTEM.md](specs/KINEIN_VECTIS_UI_COMPONENTS_SYSTEM.md) |
| Sistema visual / iconografia | [specs/KINEIN_VECTIS_VISUAL_SYSTEM_ICONS.md](specs/KINEIN_VECTIS_VISUAL_SYSTEM_ICONS.md) |
| Build / Run / Debug | [specs/KINEIN_VECTIS_PRODUCT_FLOWS_BUILD_RUN_DEBUG.md](specs/KINEIN_VECTIS_PRODUCT_FLOWS_BUILD_RUN_DEBUG.md) |
| Editor / Language Intelligence | [specs/KINEIN_VECTIS_EDITOR_LANGUAGE_INTELLIGENCE.md](specs/KINEIN_VECTIS_EDITOR_LANGUAGE_INTELLIGENCE.md) |
| IA externa (AI CLI Bridge) | [specs/KINEIN_VECTIS_AI_CLI_BRIDGE_EXTERNAL_TERMINAL.md](specs/KINEIN_VECTIS_AI_CLI_BRIDGE_EXTERNAL_TERMINAL.md) |
| Configuration Actions | [specs/KINEIN_VECTIS_SCOPED_CONFIGURATION_ACTIONS_DOC_LINKS.md](specs/KINEIN_VECTIS_SCOPED_CONFIGURATION_ACTIONS_DOC_LINKS.md) |
| Fechamento / MVP / Performance | [specs/KINEIN_VECTIS_FINALIZATION_MVP_ROADMAP_POLISH_CHECKLIST.md](specs/KINEIN_VECTIS_FINALIZATION_MVP_ROADMAP_POLISH_CHECKLIST.md) |

## adr/ e tooling/

| Documento | Assunto |
| --- | --- |
| [adr/ADR-0001-notify-filesystem-watcher.md](adr/ADR-0001-notify-filesystem-watcher.md) | Adoção do watcher `notify` e barreira compare-before-save |
| [adr/ADR-0002-tree-sitter-syntax-foundation.md](adr/ADR-0002-tree-sitter-syntax-foundation.md) | Adoção do Tree-sitter e fronteira com LSP |
| [adr/ADR-0003-linuxdeploy-appimage-packaging.md](adr/ADR-0003-linuxdeploy-appimage-packaging.md) | Empacotamento AppImage, pins, baseline Linux e auditoria |
| [adr/ADR-0004-alacritty-terminal-emulator.md](adr/ADR-0004-alacritty-terminal-emulator.md) | Adoção do `alacritty_terminal` como motor de emulação VT |
| [tooling/OPEN_COMPONENT_REGISTRY.json](tooling/OPEN_COMPONENT_REGISTRY.json) | Registro auditável de componentes open-source adotados |

## diario/ — registro de sessões (interno)

| Documento | Assunto |
| --- | --- |
| [diario/18-daily-driver-plan.md](diario/18-daily-driver-plan.md) | Diário das fatias: marcos de dogfooding, decisões por sessão e escada de rigor. É registro de processo, não contrato — o contrato vive em `arquitetura/`. |

## iconografia/ — sistema visual e ícones

| Pacote | Assunto |
| --- | --- |
| [iconografia/README.md](iconografia/README.md) | **Índice único**: qual pacote é fonte de verdade de cada família |
| [iconografia/sistema-visual/](iconografia/sistema-visual/) | Sistema de ícones da IDE (master + partes 00–08, SVGs, contrato QML) |
| [iconografia/icones-de-arquivo/](iconografia/icones-de-arquivo/) | Ícones de tipos de arquivo especiais da árvore + `FILE_ICON_MAPPINGS.json` |
| [iconografia/icones-da-arvore/](iconografia/icones-da-arvore/) | Ícones individuais da árvore de projetos |

## Guias na raiz do repositório

| Arquivo | Assunto |
| --- | --- |
| [../README.md](../README.md) | Apresentação e estado atual do projeto |
| [../MANUAL.md](../MANUAL.md) | **Manual do usuário** — operação, funções e atalhos dentro da IDE |
| [../Tutorial.md](../Tutorial.md) | Distribuição, checksum, instalação, atualização e geração do AppImage |
| [../COMO_EXECUTAR.md](../COMO_EXECUTAR.md) | Como executar a IDE pelo checkout (ícone/launcher) |

Documentos de continuidade operacional — `../GUIAIA.md`, `../ContextoIA.md`,
`../PONTO_ATUAL.md`, `../AGENTS.md` e `../prompts/` — são material interno de
desenvolvimento e não integram a documentação pública.

## Sem pasta de arquivo morto

`docs/archive/` foi removido deliberadamente em 2026-07-05: era material
histórico que nenhum documento ativo referenciava mais como fonte. Não recriar
uma pasta de arquivo "só para guardar"; se algo for descontinuado, extrair o que
ainda tiver valor para o documento relevante e então remover.

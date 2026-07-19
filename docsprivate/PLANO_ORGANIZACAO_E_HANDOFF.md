# Plano de Reorganização Documental e Handoff Linear — Kinein Vectis

> ## Status de execução (2026-07-16)
>
> O inventário abaixo descreve o estado **anterior** à reorganização e continua
> valendo como registro da decisão. O que já foi executado:
>
> ```text
> [x] Fase 1 — agrupamento físico por assunto
>        docs/{arquitetura,build,seguranca,roadmaps,integracoes,iconografia}
>        21 arquivos por git mv + 410 referências reescritas + 0 links quebrados
> [x] Fase 1b — arquivos (não só .md)
>        os 3 pacotes de ícones foram unificados em docs/iconografia/ e a raiz
>        ficou só com diretórios legítimos
> [x] Fase 2 — tom de condução eliminado dos docs faixa T
>        A métrica inicial superestimava: das 83 ocorrências do AI_CLI_BRIDGE,
>        49 eram "IA" = o produto. Preservado o que é legítimo (texto de UI,
>        cenários de produto). Saíram: "Handoff rápido para agentes / Se você é
>        uma IA", "a IA deve verificar", "handoff para a próxima IA" e as
>        atribuições datadas ("a pedido do usuário").
> [x] Fase 3 — obsoletos sinalizados
>        KV_CONTEXT_AI_ASSISTANCE marcado como SUPERADO pela Parte 7.1
>        docsprivate/prompts/GPT_TERMINAL_BOOTSTRAP marcado como OBSOLETO
>        docs/README.md reescrito como índice único
> [x] Fase 4 — camada pessoal explícita
>        docsprivate/diario/ criado; o diário (18-daily-driver-plan) saiu de roadmaps/
>        com 89 referências reescritas. Os arquivos X da raiz (ContextoIA,
>        PONTO_ATUAL, GUIAIA, AGENTS) FICAM onde estão de propósito: o fluxo de
>        reentrada e a convenção de agentes dependem desses caminhos. A exclusão
>        deles é feita por allowlist na exportação, não por mover.
> [x] Fase 5 — exportador da cópia limpa: scripts/exportar-copia-limpa.sh
>        Dry-run por padrão. Verifica denylist (nem como caminho), recusa
>        Markdown extra, audita segredos e vazamento de path do autor, e imprime
>        a lista final verificável. NÃO cria repositório e NÃO publica.
> [x] Escalonamento de plugins — docs/integracoes/README.md
> [x] Roteamento — GUIAIA §3 (por tarefa) + docs/CONTRIBUINDO.md (colaborador)
> [ ] PUBLICAÇÃO — depende de você: confirmar a lista de nomes proibidos (A6) e
>     autorizar. O exportador está pronto e verificado; nada foi publicado.
> ```
>
> O exportador já provou seu valor no primeiro dry-run: pegou `ui/README.md`
> vazando como 4º Markdown e o caminho `/home/viktor/KineinVectis` hardcoded em
> `scripts/sonda_scrollback.py` e `sonda_m43b.py` (corrigido para derivar a raiz).
>
> Nomes que mudaram e que este documento ainda cita pelo nome antigo:
> `KINEIN_VECTIS_ICONS_COMPLETE` → `docs/iconografia/sistema-visual`,
> `KINEIN_VECTIS_SPECIAL_FILE_ICONS` → `docs/iconografia/icones-de-arquivo`,
> `KINEIN_VECTIS_TREE_ICONS_INDIVIDUAL` → `docs/iconografia/icones-da-arvore`.

> **Natureza deste documento:** interno e pessoal. Ele pertence à camada
> operacional do repositório privado e **não entra em nenhuma cópia entregue a
> terceiros nem no repositório público futuro**. É o plano que rege a limpeza da
> documentação e a sequência de trabalho de engenharia; uma vez executado, seu
> valor é histórico.
>
> **Função:** (I) definir como toda a documentação do projeto será classificada,
> agrupada por assunto, reescrita e separada entre o que é público, o que é
> técnico-interno e o que é registro operacional pessoal; (II) consolidar em uma
> única sequência linear o trabalho necessário para levar a IDE ao estado mais
> completo possível a partir dos componentes e ferramentas abertas já mapeados.
>
> **Precedência:** este plano **não** substitui `docsprivate/ContextoIA.md`, `docsprivate/PONTO_ATUAL.md`
> nem os documentos de domínio. Ele organiza e lineariza o que já está decidido
> nesses arquivos (A3–A6, trilha T, M4–M7, KSWE e o roadmap de adaptação). Em
> caso de conflito, vale o estado real do código + `docsprivate/ContextoIA.md`.
>
> **Base de leitura:** produzido após leitura de `docsprivate/GUIAIA.md`, `docsprivate/ContextoIA.md`,
> `docsprivate/PONTO_ATUAL.md`, `docsprivate/AGENTS.md`, `docs/README.md`, `docs/ARCHITECTURE.md`,
> `docs/21`, `docs/specs/KINEIN_VECTIS_SPEC_INDEX.md`,
> `KINEIN_VECTIS_OPEN_PLUGIN_ADAPTATION_ROADMAP.md`,
> `docs/tooling/OPEN_COMPONENT_REGISTRY.json` e inventário completo dos 76
> arquivos Markdown do repositório (fora de `build/`, `target/`, `.kinein/`).

---

# Parte I — Reorganização da documentação

## 1. Objetivo e princípios

A documentação cresceu com o produto e hoje reúne, no mesmo repositório e muitas
vezes na mesma raiz, três tipos de material com finalidades distintas:

1. **material voltado ao leitor externo** — apresenta e ensina a usar o produto;
2. **material técnico de engenharia** — arquitetura, contrato, especificação e
   decisões, dirigido a quem desenvolve;
3. **registro operacional de continuidade** — texto escrito para orientar a
   condução do desenvolvimento (fila viva, handoffs, protocolos de reentrada,
   direção por sessão), com forte marca de "assistente informando o autor".

O objetivo desta reorganização é **separar essas três finalidades de forma
explícita**, dar a cada documento um assunto e um lugar únicos, elevar todo o
material técnico a um registro formal e impessoal (documentação que informa um
leitor, não um assistente informando um operador) e garantir que o registro
operacional pessoal permaneça exclusivamente com o autor, fora de qualquer cópia
distribuída.

Princípios que regem cada decisão abaixo:

- **um assunto, um dono:** cada documento cobre um tema; temas duplicados são
  consolidados, não mantidos em paralelo;
- **extrair antes de descartar:** material descontinuado tem seu conteúdo útil
  migrado para o documento técnico correto e então é removido, seguindo a mesma
  política já adotada quando `docs/archive/` foi eliminado;
- **tom por finalidade:** documento técnico usa registro descritivo e impessoal;
  narrativa de sessão, dogfooding e direção pelo autor não pertencem a ele;
- **separação por construção, não por ocultação:** o que é privado fica fora da
  árvore publicada por *allowlist* — nunca protegido apenas por `.gitignore`,
  que revela nomes e não impede rastreamento anterior;
- **rastreabilidade:** toda movimentação e reescrita é registrada, para que o
  histórico técnico não se perca na limpeza.

## 2. Modelo de classificação — as três faixas

Todo arquivo de documentação recebe exatamente uma faixa.

| Faixa | Nome | Destino | Tom exigido |
| --- | --- | --- | --- |
| **P** | Público | Repositório privado **e** cópia limpa/publicada | Formal, orientado ao leitor final; sem qualquer traço de processo de desenvolvimento |
| **T** | Interno-técnico | Somente repositório privado | Formal, descritivo, impessoal; documentação que informa um engenheiro |
| **X** | Pessoal / operacional | Somente cópia local do autor | Livre; é o material de condução do desenvolvimento e permanece apenas com o autor |

Regra de decisão, aplicada em ordem:

```text
1. O documento ensina a instalar, usar ou entender o produto do ponto de vista
   de quem o recebe pronto?            → Faixa P
2. O documento descreve arquitetura, contrato, especificação-alvo, decisão de
   engenharia ou referência técnica, e faz sentido para qualquer engenheiro
   que continue o projeto?             → Faixa T (reescrever tom onde houver
                                          narrativa de sessão/condução)
3. O documento existe para conduzir o desenvolvimento — fila viva, handoff,
   protocolo de reentrada, direção por sessão, prompts, diário de dogfooding —
   e só faz sentido para o autor?      → Faixa X
```

A distinção central pedida é entre **"documentação informando um leitor"**
(faixas P e T) e **"assistente informando o operador"** (faixa X). O segundo
tipo não é reescrito para parecer técnico: ele é reconhecido como registro
pessoal e mantido fora da distribuição.

### 2.1 Regra do desmembramento

Vários roadmaps misturam, no mesmo arquivo, conteúdo técnico durável (contratos,
invariantes, critérios de aceite) e narrativa de condução (o que o autor pediu,
o que foi observado numa sessão, handoffs de reentrada). Para esses casos a
ordem não é "mover o arquivo inteiro", e sim **desmembrar**:

- o conteúdo técnico durável é extraído e reescrito no documento de domínio
  correto (faixa T);
- a narrativa de sessão, dogfooding e direção permanece na camada pessoal
  (faixa X);
- o arquivo original é então substituído ou removido, sem deixar duas versões
  concorrentes.

## 3. Diagnóstico do estado atual

Números do inventário (76 arquivos Markdown de projeto, exclusos `build/`,
`target/`, `.kinein/` e o cache do Cargo):

- **raiz do repositório:** 11 documentos, misturando público (`README`,
  `MANUAL`, `Tutorial`), técnico (`KINEIN_VECTIS_OPEN_PLUGIN_ADAPTATION_ROADMAP`,
  `KINEIN_VECTIS_DEEP_SEMANTIC_ENGINE_CPP_RUST_WORKFLOW`) e pessoal (`GUIAIA`,
  `ContextoIA`, `PONTO_ATUAL`, `AGENTS`);
- **`docs/`:** 26 documentos numerados/nomeados + índice, boa parte já técnica e
  limpa, alguns com forte narrativa de sessão (`18`, `24`, `26`);
- **`docs/specs/`:** 20 especificações de visão-alvo + 19 SVG + 4 PNG;
- **iconografia:** três pacotes distintos (`docs/KINEIN_VECTIS_ICONS_COMPLETE/`,
  `KINEIN_VECTIS_SPECIAL_FILE_ICONS/`, `KINEIN_VECTIS_TREE_ICONS_INDIVIDUAL/`),
  cada um com sua documentação, somando ~180 SVG e vários formatos de apoio;
- **decisões e registro:** `docs/adr/` (3 ADRs) + `docs/tooling/` (registry);
- **prompts:** `docsprivate/prompts/GPT_TERMINAL_BOOTSTRAP.md`.

Problemas concretos identificados:

1. **Dispersão na raiz:** documentos de finalidade oposta convivem no mesmo
   nível, sem um agrupamento por assunto.
2. **Tom misto em documentos técnicos:** a varredura de marcadores de condução
   ("IA", "agente", "sessão", "handoff", "reentrada", "o usuário pediu/quer",
   "Claude/Codex/GPT", "você") encontrou concentração alta fora da camada
   pessoal — notadamente `docs/specs/KINEIN_VECTIS_AI_CLI_BRIDGE_EXTERNAL_TERMINAL.md`
   (83 ocorrências), `docs/24` (63), `docs/26` (53), `docs/18` (53) e
   `docs/03-ipc-protocol.md` (37). Parte é legítima (o Bridge de IA é uma
   funcionalidade real do produto); parte é narrativa de sessão que precisa sair
   dos documentos técnicos.
3. **Material obsoleto:** `docsprivate/prompts/GPT_TERMINAL_BOOTSTRAP.md` referencia documentos
   que já não existem (`docs/00-product-vision.md`, `docs/01-architecture.md`,
   `docs/04-command-system.md`, `docs/10-mvp-plan.md`, removidos com a extinção de
   `docs/archive/`). É registro pessoal desatualizado.
4. **Especificação superada convivendo com a vigente:** `KV_CONTEXT_AI_ASSISTANCE`
   foi conceitualmente substituída por `AI_CLI_BRIDGE_EXTERNAL_TERMINAL`, mas
   permanece no conjunto de specs sem marca clara de histórico.
5. **Artefatos gerados tratados como fonte:** `dist/Tutorial.md` é cópia byte a
   byte do `Tutorial.md` produzida pelo empacotamento; não é documento-fonte e
   não deve ser editado nem versionado como tal.
6. **Três pacotes de ícones independentes** sem um índice único que diga qual é
   a fonte de verdade de cada família visual.

## 4. Inventário classificado e agrupado por assunto

Colunas: **Faixa** (P/T/X), **Ação** e observação. "Reescrever" significa aplicar
a política de tom da Parte 5. "Desmembrar" segue a regra 2.1.

### G1 — Uso final e apresentação (produto para quem recebe)

| Documento | Faixa | Ação |
| --- | --- | --- |
| `README.md` | **P** | Manter; revisão leve de tom (apresentação e estado). |
| `MANUAL.md` | **P** | Manter; revisar para linguagem de usuário final consistente, sem termos de processo interno. |
| `Tutorial.md` | **P** | Manter; cobre instalação, checksum, atualização e geração de pacote. |
| `COMO_EXECUTAR.md` | **T** | Guia de build/execução para quem desenvolve; permanece privado (não faz parte da *allowlist* pública). |
| `ui/README.md` | **T** | README técnico do subprojeto de UI. |
| `dist/Tutorial.md` | — | **Artefato gerado.** Não é fonte; excluir do conjunto documental e do versionamento como documento. |

### G2 — Arquitetura e contrato de engenharia

| Documento | Faixa | Ação |
| --- | --- | --- |
| `docs/ARCHITECTURE.md` | **T** | Manter; já formal. Fonte da regra de camadas e crescimento modular. |
| `docs/02-repository-structure.md` | **T** | Manter. |
| `docs/03-ipc-protocol.md` | **T** | Manter; reescrever ocorrências pontuais de tom de sessão. Contrato IPC implementado. |
| `docs/06-strict-mode.md` | **T** | Manter. |
| `docs/19-architecture-tradeoffs.md` | **T** | Manter (o "porquê" das decisões). |
| `docs/15-engineering-debt-and-refactor.md` | **T** | Manter; reescrever narrativa de refator. |
| `docs/17-architecture-hygiene-plan.md` | **T** | Manter. |
| `docs/16-hidden-risks-checklist.md` | **T** | Manter. |
| `docs/14-development-environment.md` | **T** | Manter. |
| `docs/22-compilacao-c-cpp-rust.md` | **T** | Manter (referência de comandos). |
| `docs/23-rede-de-seguranca.md` | **T** | Manter; desmembrar o "status vivo" da fatia para a camada pessoal. |
| `docs/COMANDOS_BUILD_VERIFICACAO.md` | **T** | Manter (gate). |
| `docs/README.md` | **T** | Reescrever como índice único da documentação técnica reorganizada. |
| `schemas/*.json` (5) | **T** | Contrato de dados; manter junto da documentação de protocolo. |

### G3 — Especificação de produto / visão-alvo (`docs/specs/`)

Vinte especificações + diagramas pareados. Todas **faixa T**. Ação padrão:
manter como visão-alvo, reescrever ocorrências de tom de condução e marcar
explicitamente o estado (vigente / histórico).

| Documento | Ação específica |
| --- | --- |
| `KINEIN_VECTIS_SPEC_INDEX.md` | Manter como índice das specs; atualizar estados. |
| `KINEIN_VECTIS_VISUAL_SYSTEM_ICONS.md` | Manter; cruzar com o grupo G4 (iconografia). |
| `KINEIN_VECTIS_LAYOUT_SYSTEM.md` | Manter. |
| `KINEIN_VECTIS_UI_COMPONENTS_SYSTEM.md` | Manter. |
| `KINEIN_VECTIS_EDITOR_LANGUAGE_INTELLIGENCE.md` | Manter. |
| `KINEIN_VECTIS_TREE_SITTER_EDITOR_LAYER.md` | Manter (addendum do editor). |
| `KINEIN_VECTIS_PRODUCT_FLOWS_BUILD_RUN_DEBUG.md` | Manter. |
| `KINEIN_VECTIS_EMBEDDED_TARGETS_FLASH_SERIAL_QEMU.md` | Manter (majoritariamente pós-MVP). |
| `KINEIN_VECTIS_AI_CLI_BRIDGE_EXTERNAL_TERMINAL.md` | Manter (fonte de verdade de IA externa); **reescrita intensa de tom** — é a spec com maior densidade de marcadores. |
| `KINEIN_VECTIS_ASSISTANT_AI_ASSISTANCE.md` | **Marcar como histórico/superado** por `AI_CLI_BRIDGE`; preservar só as ideias úteis de contexto determinístico, ou desmembrar e remover. |
| `KINEIN_VECTIS_ONBOARDING_PROJECT_WIZARD_SETTINGS.md` | Manter. |
| `KINEIN_VECTIS_ONBOARDING_SETUP_OPTIMIZATION_LAYER.md` | Manter. |
| `KINEIN_VECTIS_INTERNAL_ARCHITECTURE_CORE_IPC_JOBS.md` | Manter (fonte arquitetural-alvo). |
| `KINEIN_VECTIS_DUAL_WORKFLOW_CONFIGURATION_ACTIONS.md` | Manter. |
| `KINEIN_VECTIS_SCOPED_CONFIGURATION_ACTIONS_DOC_LINKS.md` | Manter (fonte de Configuration Actions). |
| `KINEIN_VECTIS_RESOURCE_ON_DEMAND_PERFORMANCE_STRATEGY.md` | Manter. |
| `KINEIN_VECTIS_RESOURCE_ON_DEMAND_INTELLIGENCE_REFACTORING_STRATEGY.md` | Manter. |
| `KINEIN_VECTIS_FINALIZATION_MVP_ROADMAP_POLISH_CHECKLIST.md` | Manter (fechamento macro). |
| `KINEIN_VECTIS_IMPLEMENTATION_PLAN_UI_UX_ARCH_PERFORMANCE.md` | Manter; conferir sobreposição com `IMPLEMENTATION_TASKS`. |
| `KINEIN_VECTIS_IMPLEMENTATION_TASKS.md` | Manter; reescrever tom de condução. |

### G4 — Sistema visual e iconografia (assets + documentação)

Todos **faixa T** (design técnico). Ação prioritária: **um índice único** que
declare a fonte de verdade de cada família e a relação com a spec
`VISUAL_SYSTEM_ICONS`.

| Pacote | Conteúdo | Ação |
| --- | --- | --- |
| `docs/KINEIN_VECTIS_ICONS_COMPLETE/` | 13 md (incl. `KINEIN_VECTIS_ICON_SYSTEM_MASTER.md`, 4871 linhas), ~129 SVG, QML/CPP/JS/HTML/JSON/TXT/PNG | Manter como pacote de iconografia principal; consolidar o `MASTER` como fonte, os `01_SPECIFICATION/docs/00–08` como partes. |
| `KINEIN_VECTIS_SPECIAL_FILE_ICONS/` | 6 md, 29 SVG, `FILE_ICON_MAPPINGS.json`, `SHA256SUMS` | Manter; é o contrato de ícones de tipos de arquivo especiais da árvore. |
| `KINEIN_VECTIS_TREE_ICONS_INDIVIDUAL/` | 1 md, SVG/PNG, JSON | Manter; conferir se é subconjunto já coberto pelos outros dois e, se for, consolidar. |
| `imagens/` | `app-icon.png`, `bugs/ReformularBarra.png` | Separar: `app-icon.png` é asset de produto; `bugs/` é material de trabalho (faixa X). |

### G5 — Roadmaps e planos de execução

Grupo com maior necessidade de **desmembramento** (regra 2.1): conteúdo técnico
durável vira faixa T; narrativa de sessão/dogfooding vira faixa X.

| Documento | Faixa após desmembramento | Ação |
| --- | --- | --- |
| `KINEIN_VECTIS_OPEN_PLUGIN_ADAPTATION_ROADMAP.md` | **T** | Manter; norte de adoção de componentes abertos. Já é técnico. |
| `KINEIN_VECTIS_DEEP_SEMANTIC_ENGINE_CPP_RUST_WORKFLOW.md` | **T** | Manter; desenho do KSWE. |
| `docs/BACKEND_TO_UI_UX_ROADMAP.md` | **T** | Manter. |
| `docs/20-ui-spec-convergence-plan.md` | **T** | Manter. |
| `docs/21-long-horizon-roadmap.md` | **T** (durável) + **X** (playbook/diário) | Desmembrar: M4–M7 e a "simbiose" ficam técnicos; o "playbook para IA continuar" e os estados de sessão saem. |
| `docs/25-syntax-tree-semantic-foundation.md` | **T** | Manter (contrato da camada sintática). |
| `docs/24-paridade-e-fundacao.md` | **T** (contratos) + **X** (status vivo) | Desmembrar: o design de completion/terminal/Tree-sitter vira técnico; a narrativa D1–D4 por sessão sai. |
| `docs/26-terminal-rendering-parity-roadmap.md` | **T** (R0–R7, métricas) + **X** (handoff) | Desmembrar: o método de reprodução e os gates ficam técnicos; o handoff executável de sessão sai. |
| `docs/18-daily-driver-plan.md` | **X** (diário) | É o "diário" de fatias por sessão; camada pessoal. Extrair antes qualquer decisão durável ainda não migrada. |

### G6 — Decisões e registro de componentes

| Documento | Faixa | Ação |
| --- | --- | --- |
| `docs/adr/ADR-0001-notify-filesystem-watcher.md` | **T** | Manter. |
| `docs/adr/ADR-0002-tree-sitter-syntax-foundation.md` | **T** | Manter. |
| `docs/adr/ADR-0003-linuxdeploy-appimage-packaging.md` | **T** | Manter. |
| `docs/tooling/OPEN_COMPONENT_REGISTRY.json` | **T** | Manter; registro auditável, já em inglês técnico. |

### G7 — Continuidade operacional e direção por IA (pessoal)

Todos **faixa X**. Permanecem exclusivamente com o autor; **nunca** entram na
cópia limpa e **não** devem ser nomeados no `.gitignore` do repositório público.

| Documento | Papel |
| --- | --- |
| `docsprivate/GUIAIA.md` | Mapa operacional entre conhecimento, módulo, arquivo e gate. |
| `docsprivate/ContextoIA.md` | Estado real, decisões vigentes e direção por sessão (1798 linhas; maior densidade de marcadores). |
| `docsprivate/PONTO_ATUAL.md` | Fila viva, critérios de aceite imediatos e protocolo de reentrada. |
| `docsprivate/AGENTS.md` | Instruções de condução do desenvolvimento. |
| `docsprivate/prompts/GPT_TERMINAL_BOOTSTRAP.md` | Prompt inicial; **obsoleto** (referencia docs removidos). Atualizar ou descartar. |
| `docsprivate/imagens/bugs/*` | Material de trabalho (capturas de regressão). |
| Este `docsprivate/PLANO_ORGANIZACAO_E_HANDOFF.md` | Faixa X: plano interno; não é publicado. |

## 5. Árvore-alvo da documentação

Estrutura proposta para o **repositório privado** após a reorganização. A cópia
limpa (Parte 6) é um subconjunto por *allowlist* desta árvore.

```text
raiz/
├── README.md                     (P) apresentação e estado
├── MANUAL.md                     (P) uso da IDE
├── Tutorial.md                   (P) instalação/distribuição/atualização
├── LICENSE-MIT.txt · LICENSE-APACHE-2.0.txt   (P)
│
├── docs/
│   ├── README.md                 (T) índice técnico único
│   ├── guia-execucao.md          (T) ex-COMO_EXECUTAR.md
│   ├── arquitetura/              (T) ARCHITECTURE, 02, 03, 06, 19, 15, 17, 16, schemas
│   ├── build-e-qualidade/        (T) COMANDOS_BUILD_VERIFICACAO, 22, 14
│   ├── seguranca-de-dados/       (T) 23 (parte técnica) + ADR-0001
│   ├── specs/                    (T) as 20 especificações-alvo + diagramas
│   ├── iconografia/              (T) índice único dos três pacotes de ícones
│   ├── roadmaps/                 (T) adaptação de componentes, KSWE, 20, 21(téc),
│   │                                 24(téc), 25, 26(téc), BACKEND_TO_UI_UX
│   ├── adr/                      (T) decisões arquiteturais
│   └── tooling/                  (T) OPEN_COMPONENT_REGISTRY.json
│
└── (fora da publicação — camada pessoal, faixa X)
    docsprivate/ContextoIA.md · docsprivate/PONTO_ATUAL.md · docsprivate/GUIAIA.md · docsprivate/AGENTS.md ·
    prompts/ · docsprivate/diario/ (ex-18 e status vivos desmembrados) ·
    docsprivate/PLANO_ORGANIZACAO_E_HANDOFF.md
```

Convenções de nomenclatura para a documentação técnica e pública:

- documentos técnicos passam a nomes descritivos por assunto; o prefixo numérico
  histórico (`02-`, `03-`…) pode ser preservado por continuidade de referências
  ou migrado para nomes temáticos, desde que o índice `docs/README.md` seja
  atualizado no mesmo passo;
- o sufixo/infixo "IA" (`GUIAIA`, `ContextoIA`) fica restrito à camada pessoal;
  nenhum documento das faixas P/T carrega marca de "assistente";
- artefatos gerados (ex.: `dist/Tutorial.md`) não recebem nome de documento-fonte.

## 6. Política de reescrita de tom (condução → documentação técnica)

Aplicável a todo documento faixa T que contenha narrativa de condução. O
objetivo é que o texto informe **um leitor engenheiro**, não que registre **um
assistente executando pedidos do autor**.

Marcadores a eliminar dos documentos técnicos (mantidos apenas na faixa X):

- referências ao processo de condução: "a IA deve", "o agente", "nesta sessão",
  "handoff", "reentrada", "retomar pelo marcador";
- direção pessoal datada: "o usuário pediu/quer/aprovou em 2026-..." — o fato
  técnico é preservado; a atribuição a uma sessão é removida;
- narrativa de dogfooding e de gesto humano: "o usuário abriu por `scripts/...`
  e não percebeu mudança" — pertence ao diário pessoal;
- nomes de ferramentas de assistência quando usadas como interlocutor
  (Claude/Codex/GPT como "quem lê"), preservando-os apenas quando são o
  **produto** descrito (o Bridge de CLI de IA é funcionalidade real);
- segunda pessoa dirigida ao operador ("você deve rodar", "leia antes") — trocar
  por forma descritiva/impessoal.

Regras de reescrita:

1. **Voz:** descritiva e impessoal. Preferir "o core valida o caminho antes de
   escrever" a "você valida" ou "a IA valida".
2. **Tempo:** presente para contrato/comportamento; passado somente para
   decisões registradas, sem atribuição a sessão.
3. **Estado:** substituir "status vivo" e "fatia atual" por marcação estável
   (Implementado / Planejado / Descontinuado) no cabeçalho do documento.
4. **Datas e pedidos:** manter a decisão técnica; remover "a pedido do usuário em
   DD/MM". A autoria e a cronologia ficam no Git e na camada pessoal.
5. **Imperativos de condução → requisitos:** "não recriar terminal paralelo"
   vira "o terminal é uma superfície única; sessões adicionais reusam o
   `TerminalManager`".

Exemplo de conversão (ilustrativo):

```text
Antes (condução):
  DECISÃO OBRIGATÓRIA (usuário, 2026-07-11): adotar as TECNOLOGIAS que os
  plugins do Neovim usam, direto na nossa IDE, SEM depender do Neovim/Vim.

Depois (documentação técnica):
  As tecnologias maduras usadas pelos plugins do ecossistema (Tree-sitter, LSP,
  DAP) são integradas diretamente ao core, sem embutir o editor de origem. A
  integração se dá por biblioteca/protocolo aberto, com UI própria.
```

## 7. Separação pública × pessoal e a nova cópia limpa

A distribuição já está definida em `docsprivate/GUIAIA.md` §9, `docs/21` M7.3 e
`docsprivate/PONTO_ATUAL.md` A6. Esta seção consolida o procedimento e explica por que a
proteção **não** pode depender de `.gitignore`.

**Por que `.gitignore` não protege o material pessoal:**

- `.gitignore` **lista nomes**: publicá-lo revelaria a existência de
  `docsprivate/ContextoIA.md`, `docsprivate/PONTO_ATUAL.md`, `prompts/` etc.;
- `.gitignore` **não remove** o que já foi rastreado no histórico; um arquivo
  antes versionado continua recuperável;
- ignorar um arquivo não impede que uma cópia recursiva da árvore o inclua.

**Mecanismo correto — exportação por *allowlist* para árvore separada:**

```text
1. A árvore privada permanece a fonte completa e privada. Sua visibilidade
   nunca é alterada para gerar uma entrega.
2. A cópia limpa é uma NOVA árvore, gerada por lista explícita de inclusão
   (allowlist), com histórico próprio começando do zero.
3. Entram na cópia: o código do projeto e, entre Markdown, somente README.md,
   MANUAL.md e Tutorial.md. Licenças/atribuições por LICENSE/JSON/TXT.
4. NÃO entram: docsprivate/ContextoIA.md, docsprivate/PONTO_ATUAL.md, docsprivate/GUIAIA.md, docsprivate/AGENTS.md, prompts/,
   docs/ (specs, roadmaps, ADRs, diário), este plano, docsprivate/imagens/bugs/ e qualquer
   nota de condução. Não há entrada correspondente no .gitignore público.
5. .git/ e o histórico privado não entram; um eventual espelho começa com
   histórico próprio da árvore sanitizada.
```

**Checklist do exportador (antes de qualquer push):**

```text
[ ] modo --dry-run executado e a lista final revisada item a item
[ ] apenas README.md, MANUAL.md, Tutorial.md entre os Markdown
[ ] recusa automática de qualquer Markdown fora da allowlist
[ ] auditoria de segredos na árvore exportada (chaves, tokens, caminhos locais)
[ ] nenhum caminho da camada pessoal presente, nem sequer como nome
[ ] histórico novo; ausência de .git/ da fonte
[ ] confirmação explícita do autor da lista de nomes proibidos (A6)
```

**Sequência de decisão do autor (A6):** ao abrir a sessão de criação do
repositório público, o autor fornece a lista exata de nomes de arquivo e
diretórios que não podem aparecer nem como caminho. Até lá, não se cria
repositório, não se altera visibilidade e não se faz push.

## 8. Plano de execução da reorganização (sem tocar em código)

Fases ordenadas; nenhuma envolve alterar código de produto.

```text
Fase 0 — Congelamento e inventário (este documento).
         Classificação das três faixas fechada; nada é movido ainda.

Fase 1 — Agrupamento físico por assunto.
         Criar a árvore docs/ da Parte 5; mover documentos técnicos para os
         subdiretórios temáticos; atualizar docs/README.md como índice único e
         corrigir todos os links internos no mesmo passo.

Fase 2 — Reescrita de tom (Parte 6) nos documentos faixa T marcados.
         Prioridade pela densidade de marcadores: AI_CLI_BRIDGE, 24, 26, 03,
         IMPLEMENTATION_TASKS. Desmembrar 18/21/24/26 conforme a regra 2.1.

Fase 3 — Tratamento de obsoletos e duplicatas.
         Marcar KV_CONTEXT como histórico (ou desmembrar/remover); atualizar ou
         descartar prompts/; consolidar os três pacotes de ícones sob um índice;
         remover dist/Tutorial.md do conjunto de fontes.

Fase 4 — Camada pessoal.
         Recolher os documentos faixa X e os fragmentos desmembrados num
         agrupamento pessoal claro (ex.: docsprivate/diario/), fora da allowlist.

Fase 5 — Geração da cópia limpa (Parte 7), somente quando o autor autorizar e
         fornecer a lista de nomes proibidos (A6/M7.3).
```

Cada fase é verificável: ao fim, `docs/README.md` deve indexar 100% dos
documentos técnicos, nenhum link deve apontar para caminho movido, e a busca
por marcadores de condução nos documentos faixa T deve cair para próximo de zero
(exceto onde a IA é o produto descrito).

---

# Parte II — Handoff linear: completar a IDE

## 9. Natureza e princípios do handoff

Esta parte **consolida e lineariza** o trabalho já decidido em `docsprivate/PONTO_ATUAL.md`
(A3–A5, com a sequência de níveis L0–L10), na trilha T e nos marcos M4–M7 de
`docs/21`, e no roadmap de adaptação de componentes. Não é um roadmap
substituto: é a mesma fila, ordenada em uma única trilha para que a IDE alcance o
estado mais completo possível na ordem de menor risco e maior dependência
satisfeita.

Princípios invioláveis (herdados dos documentos de domínio):

1. **Capacidade arquitetural antes de ferramenta.** Um nível só abre quando o
   contrato e o serviço do nível anterior estão comprovados; a facilidade de
   chamar uma CLI não antecipa sua integração.
2. **Orquestrar, nunca reimplementar.** A IDE integra compiladores, LSP, DAP,
   build systems e ferramentas maduras; não escreve os seus próprios.
3. **Fluxo de camadas.** Qt/QML → CoreClient → protocolo tipado → handler →
   serviço → Job cancelável → ferramenta externa. A UI nunca chama binário ou
   filesystem do workspace diretamente.
4. **Referência profissional obrigatória** (roadmap de adaptação §2.1) antes de
   cada funcionalidade nova: estudar Code OSS, IntelliJ Community, Zed, Lapce ou
   NetBeans para extrair invariantes e modos de falha, sem copiar código.
5. **Sem telemetria; local-first.** Rede apenas por ação explícita do usuário.
6. **Gate por fatia.** `scripts/verificar.sh` + sonda do domínio + gesto real;
   operação longa é Job cancelável; estado visual ganha harness QML.
7. **Nomenclatura interna.** A aba visual pode se chamar "Plugins", mas os
   conceitos internos são `integration` / `capability` / `adapter`; não existe
   *extension host* de código de terceiros.

## 10. Estado atual consolidado (linha de base)

Já implementado e validado (protocolo IPC 0.57.0):

- **Projeto/workspace:** abertura e criação C++/CMake e Rust/Cargo, workspaces
  híbridos (`workspace.capabilities.buildSystems`), recentes fixáveis,
  recuperação de sessão, recuperação de crash do core.
- **Editor:** múltiplas abas, rascunhos com recuperação, escrita atômica +
  drafts em SQLite, Tree-sitter incremental, autocomplete (fallback estrutural →
  semântico), diagnósticos na gutter + sublinhado, navegação, rename, quick fixes
  com preview, format-on-save, EditorConfig **ainda não** integrado.
- **Build/Run/Test/Debug:** CMake (File API/presets) e Cargo como jobs, testes,
  análise (Clippy/clang-tidy no gate), execução, debug via DAP `lldb-dap`.
- **Terminal e IA externa:** PTY multi-sessão (`portable-pty` + VT), Assistente
  como bridge para Claude/Codex CLI instalados pelo usuário.
- **Git diário:** status, diff, stage, commit, branches, pull, push, stash,
  blame, log.
- **Configuração/plataforma:** settings com schema (global + workspace), Project
  Health, controles de janela client-side, empacotamento AppImage reproduzível.
- **Medição:** baseline de performance A3 medida (primeiro frame, RSS, latências)
  dentro dos orçamentos.

Componentes abertos já integrados (MODE-A): clangd, rust-analyzer, lldb-dap,
cargo, CMake (file-api/presets), git, Tree-sitter (runtime + gramáticas C/C++/
Rust), `notify`, `vte`, `linuxdeploy` (build).

Pendência técnica aberta e adiada por decisão do autor: alinhamento vertical do
cursor/TUI no renderer de terminal (retomada apenas por `docs/26` R0, nunca por
ajuste manual de offset).

## 11. Trilha linear única (L0 → L10)

A tabela abaixo é a espinha do handoff. Cada nível entrega **primeiro** a
capacidade arquitetural e **depois** as integrações que a validam. A ordem entre
níveis é vinculante; dentro de um nível, a ordem é sugerida por dependência e
valor diário. Um nível só é promovido com ao menos uma integração vertical real,
testes de falha/cancelamento, orçamento medido e ausência de processo órfão.

| Nível | Capacidade que abre o nível | Integrações/fatias que o validam | Fontes |
| --- | --- | --- | --- |
| **L0** | Fechar a baseline medida A3.1–A3.4 e os orçamentos; nenhuma plataforma nova antes disso. | Métricas de Tree-sitter (primeiro snapshot + incremental), primeira resposta semântica LSP, digitação tecla→frame e rajada PTY→frame, orçamento versionado. | `docsprivate/PONTO_ATUAL.md` A3; `docs/21` M4.2 |
| **L1** | Editor profissional diário + `integration` v1 (descriptor/health/config, aba informativa lendo o registry existente). | EditorConfig (primeira integração pequena); inventário das ferramentas já detectadas (clangd, rust-analyzer, CMake, Cargo, Git, rg, fd, lldb-dap, Clippy); confortos A4 (split editor, multicursor, zoom, busca/links no scrollback, seleção de palavra por duplo clique no terminal); trilha T de editor (T4 cargo-check-no-save, T2 clang-tidy no editor, T7 runnables/test explorer, T3 inlay hints após a decisão de engine M5.4, T9 crates inline, T8 utilidades). | `docsprivate/PONTO_ATUAL.md` A4/A5.1–A5.2; `docs/21` M5, trilha T; roadmap §11 (EditorConfig, ripgrep, fd) |
| **L2** | Resultado comum para diagnóstico/teste/cobertura, com artefatos e Jobs canceláveis. | cargo-audit/cargo-deny; um analisador C/C++ além de clang-tidy (Cppcheck **ou** Clang Static Analyzer); Valgrind/Memcheck; descoberta/consumo de GTest/Unity/Criterion via CTest; gcov/lcov; cobertura Rust por `cargo-llvm-cov` (Tarpaulin como fallback). | `docsprivate/PONTO_ATUAL.md` A5.2–A5.3; roadmap Neotest/Error Lens/Trouble |
| **L3** | Project Graph/targets/perfis explicáveis, *provenance* de cache e geração de artefatos (KSWE B1/B2). | CMake File API completa (codemodel) + Cargo Metadata → Project Graph + Context Matrix; targets/toolchains como entidades; Bear (apenas fallback de compile database); ccache/sccache; Bloaty; Doxygen; Sphinx/Breathe. | `docs/21` "simbiose", `docsprivate/PONTO_ATUAL.md` B1/B2, A5.3; KSWE workflow |
| **L4** | DAP sólido, sessão de profiling, importador de relatório, permissões de kernel (KSWE B3/B4 + M6.2). | Consolidar DAP (`lldb-dap`, avaliar `gdb-dap`) com watches/variáveis/pilha/pretty-printers; scheduler LSP + Symbol/Diagnostic Broker + Effective Compile Context; Heaptrack; perf/Hotspot; tokio-console; MI/ELF(`goblin`)/DWARF(`gimli`) só por lacuna; adapters de debug configuráveis. | `docsprivate/PONTO_ATUAL.md` B3/B4, A5.2–A5.3; `docs/21` M5.3/M6.2 |
| **L5** | `RemoteContext`: host keys, credenciais externas, mapeamento de caminho, desconexão, sync e Jobs remotos. | OpenSSH host profiles + terminal remoto + rsync/scp/sftp + port forwarding + gdbserver; SSHFS e bindings SSH (`libssh`/`ssh2-rs`) só como alternativas; Dev Container CLI. | `docsprivate/PONTO_ATUAL.md` A5.2; roadmap Open Remote SSH/Dev Container |
| **L6** | `Target`/`Device`/`Probe`, detecção USB, *package trust*, preview e confirmação de flash. | udev/udevadm; CMSIS-DAP/CMSIS-Pack; DTS/DTB; OpenOCD; pyOCD; probe-rs; avrdude/esptool/stlink; QEMU (local-first); geração por template (Tera). | `docsprivate/PONTO_ATUAL.md` A5.2–A5.3; roadmap embarcados; spec EMBEDDED_TARGETS |
| **L7** | Streaming com backpressure, timestamp, canais, retenção e segurança de rede/dispositivo. | sigrok (motor/CLI); SWO/ITM; CTF/LTTng; MQTT/CoAP/Mosquitto (opt-in); lm-sensors; D-Bus com interfaces allowlisted. | `docsprivate/PONTO_ATUAL.md` A5.2–A5.3 |
| **L8** | Armazenamento medido e API de visualização isolada do editor/core. | Banco local ou de séries temporais escolhido por benchmark (não somado); subsistema de rendering OpenGL/Qt; computação numérica específica (`ndarray`) por feature. | `docsprivate/PONTO_ATUAL.md` A5.2–A5.3 |
| **L9** | Sandbox de pacote/binário, compatibilidade de arquitetura e co-simulação reproduzível. | FMI/FMU (fundação) antes de OpenModelica/OMSimulator; Wokwi apenas opt-in externo. | `docsprivate/PONTO_ATUAL.md` A5.2–A5.3 |
| **L10** | Laboratório opt-in, sem promessa de suporte diário. | SCIP/índice persistente (avaliar antes de LSIF); Ghidra; libclang; gcov-kernel/kcov; Frama-C/Kani; OTAWA/WCET; distcc. | `docsprivate/PONTO_ATUAL.md` A5.2–A5.4 |

## 12. Trilhas transversais (paralelas, mas com gate próprio)

Estas frentes não pertencem a um único nível; avançam em paralelo, condicionadas
ao seu próprio critério de maturidade:

- **Decisão de engine do editor (M5.4).** Design + medição, resultado é um
  documento de decisão. Destrava inlay hints reais (T3), split view avançado,
  minimap e multicursor pleno. É pré-requisito de parte de L1 e deve ocorrer
  cedo, logo após L0.
- **Refatorações sobre o LSP (M5.1).** organize imports, extract function/
  variable, inline — somente o que o servidor expõe; exige `codeAction` com
  range e `applyEdit` multi-arquivo robusto. Encaixa em L1.
- **Navegação pesada (M5.3).** call/type hierarchy, recent files, bookmarks.
  Encaixa em L1/L4.
- **Task runner do usuário (M6.3)** e **language servers configuráveis (M6.1).**
  Ampliam a plataforma `integration`; encaixam em L1/L2.
- **Ponte de contexto do Assistente (M6.4).** Comando "copiar contexto do
  workspace" em formato colável, sem chamada de rede da IDE. Fatia pequena,
  independente de nível.
- **Distribuição e comunidade (M7).** Matriz de release (Ubuntu/Fedora), CI
  pública espelhando `scripts/verificar.sh`, exportador *allowlist* (Parte 7 +
  A6), diagnóstico de falhas local sem telemetria. Só amadurece com M4 robusto;
  a exportação depende da autorização do autor.

## 13. Critério de pronto por fatia (resumo operacional)

Cada fatia, em qualquer nível, segue o mesmo ritual:

```text
1. Ler o domínio (GUIAIA + fontes) e cumprir a referência profissional §2.1.
2. Desenhar a fatia (contrato IPC, arquivos, testes, o que fica fora) antes do
   código.
3. Implementar na ordem protocolo → core/serviço → handler → testes → UI.
4. Operação longa vira Job cancelável; nada bloqueia a UI.
5. Gate: scripts/verificar.sh + sonda do domínio + smoke offscreen; harness QML
   quando houver estado visual.
6. Gesto real em tela quando for UI/terminal; para packaging, os testes de
   AppImage.
7. Sincronizar contrato/schema/manual/arquitetura afetados e registrar a
   conclusão; remover o item da fila viva.
```

Gate de promoção de nível (L0→L10): uma integração vertical real comprovada,
testes de falha/cancelamento, orçamento medido, configuração reversível e
nenhum processo/handle órfão. A aba visual "Plugins" **não** desbloqueia nível
algum; o contrato e o serviço comprovados desbloqueiam.

## 14. Próximo passo imediato

Conforme `docsprivate/PONTO_ATUAL.md` (PRÓXIMO GESTO), e sem reabrir fatias já aceitas:

1. Implementar **A3.1** (estrutura local Tree-sitter) estendendo apenas
   `medir-core.py`/`medir-performance.sh`; registrar mediana e orçamento em
   `docs/21`. Seguir A3.2–A3.4 para fechar **L0**.
2. Encerrada A3, auditar **EditorConfig** como primeira integração pequena de
   **L1** (confirmar biblioteca/licença/contrato e conflito com settings antes
   do código); iniciar `integration` v1 junto, informativo.
3. Dogfooding em tempo integral: cada atrito ou saída para outra ferramenta vira
   o topo do backlog (ação/esperado/observado/ambiente), à frente de qualquer
   nível.

---

## Apêndice A — Resumo da classificação por faixa

- **Faixa P (público / cópia limpa):** `README.md`, `MANUAL.md`, `Tutorial.md`
  (+ licenças em TXT). Total de Markdown público: **3**.
- **Faixa T (interno-técnico / repositório privado):** `docs/` (arquitetura,
  build, segurança, specs, iconografia, roadmaps técnicos, ADRs, tooling),
  `COMO_EXECUTAR.md`, `ui/README.md`, os três pacotes de ícones e os roadmaps de
  adaptação/KSWE. É o grosso do acervo.
- **Faixa X (pessoal / fora da distribuição, nunca no `.gitignore` público):**
  `docsprivate/ContextoIA.md`, `docsprivate/PONTO_ATUAL.md`, `docsprivate/GUIAIA.md`, `docsprivate/AGENTS.md`, `prompts/`,
  `docs/18` (diário) e os fragmentos de status vivo desmembrados de `21/24/26`,
  `docsprivate/imagens/bugs/`, e este `docsprivate/PLANO_ORGANIZACAO_E_HANDOFF.md`.

## Apêndice B — Documentos com maior necessidade de reescrita de tom

Por densidade de marcadores de condução medida no acervo (ocorrências):

```text
docsprivate/ContextoIA.md ......................................... 93   (faixa X — não reescrever, isolar)
docs/specs/..._AI_CLI_BRIDGE_EXTERNAL_TERMINAL.md ..... 83   (faixa T — reescrita intensa)
docs/24-paridade-e-fundacao.md ........................ 63   (faixa T/X — desmembrar)
docsprivate/PONTO_ATUAL.md ........................................ 53   (faixa X — não reescrever, isolar)
docs/26-terminal-rendering-parity-roadmap.md .......... 53   (faixa T/X — desmembrar)
docs/18-daily-driver-plan.md .......................... 53   (faixa X — diário)
docs/03-ipc-protocol.md ............................... 37   (faixa T — reescrita pontual)
docs/specs/..._KV_CONTEXT_AI_ASSISTANCE.md ............ 32   (faixa T — histórico/superado)
```

A contagem é indicativa: no `AI_CLI_BRIDGE` e no `KV_CONTEXT` parte dos
marcadores é legítima (IA é o produto descrito). A reescrita separa o produto
(mantido) da narrativa de condução (removida).

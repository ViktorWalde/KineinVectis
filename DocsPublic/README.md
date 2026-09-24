# Documentação da Kinein Vectis

Este índice organiza a documentação técnica do projeto por assunto e define a
ordem de precedência quando houver conflito entre documentos.

## Como a documentação está organizada (2026-09-12)

Duas árvores, por decisão do autor em 2026-09-12 (sucede a decisão das três
árvores de 2026-08-29 — a legada virou uma pasta dentro da privada):

```text
DocsPublic/      TODA a documentacao do projeto, versionada. Uma sessao de
                 trabalho le esta pasta e mais nada: ela e' auto-suficiente.
  README.md                 este indice: o mapa, a precedencia, as classes
  leitura-tecnica.md        primeira leitura de quem e' novo: o que existe, medido
  contribuindo.md           como contribuir (versão curta)
  contribuindo/             o guia completo de quem chega, com ou sem IA (2026-09-19)
  arquitetura/              contrato de engenharia (ARCHITECTURE.md), estrutura do
                            repositorio, protocolo IPC, boot, rigor, modulos
  roadmaps/                 planos e o ESTADO vivo (40 = a fila; 42 = a trilha)
  especificacoes/           a visao-alvo do produto (layout, editor, fluxos,
                            embarcados, onboarding) — alvo, nao estado
  integracoes/              as ferramentas abertas que a IDE orquestra: licenca,
                            versao, forma (crate ou processo); o registro JSON
  build/                    ambiente de desenvolvimento, compilacao, comandos
  seguranca/                rede de seguranca de dados, cofre de credencial
  decisoes-adr/             decisoes de arquitetura datadas (ADR-0001..0005)
  iconografia/              o sistema visual e os icones (assets com checksum)

DocsPrivate/     NAO versionada (.gitignore): continuidade interna do autor —
                 o log datado (ContextoIA), o diario, os prompts de retomada
                 das sessoes com IA, o historico do que saiu do produto
                 (historico/simulacao/, historico/PONTO_ATUAL.md) e o legado
                 cancelado (legado/). Consulta-se sob demanda para "por que
                 isto ficou assim?" — nunca para descobrir "o que existe".
```

**Regras de nome:** pasta com nome explícito em português; documento com
número de ordem + nome explícito (`40-estado-e-continuidade.md`) — o número é
o sistema de citação do projeto ("o 40 §7.21") e não se remove; os nomes em
caixa alta de 2026-07 viraram kebab-case em 2026-09-12 (`leitura-tecnica.md`,
`especificacoes/sistema-de-layout.md`). `ARCHITECTURE.md` e `README.md` são
as duas exceções convencionais.

**Por que `DocsPrivate/legado/` existe.** Há documentos **grandes, completos e
persuasivos** de features **canceladas** (a linha inteira de IA na IDE) que
uma sessão nova encontraria dentro de `especificacoes/` e leria como alvo.
Deletar perderia o registro de uma decisão de produto; deixar em
`especificacoes/` é convidar a reimplementação. Regra de mão única: **um
documento entra em `legado/` e não volta.** Se algo lá dentro voltar a valer,
o conteúdo é extraído para o documento vivo relevante.

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
`DocsPrivate/legado/PLANO_ORGANIZACAO_E_HANDOFF.md`. Ele responde outra pergunta: *"se este arquivo
envelhecer, o que acontece?"*

| Classe | Regra | O que acontece se envelhecer | Onde |
| --- | --- | --- | --- |
| **CONTRATO** | Não muda sem decisão explícita e registrada. **Não contém número medido nem inventário** — número é o que apodrece. | Nada: é regra, não estado. | `AGENTS.md`, `arquitetura/ARCHITECTURE.md` §2/§4/§5, `adr/` |
| **ESTADO** | Tem que ser verdade **agora**. Todo número é verificável contra o disco. | **Mente.** Manda a próxima sessão reimplementar o que existe. | `roadmaps/40` (a fila), `leitura-tecnica.md`, `GUIAIA.md` (os mapas), `arquitetura/02`, `arquitetura/03` |
| **PLANO** | Descreve o alvo. Pode divergir da implementação — é para isso que existe. | Aceitável, mas reconciliar ao retomar. | `DocsPublic/especificacoes/`, `DocsPublic/roadmaps/` |
| **LOG** | Registro datado do que foi decidido **naquele dia**. Nunca reescrever. | Nada: envelhecer é a função dele. | `DocsPrivate/ContextoIA.md`, `diario/`, `adr/` |

### Por que o `DocsPrivate/historico/PONTO_ATUAL.md` saiu de "fila viva" (2026-09-10)

Ele era o primeiro item da linha ESTADO e a §1 do `GUIAIA.md` mandava lê-lo como
*"próxima tarefa executável"*. **Medido em 2026-09-10, ele dizia protocolo
`0.62.0`, 13 gates, 378 testes Rust e 22 arquivos em débito** — contra `0.88.0`,
19, 666 e 1 no disco. Pior que os números: a §TRILHA dá o `E1` como fechado em
2026-08-29 e a seção `PRÓXIMO GESTO`, 300 linhas abaixo, ainda o descreve como
aberto, com caminho de conserto que o código não seguiu.

O papel de fila passou para
[`roadmaps/40-estado-e-continuidade.md`](roadmaps/40-estado-e-continuidade.md),
que já era anunciado como "COMECE POR AQUI" desde 2026-09-04 — a mudança aqui só
para de mandar a sessão para dois lugares. **O `DocsPrivate/historico/PONTO_ATUAL.md` não foi legado
nem esvaziado:** ele continua sendo onde mora o *porquê* de decisões que nenhum
outro documento carrega, e o cabeçalho dele agora diz exatamente isso.

**A lição é a mesma da nota abaixo, num eixo diferente:** um documento que
*já foi* estado não vira log sozinho quando para de ser atualizado — ele vira
mentira, e continua com o crachá de estado no pescoço.

### Por que o `DocsPrivate/ContextoIA.md` saiu de "estado real" (2026-07-17)

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
[contribuindo.md](contribuindo.md): arquitetura em uma tela, tabela de "quero
mudar X → olhe aqui", ritual de uma mudança e o gate.

Para localizar os arquivos de cada domínio, use [contribuindo.md](contribuindo.md)
§3. Para a ordem de execução, use o [roadmap 40](roadmaps/40-estado-e-continuidade.md)
§4.1: Etapa 1 de backend/toolchains (fechada em 2026-09-18) e Etapa 2 de
UX/UI/HUD (F0–F8 feitas em 2026-09-18; resta o fechamento, 40 §4.2.2). **O que ainda falta, por
classe — fatia de código / prova com hardware / só o autor — está no §4.2
do mesmo roadmap**, e o estado da Etapa 2 fatia a fatia no
[roadmap 43](roadmaps/43-etapa2-hud-ui-ux.md) §7. O antigo
`GUIAIA.md` não está neste checkout (conferido em 2026-09-15).

## Estrutura

```text
DocsPublic/
├── arquitetura/   contrato de engenharia, protocolo IPC, strict mode, dívida e higiene
├── build/         ambiente, comandos de compilação e gate de verificação
├── seguranca/     rede de segurança de dados (escrita atômica + drafts)
├── roadmaps/      planos de execução, roadmap de longo prazo, adaptação e KSWE
├── specs/         especificação canônica (visão-alvo) + diagramas
├── integracoes/   como adicionar/escalar uma integração ("Plugins")
├── adr/           decisões arquiteturais registradas
├── tooling/       registro auditável de componentes open-source
└── iconografia/   sistema visual, ícones de arquivo e da árvore

DocsPrivate/
├── ContextoIA.md  log datado (por que), nunca estado (o que existe)
├── diario/        registro de sessões
└── prompts/       bootstrap de retomada em terminal

DocsPrivate/legado/       superado ou cancelado; não implementar a partir daqui
```

## Público × interno

Uma cópia entregue a terceiros leva o código e **somente três Markdown**:
`README.md`, `DocsPublic/manual.md` e `DocsPublic/tutorial.md`. Todo o resto — inclusive esta pasta
`DocsPublic/` inteira — é interno.

Isso não depende de disciplina: `scripts/exportar-copia-limpa.sh` gera a cópia
por allowlist numa árvore separada, recusa Markdown extra, verifica que nenhum
caminho interno aparece nem como nome, audita segredos e imprime a lista final.
Roda em dry-run por padrão e **não cria repositório nem publica nada**.

## Entrada para quem vai mexer no código

| Documento | Assunto |
| --- | --- |
| [leitura-tecnica.md](leitura-tecnica.md) | **Comece por aqui se você é novo no projeto** (remedida em 2026-09-10): o que a IDE é e não é, o peso medido de cada camada, o que existe de verdade por domínio, onde a arquitetura está sob tensão e o que a direção escolhida custa |
| [contribuindo.md](contribuindo.md) | **Onde olhar para alterar/implementar**: arquitetura, mapa por área, ambiente, ritual da mudança, gate e convenções |
| [contribuindo/](contribuindo/README.md) | **O guia completo de quem chega** (2026-09-19): a ideia e o que o projeto recusa ser, o ambiente e os presets, o ritual de uma fatia, cada gate e o porquê, trabalhar com um agente de IA (e sem), o mapa por área |
| [integracoes/README.md](integracoes/README.md) | **Entrada obrigatória para adotar qualquer ferramenta**: modos A–D, gate de auditoria, níveis L0–L10, checklist de 10 passos e o índice do que já está adotado |
| [integracoes/36-ferramentas-de-embarcados.md](integracoes/36-ferramentas-de-embarcados.md) | **Levantamento de embarcados** (2026-09-03): probe-rs, OpenOCD, pyOCD e QEMU com licença verificada na fonte e — o que decide o desenho — qual protocolo cada uma fala. Candidatas, **não** adotadas |
| [integracoes/37-banco-e-observabilidade.md](integracoes/37-banco-e-observabilidade.md) | **Levantamento de banco e observabilidade** (2026-09-03): Grafana (AGPL-3.0) e TimescaleDB (Apache-2.0 + Timescale License). A licença do Grafana decide a **forma** da integração; a do TimescaleDB tem parte não-OSI, e isso está dito |
| [integracoes/38-conectividade-bare-metal.md](integracoes/38-conectividade-bare-metal.md) | **Conectividade bare metal, medida com um ESP32 no USB** (2026-09-11): os três canais (serial, depuração, massa/DFU) e a camada de identidade que valem para ESP32, STM32, RP2040 e Pi; o que a placa plugada permite hoje (gravar, monitorar) e o que não (depurar); as ferramentas com licença lida no arquivo — e por que o `espflash` como crate não passa no `deny.toml` |
| [integracoes/39-toolchains-por-alvo.md](integracoes/39-toolchains-por-alvo.md) | **Toolchains por alvo, o catálogo credível** (2026-09-12): o levantamento recebido conferido linha a linha na fonte (o que está certo, o que está desatualizado — Espressif unificada, LLVM Embedded superado pelo ATfE —, o que faltava — Zephyr SDK, o sysroot), o catálogo por universo (bare metal, Linux embarcado, Rust, depuração) com versão e licença lidas, onde a IDE procura além do `PATH`, e a regra do provedor de instalação: um clique, checksum publicado, pasta da IDE, nunca o sistema |

## arquitetura/ — contrato e estado implementado

| Documento | Assunto |
| --- | --- |
| [arquitetura/ARCHITECTURE.md](arquitetura/ARCHITECTURE.md) | **LEITURA OBRIGATÓRIA — contrato de arquitetura (camadas, regra de split, crescimento). Verificado por catraca. Antes de propor arquitetura nova: MEDIR — o problema costuma ser regra não cumprida, não regra ausente (§1.1)** |
| [arquitetura/02-repository-structure.md](arquitetura/02-repository-structure.md) | **Estrutura real do repositório e crates** (remedida em 2026-09-10): a árvore como ela é, os comandos que a conferem, e o registro de quando ela mentiu — listava uma pasta `templates/` que nunca existiu |
| [arquitetura/03-ipc-protocol.md](arquitetura/03-ipc-protocol.md) | Protocolo IPC JSON-RPC implementado — a **forma** de cada mensagem, por domínio |
| [arquitetura/04-boot-e-comunicacao.md](arquitetura/04-boot-e-comunicacao.md) | **Boot e comunicação, fim a fim**: quem sobe quem, as threads do core e o que fala com quais, o caminho de uma requisição e de um evento, o que é e o que NÃO é garantido em ordem, crash e recuperação, e como falar com o core na mão |
| [arquitetura/06-strict-mode.md](arquitetura/06-strict-mode.md) | Strict mode (Rust e C++/Qt) |
| [arquitetura/15-engineering-debt-and-refactor.md](arquitetura/15-engineering-debt-and-refactor.md) | Dívida técnica e modularização |
| [arquitetura/16-hidden-risks-checklist.md](arquitetura/16-hidden-risks-checklist.md) | Riscos ocultos (dados, config, segurança de comandos, segredos, a11y, observabilidade, packaging) |
| [arquitetura/19-architecture-tradeoffs.md](arquitetura/19-architecture-tradeoffs.md) | Requisitos e trade-offs de arquitetura (o porquê das decisões) |
| [arquitetura/27-modulos-por-dominio.md](arquitetura/27-modulos-por-dominio.md) | Módulos por domínio. **Parcialmente entregue** (a catraca do core saiu daqui); resta a Frente 1 — `<X>Domain` na UI, o caminho para o `Main.qml` sair do débito |
| [arquitetura/32-editor-por-responsabilidade.md](arquitetura/32-editor-por-responsabilidade.md) | **O editor cortado por responsabilidade** (2026-09-02): o pagamento do maior débito do repositório, os quatro donos que nasceram, as invariantes que cada um guarda — e a decisão que ficou EM ABERTO, com o custo medido das duas saídas |
| [arquitetura/33-busca-no-projeto.md](arquitetura/33-busca-no-projeto.md) | **Os TRÊS buscadores e o casamento multi-linha** (2026-09-02): qual é qual e por que confundi-los é o defeito clássico, como a busca passou a casar no conteúdo, a invariante "preview conta o que a escrita faz", e por que a sintaxe `\n` mora na UI e não pode descer para o core |
| [arquitetura/35-crescer-sem-god-object.md](arquitetura/35-crescer-sem-god-object.md) | **ESTUDO, não plano aprovado** (2026-09-10): a análise da arquitetura contra o crescimento. O que está saudável e não se deve mexer, o god object que a catraca de ARQUIVO não vê (15 dos 24 handlers tocam 0 ou 1 campo do `Core`), o que a pesquisa externa recomenda — e as **duas ferramentas que ela recomenda e este projeto deve recusar**, com o motivo medido |

## build/ — ambiente, compilação e verificação

| Documento | Assunto |
| --- | --- |
| [build/14-development-environment.md](build/14-development-environment.md) | Ambiente de desenvolvimento |
| [build/22-compilacao-c-cpp-rust.md](build/22-compilacao-c-cpp-rust.md) | Referência prática de comandos de compilação C/C++ e Rust mapeados para a IDE |
| [build/comandos-de-build-e-verificacao.md](build/comandos-de-build-e-verificacao.md) | Gate único de build e verificação |

## seguranca/ — segurança de dados

| Documento | Assunto |
| --- | --- |
| [seguranca/23-rede-de-seguranca.md](seguranca/23-rede-de-seguranca.md) | Rede de segurança contra perda de dado (escrita atômica + autosave em SQLite) |
| [seguranca/40-cofre-de-credencial.md](seguranca/40-cofre-de-credencial.md) | **Decisão registrada** (2026-09-04): a IDE guarda o PERFIL, nunca a senha — onde mora a senha de banco. As três saídas com custo medido — delegar ao `.pgpass`/ambiente, Secret Service do freedesktop (+87 crates, licenças OK), ou cofre próprio (descartado) —, o modo de falha que o Code OSS paga há anos, e o critério de aceite. O Secret Service fica adiado, não descartado: o gatilho é o registro de saídas do dogfooding |

## roadmaps/ — planos de execução e visão de longo prazo

| Documento | Assunto |
| --- | --- |
| [roadmaps/48-arquitetura-executavel-da-serie-0.3.md](roadmaps/48-arquitetura-executavel-da-serie-0.3.md) | **Arquitetura executável da série 0.3 até a 0.3.5:** contratos, donos de estado, trem proposto, migração, rollback e provas; Grafana está confirmado na 0.3.5 |
| [roadmaps/47-estrutura-da-v0.3.md](roadmaps/47-estrutura-da-v0.3.md) | **Estrutura de produto da série v0.3:** cruza frontend, Remote SSH, terminal, Grafana e o mínimo da Etapa 4 até o fechamento 0.3.5 |
| [roadmaps/46-frontend-0.3-em-diante.md](roadmaps/46-frontend-0.3-em-diante.md) | **Frontend da 0.3 em diante:** extração operacional de `arquiKinein`, reconciliada com a `main`; Remote SSH utilizável, commands, tool windows mínimas, abas com identidade e área direita, sem big-bang |
| [roadmaps/backend-para-ui-ux.md](roadmaps/backend-para-ui-ux.md) | Ponte operacional backend → UI/UX |
| [roadmaps/20-ui-spec-convergence-plan.md](roadmaps/20-ui-spec-convergence-plan.md) | Convergência vinculante da UI atual para as specs (fatias C0–C6) |
| [roadmaps/21-long-horizon-roadmap.md](roadmaps/21-long-horizon-roadmap.md) | M4–M7, KSWE, distribuição e continuidade longa |
| [roadmaps/24-paridade-e-fundacao.md](roadmaps/24-paridade-e-fundacao.md) | Fases D1–D4: completion, terminal, Tree-sitter e remake |
| [roadmaps/25-syntax-tree-semantic-foundation.md](roadmaps/25-syntax-tree-semantic-foundation.md) | Contrato da camada sintática (Tree-sitter incremental, composição com LSP) |
| [roadmaps/29-verticais-de-linguagem.md](roadmaps/29-verticais-de-linguagem.md) | **Verticais C/C++, Rust e Python medidas**: o que existe hoje por linguagem, por que Python é reconhecido e ignorado, o que falta para C/C++ sem atrito, e onde está o risco proprietário real (Pylance) |
| [roadmaps/30-caminho-para-o-mvp.md](roadmaps/30-caminho-para-o-mvp.md) | **As etapas para o MVP, em ordem linear** (decidida em 2026-08-30): o que falta medido item por item contra a spec de MVP, e a ordem por dependência |
| [roadmaps/34-depois-do-mvp.md](roadmaps/34-depois-do-mvp.md) | **Sucessor do 30, o pós-MVP** (2026-09-02): as quatro frentes — dívida que cobra pedágio, atrito diário medido, profundidade (TR2) e a simulação —, o estado medido item a item, a ordem linear recomendada e o comando que decide se cada item ainda existe |
| [roadmaps/35-ambiente-cpp-e-embarcados.md](roadmaps/35-ambiente-cpp-e-embarcados.md) | **Sucessor parcial do 34** (2026-09-03): ambiente C/C++ facilitado (catálogo de bibliotecas curado e auditado, `find_package`/`FetchContent` pinado), embarcados reordenado de L6 por decisão registrada, e a simulação — com as decisões de escopo tomadas pelo autor e o que já existe medido |
| [roadmaps/42-trilha-profunda-embarcados.md](roadmaps/42-trilha-profunda-embarcados.md) | **A trilha PROFUNDA de embarcados** (2026-09-12), sucessora do 41 nessa parte: o que a IDE lê de um projeto hoje (medido — e por que isso é a primeira lacuna), a análise do que se repete em todos os levantamentos (identidade/modelo, processo, canal, permissão), oito pilares com critério de pronto POR FAMÍLIA — modelo do projeto, ambiente, ciclo MCU, depuração, Python no embarcado, teste, Linux embarcado (SSH/Yocto/Buildroot/container por dentro), Rust —, a ordem e as três decisões que pede ao autor (respondidas); e, desde a tarde de 2026-09-12, o **"efeito JetBrains" como critério de pronto** (§8: o que já existe medido, o que falta por pilar, com ferramenta e licença) e a **trilha Python completa** bare metal → edge → backend → banco (§9) |
| [roadmaps/41-ecossistema-embarcados-e-python.md](roadmaps/41-ecossistema-embarcados-e-python.md) | **O ecossistema aberto de embarcados e Python em ORDEM LINEAR** (2026-09-11): o que o VS Code oferece para Python, C/C++, Rust e embarcados, a ferramenta aberta por trás de cada funcionalidade com licença lida no arquivo, o que a IDE já tem medido, o que NÃO entra (Pylance, Serial Monitor fechado, cpptools proprietário…) e os seis blocos A–F. Registra a decisão do autor que reverte o "Python adiado" |
| [roadmaps/40-estado-e-continuidade.md](roadmaps/40-estado-e-continuidade.md) | **COMECE POR AQUI ao retomar — é a FILA VIVA** (remedido em 2026-09-10, com o gate verde): o estado medido, o único arquivo que resta na catraca e por que ele não se corta, o que cada sessão entregou, o que está aberto e as decisões que não se reabrem. Ele substituiu o `DocsPrivate/historico/PONTO_ATUAL.md` nesse papel |
| [roadmaps/39-divida-tecnica-paga.md](roadmaps/39-divida-tecnica-paga.md) | **O registro da dívida paga** (2026-09-04): a dívida da catraca paga de 8 arquivos para 1 — cada corte com a pergunta que o justifica, as regras que saíram da UI, as duplicações que já tinham divergido, e o único arquivo restante com as duas saídas medidas para o autor decidir |
| [roadmaps/38-divida-restante-e-continuidade.md](roadmaps/38-divida-restante-e-continuidade.md) | **Superado pelo 39** na parte de dívida (2026-09-03): registro de como a fila estava quando o contexto acabou. A §4 (o que está aberto e NÃO é dívida) continua valendo |
| [roadmaps/29-verticais-de-linguagem.md](roadmaps/29-verticais-de-linguagem.md) | **Verticais C/C++, Rust e Python, medidas**: por que Python é reconhecido e ignorado, o que falta para C/C++ sem atrito, e onde está o risco proprietário real (Pylance) — com fontes citadas |
| [roadmaps/28-plataforma-de-plugins-e-verticais.md](roadmaps/28-plataforma-de-plugins-e-verticais.md) | **Plataforma de plugins (`integration` v1) e as verticais**: C/C++/Rust sólidos, Docker e banco como domínios NATIVOS, embarcados — e a dívida contínua de UI/UX com o IntelliJ Community como referência adaptada |
| [roadmaps/26-terminal-rendering-parity-roadmap.md](roadmaps/26-terminal-rendering-parity-roadmap.md) | Paridade de renderização/scroll do terminal: reprodução instrumentada, métricas de célula/DPR e gates (R0–R7) |
| [roadmaps/adaptacao-de-plugins-abertos.md](roadmaps/adaptacao-de-plugins-abertos.md) | **Norte de adoção e referência open-source**: modos A–D, gate/licenças e política de estudo de Code OSS, IntelliJ, Zed, Lapce e NetBeans |
| [roadmaps/motor-semantico-profundo-cpp-rust.md](roadmaps/motor-semantico-profundo-cpp-rust.md) | Desenho profundo do KSWE (C++/Rust, scheduler, brokers e contextos) |

## specs/ — especificação canônica (visão-alvo)

Fonte de verdade de produto, UX, sistema visual e arquitetura-alvo. Comece pelo
índice: [especificacoes/indice-das-especificacoes.md](especificacoes/indice-das-especificacoes.md).

Âncoras principais:

| Área | Spec |
| --- | --- |
| Arquitetura de frontend 0.3+ | [especificacoes/arquitetura-de-frontend-0.3-em-diante.md](especificacoes/arquitetura-de-frontend-0.3-em-diante.md) |
| Remote SSH — UI/HUD diário | [especificacoes/remote-ssh-ui-hud.md](especificacoes/remote-ssh-ui-hud.md) |
| Terminal — ergonomia da 0.3 | [especificacoes/terminal-ergonomia-0.3.md](especificacoes/terminal-ergonomia-0.3.md) |
| Grafana — uso prático na 0.3.5 | [especificacoes/grafana-ui-ux-0.3.5.md](especificacoes/grafana-ui-ux-0.3.5.md) |
| Markdown — preview da série 0.3 | [especificacoes/markdown-preview-0.3.md](especificacoes/markdown-preview-0.3.md) |
| Arquitetura interna (Core/IPC/Jobs) | [especificacoes/arquitetura-interna-core-ipc-jobs.md](especificacoes/arquitetura-interna-core-ipc-jobs.md) |
| Layout principal | [especificacoes/sistema-de-layout.md](especificacoes/sistema-de-layout.md) |
| Componentes UI | [especificacoes/sistema-de-componentes-de-ui.md](especificacoes/sistema-de-componentes-de-ui.md) |
| Sistema visual / iconografia | [especificacoes/sistema-visual-e-icones.md](especificacoes/sistema-visual-e-icones.md) |
| Build / Run / Debug | [especificacoes/fluxos-de-produto-build-run-debug.md](especificacoes/fluxos-de-produto-build-run-debug.md) |
| Editor / Language Intelligence | [especificacoes/editor-e-inteligencia-de-linguagem.md](especificacoes/editor-e-inteligencia-de-linguagem.md) |
| Configuration Actions | [especificacoes/acoes-de-configuracao-com-escopo-e-links-de-documentacao.md](especificacoes/acoes-de-configuracao-com-escopo-e-links-de-documentacao.md) |
| Fechamento / MVP / Performance | [especificacoes/finalizacao-mvp-e-checklist-de-polimento.md](especificacoes/finalizacao-mvp-e-checklist-de-polimento.md) |

## adr/ e tooling/

| Documento | Assunto |
| --- | --- |
| [adr/ADR-0001-notify-filesystem-watcher.md](decisoes-adr/ADR-0001-notify-filesystem-watcher.md) | Adoção do watcher `notify` e barreira compare-before-save |
| [adr/ADR-0002-tree-sitter-syntax-foundation.md](decisoes-adr/ADR-0002-tree-sitter-syntax-foundation.md) | Adoção do Tree-sitter e fronteira com LSP |
| [adr/ADR-0003-linuxdeploy-appimage-packaging.md](decisoes-adr/ADR-0003-linuxdeploy-appimage-packaging.md) | Empacotamento AppImage, pins, baseline Linux e auditoria |
| [adr/ADR-0004-alacritty-terminal-emulator.md](decisoes-adr/ADR-0004-alacritty-terminal-emulator.md) | Adoção do `alacritty_terminal` como motor de emulação VT |
| [adr/ADR-0005-tres-arvores-de-documentacao.md](decisoes-adr/ADR-0005-tres-arvores-de-documentacao.md) | As três árvores (`DocsPublic/`, `DocsPrivate/`, `DocsPrivate/legado/`) e por que contrariam a remoção da `DocsPublic/archive/` |
| [integracoes/registro-de-componentes-abertos.json](integracoes/registro-de-componentes-abertos.json) | Registro auditável de componentes open-source adotados |

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
| [../MANUAL.md](manual.md) | **Manual do usuário** — operação, funções e atalhos dentro da IDE |
| [../Tutorial.md](tutorial.md) | Distribuição, checksum, instalação, atualização e geração do AppImage |
| [build/como-executar.md](build/como-executar.md) | Como executar a IDE pelo checkout (ícone/launcher) |

Documentos de continuidade operacional — `../GUIAIA.md` (o mapa para quem
trabalha no código), `../AGENTS.md` (as regras para agentes) e a árvore
`../DocsPrivate/` (não versionada: log, diário, prompts, histórico, legado) —
são material interno de desenvolvimento e não integram a documentação pública.

## DocsPrivate/ — continuidade interna (consulta sob demanda)

Em 2026-09-15, os registros presentes neste checkout estão em
`DocsPrivate/Codex/`, lidos a pedido do autor. Os caminhos históricos
listados abaixo e em `legado/` não estão disponíveis nesta cópia: ficam
como referências nominais, sem links nem instrução para recriá-los.
A fila vigente é o roadmap 40 §4.1.

| Documento | Assunto |
| --- | --- |
| `../DocsPrivate/ContextoIA.md` | **LOG datado.** Responde "por que isto é assim?"; nunca "o que existe hoje?" |
| `../DocsPrivate/diario/18-daily-driver-plan.md` | Diário das fatias: marcos de dogfooding, decisões por sessão e escada de rigor. Registro de processo, não contrato |
| `../DocsPrivate/diario/19-registro-de-saidas.md` | **Registro de saídas do dogfooding** (2026-09-03): cada saída da Kinein para outra ferramenta, com reprodução mínima. É o que ordena a frente C do `roadmaps/34` por dor real — entrada sem reprodução não conta |
| `../DocsPrivate/prompts/` | Prompts de bootstrap para retomada em terminal. O atual é o `RETOMADA_2026-09-13-noite.md` (a passada de sincronização de 2026-09-13); os anteriores ficam como registro |

## DocsPrivate/legado/ — superado ou cancelado (não implementar a partir daqui)

| Documento | Por que saiu de `DocsPublic/` |
| --- | --- |
| `../DocsPrivate/legado/KINEIN_VECTIS_ASSISTANT_AI_ASSISTANCE.md` | ⛔ Linha de IA na IDE **cancelada** pelo autor em 2026-07-17 |
| `../DocsPrivate/legado/KINEIN_VECTIS_AI_CLI_BRIDGE_EXTERNAL_TERMINAL.md` | ⛔ Idem — o `aiBridge` foi removido do código no protocolo 0.59.0 |
| `../DocsPrivate/legado/17-architecture-hygiene-plan.md` | Fase concluída em 2026-07-06. Os números envelheceram 3,4x e enganaram uma sessão; os guardrails vivos estão em `arquitetura/ARCHITECTURE.md` §4 |
| `../DocsPrivate/legado/PLANO_ORGANIZACAO_E_HANDOFF.md` | Descreve o estado **anterior** à reorganização de 2026-07-16, executada. O que continua valendo (faixas P/T/X) foi extraído para cá — ver abaixo |

### Extraído do handoff antes de legar: as faixas P/T/X

O eixo de **audiência** citado na tabela de volatilidade continua valendo e não
depende mais daquele documento:

```text
P  PUBLICO   entregue a terceiros. Hoje: README.md, DocsPublic/manual.md, DocsPublic/tutorial.md.
T  TECNICO   quem compila/altera o projeto. Hoje: DocsPublic/ inteira.
X  INTERNO   continuidade do autor e das sessoes. Hoje: DocsPrivate/,
             GUIAIA.md, DocsPrivate/historico/PONTO_ATUAL.md, AGENTS.md.
```

O eixo P/T/X responde *"quem pode ler?"*; as quatro classes de volatilidade
respondem *"o que acontece se envelhecer?"*. São ortogonais: um documento
TÉCNICO pode ser CONTRATO (`ARCHITECTURE.md`) ou PLANO (`DocsPublic/especificacoes/`).

## Sem pasta de arquivo morto "só para guardar"

`DocsPublic/archive/` foi removido deliberadamente em 2026-07-05: era material
histórico que nenhum documento ativo referenciava mais como fonte. Essa regra
continua: **não existe pasta de depósito.** `DocsPrivate/legado/` não é depósito — é
uma lista curta, curada e justificada de documentos que uma sessão poderia
confundir com alvo. Se algo for descontinuado e ninguém puder se enganar com
ele, extraia o que tiver valor e **remova**; legar é para o que engana.

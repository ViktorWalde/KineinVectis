# 28 — Plataforma de plugins e as verticais (C/C++/Rust, Docker, banco, embarcados)

> **Classe: PLANO** (`docs/README.md`). Descreve o **alvo**; diverge da
> implementação por natureza. O que existe hoje se **mede no código**. Reconciliar
> a cada retomada — não citar este documento como estado.
>
> **Escrito em 2026-07-17** a pedido do autor. Não inventa processo: encaixa o que
> ele pediu no framework que já existe (`docs/integracoes/README.md` — modos A–D,
> níveis L0–L10, checklist de 10 passos, gate de auditoria).

## 0. Decisão do autor (2026-07-17): Docker e banco são NATIVOS

> "vamos integrar de forma nativa o docker e banco de dados, vão ser cidadãos
> 'nativos' ou 'de primeira classe'… vai ser demorado mas não tem problema"

Isso resolve uma ambiguidade que o resto do documento carregaria. **Docker e banco
não são "plugins de terceiro": são domínios nativos do core**, como `git`, `lsp` e
`terminal` já são. Coerente com a arquitetura que este projeto escolheu — sem
registro dinâmico, sem host de extensões, tudo compilado junto e auditável.

```text
integration v1 (L1)   O QUE UM "PLUGIN" E' NA KINEIN: descriptor + health +
                      config + escopo de permissao. E' a PLATAFORMA.
                      Docker e banco se REGISTRAM nela (health, config,
                      ligar/desligar) — e' assim que aparecem para o usuario.

dominio nativo        A FUNCIONALIDADE. `kinein-core/src/container/` e
                      `kinein-core/src/db/`, com handler fino, tipos no
                      kinein-protocol e Jobs cancelaveis. Rust nosso.
```

Os dois lados são necessários: sem o L1 a IDE não sabe **dizer** que Docker existe,
está instalado e está ligado; sem o domínio nativo ela não sabe **fazer** nada com
ele. Por isso o L1 continua sendo o gargalo mesmo com a decisão de nativo.

**Implementar do zero: sim, e é o caminho certo aqui.** Não existe biblioteca a
"orquestrar" que entregue isto pronto sem trazer runtime alheio. O que se estuda
(MODE-D) são invariantes, decisões e modos de falha — não código.

---

## 1. A regra que ordena tudo, e ela já existe

> **Capacidade antes de ferramenta.** Um nível só abre quando o contrato e o
> serviço do anterior estão comprovados. A facilidade de chamar uma CLI **não**
> antecipa a integração dela. (`docs/integracoes/README.md`)

Isso decide a fila inteira abaixo, e decide contra a vontade: Docker é fácil de
chamar (`docker ps` é uma linha) e mesmo assim **não vem antes** do L1. Um `docker
ps` numa aba não é integração — é uma CLI com fantasia de IDE.

**O gargalo é um só, e tudo depende dele:**

```text
L1 — dominio `integration` v1     NAO EXISTE (medido em 2026-07-17)
     handlers/ do core: build, cargo, cmake, debug, draft, format, fs, git,
     jobs, lsp, runconfig, run, settings, syntax, terminal, workspace.
     Nao ha `integration.rs`.
```

Enquanto o L1 não existir, **plugin nenhum tem onde nascer**: cada ferramenta
nova vira mais um handler ad-hoc, e o projeto reconstrói a mesma fiação (detectar,
health, configurar, ligar/desligar, reportar) uma vez por ferramenta. É
exatamente a dívida que a `ARCHITECTURE.md` §7 chama de "a refatoração massiva que
este documento existe para prevenir".

## 2. L1 — a plataforma de plugins (o que precisa existir para suportar plugins)

Entrega **arquitetural**, não visual. A aba não desbloqueia nível; o contrato e o
serviço desbloqueiam.

```text
kinein-protocol/src/integration.rs
  IntegrationDescriptor  id, nome, capacidades, versao-min, escopo de permissao
  IntegrationHealth      instalado? versao? caminho? por que nao?
  IntegrationConfig      valores + escopo (global/workspace) + reversivel
  IntegrationEvent       health mudou, config mudou, job de integracao

kinein-core/src/handlers/integration.rs    fino: parse + delega
kinein-core/src/integration/              o dominio, ja em pasta (§4 regra 4)
  ├── registry.rs    le o registro existente; NAO e' registro dinamico novo
  ├── health.rs      health por descriptor, reusando o `tools.rs` que ja existe
  └── config.rs      configuracao por escopo, reversivel
```

**Três invariantes que decidem se o L1 saiu certo:**

1. **Reusa o `tools.rs`, não o duplica.** A detecção já existe e já é agnóstica de
   distro (`ToolSpec`, `install_command: Option<&str>`, sem ramo por programa).
   Se o L1 criar um segundo detector, saiu errado.
2. **Sem registro dinâmico / host de extensões.** Decisão registrada do projeto
   (`ARCHITECTURE.md` §2.1): não se importa a máquina do Code OSS/IntelliJ. O
   "plugin" da Kinein é um **descriptor tipado compilado junto**, não código de
   terceiro carregado em runtime. Isso é limitação escolhida, e é ela que mantém
   o offline-first, o strict mode e a auditabilidade.
3. **A UI é burra:** lista, configura, pede ação. Nunca inicia processo.

**Gate de promoção (do `integracoes/README.md`):** ao menos uma integração
vertical real, testes de falha/cancelamento, orçamento medido, configuração
reversível e nenhum processo/handle órfão.

**Primeira vertical, recomendada (§0.2e do `PONTO_ATUAL`):** o **inventário das
ferramentas já detectadas** (clangd, rust-analyzer, CMake, Cargo, Git, rg, fd,
lldb-dap, Clippy). Zero dependência nova, valida o contrato inteiro e a aba
informativa. **Não** começar pelo EditorConfig: a auditoria de 2026-07-16 derrubou
a premissa (não existe biblioteca EditorConfig Rust madura; ver §0.2e).

## 3. A fila, em ordem de dependência

```text
L1  plataforma (`integration` v1)      FECHADO em 2026-07-19 (protocolo 0.62.0)
      └─ vertical 1: inventario das ferramentas ja detectadas

L2  C/C++/RUST SOLIDOS — resultado comum        EM ANDAMENTO
      diagnostico/teste/cobertura num contrato so + artefatos + Jobs cancelaveis
      valida com: Cppcheck, Clang Static Analyzer, cargo-audit/deny, Valgrind,
                  GTest/Unity/Criterion, gcov/lcov, cobertura Rust
      FEITO   Cppcheck no funil de qualidade          (fatia 1, 2026-07-19)
      FEITO   GTest/Unity nomeados no painel          (fatia 2, 2026-07-19)
      FEITO   cobertura: cargo llvm-cov + lcov        (fatia 3, 2026-08-21)
      FEITO   Clang Static Analyzer (clang-tidy)      (fatia 4, 2026-08-21)
      FALTA   cargo-audit/deny, Valgrind
      FORA    Criterion — formato por-caso nao confirmado na fonte

L3  Project Graph, targets e perfis explicaveis; cache provenance
      valida com: Bear, ccache/sccache, Bloaty, Doxygen

L4  DAP solido + profiling
      valida com: lldb/gdb DAP (ja parcial), Heaptrack, perf/Hotspot

L5  RemoteContext  <- E' AQUI QUE O DOCKER ENTRA. ver §4.
      host keys, credenciais externas, path mapping, desconexao, Jobs remotos

L5.5 BANCO DE DADOS  <- vertical propria. ver §5. Precisa de L1 + L2 + L5.
      (credenciais e path/porta vem do L5; query cancelavel vem do L2)

L6  EMBARCADOS: Target/Device/Probe, deteccao USB, package trust, flash preview
      valida com: udev, CMSIS-DAP/Pack, OpenOCD, probe-rs, esptool, QEMU

L7+ streaming/trace, armazenamento+visualizacao, sandbox, laboratorio
```

**Por que C/C++/Rust vem antes de Docker e banco, e não é gosto:** L2–L4 é o que
transforma a Kinein numa IDE que substitui o que o autor usa hoje. Docker e banco
são superfície nova; L2–L4 é profundidade no que já existe. Uma IDE com Docker e
sem cobertura de teste é uma demo.

## 4. Docker — pertence ao L5 (RemoteContext), não a um nível próprio

**A decisão arquitetural:** um container **é um contexto remoto**. Ele tem
filesystem próprio, path mapping, ciclo de vida, execução fora do host e falha por
desconexão. É o mesmo contrato do SSH — e é assim que Code OSS (Dev Containers) e
JetBrains modelam. Dar a Docker um nível próprio duplicaria `RemoteContext` inteiro.

```text
RemoteContext (L5)
  ├── ssh://          host keys, credenciais, path mapping
  └── container://    imagem/compose, path mapping, ciclo de vida
```

**Fatos verificados em 2026-07-17 (não deduzidos):**

- o plugin Docker do IntelliJ **não vem embutido no Community** — é instalável do
  marketplace. Ou seja: dá para estudar o comportamento, mas ele **não é** parte
  do Community "de fábrica";
- a especificação **devcontainer.json** é aberta e é o padrão de fato. Adotá-la
  como formato de entrada é MODE-D (referência) sem transplantar runtime.

**Invariantes:** a UI nunca chama `docker` direto (§2 — proibição não negociável);
todo comando de container é Job cancelável; permissões (socket do Docker,
privilégios) são visíveis e confirmáveis; **não** rodar a própria IDE dentro do
container. Podman é alternativa a auditar (rootless muda a superfície de permissão).

## 5. Banco de dados — a premissa mudou, e isso é bom saber antes

**MEDIDO em 2026-07-17, e derruba a expectativa:**

```text
IntelliJ IDEA Community  ->  Database Tools and SQL: NAO TEM. E' Ultimate.
                             DataGrip e' produto pago separado.
```

Fonte: a própria documentação da JetBrains (links no fim). **Portanto o IntelliJ
Community não serve de referência para banco de dados — a funcionalidade não está
lá para ser estudada.** Isso não cancela o objetivo; troca a fonte.

**Candidatos a referência (MODE-D), todos sujeitos ao gate de auditoria do
`integracoes/README.md` — licença SPDX, manutenção, releases, telemetria, rede em
runtime, dependências transitivas):**

```text
DBeaver              a referencia funcional obvia de cliente de banco.
                     AUDITAR: licenca, e' Java/Eclipse — peso e fronteira.
Database Navigator   plugin de terceiro do marketplace IntelliJ; existe
                     justamente porque o Community nao tem DB tools. AUDITAR.
SQLTools (Code OSS)  MIT; arquitetura de driver por adaptador. AUDITAR.
```

**O que a Kinein precisa antes de qualquer driver, e é o ponto:**

```text
1. conexao         credencial NUNCA em texto plano no projeto; vem do L5.
2. schema browser  arvore, e' leitura — cabe na aba informativa do L1.
3. query console   editor + execucao. A query e' JOB CANCELAVEL (L2), senao
                   um SELECT ruim trava a IDE.
4. result grid     o componente visual mais caro desta lista. Paginacao,
                   tipos, edicao. Nao existe hoje e nao se improvisa.
```

**Nativo, e isso decide o driver.** Sendo domínio do core, o cliente de banco é
Rust nosso falando com o servidor — não um processo de terceiro embrulhado. Isso
torna a escolha do driver uma decisão de dependência séria (`sqlx`/`tokio-postgres`
/`rusqlite` a auditar), e mantém o precedente do ADR-0004: **Rust puro antes de
FFI**.

**Ordem honesta:** 1 → 2 antes de 3 → 4. Um browser de schema read-only já entrega
valor real e valida o contrato; o result grid editável é uma fatia grande e não
deve ser o começo. **Driver:** começar por **um** (PostgreSQL ou SQLite), não por
uma matriz. Rust puro é preferível a FFI (precedente: ADR-0004 escolheu
`alacritty_terminal` por ser "Rust puro… sem Node, Electron, WebView ou FFI").

## 6. Embarcados — L6, e o que já está escrito

Já especificado (tabela L0–L10 e `docs/specs/KINEIN_VECTIS_EMBEDDED_TARGETS_FLASH_SERIAL_QEMU.md`).
Depende de L1 (descriptor/health), L4 (DAP) e L5 (contexto/permissão). O que o L6
acrescenta: `Target`/`Device`/`Probe`, detecção USB (udev), **package trust**,
flash **preview** e confirmação explícita.

**Não antecipar por facilidade:** `openocd`/`probe-rs`/`esptool` são triviais de
chamar, e é exatamente por isso que a regra "capacidade antes de ferramenta"
existe. Flash é operação `dangerous` (§7): confirmação muito explícita, nunca
automática, nunca vinda de IA.

## 7. UI/UX — dívida explícita, contínua e ortogonal aos níveis

> **Registro do autor, 2026-07-17: a UI/UX da IDE tem MUITO a ser
> otimizado/polido.** Isso não é um nível da fila: é trabalho contínuo que
> atravessa todos eles.

**Referência: IntelliJ IDEA Community — visual e de fluxo.** Decisão do autor:
*"essa UI/UX da JetBrains é muito agradável/confortável para o desenvolvimento"*.

**Mas a referência não cobre tudo, e é importante saber onde ela acaba:**

```text
IDIOMA GERAL DA IDE      IntelliJ Community SERVE e e' a referencia:
  tool windows, densidade, navegacao, arvore, abas, popups, settings,
  hierarquia visual, atalhos, "onde as coisas ficam".

A UI DE BANCO DE DADOS   O Community NAO TEM (verificado; e' Ultimate).
  Nao ha o que estudar la. Para o cliente de banco a referencia funcional e'
  o DBeaver (a auditar); o IDIOMA visual continua sendo o do IntelliJ
  Community, aplicado a um dominio que ele nao possui.

A UI DE DOCKER           O plugin existe para o Community (instalavel), entao
  o comportamento e' estudavel — mas ele nao vem "de fabrica".
```

E a regra que governa, que já é contrato do projeto (`ARCHITECTURE.md` §2.1 e o
`OPEN_PLUGIN_ADAPTATION_ROADMAP` §2):

```text
NAO E' "copiar e colar".  A adaptacao e' obrigatoria e e' REALISTA/PRAGMATICA:
  - importa-se INVARIANTE, DECISAO, MODO DE FALHA e ESTRATEGIA DE TESTE;
  - NAO se importa codigo, runtime, IntelliJ Platform/Swing nem modelo interno;
  - o que se ve no IntelliJ precisa ser REDESENHADO no fluxo nativo Qt/QML da
    Kinein, com o Theme, a iconografia e a arquitetura desta IDE;
  - quando o contexto da Kinein diverge (IDE de C/C++/Rust, offline-first, sem
    host de extensoes, frameless com chrome proprio), **vence o contexto da
    Kinein** — nao a fidelidade ao IntelliJ.
```

**Por que o IntelliJ Community e não outra:** é Apache-2.0, o código está
disponível para estudo, e a densidade/organização dele é o alvo estético que o
autor escolheu (registro de 2026-07-16: "o visual limpo da IDE aberta do
JetBrains — inspiração, não cópia; referência de print em `docsprivate/imagens/prints`").

**Onde a dívida de UI/UX já é conhecida:** `docs/roadmaps/20-ui-spec-convergence-plan.md`
(fatias C0–C6) e `docs/specs/KINEIN_VECTIS_UI_COMPONENTS_SYSTEM.md`. Este
documento não os substitui — aponta para eles e registra que a barra subiu.

**A armadilha, medida em 2026-07-17 e que vale para toda fatia de UI:** o débito
de arquivo bloqueia. `EditorController.qml` está em 1070 (limite 400) e
`GitPanel.qml` em 764 (limite 300) — **qualquer polimento nessas áreas paga o
débito antes**. A catraca não é opcional e já cobrou 4x num único dia.

## 8. O que este documento NÃO faz

- **não abre nível nenhum:** a fila real e o que está em aberto vivem na TRILHA
  do `docsprivate/PONTO_ATUAL.md`, medida contra o código;
- **não adota nada:** toda ferramenta citada aqui é candidata e passa pelo gate de
  auditoria + `OPEN_COMPONENT_REGISTRY.json` antes de entrar;
- **não decide licença:** DBeaver, Database Navigator e SQLTools estão listados
  como candidatos a estudo, **não** como escolhidos.

## Fontes consultadas (2026-07-17)

- [Database Tools and SQL — IntelliJ IDEA Documentation](https://www.jetbrains.com/help/idea/relational-databases.html)
  — confirma que a funcionalidade exige assinatura Ultimate.
- [Database tool window — IntelliJ IDEA Documentation](https://www.jetbrains.com/help/idea/database-tool-window.html)
- [Docker — IntelliJ IDEA Documentation](https://www.jetbrains.com/help/idea/docker.html)
  — confirma que o plugin vem por padrão só no Ultimate; no Community, instala-se.
- [Database Navigator — JetBrains Marketplace](https://plugins.jetbrains.com/plugin/1800-database-navigator)

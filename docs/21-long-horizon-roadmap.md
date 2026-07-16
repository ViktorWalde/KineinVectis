# 21 — Roadmap de longo horizonte (M4–M7) e playbook de continuidade

> **Status:** ativo (escrito em 2026-07-09 a pedido do usuário)
> **Para quem:** qualquer pessoa ou IA que continue este projeto. Este
> documento existe para que a continuidade NÃO dependa de prompts
> profissionais: o que fazer, em que ordem, com quais decisões já
> tomadas e quais réguas, está tudo aqui e em `docs/18`.
> **Regra de ouro:** em conflito entre este doc e o código/`ContextoIA.md`,
> vale o que está implementado + ContextoIA; atualize este doc em vez de
> obedecê-lo cegamente.

---

## Como continuar este projeto (playbook — leia antes de qualquer código)

O ritual por fatia é FIXO e não-negociável (pedido explícito do autor):

```text
1. LER: ContextoIA.md (estado atual + próxima fatia) → docs/18 (ordem e
   designs) → este doc (se a fatia for de M4+).
2. ESTRUTURAR: escrever o design da fatia em docs/18 ANTES do código:
   decisões COM porquês, contrato IPC (se houver), lista de arquivos,
   testes planejados, o que fica FORA (com gatilho para entrar).
3. IMPLEMENTAR: protocolo (kinein-protocol, bump de versão) → domínio no
   core (crates/kinein-core/src/<dominio>.rs, funções puras testáveis) →
   handler fino (handlers/<dominio>.rs) → registro (lib.rs/commands.rs)
   → testes de domínio e dispatch → UI (controller QML não-visual +
   roteador ipc/ + componente visual burro + fios em Main.qml +
   ui/CMakeLists.txt).
4. VALIDAR: cargo fmt --all → scripts/verificar.sh (gate completo, para
   no primeiro erro) → smoke offscreen (QT_QPA_PLATFORM=offscreen
   timeout 8 ./scripts/kinein-vectis; exit 124 = vivo = ok) → SONDA e2e
   com a ferramenta real via stdio (padrão: scripts python que sobem
   target/debug/kinein-core e falam JSON-RPC linha a linha; ver sondas
   de LSP/DAP/git citadas em docs/18).
5. SINCRONIZAR: docs/03 (contrato), docs/18 (checklist [feito] +
   descobertas incorporadas), MANUAL.md (linguagem de usuário final),
   ContextoIA.md (fatia feita + próxima).
6. NUNCA COMMITAR: o autor faz um único commit ele mesmo. A árvore
   acumula tudo.
```

Convenções que pegam quem chega agora (aprendidas em M0–M3):

```text
- Rigor é gate, não sugestão: clippy pedantic/nursery -D warnings,
  clang-tidy Werror, qmllint estrito zero warnings. Conflitos comuns:
  unreachable_pub × redundant_pub_crate → módulo `pub` com itens
  `pub(crate)` (padrão rpc.rs); const fn com tipo Drop movido é ok,
  dropado não.
- Nunca parsear saída LOCALIZADA de ferramenta externa: exit codes e
  formatos estáveis (--porcelain=v2 -z, JSON-RPC, DAP, file-api).
- Respostas ECOAM a chave pedida (path/frameId/ref) para a UI
  correlacionar e descartar resposta velha (padrão format.text).
- Mutações respondem o estado COMPLETO novo (padrão runConfig/git): a
  UI nunca calcula estado derivado.
- Popups/dropdowns NUNCA são filhos do header/painéis: sempre overlay
  em ShellOverlays (z 90–102), com dismiss por clique-fora.
- Estado em controller QML não-visual; componente visual é burro
  (property in, signal out); rebind de função em delegate usa o padrão
  "revision counter" (breakpointsRevision/gitRevision).
- Todo atalho com F-key tem alternativa sem F-key e acionamento manual
  por botão/comando (regra do usuário; notebooks com Fn).
- cargo test NÃO recompila o binário target/debug/kinein-core — rodar
  `cargo build -p kinein-core` antes de sondas e2e.
- Pequenas coisas acumuladas = fluidez (princípio do usuário): cada
  fatia olha a ergonomia do caminho que toca e registra no radar de
  docs/18 o que não couber nela.
```

Estado ao escrever este doc: M0–M2 prontos e validados pelo usuário;
M3.1–M3.3 prontos (status/diff/gutter/stage/commit), M3.4 (blame) e
trilha E2/E3 pendentes; radar tem "auto-setup ao abrir projeto pronto".
Protocolo IPC 0.32.0. Testes: 187 core + 47 protocol.

---

## Norte de produto — “simbiose com os compiladores”

> **Meta registrada pelo usuário em 2026-07-14:** fazer a Kinein Vectis
> começar como uma IDE profissional “um nível abaixo” do CLion e evoluir, por
> profundidade de integração, até um fluxo equiparável. Isso não significa
> escrever compilador, build system, language server ou debugger próprio.

“Simbiose com os compiladores” é o nome de produto para uma integração em que,
ao abrir um projeto, a IDE constrói e mantém um contexto incremental profundo:
targets, arquivos, flags, includes, defines, dependências, toolchains, perfis,
artefatos, testes, diagnósticos e capacidades de execução/debug. O usuário deve
ter a sensação de que a IDE compreende o projeto inteiro, mas a implementação
deve consultar fontes maduras e estruturadas, sem reler tudo monoliticamente e
sem duplicar a autoridade das ferramentas:

```text
CMake File API / compile_commands.json / Cargo Metadata
            ↓
Unified Project Graph + Context Matrix (KSWE)
            ↓
clangd / rust-analyzer / compilador real / clang-tidy / Clippy
            ↓
Build / Test / Run / DAP / GDB / LLDB / targets embarcados
            ↓
UI coerente, incremental, explicável e orientada ao target ativo
```

Para atingir primeiro o patamar “um nível abaixo do CLion”, fortalecer, nesta
ordem arquitetural e por fatias medidas:

1. Project Graph, Context Matrix e CMake File API completa;
2. modelos de targets e toolchains como entidades de primeira classe;
3. debugger, watches, inspeção de variáveis e qualidade dos pretty-printers;
4. flash, serial, QEMU, OpenOCD/pyOCD e debug remoto;
5. confortos de editor: split view, multi-cursor e EditorConfig;
6. testes prolongados em projetos C/C++/Rust e embarcados reais.

O CLion também orquestra compiladores, CMake, Ninja, GDB/LLDB e analisadores;
logo, a Kinein pode alcançar profundidade profissional sem possuir compilador
próprio. A diferença será a qualidade do modelo de projeto, do agendamento, da
correlação de contexto e das interações. O desenho detalhado é o
`KINEIN_VECTIS_DEEP_SEMANTIC_ENGINE_CPP_RUST_WORKFLOW.md`; implementar sempre
por fatias, reaproveitando CMake/Cargo/LSP/DAP/Jobs existentes.

---

## M4 — Confiança e configuração ("estável, rápido e meu o dia inteiro")

**Pergunta de pronto:** *"A IDE aguenta um dia inteiro de trabalho real
sem susto, e se adapta a mim sem eu editar código dela?"* Regressão em
M0–M3 bloqueia o M4 (regra de docs/18).

### M4.1 — Settings/Storage com schema (a fatia que destrava as outras)

> **[FEITA 2026-07-11, protocolo 0.36.0]** Infra completa (global XDG +
> workspace, settings.get/set, UI SettingsDialog + Ctrl+Alt+S) com 3
> consumidores reais: editorFontSize → Theme.fontSizeEditor, autoClosePairs
> → auto-close da E1, formatOnSave → Ctrl+S formata-então-salva. diffBase
> (head|index) adiado (precisa de `base` no git.fileDiff). Validada e2e
> (persistência global/workspace entre processos). Design/descobertas:
> docs/18, "Fatia M4.1".

```text
Decisões já tomadas (honrar ou registrar por que mudou):
- Dois níveis: global (~/.config/kinein-vectis/settings.json, XDG) e
  por-workspace (.kinein/settings.json). Workspace SOBREPÕE global,
  campo a campo. Ambos com schemaVersion + migração silenciosa (mesmo
  padrão de session.json/runconfigs.json: arquivo inválido → default,
  nunca quebra).
- Contrato: settings.get {} → { settings (efetivo), global, workspace };
  settings.set { scope: global|workspace, values: {...} } → estado novo
  completo (padrão de mutação do repo).
- Primeiros consumidores REAIS (não inventar setting sem consumidor):
  1. format-on-save (bool; gatilho registrado desde a M1.1);
  2. tamanho da fonte do editor (int; consome Theme.fontSizeEditor);
  3. auto-close pairs on/off (trilha E1);
  4. base do diff do gutter (head|index; gatilho da M3.2).
- UI: página Settings simples (overlay ou aba IDE) gerada da lista de
  settings conhecidos — SEM engine genérica de formulário; cada setting
  novo adiciona seu controle. First Run/onboarding das specs NÃO entra
  aqui (é M4.4).
Fora: atalhos customizáveis (M5), settings de plugins (M6).
```

### M4.2 — Orçamento de performance medido (sem telemetria, 100% local)

> **[FEITA 2026-07-11]** `scripts/medir-performance.sh` (+ `medir-core.py`)
> mede offscreen + stdio + `/proc`, zero rede. Design/decisões: docs/18
> "Fatia M4.2". Tabela abaixo com números REAIS.

```text
- scripts/medir-performance.sh: (a) startup até primeiro frame (offscreen,
  marker KINEIN_PERF env-gated na main.cpp), (b) workspace.open no próprio
  repo, (c) fs.read de .txt de 10k linhas (proxy de abrir arquivo grande),
  (d) RSS: UI em boot vazio + core em regime + LSP Rust vivo. Latência de
  digitação fica MANUAL (item abaixo) — precisa de GUI com injeção de tecla.
- Regressão de orçamento = bug: entra como fatia de correção antes de
  feature nova. Otimizações conhecidas à espera de medição: janela de
  delegates da gutter já é windowed; suspeitos são o TextEdit com
  arquivos enormes e o ListView do terminal.
- Explícito: NUNCA telemetria/rede. Medição é manual e local.
```

**Orçamento medido (1ª medição — 2026-07-11).** Máquina de referência:
AMD Ryzen 7 7735HS (16 threads), ~22 GB RAM, Arch, Qt 6.11 (docs/14). O
ORÇAMENTO = mediana medida com folga; ultrapassar = investigar como bug.

| Métrica | Medido (mediana) | Orçamento (alerta se >) |
| --- | --- | --- |
| UI: time-to-first-frame (offscreen) | ~101 ms | 400 ms |
| UI: RSS em boot vazio | 88 MB | 200 MB |
| Core: `workspace.open` (repo Kinein) | 1.5 ms | 50 ms |
| Core: `fs.read` de 10k linhas | 0.1 ms | 20 ms |
| Core: RSS em regime (ws + arquivo) | 5 MB | 60 MB |
| LSP `rust-analyzer`: RSS (~8 s indexando o repo) | 677 MB | informativo¹ |

¹ É footprint do **rust-analyzer** (ferramenta externa) indexando o
workspace inteiro, não memória do Kinein — some ~700 MB no repo Kinein é
esperado; o número da IDE em si é o "Core RSS em regime" (5 MB). Medição
offscreen SUBESTIMA o startup real (sem compositor); o startup da janela
GUI de verdade é o item manual abaixo.

**Revalidação release após protocolo 0.57 (2026-07-15, N=3).** O gate
reconstruiu os binários exatos do launcher
(`build/linux-clang-release-hardened/ui/kinein-vectis` e
`target/release/kinein-core`) e `scripts/medir-performance.sh` foi executado
com esses caminhos explícitos. Resultado: primeiro frame 250 ms, UI 103 MB,
`workspace.open` 3,4 ms, `fs.read` 10k 0,0 ms e core 7 MB — todos dentro dos
orçamentos. O rust-analyzer usou 1093 MB e permanece informativo/externo.

**Continuação A3 preparada, ainda não implementada.** A mesma infraestrutura
será estendida, nesta ordem, para primeiro snapshot Tree-sitter, atualização
incremental, primeira resposta semântica/completion, digitação tecla→frame e
rajada PTY→frame. A fila executável e critérios estão em `PONTO_ATUAL.md`, A3.1
a A3.4; não criar um segundo runner. Referências profissionais consultadas:

- Code OSS `234638618394269563dd77c0c395c270d8df8b12`, MIT/MODE-B,
  `src/vs/base/common/performance.ts` e
  `src/vs/workbench/services/timer/browser/timerService.ts`: marcos nomeados,
  durações entre fases prontas e separação entre custo próprio/ambiente;
- Zed `1e22d1a83f8b1b7acc528d15cfab0644852380c0`, MODE-D,
  `crates/benchmarks/benches/editor_render.rs` e `display_map.rs`: seed fixa,
  tamanhos explícitos, amostras repetidas e caminho real de input/render.

Adaptação: `std::time::Instant`/`QElapsedTimer`, fixtures locais, stdio e
mediana/p95; nenhuma telemetria, runtime, função ou teste das referências.

**Métricas manuais (registrar quando observadas):**

| Métrica manual | Procedimento | Observado |
| --- | --- | --- |
| Startup até janela interativa (GUI real) | cronometrar do lançar até poder digitar | _(a preencher)_ |
| Latência de digitação (arquivo 10k linhas) | abrir arquivo grande, segurar uma tecla, ver se há atraso perceptível (>~50 ms incomoda) | _(a preencher)_ |

### M4.3 — Robustez de processos (a IDE não morre junto de ninguém)

> **[Parte A FEITA 2026-07-11]** Recuperação de crash do core: o
> CoreClient detecta a saída inesperada, relança o core, reabre o mesmo
> workspace (UI preserva as abas — mesmo root não limpa) SEM restaurar a
> sessão (não sobrescreve edições), com anti-loop de fork e re-sync do LSP
> do arquivo ativo. Validado com kill -9 real (reconecta com tabs
> preservadas). FALTA (parte B/C, fatia M4.3b): lsp.restart + auto-restart
> após N timeouts; cancelar jobs órfãos no shutdown. Design: docs/18
> "Fatia M4.3".

```text
- Core morreu (crash/kill): CoreClient detecta (QProcess finished),
  mostra estado na status bar, RELANÇA o core e replay: workspace.open
  do root atual + reaberto o que a sessão tinha. Guardar o último root
  na UI basta (a sessão persiste em .kinein/session.json).
- LSP travado: timeout já existe por request; adicionar
  "lsp.restart { language }" + ação na aba IDE/Ferramentas quando um
  server passa de N timeouts seguidos (contador no manager).
- Jobs órfãos: job.cancel em tudo que estiver vivo no shutdown limpo.
- Critério de aceite: kill -9 no kinein-core com a IDE aberta → em até
  ~2s a IDE volta a responder com o mesmo workspace, sem reiniciar a
  janela; matar clangd → diagnostics voltam sozinhos após restart.
```

### M4.4 — First Run / Start Screen (specs ONBOARDING_*)

```text
- Só entra quando houver >1 usuário real (amigos testando contam;
  gatilho: primeiro feedback de instalação confusa).
- Escopo: Start Screen (projetos recentes — persistir lista global no
  storage da M4.1 — + criar/abrir), primeiro-uso com environment.scan
  guiado (a infra já existe: tools.detect/environment.scan/Project
  Health). A UI segue as specs de docs/specs/ONBOARDING_*.
- Projetos recentes é a "pequena coisa" de maior valor: entregar antes
  do resto se houver janela (menu no botão "Abrir pasta...").
```

### M4.5 — Perfis de rigor Strict/Balanced/Relaxed

```text
- Setting (M4.1) que regula APENAS o que a IDE roda para o usuário
  (clippy pedantic vs default no quality.run, -Werror on/off no build
  dele), NUNCA o gate do próprio repositório Kinein (imutável).
- Default: Strict (identidade do produto). Documentar no MANUAL o que
  cada perfil liga.
```

---

## M5 — Paridade diária JetBrains ("não sinto falta do CLion")

**Pergunta de pronto:** *"Uma semana inteira no Kinein sem abrir outra
IDE nem sentir falta concreta."* Régua: JetBrains para
navegação/refactor; Neovim para velocidade/composição.

### M5.1 — Refactorings por cima do LSP

```text
- Já existe: rename multi-arquivo, code actions (Alt+Enter).
- Entram: organize imports (rust-analyzer: source.organizeImports via
  codeAction kind; clangd: include-cleaner actions), extract
  function/variable (rust-analyzer expõe como code action com range —
  exigirá mandar SELEÇÃO no lsp.codeActions, hoje é posição), inline.
- Decisão de arquitetura: NADA de refactor próprio; se o server não
  expõe, o Kinein não tem (registrar a lacuna no MANUAL). Custo real:
  suporte a codeAction com range + applyEdit multi-arquivo mais robusto
  (já existe base da M1.3).
```

### M5.2 — Git avançado (expande o MVP de M3)

```text
- push/pull/fetch como JOBS canceláveis (rede = job, nunca síncrono;
  reusar JobManager) com progresso na status bar; erros de auth viram
  mensagem acionável (a IDE NÃO gerencia credenciais; usa o credential
  helper do git do sistema).
- branches: listar/criar/trocar (git branch/switch, exit codes) num
  seletor na status bar (clicar na ⎇); merge SEM UI de conflito própria
  no início — conflitos aparecem como "conflicted" no status (M3.1 já
  colore) + instrução; UI de merge é fatia própria futura.
- log básico: git log --format estável -z, painel com lista (hash curto,
  autor, data relativa, mensagem); clique mostra o diff do commit
  (git show, reusa o GitDiffDialog).
- stash: stash/pop com lista simples.
- blame (se M3.4 não tiver entregue): git blame --porcelain por arquivo,
  coluna opcional na gutter (autor+idade compactos), toggle por comando.
```

### M5.3 — Navegação pesada de código

```text
- call hierarchy (LSP callHierarchy/incomingCalls) e type hierarchy
  quando os servers expuserem — painel lateral reutilizando o padrão do
  usages popup, mas em árvore.
- "Recent files" (Ctrl+E, JetBrains): lista das últimas abas — barato,
  alto valor; pode entrar antes como "pequena coisa".
- Bookmarks simples (linha marcada, lista no painel) — avaliar custo.
```

### M5.4 — Decisão de engine do editor (a MAIOR decisão técnica pendente)

> **MANDATO (usuário, 2026-07-11):** adotar **tree-sitter** e as demais
> TECNOLOGIAS que os plugins do Neovim usam, direto na IDE, SEM embutir o
> Neovim (a tensão modal × identidade JetBrains foi descartada). tree-sitter
> é OBRIGATÓRIO (realce/indentação/textobjects); LSP e DAP já são plugados
> direto pelo core. Esta decisão de engine deve contemplar onde o
> tree-sitter entra (highlighter C++ atual → parser incremental). Ver
> ContextoIA, "Direção do produto".

```text
Contexto honesto: o TextEdit do QtQuick nos levou até aqui (custo
baixíssimo), mas bloqueia: split view, minimap, multi-cursor real,
virtualização de arquivos gigantes, inlay hints posicionados.
- ANTES de escrever qualquer engine: medir (M4.2) e listar o que o
  QQuickTextDocument + QSyntaxHighlighter atuais aguentam.
- Caminhos, em ordem de preferência da filosofia do projeto:
  a) continuar no TextEdit e aceitar os limites (documentando-os);
  b) TextArea/QQuickTextEdit com camadas custom (gutter/overlays já são
     nossos — só o miolo é Qt);
  c) engine própria de rendering de texto (ÚLTIMO recurso; é
     "reimplementar", só com justificativa medida e registrada).
- Split view e minimap SÓ depois dessa decisão. Multi-cursor idem.
- Esta fatia é de DESIGN + medição; o resultado é um doc de decisão
  (docs/2x) com a escolha e o plano, não código.
```

---

## M6 — Extensibilidade orquestrada ("cresce sem reescrever o core")

**Pergunta de pronto:** *"Adiciono uma linguagem/ferramenta nova SEM
recompilar a IDE?"* Filosofia mantida: extensão = ORQUESTRAR mais
ferramentas, não rodar código de terceiros dentro da IDE. Explícito: NÃO
haverá sistema de plugins com código arbitrário no M6 (risco/custo
desproporcionais para uso pessoal; reavaliar só com comunidade real, e
aí como M7+).

### M6.1 — Language servers configuráveis

```text
- Hoje: clangd/rust-analyzer hardcoded em lsp/server.rs (SERVERS).
- Entra: settings (M4.1) com lista de servers custom:
  { language, command, args, extensions[], languageId } — mesmo shape do
  ServerSpec. pyright/gopls/lua-language-server passam a funcionar sem
  tocar código. Validação: server desconhecido some com mensagem clara
  se o binário faltar (tools.detect dinâmico p/ binários custom).
- Semantic tokens/kinds já são genéricos (legend do server); testar com
  1 server não-nativo (pyright) e registrar limitações reais.
```

### M6.2 — Debug adapters configuráveis

```text
- Hoje: lldb-dap hardcoded (ADAPTER_BINARY).
- Entra: settings com adapters { kind, command, launchTemplate } e
  runConfig ganhando "debugAdapter" opcional; debugpy (python) é o caso
  de aceite. A dança DAP já é genérica (M2.5); o custo é só spawn +
  launch args por adapter.
```

### M6.3 — Task runner (tarefas do usuário)

```text
- tasks.json-like no .kinein: { name, command, problemMatcher? } →
  aparecem no Search Everywhere e rodam como jobs canceláveis.
- problemMatcher v1: regex nomeada (file/line/col/message/severity) que
  alimenta a MESMA aba Problems (Diagnostic comum). Cobre linters
  arbitrários (shellcheck, eslint...) sem código novo por linter.
```

### M6.4 — Ponte de IA formalizada (specs ai-bridge / KV Context)

```text
- [FATIA INICIAL FEITA 2026-07-14, protocolo 0.50.0] KV Context detecta
  Claude/Codex instalados pelo usuário, inicia a CLI escolhida explicitamente
  no TerminalManager/PTY existente e permite sair/trocar; sem provider/API
  embutido e sem enviar contexto automaticamente.
- Próxima fatia: formalizar SÓ a ponte de
  contexto: comando "copiar contexto do workspace" (arquivo atual,
  seleção, diagnostics, git status) em formato colável + variável de
  ambiente/arquivo para CLIs de IA lerem. SEM chamadas de rede da IDE;
  quem fala com modelo é a CLI do usuário (offline-first preservado).
- Uma opção para outra CLI deve apenas orientar e abrir o terminal para comando
  manual; entra depois do aceite funcional da fatia inicial.
```

---

## M7 — Distribuição e comunidade ("outra pessoa instala sem mim")

**Pergunta de pronto:** *"Um desconhecido instala, usa e reporta bug sem
falar comigo antes?"* Só faz sentido com M4 (robustez) maduro.

### M7.1 — Empacotamento e release

```text
- [FUNDAÇÃO FEITA 2026-07-14] AppImage x86_64 reproduzível em builder Debian
  12, com UI + core + Qt/QML + metadados/licenças/checksum. Smoke aprovado no
  Arch e sem rede num Debian mínimo sem Qt/Rust/compiladores. Decisão e pins:
  ADR-0003. Baseline glibc 2.36; não prometer distros anteriores sem nova base.
- Próximos passos de release: matriz Ubuntu/Fedora, automação de artefatos,
  changelog, assinatura/proveniência e diagnóstico de runtime. PKGBUILD/AUR é
  alternativa para Arch e Flatpak só entra se houver demanda.
- Compiladores, LSPs, build systems e debugadores permanecem dependências
  externas por linguagem. O pacote deve declará-las como opcionais/
  recomendadas, nunca instalar tudo silenciosamente.
- Versionamento: a IDE ganha versão própria (0.x) desacoplada do
  protocolo IPC; changelog GERADO das seções [feito] de docs/18 (fonte
  única, sem duplicar histórico).
- AppImage cobre outras distribuições Linux, não Windows. Uma port Windows é
  trilha posterior própria: CI nativa, PowerShell/ConPTY, caminhos e `.exe`,
  storage sem XDG, descoberta de toolchains e instalador assinável. Só declarar
  suporte depois de gate e testes reais nessa plataforma.
```

### M7.2 — CI pública

```text
- GitHub Actions rodando EXATAMENTE scripts/verificar.sh (o gate local
  é a CI; nunca deixar a CI divergir do gate) em Arch (container) +
  Ubuntu LTS. Cache de cargo/cmake. Release action que empacota M7.1.
- As sondas e2e (LSP/DAP/git) entram na CI como job separado tolerante
  a ambiente (instala clangd/lldb/git no container).
```

### M7.3 — Documentação pública e contribuição

```text
- O repositório privado continua sendo a fonte completa. Publicar por um
  exportador allowlist para um espelho separado; nunca tornar o remoto privado
  atual público por engano.
- Qualquer cópia do código entregue a terceiros contém, entre arquivos
  Markdown, somente README.md, MANUAL.md e Tutorial.md. GUIAIA.md,
  ContextoIA.md, PONTO_ATUAL.md, AGENTS.md, `docs/`, `prompts/`, roadmaps e
  notas de agentes ficam privados. Licenças/atribuições usam LICENSE, JSON ou
  TXT. `.git/` e o histórico privado não entram; eventual espelho começa com
  histórico próprio. O repositório-fonte nunca muda de visibilidade para essa
  entrega.
- README apresenta o produto e o status; MANUAL.md cobre somente o uso da IDE
  e alimenta o visualizador interno; Tutorial.md cobre instalação,
  distribuição, atualização e geração do pacote.
- O exportador precisa de `--dry-run`, lista explícita do que entra, rejeição
  de Markdown extra e auditoria de segredos antes de qualquer push.
- i18n: extrair strings (qsTr já usado em toda a UI) e gerar inglês
  como segunda língua; português continua a língua de desenvolvimento.
```

### M7.4 — Diagnóstico de falhas sem telemetria

```text
- Crash do core/UI → dump LOCAL em ~/.local/state/kinein-vectis/
  (stderr do core já vai para errorLogFile hoje) + diálogo no restart
  oferecendo abrir o arquivo para o usuário COLAR num issue (ação
  manual dele; nada sai da máquina sozinho — princípio inegociável).
```

---

## Trilha T — Paridade de toolchain C/C++ e Rust (régua Neovim, adaptada)

Pedido do usuário (2026-07-09): a IDE deve cobrir TUDO que uma
configuração Neovim profissional para C/C++ e Rust usa — mas adaptado à
arquitetura do Kinein (core orquestrando ferramentas via contrato IPC
tipado + UI própria das specs), nunca "um Neovim com outra casca". A
regra de adaptação é sempre a mesma: o que no Neovim é um plugin Lua,
aqui é (a) um request/evento no protocolo, (b) um domínio fino no core
chamando a MESMA ferramenta madura, e (c) UI burra conforme docs/specs.

### Inventário: stack Neovim → estado no Kinein (2026-07-09)

```text
FERRAMENTA/RECURSO (plugin típico)      → EQUIVALENTE KINEIN     ESTADO
-- comuns ------------------------------------------------------------
clangd/rust-analyzer (nvim-lspconfig)   → lsp/ do core           FEITO
completion (nvim-cmp)                   → Ctrl+Space + auto      FEITO
diagnostics em buffer (vim.diagnostic)  → Problems + gutter/underline
                                          (T6 FEITA 2026-07-11)         FEITO
formatação (conform.nvim)               → format.text Ctrl+Alt+L FEITO
format-on-save (conform autosave)       → setting M4.1 (FEITA)   FEITO
fuzzy finder (telescope)                → Search Everywhere      FEITO
símbolos @/# (telescope lsp_*)          → M1.4                   FEITO
git signs/hunks (gitsigns)              → M3.1/M3.2              FEITO
git stage/commit (fugitive/neogit)      → M3.3 aba Git           FEITO
blame (gitsigns blame_line)             → M3.4 (FEITA 2026-07-10) FEITO
debugger (nvim-dap + nvim-dap-ui)       → M2.5 DAP lldb-dap      FEITO
  breakpoint condicional (dap)          → pós-M2 (gatilho)       FALTA
  watch/evaluate (dap-ui)               → pós-M2 (gatilho)       FALTA
terminal (toggleterm)                   → aba Terminal PTY       FEITO
tasks (overseer.nvim)                   → M6.3 task runner       FALTA
treesitter highlight                    → QSyntaxHighlighter +
                                          semantic tokens LSP    FEITO*
  (*regex+tokens cobre 90%; textobjects/indent do treesitter não
   têm equivalente — reavaliar na decisão de engine M5.4)
-- C/C++ -------------------------------------------------------------
compile_commands (cmake-tools)          → cmake.configure M2.2   FEITO
  auto-configure ao abrir               → radar (em execução)    PARCIAL
clang-tidy no editor (nvim-lint)        → T2 abaixo              FALTA
switch header/source (clangd ext)       → T1 (FEITA 2026-07-10)  FEITO
inlay hints (clangd_extensions)         → T3 abaixo              FALTA
include cleaner/IWYU (clangd)           → code actions já pegam
                                          parte; T2 completa     PARCIAL
targets de build/run (cmake-tools)      → cmake.targets (M2.2) +
                                          Target selector C5     PARCIAL
sanitizers por perfil (profiles)        → T5 abaixo              FALTA
ctest por caso (neotest)                → test.run ctest cobre o
                                          básico; explorer T7    PARCIAL
doxygen skeleton (neogen)               → T8 (baixa prioridade)  FALTA
-- Rust --------------------------------------------------------------
rust-analyzer full (rustaceanvim)       → lsp/ do core           FEITO
cargo check no save (bacon/cargo-watch) → T4 abaixo              FALTA
runnables/lens (run/debug teste único)  → T7 abaixo              FALTA
inlay hints (types/params/chaining)     → T3 abaixo              FALTA
expand macro (rust-analyzer)            → T8 (baixa prioridade)  FALTA
crates/versões no Cargo.toml (crates.nvim)
                                        → T9, ADAPTADO offline   FALTA
pretty printers Rust no debugger        → T5 (qualidade dos
                                          valores no lldb-dap)   VERIFICAR
clippy (nvim-lint)                      → quality.run            FEITO
```

### Fatias técnicas da trilha T (cada uma segue o ritual do playbook)

```text
T1 Switch header/source (C/C++) [FEITA 2026-07-10, protocolo 0.34.0]:
   request LSP textDocument/switchSourceHeader do clangd (extensão, não
   LSP padrão) → lsp.switchSourceHeader { path, content } → { path? } →
   atalho Alt+O (sem F-key, não precisa de alternativa) e comando
   "C/C++: Alternar header/source" no Search Everywhere. Gate de
   linguagem no core (arquivo não-C/C++ → INVALID_PARAMS). Validada e2e
   com clangd real. Design/descobertas: docs/18, "Fatia T1".
T2 clang-tidy no fluxo do editor: hoje o quality.run roda clippy
   (Rust); para C++ o tidy só roda no gate do repositório. Entra:
   quality.run com kind=cmake roda clang-tidy usando o
   compile_commands.json do .kinein/build (mesmo pipeline de
   diagnostics da aba Problems; -p build dir; arquivo único ao salvar é
   caro — começar com run manual/projeto e medir). Inclui expor os
   checks do .clang-tidy do projeto (nunca lista própria).
T3 Inlay hints (o recurso mais sentido vindo do Neovim modeno):
   textDocument/inlayHint (LSP padrão, clangd e rust-analyzer):
   lsp.inlayHints { path, range } → hints tipados; renderização exige
   texto "fantasma" entre caracteres — CUSTO ALTO no TextEdit atual
   (ver M5.4). Decisão registrada: implementar primeiro como coluna/
   overlay de fim de linha (types only, estilo "-> tipo" à direita da
   linha do cursor/linhas visíveis), e o inline real fica pendurado na
   decisão de engine M5.4. Não fingir o impossível.
T4 Cargo check contínuo (bacon-like): setting (M4.1) "check ao salvar"
   que dispara cargo.check (já existe, M2.3) com debounce de ~1.5s no
   save de *.rs; resultados na MESMA aba Problems substituindo os do
   check anterior (ids estáveis por arquivo+linha+mensagem). clang-tidy
   ao salvar (C++) entra depois com a medição da T2.
T5 Perfis de execução com sanitizers (C++ e Rust): run configs (M2.4)
   ganham "environment/preset": ASAN/UBSAN/TSAN via CMake preset ou
   RUSTFLAGS; debug com sanitizer avisa incompatibilidades conhecidas.
   Inclui VERIFICAR e documentar a qualidade dos pretty printers de
   Rust no lldb-dap (strings/Vec/Option legíveis? se não: avaliar
   --source-init dos scripts rust-lldb no spawn do adapter).
T6 Diagnostics na gutter + underline real no editor [FEITA 2026-07-11,
   protocolo 0.35.0]: Diagnostic ganhou endLine/endColumn/code;
   DiagnosticsController é o store por arquivo (consome o mesmo
   lspDiagnostics da aba Problemas); sublinhado ondulado por severidade
   no EditorHighlighter (SpellCheckUnderline, merge por caractere) +
   marca na gutter com tooltip + navegação F2/Shift+F2 (+Ctrl+Alt+E);
   aba Problemas mostra o code. Validada e2e com clangd e rust-analyzer
   reais. Design/descobertas: docs/18, "Fatia T6". Falta refino do
   Problems 2.0 (marker bar, contadores, underline de build/quality).
T7 Runnables/test explorer: rust-analyzer expõe runnables (lens "Run/
   Debug" em cima de cada teste/main); ctest lista casos. Entra:
   lsp.runnables { path } (r-a experimental/runnables) → chips "▶ test"
   na gutter/linha (usa a infra de gutter existente) que geram run
   config efêmera (cargo test nome_exato / ctest -R caso) e rodam pelo
   pipeline test.run; aba Testes ganha filtro por caso. Debug de teste
   único reusa M2.5 (o binário de teste do cargo com --exact).
T8 Utilidades de linguagem sob demanda (baixa prioridade, alto charme):
   rust-analyzer/expandMacro num popup read-only; geração de doc
   comment skeleton (/// e /** */) no Enter da E2 (comment
   continuation já cobre metade); "Open Cargo.toml"/"related tests"
   como comandos do Search Everywhere.
T9 Dependências do Cargo.toml (crates.nvim ADAPTADO ao offline-first):
   NUNCA consultar crates.io sozinho (rede só por ação explícita).
   v1: cargo metadata + Cargo.lock mostram versão RESOLVIDA inline
   (overlay de fim de linha no Cargo.toml) + comando explícito
   "Cargo: Verificar atualizações" que roda `cargo update --dry-run`
   e lista o resultado (a rede é do cargo, disparada pelo usuário).
```

Ordem sugerida DENTRO da trilha T: T1 → T6 → T4 → T2 → T7 → T5 → T3 →
T9 → T8. Justificativa: T1 é micro; T6 destrava a sensação diária de
editor profissional; T4/T2 fecham o loop de feedback contínuo por
linguagem; T7 muda o fluxo de testes; T3 espera a decisão de engine
para não nascer capado; T9/T8 são polimento.

## Ordem sugerida de execução (depois do M3 fechar)

```text
1. [FEITO] Radar "auto-setup ao abrir" + E2/E3 (trilha E completa).
2. [FEITO 2026-07-10] M3.4 blame/log (M3 FECHADO) + T1 (switch header/
   source). Protocolo 0.34.0.
3. [FEITO 2026-07-11] T6 (diagnostics na gutter/underline) — protocolo
   0.35.0. A maior lacuna diária de editor da trilha T fechada.
4. [FEITO 2026-07-11] M4.1 Settings (protocolo 0.36.0; destravou
   format-on-save, fonte e auto-close; T4 check-no-save e perfis
   consomem este storage a seguir).
5. T4 (cargo check contínuo) + T2 (clang-tidy no editor).
6. M4.3 Robustez (core restart) — antes de dar a IDE a mais amigos.
7. M4.2 Medição de performance (estabelece o orçamento) + T7
   (runnables/test explorer).
8. C5/C6 de docs/20 intercaladas (convergência visual termina no M4);
   M4.4 Start Screen quando os amigos começarem a instalar.
9. M5 na ordem 5.2 → 5.1 → 5.3 → 5.4; T3 (inlay hints) e T5
   (sanitizers/pretty printers) entram colados na decisão de engine
   M5.4 e nos run profiles.
10. M6 na ordem 6.3 → 6.1 → 6.2 → 6.4; T9/T8 como polimento entre elas.
11. [FUNDAÇÃO M7.1 FEITA 2026-07-14] AppImage portátil; matriz de release,
    M7.2–M7.4 e distribuição ampla avançam quando houver testadores reais.
```

Cada item acima, ao ser executado, ganha sua seção "Fatia" em docs/18
com o ritual do playbook. Este doc é atualizado quando um marco inteiro
fecha (mover aprendizados para cá) — ele é o mapa, docs/18 é o diário.

# 19 — Requisitos e trade-offs de arquitetura

> **Status:** ativo
> **Prioridade:** referência permanente (ler antes de decisão estrutural)
> **Fonte de verdade:** o *porquê* das decisões. O *como* está em
> `docs/arquitetura/ARCHITECTURE.md`; o *o quê/quando* em `docs/diario/18-daily-driver-plan.md`
> e `docs/roadmaps/BACKEND_TO_UI_UX_ROADMAP.md`; o visual-alvo em `docs/specs/`
> **Ultima revisao:** 2026-07-17 (D5 com as três camadas de realce e a
> armadilha medida do rehighlight; D9 superado — harnesses QML e teste C++
> existem e estão no gate; RNF5/RNF7 com números medidos e datados)

## Por que este documento existe

O projeto avançou rápido e as decisões de arquitetura, requisitos funcionais
e não funcionais ficaram implícitas em código e commits. Este doc as torna
explícitas — cada trade-off registra o ganho, o custo **aceito de forma
consciente** e o gatilho para revisitar. Pragmatismo aqui significa: a
solução mais simples que atende o requisito; complexidade nova só entra
comprando algo mensurável (desempenho, correção ou manutenção).

## Requisito inegociável: UI/UX dos specs

**A parte visual não é negociável.** Layout, espaçamento, cor, iconografia,
copy e comportamento visual seguem estritamente os `.md` de `docs/specs/`
(entrada: `KINEIN_VECTIS_SPEC_INDEX.md`). Funcionalidade pode ser fatiada,
adiada ou simplificada; o visual **não se inventa nem se "melhora" de
passagem** — divergência de spec é bug ou tarefa explícita com o usuário
ciente. (Já registrado em `ContextoIA.md` e `AGENTS.md`; reafirmado aqui em
2026-07-09.)

## Requisitos funcionais (resumo por área)

Detalhe e sequência vivem em `docs/diario/18` (marcos M0–M4) e no roadmap; aqui só
o mapa com estado em 2026-07-09:

```text
RF1  Workspace: abrir/criar/fechar projeto, browse confinado      [feito]
RF2  Editor: abas, salvar, highlight sintático + semântico        [feito]
RF3  Arquivos: tree, criar/renomear/apagar, busca conteúdo/nome   [feito]
RF4  Build/Test/Quality como jobs canceláveis com Problems        [feito]
RF5  LSP: diagnostics, hover, completion, definition, references,
     rename, semantic tokens                                      [feito]
RF6  Run + terminal PTY integrados                                [feito]
RF7  Ambiente: detecção de ferramentas + scan como job            [feito]
RF8  Project Health mínimo (banner acionável)                     [feito]
RF9  Formatação orquestrada (rustfmt/clang-format, Ctrl+Alt+L)    [feito]
RF10 Code actions/quick fixes LSP                                 [M1.3]
RF11 Go-to-symbol arquivo/workspace                               [M1]
RF12 Sessão por workspace (reabrir abas)                          [M1]
RF13 CMake presets/targets + Cargo metadata/features na UI        [M2]
RF14 Run configurations persistidas                               [M2]
RF15 Debugger via DAP (lldb-dap/gdb)                              [M2]
RF16 Git integrado (status/diff/stage/commit)                     [M3]
RF17 Settings com schema/migração + Strict/Balanced/Relaxed       [M4]
```

## Requisitos não funcionais (explícitos a partir de agora)

```text
RNF1 Privacidade: 100% local/offline; ZERO telemetria; nada sai da
     máquina sem ação explícita do usuário (AGENTS.md).
RNF2 Segurança de workspace: todo acesso a arquivo pelo core é
     canonicalizado e confinado à raiz aberta; a UI nunca toca disco.
RNF3 Responsividade: a UI nunca bloqueia em trabalho pesado; operação
     longa vira job cancelável com eventos. Requests síncronos só quando
     comprovadamente curtos (< ~300ms típicos, ex.: format.text).
RNF4 Robustez: erro estruturado no protocolo com mensagem humana;
     falha de ferramenta externa nunca derruba core nem UI; erros da IDE
     logados em ~/.cache/kinein-vectis/logs/.
RNF5 Manutenibilidade: strict mode máximo com gate único fix-first
     (Rust: clippy pedantic/nursery -D warnings, missing_docs; C++:
     clang-format/tidy Werror; QML: qmllint estrito zero warnings);
     domínios separados, handler fino, teste de comportamento no core.
     Desde 2026-07-17 o verificar.sh soma 7 gates dedicados (catraca de
     split, fiação QML, veracidade dos .md, colisão de presets, lógica
     QML, C++ estático, ctest) — cada um nascido de uma falha que passou
     verde (ARCHITECTURE §4 regra 11); a lista viva é o próprio script.
RNF6 Portabilidade: Linux-first, agnóstico de distro (Arch é o alvo
     principal); bootstrap por scripts/instalar-ambiente.sh; nada de
     caminho hardcoded de distro em código.
RNF7 Performance percebida: startup rápido (LSP/ferramentas sob demanda,
     nunca no boot da UI); highlight semântico assíncrono com debounce.
     Orçamento de digitação JÁ MEDIDO (L0, 2026-07-17): mediana 7,4 ms /
     p95 8,4 ms por tecla, contra orçamento 16/20 ms — harness
     typing_perf_harness.cpp. Primeiro frame do AppImage em backend
     software: 936 ms (2026-07-15). Régua de regressão, não troféu.
RNF8 UX: memória muscular JetBrains (atalhos/fluxos) + visual dos specs.
```

## Decisões de arquitetura e seus trade-offs

Formato: decisão → ganho → custo aceito → quando revisitar.

### D1. UI Qt/QML e core Rust em processos separados via JSON-RPC (stdio)

- **Ganho:** isolamento de falha (core morre ≠ UI trava), linguagem certa
  para cada camada, testabilidade do core por linhas JSON puras (as sondas
  de M1.1/M1.2 provaram o valor), caminho aberto para CLI/clientes futuros.
- **Custo aceito:** serialização em cada chamada; dois toolchains e dois
  builds; contrato para manter (`docs/arquitetura/03` + schemas).
- **Revisitar:** transporte vira Unix socket/gRPC apenas se surgir segundo
  cliente simultâneo ou payloads grandes mensuráveis (não antes).

### D2. Orquestrar ferramentas maduras, nunca reimplementar

rustfmt/clang-format, clangd/rust-analyzer, cargo/cmake/ninja, fd, script(1).

- **Ganho:** qualidade de ferramenta de década no dia 1; manutenção mínima;
  comportamento idêntico ao que o usuário já confia no terminal.
- **Custo aceito:** dependência de PATH/versões da máquina (mitigado por
  tools.detect/environment.scan + modo degradado + instalar-ambiente.sh);
  variação de saída entre versões de ferramenta.
- **Revisitar:** nunca por princípio (AGENTS.md); exceções pontuais só com
  registro aqui.

### D3. Core síncrono de request único + JobManager para o resto

O loop stdio atende um request por vez; build/test/quality/scan são jobs em
thread com eventos.

- **Ganho:** dispatch trivial de raciocinar/testar (sem async runtime, sem
  locks espalhados); jobs dão cancelamento e progresso uniformes.
- **Custo aceito:** um request síncrono lento bloqueia a fila (por isso a
  regra RNF3: longo = job; format.text ficou síncrono por ser curto).
- **Revisitar:** se um request legitimamente curto passar a estourar o
  orçamento (medir antes; a resposta certa pode ser virar job, não async).

### D4. QML por domínio: composition root + controllers + roteadores IPC

`Main.qml` 336 linhas; visual recebe `property`/emite `signal`; estado em
controllers; eventos em `ipc/*`; `CoreClient` fachada única (docs/arquitetura/17).

- **Ganho:** fim do god-file de 4.6k linhas; mudança local fica local;
  qmllint estrito vira viável.
- **Custo aceito:** mais arquivos e indireção; fio manual de sinais no
  composition root a cada feature.
- **Revisitar:** não regride; se o fio manual doer, gerar/agrupar — nunca
  voltar estado para componente visual.

### D5. Editor sobre TextEdit + QSyntaxHighlighter (não engine própria)

- **Ganho:** editor funcional imediato. O realce hoje é em **três camadas**
  (atualizado em 2026-07-17; era só regex+LSP na escrita original): regex como
  fallback offline → captures estruturais do **Tree-sitter** (ADR-0002) →
  semantic tokens do LSP como autoridade. As três desembocam no mesmo
  `QSyntaxHighlighter` (`ui/src/editor_highlighter.cpp`); Tree-sitter NÃO virou
  engine de editor — alimenta tokens, como o LSP.
- **Custo aceito:** sem multi-cursor real, minimap, split; regex é aproximação
  (as camadas de cima corrigem). E um custo **medido em 2026-07-17**: o
  `rehighlight()` do QSyntaxHighlighter marca o documento como alterado mesmo
  quando só o formato mudou (Qt 6.11.1) — quem consome `onTextChanged` precisa
  da barreira de texto-realmente-mudou (ver `EditorTextSurface.qml`), senão
  cada passada de realce se apresenta como edição do usuário.
- **Revisitar:** pós-V1, se edição avançada virar o gargalo do daily use —
  aí avaliar engine de editor dedicada (decisão grande, nova entrada aqui).

### D6. Semantic tokens: full-file com debounce, ancorados por linha

- **Ganho:** zero sincronização incremental (delta encoding, shifts de
  edição); implementação pequena e correta na prática.
- **Custo aceito:** ao inserir/remover linhas, cores podem ficar ~600ms +
  roundtrip deslocadas até o refresh; requisição full a cada pausa de
  digitação (payload pequeno em arquivos típicos).
- **Revisitar:** se arquivos grandes (>5k linhas) mostrarem custo medido,
  adotar range/delta do protocolo LSP.

### D7. Formatação: ferramenta direta, síncrona, sobre o buffer (M1.1)

Registrado em detalhe em `docs/diario/18` (design M1.1). Resumo do trade-off:
independência de LSP vivo e zero side effect em disco, ao custo de manter
seleção de formatter por extensão no core.

### D8. Strict mode máximo com gate fix-first

- **Ganho:** regressão aparece no gate, não no uso; código homogêneo;
  degraus novos (qmllint 117→0) provaram o método fix-first.
- **Custo aceito:** fricção por entrega (minutos de gate completo); recusas
  do clippy pedantic exigem justificar exceções raras.
- **Revisitar:** relaxar exige motivo registrado (AGENTS.md); a direção
  preferida é subir degraus (docs/diario/18, escada de rigor).

### D9. Testes em todas as camadas — o "UI sem harness" foi SUPERADO

> **Atualizado em 2026-07-17.** O texto original ("UI sem harness automatizado
> (hoje)") era verdade em 2026-07-09 e o gatilho de revisita disparou: os
> harnesses existem e estão no gate.

- **Estado medido (2026-07-17):** testes Rust de comportamento no core;
  **15 harnesses QML headless** (`scripts/qml-harness/tst_*.qml`, controllers
  reais com fakes injetados — nenhum importa o módulo C++, por construção); e o
  **primeiro teste C++** (`ui/tests/`, QTest via ctest no preset debug com
  sanitizers). Tudo dentro do `verificar.sh`.
- **Ganho:** os harnesses já provaram reprovar (a §0.2i do `PONTO_ATUAL`
  documenta a suíte que não sabia falhar e o conserto); teste novo só entra
  provado por mutação.
- **Custo aceito que PERMANECE:** pintura (pixels do highlighter, layout
  visual) segue sem teste automatizado — smoke offscreen + aceite humano.
  Regressão puramente visual ainda pode passar.
- **Revisitar:** quando uma regressão visual escapar, avaliar snapshot de
  render offscreen — decisão nova aqui.

### D10. Protocolo 0.x sem compatibilidade retroativa formal

- **Ganho:** liberdade de evoluir contrato rápido enquanto o único cliente
  é a própria UI (mesma árvore, mesmo commit).
- **Custo aceito:** nenhum cliente externo pode depender do protocolo antes
  do V1; toda mudança exige docs/arquitetura/03 + schemas sincronizados no mesmo commit.
- **Revisitar:** congelar/versionar de verdade quando existir consumidor
  fora do repositório (CLI pública, plugins).

### D11. Sem telemetria; diagnóstico por log local

- **Ganho:** privacidade absoluta (RNF1), confiança, zero infra.
- **Custo aceito:** melhoria guiada só por uso próprio e reports manuais.
- **Revisitar:** não revisitar; decisão de produto permanente (AGENTS.md).

### D12. UI atual → specs por convergência gradual, não remake big-bang

Decisão de 2026-07-09, confirmada com o usuário. Plano vinculante completo
(regras, inventário de divergências, ordem C0–C6 e definition of done):
`docs/roadmaps/20-ui-spec-convergence-plan.md`.

- **Ganho:** IDE utilizável durante toda a transição (dogfooding contínuo,
  que é o critério dos marcos); risco de regressão proporcional ao tamanho
  da fatia (não há teste visual automatizado — D9); reaproveita a separação
  visual/lógica já paga em docs/arquitetura/17; UI nova (Main Toolbar, Assistente,
  Start Screen) nasce conforme spec de graça.
- **Custo aceito:** estados intermediários visualmente híbridos (área nova
  conforme convivendo com área antiga) até a C6.
- **Revisitar:** gatilhos objetivos na seção "Gatilho de remake" de
  docs/roadmaps/20 (acoplamento visual pior que o mapeado na C1; estouro de 2x em
  duas fatias C seguidas; híbrido atrapalhando o uso diário — relato do
  usuário).

## Como usar este documento

```text
1. Decisão estrutural nova (ou exceção a uma existente) entra aqui no
   mesmo formato: ganho / custo aceito / gatilho de revisita.
2. Custo aceito começou a doer de verdade (medido, não intuído)? Esse é o
   gatilho: reabrir a decisão numa fatia própria, nunca "de passagem".
3. Requisito novo (funcional ou não) primeiro ganha linha aqui; depois
   vira fatia em docs/diario/18.
```

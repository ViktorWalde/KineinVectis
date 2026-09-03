# 34 — Depois do MVP: as quatro frentes, medidas

> **Classe: PLANO.** Diverge da implementação por natureza — é para isso que
> existe. O que **não** pode divergir são os números da §1: eles foram medidos em
> **2026-09-02** e cada um diz como remedi-lo.
>
> Sucessor de [30-caminho-para-o-mvp.md](30-caminho-para-o-mvp.md), que fechou
> em 2026-09-02 com 9 das 10 etapas feitas e uma paga em parte.

## 0. Como ler isto

**Regra zero: fila é hipótese, não estado.** Antes de aceitar qualquer item
abaixo como pendente, MEÇA — `ls` no artefato, `git log` na área, `grep` no
gate. Em 2026-08-30 essa regra derrubou duas entradas falsas de fila no mesmo
dia; em 2026-09-02 ela mostrou que a etapa 4 já estava verde com a fiação
removida.

Cada item desta lista carrega **o comando que decide se ele ainda existe**. Item
sem forma de medir não entra aqui.

## 1. Onde o projeto está, medido em 2026-09-02

```text
433 testes Rust verdes          cargo test
protocolo 0.65.0                crates/kinein-protocol/src/lib.rs
114 metodos IPC, 19 dominios    docs/LEITURA_TECNICA.md §3
gate completo verde             bash scripts/verificar.sh
AppImage no gate                scripts/verificar-appimage.sh
5 sondas                        scripts/sonda_*.py
17 arquivos em debito           cat scripts/arquitetura-baseline.txt
```

O MVP fechou. O que vem agora não é "terminar" — é **transformar MVP em
ferramenta de uso diário** (TR1 do `GUIAIA.md`) e depois em algo que se compare
ao CLion em profundidade (TR2).

## 2. As quatro frentes, e por que a ordem não é óbvia

```text
A  DIVIDA QUE JA COBRA PEDAGIO   17 arquivos acima do limite da catraca.
                                 Nao e' limpeza: e' imposto sobre a PROXIMA
                                 fatia.

B  ATRITO DIARIO MEDIDO          o que forca sair da Kinein para outra
   (TR1)                         ferramenta. Hoje NAO EXISTE esse registro.

C  PROFUNDIDADE                  targets, contextos, brokers, debugger de
   (TR2)                         verdade, split/multicursor.

D  SIMULACAO FISICA/MATEMATICA   roadmaps/31 — ESTUDO, nao fila. Vira etapa
                                 quando as perguntas dele tiverem resposta.
```

### A frente A não se paga de uma vez, e a evidência é o roadmap 30 inteiro

Cinco das dez etapas do MVP tiveram de cortar um arquivo em débito **antes** de
poder acrescentar:

| etapa | o que ia acrescentar | o que teve de cortar antes |
| --- | --- | --- |
| 2 | Configuration Actions | `commands.rs` 696 → pasta `commands/` |
| 3 | servidor LSP falso | `lsp/manager.rs` → `lsp/session.rs` |
| 4 | reabrir após configure | `core_client_dispatch.cpp` 751 → 640 |
| 6 | (era o próprio corte) | `EditorController.qml` 1070 → 791 |
| 9 | busca multi-linha | `SearchController.qml` 420 → 185 |

**A leitura correta disso não é "pague tudo antes".** É: o débito cobra na hora
em que uma fatia toca o arquivo, e o corte sai melhor quando há uma fatia
concreta dizendo *qual* responsabilidade está sobrando. Cortar por cortar produz
divisão por tamanho — exatamente o que a `ARCHITECTURE.md` §4 regra 9 proíbe.

**As duas exceções, que se pagam independentemente de fatia:**

1. o que viola uma **regra**, não só um número (§3.1);
2. a **decisão em aberto** do autor, que bloqueia por não estar respondida (§3.2).

### Por que a frente B vem antes da C, mesmo sendo menor

Porque a C tem nove itens e nenhum critério para ordená-los. A B **produz esse
critério**: um registro de saídas ("precisei do VS Code para X") ordena a C por
dor real em vez de por intuição. O `GUIAIA.md` §2 já diz isso — *"esses dados
passam a ordenar o backlog antes de confortos hipotéticos"* — e o dado não
existe.

## 3. FRENTE A — a dívida, medida e ordenada

`cat scripts/arquitetura-baseline.txt` (2026-09-02, 17 arquivos):

| arquivo | linhas/limite | o que provavelmente está misturado |
| --- | ---: | --- |
| `ui/src/editor_highlighter.cpp` | 910/500 | o maior do repositório: regex fallback + Tree-sitter + semantic tokens + diagnósticos + busca, quatro camadas de composição num arquivo |
| `ui/qml/editor/EditorController.qml` | 791/400 | fachada (64 de 97 funções delegam) — **decisão em aberto**, §3.2 |
| `ui/qml/panels/bottom/GitPanel.qml` | 764/300 | status + diff + stage + commit + histórico |
| `crates/kinein-core/src/dap/session.rs` | 672/500 | handshake DAP + breakpoints + stepping + frames + variáveis |
| `ui/src/core_client_requests.cpp` | 660/500 | todo request de todo domínio numa fachada |
| `crates/kinein-core/src/handlers/lsp.rs` | 653/500 | **viola a §4**, não só o número — §3.1 |
| `ui/src/core_client_dispatch.cpp` | 640/500 | já cortado uma vez (751→640) na etapa 4 |
| `crates/kinein-core/src/lsp/parse.rs` | 598/500 | parse de cada resposta LSP no mesmo arquivo |
| `ui/qml/shell/ShellWorkspaceHost.qml` | 576/400 | host central; **92 leituras** de `editorController` — trava a §3.2 |
| `ui/qml/editor/EditorTextController.qml` | 574/400 | cursor + seleção + indentação + gestos |
| `ui/qml/editor/EditorDocumentController.qml` | 548/400 | abas + buffers + save + externo |
| `ui/qml/editor/EditorPane.qml` | 538/300 | visual |
| `ui/qml/editor/EditorTextSurface.qml` | 504/300 | visual |
| `ui/qml/git/GitController.qml` | 464/400 | |
| `ui/qml/panels/bottom/DebugPanel.qml` | 370/300 | visual |
| `ui/qml/editor/EditorFindBar.qml` | 329/300 | visual |
| `ui/qml/panels/bottom/SearchPanel.qml` | 325/300 | trava o `TextArea` do replace (`arquitetura/33` §8a) |
| `ui/qml/panels/bottom/TerminalViewport.qml` | 319/300 | visual |

**Como medir se esta tabela ainda vale:** `bash
scripts/verificar-arquitetura.sh` — a catraca só permite encolher, então
qualquer número aqui só pode ter melhorado.

### 3.1 A dívida que é violação de REGRA (paga-se sem esperar fatia)

`crates/kinein-core/src/handlers/lsp.rs` — 653 linhas. A `ARCHITECTURE.md` §4
diz, com todas as letras:

> *"`handlers/<dominio>.rs` é fino. Roteia, valida params, delega, formata a
> resposta. Sem lógica pesada."*

Um handler de 653 linhas não é um handler grande: é um handler que virou
domínio. O corte aqui não precisa de uma fatia funcional para saber o que sobra —
a regra já disse. Alvo: `lsp.rs` roteia; o que ele faz de pesado desce para
`lsp/`, que já é pasta.

**Medir:** `wc -l crates/kinein-core/src/handlers/lsp.rs` e `grep -c 'fn '` nele.

### 3.2 A decisão em aberto: `EditorController.qml` 791/400

Registrada em
[`arquitetura/32-editor-por-responsabilidade.md`](../arquitetura/32-editor-por-responsabilidade.md)
§8, e **é decisão do autor, não da sessão** (`ARCHITECTURE.md` §4 regra 8:
levantar um limite é decisão explícita registrada).

O estado, medido em 2026-09-02: das 97 funções restantes, **64 são delegação de
uma linha**. O arquivo é fachada, não lógica. Quebrá-la custa ~180 pontos de
chamada em 13 arquivos, **92 deles em `ShellWorkspaceHost.qml`** — que está em
576/400 e ficaria pior.

Três saídas, e nenhuma é obviamente certa:

```text
(a) TERCEIRA FATIA        cortar ANTES o ShellWorkspaceHost: as 92 leituras de
    -- recomendada        propriedade do editor viram um punhado de
                          propriedades agregadas. Os dois arquivos melhoram, e
                          so' entao o EditorController se quebra.
(b) LEVANTAR O LIMITE     decisao registrada do autor: "fachada de composicao
                          tem limite proprio". Honesto se for escrito e
                          justificado; perigoso se virar habito.
(c) DEIXAR COMO ESTA      a catraca ja' impede crescer. Custa nada hoje e
                          cobra na proxima fatia que tocar o editor.
```

**Esta é a primeira pergunta que a próxima sessão deve fazer ao autor.**

## 4. FRENTE B — o atrito diário (TR1)

### 4.1 O registro de saídas NÃO EXISTE

O `GUIAIA.md` §2 define o protocolo: ao ouvir *"estou no Kinein"*, registrar cada
saída para outra ferramenta com **motivo exato, projeto/arquivo, ação que
faltou, impacto e reprodução mínima**. O gatilho já foi recebido (PONTO_ATUAL
§0), e o que existe é **prosa de sessão**, não uma lista consultável.

**Medir:** não há artefato. `ls docs*/` não mostra nenhum registro de saídas.

A fatia é pequena e desbloqueia toda a frente C: um arquivo append-only
(`docs-privada/diario/` é o lugar natural) com uma linha por saída, e a regra de
que **entrada sem reprodução mínima não conta**. Sem isso, "o que falta na IDE" é
opinião.

### 4.2 O que está medidamente ausente hoje

Medido em 2026-09-02, por `grep` no repositório:

```text
split editor        AUSENTE   grep -rn "split" ui/qml/editor/ so' acha
                              String.split() de breadcrumb
multicursor         AUSENTE   grep -rln multicursor ui/ -> vazio
EditorConfig        AUSENTE   e' DECISAO, nao esquecimento: auditoria de
                              2026-07-16 (PONTO_ATUAL §0.2e) deu RESULTADO
                              NEGATIVO — nao ha' crate Rust madura. Nao
                              reabrir sem auditoria nova.
watches no debug    AUSENTE   grep -rn evaluate crates/kinein-core/src/dap/
                              -> vazio. Ha' `debug.variables` (escopos e
                              frames), nao ha' avaliacao de expressao.
breakpoint          AUSENTE   `debug.setBreakpoints` aceita file + lines;
  condicional                 nao ha' condicao nem hit count.
TextArea no replace AUSENTE   `arquitetura/33` §8a — a sintaxe `\n` e' a
                              saida honesta, nao a definitiva.
```

Nenhum destes é "bug": são ausências. O que decide a ordem entre eles é a §4.1,
não esta lista.

## 5. FRENTE C — profundidade (TR2), com o estado medido item a item

A ordem arquitetural do `GUIAIA.md` §2, conferida contra o disco em 2026-09-02:

| # | item do TR2 | estado medido | onde |
| ---: | --- | --- | --- |
| 1 | CMake File API + Cargo Metadata | ✅ `codemodel-v2` oficial, nunca parser próprio | `crates/kinein-core/src/cmake.rs` |
| 2 | Unified Project Graph + Context Matrix | ❌ ausente | — |
| 3 | targets, perfis e **toolchains** como entidades | ⚠️ **em parte** (2026-09-02): papel → executável, persistido, chega ao configure e ao build. Faltam **sysroot, cross-compilação e kit por preset** | `crates/kinein-core/src/toolchain/` |
| 4 | scheduler LSP por documento/contexto, cancelamento, backpressure | ❌ ausente. Existem os **dois relógios** (`syntaxVersion`/`semanticVersion`), que descartam resposta obsoleta — é outra coisa | `arquitetura/32` §3 |
| 5 | Symbol Broker e Diagnostic Broker | ❌ ausente | — |
| 6 | painel de Effective Compile Context | ❌ ausente. O diagnóstico de CDB (0.62.0) é o primeiro degrau dele | `crates/kinein-core/src/cdb.rs` |
| 7 | debugger com watches, variáveis, pilha, pretty-printers | ⚠️ **em parte**: pilha ✅, variáveis ✅, watches ❌, pretty-printers ❌ | `crates/kinein-core/src/dap/` |
| 8 | split editor, multicursor, EditorConfig | ❌ os três (ver §4.2) | — |
| 9 | testes prolongados em projetos reais | ⚠️ existe soak sintético; falta projeto real | `scripts/sonda_soak.py` |

**O item 3 é o mais barato agora**, porque o encaixe existe: o domínio
`toolchain/` foi construído em 2026-09-02 e sysroot/cross/kit são campos e regras
dentro dele, não uma entidade nova.

**O item 7 encaixa na dívida**: `dap/session.rs` está em 672/500, e `evaluate`
(watches) + breakpoint condicional são exatamente o tipo de acréscimo que a
catraca vai barrar. Cortar e acrescentar na mesma fatia é o padrão que o roadmap
30 usou cinco vezes.

## 6. FRENTE D — a simulação física/matemática

[`31-simulacao-fisica-matematica.md`](31-simulacao-fisica-matematica.md), pedida
pelo autor em 2026-09-01: configuração e entrada de fórmulas **por layout**, a
IDE calculando a partir do conceito matemático/físico selecionado mais a equação
do usuário, exibição via OpenGL.

**Continua sendo ESTUDO, e de propósito.** Ele lista as perguntas que precisam de
resposta antes de existir arquitetura — e uma delas colide com um invariante já
travado no gate: `scripts/verificar-appimage.sh` verifica que a UI é **100% 2D**
(sem `ShaderEffect`, `QOpenGL`, `QRhi`, `QtQuick3D`), porque o AppImage força
renderer por software.

**A etapa, quando vier, é responder às perguntas do 31 e produzir arquitetura** —
não começar a implementar. E a primeira pergunta é essa: OpenGL na mesma janela
que hoje é garantidamente 2D, ou processo/janela separada?

## 7. Ordem linear recomendada

Recomendação, não decreto. O autor corta onde quiser — mas cada troca de ordem
tem um custo escrito acima.

```text
11  DECIDIR o EditorController (§3.2)      pergunta, nao codigo. Bloqueia
                                           qualquer fatia que toque o editor.

12  Registro de saidas do dogfooding       §4.1. Pequeno, e ordena a frente C
    (§4.1)                                 inteira. Sem ele, o resto e' palpite.

13  handlers/lsp.rs: handler volta a ser   §3.1. Violacao de REGRA; nao precisa
    fino                                   esperar fatia funcional.

14  Toolchain: sysroot + cross + kit por   TR2 item 3. O encaixe ja' existe;
    preset                                 e' o item barato da frente C.

15  Debug: `evaluate` (watches) +          TR2 item 7 + divida do dap/session.rs
    breakpoint condicional                 (672/500) na MESMA fatia.

16  editor_highlighter.cpp (910/500)       o maior do repositorio, e as quatro
                                           camadas de composicao sao a proxima
                                           coisa a mudar quando o realce mudar.

17  GitPanel.qml (764/300) +               a frente git inteira, cortada por
    GitController.qml (464/400)            responsabilidade.

18  Simulacao: responder as perguntas do   §6. Arquitetura, nao implementacao.
    roadmaps/31
```

**Se o dogfooding (12) produzir um bloqueador concreto, ele fura esta fila.** É a
regra do `GUIAIA.md` §2 e da PONTO_ATUAL §0: perda de dados / crash / bloqueio
diário vêm antes de qualquer item planejado.

## 8. O que este documento deliberadamente NÃO inclui

Nada aqui reabre decisão registrada:

```text
IA na IDE         FORA DE ESCOPO desde 2026-07-17 (docs-legada/)
Python            adiado por decisao de 2026-08-30; foco em C/C++ e Rust.
                  A excecao do Tree-sitter de Python NAO foi tomada.
Pylance           PROIBIDO (licenca so' para produtos Microsoft). Se Python
                  voltar: basedpyright (MIT).
Docker / banco    NATIVOS, nao plugins.
host de plugins   nunca houve; nao ha' runtime de plugin de terceiro.
EditorConfig      auditado em 2026-07-16 com resultado NEGATIVO (§4.2). So'
                  volta com auditoria nova.
```

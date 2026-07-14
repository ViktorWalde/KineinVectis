# 24 — Fase pós-rede-de-segurança: paridade e fundação (D1–D4)

> **Status vivo.** Ordem definida pelo usuário em 2026-07-11, depois da
> rede de segurança (docs/23). Princípio: **bugs de uso diário primeiro**
> (D1, D2), **fundação depois** (D3, D4) — o dogfooding informa a fundação.
> O usuário está estruturando os **ícones próprios da IDE** em paralelo.
>
> ⚠️ **2026-07-12 — o princípio acima perdeu a base: DOGFOODING PAUSADO.**
> O Viktor tentou desenvolver o Kinein no próprio Kinein, concluiu que ainda
> não se sustenta pro trabalho real, e **voltou pro CLion até o projeto ficar
> pronto**. Sem uso diário, não chega mais bug de uso diário (a fonte que
> gerou D1, D1b e D2 secou), e a régua vira **"está pronto pra virar o daily
> driver?"** — o que empurra o valor pra **fundação (D3/D4)** e pro
> **inventário do que ainda falta**. A reordenação é decisão do Viktor e está
> em aberto (ver `PONTO_ATUAL.md`). Nada foi reordenado sem ele.
>
> ⚠️ **D1 e D1b nunca foram confirmados AO VIVO** pelo Viktor — estão verdes
> no código, no gate e em harness headless, mas ninguém viu na tela.

| Fase | Tema | Tipo | Status |
|---|---|---|---|
| **D1** | Autocomplete LSP aparecer AO VIVO na GUI | 🐞 bug de uso diário | 🟢 CAUSA-RAIZ ACHADA E CORRIGIDA (2026-07-12; falta o OK ao vivo do usuário) |
| **D1b** | Find/Replace no arquivo (Ctrl+F / Ctrl+H) | 🐞 bug de uso diário | 🟢 IMPLEMENTADO (2026-07-12; falta o OK ao vivo do usuário) |
| **D2** | Terminal → paridade VS Code/JetBrains | 🐞 lacuna de uso diário | 🟢 D2.1 + D2.2 + D2.3 FEITOS (0.44.0; falta OK ao vivo das abas) |
| **D3** | Tree-sitter + arquitetura de plugins + views em árvore | 🏗️ fundação | 🟢 FEITO (2026-07-14; protocolo 0.46.0) |
| **D4** | Convergência UI/UX (docs/20 + specs) | 🏗️ fundação | 🟢 C2/C3/C5 implementadas; C6 aguarda validação visual do usuário |

> **"O que falta para usar sem medo (não ir pro Zed)?"** (usuário,
> 2026-07-12): a infra difícil já existe. Os **bloqueadores reais** são os
> dois 🐞 de uso diário — **D1 (autocomplete ao vivo)** e **D1b
> (Find/Replace no arquivo, que hoje NÃO existe — só há busca no projeto
> Ctrl+Shift+F e Search Everywhere)**. Eles valem MAIS que a fundação (D3/
> D4) para usabilidade. Recomendado: encaixar D1 + D1b antes/junto do D3.
> Ordem final é decisão do usuário.

---

## D1 — Autocomplete LSP ao vivo

**Sintoma (usuário):** ao digitar, o popup de sugestão do LSP não aparece
em tempo real na GUI. Já foi "corrigido" antes por **sonda** (o backend
responde `std::cou → cout`), mas sonda testa o BACKEND, não o popup na
tela — por isso passou batido. É bug de **UI**.

**Diagnóstico de código (2026-07-11):** o fluxo está fiado corretamente
ponta-a-ponta:
`EditorTextSurface.onTextEdited → ShellWorkspaceHost → EditorController.
handleTextEdited → EditorCompletionController.handleTextEdited →
completionDebounce(250ms) → requestCompletion → completionRequested →
coreClient.requestCompletion → lsp.completion → lspCompletionResolved →
EditorController.handleCompletionResolved → completionController.
handleResolved → refilter → visible`; e o `EditorCompletionPopup` está
ligado a `completionVisible`/`completionModel` e posicionado no cursor.
Ou seja, **não é fiação quebrada** — é algo mais sutil no meio.

**Hipóteses ranqueadas (confirmar no app VIVO — dogfooding):**

1. **[ALTA] Filtro só-prefixo esconde o resultado do servidor.** O
   `refilter()` só mantém item cujo `insertText`/`label` faz
   `indexOf(prefix) === 0` (prefixo exato). LSP (clangd/rust-analyzer)
   devolve matches FUZZY/subsequência; se o rótulo não começa pelo
   prefixo digitado, some tudo → popup vazio → `visible=false`. Provável
   causa de "não aparece nada" mesmo com o servidor respondendo.
2. **[MÉDIA-ALTA] Guard de foco.** O `completionDebounce` só dispara se
   `editorSurface.editorActiveFocus` (= `textEditor.activeFocus`). Se o
   `TextEdit` não segura o activeFocus do Qt de forma confiável (foco num
   FocusScope acima), o debounce sai cedo e nunca pede.
3. **[MÉDIA] Prontidão do servidor.** rust-analyzer indexando (~s após
   abrir) devolve vazio; clangd em C++ **precisa de `compile_commands.
   json`** (rodar "CMake: Configure") pra completar stdlib. Sem feedback
   de "indexando", parece que "não funciona".
4. **[trigger] `std::cout <<` não sugere "o resto".** `<<` não é
   caractere de auto-open (só palavra/`.`/`:`), e "o que imprimir" é
   signature help / snippet do LSP — gap conhecido (radar), não o mesmo
   bug do popup geral.

**Plano D1:** confirmar (1) e (2) no app rodando; provável fix =
(a) refilter por SUBSEQUÊNCIA/fuzzy em vez de só-prefixo (e/ou confiar no
ranking do servidor sem re-filtrar tão duro), (b) endurecer o gatilho de
foco, (c) honrar os TRIGGER CHARACTERS do LSP (`.`, `::`, `<`, `(`) e dar
feedback de "servidor indexando". Validar com o usuário digitando de
verdade (não sonda).

**Progresso e estado (2026-07-12): AINDA NÃO RESOLVIDO.**
- Aplicado o fix (a): `refilter()` usa `fuzzyMatch` (subsequência
  case-insensitive) em vez de só-prefixo. **Mantido** (é melhoria real),
  mas NÃO resolveu — o popup continua não aparecendo ao digitar.
- Tentativa de diagnóstico via `console.log` (`[D1]`) num terminal
  EXTERNO: o usuário seguiu as instruções, digitou por ~1 min num arquivo,
  e **nenhum autocomplete apareceu**; o loop de capturar log em terminal
  externo se mostrou impraticável (não trouxe os `[D1]` de forma útil).
  Instrumentação `console.log` foi **REMOVIDA** (não pode spammar o
  console durante o uso). Só o fuzzy ficou.

**Decisão (2026-07-12): D1 ESTACIONADO, priorizar D2 (terminal) primeiro.**
Motivo (do usuário): um terminal profissional + a aba IDE legível DENTRO
do Kinein tornam o diagnóstico deste bug (e todo o loop de dev) muito mais
prático — dá pra ver o log ao vivo sem ginástica de terminal externo.

### ✅ CAUSA-RAIZ ENCONTRADA E CORRIGIDA (2026-07-12)

Nenhuma das hipóteses acima era a causa. Não era foco, nem prontidão do
servidor, nem posição do popup, nem o filtro. **Era uma armadilha de
semântica do Qt Quick.**

**A causa (`Item.visible` NÃO é um flag booleano qualquer):**

```text
EditorController        → Item { visible: false }   // é CONTROLLER, não UI
  └─ EditorCompletionController → Item { visible: false }
         refilter():  visible = true                // grava explicitVisible
         EditorController: property alias completionVisible:
                           completionController.visible   ← LÊ o Item.visible
```

`Item.visible` LÊ a **visibilidade EFETIVA** (`explicitVisible && pai
efetivamente visível`), não o valor que você gravou. Como o controller vive
dentro do `EditorController` — que é um `Item` **invisível** (é controller,
não UI) — a leitura de `completionController.visible` ficava **presa em
`false` para sempre**, e o `visibleChanged` **nem era emitido** (o efetivo
não mudou). Resultado: `refilter()` gravava `visible = true`, o modelo
enchia com os itens certos, e o popup **nunca abria** — o binding do
`EditorCompletionPopup` jamais reavaliava.

Isso explica cada sintoma: o backend respondia (sonda OK), a fiação estava
correta (rastreada no código), e o **fuzzy funcionava** — os itens estavam
lá, só que o popup era incapaz de ficar visível. Por isso o fix do fuzzy
"não resolveu nada".

**O fix:** estado do popup numa property PRÓPRIA
(`property bool popupVisible`), nunca no `visible` do `Item`; o alias passa
a apontar para ela. Properties `bool` comuns não sofrem influência do pai.

```diff
- property alias completionVisible: completionController.visible
+ property alias completionVisible: completionController.popupVisible
```

**Regra pro projeto (evita a reincidência):** controller QML é `Item
{ visible: false }` por ser não-visual — então **nunca** use o `visible`
dele (nem de um filho dele) como estado de UI, e **nunca** faça alias para
`.visible`. Use property própria. Os irmãos já faziam certo
(`hoverVisible`, `usagesVisible`, `actionsVisible` são `bool` próprios) —
`completionVisible` era o **único** alias para `.visible` em todo o módulo
QML, e era exatamente a única feature quebrada.

**Como foi provado (sem depender de digitar na GUI):**
1. Cena QML mínima reproduzindo a hierarquia (`Item` invisível > `Item`):
   set `visible = true` → leitura devolve `false`; o mesmo `Item` sob pai
   visível devolve `true`. Semântica do Qt confirmada empiricamente.
2. Harness headless (`qml6`, offscreen) carregando o
   **EditorCompletionController.qml REAL** com bridges falsos: dispara
   `handleResolved([String, Struct, into_iter])` com prefixo `St`.
   - código PRÉ-fix → `popupVisible` **false** (popup não abre) e o modelo
     com os 2 itens certos = o bug exato do usuário, reproduzido.
   - código PÓS-fix → popup abre, fuzzy mantém `String`+`Struct`, `dismiss`
     fecha, lista vazia não abre. Tudo verde.
3. Gate `scripts/verificar.sh` verde + smoke offscreen (exit 124).

**Instrumentação removida:** o singleton C++ `DebugLog` (`ui/src/debuglog.*`)
e as notas `[D1]` foram **apagados** — eram scaffolding temporário e a causa
foi achada de forma determinística, sem precisar do log ao vivo.

**Pendente:** confirmação do usuário digitando de verdade num `.rs`/`.cpp`
(o único passo que falta pra fechar o D1).

**Lacuna que este bug expôs (vale uma fatia):** **não existe teste de QML**
no projeto. O gate cobre Rust (fmt/clippy/testes), C++ (clang-format/tidy) e
`qmllint` — mas nada EXECUTA a lógica QML, que é justamente onde a IDE
guarda o estado da UI. Um bug de UI silencioso sobreviveu a dois ciclos de
"correção" por causa disso. Proposta: alvo Qt Quick Test (`qmltestrunner`)
no CMake + teste do `EditorCompletionController` (o harness acima vira o
primeiro caso) plugado no `verificar.sh`.

---

## D1b — Find/Replace no arquivo (design 2026-07-12)

**O buraco:** hoje só existe busca NO PROJETO (`Ctrl+Shift+F`, ripgrep no
core) e o Search Everywhere. **Buscar dentro do arquivo aberto não existe** —
é o gesto mais usado de um editor (`Ctrl+F`/`Ctrl+H`) e o último bloqueador
de uso diário.

**Decisão de arquitetura: 100% UI, ZERO core.**

```text
- O buffer JÁ ESTÁ na UI (o TextEdit é a fonte da verdade do texto aberto).
  Buscar nele não precisa de IPC: mandar o texto pro core e voltar só
  adicionaria latência e um contrato novo pra manter em sincronia.
- Logo: protocolo SEGUE 0.42.0, nenhum RPC novo, nenhum handler novo.
  (Contraste com Ctrl+Shift+F, que É do core: lá o ripgrep varre o DISCO,
  arquivos que a UI nem tem em memória.)
- O core só ganha os DESCRIPTORS dos comandos (`editor.find`,
  `editor.replace`) pra aparecerem na paleta (Ctrl+Shift+A) — a paleta é
  alimentada por `command.list`. Execução é UI-side no CommandDispatcher,
  igual ao `settings.get`.
```

**Peças:**

| Arquivo | Papel |
|---|---|
| `EditorFindController.qml` (novo) | Lógica pura: acha matches, índice atual, wrap, substituir, opções |
| `EditorFindBar.qml` (novo) | A barra (campo, contador, toggles, ▲▼, substituir, fechar) |
| `EditorPane.qml` | Hospeda a barra ancorada no topo do editor |
| `EditorTextSurface.qml` | Repassa os matches pro highlighter |
| `editor_highlighter.{h,cpp}` | Canal NOVO de spans: destaca TODAS as ocorrências, a atual mais forte |
| `GlobalShortcuts.qml` | `Ctrl+F`, `Ctrl+H`, `F3`/`Shift+F3` (+ alternativas sem F-key) |
| `CommandDispatcher.qml` + `commands.rs` | Comandos na paleta (categoria Editor) |

**Comportamento (paridade VS Code/JetBrains):**
- `Ctrl+F` pré-preenche com a **seleção**; sem seleção, com a **palavra sob
  o cursor**. Foca o campo e seleciona tudo (digitar substitui na hora).
- `Ctrl+H` abre a mesma barra **com a linha de substituição**.
- `Enter` = próximo, `Shift+Enter` = anterior, `Esc` = fecha e devolve o
  foco ao editor no match atual. Busca **circular** (wrap), começando do
  cursor. Contador **"3 de 17"** / "Nenhum resultado".
- Toggles: **Aa** (case), **W** (palavra inteira), **.\*** (regex).
- **Substituir** troca o atual e avança; **Substituir tudo** troca numa
  passada **de trás pra frente** (não invalida os offsets à frente).
- Regex inválida → campo em estado de erro, 0 matches, sem crash.
- Editar o texto **recomputa** os matches.

**Armadilhas endereçadas no design:**
- **Regex de largura zero** (`a*`, `\b`) → laço infinito. O scanner AVANÇA
  o índice à força quando o match tem length 0.
- **`Item.visible`** → a regra do D1 vale aqui: o estado da barra é
  `property bool barVisible` no controller, **nunca** o `visible` do `Item`.
- **F-key** → regra do repo: toda ação com F tem alternativa sem F
  (`F3`/`Shift+F3` ⇒ `Ctrl+Alt+G`/`Ctrl+Alt+Shift+G`) e acionamento manual
  por botão (▲▼ na barra).

**Fora do v1 (radar):** histórico de buscas, "buscar na seleção",
multi-cursor a partir dos matches, preservar caixa ao substituir.

### ✅ IMPLEMENTADO (2026-07-12)

Tudo do v1 acima, exatamente como desenhado. Protocolo **segue 0.42.0** (zero
mudança de contrato) — só entraram os dois descriptors de comando.

**Validação (harness headless, `qml6` offscreen, dirigindo o
`EditorFindController` REAL com uma superfície falsa que muta o texto de
verdade):** 11 asserções, todas verdes. As que importam:

| # | Caso | Por que é o que quebra |
|---|---|---|
| 6 | Regex `a*` (largura zero) | Sem o avanço forçado, é **laço infinito** — a IDE trava. O teste falha por TIMEOUT se regredir. |
| 8 | `Substituir tudo` com substituto **maior** ("x"→"LONGO") | É o caso que corrompe offsets se a passada não for de trás pra frente. |
| 10 | Regex com grupo (`foo\((\d)\)` → `bar[$1]`) | Prova que `$1` resolve. |
| 2 | `barVisible` lê `true` | Regressão do **D1**: o controller novo vive sob o mesmo pai invisível. |
| 5 | Navegação circular nos dois sentidos | Wrap pra frente e pra trás. |
| 7 | Regex inválida (`(unclosed`) | Estado de erro, 0 matches, sem crash. |

Gate `scripts/verificar.sh` **verde** + smoke offscreen (exit 124).

**Dois bugs que o design pegou antes de existirem:** (a) **trocar de aba** com
a barra aberta deixaria os offsets do texto ANTIGO apontando pro texto NOVO
(realce e seleção errados) → `selectTab` revarre; (b) **digitar** com a barra
aberta envelhece os matches → debounce de 200ms revarre.

---

## D2 — Terminal → paridade VS Code/JetBrains (design 2026-07-12)

**Estado atual (a raiz do problema):** `crates/kinein-core/src/terminal.rs`
usa o utilitário `script` como PTY falso, com `TERM=dumb` e um
`AnsiSanitizer` que **REMOVE todo o ANSI** → texto puro, sem cor, sem
cursor, sem TUI. A UI (`TerminalPanel.qml`) é **linha-a-linha** (digita
uma linha + Enter). Sessão única. Sem resize. Por isso não parece nem
funciona como VS Code/JetBrains.

**Decisão de arquitetura (e porquês):**

```text
- EMULADOR VT NO CORE (Rust), não na UI. Não existe widget de terminal Qt
  maduro; portar um emulador VT pra C++/QML é caro e frágil. Usa-se a lib
  madura `vt100` (parser + grid de células com cor/atributos + cursor +
  scrollback + alternate screen p/ TUIs). A UI vira um RENDERER BURRO de
  grade. Alinhado ao mandato (adotar tech madura DIRETO no core).
- PTY REAL via `portable-pty` (do wezterm — equivalente ao node-pty do VS
  Code): openpty(PtySize) → spawn do $SHELL no slave, TERM=xterm-256color,
  cwd na raiz. Thread lê bytes crus do master e faz parser.process(bytes).
- ENVIAR O GRID, não bytes crus. A UI recebe CÉLULAS PRONTAS (não precisa
  interpretar ANSI). Robusto p/ prompt com \r, barra de progresso do
  cargo, e TUIs. Formato por LINHA como lista de SPANS (runs de células de
  mesmo estilo — compacto).
- INPUT caractere-a-caractere: a UI captura Key events e traduz p/ bytes do
  terminal (incl. control chars: Ctrl+C=\x03, setas=\x1b[A/B/C/D, Tab, etc.)
  e manda `terminal.input`. Fim do input linha-a-linha.
- RESIZE: a UI calcula cols/rows a partir do tamanho do painel ÷ métrica da
  fonte mono e manda `terminal.resize`; o core faz pty.resize + parser
  .set_size.
- THROTTLE do render: emitir no máx ~a cada 16–33ms quando há mudança
  (senão flood de JSON). v1 manda a tela visível inteira; diffs depois.
```

**Contrato (protocolo 0.41.0):**

```text
terminal.open {}                 → { id, shell }   (id p/ multi-sessão)
terminal.input { id, data }      → ok              (bytes/keys crus)
terminal.resize { id, cols, rows } → ok            (reflui uma sessão)
terminal.scroll { id, offset }   → ok              (scrollback de uma sessão)
terminal.close { id }            → ok              (fecha uma sessão)
event.terminal.render {          (NOVO — substitui event.terminal.data)
  id,
  cols, rows,
  cursor: { row, col, visible },
  lines: [ [ { text, fg?, bg?, bold?, italic?, underline?, inverse? } ... ] ]
}
event.terminal.closed { id, exitCode }
```

`fg`/`bg` como índice 0–255 (paleta) ou `#rrggbb` (truecolor); ausência =
cor default do tema.

**Faseamento:**
- **D2.1 (fundação, esta fatia):** portable-pty + vt100 + `terminal.render`
  estruturado + `terminal.resize` + input char-level + TERM=xterm-256color
  + renderer de grade na UI. UMA sessão (como hoje), mas agora com CORES,
  cursor, TUIs, resize e input real. Já é o salto "profissional".
- **D2.2:** seleção + copiar/colar e scrollback UI polido.
- **D2.3:** múltiplos terminais em abas; ids ponta a ponta no protocolo.

**[D2.1 FEITA] em 2026-07-12, protocolo 0.41.0.** Core (`terminal.rs`
reescrito): PTY real via `portable-pty`, `TERM=xterm-256color`, `$SHELL`
na raiz; emulador `vt100` (3 threads: leitora → `parser.process`;
emissora throttled ~30fps; waiter → `closed`). Emite `event.terminal.
render` (grid de spans + cursor); `terminal.resize` reflui PTY + parser.
UI (`TerminalPanel.qml` reescrito): renderer de grade (spans com cor 256/
truecolor via `Theme.terminalPalette`, bg por célula, cursor), captura de
teclado CHAR-A-CHAR (Enter=`\r`, Backspace=`\x7f`, setas/Home/End/PgUp/Del,
Ctrl+letra, texto) → `terminal.input` bytes crus; calcula cols/rows do
tamanho ÷ métrica mono e manda `terminal.resize`. Fiação:
CoreClient(`terminalRender`/`terminalResize`) → RuntimeController → Shell/
BottomPanelHost → TerminalPanel. Validação: gate/clippy/qmllint/smoke
verdes; unit test (grid recebe o echo); sonda e2e `sonda_terminal.py`
(comando aparece no grid; resize 100×30 reflete). Ajuste de lint:
`multiple_crate_versions` allow (bitflags 1.x transitivo do portable-pty).

**[D2.2 FEITA] em 2026-07-12, protocolo 0.42.0.** Copiar/colar + scrollback.
Core: `terminal.scroll { offset }` → `vt100 set_scrollback` + emit render.
UI: **colar** (Ctrl+Shift+V / clique-do-meio via singleton C++ `Clipboard`),
**scrollback** (roda do mouse → `scroll`; snap-to-bottom ao digitar; cursor
some quando rolado), **copiar** (seleção linear com o mouse — realce em até
3 retângulos, Ctrl+Shift+C extrai o texto dos spans → `Clipboard`).
Validação: gate/clippy/qmllint verdes; sonda `sonda_terminal.py` (comando
no grid, resize 100×30, **scroll traz o início do histórico**); copiar/colar
verificado por build (precisa de GUI+clipboard pra e2e). Nota: bin do core
precisa de `cargo build` fresco antes das sondas (armadilha conhecida).

**[D2.3 FEITA] em 2026-07-14, protocolo 0.44.0.** Cada `terminal.open` cria
uma sessão com id monotônico; `input`/`resize`/`scroll`/`close` e os eventos
`render`/`closed` carregam o id. O `TerminalManager` guarda até 12 sessões num
`HashMap`; cada aba preserva seu próprio grid/scrollback, e fechar uma não
derruba as demais. A UI ganhou chips `Terminal N`, botão `+`, fechamento por
aba, troca com restauração do último render e limpeza segura após crash do
core/workspace. Durante a validação foi corrigido um deadlock antigo:
`wait()` segurava o mutex do child e impedia `close()` de chamar `kill()`;
agora o waiter é dono do child e a sessão usa `ChildKiller::clone_killer()`.

**Provas:** 215 testes do core; teste unitário com duas sessões coexistindo;
`scripts/sonda_scrollback.py` contra o core real (grids isolados + fechar t2
preserva t1); `tst_multi_terminal.qml` headless (troca, fechamento, render
atrasado e limpeza após crash); clippy/qmllint verdes. Falta apenas validar
os gestos das abas ao vivo.

**Fora (conforto):** links clicáveis (URLs/paths), busca no scrollback e
refinos da seleção (arrastar além da borda, duplo-clique = palavra).

### ✅ B1 — "a roda do mouse não rola o terminal" (RESOLVIDO 2026-07-12, protocolo 0.43.0)

**Sintoma (usuário):** "uso o scroll do mouse e não percebo / não acontece
essa rolagem".

**A causa NÃO era a fiação** (ela estava correta ponta a ponta) nem o
MouseArea. Era um **bug do `vt100` 0.15.2** que **derrubava o core**:

```rust
// vt100-0.15.2/src/grid.rs:125  — visible_rows()
.chain(self.rows.iter().take(rows_len - self.scrollback_offset))
//                            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ 24 - 278 → OVERFLOW
```

`visible_rows()` monta a tela como *(últimas `offset` linhas do histórico) +
(`rows - offset` linhas vivas)* — então um `offset` MAIOR que a altura da tela
faz uma subtração negativa em `usize`. Em **debug** isso é `panic` (o core
morre); em **release** (o binário do ícone) a subtração dá a volta e o
emulador monta uma tela errada.

**E a UI alimentava exatamente esse caso:** o `onWheel` somava sem teto —
`Math.max(0, scrollOffset + step)` — sem NENHUM limite superior. Com passo de
3 linhas e tela de ~24, bastavam **~9 cliques de roda** para o offset passar da
altura da tela. O core caía, a **recuperação de crash da M4.3 o relançava
calada**, a sessão de terminal ia junto — e o usuário via "não acontece nada".

Isso fecha por que as sondas passavam: elas testavam offsets pequenos e o
caminho do core, nunca o gesto real repetido.

**O fix (três camadas, fix-first):**
1. **`vt100` 0.15 → 0.16.2.** O upstream corrigiu exatamente isso (o código
   deles agora satura a subtração; o comentário no fonte diz "it'll panic with
   overflow"). Bônus grande: no 0.15 o scrollback era inalcançável além de UMA
   tela; agora o histórico inteiro (5000 linhas) é navegável de verdade.
   API mudou: `set_scrollback`/`set_size` saíram do `Parser` para o `Screen`.
2. **O core passou a dizer a verdade** (protocolo **0.43.0**): o
   `event.terminal.render` agora carrega `scrollback` (offset atual, já
   clampado) e `scrollbackMax` (quanto histórico existe). O `vt100` não expõe o
   total, então o core o obtém pedindo `set_scrollback(usize::MAX)`, lendo
   quanto foi clampado e restaurando — sem efeito observável.
3. **A UI parou de chutar:** o wheel virou um `WheelHandler` no root do painel
   (o `MouseArea.onWheel` perde a roda quando outro item segura o grab do
   ponteiro), com **clamp em `scrollbackMax`**, e o `scrollOffset` local é
   **reconciliado** pelo `scrollback` do render — o core é a fonte da verdade.

**Prova:** `scripts/sonda_scrollback.py` (e2e, core real). Antes: offset
absurdo → core em silêncio + `panicked at grid.rs:125: attempt to subtract with
overflow` no stderr. Depois: `scrollbackMax=278`, o offset ecoa a verdade, a
tela muda ao rolar, e offset 9999 **clampa em 278 sem matar o core**.

**Lição (a mesma do D1, de novo):** sonda de core verde ≠ gesto do usuário
funcionando. O que fechou o caso foi a sonda **exercitar o caso extremo** que o
gesto real produz, não o caso feliz.

### ✅ B2 — não havia barra de rolagem em lugar NENHUM (RESOLVIDO 2026-07-12)

Varredura confirmou: **`ScrollBar`/`ScrollView` não apareciam uma única vez em
todo o `ui/qml`**. O editor era `TextEdit` (`NoWrap`) dentro de um `Flickable`
sem indicador; os painéis inferiores rolam por `ListView` sem indicador; o
terminal rolava sem indicador. Sem barra o usuário não sabe que há conteúdo
fora da tela, onde está no arquivo, nem **se uma rolagem aconteceu** — foi o
que tornou o B1 invisível por tanto tempo.

**Feito:** `ui/qml/shell/VerticalScrollBar.qml` — barra própria (o projeto não
usa `QtQuick.Controls`, mesma decisão do tooltip e do SettingsDialog),
**genérica na unidade**: serve pra PIXELS (o `Flickable` do editor) e pra
LINHAS (o terminal, onde a barra é SINTÉTICA — a posição vem do scrollback do
emulador, não de um `contentY`; o eixo é invertido, porque `scrollOffset` conta
do fundo e a barra conta do topo). Ela nunca muda a posição sozinha: **pede**
por `moveRequested` e o dono aplica, mantendo uma fonte da verdade só.
Aplicada no **editor**, no **terminal** e nos `ListView` dos painéis inferiores
(Build, Testes, Problemas, Git, Jobs, IDE, Busca, Run, Tools e saída de Debug).
O mesmo componente foi reaproveitado; não nasceu uma segunda barra.

**Fora (com gatilho):** shell integration (cwd/exit por comando via OSC
133), imagens (sixel).

**Arquivos (D2.1):** Cargo (portable-pty, vt100 — já adicionados);
`terminal.rs` (reescrever: PTY + vt100 grid + emitir render + resize +
multi-id opcional); protocolo (`TerminalResizeParams`, tipos de render,
0.41.0); `handlers/terminal.rs` (resize); UI `TerminalPanel.qml` (renderer
de grade + captura de teclado → bytes); router; MANUAL.

---

## T2 — Mudanças externas sem perda de dados (feito em 2026-07-14)

O complemento semântico (`KINEIN_VECTIS_DEEP_SEMANTIC_ENGINE_CPP_RUST_WORKFLOW.md`)
confirma que versões de documento e workspace edits conservadores são parte da
fundação, não um detalhe visual. A primeira fatia entrou no protocolo `0.45.0`:

- `notify 8.2.0` integra o backend maduro do SO (`inotify` no Linux), com
  polling como fallback; adoção Modo A auditada no registro/ADR;
- observação lazy e não recursiva da raiz + diretórios que a árvore/editor
  realmente alcançaram; caches/builds e temporários são filtrados;
- `event.fs.changed` é tipado, deduplicado e debounced (180 ms); árvore, Git e
  editor consomem o mesmo lote por routers de domínio;
- aba limpa recarrega; aba suja mantém o buffer e exige decisão explícita;
- `fs.write` exige `expectedContent`. Snapshot divergente retorna
  `FILE_CHANGED` sem tocar o disco; save aceito continua atômico.

Isso fecha o vetor de `git checkout`/formatter/editor externo sobrescreverem
silenciosamente um buffer velho. O passo semântico posterior é levar o mesmo
modelo (versão, preview, rollback) aos workspace edits multi-arquivo do LSP,
junto da fundação D3 — não duplicar essa transação na UI.

---

## D3 — Tree-sitter + arquitetura de plugins + views em árvore

**Conceito (alinhado ao mandato + roadmap de adoção):** "plugin" aqui
**NÃO é extension host** (Neovim/VSCode). É **integrar biblioteca/
ferramenta open-source madura DIRETO no core** (Modo A: linkar a lib;
Modo B: orquestrar o binário). Escopo de linguagem: **C/C++/Rust**.

**Arquitetura proposta:**
- `crates/kinein-core/src/lang/` — **registry por linguagem**:
  `language → { grammar tree-sitter, queries (highlights/folds/locals/
  outline), spec do LSP existente }`.
- **tree-sitter** via crate `tree-sitter` + grammars vendorizadas
  (`tree-sitter-c`, `-cpp`, `-rust`).
- Capacidades como módulos **independentes do LSP**: highlighting
  (tokens robustos), folding (ranges), outline (símbolos). O LSP segue
  para a SEMÂNTICA (go-to-def, diagnostics, completion).
- Protocolo: tokens do tree-sitter entram no mesmo canal dos semantic
  tokens (a UI consome como já faz) — ou um `lang.tokens` dedicado.
- **Views em árvore:** explorer de arquivos (polir) + **outline/símbolos**
  (árvore de estrutura do arquivo, alimentada por tree-sitter/LSP). O
  usuário está fazendo os **ícones próprios** desta parte.

### ✅ D3 implementada (2026-07-14, protocolo 0.46.0)

O design vinculante está em `docs/25-syntax-tree-semantic-foundation.md` e a
implementação segue o Modo A do roadmap de componentes abertos:

- `tree-sitter`, `tree-sitter-c`, `tree-sitter-cpp` e `tree-sitter-rust`
  com versões exatas, licença/pin auditados no registro e ADR;
- `lang/` contém registry, parser incremental, conversão UTF-8→UTF-16,
  highlights, folding, outline aninhado e locals sintáticos;
- cache LRU limitado a 32 documentos, buffer máximo de 4 MiB, reset por
  workspace e versão ponta a ponta para stale-drop;
- o editor compõe regex < Tree-sitter < semantic tokens LSP < decorações;
  folding e outline funcionam sem LSP e não tentam resolver tipos/referências;
- `syntaxTree.update` é assíncrono na UI, com debounce; não há parser QML/C++
  paralelo.

Na mesma fundação, rename/code actions deixaram de gravar diretamente: o
protocolo 0.47.0 cria preview confirmável, compara snapshots e usa transação
multi-arquivo com rollback (`fsops/transaction.rs`) compartilhada com
`fs.replace`. Esse é o recorte imediatamente acionável do KSWE descrito no
complemento semântico; grafo de targets/contextos e scheduler multi-LSP ficam
para uma fase própria, sem inflar o D3.

---

## D4 — Remake de UI/UX

Convergência da UI atual para `docs/specs` conforme o plano vinculante
`docs/20` (fatias C0–C6). Por último: é o maior esforço e o dogfooding +
D1–D3 informam o que priorizar. Régua: JetBrains (não "virar um VS Code").

### ✅ Implementação material concluída (2026-07-14)

- C2: `KvIcon` vetorial central (grid 24×24/stroke 1.75) e componentes
  acessíveis `KvIconButton`, `KvButton`, `KvTooltip`; glifos de ação residuais
  removidos das superfícies QML.
- C3: Main Toolbar 44px com target/perfil/configure/build/test/quality/run/
  debug, ligada aos controllers existentes.
- C5: Title/App Bar 40px com menus completos em overlay global e ações ligadas;
  Start Screen com detecção visível
  e criação C++/Rust; preview de arquivos/comandos; template C++23 moderno,
  target-based e presets Debug/Release; KV Context como ponte explícita para
  Claude/Codex instalados pelo usuário sobre o PTY existente; diálogo Sobre.
- Remediação 0.50: tooltips em overlay superior, criar arquivo/pasta pelo menu
  e clique direito, dimensões responsivas/persistidas e Estrutura
  redimensionável/recolhível para aba direita.
- Nenhuma lógica de negócio ou filesystem foi movida para a UI: Start Screen
  reutiliza `FolderPickerDialog`, `tools.detect` e `workspace.createProject`.

C6 fica objetivamente limitada à validação visual R7 do usuário em tela real
e à auditoria visual final. Os gates automatizados, qmllint estrito e smoke
offscreen não substituem esse aceite humano.

## Lacunas daily-driver T3–T6 fechadas junto da fundação

- T3 Git: branches/checkout/create, pull/push como jobs e stash push/pop
  protegido contra buffers sujos; `.kinein` excluído do stash.
- T4 `fs.replace`: substituição literal de projeto com confirmação, limites,
  ignores, compare-before-save e rollback multi-arquivo.
- T5 Salvar tudo (`Ctrl+Shift+S`): fila determinística, inclusive
  format-on-save, sem aceitar respostas obsoletas.
- T6 arquivos recentes (`Ctrl+E`): MRU reutiliza Search Everywhere, sem modelo
  de busca duplicado.

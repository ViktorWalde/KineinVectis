# `terminal.*` — o domínio no protocolo IPC

> Parte do [03](../03-protocolo-ipc.md), que tem o formato base, as listas
> canônicas de métodos e eventos e as regras. Movido sem mudar o texto em
> 2026-10-16 ([61](../../roadmaps/61-organizacao-da-documentacao.md) §3.7, O6).

### Terminal (`terminal.open` / `terminal.input` / `terminal.resize` / `terminal.scroll` / `terminal.mouse` / `terminal.clearScrollback` / `terminal.selectAll` / `terminal.copySelection` / `terminal.close`)

Terminal profissional (reescrito no protocolo `0.41.0`, fatia D2 de
`DocsPublic/roadmaps/24`). Requer workspace aberto. **PTY real** via `portable-pty` (do
wezterm) rodando o `$SHELL` interativo em `TERM=xterm-256color` na raiz.
Um **emulador VT** (`alacritty_terminal`, ADR-0004 — o mesmo motor do Alacritty
e do Zed) no core mantém o GRID (células com cor/atributos), cursor, modos, tela
alternada e scrollback; a UI recebe o grid PRONTO e só desenha — sem interpretar
ANSI. Suporta cores, prompts com `\r`, barra de progresso do cargo e TUIs. Desde
o protocolo `0.44.0` (D2.3), o manager mantém até 12 sessões simultâneas. Cada
`open` cria um id monotônico (`t1`, `t2`, …); todos os comandos e eventos
seguintes carregam esse id.

**Invariante de camada.** O terminal é dono da semântica do input. A UI reporta
gestos crus (tecla, roda, clique) e desenha o grid que recebe; ela **não** decide
o que um gesto significa, porque isso depende do modo VT que só o emulador
conhece. Nenhuma política por programa existe no core: uma CLI de IA recebe o
mesmo tratamento de um `ls`.

- `terminal.open { cwd? }` → `{ id, shell }`. Cada chamada cria uma sessão
  nova; sem `cwd` começa na raiz. Desde `0.140.0`, `cwd` precisa ser diretório
  absoluto existente dentro do workspace. Erro `INVALID_REQUEST` ao atingir
  o limite de 12 sessões.
- `terminal.input { id, data }` → `{ status: "ok" }`. Encaminha `data` **cru**
  ao PTY. A UI manda CADA tecla (char-a-char), incl. control chars
  (Enter=`\r`, Backspace=`\x7f`, setas=`\x1b[A..D`, Ctrl+letra, …) — não
  linha+Enter.
- `terminal.resize { id, cols, rows }` → `{ status: "ok" }`. Reflui o PTY e o
  grid. A UI calcula cols/rows do tamanho do painel ÷ métrica da fonte mono.
- `terminal.scroll { id, offset }` → `{ status: "ok" }` (D2.2, `0.42.0`).
  Rolagem **explícita** do histórico: `offset` linhas acima do fundo (0 = ao
  vivo; o core **clampa** ao tamanho real do scrollback). É o que a barra de
  rolagem pede, e o snap-to-bottom ao digitar. **Um gesto de roda não é isto** —
  vai por `terminal.mouse`, porque só o core sabe se a aplicação capturou o
  mouse. O clipboard pertence à UI (singleton `Clipboard`); desde `0.131.0`,
  a cópia da seleção completa lê o texto do emulador por RPC sob demanda.
  **Nunca confie no offset da UI:** até `0.43.0` um offset maior que o
  histórico **derrubava o core** (bug de overflow do `vt100` 0.15 —
  corrigido no 0.16, que satura a subtração; o `alacritty_terminal` clampa por
  conta própria). A verdade do offset volta no render (`scrollback`), não na
  resposta.
- `terminal.clearScrollback { id }` → `{ status: "ok" }` (`0.130.0`). Apaga
  o histórico mantido pelo emulador **só da sessão indicada**, retorna a tela
  ao fundo e emite um render novo (`scrollback = 0`, `scrollbackMax = 0`). Não
  injeta comandos no shell e não apaga as linhas que ainda estão no grid
  visível. “Limpar tela” é outra ação: a UI envia `Ctrl+L` (`\x0c`) por
  `terminal.input`, preservando a semântica do programa em primeiro plano.
- `terminal.selectAll { id, selectionId }` → `{ id, selectionId }`
  (`0.131.0`). Seleciona o buffer ativo retido, incluindo histórico e linhas
  utilizadas da tela, independente da rolagem. Não inclui tela inativa,
  outras sessões, texto descartado nem linhas não utilizadas abaixo do prompt.
  `selectionId` é uma identidade opaca do gesto, não vazia, até 128 bytes.
  Publica a identidade no render sob o lock do buffer; não lê clipboard.
- `terminal.copySelection { id, selectionId }` → `{ id, selectionId, text }`
  (`0.131.0`). Extrai a seleção nativa, preservando Unicode e distinguindo
  wrap visual de quebra real. `text: null` significa identidade obsoleta;
  string vazia significa seleção válida sem texto. Nova saída, resize,
  limpeza e troca de tela invalidam; scroll preserva. Sessão inexistente
  devolve `INVALID_REQUEST`. A UI descarta respostas de outro gesto/sessão
  e não registra o texto copiado no log.
- `terminal.mouse { id, col, row, event, modifiers? }` → `{ status: "ok" }`
  (R4, `0.60.0`). Um gesto de mouse na grade. `col`/`row` são a célula sob o
  ponteiro, **0-based** (mesma origem do cursor no render); o core converte para
  1-based ao montar o relatório, como o xterm especifica. `modifiers` é
  `{ shift?, alt?, ctrl? }`, todos `false` por omissão.

  `event` é marcado por `kind`:

  ```text
  { "kind": "wheel",   "lines": i16 }          implementado
  { "kind": "press",   "button": "left"|"middle"|"right" }   R5 — recusado
  { "kind": "release", "button": ... }                        R5 — recusado
  { "kind": "motion",  "button": ...|null }                   R5 — recusado
  ```

  O contrato de R5 está fixado para a superfície não mudar de novo; o core
  recusa essas variantes com `INVALID_REQUEST` em vez de fingir que funcionam.

  **A decisão é do core**, lendo o modo VT (ordem vinda do `scroll_wheel` do Zed,
  referência MODE-D):

  ```text
  1. shift ligado                      → rola o histórico local
                                         (válvula de escape do xterm)
  2. aplicação capturou o mouse        → relatório à aplicação
     (MOUSE_REPORT_CLICK/DRAG/MOTION)    SGR se ?1006, senão formato legado
  3. ALT_SCREEN + ALTERNATE_SCROLL     → cursor keys (ESC O A / ESC O B)
  4. caso contrário                    → rola o histórico local
  ```

  A ordem entre 2 e 3 é carga estrutural: `ALTERNATE_SCROLL` nasce **ligado**
  (default do emulador, como no xterm), então uma TUI que captura o mouse
  satisfaz os dois. Quem captura tem precedência — inverter faz a aplicação
  receber setas e não rolar.

  `lines` positivo = para cima (histórico mais antigo); negativo = para baixo;
  `0` é no-op. Um relatório é emitido por linha do gesto. No formato legado uma
  coordenada acima de 223 não é representável e o evento é **suprimido**, não
  truncado (um campo truncado viraria clique em outra célula).
- `terminal.close { id }` → `{ status: "ok" }`. Fecha só a sessão indicada;
  fechar/trocar o workspace ou encerrar o core fecha todas.

```text
event.terminal.render {           (throttle ~30fps; substitui event.terminal.data)
  "id": string,                  (0.44.0 — sessão dona deste grid)
  "selectionId": string,         (0.131.0 — vazio se não há seleção completa válida)
  "cols": u16, "rows": u16,
  "cursor": { "row": u16, "col": u16, "visible": bool,
              "shape": "block"|"underline"|"bar",
              "blinking": bool },       (0.57.0 — DECSCUSR da aplicação)
  "alternateScreen": bool,       (0.51.0 — TUI em tela alternativa)
  "applicationCursor": bool,     (0.51.0 — setas SS3 quando solicitado)
  "bracketedPaste": bool,        (0.51.0 — paste delimitado e seguro)
  "scrollback": usize,            (0.43.0 — offset ATUAL, já clampado: a verdade)
  "scrollbackMax": usize,         (0.43.0 — quanto histórico existe; 0 = nenhum)
  "lines": [ [ { "text": str, "cells": u16,
                 "fg"?: idx|"#rrggbb", "bg"?: idx|"#rrggbb",
                 "bold"?: bool, "italic"?: bool, "underline"?: bool,
                 "inverse"?: bool } ... ] ... ]
}
event.terminal.closed  { "id": string, "exitCode": int|null }
```

`fg`/`bg`: índice 0–255 (paleta) ou `#rrggbb` (truecolor); ausência = cor
default do tema. Cada linha é uma lista de SPANS (runs de células de mesmo
estilo). Desde `0.56.0`, `cells` informa a largura autoritativa do span em
colunas VT; ela não deve ser inferida de `text.length` nem da largura em pixels
da fonte, pois glifos largos, combinantes e fallback tipográfico podem divergir.
O core coalesce runs e descarta o espaço final em estilo default.
Desde `0.57.0`, `shape` e `blinking` preservam o estado VT solicitado por
`DECSCUSR` (`CSI Ps SP q`). O core resolve reset/`DefaultUserShape` para a
preferência do terminal Kinein (`bar`) e sempre emite uma forma concreta; forma
explícita e piscagem pertencem ao programa no PTY. A UI não deve inferir a CLI
ativa nem aplicar geometria específica para Claude/Codex.

`scrollback`/`scrollbackMax` (`0.43.0`, B1/B2 de DocsPublic/roadmaps/24) existem porque a UI
**não tem como saber sozinha** se há histórico nem onde a view está: o core é
quem clampa. Sem eles não dá pra desenhar barra de rolagem honesta, e a UI
acabava pedindo offsets impossíveis. A UI trata `scrollback` como fonte da
verdade (reconcilia o estado local a cada render).

Desde `0.51.0`, o render também expõe os modos VT que alteram a tradução de
entrada. A UI respeita application cursor, envolve colagens com bracketed
paste quando a aplicação o pede e continua sem interpretar a interface do
programa. Resize de painel é coalescido antes de `terminal.resize`, evitando
reserializar um grid por pixel durante o arrasto.

# ADR-0004 — Adotar `alacritty_terminal` como motor de emulação VT

> **Status:** **implementado**; aguarda o gesto visual do autor com Claude/Codex
> reais. O motor foi trocado em `crates/kinein-core/src/terminal.rs`, o contrato
> `event.terminal.render` não mudou, e `vt100`/`vte` saíram das dependências.
> **Data:** 2026-07-16.

## Contexto

O terminal integrado é requisito central do produto: o autor pediu, desde o
início, "meu terminal inteiro dentro da IDE", com os agentes de CLI (Claude,
Codex) funcionando como funcionam fora dela.

O sintoma reportado — cursor "flutuando", scroll que não navega o histórico e a
sensação de que a UI da IDE interfere ao rodar `claude`/`codex` — foi
investigado até a causa-raiz:

1. **O ambiente entregue ao processo está correto.** O core já exporta
   `TERM=xterm-256color`, `COLORTERM=truecolor` e `TERM_PROGRAM=KineinVectis`,
   idênticos ao terminal do sistema do autor. Descartado como causa.
2. **As mesmas CLIs funcionam no terminal do sistema e falham na Kinein.** Mesma
   máquina, mesmo binário, mesma fonte. Logo a variável independente é o
   emulador da IDE, não a TUI do agente.
3. **O emulador é raso.** O core usa a crate `vt100`, um emulador VT
   deliberadamente simples. A Kinein **anuncia** `xterm-256color` e o aplicativo
   emite tudo que um xterm-256color aceita, mas recebe um subconjunto. Tela
   alternada, protocolos de mouse, reflow e o ciclo de cursor de aplicações TUI
   ficam sem suporte real.

Evidência complementar (`docs/roadmaps/26` §4.4): no modo de transcript plano o
cursor é correto e o scroll funciona; só a TUI decorativa quebra. Isso enfraquece
as hipóteses de métrica de fonte/baseline (quebrariam nos dois modos) e reforça
que o problema está na profundidade do emulador.

Correções anteriores trataram o sintoma no agente (`--ax-screen-reader` no perfil
Claude). Isso é paliativo: adapta a ferramenta ao emulador incompleto em vez de
corrigir o emulador.

## Decisão

Substituir `vt100` por [`alacritty_terminal`](https://github.com/alacritty/alacritty)
`0.26.0` como motor de grade e estado VT do core, preservando `portable-pty`, o
contrato IPC tipado e o renderer Qt/QML.

Alinhamento com a política do repositório (`docs/roadmaps/KINEIN_VECTIS_OPEN_PLUGIN_ADAPTATION_ROADMAP.md`):
é **MODE-A** — integrar biblioteca aberta madura em vez de reimplementar. As
referências do próprio roadmap resolveram o mesmo problema do mesmo jeito:
Code OSS usa xterm.js, IntelliJ usa JediTerm e **Zed usa `alacritty_terminal`**.

### Por que este componente

- **Rust puro**, encaixa direto no core; sem Node, Electron, WebView ou FFI.
- **Apache-2.0**, compatível com o duplo MIT/Apache-2.0 do projeto.
- **Mesmo ecossistema do `vte` 0.15 que o core já linka** (o `alacritty_terminal`
  reexporta `vte`), então o parser ANSI passa a ser um só.
- Emulador de um terminal em produção há anos: tela alternada, mouse, reflow,
  wide chars, cores 16/256/truecolor e DECSCUSR são suportados de fato.
- Substitui código nosso: o rastreador de DECSCUSR escrito à mão sobre `vte`
  (`CursorStyleTracker`) deixa de existir — `Term::cursor_style()` cobre isso.

### Custo aceito

`alacritty_terminal` embute os módulos `tty`, `event_loop` e `thread`, que a
Kinein **não** usa (o PTY continua sendo `portable-pty`). Não há feature para
removê-los (a única feature é `serde`). Isso adiciona ~15 dependências
transitivas (`rustix`, `rustix-openpty`, `polling`, `piper`, `signal-hook`,
`miow`, `home`, `base64`, `windows-sys` …) e ~322 linhas de `Cargo.lock`.

O custo é aceito porque a alternativa é manter/estender um emulador VT próprio —
exatamente o que a filosofia do projeto proíbe ("orquestrar ferramentas maduras,
não reimplementar"). `windows-sys`/`miow` são deps de plataforma e não entram no
alvo Linux.

## Mapeamento de API (levantado do fonte da crate, não de memória)

Fonte auditado: `~/.cargo/registry/src/index.crates.io-*/alacritty_terminal-0.26.0/src`.

| Hoje (`vt100`) | Equivalente (`alacritty_terminal`) |
| --- | --- |
| `Parser::new(rows, cols, scrollback)` | `Term::new(Config { scrolling_history, .. }, &dims, VoidListener)` |
| `parser.process(bytes)` | `Processor::advance(&mut term, bytes)` — `impl<T: EventListener> Handler for Term<T>` |
| `parser.screen()` | `term.grid()` → `Grid<Cell>` |
| `screen.size()` | `Dimensions::{screen_lines, columns}` |
| `screen.cell(row, col)` | `grid[Line(row as i32 - display_offset as i32)][Column(col)]` |
| `screen.cursor_position()` | `grid.cursor.point` (`Line`,`Column`); linha visível = `point.line.0 + display_offset` |
| `screen.hide_cursor()` | `!term.mode().contains(TermMode::SHOW_CURSOR)` |
| `screen.alternate_screen()` | `term.mode().contains(TermMode::ALT_SCREEN)` |
| `screen.application_cursor()` | `term.mode().contains(TermMode::APP_CURSOR)` |
| `screen.bracketed_paste()` | `term.mode().contains(TermMode::BRACKETED_PASTE)` |
| `screen.scrollback()` | `grid.display_offset()` |
| `scrollback_capacity(parser)` (hack de ida e volta) | `grid.total_lines() - grid.screen_lines()` |
| `screen_mut().set_scrollback(n)` | `term.scroll_display(Scroll::Delta(delta))` (relativo) |
| `screen_mut().set_size(rows, cols)` | `term.resize(dims)` |
| `CursorStyleTracker` próprio sobre `vte` | `term.cursor_style()` → `CursorStyle { shape, blinking }` |
| `cell.contents()/is_wide()/is_wide_continuation()` | `cell.c` + `Flags::{WIDE_CHAR, WIDE_CHAR_SPACER}` |
| `cell.bold()/italic()/underline()/inverse()` | `Flags::{BOLD, ITALIC, UNDERLINE, INVERSE, HIDDEN}` |
| `vt100::Color` | `vte::ansi::Color::{Named(NamedColor), Spec(Rgb), Indexed(u8)}` |

Fatos de coordenada confirmados no fonte:

```text
Line(0) = topo do viewport quando display_offset == 0; Line negativa = scrollback.
Linha visível r (0..screen_lines) → Line(r as i32 - display_offset as i32).
grid.display_iter() percorre exatamente a região visível já com o offset.
`TermSize` existe, mas vive em `term::test` (helper); implementar um
`Dimensions` próprio no core em vez de depender do helper de teste.
`VoidListener` (em `event.rs`) é o listener nulo — a Kinein faz poll do grid,
não precisa de eventos.
```

## Plano de migração (ordem obrigatória)

1. `TerminalState` passa a conter `Term<VoidListener>` + `Processor`, e um
   `Dimensions` próprio; remover `CursorStyleTracker` e o uso direto de `vte`.
2. `emit_render` remapeado preservando **o contrato atual byte a byte**:
   `{ id, cols, rows, cursor{row,col,visible,shape,blinking}, alternateScreen,
   applicationCursor, bracketedPaste, scrollback, scrollbackMax, lines }`.
3. `build_line`: manter a semântica de spans (runs de mesmo estilo, `cells` com
   a largura VT autoritativa, isolamento de wide chars). Mapear `vte::ansi::Color`
   para a mesma representação JSON que a UI já consome — **não** mudar o formato
   sem mudar a UI junto.
4. `scroll(offset)` do contrato é **absoluto**; `scroll_display` é **relativo**.
   Converter: `delta = offset_desejado - display_offset_atual`, clampando ao
   histórico. Preservar o eco da verdade (`scrollback` no render).
5. `resize` via `term.resize(dims)` mantendo o `master.resize` do PTY.
6. Testes: os existentes de `terminal.rs` + `tests/ai.rs` + a sonda
   `scripts/sonda_scrollback.py` devem passar sem alteração de contrato. O teste
   `ai_scrollback_filter_survives_chunk_boundaries` (filtro de `CSI 3 J`) deve
   ser reavaliado: com um emulador completo, talvez o filtro deixe de ser
   necessário — decidir por evidência, não por suposição.
7. Rodar o gate integral. Atenção: `verificar-qml.sh` usa response file do build
   `linux-clang-debug-strict` (rebuildar antes de lintar) e `verificar.sh | tail`
   mascara o exit code — conferir o texto "✗ FALHOU".

## Consequências

- **Esperado:** cursor, tela alternada, mouse e scroll de TUIs passam a se
  comportar como no terminal do sistema, porque o emulador passa a ser um de
  verdade. O `--ax-screen-reader` do perfil Claude deixa de ser necessário e o
  toggle `aiCliFlatTranscript` vira preferência estética, não muleta.
- **Reordena o R3 de `docs/roadmaps/26`:** a decisão de *emulador* precede a de
  *renderer*. Não investigar métrica de fonte antes desta troca.
- **Risco:** o contrato de render é consumido pela UI e por harnesses; qualquer
  mudança de formato quebra a tela. Mitigação: preservar o JSON e provar com os
  testes/sonda existentes antes de tocar na UI.

## Rollback

A troca é local a `crates/kinein-core/src/terminal.rs` e ao `Cargo.toml`. Voltar
para `vt100` é reverter esses arquivos; o contrato IPC não muda, então UI,
harnesses e sonda continuam válidos nos dois caminhos.

## Resultado da implementação (2026-07-16)

O contrato `event.terminal.render` saiu **idêntico**; a UI, os harnesses e a
sonda não precisaram de mudança alguma. Provas:

```text
261 testes do core .................. verdes (11 do terminal)
clippy pedantic -D warnings ......... limpo
sonda_scrollback.py (e2e, stdio) .... tudo verde
    scrollbackMax=278, eco do offset=20, clamp 9999→278, multi-sessão
```

Código que a troca **eliminou**: o `CursorStyleTracker` escrito à mão sobre
`vte` (~75 linhas de máquina de estado DECSCUSR) e o `scrollback_capacity`, um
hack de ida e volta que existia só porque o `vt100` não expunha o total do
histórico. Ambos viraram chamada direta ao emulador (`term.cursor_style()` e
`grid.history_size()`).

Ajustes de strict mode registrados: `named_color_index` funde as variantes
`Dim*` na cor base (clippy `match_same_arms`) e `cell_style` é `const fn`.

## Pendências antes de fechar o ADR

- [x] Implementar a migração conforme o plano acima.
- [x] Registrar o componente em `docs/tooling/OPEN_COMPONENT_REGISTRY.json`
      (pin, licença, auditoria de telemetria/rede, escopo). A entrada
      `vte-cursor-style-parser` foi substituída: o componente deixou de ser
      usado diretamente.
- [ ] **Gesto visual do autor** com Claude/Codex reais no KV Context — é o
      único critério de aceite que importa para terminal, e o motivo de origem
      deste ADR (cursor "flutuando", scroll e interferência da UI).
- [ ] Reavaliar o filtro `CSI 3 J` (`ScrollbackPreserver`): mantido por ora para
      não misturar mudanças, mas com um emulador completo ele pode ser
      desnecessário. Decidir por evidência.
- [ ] Reavaliar o toggle `aiCliFlatTranscript`: se o emulador resolver a TUI do
      Claude, ele volta a ser só preferência estética.
- [x] Encaminhamento de mouse ao aplicativo (fatia R4): **roda implementada** no
      protocolo `0.60.0` (`terminal.mouse`), decidida no core por `term.mode()`.
      A previsão do ADR se confirmou — o emulador tornou isso barato, porque os
      modos que a decisão precisa (`MOUSE_MODE`, `SGR_MOUSE`, `ALTERNATE_SCROLL`)
      já estavam no `Term` e só não eram lidos. Clique/arrasto/movimento têm o
      contrato fixado e ficam para R5.
- [ ] Avaliar `Scroll::PageUp`/`PageDown`.
- [ ] **O gesto visual do autor segue sendo o critério.** O emulador novo NÃO
      corrigiu o alinhamento do cursor: em 2026-07-16 o autor reportou que
      continua torto, agora nos dois agentes. Isso fecha a hipótese de §4.5 (a
      profundidade do emulador não era a causa do cursor) e devolve o problema ao
      R0–R3 (métrica de célula/baseline/DPR) — agora com um VT de verdade
      embaixo, o que torna a investigação honesta. O mapeamento do cursor no core
      (`grid.cursor.point.line.0 + display_offset`) foi conferido e está correto.

# Roadmap de paridade do terminal e das TUIs

> **Revisão de produto de 2026-09-22:** permanece válida a investigação de
> paridade do terminal comum e das TUIs. A composição “Assistente”, o AI CLI
> Bridge e integrações especiais com ferramentas de IA estão cancelados. Uma
> CLI que o usuário execute é apenas mais um processo no terminal comum.

> **Status:** aberto, explicitamente adiado pelo usuário em 2026-07-15.
> **Próxima retomada:** começar por R0 (reprodução instrumentada), sem novo
> ajuste manual de `y`, sem AppImage e sem afirmar causa-raiz antes das provas.
> **Decisão do usuário:** a experiência funcional do terminal do Code OSS é a
> base de referência para a Kinein, inclusive ao executar Claude e Codex. A
> adaptação deve ser comportamentalmente fiel, mas nativa nas camadas da
> Kinein; não incorporar Electron, Node, xterm.js ou código copiado.

Este documento separa o que já está implementado, o que a validação em tela
real mostrou, as referências oficiais estudadas e a ordem de
investigação/implementação. Deve ser lido junto de
`AGENTS.md`, `DocsPrivate/ContextoIA.md`, `GUIAIA.md`, `DocsPublic/arquitetura/ARCHITECTURE.md`,
`DocsPublic/arquitetura/03-ipc-protocol.md`, `DocsPublic/arquitetura/06-strict-mode.md`, da spec do AI CLI Bridge e
da seção D2 de `DocsPublic/roadmaps/24-paridade-e-fundacao.md`.

## 1. Contrato de produto que não pode regredir

O Assistente não é um chat embutido nem um terminal alternativo. Ele é somente
uma composição visual que mantém uma sessão de IA CLI visível enquanto a aba
Terminal continua livre. Terminal comum, Claude, Codex e futuras CLIs usam a
mesma cadeia:

```text
teclado/clipboard/resize da UI
        ↓
TerminalPanel + TerminalInputController
        ↓ IPC tipado
TerminalManager → PTY real → shell ou CLI escolhida
        ↓ bytes VT
parser/estado VT autoritativo no core
        ↓ event.terminal.render
um único renderer de grade usado por Terminal e Assistente
```

Invariantes obrigatórios:

1. A TUI é soberana sobre conteúdo, posição, forma e piscagem do cursor.
2. A IDE não cria `TextInput`, composer, guia de entrada nem parser de prompt
   sobre Claude/Codex.
3. Atalho, aba, seletor de perfil, foco, largura e maximização do Assistente
   continuam no frontend; não justificam um segundo backend ou renderer.
4. Texto, cursor, seleção, mouse, resize e scroll usam uma única geometria de
   células VT.
5. A UI nunca executa Claude/Codex diretamente; o ciclo do processo continua
   no core.
6. A base desejada é a paridade comportamental do terminal Code OSS/xterm.js,
   adaptada para `portable-pty` + Rust + IPC + Qt, não o transplante do runtime.

## 2. Resultado real do dogfooding em 2026-07-15

### Esperado

Ao abrir Claude ou Codex pelo script/atalho de desenvolvimento, o caret nativo
da TUI deve ocupar a mesma célula da linha de entrada e parecer verticalmente
centralizado como em um terminal externo. A geometria precisa ser idêntica no
Terminal comum e no Assistente.

### Observado

- Em terminal externo, Claude e Codex renderizam normalmente.
- Na Kinein aberta por `scripts/kinein-vectis`, o usuário não percebeu mudança
  relevante após a implementação de DECSCUSR e remoção dos offsets.
- O caret da TUI ainda parece baixo/desalinhado na barra de entrada do
  Claude/Codex.
- O alinhamento horizontal letra a letra, a paleta verde suave e o Terminal
  comum já haviam sido aprovados e não devem ser alterados sem nova evidência.
- Testes com `cursorVerticalOffset: -2`, depois `0`, e por fim a remoção total
  da propriedade não resolveram o sintoma percebido.

Conclusão correta: o suporte a DECSCUSR fechou uma lacuna real de contrato, mas
**não provou ser a causa-raiz do desalinhamento visual**. O problema permanece
aberto. Não registrar a versão atual como aceita e não gerar novo AppImage com
base nesse cursor.

## 3. Snapshot da implementação que deve ser preservado até R0

Estado do worktree no momento deste registro:

- protocolo IPC `0.57.0`;
- `TerminalState` mantém o grid por `vt100` e observa DECSCUSR de modo
  chunk-safe com `vte 0.15`;
- o cursor de `event.terminal.render` contém `row`, `col`, `visible`, `shape` e
  `blinking`;
- `DefaultUserShape` é resolvido no core para `bar`;
- `cursorVerticalOffset` não existe mais em `AssistantPanel.qml`,
  `TerminalPanel.qml` ou `TerminalViewport.qml`;
- `AssistantPanel.qml` instancia diretamente o mesmo `TerminalPanel` usado no
  Terminal comum;
- o texto é hoje composto por `Column → Row → Rectangle → Text`, um `Text` por
  span ANSI;
- `TerminalPanel.qml` mede `"M"` com `TextMetrics`; `advanceWidth` vira
  `charWidth` e `height` vira `lineHeight`;
- cada `Text` preenche a célula e usa `Text.AlignVCenter`;
- o cursor é outro `Rectangle`, posicionado por `row * lineHeight` e
  `col * charWidth`;
- spans, resize, seleção e cursor já usam a largura em células VT, inclusive
  para glifos largos.

Arquivos centrais:

```text
ui/qml/panels/bottom/TerminalPanel.qml
ui/qml/panels/bottom/TerminalViewport.qml
ui/qml/panels/bottom/TerminalInputController.qml
ui/qml/panels/bottom/TerminalSelectionController.qml
ui/qml/panels/bottom/TerminalScrollController.qml
ui/qml/assistant/AssistantPanel.qml
crates/kinein-core/src/terminal.rs
crates/kinein-core/src/handlers/terminal.rs
crates/kinein-protocol/src/terminal.rs
scripts/qml-harness/tst_terminal_*.qml
```

O último gate, anterior ao teste humano, passou com 337 testes Rust, Clippy
`-D warnings`, C++/QML estritos, 12 harnesses, builds Debug/Release e smoke
release de 8 segundos (`exit 124` esperado). Esse gate demonstra integridade
automatizada, não aceite visual.

## 4. Auditoria profissional atual

### 4.1 Code OSS — base funcional autorizada

- Repositório: `microsoft/vscode`.
- Revisão estudada: `234638618394269563dd77c0c395c270d8df8b12`.
- Licença/modo: MIT, MODE-B.
- Subsistemas:
  - `src/vs/workbench/contrib/terminal/browser/xterm/xtermTerminal.ts`;
  - `src/vs/workbench/contrib/terminal/browser/terminalConfigurationService.ts`;
  - `src/vs/workbench/contrib/terminal/browser/terminalInstance.ts`;
  - `src/vs/workbench/contrib/terminal/browser/terminalProcessManager.ts`;
  - `src/vs/workbench/contrib/terminal/browser/terminalResizeDebouncer.ts`;
  - testes em `src/vs/workbench/contrib/terminal/test/browser/`.

Invariantes extraídos:

1. O workbench configura e coordena o terminal, mas não redesenha o cursor em
   uma camada paralela: font, line height, letter spacing, cursor e PTY são
   entregues a uma instância xterm.
2. Quando o renderer já existe, suas dimensões de célula são a fonte das
   métricas devolvidas ao restante do workbench.
3. Resize deriva linhas/colunas das dimensões escaladas por DPR, com regras de
   arredondamento explícitas; `charWidth/charHeight` isolados são considerados
   insuficientes quando o DPR muda.
4. Configuração de cursor é preferência de fallback. Estado solicitado pela
   aplicação continua pertencendo ao emulador/renderizador.
5. A sessão e o processo são separados da superfície; a mesma infraestrutura
   atende shell e aplicações TUI.

Adaptação Kinein:

```text
Code OSS workbench/configuração     → composição QML/controllers
xterm terminal instance            → TerminalManager + TerminalPanel
pseudoterminal/process manager      → portable-pty no Rust Core
xterm buffer/parser                 → vt100/vte no Rust Core
xterm renderer e cell dimensions    → renderer Qt com métricas centralizadas
```

Não copiar funções, classes, testes ou fórmulas como tradução mecânica. As
regras numéricas abaixo são referências de comportamento a validar com Qt.

### 4.2 xterm.js — contrato de emulação e geometria usado pelo Code OSS

- Repositório: `xtermjs/xterm.js`.
- Revisão estudada: `ce2169485677951c7701129516cbb68e01330d86`.
- Licença/modo: MIT, MODE-B; não será incorporado como runtime.
- Subsistemas:
  - `src/common/InputHandler.ts`;
  - `addons/addon-webgl/src/WebglRenderer.ts`;
  - `addons/addon-webgl/src/RectangleRenderer.ts`;
  - `addons/addon-webgl/src/GlyphRenderer.ts`.

Invariantes extraídos:

1. `CSI Ps SP q` mantém estilo e piscagem em estado separado; `Ps=0` restaura
   a preferência configurada, `1/2` escolhem bloco, `3/4` underline e `5/6`
   barra.
2. O renderer separa **caixa do glifo** de **caixa da célula**. A célula é
   derivada do glifo e do line height; o glifo recebe um deslocamento interno.
3. Métricas são primeiro arredondadas em pixels físicos. Dimensões lógicas são
   derivadas dessas dimensões físicas, reduzindo borrão e divergência em DPR
   fracionário.
4. O cursor de barra ocupa a altura inteira da célula; underline usa a borda
   inferior. Ambos usam exatamente a mesma largura/altura de célula do texto.
5. O mesmo renderer possui modelo de glifos, fundos, cursor e dimensões. Isso
   evita que `Text.AlignVCenter` e um `Rectangle` independente discordem sobre
   baseline ou arredondamento.

### 4.3 Referências complementares já auditadas

- OpenAI Codex, revisão
  `7d1218a9975fa6e8151f683b182b6eb33294596e`, Apache-2.0, MODE-B:
  posiciona o cursor nativo por frame e pode solicitar `SteadyBar`.
- Claude Code público, revisão
  `c39cb0f14bfe8bb519bae5bfc55add6867c5e2ab`: o changelog chama o caret de
  `native terminal cursor`; a implementação da TUI não está publicada e não
  deve ser inferida.
- Zed, revisão `1e22d1a83f8b1b7acc528d15cfab0644852380c0`, MODE-D:
  preserva a forma do cursor no backend Alacritty e o cria a partir dos mesmos
  bounds/line height da célula.

## 4.4 Evidência nova de 2026-07-16 — o modo plano isola o sintoma

Ao testar o transcript plano (`--ax-screen-reader`, seção 11), o autor relatou:

```text
modo plano (buffer principal, texto linear) → cursor de texto NORMAL, scroll normal
TUI decorativa (tela alternada)             → cursor "flutuando"/desalinhado
```

Isto é a primeira reprodução que **separa o sintoma por modo de renderização**,
usando o mesmo renderer, a mesma fonte e a mesma grade VT. Consequências para as
hipóteses da seção 5:

- **enfraquece H1, H2, H3 e H4** (caixa do glifo × célula, arredondamento
  lógico/físico, baseline por span, fallback de fonte): todas são propriedades da
  grade e da fonte, portanto quebrariam nos **dois** modos. O cursor correto no
  modo plano mostra que a geometria base está sã;
- **fortalece H6** (o desalinhamento percebido está na relação entre o cursor e a
  moldura que a própria TUI desenha, não na célula) e a família de causas ligadas
  ao caminho de **tela alternada**: a TUI posiciona o caret dentro de um quadro
  decorativo, e é a discrepância entre esse quadro e a célula VT que aparece como
  "flutuando";
- mantém H5 em aberto: falta provar qual sequência a TUI emite naquele estado.

Portanto a retomada do R0 deve comparar os dois modos lado a lado com a mesma
fixture e overlay, em vez de investigar métricas de fonte isoladamente. Ainda
**não é causa-raiz**: continua proibido ajustar `y` ou criar offset por agente.

## 4.5 Reenquadramento de 2026-07-16 — o alvo é o emulador, não o agente

Observação do autor que reorienta este roadmap:

```text
Claude e Codex funcionam normalmente no terminal do sistema.
Só quebram dentro da Kinein.
O pedido sempre foi: "meu terminal, inteiro, dentro da IDE".
```

Isto é decisivo. Se o mesmo binário, na mesma máquina e fonte, se comporta bem
fora e mal dentro, então a variável independente é o **emulador da Kinein**, não
a TUI do agente. Consequências:

- **`--ax-screen-reader` é paliativo, não correção.** Ele muda o agente para
  caber num emulador incompleto. Mantido como toggle (seção 11), mas não é o
  norte e não deve ser default.
- **Causa provável: profundidade do emulador.** O core usa a crate `vt100`, um
  emulador VT deliberadamente simples. Tela alternada, mouse, reflow, cursor e
  scrollback de aplicações TUI exigem um emulador completo. As referências que o
  próprio roadmap adota já resolveram isso adotando um emulador maduro em vez de
  escrever um: Code OSS usa xterm.js, IntelliJ usa JediTerm, **Zed usa
  `alacritty_terminal`**.
- **Candidato natural e alinhado à política do repositório:**
  [`alacritty_terminal`](https://github.com/alacritty/alacritty) — Rust puro,
  Apache-2.0 (compatível com o duplo MIT/Apache-2.0), mantido, e do **mesmo
  ecossistema do `vte` que a Kinein já linka**. Substituiria `vt100` como motor
  de grade/estado VT, preservando `portable-pty`, o IPC tipado e o renderer
  Qt/QML. É MODE-A do roadmap de adaptação: integrar biblioteca aberta madura em
  vez de reimplementar.

**Troca executada em 2026-07-16** — `DocsPublic/decisoes-adr/ADR-0004-alacritty-terminal-emulator.md`.
O motor passou a ser o `alacritty_terminal`, com o contrato
`event.terminal.render` **inalterado** (UI, harnesses e sonda não mudaram).
261 testes, clippy estrito e a sonda e2e (scrollbackMax, eco de offset, clamp,
multi-sessão) verdes. `vt100` e `vte` saíram das dependências.

Consequência para este roadmap: as hipóteses H1–H6 da seção 5 foram escritas
quando o emulador era raso. **Antes de retomar qualquer uma delas, refazer o
gesto visual com Claude/Codex no emulador novo** — é plausível que o sintoma do
cursor tenha ido embora junto com a causa. Se persistir, o R0 continua válido,
mas agora com um VT de verdade por baixo, o que torna a investigação de métrica
de fonte finalmente honesta.

A ordem do R3 permanece invertida: emulador primeiro (feito), renderer depois.

## 4.6 R0 executado — medição de 2026-07-16

Ambiente registrado (R0 item 2): Fedora 44, Wayland/GNOME, Qt 6.11.1,
`Screen.devicePixelRatio = 1`, sem `QT_SCALE_FACTOR`/`QT_FONT_DPI`;
`Theme.monoFont = "monospace"` resolvido por fontconfig para **Noto Sans Mono**;
`Theme.fontSizeTerminal = 13`; `claude` 2.1.211, `codex-cli` 0.144.4.

Observação do autor que orientou a medição: o cursor "está fixo e não acompanha
o dimensionamento, fica desalocado e não segue a barra de texto"; ocorre nos
**dois** agentes, já com o emulador `alacritty_terminal`.

### Tabela numérica (R0 item 6)

```text
charWidth  (advanceWidth "M") = 7.796875     ← fracionário
lineHeight (height)           = 17.6875      ← fracionário
ascent 13.890625 | descent 3.796875 | leading 0
DPR = 1  →  nenhuma das duas cai em pixel físico inteiro

avanço por atributo:  Normal 7.796875 | Medium 7.796875 | Italic 7.796875
altura  por atributo: Normal 17.6875  | Medium 17.6875

Text.contentHeight = 18   vs   caixa da célula = 17.6875
    folga = -0.3125  →  AlignVCenter desloca o glifo em -0.15625 e a tinta
                        transborda a caixa

erro do cursor (Math.floor(col * charWidth) vs a posição real do texto):
    col  7 → real  54.578 | floor  54 | erro 0.578
    col 13 → real 101.359 | floor 101 | erro 0.359
    col 40 → real 311.875 | floor 311 | erro 0.875
    col 79 → real 615.953 | floor 615 | erro 0.953
```

### Causa-raiz

Texto e cursor usam **sistemas de posicionamento independentes**:

```text
texto   → layout (Column/Row), span em cells * charWidth, glifo centralizado
          implicitamente por Text.AlignVCenter. Posições REAIS, não arredondadas.
cursor  → aritmética explícita: Math.floor(col * charWidth),
          Math.floor(row * lineHeight). ARREDONDADO para baixo.
```

Com a célula fracionária, `Math.floor` descarta até 0,95px, e o erro **varia com
a coluna** (0,36 a 0,95px) em vez de ser constante. O cursor barra tem
`thickness = max(2, round(charWidth * 0.18))` = **2px**, então um erro de 0,95px
é ~metade da largura do cursor — visível, e mudando conforme o texto avança. É
exatamente o relato: o cursor está sobre a grade matemática, o texto não.

### Estado das hipóteses da seção 5

| Hipótese | Veredito pela medição |
| --- | --- |
| H1 caixa do glifo e célula fundidas | **confirmada** — `contentHeight` 18 > célula 17,6875; `AlignVCenter` desloca −0,15625 e a tinta transborda |
| H2 arredondamento lógico/físico diverge | **confirmada, é o driver** — `floor` no cursor e real no texto, com célula fracionária |
| H3 um `Text` por span cria baselines independentes | aberta, mas improvável como driver: todos os spans sofrem o mesmo deslocamento |
| H4 fallback de fonte muda métricas | **refutada para o caso do cursor** — peso e itálico não mudam avanço nem altura; box drawing resolve para a mesma família. CJK e emoji caem em outras famílias (Noto Sans Mono CJK, Symbola), o que importa para R4, não aqui |
| H5 caminho DECSCUSR não exercitado | refutada — 0.57 implementou e a sonda confirma forma/piscagem |
| H6 sintoma está na baseline da linha | **parcial** — o glifo de fato transborda a caixa, mas o desvio dominante é horizontal e cresce com a coluna |

### Consequência para R1

A correção é a do R1 como já estava escrita, e a medição diz por quê:
arredondar a grade em **pixels físicos primeiro** e derivar os lógicos, e fazer
texto, cursor, seleção, mouse e scrollbar consumirem a **mesma** instância de
métricas. Não é offset: um offset constante não corrige um erro que varia com a
coluna.

### Artefatos

- `scripts/fixture_terminal_geometria.py` — fixture determinística (R0 item 4):
  régua de colunas, moldura de box drawing, classes de glifo, as sete formas
  DECSCUSR, tela alternada e cursor na primeira/última coluna. Sem rede, sem
  leitura de arquivo, sem conteúdo do usuário; dois runs são idênticos byte a
  byte. Serve também à matriz de fixtures do R4.
- Overlay `KINEIN_TERMINAL_DEBUG_GEOMETRY=1` (R0 itens 5 e 6): **implementado**
  em `ui/qml/panels/bottom/TerminalGeometryOverlay.qml`, ligado pelo singleton
  `DebugFlags` (`ui/src/debug_flags.*`). Sem a env o `Loader` não instancia e o
  uso normal não paga nada. Harness:
  `scripts/qml-harness/tst_terminal_geometry_overlay.qml`.

  ```text
  KINEIN_TERMINAL_DEBUG_GEOMETRY=1 QT_ASSUME_STDERR_HAS_CONSOLE=1 scripts/kinein-vectis
  ```

  Desenha: retângulo lógico de cada célula (verde), baseline (magenta), topo do
  ascent (amarelo), célula onde o core diz que o cursor está (ciano) e o
  retângulo **efetivo** do cursor desenhado (vermelho), medido do item real e
  não recalculado — se fosse recalculado, o overlay concordaria consigo mesmo e
  não provaria nada. Leitura numérica no canto e uma linha de log por mudança de
  estado, só com métricas e cursor.

  Usa `console.warn` e não `console.log`: a categoria de debug do QML vem
  desligada por padrão no Qt do Fedora, e o log não apareceria.

### R1 executado, sintoma NÃO resolvido (2026-07-16)

O R1 zerou o erro aritmético (0,000px em toda coluna, ver o bloco no R1) e o
autor reprovou: **o comportamento visual do cursor continua o mesmo**. Registro
do que isso significa, para a próxima sessão não repetir o caminho:

- o erro horizontal por `Math.floor` era **real e foi corrigido**, mas não era a
  causa do que se vê;
- em DPR 1 o R2 é previsivelmente nulo: a folga vertical já é exatamente 0 e a
  baseline explícita difere do `AlignVCenter` em ~0,11px. **Não executar o R2
  esperando corrigir este sintoma**;
- como a aritmética virou exata sem efeito visual, a divergência está **depois**
  das coordenadas — na composição/rasterização do `Text` por span. É a condição
  que o R3 nomeia para trocar de renderer;
- hipótese concorrente não descartada: o `claude` emite `?25l` (esconde o cursor
  VT) e desenha o próprio cursor como célula. Nesse caso não há cursor da Kinein
  na tela e o que parece torto é um span. O overlay separa os dois casos: célula
  ciano sem retângulo vermelho = a aplicação escondeu o cursor.
- **O item 3 do R0 (screenshot comparável) nunca foi produzido.** Ele é o que
  falta; o R0 diz "sem isso, parar" e seguir sem ele custou uma fatia inteira.

## 4.7 CAUSA-RAIZ ENCONTRADA — linha vazia colapsa no positioner (2026-07-16)

O que destravou foi a **captura de tela** (`imagens/bugs/`), o item 3 do R0 que
nunca tinha sido produzido. Ela mostrou de imediato o que nenhuma medição tinha
mostrado: o desvio é de **linha inteira**, não de sub-pixel — o cursor aparece
solto no canto inferior esquerdo, ~2–3 linhas abaixo da caixa de entrada, nos
dois agentes.

### Evidência

Bytes reais do `claude` alimentados no emulador de produção
(`alacritty_terminal`, mesmo caminho do core):

```text
SHOW_CURSOR   = true
ALT_SCREEN    = true
cursor point  = linha 27 coluna 2
 27|❯ Try "write a test for build.rs"|     ← o cursor ESTÁ na linha da entrada
 28|──────────────────────────────────|
 29|  ⏸ manual mode on · ? for shortcuts|
```

**O core está correto.** O cursor está na linha e coluna certas. O erro é da UI.

Medição do `Column`/`Row` do `TerminalViewport`, com linhas 2–4 vazias:

```text
row 0: y=0    esperado=0
row 1: y=18   esperado=18
row 2: y=0    esperado=36    <<<< linha vazia: NÃO POSICIONADA
row 3: y=0    esperado=54    <<<<
row 4: y=0    esperado=72    <<<<
row 5: y=36   esperado=90    <<<< subiu 3 linhas
row 6: y=54   esperado=108   <<<<
```

### Causa

O positioner do Qt Quick **descarta filhos de largura zero**. Uma linha vazia
chega do core como `[]` — `build_line` corta o run final quando é só espaços no
estilo default — e vira um `Row` sem filhos, logo sem largura. O `Column` a
ignora como se não existisse, e todo o texto abaixo sobe uma linha por linha
vazia acima. O cursor não sobe junto, porque é posicionado por `yForRow(row)`,
a grade matemática — que estava certa o tempo todo.

**O cursor nunca esteve errado. O texto é que escorregava para cima.**

Isso explica tudo o que não fechava:

- as correções de sub-pixel (R1) não mudaram nada porque o erro é de linha
  inteira;
- atinge os dois agentes porque toda TUI tem linha vazia;
- o Terminal comum passou no aceite porque no shell as linhas vazias ficam
  **abaixo** do cursor; numa TUI ficam acima;
- `?25l`/`?25h` era pista falsa: o `claude` termina com `SHOW_CURSOR = true` e o
  cursor deve mesmo aparecer.

### Correção

A regra do R1.4 levada até o fim: a linha é posicionada pela **mesma métrica que
o cursor** (`y: metrics.yForRow(index)`), não por um positioner. O `Column` saiu.
Layout não decide geometria de grade. Spans continuam num `Row` — `flush_span`
descarta span de 0 células, então eles têm largura ≥ 1 e não colapsam.

### Lição de processo

O R0 exige três saídas — tabela numérica, fixture e **screenshot comparável** —
e diz "sem isso, parar". As duas primeiras foram produzidas e a terceira foi
tratada como opcional. As duas primeiras confirmaram hipóteses reais (H1, H2) que
**não eram a causa**, e custaram a fatia R1 inteira. A captura resolveu em um
minuto. Quando o roadmap lista uma evidência como obrigatória, ela é o gate, não
uma sugestão.

## 5. Hipóteses ordenadas — ainda não são causa-raiz

### H1 — caixa do glifo e célula foram fundidas

É a hipótese principal. `TextMetrics.height` é usado como `lineHeight`, o
`Text` é centralizado implicitamente e o cursor ocupa a caixa inteira. O Qt
pode posicionar a tinta/baseline do glifo de modo diferente do retângulo da
célula. Alterar apenas o `y` do cursor não corrige essa relação.

### H2 — arredondamento lógico/físico diverge

`Math.floor` opera em pixels lógicos no QML atual. Em escala 125%, 150% ou com
DPR não inteiro, texto e retângulos podem terminar em pixels físicos distintos.

### H3 — um `Text` por span cria baselines/rasterizações independentes

Bold, itálico e fallback podem produzir caixas diferentes por span. A Row
mantém larguras VT corretas, mas a tinta visível pode não compartilhar uma
baseline explícita entre spans.

### H4 — fallback de fonte muda métricas efetivas

`Theme.monoFont` é `monospace`, portanto a família concreta depende do host.
Emoji, símbolos e box drawing podem acionar fallback. É preciso registrar a
fonte realmente resolvida e verificar largura monoespaçada, não adivinhar.

### H5 — o caminho DECSCUSR não é exercitado pela tela observada

Codex normalmente usa `DefaultUserShape` e só muda para `SteadyBar` em estados
específicos. Claude não publica a implementação. A fixture precisa provar qual
sequência chegou ao core e qual forma saiu no frame; suporte correto a uma
sequência não garante que ela explica o caso visual.

### H6 — o sintoma percebido está na baseline da linha, não no cursor

A barra pode estar geometricamente na célula correta e ainda parecer baixa
porque os glifos e a moldura desenhada pela TUI ficam altos dentro da célula.
Somente screenshot e overlay de caixas/baseline distinguem as duas coisas.

## 6. Roadmap executável

Cada etapa termina em uma evidência. Não avançar para um renderer novo apenas
porque uma hipótese parece plausível.

### R0 — congelar, reproduzir e medir

Objetivo: transformar o feedback visual em uma reprodução determinística.

1. Preservar o código atual e o worktree; não voltar a offsets.
2. Registrar ambiente do teste:
   - distro/sessão gráfica;
   - escala e DPR;
   - família de fonte resolvida e tamanho;
   - comando/versão de Claude ou Codex;
   - dimensões do painel, colunas e linhas.
3. Capturar a mesma tela de entrada na Kinein e em um terminal externo com
   fonte/tamanho equivalentes. Não incluir prompts, segredos ou conteúdo do
   usuário em fixtures ou logs.
4. Criar uma fixture PTY local, versionada e sem rede que emita:
   - uma régua de células e box drawing semelhante a uma barra de TUI;
   - caracteres ASCII, CJK/largo, combinante e emoji/fallback;
   - DECSCUSR reset, bloco, underline e barra, steady/blinking;
   - alternate screen, resize e cursor em primeira/última coluna.
5. Adicionar overlay temporário somente sob
   `KINEIN_TERMINAL_DEBUG_GEOMETRY=1`, mostrando:
   - retângulo lógico e físico da célula;
   - baseline, ascent, descent e leading do glifo;
   - retângulo efetivo do cursor;
   - char width, cell width/height e DPR.
6. Logar uma vez por mudança de frame apenas métricas e estado do cursor. Não
   logar texto do terminal, prompt, clipboard ou bytes da sessão.

**Saída obrigatória:** screenshot comparável + fixture reproduzindo ou
refutando o desalinhamento + tabela numérica das métricas. Sem isso, parar.

### R1 — criar uma fonte única de métricas de célula

> **Estado: implementado em 2026-07-16.** Componente:
> `ui/qml/panels/bottom/TerminalMetrics.qml` (QtQuick puro, sem regra de
> negócio, sem IPC). Harness: `scripts/qml-harness/tst_terminal_metrics.qml`.
> Resultado medido:
>
> ```text
> celula em pixel FISICO inteiro, por DPR:
>   DPR 1.00 →  8 x 18 px  →  8.0000 x 18.0000 logicos | baseline 14px
>   DPR 1.25 → 10 x 22 px  →  8.0000 x 17.6000 logicos | baseline 17px
>   DPR 1.50 → 12 x 27 px  →  8.0000 x 18.0000 logicos | baseline 21px
>   DPR 2.00 → 16 x 35 px  →  8.0000 x 17.5000 logicos | baseline 28px
>
> erro cursor-vs-texto:      ANTES  →  DEPOIS
>   col  7                   0.578  →  0.000
>   col 13                   0.359  →  0.000
>   col 40                   0.875  →  0.000
>   col 79                   0.953  →  0.000
>
> folga vertical do glifo:  -0.3125 (transbordava)  →  0.0000 (DPR 1)
> ```
>
> Consumidores unificados (R1.4): texto (`widthForCells`), cursor
> (`xForColumn`/`yForRow`), seleção (`columnAt`/`rowAt`), resize
> (`columnsIn`/`rowsIn`) e a camada visual de seleção. `Math.floor` sobre a
> célula desapareceu do `TerminalViewport`. O contrato IPC **não** mudou (R1.5):
> pixels continuam fora do protocolo.
>
> Escolha registrada: arredondamento ao pixel físico **mais próximo**. `floor`
> comprimiria a célula (7,796875 → 7 perde ~0,8px por coluna, ~64px em 80
> colunas); `round` → 8 expande ~0,2px por coluna. O valor exato importa menos
> que ser único e determinístico.
>
> **O que R1 NÃO fechou:** em DPR fracionário a célula lógica pode ficar menor
> que a altura natural do `Text` (DPR 1,25 → 17,6; DPR 2 → 17,5, contra
> `contentHeight` 18), e aí o `AlignVCenter` volta a deslocar o glifo. Em DPR 1
> — o caso do autor — a folga é exatamente 0 e o sintoma sai por consequência,
> mas isso é coincidência aritmética, não garantia. Fechar por baseline
> explícita é exatamente o R2.

Objetivo: separar medida do glifo e medida da célula, com arredondamento
determinístico.

1. Introduzir um pequeno objeto de métricas da UI, sem regra de negócio, que
   exponha no mínimo:
   - família/tamanho/peso base resolvidos;
   - char width e char height;
   - ascent, descent, leading e baseline;
   - cell width, cell height;
   - glyph left/top dentro da célula;
   - DPR e equivalentes em pixels físicos.
2. Usar apenas APIs disponíveis no baseline Qt 6.4.
3. Arredondar primeiro a grade em pixels físicos e derivar pixels lógicos.
4. Fazer resize, texto, cursor, seleção, mouse e scrollbar consumirem essa
   mesma instância.
5. Não alterar o contrato IPC se a mudança for puramente visual. Se o core não
   precisa conhecer pixels, pixels não entram no protocolo.

**Gate:** testes puros de métricas para DPR 1, 1.25, 1.5 e 2; nenhum cálculo
duplicado em Terminal/KV; fixture visual melhor ou numericamente explicada.

### R2 — posicionar glifos por baseline explícita — CONDICIONAL desde 2026-07-17

> **Não executar por causa do cursor: o sintoma acabou.** A causa-raiz era a linha
> vazia colapsando no positioner (§4.7), corrigida em `ab3becc`, e o autor
> **aprovou o visual em 2026-07-17**. O próprio R1 já media que em DPR 1 o R2 é
> nulo (baseline explícita difere do `AlignVCenter` em ~0,11px).
>
> **O que sobra de R2, e é outra coisa:** determinismo em DPR ≠ 1 (125%, 150%),
> que continua **não demonstrado**. Só executar se uma MEDIÇÃO em DPR ≠ 1 mostrar
> divergência — nunca por impressão visual, que foi o que custou a fatia R1
> inteira.

Objetivo: eliminar `Text.AlignVCenter` como decisão implícita do terminal.

1. Posicionar todos os spans da linha por uma baseline comum calculada em R1.
2. Preservar células VT como largura autoritativa; não voltar a
   `implicitWidth`, `text.length` ou parsing de prompt.
3. Confirmar bold/italic/underline, box drawing, wide e combining.
4. Cursor continua usando o retângulo da célula, nunca bounds da tinta.

**Gate:** Terminal comum e Assistente geram a mesma geometria para o mesmo
frame; screenshot da fixture e gesto humano em Claude/Codex.

### R3 — decidir o renderer Qt definitivo — DECIDIDO em 2026-07-17: opção 1

> **A decisão que este R3 pedia foi tomada, e por evidência.** O critério estava
> escrito aqui: *"QML com baseline/métricas explícitas: manter se R2 corrigir o
> visual e cumprir orçamento de frame/memória"*. O visual foi corrigido (§4.7,
> `ab3becc`) e **aprovado pelo autor em 2026-07-17**; os orçamentos de frame estão
> medidos em `DocsPublic/roadmaps/21` §A3.
>
> **Fica a composição QML atual. Não trocar de renderer.** As opções 2
> (`QQuickItem` nativo) e 3 (componente Qt maduro) existiam para o caso de o
> modelo por span continuar divergindo — não continuou. Reabrir exige medição
> nova que mostre divergência ou estouro de orçamento, registrada em ADR.

Objetivo (histórico): escolher por evidência entre a composição QML atual e um
item nativo especializado.

Opções em ordem de custo:

1. **QML com baseline/métricas explícitas:** manter se R2 corrigir o visual e
   cumprir orçamento de frame/memória.
2. **`QQuickItem` nativo dedicado:** adotar se o modelo por span continuar
   divergindo ou ficar caro. Um único item deve desenhar fundo, glifo e cursor
   com a mesma geometria; input/processo continuam fora dele.
3. **Componente terminal Qt maduro:** só avaliar após auditoria de licença,
   manutenção, compatibilidade VT, acessibilidade e custo de integração. Não
   substituir a arquitetura silenciosamente.

Não adotar xterm.js/Node/WebView. A política do repositório exige adaptação
nativa: a fidelidade é funcional, não transplante de runtime.

**Gate de decisão:** benchmark, compatibilidade visual, acessibilidade e
complexidade de manutenção registrados em ADR se houver troca de renderer.

### R4 — paridade VT/TUI necessária para Claude/Codex

Objetivo: o mesmo programa TUI se comportar na Kinein como em terminal moderno.

Auditar por fixture, sem implementar sequências aleatórias:

- cursor save/restore, visibility e DECSCUSR;
- alternate screen e scrollback;
- bracketed paste;
- application cursor/keypad;
- resize/reflow e cursor na margem;
- erase display/line sem apagar transcript indevidamente;
- wide, combining, emoji/fallback e box drawing;
- SGR 16/256/truecolor, inverse, bold, italic e underline;
- foco e protocolos de mouse somente quando a aplicação os habilitar;
- OSC 8 links e título apenas com política de segurança explícita.

**Gate:** matriz da fixture externa versus Kinein, cobrindo terminal comum,
Codex inline e Claude. Toda diferença fica documentada como bug, decisão ou
fora de escopo.

### R5 — interações profissionais do terminal

Objetivo: aproximar a experiência funcional autorizada do Code OSS sem copiar
seu workbench.

Ordem sugerida:

1. seleção por célula/palavra/linha e copiar/colar seguro;
2. busca no buffer sem bloquear a UI;
3. links detectados/OSC 8 com confirmação e escaping seguro;
4. configuração de fonte, tamanho, line height, cursor e scrollback;
5. zoom e reflow previsíveis;
6. acessibilidade de leitura, foco e contraste;
7. reconexão/estado de processo apenas se o produto precisar;
8. shell integration tipada como fatia própria, nunca parsing improvisado.

Cada item requer sua própria referência atual, ameaça/erro, teste e critério de
aceite. “Paridade” não autoriza implementar tudo em um commit.

### R6 — desempenho e robustez

Objetivo: garantir que a fidelidade não torne a IDE pesada.

Medir antes/depois:

- tempo de frame em rajadas de saída;
- frames descartados/coalescidos;
- CPU/memória por grade 80x24, 160x50 e scrollback longo;
- latência tecla → PTY → frame;
- resize contínuo;
- criação/destruição de objetos por span;
- DPR e troca de tela;
- sessão longa de Claude/Codex sem crescimento não limitado.

Aplicar backpressure/coalescência no dono correto. Não resolver custo do
renderer descartando estado VT ou movendo lógica de negócio para QML.

### R7 — aceite e distribuição

Só concluir quando:

1. fixture automatizada estiver verde;
2. gate integral estiver verde;
3. smoke do launcher de desenvolvimento estiver verde;
4. o usuário aprovar visualmente Claude ou Codex no Assistente;
5. Terminal comum não regredir;
6. documentação e registro de referências estiverem atualizados.

Somente depois do aceite explícito gerar o novo AppImage, validar host e
Debian mínimo, atualizar checksums e criar checkpoint local. Push/publicação
continuam sem autorização separada.

## 7. Matriz inicial de paridade

| Capacidade | Estado atual | Alvo | Etapa |
| --- | --- | --- | --- |
| PTY real e shell/CLI | implementado | preservar | regressão contínua |
| Terminal e KV no mesmo backend | implementado | preservar | regressão contínua |
| Input caractere a caractere | implementado | validar TUIs | R4 |
| Resize coalescido | implementado | validar DPR/reflow | R1/R4 |
| Grid VT + cores/atributos | implementado parcial | matriz compatível | R4 |
| Largura de glifo em células | implementado | preservar | R1/R2 |
| Forma/piscagem DECSCUSR | implementado; coberto pelo aceite de 2026-07-17 | preservar | R4 |
| Baseline e caixa da célula | fonte única (`metrics.yForRow`), R1 feito | preservar | R2 só se DPR≠1 divergir |
| DPR físico | não demonstrado | determinístico | R1 |
| Cursor visual Claude/Codex | **APROVADO pelo autor em 2026-07-17** | preservar | regressão contínua |
| Seleção/clipboard | implementado básico | paridade profissional | R5 |
| Busca/links/a11y | incompleto | fatias próprias | R5 |
| Rajadas/performance | infraestrutura parcial | orçamento medido | R6/A3 |

## 8. Testes mínimos da solução

### Rust/core

- parser chunk-safe para sequências VT mantidas pela fatia;
- posição/visibilidade/forma do cursor;
- wide/combining e continuidade de célula;
- resize, alternate screen, scrollback e múltiplas sessões;
- nenhum log de conteúdo/sigilo.

### Qt/QML/UI

- métricas de célula em DPR variados;
- mesma geometria para `TerminalPanel` no Bottom Panel e no AssistantPanel;
- baseline comum entre spans normal/bold/italic;
- cursor block/bar/underline e steady/blinking;
- seleção/hit-test depois de glifo largo;
- resize sem loop e troca de sessão sem estado residual.

### E2E visual e humano

- fixture PTY determinística;
- comparação em terminal externo;
- Claude e Codex reais somente no gesto humano, sem snapshotar conteúdo;
- Terminal comum como controle de regressão;
- escala 100% e ao menos um DPR fracionário disponível.

## 9. Não fazer

- não recolocar `cursorVerticalOffset` ou constante por Claude/Codex;
- não mover o cursor com base na moldura visual da TUI;
- não interpretar prompt, texto, status ou nome do agente;
- não criar um renderer exclusivo do Assistente;
- não executar CLI diretamente da UI;
- não copiar/verter funções do Code OSS ou xterm.js;
- não incorporar Node, Electron, WebView ou Extension Host;
- não adicionar dependência sem auditoria/ADR quando aplicável;
- não gerar AppImage nem chamar o problema de resolvido antes do aceite humano;
- não misturar esta correção com A3, biblioteca de plugins ou nova UI.

## 10. Instrução curta para a próxima sessão

```text
Leia AGENTS.md e DocsPublic/roadmaps/26-terminal-rendering-parity-roadmap.md. Preserve o
worktree. O cursor da TUI continua reprovado no teste humano; não ajuste y.
Comece por R0: fixture PTY + métricas/overlay sob flag, compare com terminal
externo e só então implemente R1. Code OSS/xterm.js são a base de paridade
comportamental, mas a solução permanece nativa Rust + IPC + Qt. Não gere
AppImage ou push antes do gate e do aceite explícito. Um checkpoint local pode
registrar este estado adiado depois de gate verde, desde que não o descreva
como paridade visual concluída.
```

## 11. Scrollback do agente em tela alternada — diagnóstico 2026-07-16

> **Origem:** relato de que o scroll do agente Claude no Assistente dificulta
> rever o histórico do raciocínio. Pedido: entender o que na TUI ou na UI/UX da
> IDE interfere, tratando a TUI do agente como soberana (referência Code OSS,
> Zed, IntelliJ IDEA Community).
> **Status:** diagnóstico com evidência empírica concluído; correção **não**
> aplicada porque redefine a experiência do Assistente e exige gesto visual,
> conforme a regra deste documento. Duas opções de correção prontas abaixo.

### 11.1 Evidência empírica das CLIs

Consulta direta ao `--help` das ferramentas instaladas no host:

```text
codex --no-alt-screen  → "Disable alternate screen mode";
                         "Runs the TUI in inline mode, preserving terminal
                          scrollback history."
claude                 → NÃO possui --no-alt-screen nem modo inline. A sessão
                         interativa padrão usa uma TUI de tela cheia. A opção
                         mais próxima é --ax-screen-reader: "Render screen-reader
                         friendly output (flat text, no decorative borders or
                         animations)".
```

Configuração dos perfis **como era até o 0.59.0**, em
`crates/kinein-core/src/handlers/ai.rs` — arquivo REMOVIDO naquele protocolo
(registro histórico; não existe mais e a linha de IA está fora de escopo):

```text
Claude → args: &[]                    (TUI de tela cheia, sem inline)
Codex  → args: &["--no-alt-screen"]   (inline, scrollback preservado)
```

### 11.2 Causa-raiz

O sintoma **não é um defeito no código de scroll da Kinein**. O
`TerminalScrollController`, o auto-ancoramento do `vt100` e a sonda
`sonda_scrollback.py` estão corretos para conteúdo de buffer principal. A causa é
a semântica de VT combinada com a ausência de encaminhamento de mouse:

1. A sessão interativa padrão do Claude entra na **tela alternada** (alternate
   screen). Por definição, a tela alternada **não tem scrollback** — igual a
   `vim`, `less` ou `htop`. O `event.terminal.render` já expõe
   `alternateScreen: true` nesse estado; `scrollbackMax` fica em zero e não há
   histórico para a barra rolar. Logo, rolar não mostra o raciocínio anterior
   porque o app repinta a tela inteira no lugar.
2. A Kinein **não encaminha a roda do mouse para o aplicativo** quando ele está
   em tela alternada e/ou habilitou rastreamento de mouse. O `WheelHandler` de
   `TerminalPanel.qml` sempre rola o scrollback (vazio, na tela alternada) da
   própria IDE e nunca envia o gesto ao programa. Assim, o scroll interno do
   próprio Claude também não recebe o evento. Este item está listado como
   pendente em R4 ("foco e protocolos de mouse somente quando a aplicação os
   habilitar").
3. O Codex não sofre disso porque `--no-alt-screen` o mantém no buffer
   principal, onde o scrollback da IDE funciona. O Claude não oferece flag
   equivalente.

### 11.3 Como referências profissionais tratam isto

- **Code OSS / xterm.js:** o emulador mantém scrollback apenas para o buffer
  principal. Em tela alternada com rastreamento de mouse habilitado, a roda é
  **traduzida em eventos de mouse e enviada ao aplicativo**, que rola o próprio
  conteúdo; a IDE não tenta inventar histórico para a tela alternada.
- **Zed:** backend Alacritty com a mesma semântica — buffer alternado sem
  scrollback da IDE; a aplicação recebe o gesto quando pede mouse.
- **IntelliJ IDEA Community (JediTerm):** emulador de terminal com scrollback do
  buffer principal e encaminhamento ao app no buffer alternado; aplicações TUI
  scrollam a si mesmas.

Conclusão de referência: a experiência correta **não** é a IDE forjar scrollback
para a tela alternada, e sim (a) deixar programas inline usarem o buffer
principal, e (b) encaminhar a roda ao app quando ele estiver em tela alternada
ou com mouse habilitado. É literalmente "usar a TUI do agente" como pedido.

### 11.4 Opções de correção

**Opção A — encaminhar a roda ao aplicativo (correção arquitetural).
IMPLEMENTADA em 2026-07-16, protocolo `0.60.0`; aguarda gesto visual.**

O esboço original desta opção previa a codificação **na UI**:

```text
UI → WheelHandler: se (alternateScreen || mouseTracking), codificar a roda
     como evento de mouse SGR/X10 e mandar por terminal.input
```

Isso está **errado** e não foi seguido: colocar a codificação e a condição no
frontend é exatamente a lógica de negócio na UI que a `ARCHITECTURE.md` proíbe —
a mesma classe de defeito que causou o problema original. Nas três referências a
decisão mora no backend (`scroll_wheel` do Zed, `MouseStateService` do xterm.js,
`JediTerminal.mouseReport` do `JediTerm`). O desenho aplicado:

```text
UI       → reporta o GESTO CRU: célula sob o ponteiro, linhas e modificadores.
           Não sabe o modo VT e não decide nada.
protocolo→ terminal.mouse { id, col, row, event{kind}, modifiers } — contrato de
           mouse completo; só `wheel` implementado, R5 fixado e recusado.
core     → wheel_action(mode, shift) decide: relatório à aplicação / cursor keys
           / histórico local. Codificação SGR e legada no core.
testes   → codificação (SGR, legado, supressão fora de alcance, bits de
           modificador), decisão por modo e a precedência entre os ramos.
```

Resolve Claude, Codex e qualquer TUI (vim, htop) de forma geral. Falta o gesto
visual e a matriz de fixtures do R4.

**Opção B — oferecer modo de texto plano do Claude (rápida, um parâmetro).**

```text
crates/kinein-core/src/handlers/ai.rs, perfil Claude:
    args: &[]  →  args: &["--ax-screen-reader"]
```

Prós: uma linha, espelha o padrão do Codex, produz saída linear no buffer
principal com scrollback navegável — exatamente "ver o histórico do raciocínio".
Contras: remove bordas/animações decorativas do TUI do Claude; é uma mudança
estética grande que o autor deve aprovar visualmente. Idealmente vira um
**ajuste/toggle** no seletor do Assistente, não um default imposto.

### 11.5 Recomendação — HISTÓRICO, superado

> **Estado (2026-07-16):** esta seção descreve a Opção B, que **não existe
> mais**. A remoção do `aiBridge` (protocolo `0.59.0`) apagou a setting
> `aiCliFlatTranscript`, o `ProfileSpec`/`flat_args` e os três testes citados
> abaixo: injetar `--ax-screen-reader` era a IDE se metendo entre o programa e o
> terminal, política por programa que nenhuma IDE profissional aplica. A correção
> vigente é a **Opção A** (§11.4), implementada no protocolo `0.60.0`, que trata
> qualquer TUI igualmente e não conhece o nome do programa.
>
> O texto abaixo fica como registro do que foi tentado e por quê.

**Opção B implementada como toggle em 2026-07-16 (protocolo 0.58.0).** O teste
ao vivo confirmou o diagnóstico: com `--ax-screen-reader` o scroll e o cursor
passaram a funcionar, mas a TUI decorativa do agente deu lugar a texto puro.
Por isso o modo virou uma preferência explícita em vez de um default imposto:

```text
setting `aiCliFlatTranscript` (bool, default false)
    false → cada CLI usa sua TUI decorativa (comportamento histórico)
    true  → transcript plano no buffer principal, com scrollback navegável
```

- `ProfileSpec` ganhou `flat_args` ao lado de `args`; `profile_args(profile,
  flat)` escolhe o par. Claude: `[]` / `["--ax-screen-reader"]`. Codex:
  `["--no-alt-screen"]` nos dois modos — o inline oficial já entrega TUI **e**
  scrollback, então não há apresentação a sacrificar.
- A setting é lida no `aiBridge.terminal.open` e vale na próxima sessão.
  Exposta em Configurações (`Ctrl+Alt+S`) como "Assistente: histórico
  navegável", documentada no `DocsPublic/manual.md` e no contrato `DocsPublic/arquitetura/03`.
- Cobertura: `claude_keeps_its_decorative_tui_by_default`,
  `claude_uses_flat_transcript_only_when_enabled` e
  `codex_keeps_its_official_inline_mode_in_both_transcript_modes`.
- `EffectiveSettings` recebeu `#[allow(clippy::struct_excessive_bools)]` com
  justificativa: é um DTO espelhando o JSON schema, onde `boolean` é a
  representação documentada de cada preferência independente.

Isto fecha o sintoma relatado sem escolher pelo usuário entre apresentação e
histórico: quem quiser reler o raciocínio liga o toggle; quem quiser a TUI rica
não perde nada.

**Correção do diagnóstico desta seção (2026-07-16).** A §11.2 atribuía o sintoma
só à tela alternada. A medição direta das CLIs — abrir cada agente num PTY real,
em workspace confiado, e capturar os modos DEC privados que ele liga — mostrou
que a causa é mais específica:

```text
claude: ?1049h ?1000h ?1002h ?1003h ?1006h  → alt-screen E captura de mouse SGR
codex : ?2004h ?1004h                        → tela normal, sem mouse
```

O Claude **captura o mouse**. Por isso o conserto dele é o **relatório**, não o
`alternate scroll`: uma correção que só traduzisse a roda em cursor keys seria
inerte, porque ele não pede `?1007`. O Codex desenha inline, tem histórico real
na grade, e por isso já rolava — o contraste que provou o diagnóstico.

Detalhe que só apareceu no fonte do emulador: `ALTERNATE_SCROLL` nasce **ligado**
(`TermMode::default()` do `alacritty_terminal`, como no xterm). O Claude satisfaz
os dois ramos ao mesmo tempo, então a **ordem** entre eles é carga estrutural —
captura de mouse tem precedência, e há teste dedicado fixando isso.

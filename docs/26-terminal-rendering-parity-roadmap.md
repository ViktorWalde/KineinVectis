# Roadmap de paridade do terminal e das TUIs

> **Status:** aberto, explicitamente adiado pelo usuário em 2026-07-15.
> **Próxima retomada:** começar por R0 (reprodução instrumentada), sem novo
> ajuste manual de `y`, sem AppImage e sem afirmar causa-raiz antes das provas.
> **Decisão do usuário:** a experiência funcional do terminal do Code OSS é a
> base de referência para a Kinein, inclusive ao executar Claude e Codex. A
> adaptação deve ser comportamentalmente fiel, mas nativa nas camadas da
> Kinein; não incorporar Electron, Node, xterm.js ou código copiado.

Este documento é o handoff executável para a próxima IA. Ele separa o que já
está implementado, o que o teste humano realmente mostrou, as referências
oficiais estudadas e a ordem de investigação/implementação. Leia-o junto de
`AGENTS.md`, `ContextoIA.md`, `GUIAIA.md`, `docs/ARCHITECTURE.md`,
`docs/03-ipc-protocol.md`, `docs/06-strict-mode.md`, da spec do AI CLI Bridge e
da seção D2 de `docs/24-paridade-e-fundacao.md`.

## 1. Contrato de produto que não pode regredir

O KV Context não é um chat embutido nem um terminal alternativo. Ele é somente
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
um único renderer de grade usado por Terminal e KV Context
```

Invariantes obrigatórios:

1. A TUI é soberana sobre conteúdo, posição, forma e piscagem do cursor.
2. A IDE não cria `TextInput`, composer, guia de entrada nem parser de prompt
   sobre Claude/Codex.
3. Atalho, aba, seletor de perfil, foco, largura e maximização do KV Context
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
Terminal comum e no KV Context.

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

Estado do worktree no momento deste handoff:

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

### R2 — posicionar glifos por baseline explícita

Objetivo: eliminar `Text.AlignVCenter` como decisão implícita do terminal.

1. Posicionar todos os spans da linha por uma baseline comum calculada em R1.
2. Preservar células VT como largura autoritativa; não voltar a
   `implicitWidth`, `text.length` ou parsing de prompt.
3. Confirmar bold/italic/underline, box drawing, wide e combining.
4. Cursor continua usando o retângulo da célula, nunca bounds da tinta.

**Gate:** Terminal comum e KV Context geram a mesma geometria para o mesmo
frame; screenshot da fixture e gesto humano em Claude/Codex.

### R3 — decidir o renderer Qt definitivo

Objetivo: escolher por evidência entre a composição QML atual e um item nativo
especializado.

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
nativa, e o usuário autorizou fidelidade funcional, não transplante de runtime.

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
4. o usuário aprovar visualmente Claude ou Codex no KV Context;
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
| Forma/piscagem DECSCUSR | implementado, sem aceite visual | validar fixture | R0/R4 |
| Baseline e caixa da célula | implícitas/divididas | fonte única | R1/R2 |
| DPR físico | não demonstrado | determinístico | R1 |
| Cursor visual Claude/Codex | reprovado no gesto humano | aprovado | R0–R3 |
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
- não criar um renderer exclusivo do KV Context;
- não executar CLI diretamente da UI;
- não copiar/verter funções do Code OSS ou xterm.js;
- não incorporar Node, Electron, WebView ou Extension Host;
- não adicionar dependência sem auditoria/ADR quando aplicável;
- não gerar AppImage nem chamar o problema de resolvido antes do aceite humano;
- não misturar esta correção com A3, biblioteca de plugins ou nova UI.

## 10. Instrução curta para a próxima sessão

```text
Leia AGENTS.md e docs/26-terminal-rendering-parity-roadmap.md. Preserve o
worktree. O cursor da TUI continua reprovado no teste humano; não ajuste y.
Comece por R0: fixture PTY + métricas/overlay sob flag, compare com terminal
externo e só então implemente R1. Code OSS/xterm.js são a base de paridade
comportamental, mas a solução permanece nativa Rust + IPC + Qt. Não gere
AppImage ou push antes do gate e do aceite explícito. Um checkpoint local pode
registrar este estado adiado depois de gate verde, desde que não o descreva
como paridade visual concluída.
```

# PONTO ATUAL — fila viva do Kinein Vectis (2026-07-15)

> Este arquivo contém somente trabalho presente ou futuro, em ordem de
> execução. Trabalho concluído deve ser registrado no documento do domínio e
> em `ContextoIA.md`, e então removido daqui.
>
> Estado implementado: `ContextoIA.md` + docs numerados + código. Mapa de
> conhecimento e arquivos conectados: `GUIAIA.md`. Histórico de checkpoints:
> Git. Base remota atual: `928fbb5`; checkpoint anterior: `948535d`;
> checkpoint funcional atual: `HEAD` local; protocolo `0.58.0`.
>
> Não alterar a UI fora das specs. Commits locais de checkpoint após marco
> crítico/teste verde foram autorizados em 2026-07-15; push e publicação não
> foram. **Dogfooding/self-hosting ativo desde 2026-07-14:** o usuário já
> está na Kinein, e cada bloqueio ou saída para outra IDE passa a ordenar o
> backlog antes de funcionalidade nova.

## 0. Dogfooding ativo

O gatilho **“estou no Kinein”** já foi recebido. A primeira regressão concreta
é o KV Context não se comportar visualmente como um terminal profissional:
faltavam scrollback/barra perceptível no Codex, largura adequada e fluidez no
resize. Após o primeiro reinício, surgiu um segundo detalhe: a linha de
digitação inline ficava sem limite visual entre o aviso de usage e o status do
modelo. No segundo reinício foram reportados quatro sintomas adicionais: a
faixa não envolvia os glifos, o divisor livre desaparecia durante a sessão, a
árvore `Project` era fechada e o scroll se perdia em chats longos. A correção
0.52 tratou esses quatro sem criar input ou terminal paralelo. Após o aceite do
dimensionamento, o feedback restante ficou restrito à faixa da entrada
multilinha e à roda do mouse; a correção pós-0.52 de 2026-07-15 aguarda o gesto
real apenas nesses dois pontos, conforme `REENTRADA-KV`. O usuário pausou esse
gesto e autorizou expressamente a continuação do roadmap. A1 e A2 foram
entregues nos protocolos 0.53.0 e 0.55.0. A tentativa posterior de demarcar a
entrada foi removida: grade VT, spans e cursor voltaram a ser a única fonte
visual, no modelo terminal-first. Antes do início efetivo de A3, o dogfooding
revelou que o caret ficava muito depois do texto e que verde/bold do prompt era
visualmente agressivo. A correção 0.56 unifica spans e cursor pela largura VT,
usa família monoespaçada real e suaviza peso/paleta sem interpretar o prompt;
o usuário aprovou posição horizontal, Terminal comum e verdes suaves. Restou
somente um ajuste fino: no KV Context o caret de Claude/Codex não parecia
centralizado. O teste exclusivo com `-2` ficou perto, mas o usuário pediu valor
`0`, igual ao Terminal puro aprovado. A pesquisa seguinte mostrou que as TUIs
usam o cursor nativo do terminal e podem solicitar forma/piscagem por DECSCUSR,
estado que o render anterior descartava. A correção 0.57 preserva esse estado
genericamente. O usuário esclareceu em seguida que KV Context é somente outra
apresentação da mesma base de terminal, não um backend diferente; por isso todo
offset por superfície foi removido e `DefaultUserShape` também usa a célula VT
integral. O usuário abriu a Kinein por `scripts/kinein-vectis` e não percebeu
mudança relevante: o caret da TUI ainda parece desalinhado. Portanto 0.57 fecha
a lacuna de forma/piscagem, mas não a causa visual. Esse polimento foi adiado
sem novo offset; sua retomada começa por reprodução instrumentada e métricas de
célula/DPR em `docs/roadmaps/26-terminal-rendering-parity-roadmap.md`. Por decisão do
usuário, a experiência funcional do terminal do Code OSS é a base de paridade,
traduzida para Rust + IPC + Qt, sem Electron, Node, WebView ou xterm.js.

1. registrar cada problema observado pelo usuário ou por um testador com ação,
   esperado, resultado, reprodução, distro e log quando houver;
2. corrigir primeiro perda de dados, crash, corrupção, falha de abertura/build
   ou bloqueio que force a saída para outra IDE;
3. depois tratar regressões funcionais e atritos reproduzíveis de uso diário;
4. quando não houver feedback bloqueador e o usuário mandar prosseguir no
   roadmap, iniciar **A3 — responsividade medida**; A1 e A2 foram entregues;
5. continuar pelas etapas deste arquivo e pelos docs existentes, sem inventar
   outro remake, subsistema paralelo ou roadmap substituto.

O dogfooding não autoriza push, publicação, mudança de visibilidade, envio
externo ou implementação aleatória fora da fila. Commits locais são feitos
somente nos checkpoints verdes já autorizados. O repositório-fonte continua
privado.

### 0.1 Protocolo econômico de reentrada após reiniciar a própria Kinein

Reiniciar a IDE encerra a sessão de IA que está rodando dentro dela. Para não
gastar outra conversa reconstruindo contexto, o handoff deve ficar neste
arquivo antes de fechar. Na sessão nova, o usuário precisa escrever somente:

```text
Leia AGENTS.md e retome pelo marcador REENTRADA-KV em PONTO_ATUAL.md. Siga as
leituras obrigatórias silenciosamente, não resuma o histórico e não refaça o
que já está validado.
```

A sessão nova deve executar esta sequência:

1. ler `AGENTS.md` e a documentação obrigatória indicada nele, sem devolver
   uma recapitulação extensa ao usuário;
2. localizar primeiro o marcador `REENTRADA-KV` abaixo e confrontá-lo com
   `ContextoIA.md` + código apenas onde houver divergência;
3. rodar `git status --short` para preservar o worktree existente; arquivos
   presentes não autorizam limpeza/descarte. Commit local só depois do gate
   verde do marco; push continua sem autorização;
4. não repetir investigação, implementação, gate ou build já registrados como
   verdes, a menos que o código tenha mudado depois do marcador ou apareça
   evidência concreta de regressão;
5. retomar diretamente pela ação `PRÓXIMO GESTO`; responder inicialmente com
   uma frase curta de orientação, não com outro plano ou resumo;
6. depois do gesto real, registrar o resultado no marcador: se falhar,
   ação/esperado/observado/ambiente viram a prioridade do dogfooding; se passar,
   marcar o aceite e seguir a próxima fatia desta fila somente quando o usuário
   mandar prosseguir.

#### REENTRADA-KV — estado exato para a próxima reentrada

```text
ESTADO
- Protocolo atual 0.58.0 (`aiCliFlatTranscript` em `settings.*`; 0.57 fechou
  DECSCUSR). A1 (recentes) e A2 (capacidades Cargo+CMake) estão
  implementadas; `workspace.kind` é primário compatível e
  `workspace.capabilities.buildSystems` é a fonte das ações híbridas.
- Codex abre com argumento fixo --no-alt-screen; Claude permanece sem argumento.
- KV ativo reutiliza TerminalPanel/TerminalManager, tem barra persistente,
  roda/arrasto, teclado/paste VT, largura livre persistida 300–720px e
  ampliar/restaurar; Project permanece independente fora da maximização.
- O bridge preserva o transcript contra CSI 3 J ainda emitido por versões do
  Codex; o Terminal comum continua honrando clear.
- Não existe guia, faixa ou input paralelo: grade VT, spans ANSI e cursor são
  a única representação da entrada, seguindo comportamento terminal-first.
- Cada span informa a largura autoritativa em células VT; fonte, resize,
  seleção e caret usam a mesma grade. ANSI bold usa peso médio e a paleta verde
  é suave; o shell continua dono do texto e dos atributos do prompt.
- Não existe mais propriedade de offset vertical no AssistantPanel,
  TerminalPanel ou TerminalViewport. Forma e piscagem solicitadas por DECSCUSR
  seguem no render; reset/default é resolvido no core para `bar`. Barra/bloco
  usam a célula VT integral, underline usa a base e `steady` não pisca. Não há
  regra por agente ou superfície.
- O teste humano pelo script de desenvolvimento não percebeu correção do
  alinhamento vertical. O problema continua aberto e foi adiado; 0.57 não tem
  aceite visual. A retomada está integralmente especificada em `docs/roadmaps/26` e
  começa por fixture PTY, overlay de baseline/célula e DPR, não por offset.
- Code OSS/xterm.js são a base de paridade comportamental autorizada para o
  terminal. A implementação continua nativa em `portable-pty` + Rust + IPC +
  Qt; Electron, Node, WebView, Extension Host e runtime xterm.js não entram.
- A roda aceita os dois formatos do Qt (`angleDelta` e `pixelDelta`) e segue o
  mesmo `terminal.scroll` do Terminal comum.
- Gutter usa faixas independentes para folding/breakpoint, diagnóstico, blame,
  diff e números medidos por FontMetrics; marcador não invade o número.
- Semantic tokens carregam path+version e respostas obsoletas são descartadas;
  Tree-sitter permanece fallback estrutural instantâneo.
- Os cinco SVGs de árvore fornecidos foram integrados sem alterar seus bytes.
- Scripts shell reconhecidos têm ação de execução na árvore; o core confina o
  caminho e usa argv explícito, sem interpolação.
- Packaging corrigido para cache host/container isolado, mounts Podman/SELinux
  e invocação por bash. A entrada direta delega ao builder Debian auditado; o
  worker não é um build nativo. `dist/` só recebe por staging o conjunto
  completo AppImage/checksum/instalador/Tutorial, preservando a entrega anterior
  em caso de falha.
- Barra da janela agora é client-side: `Main.qml` usa `FramelessWindowHint` e a
  App Bar hospeda Minimizar, alternável Maximizar/Restaurar e Fechar
  (`WindowControls.qml`), guiados pelo `QWindow` via `WindowChromeController`
  (`ui/src/window_chrome_controller.*`). Região livre arrasta/duplo-clica;
  `WindowResizeHandles.qml` cobre as oito bordas. ACEITO pelo usuário em
  Fedora/Wayland (2026-07-15); regressão P2 encerrada. Ver `docs/roadmaps/20` §barra.

VALIDAÇÃO JÁ FEITA — NÃO REPETIR SEM MUDANÇA DE CÓDIGO
- `scripts/verificar.sh` completo: verde; binários release do atalho de
  desenvolvimento atualizados.
- Testes Rust, clippy -D warnings, C++/QML estritos e harnesses QML: verdes.
- Build Clang debug strict e `scripts/verificar-cpp.sh`: verdes.
- `scripts/verificar-qml.sh`: verde usando o response file do build strict
  atualizado; o antigo `build/dev-local` não tem precedência.
- tst_assistant_layout cobre divisor ativo/largura/Project; tst_terminal_scroll
  cobre nova saída, snap, troca de sessão, roda tradicional e `pixelDelta`;
  Rust cobre CSI 3 J entre chunks.
- AppImage final (33.737.208 bytes; SHA256 `86b335b2b1ba8c81d958df4f1e45f7d9c0838fdad2a2567c84199f84b8dbdc0d`),
  teste host e Debian mínimo sem rede: verdes. Ambos validam também instalador
  executado fora da pasta, `.desktop`, PNG e `Tutorial.md` idêntico à fonte.
- Correção 0.56: gate integral verde com 335 testes Rust, Clippy, C++/QML
  estritos, 12 harnesses, builds Debug/Release e smoke offscreen de 8 s
  (`exit 124` esperado). Binários do atalho de desenvolvimento atualizados.
- O teste anterior `cursorVerticalOffset: -2` exclusivo do KV Context passou
  no gate, mas não recebeu aceite visual. O novo valor `0` passou em qmllint,
  12 harnesses, rebuild Release e smoke offscreen de 8 s; o gesto humano
  posterior não aprovou o cursor e `dist/` não foi regenerado.
- Primeiro recorte DECSCUSR 0.57: gate integral verde com 337 testes Rust,
  Clippy, C++/QML, 12 harnesses, builds e smoke. O feedback posterior removeu
  todo offset e unificou `DefaultUserShape`; o mesmo gate integral passou
  novamente e o smoke release ficou vivo por 8 s sem saída (`exit 124`).
  Binários de desenvolvimento atualizados; `dist/` continua intocado.
- Gesto humano posterior: aberta por `scripts/kinein-vectis`, a versão não
  apresentou mudança visual relevante para o usuário. O caret de Claude/Codex
  continua sem aceite. Automação verde não equivale a correção visual.
- A auditoria seguinte encontrou `target/release/kinein-core` anterior a
  `terminal.rs`; o launcher podia combinar UI nova com core antigo. O gate foi
  refeito com `debug-strict`/`release-hardened`: 337 testes Rust, Clippy,
  C++/QML, 12 harnesses e os builds passaram. Smoke real pelo
  `scripts/kinein-vectis` ficou vivo por 8 s sem saída (`exit 124`). UI/core
  release agora são os binários atuais; hashes e caminhos estão em
  `ContextoIA.md`. `dist/` permaneceu intocado.
- A baseline release A3 com `N=3` passou os orçamentos: 250 ms primeiro frame,
  103 MB UI, 3,4 ms workspace, 0,0 ms leitura 10k e 7 MB core. A expansão
  A3.1–A3.4 está detalhada abaixo; Code OSS/Zed e resultados estão em docs/roadmaps/21.
- Controles de janela (barra client-side): `scripts/verificar.sh` integral verde
  + smoke offscreen debug/release (`exit 124`); ACEITO pelo usuário em
  Fedora/Wayland. AppImage 0.1.0 regenerado e testado com este código.

PRÓXIMO GESTO
1. Os controles de janela estão aceitos e já embarcados no AppImage novo de
   `dist/`. Não reabrir essa fatia; o polimento P3 (snap/escala/multimonitor) de
   `docs/roadmaps/20` é opcional e não bloqueia o roadmap.
2. **A3.1 FEITA em 2026-07-16** (`docs/roadmaps/21` §A3.1). Fixture Rust
   determinística de 2463 linhas; frio 323 ms, incremental 281 ms, payload
   1138 KB. **Achado que sobra para A3.4:** o ganho do parse incremental
   evapora com o tamanho (6,2x em 208 linhas → 1,04x em 4923) e o custo cresce
   superlinearmente. A resposta é 20x o fonte e carrega highlights+outline do
   arquivo inteiro a cada tecla — o dono do custo não é o parser. Não otimizar
   antes de A3.4 e de perfil local confirmar. Seguir para A3.2.
3. Encerrada A3, auditar EditorConfig como primeira integração pequena
   recomendada. Não iniciar um host genérico de plugins.
4. Dogfooding em tempo integral: o usuário saiu do CLion e passou a usar a
   Kinein para ganhar o log da aba IDE como vantagem de desenvolvimento. Cada
   atrito ou saída para outra ferramenta vira o topo do backlog
   (ação/esperado/observado/ambiente), na frente de A3.
5. Se o usuário retomar o cursor/TUI, voltar por R0 de `docs/roadmaps/26`, nunca por
   offset.

RESULTADO PENDENTE
- Cursor/TUI reprovado no gesto humano e adiado; causa visual ainda aberta.
- R0–R7 detalhados em `docs/roadmaps/26-terminal-rendering-parity-roadmap.md`.
- O AppImage 0.1.0 de 2026-07-15 foi regenerado a pedido do usuário para
  embarcar a barra client-side aceita; ele carrega o estado atual do cursor/TUI,
  que o usuário optou por não deixar bloquear a entrega. Não regerar o AppImage
  *por causa do cursor* antes do aceite visual dele.
- Se qualquer gesto falhar, registrar ação/esperado/observado/ambiente e
  priorizar a regressão antes de A3.

LIMITES
- Commit local somente após checkpoint verde; não fazer push/publicação.
- Não reabrir a discussão de chat embutido: KV Context é terminal dedicado.
- Fidelidade ao Code OSS significa comportamento adaptado à arquitetura da
  Kinein; não incorporar Electron/Node/xterm.js nem copiar implementação.
```

### Loop de desenvolvimento a partir do dogfooding

```text
feedback real (autor ou testador)
        ↓
reprodução e causa-raiz
        ↓
teste/harness de regressão quando aplicável
        ↓
correção pequena na camada dona
        ↓
gate + gesto real
        ↓
docs sincronizadas e retorno ao uso
```

Se vários feedbacks chegarem juntos, usar esta prioridade:

```text
P0  perda/corrupção de dados, segurança, crash ou IDE não abre
P1  bloqueio de edição, build, run, debug, terminal, Git ou navegação
P2  comportamento incorreto/repetível que prejudica o fluxo diário
P3  conforto, polimento ou funcionalidade nova
```

Feedback de testador não vira feature automaticamente: reproduzir, conferir se
já existe solução no core/UI e encaixar no domínio/roadmap correto. Se for uma
ideia nova sem bloqueio, registrar atrás dos problemas reais e de A3.

### 0.2 Controles de janela integrados (P2 — ACEITO em 2026-07-15)

O dogfooding do AppImage em Fedora/Wayland havia revelado que **Minimizar**,
**Maximizar** e **Restaurar** não estavam visíveis. A pedido explícito do
usuário, os controles foram integrados à barra da própria IDE (estilo JetBrains,
sem copiar): `Main.qml` usa `Qt.Window | Qt.FramelessWindowHint`,
`WindowControls.qml` traz Minimizar + alternável Maximizar/Restaurar + Fechar
guiados pelo estado real do `QWindow` via `WindowChromeController`, a região
livre arrasta/duplo-clica e `WindowResizeHandles.qml` cobre as oito bordas.
Detalhes e referência profissional (IntelliJ IDEA Community `e3b4dba`) em
`docs/roadmaps/20` e `ContextoIA.md`.

**Aceito:** o usuário testou em Fedora/Wayland e confirmou que funciona. A
regressão P2 está encerrada. Um AppImage 0.1.0 novo foi gerado com este código e
entregue em `dist/`. O polimento P3 restante de `docs/roadmaps/20` (snap, escala
fracionária, multimonitor e acabamento visual) segue como fatia própria, não
bloqueante; a imagem de referência é `imagens/bugs/ReformularBarra.png`.

### 0.2b aiBridge REMOVIDO e KV Context desabilitado (2026-07-16, protocolo 0.59.0)

**O `aiBridge.*` era a interferência e foi removido.** Ele abria `claude`/`codex`
por um caminho especial: argumentos injetados pelo core (`--no-alt-screen`,
`--ax-screen-reader`) e um filtro que engolia `CSI 3 J` da própria aplicação.
Isso é a IDE se metendo entre o programa e o terminal — o que nenhuma IDE
profissional faz, e o que fazia o agente se comportar diferente dentro e fora
da Kinein.

Uma CLI de IA agora é **um programa como outro qualquer**: abrir o terminal e
rodar `claude`. Mesmo PTY, mesmo emulador, mesmo contrato de um `ls`.

Removidos: `handlers/ai.rs`, `protocol/ai.rs`, `tests/ai.rs`, a superfície
`ui/qml/assistant/*`, o `AiBridgeEventRouter`, o atalho do rail, o estado do
shell, as settings `aiCliProfile`/`aiCliFlatTranscript`, e — no `terminal.rs` —
o `ScrollbackPreserver` e o `open_command_with_policy`. Não existe mais política
por programa no terminal.

**KV Context volta como UI pura:** um atalho visual que abre uma sessão de
terminal comum, para desacoplar visualmente do uso padrão, **sem regra de
negócio no core**. A UI representa o backend; não o define. Depende do terminal
consolidado (ADR-0004 já trocou o emulador para o `alacritty_terminal`, motor do
Zed; falta encaminhamento de mouse ao app e a decisão de renderer).

O modelo de IA não mudou: externa, por CLI do usuário, sem chat embutido e sem
rede pela IDE. Caiu o **mecanismo**, não o princípio.

### 0.2c KV Context reativado como UI pura (2026-07-16)

A dependência registrada em §0.2b ("depende do terminal consolidado") caiu: a
roda ao aplicativo está feita (protocolo 0.60.0) e a grade foi corrigida e
aceita. O atalho voltou, e agora ele é o que sempre deveria ter sido.

`Exibir → KV Context` (`view.context`) abre uma sessão de terminal **comum**,
rotulada `KV Context N`, para a sessão do agente não se perder entre os terminais
de build. Se já existe uma viva, foca ela em vez de acumular aba.

**Não existe regra de negócio em camada nenhuma.** O core não sabe o que é "KV
Context": para ele é o mesmo `terminal.open` do Alt+F12, um `$SHELL` no PTY. O
rótulo é estado de UI (`pendingContext`/`contextSeq` no `RuntimeController`) e
não é parâmetro do protocolo — se um dia virar, a política por programa que o
0.59.0 removeu voltou. Quem roda `claude`/`codex` é o usuário, digitando.

Detalhe de robustez: `terminal.open` pode falhar (teto de 12 sessões) e o erro
vai para o handler genérico do `CoreClient`, sem chegar ao QML. Sem tratamento a
marca ficaria presa e a próxima aba comum nasceria rotulada "KV Context"; um
timeout de 4 s a solta.

O atalho tem **duas entradas**: `Exibir → KV Context` e o ícone dedicado no
`SideRail` (o `context` do `KvIcon` sobreviveu ao 0.59.0; só o botão tinha
sido arrancado junto com o painel do assistente). O ícone acende conforme a
aba ATIVA do terminal ser de contexto — não há painel próprio para alternar.

Cobertura: `tst_multi_terminal.qml` (rótulo, marca consumida, aba comum não
herda, estado aceso segue a aba ativa, foco em vez de acumular, numeração não
repete, inerte sem workspace).

Fio solto que isso fechou: o item de menu "KV Context" existia desde o 0.59.0
apontando para uma ação `view.context` que **não existia** — opção morta na barra.

Pendência aberta: `assistantTerminalWidth` sobreviveu à remoção do painel do
assistente em três camadas (`SettingsController.qml`, `settings.rs`, schema). É
setting órfã. Decidir: ou o KV Context passa a usar largura persistida, ou sai.

**Opção B (a UI digitar o comando do agente) segue em aberto** e é preocupação
válida do autor: hoje o atalho abre a aba e o usuário digita `claude`. Subir para
a B exige que o comando seja **configurável**, senão é a regra por programa
apenas migrando de camada — o core deixaria de conhecer "claude" e a UI passaria
a conhecer. Analisar em fatia própria.

### 0.2d Backlog levantado pelo autor em 2026-07-16 (pontuado, não implementado)

**1. KV Context: confirmar em tela.** O autor reportou que o item aparece em
`Exibir` e não funciona. Essa é exatamente a descrição do estado **anterior** aos
commits de hoje: até `034b773` o item apontava para uma ação `view.context`
inexistente. Depois de `034b773` (menu) e `19eef85` (ícone no rail) o wiring foi
verificado — `runtimeController` chega ao `ShellHeaderHost` e ao
`ShellWorkspaceHost`, e ambos chamam `openContext()`. **Falta o gesto humano no
build novo.** Se continuar morto, é regressão real e tem prioridade: registrar
ação/esperado/observado/ambiente.

**2. Renomear "KV Context" (P3, decisão do autor).** O nome não explica o que a
coisa é. Direção sugerida pelo autor: algo como "Agente Auxiliar". A renomeação
é de PRODUTO e atinge: rótulo do menu (`AppMenuBar`), tooltip do rail
(`SideRail`), título da aba (`RuntimeController.handleTerminalOpened`), o nome do
ícone `context` no `KvIcon`, `MANUAL.md` e a spec
`docs/specs/KINEIN_VECTIS_AI_CLI_BRIDGE_EXTERNAL_TERMINAL.md`. Os identificadores
internos (`view.context`, `openContext`, `isContext`) podem acompanhar ou não —
decidir de uma vez para não ficar meio renomeado. Nada disso toca o core: ele não
conhece o conceito.

**3. Autocomplete travado na primeira sugestão (P2, dogfooding).** Relato do
autor: a sugestão do LSP fica presa no primeiro item; não dá para selecionar
outra opção além da que aparece primeiro. Se confirmado, é bloqueio de uso diário
e passa na frente de polimento. Suspeitos: navegação por seta no
`EditorCompletionController` (`index`) versus quem consome a tecla antes —
`EditorPane`/`Keys.onPressed` da superfície. Reproduzir primeiro; provavelmente é
tecla capturada por outra camada, não o modelo de completion.

**4. Ícones no app (ver §6): 158 dos 163 SVGs não estão na IDE.** Pré-requisito
do AppImage "completo" que o autor quer distribuir.

**5. AppImage para testadores.** Plano do autor: fechar A3.1–A3.4 (L0), L1, a UI
do KV Context e os ícones, e então gerar um AppImage completo para distribuir.
Não gerar antes disso; `dist/` só recebe conjunto completo por staging.

**6. Código vai ser open source — codar pensando nisso.** Observação do autor de
que há muito "comentário de IA" no projeto. Isso é uma **varredura própria**, na
mesma família da varredura de camada: comentário que narra a sessão ("a IA deve",
"nesta fatia", "o usuário pediu") não é documentação técnica e não sobrevive à
publicação. A política de tom já existe em `PLANO_ORGANIZACAO_E_HANDOFF.md` §6,
mas ela cobria `.md` — falta aplicá-la a **comentário de código**. Regra a partir
de agora: comentário explica invariante e causa, não processo nem autoria. Fatia
própria, depois da trilha atual.

### 0.3 Sessão de organização e feedback (2026-07-16)

Sessão de trabalho autônoma autorizada pelo autor (com backup; proibido git
destrutivo/histórico). Entregas e feedback:

- **Reorganização da documentação (concluída, sem commit):** os `.md` técnicos
  foram agrupados por assunto sob `docs/{arquitetura,build,seguranca,roadmaps}`
  via `git mv` (nomes preservados), 410 referências raiz-relativas reescritas em
  77 arquivos (docs + comentários de código + scripts + CMake), `docs/README.md`
  reescrito como índice e 0 links markdown quebrados. `cargo check` verde.
  Raiz intacta: `README/MANUAL/Tutorial/AGENTS` e os pessoais. Plano completo
  (faixa pessoal, não publicar): `PLANO_ORGANIZACAO_E_HANDOFF.md`.
- **Scroll do agente Claude no KV Context — RESOLVIDO como toggle (protocolo
  0.58.0):** a causa-raiz era o Claude interativo usar **tela alternada** (sem
  scrollback por semântica VT), sem flag inline como o `--no-alt-screen` do
  Codex. O teste ao vivo confirmou: com `--ax-screen-reader` o scroll e o cursor
  funcionam, mas a TUI decorativa vira texto puro. Por isso virou preferência:
  setting **`aiCliFlatTranscript`** (default `false` = TUI decorativa), exposta
  em Configurações como "KV Context: histórico navegável", lida no
  `aiBridge.terminal.open` e válida na próxima sessão. `ProfileSpec` ganhou
  `flat_args`; Codex mantém `--no-alt-screen` nos dois modos (já tem TUI **e**
  histórico). Contrato/manual/schema atualizados; 3 testes novos de perfil.
  Detalhes em `docs/roadmaps/26` §11. Opção A (encaminhar roda ao app em
  alt-screen — corrige vim/htop também) segue no backlog §6.
- **"Modo imagem" para `.md` (backlog, ver seção 6):** preview de markdown
  renderizado no editor, estilo JetBrains, reusando `TextEdit.MarkdownText` já
  usado pelo `DocumentationDialog`.

## 1. TR0 — aceite funcional da rodada atual

Antes de ampliar o produto, validar a aplicação real em tela. Regressões
encontradas aqui têm prioridade e não autorizam outro remake visual.

### 1.1 Checklist de validação manual

- **Autocomplete:** a primeira sugestão estrutural aparece sem esperar o LSP;
  a resposta semântica de clangd/rust-analyzer substitui o fallback quando
  estiver pronta, sem popup duplicado ou salto de seleção.
- **Terminal sob rajada:** saída contínua acompanha a linha atual sem atrasos;
  a barra aparece assim que existe histórico; rolar ou arrastar preserva a
  leitura; nova entrada do usuário volta ao final; múltiplas abas permanecem
  independentes.
- **Menus Arquivo–Ajuda:** todas as opções ficam acima do editor, legíveis e
  acionáveis. `Ajuda → Manual da IDE` abre a documentação interna.
- **Criação no projeto:** menu Arquivo e clique direito oferecem adicionar
  arquivo/pasta e reutilizam o fluxo confinado ao workspace.
- **KV Context:** Claude/Codex instalados pelo usuário são descobertos; a
  escolha abre uma sessão própria sobre o terminal real; Codex preserva
  scrollback em modo inline; barra/roda/arrasto, teclado, seleção, copiar/colar
  e resize se comportam como no Terminal integrado, inclusive durante nova
  saída; a sessão tem largura livre/persistida, coexiste com `Project` e pode
  ser ampliada sem acoplar o painel ao terminal comum.
- **Estrutura e painéis:** a aba Estrutura redimensiona, recolhe e restaura;
  o layout inicial se adapta à janela sem cobrir editor ou menus.
- **Barras e tooltips:** editor, terminal e listas longas mostram posição e
  permitem arrastar; descrições de ícones nunca aparecem sob o editor.

### 1.2 Critério de saída do TR0

- checklist acima aceito em uma sessão real;
- qualquer falha reproduzível ganhou teste/harness quando aplicável;
- `bash scripts/verificar.sh` verde;
- aceite visual R7/C6 confrontado com as specs, sem mudança estética lateral.

O início do dogfooding não precisa esperar uma cerimônia separada de TR0: a
primeira sessão dentro da Kinein deve percorrer este checklist naturalmente.
Falhas encontradas nela interrompem a próxima fatia do roadmap até a regressão
correspondente ficar corrigida e protegida.

## 2. TR1 — distribuição e substituição de editores generalistas

### A3 — responsividade medida

**Estado em 2026-07-15:** preparada sobre a infraestrutura M4.2 já existente;
não criar segundo runner, protocolo de telemetria ou framework de benchmark.
`scripts/medir-performance.sh` + `scripts/medir-core.py` continuam sendo a
entrada única, com cenários nomeados, mediana de `N` e saída local.

Baseline release revalidada depois do protocolo 0.57, com `N=3` e os binários
exatos usados por `scripts/kinein-vectis`:

| Métrica existente | Mediana atual | Orçamento vigente |
| --- | ---: | ---: |
| primeiro frame offscreen | 250 ms | 400 ms |
| UI RSS vazia | 103 MB | 200 MB |
| `workspace.open` no repo | 3,4 ms | 50 ms |
| `fs.read` de 10 mil linhas | 0,0 ms | 20 ms |
| core RSS em regime | 7 MB | 60 MB |
| rust-analyzer externo | 1093 MB | informativo |

Todos os itens com orçamento passaram. O rust-analyzer continua separado do
RSS próprio da Kinein e não reprova o gate sem cenário/limite específico.

Referências profissionais atuais para a expansão de A3:

- Code OSS `234638618394269563dd77c0c395c270d8df8b12`,
  `src/vs/base/common/performance.ts` e
  `src/vs/workbench/services/timer/browser/timerService.ts`, MIT/MODE-B:
  marcos nomeados, durações derivadas entre marcos, espera explícita por fases
  prontas e separação entre custo próprio, ambiente e processos externos;
- Zed `1e22d1a83f8b1b7acc528d15cfab0644852380c0`,
  `crates/benchmarks/benches/editor_render.rs` e `display_map.rs`, somente
  referência MODE-D: fixtures com seed fixa, tamanhos de entrada explícitos,
  amostras repetidas e benchmark do caminho real de input/render.

Adaptação nativa: usar `std::time::Instant`/`QElapsedTimer`, RPC stdio e fixtures
locais; não incorporar timer, telemetria, runtime ou código das referências.

#### A3.1 — estrutura local Tree-sitter

1. estender `medir-core.py`, sem RPC novo, para medir separadamente:
   - primeiro `syntaxTree.update` frio em arquivo Rust/C++ real;
   - atualização incremental de um caractere no mesmo documento;
   - contagem/validação mínima do snapshot para impedir número rápido vazio;
2. usar fixture versionada e tamanho explícito; não depender de rede ou LSP;
3. registrar mediana e orçamento inicial em `docs/roadmaps/21`.

Aceite: `syntax_first_snapshot_ms` e `syntax_incremental_update_ms` aparecem na
mesma tabela local, com cenário reproduzível e resultado estrutural não vazio.

#### A3.2 — primeira semântica e estabilização LSP

1. medir separadamente clangd e rust-analyzer quando instalados;
2. iniciar em workspace conhecido, sincronizar documento e medir até a primeira
   resposta válida de `lsp.semanticTokens` e `lsp.completion`;
3. distinguir startup/indexação externa do round-trip da Kinein; ausência da
   ferramenta produz `n/d` explícito, nunca sucesso falso;
4. aplicar timeout e encerrar todos os filhos ao final da amostra.

Aceite: primeira resposta, resposta aquecida e RSS externo ficam separados; a
medição valida `path`/`version` e ao menos um token/item quando o cenário exigir.

#### A3.3 — digitação real e rajada do terminal

1. criar um harness Qt opt-in que marque tecla recebida → frame apresentado em
   arquivo grande, sem rodar no uso normal;
2. medir mediana e cauda visível (`p95`) porque travadas de digitação podem
   desaparecer na mediana;
3. reutilizar a sonda PTY existente para uma rajada determinística, medindo
   `terminal.input` → frame contendo marcador final, além de perda de input,
   scrollback e responsividade durante a saída;
4. Terminal comum e KV Context usam o mesmo cenário/renderer; o cursor adiado
   de `docs/roadmaps/26` não altera esta medição.

Aceite: nenhuma tecla perdida, marcador final presente, UI interativa durante
a rajada e números separados para editor e terminal.

#### A3.4 — orçamento e reação a regressões

1. versionar máquina, distro, Qt, binários, `N`, fixture e perfil release;
2. orçamento inicial = medição repetida com folga explícita, não número
   aspiracional inventado;
3. regressão acima do orçamento abre fatia de causa-raiz antes de nova
   profundidade semântica;
4. otimização só entra depois de perfil local mostrar o dono do custo; trabalho
   pesado permanece cancelável/assíncrono e fora da thread da UI.

Aceite: métricas e cenários ficam versionados; não se depende apenas de
impressão visual para afirmar que autocomplete/editor/terminal são responsivos.

Depois de A3, a primeira integração pequena recomendada para a sessão de
“novo plugin” é **EditorConfig**: está no P0 do roadmap aberto, serve C/C++ e
Rust, melhora dogfooding imediatamente e cabe em uma fatia auditável sem host
de extensões. Isso é recomendação de ordem, não adoção definitiva; a sessão
deve confirmar biblioteca/licença, contrato e conflito com settings antes do
código. Integrações grandes de A5 continuam atrás dessa análise.

### A4 — confortos que bloquearem o dogfooding

Prioridade inicial, ajustada pelos motivos reais de saída para outro editor:

1. split editor;
2. multicursor;
3. EditorConfig;
4. zoom do editor;
5. links e busca no scrollback do terminal;
6. duplo clique para selecionar palavra no terminal.

Aceite do TR1: uma semana de desenvolvimento C/C++ e Rust sem abrir editor
generalista auxiliar. Quando o usuário disser **“estou no Kinein”**, registrar
cada exceção e corrigir primeiro o bloqueio reproduzível. Feedback dos
testadores entra no mesmo funil, identificado pela origem e pelo ambiente, sem
substituir evidência de reprodução.

### A5 — candidatos explícitos para análise futura de integrações

Lista solicitada pelo usuário em 2026-07-15. **Não é decisão de adoção nem fila
de implementação imediata**: alguns itens já estão presentes ou registrados,
outros podem ser substituídos por opção mais adequada. A sessão própria deve
confrontar manutenção atual, licença, segurança, compatibilidade Linux-first,
duplicação do que existe e encaixe Qt/QML → IPC → Rust Core antes de escolher:

1. **Open Remote SSH (Open VSX/comunidade):** já consta no roadmap de adaptação
   como referência FOSS. Reavaliar UX de abrir pasta, arquivos e sessão remotos
   sem executar extensão VS Code/VSCodium dentro da Kinein nem depender de
   servidor proprietário.
2. **SSHFS / sshfs-win por CLI:** estudar montagem de Raspberry Pi, satélite ou
   host embarcado como árvore local. Comparar com OpenSSH direto e considerar
   latência, desconexão, watcher, escrita atômica e o fato de `sshfs-win` não
   ser a variante primária de uma IDE Linux-first.
3. **Bear (Build EAR):** avaliar geração de `compile_commands.json` ao
   interceptar builds C/C++ quando CMake File API/presets ou banco de compilação
   nativo do projeto não estiverem disponíveis; não substituir CMake nem
   clangd.
4. **CodeLLDB:** já é referência explícita no roadmap e a Kinein já possui
   fundação DAP com `lldb-dap`. Analisar apenas lacunas concretas de experiência
   e embarcados, sem incorporar o host de extensão.
5. **libssh / ssh2-rs:** candidatos nativos para sessão, túnel e SFTP. Comparar
   custo de dependência, superfície de segurança e manutenção com a política
   atual de orquestrar OpenSSH CLI antes de escolher biblioteca de binding.
6. **Valgrind / Memcheck:** avaliar integração de análise de vazamentos e
   acessos inválidos C/C++ como job cancelável, com parser no core e resultados
   navegáveis na UI.
7. **Heaptrack:** avaliar profiling de memória C/C++ e Rust e visualização de
   resultados sem criar profiler próprio; medir custo e disponibilidade nas
   distribuições suportadas.
8. **Clippy:** já está implementado no strict gate e em `quality.run` para
   Rust. A análise futura deve tratar somente lacunas de UX/diagnósticos, não
   adicionar outro linter equivalente.
9. **Sigrok / PulseView:** avaliar captura e decodificação de sinais como I2C,
   SPI e UART e uma visualização integrada para analisadores lógicos. Preferir
   orquestrar o motor/protocolo aberto e não recriar decoders ou osciloscópio.
10. **Serial Studio:** avaliar telemetria serial em tempo real e painéis de
    gráficos, mapas, bússolas e medidores. Comparar integração externa,
    formatos de dados e custo de uma UI nativa antes de qualquer adoção.
11. **Wokwi CLI / QEMU:** QEMU já pertence à trilha embarcada; acrescentar
    Wokwi como candidato para simulação sem placa de ESP32/STM32 e afins.
    Verificar licença, dependência de serviço/rede, reprodutibilidade e
    adequação à política local-first antes de escolher.
12. **Unity / Google Test (GTest) / Criterion:** estudar descoberta e consumo
    de relatórios de testes C/C++ no painel existente. Comparar com CTest já
    suportado; para Rust, preservar `cargo test` como fonte autoritativa.
13. **Doxygen:** candidato para geração acionável de documentação C/C++/Rust a
    partir do projeto, como job externo e cancelável, sem gerador próprio.
14. **Sphinx / Breathe:** avaliar composição de manuais sobre a saída do
    Doxygen e geração HTML/PDF, mantendo templates e configuração pertencentes
    ao projeto do usuário.
15. **Bloaty McBloatface:** avaliar análise de tamanho de binários e seções para
    firmware, com resultados navegáveis e visualizações de consumo de flash;
    não criar analisador binário interno.
16. **GitOxide (gix) / libgit2:** candidatos nativos para Git. A Kinein hoje já
    oferece status, diff, blame, histórico, branches, stash e operações remotas
    orquestrando Git no core; comparar segurança, paridade e custo antes de
    substituir uma CLI madura ou duplicar funcionalidades existentes.
17. **DAP / lldb-dap / gdb-dap:** DAP e `lldb-dap` já formam a fundação de
    debug da Kinein. Avaliar `gdb-dap` e lacunas de hardware sem criar outro
    protocolo nem interpretar a saída humana de GDB/LLDB.
18. **LSIF:** avaliar índices persistentes para navegação em bases C/C++ muito
    grandes e comparar com formatos/ecossistemas atuais antes de escolher. Não
    duplicar clangd nem iniciar indexador global sem orçamento medido.
19. **OpenOCD:** candidato principal para flash e debug JTAG/SWD, orquestrado
    como processo confinado e observável. Separar transporte GDB, comandos de
    controle e diagnóstico; Telnet/RPC entram na mesma análise, sem comandos
    montados por shell.
20. **pyOCD:** candidato Cortex-M complementar ao OpenOCD, especialmente para
    automação e probes CMSIS-DAP; comparar cobertura de targets, distribuição,
    dependência Python e paridade antes de ativar por placa.
21. **CMSIS-DAP:** protocolo/firmware de probe ARM, não plugin de UI. Modelar
    como capacidade detectada do adaptador físico e fonte para OpenOCD/pyOCD.
22. **avrdude / esptool / stlink:** candidatos específicos para flash AVR,
    ESP e STM32. Cada ferramenta precisa de adapter tipado, detecção de versão,
    preview do comando, cancelamento e logs; nada de executor shell genérico.
23. **Cppcheck / LLVM Clang Static Analyzer:** candidatos C/C++ para
    `quality.run`, comparados com clang-tidy já adotado. Resultados devem virar
    diagnósticos comuns e evitar três análises equivalentes por padrão.
24. **cargo-audit / cargo-deny:** candidatos Rust para vulnerabilidades,
    advisories, fontes e licenças. Diferenciar análise local do lockfile de
    atualização de bancos pela rede e exigir ação/consentimento explícitos.
25. **gcov / lcov:** candidatos de cobertura C/C++ com build instrumentado,
    coleta separada e relatório por arquivo/linha; preservar presets e targets
    do projeto, sem injetar flags silenciosamente.
26. **cargo-tarpaulin:** candidato de cobertura Rust, a comparar com alternativas
    atuais, compatibilidade de toolchain e custo Linux. A fonte de testes segue
    sendo `cargo test`.
27. **Ghidra:** candidato de engenharia reversa e decompilação como ferramenta
    externa pesada. Avaliar importação/exportação e navegação ELF/assembly sem
    embutir toda a suíte ou prometer edição round-trip.
28. **GDB/LLDB MI:** fallback estruturado apenas para capacidades de baixo nível
    realmente ausentes no DAP. Preferir DAP; qualquer MI precisa de parser de
    protocolo, máquina de estados, timeout e testes, nunca scraping do terminal.
29. **libelf / goblin:** candidatos para ELF, seções, símbolos e mapas de
    memória. Comparar biblioteca nativa/FFI com crate Rust segura e pequena;
    não duplicar Bloaty quando um relatório externo bastar.
30. **DWARF / gimli:** padrão e biblioteca candidata para mapear endereços a
    fontes/símbolos quando DAP não fornecer o dado. Exigir limites de memória,
    parsing lazy e fixtures de ELF reais.
31. **udevadm / libudev:** candidatos Linux para detectar probes e placas por
    VID/PID. O core deve observar eventos e sugerir configuração; nunca iniciar
    flash automaticamente sem confirmação do usuário.
32. **PlatformIO Storage Architecture / sysroots isolados:** estudar somente a
    estratégia de manifests, pins, checksums, isolamento e cache, sem incorporar
    PlatformIO Core ou baixar toolchains silenciosamente. Instalação, rede,
    licença, proveniência, rollback e espaço em disco exigem política própria.
33. **CMSIS-Pack:** candidato para descrição ARM, SVD, startup, drivers e mapa
    de memória. Tratar `.pack` como conteúdo não confiável, validar manifestos,
    caminhos e licenças e nunca executar código do pacote durante a leitura.
34. **DTS / DTB:** candidato para navegação, validação e compilação de Device
    Tree via ferramentas maduras. Qualquer editor gráfico deve preservar o
    texto autoritativo e round-trip verificável.
35. **SWO / ITM:** candidatos de trace não intrusivo ARM, integrados ao
    transporte/probe selecionado. Separar aquisição de alta taxa, decode,
    backpressure, persistência e visualização.
36. **CTF / LTTng:** candidatos para trace estruturado e timelines, sobretudo
    Linux/RTOS. Avaliar leitores maduros e streaming limitado antes de criar
    armazenamento ou gráficos próprios.
37. **libclang / Clang C API:** candidato explícito do usuário para AST e
    análises profundas/call graphs. A sessão futura deve medir custo de memória,
    ABI/distribuição e duplicação com clangd + Tree-sitter. Só adotar quando uma
    capacidade concreta não puder ser obtida pelos serviços existentes; o peso
    maior é aceito apenas com benefício e orçamento demonstrados.
38. **gcov-kernel / kcov:** candidatos especializados para cobertura de kernel
    Linux e drivers, fora do fluxo padrão de usuário; exigem privilégios,
    isolamento e documentação de ambiente próprios.
39. **MQTT / CoAP / Mosquitto:** candidatos para telemetria de bancada remota.
    Diferenciar cliente, broker e protocolo; rede permanece opt-in, com destino
    visível, TLS/credenciais protegidos e nenhuma telemetria da IDE.
40. **TimescaleDB / InfluxDB / LMDB / RocksDB:** candidatos distintos para
    séries temporais ou armazenamento local de captura. Bancos-servidor não são
    equivalentes a motores embutidos; escolher somente após medir volume,
    retenção, consulta, operação e portabilidade.
41. **Jinja2 / Tera:** candidatos de template para geração explícita de código
    de inicialização. Preferir Tera se a camada permanecer Rust, mas só após
    definir arquivos gerados, ownership, preview, diff, regeneração e proteção
    contra sobrescrever edição manual.
42. **OpenModelica / OMSimulator:** candidatos para simulação física
    mecânica/elétrica/térmica por CLI/API. Avaliar licença, distribuição,
    execução longa cancelável, artefatos e importação de resultados sem acoplar
    o core ao runtime da suíte.
43. **FMI / FMUs:** padrão candidato para co-simulação. Tratar FMU como pacote
    de binário não confiável: inspeção, compatibilidade de arquitetura,
    sandbox, limites e consentimento precedem qualquer carregamento.
44. **GSL / ndarray:** candidatos de computação numérica para ferramentas ou
    fixtures científicas, não APIs automaticamente expostas ao código do
    usuário. Definir primeiro o caso de uso e evitar transformar a IDE em um
    runtime matemático próprio.
45. **perf / Hotspot:** candidatos Linux para profiling e flame graphs. Preferir
    produzir/consumir formatos maduros, detectar permissões do kernel e manter
    coleta separada da visualização.
46. **tokio-console:** candidato Rust para tarefas assíncronas instrumentadas.
    A integração deve detectar a instrumentação necessária e apresentar
    claramente que não funciona em binários Tokio não preparados.
47. **Frama-C / Kani:** candidatos de verificação formal para C e Rust. Entram
    como jobs especializados, opt-in e reprodutíveis; não devem ser rotulados
    como prova de segurança total nem misturados ao lint rápido padrão.
48. **OTAWA / análise WCET:** candidato de pior tempo de execução para sistemas
    críticos. Avaliar maturidade, arquiteturas suportadas, modelos de hardware,
    licença e validade das premissas antes de qualquer promessa de certificação.
49. **lm-sensors / Open Hardware Monitor CLI:** candidatos de observação do host
    durante simulação/profiling. Para Linux-first, avaliar `lm-sensors` primeiro;
    correlação energética não autoriza coleta persistente ou telemetria padrão.
50. **D-Bus:** API de integração Linux para eventos realmente necessários de
    sistema/energia/rede. Assinar apenas interfaces allowlisted; não usar como
    barramento genérico nem monitorar o desktop inteiro.
51. **ccache / sccache:** candidatos de cache de compilação, ativados por
    toolchain/preset explícito e diagnóstico de hit/miss. Não reescrever linhas
    de compilação nem enviar cache remoto sem configuração e consentimento.
52. **distcc:** candidato de compilação distribuída C/C++ para laboratórios.
    Exige confiança entre hosts, toolchains idênticas, rede configurada pelo
    usuário, falha segura e medição que justifique a complexidade.
53. **OpenGL e tecnologias gráficas afins:** trilha de rendering para simulação
    gráfica Linux-native citada pelo usuário. Tratar como subsistema de
    visualização/simulação com API e isolamento próprios, não como plugin de
    linguagem nem lógica embutida no editor.

#### A5.1 — arquitetura futura da biblioteca de integrações

O usuário quer uma aba visual semelhante a uma biblioteca, capaz de mostrar e
ativar/desativar integrações. O nome de produto pode ser “Plugins”, mas a
arquitetura deve registrar a natureza real de cada item: ferramenta externa,
protocolo, formato, biblioteca vinculada, serviço ou visualizador. Desenhar em
sessão própria antes de implementar. A arquitetura-base desta sessão fica
fechada assim:

- registry tipado no Rust Core com id estável, categoria, versão detectada,
  licença, origem, capacidades, requisitos, conflitos e estado de saúde;
- ativação global ou por workspace persistida por schema, com defaults mínimos,
  lazy start, orçamento de CPU/RAM e desligamento/cancelamento determinísticos;
- adapters pequenos por integração atrás de contratos de domínio; a UI lista,
  configura e solicita ações, mas não inicia processos nem carrega plugins;
- dependências, permissões, USB/rede/privilégios e downloads sempre visíveis e
  confirmáveis; pins, checksums, rollback e diagnóstico para pacotes geridos;
- nenhum extension host genérico, marketplace executável, código remoto ou
  telemetria. Ativar/desativar não pode alterar arquivos/toolchains do projeto
  silenciosamente;
- separar aquisição, processamento, armazenamento e visualização para sinais,
  trace, profiling e simulação; aplicar backpressure e limites desde o início;
- estudar Code OSS, IntelliJ Community, Zed/Lapce/NetBeans e ferramentas
  oficiais pertinentes por revisão atual, registrando invariantes e adaptação
  Qt/QML → IPC → Rust Core antes de cada fatia.

O nome visível pode ser **Plugins**, mas os nomes internos serão
`integration`/`capability`/`adapter`. Não haverá API para carregar código de
terceiros. O crescimento é incremental, sem criar um crate ou framework vazio:

```text
QML Plugins/Integrations view (apresenta e solicita)
        ↓ CoreClient
kinein-protocol::integration (descriptor, state, action, permission)
        ↓
kinein-core/src/integration/ (registry + policy + health)
        ↓
adapter pequeno do domínio → Job cancelável → ferramenta/protocolo externo
        ↓
eventos tipados → Problems/Tests/Profiler/Trace/Simulation
```

O primeiro recorte nasce junto de EditorConfig e registra também as ferramentas
já detectadas; só depois o módulo é dividido. Um descriptor precisa carregar
`id`, natureza, categoria, modo A–D, origem/licença, versão detectada,
capacidades, requisitos, permissões, instalação, escopo de ativação e saúde.
Uma ação carrega risco, efeitos, necessidade de rede/USB/privilégio, preview e
tipo de Job. Estado e configuração usam schema; segredo fica fora do registry.

#### A5.2 — sequência linear vinculante de capacidade

Cada nível depende dos anteriores. Não começar uma integração de nível alto
apenas porque sua CLI é fácil de chamar.

| Nível | Entrega arquitetural primeiro | Integrações que validam o nível |
| --- | --- | --- |
| L0 | fechar A3.1–A3.4 e orçamentos; sem plataforma genérica antes da baseline | nenhuma nova |
| L1 | `integration` v1 por fatia vertical: descriptor/health/configuração, leitura do registry existente e aba somente informativa | EditorConfig; inventário de clangd, rust-analyzer, CMake, Cargo, Git, rg, fd, lldb-dap e Clippy |
| L2 | resultado comum para diagnóstico/teste/cobertura, artefatos e Jobs canceláveis | cargo-audit/deny, Cppcheck/Clang Static Analyzer, Valgrind, GTest/Unity/Criterion, gcov/lcov e cobertura Rust |
| L3 | Project Graph/targets/perfis explicáveis, cache provenance e geração de artefatos | Bear, ccache/sccache, Bloaty, Doxygen e Sphinx/Breathe |
| L4 | DAP sólido, sessão de profiling, importador de relatório e permissões do kernel | lldb/gdb DAP, Heaptrack, perf/Hotspot, tokio-console; MI/ELF/DWARF apenas por lacuna |
| L5 | `RemoteContext`: host keys, credenciais externas, path mapping, desconexão, sync e Jobs remotos | OpenSSH/Open Remote UX; SSHFS e bindings SSH somente como alternativas; distcc não entra ainda |
| L6 | `Target`/`Device`/`Probe`, detecção USB, package trust, flash preview e confirmação | udev, CMSIS-DAP/Pack, DTS/DTB, OpenOCD, pyOCD, probe-rs, avrdude, esptool, stlink, QEMU e Tera |
| L7 | streaming com backpressure, timestamp, canais, retenção e segurança de rede/dispositivo | sigrok, SWO/ITM, CTF/LTTng, MQTT/CoAP/Mosquitto, lm-sensors e D-Bus allowlisted |
| L8 | armazenamento medido e API de visualização isolada do editor/core | banco local ou de séries temporais escolhido por benchmark, OpenGL/Qt rendering e computação numérica específica |
| L9 | sandbox de pacote/binário, compatibilidade de arquitetura e co-simulação reproduzível | FMI/FMU antes de OpenModelica/OMSimulator; Wokwi somente opt-in externo/nuvem |
| L10 | laboratório opt-in, sem promessa de suporte diário | SCIP/indexação persistente, Ghidra, libclang, gcov-kernel/kcov, Frama-C/Kani, OTAWA e distcc |

Gate de promoção de nível: ao menos uma integração vertical real, testes de
falha/cancelamento, orçamento medido, configuração reversível e nenhum
processo/handle órfão. A aba visual não desbloqueia o nível; o contrato e o
serviço comprovados o desbloqueiam.

#### A5.3 — decisão linear para os 53 candidatos

Legenda: **manter** = já existe; **adotar** = candidato principal após o gate;
**condicional** = só com lacuna/PoC; **referência** = estudar ou interoperar,
sem incorporar; **substituir** = avaliar primeiro a alternativa indicada.

| # | Candidato | Decisão | Ordem e limite |
| ---: | --- | --- | --- |
| 1 | Open Remote SSH | referência | L5; UX de referência, implementação por OpenSSH direto |
| 2 | SSHFS / sshfs-win | condicional | L5 depois do RemoteContext; nunca transporte padrão; `sshfs-win` fora do foco Linux |
| 3 | Bear | adotar | L3, somente fallback quando o build não fornece compile database confiável |
| 4 | CodeLLDB | manter/referência | L4; `lldb-dap` via DAP já é a integração, sem Extension Host |
| 5 | libssh / ssh2-rs | condicional | L5, apenas se OpenSSH CLI não cobrir sessão/SFTP/túnel necessário |
| 6 | Valgrind/Memcheck | adotar | L2, Job opt-in e achados navegáveis; não bloquear lint rápido |
| 7 | Heaptrack | adotar | L4, coleta separada de visualização e importação de artefato |
| 8 | Clippy | manter | L1/L2, melhorar UX de diagnósticos; não integrar segundo Clippy |
| 9 | sigrok/PulseView | adotar/referência | L7; motor/CLI e formatos em Mode-A, PulseView pode abrir externamente |
| 10 | Serial Studio | referência | L7; interoperabilidade/formato apenas, sem copiar GPLv3 nem módulos Pro |
| 11 | Wokwi CLI / QEMU | separar | QEMU em L6 local-first; Wokwi em L9, externo, token/rede/upload explícitos |
| 12 | Unity/GTest/Criterion | adotar protocolos | L2; descobrir/executar via CTest/Cargo e consumir relatórios, sem empacotar frameworks |
| 13 | Doxygen | adotar | L3 como Job de projeto explícito |
| 14 | Sphinx/Breathe | condicional | L3 depois de Doxygen, para projetos que já escolheram essa cadeia |
| 15 | Bloaty | adotar | L3, análise de artefato/flash sem parser binário próprio |
| 16 | gix/libgit2 | não agora | L10 apenas se Git CLI demonstrar lacuna; evitar duplicação e FFI |
| 17 | DAP/lldb-dap/gdb-dap | manter/adotar | L4; consolidar DAP/lldb e avaliar `gdb.dap`; uma sessão/contrato comum |
| 18 | LSIF | substituir | L10; avaliar SCIP atual, não iniciar implementação nova em LSIF |
| 19 | OpenOCD | adotar | L6, principal adapter JTAG/SWD/GDB remoto |
| 20 | pyOCD | adotar condicional | L6, complementar Cortex-M/CMSIS-DAP por cobertura de target |
| 21 | CMSIS-DAP | modelar | L6 como capacidade de probe, não plugin/UI |
| 22 | avrdude/esptool/stlink | adotar | L6, adapters distintos com argv/preview/versão/cancelamento |
| 23 | Cppcheck/Clang Static Analyzer | condicional | L2; no máximo um default além de clang-tidy, sem diagnósticos triplicados |
| 24 | cargo-audit/cargo-deny | adotar | L2; banco/rede separados da análise local e consentimento explícito |
| 25 | gcov/lcov | adotar | L2; perfil instrumentado explícito e resultado por arquivo/linha |
| 26 | cargo-tarpaulin | substituir primeiro | L2; avaliar `cargo-llvm-cov` primeiro; Tarpaulin fica fallback validado |
| 27 | Ghidra | referência/externo | L10; abrir/importar artefato, nunca embutir a suíte |
| 28 | GDB/LLDB MI | fallback | L4 depois de DAP, somente capacidade comprovadamente ausente |
| 29 | libelf/goblin | condicional | L4/L10; preferir crate Rust segura (`goblin`) se Bloaty/DAP não bastarem |
| 30 | DWARF/gimli | condicional | L4/L10; parsing lazy e limitado somente para contrato concreto |
| 31 | udevadm/libudev | adotar em degraus | L6; `udevadm`/sysfs primeiro, binding apenas se eventos persistentes exigirem |
| 32 | PlatformIO Storage Architecture | referência | L6; aprender pins/cache/isolamento, sem incorporar Core nem download silencioso |
| 33 | CMSIS-Pack | adotar | L6; parser de pacote não confiável, licença e paths validados, zero execução |
| 34 | DTS/DTB | adotar | L6; ferramentas oficiais e texto autoritativo com round-trip |
| 35 | SWO/ITM | adotar | L7 depois de Probe/Target e streaming com backpressure |
| 36 | CTF/LTTng | adotar condicional | L7 para Linux/RTOS, formatos maduros e retenção limitada |
| 37 | libclang/Clang C API | não agora | L10; somente caso medido que clangd + Tree-sitter não resolvam |
| 38 | gcov-kernel/kcov | laboratório | L10, privilégios/isolamento próprios e fora do fluxo padrão |
| 39 | MQTT/CoAP/Mosquitto | adotar opt-in | L7; cliente separado do broker, TLS/segredos e destino visível |
| 40 | TimescaleDB/InfluxDB/LMDB/RocksDB | escolher, não somar | L8 após benchmark de volume/retenção; servidor e embutido são decisões distintas |
| 41 | Jinja2/Tera | condicional | L6; preferir Tera no Rust, com preview/diff/ownership antes de gerar |
| 42 | OpenModelica/OMSimulator | adotar tarde | L9 depois do contrato FMI/FMU, como Job externo cancelável |
| 43 | FMI/FMUs | adotar fundação | L9; inspecionar/sandbox antes de carregar binário de FMU |
| 44 | GSL/ndarray | por feature | L8/L9; preferir `ndarray` em módulo Rust quando um visualizador provar necessidade |
| 45 | perf/Hotspot | adotar/referência | L4; `perf` coleta, Hotspot abre/importa externamente antes de UI própria |
| 46 | tokio-console | condicional | L4; só para binários instrumentados e com pré-requisito visível |
| 47 | Frama-C/Kani | laboratório | L10, Jobs separados e sem alegação genérica de prova de segurança |
| 48 | OTAWA/WCET | pesquisa | L10, somente após maturidade/arquitetura/modelo de hardware comprovados |
| 49 | lm-sensors/Open Hardware Monitor | substituir | L7; usar lm-sensors no Linux; não depender de Mono/WinForms do OHM |
| 50 | D-Bus | condicional de plataforma | L7, interfaces allowlisted; não aparece como plugin genérico |
| 51 | ccache/sccache | adotar | L3, ativação explícita por perfil e telemetria de hit/miss só local |
| 52 | distcc | laboratório | L10 depois de RemoteContext, confiança/toolchain/rede explícitas e benchmark |
| 53 | OpenGL/afins | subsistema, não plugin | L8; renderer isolado, API de dados limitada e compatível com o stack Qt |

#### A5.4 — correções trazidas pela auditoria atual

- [SSHFS](https://github.com/libfuse/sshfs) 3.7.6 tem release de 2026, mas o
  próprio projeto informa ausência de contribuidores ativos regulares e
  manutenção focada em problemas de alto impacto; por isso não é a base remota.
- [Wokwi CLI](https://docs.wokwi.com/wokwi-ci/cli-usage) usa token; a
  [arquitetura oficial](https://docs.wokwi.com/wokwi-ci/getting-started)
  executa a simulação em nuvem e recebe o firmware. Só pode ser opt-in com
  preview/envio explícito, nunca substituto local do QEMU.
- [Serial Studio](https://github.com/Serial-Studio/Serial-Studio) separa core
  GPL-3.0 e recursos Pro proprietários; é referência/interoperabilidade, não
  fonte de código nem componente redistribuído pela Kinein.
- Sourcegraph recomenda [SCIP no lugar de
  LSIF](https://sourcegraph.com/blog/announcing-scip), e em 2026 anunciou
  governança aberta para SCIP; qualquer índice persistente novo começa pela
  avaliação de SCIP.
- Para Rust, [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov)
  trabalha sobre instrumentação LLVM, Cargo/nextest e múltiplas arquiteturas;
  o Tarpaulin ainda usa ptrace x86_64 como backend Linux padrão. Por isso
  `cargo-llvm-cov` é o primeiro PoC.
- Open Hardware Monitor exige Mono/WinForms no Linux; `lm-sensors` mantém a
  decisão Linux-first mais simples e nativa.

Esta triagem não dispensa o gate completo de licença, revisão, checksum,
segurança e manutenção imediatamente antes de cada adoção. Ela decide a ordem
e evita gastar arquitetura em candidatos já superados ou inadequados.

### A6 — sessão reservada para o novo repositório público

O usuário pretende abrir uma sessão própria em breve para organizar o projeto
inteiro antes de criar um **novo** repositório público no GitHub. `.gitignore`
não é barreira de publicação nem remove nomes/histórico já rastreados. Aplicar
`docs/roadmaps/21` M7.3: exportação por allowlist para árvore separada, histórico novo,
dry-run, rejeição de Markdown extra e auditoria de segredos. No início dessa
sessão, obter do usuário a lista exata dos nomes de arquivos `.md` e diretórios
que não podem aparecer nem como caminho. Até lá, não criar repositório, não
alterar visibilidade e não fazer push.

## 3. Consolidações necessárias antes de declarar substituição diária

| Frente | Consolidação pendente | Evidência de aceite |
| --- | --- | --- |
| Distribuição | ampliar matriz Ubuntu/Fedora, canal de release, atualização e diagnóstico de runtime | AppImage validado em distros-alvo e procedimento de release repetível |
| Entrada no trabalho | validar workspaces recentes e restauração previsível em uso prolongado | retomar projeto em um clique, sem sessão cruzada |
| Modelo de projeto | aprofundar capacidades já detectadas em Project Graph/Context Matrix | targets e contextos Cargo/CMake/Qt explicáveis por arquivo |
| Editor | resposta imediata local + semântica progressiva | cenários e latências medidos |
| Terminal | rajadas, scrollback e múltiplas sessões prolongadas | teste de estresse sem atraso ou perda de interação |
| Build/Run/Test | fluxos reais e cancelamento em projetos externos | processos encerram sem órfãos e resultados são navegáveis |
| Debug | sessão básica confiável e inspeção útil | breakpoint, step, pilha e variáveis em fixture real |
| Segurança de dados | soak tests de save, mudança externa, crash e drafts | nenhuma escrita silenciosa sobre snapshot antigo |
| Ergonomia | confortos puxados pelo dogfooding | nenhum editor auxiliar necessário por lacuna diária |

## 4. TR2 — primeiro patamar “um nível abaixo do CLion”

Implementar o KSWE em fatias pequenas, mantendo CMake/Cargo/clangd/
rust-analyzer como fontes autoritativas e sem criar uma engine genérica
paralela.

### B1 — Project Model autoritativo

- ampliar CMake File API para codemodel, targets, configurações, sources,
  compile groups, includes, defines e artefatos;
- consumir Cargo Metadata para packages, targets, features e workspace;
- normalizar ambos em snapshot versionado de Project Graph + Context Matrix;
- atualizar por geração, descartar resultado obsoleto e respeitar orçamento;
- criar fixtures CMake, Cargo e híbrida.

### B2 — targets, perfis e toolchains como entidades

- seleção explícita de target/configuração/toolchain;
- contexto efetivo por arquivo e target;
- presets CMake, perfis Cargo, sysroot e compile database rastreáveis;
- interface visual fiel às Configuration Actions das specs.

### B3 — inteligência semântica coordenada

- scheduler LSP com prioridade ao arquivo visível e cancelamento;
- Symbol Broker e Diagnostic Broker sem duplicar clangd/rust-analyzer;
- Effective Compile Context explicável ao usuário;
- caches limitados e invalidação por geração/fingerprint;
- fallback Tree-sitter continua instantâneo e independente do LSP.

### B4 — debug de IDE

- watches/expressões, variáveis, pilha, breakpoints condicionais e
  pretty-printers;
- GDB/LLDB via DAP, sem UI chamar debugger diretamente;
- sessões reais C/C++ e Rust com encerramento/cancelamento confiável.

Aceite do TR2: desenvolver a Kinein por uma semana sem abrir CLion por falta de
compreensão do projeto, build, navegação semântica ou debug básico.

## 5. TR3 — profundidade equiparável e embarcados

- múltiplos targets/contextos concorrentes e cache semântico persistente;
- correlação de símbolos, diagnósticos e refatorações mais profundas;
- toolchains cruzadas, sysroots e SDKs Yocto/Buildroot;
- flash, serial, QEMU, OpenOCD/pyOCD e GDB remoto;
- testes prolongados em projetos C, C++, Rust e embarcados reais;
- otimização de CPU/RAM/latência sem transformar indexação em trabalho eager.

## 6. Backlog complementar, depois das consolidações imediatas

- opção guiada **Outra IA**: abrir uma sessão de terminal dedicada e instruir
  o usuário a digitar o comando de inicialização da CLI que já instalou;
- biblioteca de funções / Configuration Actions para configurar CMake/Cargo e
  ambiente com preview, evidência e controle do usuário;
- exportador allowlist para qualquer cópia do código entregue a terceiros, com
  `--dry-run`, auditoria de segredos e recusa de Markdown além de `README.md`,
  `MANUAL.md` e `Tutorial.md`; sem `.git/`/histórico privado e com o
  repositório-fonte permanecendo privado;
- **preview de markdown renderizado ("modo imagem") no editor**, estilo
  JetBrains: alternar entre fonte e visualização renderizada de arquivos `.md`.
  Reusar `TextEdit.MarkdownText` (já validado no `DocumentationDialog`); fatia de
  UI com toggle por aba/atalho, sem editar o markdown pela visualização.
  Pedido do autor em 2026-07-16.
- **remake de UI/UX do shell, com o IntelliJ IDEA Community como layout-base.**
  Pedido do autor em 2026-07-16. Não é fatia pequena e não deve ser iniciada de
  improviso: exige spec antes de código, como a barra da janela (`docs/roadmaps/20`).

  Sintomas concretos já relatados:

  1. **A barra de "Target: host local", perfil, Compilar e Run não tem bordas
     arredondadas**, ao contrário dos demais painéis (o `SideRail` usa
     `Theme.radiusLarge`, a faixa da toolbar não). Inconsistência visível entre
     superfícies vizinhas.
  2. O conjunto App Bar + toolbar + rail + painéis cresceu por acréscimo e não
     por um sistema; falta uma regra única de raio, gap, elevação e densidade.

  Referência profissional autorizada: **IntelliJ IDEA Community** (política do
  `AGENTS.md`/roadmap de adaptação — estudar revisão atual, registrar
  invariantes, adaptar para Qt/QML **sem copiar** código ou tradução mecânica).
  Extrair o *layout base*: hierarquia tool window / editor / status bar, a faixa
  de ações, e como a densidade e o raio são sistematizados. Cruzar com
  `docs/specs/KINEIN_VECTIS_LAYOUT_SYSTEM.md` e
  `KINEIN_VECTIS_UI_COMPONENTS_SYSTEM.md`, que já são a visão-alvo — o remake
  deve convergir para as specs, não inventar uma terceira direção.

  Imagem de referência do autor para a barra: `imagens/bugs/ReformularBarra.png`.
  O polimento P3 da barra da janela (`docs/roadmaps/20`: snap, escala
  fracionária, multimonitor) pertence a esta mesma frente.

  Não bloqueia L0/L1. Entra quando o autor priorizar.
- **integrar os pacotes de ícones ao app (não é fatia de packaging).** Pedido do
  autor em 2026-07-16: "os ícones atuais e os novos devem estar no AppImage".

  Auditoria do estado real: **isso já é automático e não pode ser esquecido.**
  Ícones entram pelo `RESOURCES` do `qt_add_qml_module` em `ui/CMakeLists.txt`,
  são compilados no executável, e o `empacotar-appimage.sh` empacota o
  executável. Não existe passo separado de copiar ícone para o AppImage — o que
  está no `RESOURCES` está no AppImage por construção.

  O gap real é outro: **de 163 SVGs em `docs/iconografia/`, apenas 5 estão no
  app** (`ui/assets/icons/tree/`: folder-closed/open, file-c/cpp/rust). O pacote
  de ícones de arquivos especiais foi commitado em `docs/`, que é design, não
  asset. Então a fatia é de **UI**, não de empacotamento:

  1. escolher quais famílias entram (o índice `docs/iconografia/README.md` diz
     qual é a fonte de verdade de cada uma) e copiar os SVGs para `ui/assets/`
     preservando os bytes;
  2. declarar em `RESOURCES` no `ui/CMakeLists.txt`;
  3. mapear extensão/tipo → ícone (existe `FILE_ICON_MAPPINGS.json` no pacote de
     arquivos especiais) e consumir na árvore;
  4. o AppImage passa a carregá-los sem nenhuma mudança de packaging; validar com
     `testar-appimage.sh` e `testar-appimage-portatil.sh`.

  Regra permanente: ícone novo entra em `RESOURCES` no mesmo commit em que entra
  na UI. Se está no `RESOURCES`, está no AppImage.
- **varredura de lógica de negócio na UI/UX — EXECUTADA em 2026-07-16.**
  Pedido do autor após a fatia da roda. Os dois defeitos daquele dia (`handleWheel`
  decidindo o gesto e o `Column` decidindo a geometria da grade) foram o **mesmo
  erro duas vezes**: a UI decidindo o que é do backend. A varredura procurou por
  (a) tradução de gesto/tecla em semântica de domínio, (b) política/validação que
  o core deveria impor, (c) heurística sobre estado do backend, (d) montagem de
  comando/caminho e (e) regra por programa/ferramenta.

  **Achado 1 — RESOLVIDO em 2026-07-16 (protocolo 0.61.0).** Era: a UI decidia
  o que é formatável, duplicando o core.
  `ui/qml/editor/EditorController.qml`:

  ```qml
  function formattableLanguage() {          // linha 310
      return language === "rust" || language === "cpp";
  }
  function formattablePath(path) {          // linha 327
      return lower.endsWith(".rs") || lower.endsWith(".c") || ... // 9 extensões
  }
  ```

  O core **já é a autoridade**: `format::formatter_for_path()` decide e o handler
  recusa com `InvalidParams` ("nenhum formatter registrado para esta extensão").
  A UI mantinha uma segunda lista, escrita à mão, que divergia por construção:
  `formattableLanguage` conhecia 2 linguagens, `formattablePath` conhecia 9
  extensões, e as duas nem concordavam entre si. Adicionar linguagem exigia
  editar QML.

  Correção: `format.capabilities` publica o catálogo, derivado da MESMA
  constante (`FORMATTER_EXTENSIONS`) que `formatter_for_path` usa para decidir —
  o que a UI recebe é, por construção, o que o `format.text` aceita. A UI perdeu
  as duas listas e passou a ter uma pergunta só, respondida pelo core, pedida uma
  vez por conexão (o catálogo é estático). Capacidade e disponibilidade seguem
  distintas: binário ausente no `PATH` continua sendo `TOOL_NOT_FOUND` do
  `format.text`.

  Cobertura: 3 testes Rust (catálogo publicado == decisão real, extensão fora do
  catálogo, sem sobreposição entre formatters) e
  `scripts/qml-harness/tst_format_capabilities.qml` — que falha se alguém
  reintroduzir lista literal no QML, porque testa que **nada** é formatável antes
  de o catálogo chegar.

  **Achado 2 — P3, latente: encoding VT mora na UI.**
  `TerminalInputController.qml` traduz tecla em bytes (`\x1b[A` vs `\x1bOA`
  conforme `applicationCursor`) e `TerminalPanel.qml` embrulha o paste
  (`\x1b[200~`) conforme `bracketedPaste`. É o **mesmo formato** do bug da roda —
  encoding VT no frontend —, mas **funciona**, porque ao contrário do
  `handleWheel` estas duas LEEM o modo que o core publica. É escolha registrada
  ("a UI continua burra em relação ao TUI: apenas respeita o estado VT"). Fica
  como dívida consciente: qualquer modo novo que afete encoding (kitty keyboard
  protocol, `modifyOtherKeys`) vai exigir mudança na UI em vez de só no core.
  Não corrigir sem necessidade concreta.

  **Achado 3 — P3, trivial: `alternateScreen` é código morto.**
  `TerminalPanel.qml:41` expõe a propriedade e **ninguém a consome** desde que a
  decisão da roda foi para o core. Era o dado que o `handleWheel` ignorava.

  **Achado 4 — P3: `assistantTerminalWidth` órfã** em `SettingsController.qml`,
  `settings.rs` e no schema — resto do painel do assistente removido no 0.59.0.
  Ver §0.2c.

  **Limpo:** não há montagem de comando/caminho de ferramenta na UI (o `+ "/"` do
  `EditorDocumentController`/`ProjectTreeController` é composição de caminho para
  exibição, sobre dados que o core já confinou); não há allowlist nem validação de
  segurança no frontend; não há heurística adivinhando estado do backend; os
  nomes de ferramenta no QML (`"cargo"`, `"cmake"`, `"git"`) são rótulo, chave de
  aba ou parâmetro repassado ao core — não decisão.

  Conclusão: a camada está mais saudável do que os dois defeitos do dia sugeriam.
  O padrão perigoso é específico — **UI decidindo semântica de terminal/ferramenta
  em vez de ler o contrato** — e sobrou concentrado no Achado 1. Cada achado vira
  fatia própria; a varredura não mudou comportamento.
- **FEITO (aguarda seu gesto visual) — emulador VT migrado para
  `alacritty_terminal`** (`docs/adr/ADR-0004-alacritty-terminal-emulator.md`).
  Causa-raiz era entregar um VT raso (`vt100`) enquanto anunciávamos
  `xterm-256color` — por isso Claude/Codex funcionam fora e quebravam aqui.
  Agora o motor é o mesmo do Alacritty/Zed. Contrato `event.terminal.render`
  **inalterado**: UI, harnesses e sonda não mudaram. 261 testes verdes, clippy
  estrito limpo, sonda e2e verde (scrollbackMax, eco, clamp, multi-sessão).
  `vt100`/`vte` saíram das deps; o `CursorStyleTracker` manual e o
  `scrollback_capacity` foram deletados (o emulador expõe os dois). **Teste com
  Claude/Codex reais e me diga se o cursor/scroll ficaram certos.**
- Contexto da decisão (`docs/roadmaps/26` §4.5). Pedido real do autor: "meu
  terminal inteiro dentro da IDE". Claude/Codex
  funcionam no terminal do sistema e só quebram na Kinein, logo a variável é o
  emulador. A crate `vt100` é rasa; o candidato é **`alacritty_terminal`** (Rust,
  Apache-2.0, mantido, mesmo ecossistema do `vte` já linkado — é o que o Zed usa;
  Code OSS usa xterm.js e IntelliJ usa JediTerm pelo mesmo motivo). Substitui só
  o motor de grade/estado VT, preservando `portable-pty`, IPC tipado e renderer
  Qt/QML. Exige ADR, auditoria/pin, mapeamento de `event.terminal.render` e
  testes de paridade. Deve vir ANTES da decisão de renderer (R3) e tende a
  resolver de uma vez cursor flutuante, tela alternada, mouse e scroll de TUIs.
- **ACEITO em 2026-07-16 — cursor/TUI corrigido. Causa-raiz: linha vazia
  colapsava no positioner.** O `Column` do Qt Quick descarta filhos de largura
  zero; uma linha vazia chega do core como `[]` e vira um `Row` sem filhos, logo
  sem largura. Cada linha vazia sumia do layout e todo o texto abaixo subia uma
  linha, enquanto o cursor ficava na posição correta da grade. **O cursor nunca
  esteve errado — o texto escorregava para cima.** Correção: a linha é
  posicionada por `metrics.yForRow(index)`, a mesma métrica do cursor; o
  `Column` saiu. Detalhes e evidência em `docs/roadmaps/26` §4.7. O usuário
  testou e aprovou. A pendência de aceite visual do ADR-0004 está encerrada.

  O que destravou foi a **captura de tela** — item 3 do R0, tratado como
  opcional por três fatias. As medições numéricas (R0 §4.6) e o R1 confirmaram
  hipóteses reais (H1, H2) que não eram a causa: o erro era de linha inteira, não
  de sub-pixel. Lição registrada no §4.7 do roadmap.
- **FEITO (aguarda seu gesto) — roda do mouse encaminhada ao aplicativo**
  (fatia R4, protocolo `0.60.0`, `docs/roadmaps/26` §11.4 Opção A). A causa não
  era interferência: era **ausência**. O `open_command` não tem política por
  programa nenhuma; o que faltava é que a UI decidia sozinha que roda = rolar
  histórico, sempre. O Codex rolava (desenha inline, tem histórico na grade) e o
  Claude não (alt-screen, sem histórico por semântica VT).

  Medido nas CLIs reais, não suposto: `claude` liga `?1049 ?1000 ?1002 ?1003
  ?1006` — alt-screen **e captura de mouse SGR**; `codex` não liga nenhum. Logo o
  conserto do Claude é **relatório de mouse**, não `alternate scroll` (ele não
  pede `?1007`). O desenho que eu ia escrever primeiro teria sido inerte.

  `terminal.mouse` carrega o contrato de mouse completo; só `wheel` está
  implementado, `press`/`release`/`motion` ficam fixados e recusados
  explicitamente (R5). A decisão virou `wheel_action(mode, shift)` no core — a
  regra que estava no QML, agora num lugar só. Referências: `scroll_wheel` do
  Zed, `MouseStateService` do xterm.js, `JediTerminal.mouseReport` do JediTerm
  (cujas constantes `SCROLLDOWN/SCROLLUP` estão trocadas e foram descartadas).

  Gate: `scripts/verificar.sh` integral verde, 264 testes Rust (10 novos),
  clippy/C++/QML estritos, 11 harnesses e smoke offscreen (`exit 124`, sem
  saída). **Falta o seu gesto:** abrir o Claude no terminal e rolar.
- **cursor/TUI continua torto e NÃO é consertado pela roda.** Reportado em
  2026-07-16 nos dois agentes, já com o emulador novo — o que fecha a hipótese
  de que a profundidade do emulador era a causa. É renderer (R0–R3 de
  `docs/roadmaps/26`: métrica de célula, baseline, DPR), fatia própria. O
  mapeamento no core foi conferido e está correto.
- Windows é uma frente futura separada; não diluir o objetivo Linux-first atual.

## 7. Gate e definição de pronto de cada fatia

1. Ler `GUIAIA.md` e as fontes do domínio antes de editar.
2. Para funcionalidade de IDE, cumprir a referência profissional obrigatória:
   fonte oficial atual, revisão/invariantes registradas e adaptação própria sem
   cópia, conforme a seção 2.1 do roadmap de adaptação.
3. Manter UI → CoreClient → protocolo → handler → serviço; UI não chama
   ferramenta externa ou filesystem de workspace diretamente.
4. Operação longa vira Job cancelável e nunca bloqueia a UI.
5. Criar teste de core e harness QML quando houver estado visual.
6. Executar `bash scripts/verificar.sh` e a sonda específica do domínio.
7. Para UI/layout, validar o gesto em tela real; para packaging, executar
   `scripts/testar-appimage.sh` e `scripts/testar-appimage-portatil.sh`.
8. Atualizar contrato, schema, manual e arquitetura quando afetados.
9. Registrar conclusão em `ContextoIA.md` e no doc do domínio; remover o item
   concluído deste arquivo, sem manter listas riscadas ou post-mortems aqui.

## 8. Onde ficou o histórico concluído

Este é apenas um mapa para evitar duplicação:

- estado técnico, decisões e checkpoints: `ContextoIA.md`;
- rede de segurança, drafts e escrita atômica: `docs/seguranca/23-rede-de-seguranca.md`;
- autocomplete, terminal, paridade diária e UI: `docs/roadmaps/24-paridade-e-fundacao.md`;
- Tree-sitter e workspace edits: `docs/roadmaps/25-syntax-tree-semantic-foundation.md`;
- plano de daily driver e longo prazo: `docs/diario/18-daily-driver-plan.md` e
  `docs/roadmaps/21-long-horizon-roadmap.md`;
- apresentação pública: `README.md`; uso da IDE: `MANUAL.md`; distribuição e
  instalação: `Tutorial.md`;
- alterações exatas e checkpoints: histórico Git.

# Boot e comunicação: do processo ao primeiro frame

> **Classe: ESTADO** (`DocsPublic/README.md`). Descreve o que o código faz **hoje**.
> Medido em **2026-08-30** no protocolo `0.62.0` e revisto em **2026-09-02** no
> `0.63.0` (a §3 ganhou a terceira travessia da fronteira de thread), lendo o
> código, não a intenção. Se divergir do código, o código vence e este documento se
> corrige no mesmo gesto.
>
> **Para que serve:** o `03-ipc-protocol.md` responde *"qual é a forma da
> mensagem X?"*. Este responde a pergunta anterior, que não estava escrita em
> lugar nenhum: **quem sobe quem, por onde a mensagem anda, em qual thread ela
> é processada, e o que acontece quando um dos lados morre.** É a leitura que
> faltava para mexer em qualquer coisa que atravesse a fronteira.

## 1. Dois processos, e um deles é dono do outro

```text
┌────────────────────────────────────┐
│  kinein-vectis   (C++/Qt + QML)    │   processo PAI
│  ─ QQmlApplicationEngine           │   dono da janela e do ciclo de vida
│  ─ CoreClient  (QProcess)  ────────┼──┐
└────────────────────────────────────┘  │  stdin/stdout, uma linha por mensagem
                                        │  (JSON-RPC 2.0, UTF-8, sem framing
┌────────────────────────────────────┐  │   Content-Length)
│  kinein-core     (Rust)            │◄─┘   processo FILHO
│  ─ run_stdio()                     │
│  ─ Core: dispatch + estado         │
└────────────┬───────────────────────┘
             │  cada ferramenta externa é OUTRO processo filho
             ▼
   clangd · rust-analyzer · cmake · cargo · git · lldb-dap · $SHELL (PTY)
```

**O adaptador de debug é escolha do KIT desde 2026-09-03** (protocolo
`0.69.0`), e era uma constante até então. `lldb-dap` continua sendo o padrão —
sem escolha, o comportamento é byte a byte o de antes —, e `probe-rs` entra com
o subcomando `dap-server`, que faz ele falar DAP **por stdin/stdout**: a mesma
forma que este diagrama já descreve. Ver
[`roadmaps/35`](../roadmaps/35-ambiente-cpp-e-embarcados.md) §5.3 e
[`integracoes/36`](../integracoes/36-ferramentas-de-embarcados.md) §3.

**A UI é o pai e o core é o filho, não o contrário.** Isso decide três coisas
que aparecem o tempo todo:

- o core **não tem janela, não tem sessão de usuário e não sobrevive à UI**;
  fechar a IDE mata o core, e o core matando-se derruba só a si mesmo;
- **stdout do core é o canal de dados.** Nada mais pode escrever ali. É por isso
  que o `ARCHITECTURE.md` §4 regra 7 mantém `clippy::print_stdout` como aviso: um
  `println!` de depuração corrompe o protocolo, e o sintoma aparece na UI como
  "resposta inválida", longe da causa;
- **stderr é o canal de diagnóstico** e a UI o repassa para o log
  (`core stderr: …`), sem interpretá-lo.

O binário é resolvido em ordem explícita (`CoreClient::resolveCoreBinary`):
`$KINEIN_CORE_BIN`, depois ao lado do executável da UI, depois
`target/debug/kinein-core` a partir do diretório atual. **Sem core encontrado a
IDE sobe assim mesmo**, em estado desconectado e dizendo o porquê — ela não
falha no boot por causa disso.

## 2. A sequência de boot, na ordem em que acontece

```text
UI                                        CORE
──                                        ────
QQmlApplicationEngine carrega Main.qml
CoreClient::start()
  m_process.start()               ──────► processo nasce
                                          run_stdio():
                                            thread LEITORA de stdin
                                              (linha -> LoopEvent::Line)
                                            thread PONTE de notificacoes
                                              (LSP/eventos -> LoopEvent::Notification)
                                            Core::new()
                                            core.enable_lsp(events)
                                            core.enable_persistence(global_dir())
                                            loop principal comeca a drenar
handleStarted()  (sinal do QProcess)
  ping()                          ──────► core.ping
                                  ◄────── resposta: conectado
  [se recuperando] openWorkspace(ultimo)
```

**`enable_lsp` e `enable_persistence` são chamados SÓ no processo real.** Um
`Core::new()` puro — o que os 658 testes usam — tem LSP, run, debug, jobs e
terminal **desligados**, e persistência global **ausente**. Isso não é detalhe de
teste: é a barreira que impede a suíte de escrever no `$XDG_CONFIG_HOME` do
autor. Antes de 2026-08-29 ela não existia, e a suíte apagou a lista de projetos
recentes de verdade.

**Consequência prática para quem escreve sonda ou teste manual:** rodar o
binário real **liga** a persistência global. Toda sonda deste repositório
redireciona `XDG_CONFIG_HOME` para um diretório temporário por isso.

## 3. As threads do core, e por que existem

O core não é single-thread, mas **o estado é**. Só o loop principal toca o
`Core`; todo o resto conversa com ele por canal.

```text
THREAD                     O QUE FAZ                        FALA COM O LOOP POR
─────────────────────────  ──────────────────────────────   ────────────────────
principal                  drena `inbox`, aplica no `Core`,  — (e o dono do estado)
                           escreve a resposta em stdout
leitora de stdin           bloqueia em `lines()`             LoopEvent::Line
ponte de notificacoes      repassa o canal de eventos        LoopEvent::Notification
1 por servidor LSP         le o wire Content-Length          o canal de eventos
1 por job                  cmake/cargo/build/test/quality    o canal de eventos
3 por sessao de terminal   leitora do PTY, emissora ~30fps,  o canal de eventos
                           waiter do processo
```

**Por que a leitura de stdin vive em thread separada.** `lines()` bloqueia. Se o
loop principal lesse stdin direto, um core quieto (usuário sem digitar) não
conseguiria entregar um evento assíncrono — o render do terminal, a saída de um
build — porque estaria parado esperando uma linha que não vem. Os dois viram
`LoopEvent` no mesmo canal, e o loop atende quem chegar primeiro.

**O preço dessa arquitetura, e ele é real.** Um job roda numa thread própria e
recebe apenas um `JobContext` (id, cancelamento, `emit_output`, `emit_event`).
Ele **não alcança o `Core`** e portanto não alcança o `LspManager`, o
`TerminalManager` nem a store de rascunhos. Medido em 2026-08-30 ao investigar
o §5b do `roadmaps/29`: um job de `cmake.configure` **não consegue** invalidar
as flags de compilação do LSP como efeito colateral. Não é limitação a corrigir;
é a fronteira que mantém o estado com um dono só.

**Como atravessá-la, medido em 2026-09-02.** Há três formas, e a terceira é a
barata — ela estava no código desde sempre e passou despercebida até a etapa 4
do `roadmaps/30`:

```text
(a) objeto compartilhado explicito (Arc<Atomic…>)  cria um SEGUNDO dono do fato
(b) requisicao nova vinda da UI                    poe politica de core na UI
(c) reagir ao EVENTO no loop principal             <- sem estado novo, um dono
```

A (c) funciona porque **o evento que o job emite já volta ao dono do estado**: a
ponte de notificações o entrega como `LoopEvent::Notification`, e o loop
principal — que possui o `Core` — o vê *antes* de escrevê-lo no stdout. É ali
que mora `Core::observe_notification`, o irmão do `handle_request`: aquele
roteia PEDIDO, este roteia FATO. Foi assim que o fechamento dos documentos C/C++
após um configure saiu sem sincronização nova.

## 4. O caminho de uma requisição

```text
QML  coreClient.cmakeStatus()
 │
 ▼  ui/src/core_client_requests.cpp
CoreClient::sendRequest("cmake.status", {…})
 │   ├─ id = m_nextRequestId++
 │   ├─ m_pendingMethods[id] = metodo     ◄── é isto que faz a resposta saber
 │   └─ escreve UMA linha no stdin do core     de quem ela é
 ▼
core: LoopEvent::Line -> Core::handle_json_line
 │   ├─ parse; método desconhecido -> METHOD_NOT_FOUND
 │   └─ dispatch por domínio (handlers/<dominio>.rs)
 ▼  responde na MESMA linha de saída, com o mesmo id
UI: handleStdout() acumula bytes, corta em '\n'
 │   └─ handleResponseLine -> dispatchResult(metodo, result)
 ▼  emite um sinal Qt TIPADO (ex.: cmakeStatusResolved(...))
QML: <X>EventRouter recebe e chama o controller do domínio
```

**O `id` não viaja de volta para o QML.** A UI resolve o método pelo
`m_pendingMethods` e emite um sinal **tipado** por domínio. É o que sustenta a
proibição da §2 do `ARCHITECTURE.md`: o QML nunca vê JSON solto, vê
`cmakeStatusResolved(configured, hasCompileCommands, cdbStale, cdbStaleBecause)`.
Alargar um desses sinais é mudança de contrato em três arquivos — header, dispatch
e roteador — e é de propósito.

**Armadilha registrada (`ARCHITECTURE.md` §1.3):** `QProcess::start()` é
assíncrono. `sendRequest` chamado antes do estado `Running` **descarta a
mensagem em silêncio** — o pedido não falha, ele some. Por isso todo o
disparo inicial pendura em `handleStarted()`, nunca em `start()`.

## 5. O caminho de um evento (o core falando sem ser perguntado)

Evento é notificação JSON-RPC: **sem `id`, e a UI nunca responde**.

```text
event.terminal.render     grid pronto, ~30fps, por sessão
event.lsp.diagnostics     publishDiagnostics ja traduzido
event.job.*               progresso/saida/fim de job
event.cmake.started|finished
event.fs.changed          watcher (notify), coalescido
event.terminal.closed     shell saiu, com exitCode
```

A regra que os separa: **resposta é consequência de um pedido; evento é
consequência do mundo.** Um build não devolve o resultado na resposta — ela
devolve um `jobId`, e o resultado chega como evento. Isso é o que mantém a UI
sem bloquear em nada.

**Um evento pode chegar entre um pedido e a resposta dele.** O loop escreve na
ordem em que os `LoopEvent` chegam ao canal, então o cliente **não** pode supor
que a próxima linha após um pedido é a resposta dele. O `CoreClient` trata isso
naturalmente (casa por `id`); uma sonda escrita à mão precisa drenar e filtrar —
é exatamente o que o `pump()` das sondas faz.

## 6. Ordem, e a única garantia que existe

**As requisições são processadas na ordem em que chegam**, porque há um loop só
consumindo um canal. Isso é garantia de verdade e dá para contar com ela: mandar
`A` e depois `B` significa que o efeito de `A` no `Core` já ocorreu quando `B`
roda.

**Respostas ADIADAS (2026-09-18, Etapa 2 F6).** Uma classe de pedido — as
consultas interativas ao servidor de linguagem (`lsp.hover`, `definition`,
`completion`, `references`, `documentSymbols`, `workspaceSymbols`,
`semanticTokens`) — é PROCESSADA na ordem, mas RESPONDIDA fora dela: o handler
sincroniza o documento e escreve o pedido ao servidor, devolve
`RequestOutcome::Deferred` (o loop não escreve nada) e uma thread espera a
resposta (até 4 s) e a manda pelo canal de respostas adiadas —
`LoopEvent::Response`, escrita no stdout como um evento seria. Medido antes
disso: um `lsp.semanticTokens` segurava o loop até 4 s (15 s no
`initialize` do rust-analyzer) e o `fs.list` seguinte só saía depois — a IDE
inteira muda enquanto o servidor subia. O `id` continua casando a resposta
com o pedido na ponte; o que muda é que **a resposta de um pedido LSP pode
chegar depois da resposta de um pedido posterior**. O handshake do servidor
também corre numa thread: o loop espera no máximo 300 ms por ele (um servidor
rápido fica pronto na mesma chamada); depois, pedidos ao servidor que ainda
sobe voltam na hora com "ainda está subindo" e a UI reenvia o buffer ao ver
`event.lsp.status { running }`. **Rename e code actions (2026-09-19,
fechamento da Etapa 2)** usam a terceira forma, a CONTINUAÇÃO
(`Core::defer_then`): a espera pelo servidor é fora do loop e o que precisa
do `Core` — validar versões e abrir a transação do `workspaceEdit`, juntar
as ações de todos os servidores e guardar as cruas para o `apply` — volta
ao loop como `LoopEvent::Continue(Box<dyn FnOnce(&mut Core) ->
JsonRpcResponse>)`, roda com `&mut Core` na ordem em que chegou e a
resposta sai dali. Sem o canal (testes), `defer_then` roda as duas partes
inline. Só `workspaceEdit.apply/cancel` seguem síncronos: são escrita local,
sem servidor. O mesmo mecanismo simples (`Core::defer_work`) responde
`tools.detect` e `container.status` fora do loop: os 62 `--version` custam
~1,4 s, o `podman info` + `version` ~2,4 s, e a UI pede os dois na
abertura — antes, tudo esperava atrás deles; e o `workspace.open` de um
registro vazio usa só a presença das ferramentas (25 ms), a versão vem do
scan.

**O que NÃO é garantido:** a ordem entre uma resposta e os eventos que a
operação dispara. Um `terminal.input` responde imediatamente e os `render`
chegam depois, quantos o emulador achar necessário. Código que espera "a
resposta veio, então a tela já está atualizada" está errado — e é por isso que
as sondas drenam por tempo (`pump(seconds=…)`) em vez de contar mensagens.

## 7. Quando um lado morre

**O core morre (crash).** `QProcess::finished` com `CrashExit` dispara
`handleFinished`, que:

1. limpa TODO o estado derivado da UI (pendências, flags de build/test/run, ids
   de terminal, buffer de stdout) — estado que se refere a um processo que não
   existe mais é mentira;
2. relança o core, com **guarda anti-fork-bomb medida**: no máximo **3**
   tentativas dentro de uma janela de **4 s**; estourou, a recuperação **pausa**
   e a IDE diz isso em vez de ficar reiniciando para sempre;
3. no core novo, `handleStarted` reabre o **último workspace**;
4. `handleWorkspaceOpened` **suprime o restore de sessão** durante a recuperação
   (`m_recovering`). Reler as abas do disco sobrescreveria as edições não salvas
   que ainda estão vivas na UI.

**É aqui que a rede de segurança de dados entra** (`DocsPublic/seguranca/23`): os
rascunhos vivem em `<root>/.kinein/kinein.db` (SQLite, WAL), gravados por
`draft.save` com debounce de 1,5 s. Se quem morreu foi a **UI** — e aí não há
buffer nenhum a preservar —, o `workspace.open` seguinte devolve os rascunhos
em `drafts` e a IDE recompõe os buffers não salvos. Prova reproduzível:
`python3 scripts/sonda_drafts.py`, que mata o core com **SIGKILL** de verdade.

**A UI morre.** O core é filho: o `QProcess` é encerrado junto. O `Drop` do
`TerminalManager` mata todas as sessões de shell para não deixar processo órfão.

## 8. Como falar com o core na mão

O core é um binário que lê JSON-RPC de stdin. Isso é a ferramenta de
diagnóstico mais direta do projeto, e não precisa de UI nenhuma:

```bash
cargo build -p kinein-core
printf '%s\n' '{"jsonrpc":"2.0","id":1,"method":"core.ping","params":{}}' \
  | ./target/debug/kinein-core
```

As sondas do repositório são esse mesmo mecanismo, com um drenador de eventos
por cima. **Todas isolam `XDG_CONFIG_HOME`** (§2), e todas exigem um
`cargo build` fresco — armadilha conhecida: sonda rodando contra binário velho
prova o passado.

```text
scripts/sonda_drafts.py      rede de seguranca: crash SIGKILL -> recupera;
                             salvar limpa a store; escrita atomica; barreira
                             compare-before-save
scripts/sonda_terminal.py    D2.1/D2.2: comando no grid; resize 100x30 visto
                             pelo PROGRAMA (tput cols); topo do historico
scripts/sonda_scrollback.py  clamp do offset de scroll e multi-sessao (D2.3)
scripts/sonda_m43b.py        auto-restart de servidor LSP por timeout
scripts/fake_lsp_server.py   NAO e sonda: e o language server FALSO que os
                             testes do core sobem para observar o wire
                             (didOpen/didChange/didClose). Ver
                             crates/kinein-core/src/tests/lsp_server.rs
```

## 9. O que este desenho custa, dito de frente

- **Uma serialização por mensagem.** Todo dado atravessa JSON. Para o volume
  atual (o maior é o `event.terminal.render`, um grid de spans a ~30fps) isso
  se paga; é a razão de o render ser *throttled* no core em vez de a UI filtrar.
- **Sem tipos compartilhados em tempo de compilação.** O contrato é o
  `kinein-protocol` no lado Rust e sinais Qt tipados no lado C++, e nada obriga
  os dois a concordarem. A ponte é revisão e teste — e foi exatamente aqui que
  o `cdbStale` ficou meses medido pelo core e descartado pela UI, até
  2026-08-30.
- **Depuração atravessa processos.** Um sintoma na tela pode nascer em qualquer
  um dos dois lados; o log do `CoreClient` (`-> pedido` / `core stderr:`) existe
  para tornar isso legível.

O que o desenho compra em troca: um core testável sem GUI (658 testes rodam sem
subir Qt), uma UI que não pode chamar ferramenta externa nem por acidente, e
um crash de qualquer ferramenta externa que não derruba a IDE.

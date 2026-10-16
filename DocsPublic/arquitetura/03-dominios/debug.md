# `debug.*` — o domínio no protocolo IPC

> Parte do [03](../03-protocolo-ipc.md), que tem o formato base, as listas
> canônicas de métodos e eventos e as regras. Movido sem mudar o texto em
> 2026-10-16 ([61](../../roadmaps/61-organizacao-da-documentacao.md) §3.7, O6).

### Debug (`debug.*` — sessao DAP via lldb-dap)

Implementado no protocolo `0.28.0` (fatia M2.5a). O core
orquestra o `lldb-dap` (pacote `lldb`) pelo Debug Adapter Protocol; a UI
nunca fala DAP — recebe eventos `event.debug.*` ja mastigados. Uma sessao
por workspace.

- `debug.start { program?, connect? }` → `{ program, attached }`. Sem ambos, resolve o
  alvo "Automatico" espelhando o run: cargo → unico executavel no topo de
  `target/debug`; cmake → unico executavel de `.kinein/build`; **Python
  (`0.101.0`) → o mesmo ponto de entrada do Executar** (`main.py`/`app.py`/
  `__main__.py` na raiz, ou o script de `[project.scripts]` instalado; um
  pacote com `__main__.py` vira `module: "x"` desde `0.107.0`); zero ou
  varios candidatos → erro claro com a acao a tomar. NAO compila antes
  (build e acao explicita, Ctrl+F9). Erros: `TOOL_NOT_FOUND` (lldb-dap
  ausente), `INVALID_REQUEST` (sem alvo/sessao ja viva), `INVALID_PARAMS`
  (program inexistente), `INTERNAL_ERROR` (falha do adapter).

  **Alvo `.py` — ou um MÓDULO (`0.101.0`/`0.107.0`, cadeia Python,
  2026-09-13).** Seja qual for o tipo do workspace (um CMake com
  `tools/gera.py` inclusive), um `program` terminado em `.py` não passa pelo
  kit; e num workspace Python cujo ponto de entrada é um pacote com
  `__main__.py`, o alvo automático é o módulo — o `launch` leva `module:
  "pacote"` em vez de `program` (medido no debugpy 1.8.21: para no
  breakpoint dentro do pacote; `DebugStartResult.program` = `-m pacote`;
  configurações de servidor/chip do kit nativo não são herdadas): o adaptador é o
  **debugpy do interpretador do projeto** (`python/env.rs`, a precedência do
  `29` §4.1) — `<interpretador> -m debugpy.adapter`, DAP por stdin/stdout,
  `launch { program, cwd }` como o desktop. O `console` fica de fora de
  propósito: o `initialize` desta sessão não declara
  `supportsRunInTerminalRequest`, e o debugpy cai sozinho no
  `internalConsole` (stdout/stderr do programa chegam como `event.debug.
  output`) — medido no 1.8.21 com e sem o campo. Antes de subir, o core
  roda `<interpretador> -I -c "import debugpy"` (isolado: o que o adaptador
  vai ver; prazo de 10 s): sem interpretador → `INVALID_REQUEST` orientando
  a criar o ambiente; sem o módulo → `TOOL_NOT_FOUND` dizendo o passo para
  instalar NELE (`uv add --dev debugpy` ou `.venv/bin/python -m pip install
  debugpy`) — sem isso o adaptador morreria no primeiro request e a falha
  seria um timeout do `initialize`, longe da causa. O debugpy **não sai
  sozinho** no `disconnect` (medido): é o `Drop` da sessão que o mata, como
  já fazia com os outros. Provado ponta a ponta pelo core real em
  `scripts/verificar-python-debug.sh` (breakpoint em `app.py:2`, `soma` no
  topo da pilha, `a=2 b=3` em Locals — não Globals —, `a + b` = `5`,
  `resultado 5` por `event.debug.output`, `exitCode 0`, adaptador morto;
  1,4 s).
  **Attach Python (`0.109.0`).** `connect: { host: "127.0.0.1", port: 5678 }`
  conecta por TCP ao adaptador já criado por `debugpy --listen`/`debugpy.listen`.
  `program` e `connect` juntos são `INVALID_PARAMS`; host vazio ou com espaços,
  controles/URL e porta fora de 1–65535 também. Campos desconhecidos são recusados.
  Não resolve executável automático nem exige interpretador local. O mesmo
  handshake envia `attach { connect }`, reaplica breakpoints e configurationDone.
  Resultado/evento `attached: true`; `program` exibe `debugpy host:porta`.
  Launch devolve `attached: false`. Fechar/trocar workspace limpa a sessão e
  seus breakpoints. Sem SSH, attach por PID ou mapeamento de caminhos nesta fatia.

- `debug.setBreakpoints { file, lines: [int] }` →
  `{ breakpoints: [{ line, verified }] }`. Conjunto COMPLETO por arquivo
  (lines vazio limpa); `file` confinado ao workspace. Sem sessao viva o
  conjunto e guardado (`verified: false`) e replayado no proximo launch;
  com sessao, o adapter responde o que de fato amarrou.
- `debug.continue` / `debug.next` / `debug.stepIn` / `debug.stepOut`
  exigem processo pausado; `debug.pause` pausa o processo em execucao;
  `debug.stop` desconecta educadamente: no attach Python envia
  `terminateDebuggee: false` e fecha o socket, preservando o processo externo;
  em launch encerra o processo e o adaptador filho da IDE. Todos respondem
  `{ status: "ok" }`; exigem apenas o manager (como `run.stop`).

Inspeção (protocolo `0.29.0`, fatia M2.5c), sempre da thread pausada
(`INVALID_REQUEST` fora disso):

- `debug.stackTrace {}` → `{ frames: [{ id, name, file?, line? }] }` —
  topo primeiro, até 20 frames.
- `debug.variables { frameId }` → `{ frameId, variables, notice? }` — o core
  resolve os scopes DAP internamente e devolve as variáveis do primeiro
  escopo não-caro que não seja de registradores (Locals, Globals);
  registradores só quando não há mais nada, e então com `notice` dizendo
  por quê (0.145.0).
- `debug.variables { ref }` → `{ ref, variables }` — expande uma variável
  estruturada. Variável: `{ name, value, type?, ref }` (`ref` 0 = folha,
  > 0 = expansível). Exatamente um de `frameId`/`ref` → senão
  `INVALID_PARAMS`. A resposta ECOA a chave pedida para a UI correlacionar
  (mesmo padrão do `format.text`).
- `debug.scopes { frameId }` → `{ frameId, scopes: [{ name, ref, expensive }] }`
  (`0.118.0`, P3) — os escopos INTEIROS do frame, na ordem do adaptador:
  `Locals`, `Registers` (probe-rs e GDB), e `Peripherals` quando o kit tem
  `svdFile` e o adaptador é o probe-rs. `expensive` é o que o adaptador diz
  (ler todos os registradores de periférico custa); cada `ref` alimenta o
  MESMO `debug.variables { ref }`. É o D2+D4 do `roadmaps/41`: o que o
  `debug.variables { frameId }` esconde ao escolher o primeiro escopo.
- `debug.readMemory { memoryReference, offset?, count }` → `{ address,
  unreadableBytes?, data? }` (`0.118.0`) — o `readMemory` do DAP, verbatim:
  `memoryReference` é um endereço (`0x3ff00000`) ou a referência que uma
  variável/frame carrega; `data` é base64, como o DAP a manda (a UI mostra
  hexadecimal). `count` 0 ou referência vazia → `INVALID_PARAMS`; sem
  sessão parada → `INVALID_REQUEST`.
- `debug.disassemble { memoryReference, offset?, instructionOffset?,
  instructionCount }` → `{ instructions: [{ address, instruction,
  instructionBytes?, symbol?, file?, line? }] }` (`0.118.0`) — o
  `disassemble` do DAP, verbatim (`location.path` vira `file`). Os dois são
  anunciados pelo `probe-rs dap-server` 0.32 e pelo `gdb -i dap` 17 desta
  máquina (`supportsReadMemoryRequest`/`supportsDisassembleRequest`,
  medidos no `initialize` em 2026-09-17).
- `debug.evaluate { expression, frameId? }` → `{ expression, result, typeName?,
  reference }` (protocolo `0.66.0`) — avalia uma expressão (watch) no frame
  pedido. **Sem `frameId`, o core usa o frame do TOPO** da thread parada: é o
  que a UI quer quando o usuário digita um watch sem ter escolhido um frame.
  `reference > 0` significa expansível e alimenta o MESMO `debug.variables
  { ref }` acima — um watch é uma variável avaliada sob demanda, não uma
  árvore paralela. A resposta ECOA `expression`, e **o erro também a leva em
  `details`**: sem isso a UI não sabe qual watch falhou e marcaria todos.
  Expressão vazia ou só de espaços é `INVALID_PARAMS` no core, antes de chegar
  ao adapter — a mensagem do lldb para string vazia não ajuda ninguém.

**Mudança de contrato em `debug.setBreakpoints` (protocolo `0.66.0`).** O campo
`lines: [u32]` virou `breakpoints: [{ line, condition?, hitCondition? }]`.

```text
ANTES  { file, lines: [3, 7] }
AGORA  { file, breakpoints: [{ "line": 3 },
                             { "line": 7, "condition": "i == 42" }] }
```

`condition` e `hitCondition` mapeiam 1:1 nos campos de mesmo nome do
`SourceBreakpoint` do DAP — *"the breakpoint stops only when this evaluates to
true"* e *"how many times the breakpoint must be hit before stopping"* —, e o
`lldb-dap` anuncia `supportsConditionalBreakpoints` e
`supportsHitConditionalBreakpoints`. **Campo ausente não vira `null` no wire:**
mandar `"condition": null` (ou `"  "`) é pedir para um adapter tratar como
expressão vazia e o breakpoint nunca parar; o core descarta condição em branco
antes de montar o argumento. Array paralelo de condições foi recusado de
propósito — é a forma de linha e condição saírem de sincronia em silêncio.

```text
event.debug.started   { program, attached }
event.debug.output    { category, line, channel?, channelName? }
                                           (stdout|stderr|console|adapter|rtt)
event.debug.stopped   { reason, file?, line?, threadId }  (file/line podem
                        vir null; o core ja enriquece com o frame do topo)
event.debug.continued {}                   (um por retomada, deduplicado)
event.debug.finished  { exitCode? }        (exatamente um por sessao)
```

**`line` é sempre uma linha inteira (2026-10-08, 40.7 §7.256).** O adaptador
DAP pode entregar uma linha do programa em vários eventos `output` (o debugpy,
sob carga, mandou "resultado" e " 5\n"). O core junta os pedaços por
categoria até a quebra de linha. O que ficou sem quebra sai quando o programa
para (`stopped`), termina (`exited`/`terminated`) ou o fluxo do adaptador
acaba. Nada muda no formato do evento.

**`category: "rtt"` (`0.118.0`, D1 do `roadmaps/41`)** é o console da placa
pelo RTT/defmt do probe-rs: cada canal que o adaptador anuncia
(`probe-rs-rtt-channel-config { channelNumber, channelName, dataFormat }`)
vira uma linha `canal RTT <n> \`<nome>\` (<formato>) aberto` com `channel` e
`channelName`, e o core responde na hora o request `rttWindowOpened
{ channelNumber, windowIsOpen: true }` — sem ele o probe-rs NÃO lê o canal
("will delay polling RTT channels until the data window has opened",
`dap_types.rs` do 0.32.0). Os dados (`probe-rs-rtt-data { channelNumber,
data }`) saem linha a linha com `channel`; o defmt já chega decodificado
pelo adaptador quando o canal foi declarado assim (`rttChannelFormats` — hoje
o `launch` só liga `rttEnabled` e deixa o probe-rs detectar os canais). O
`probe-rs-show-message` (o que o adaptador mostraria numa caixa de diálogo)
vira uma linha `console` `probe-rs [<severidade>]: <mensagem>`.

**`category: "adapter"` (`0.111.0`)** é o stderr do PRÓPRIO adaptador que a
IDE criou (`lldb-dap`, `probe-rs dap-server`, `gdb -i dap`, `python -m
debugpy.adapter`), uma linha por evento — as outras três categorias vêm do
evento DAP `output` e falam do programa. Num attach TCP não há `adapter`: o
processo é de outro dono. As últimas 64 linhas ficam na sessão e entram na
mensagem do `debug.start` que falha no handshake (sob `--- stderr do
adaptador ---`): "o adapter nao respondeu a `initialize`" agora vem com o
`ImportError` que o causou. O servidor de debug do kit (`debugServer`) tem
a mesma cauda, sem evento por linha (um QEMU é verboso): ela entra em "o
servidor de debug saiu (…) antes de abrir …" e "nao abriu … em N s". A UI
pinta `adapter` como `stderr`.

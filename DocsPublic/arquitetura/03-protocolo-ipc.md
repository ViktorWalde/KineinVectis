# 03 — Protocolo IPC

> **As notas por versão** (o que cada versão acrescentou, da mais nova para
> a mais antiga) estão no [03.1](03.1-notas-por-versao-do-protocolo.md) desde
> 2026-10-15 ([61](../roadmaps/61-organizacao-da-documentacao.md) §3.5). Este
> arquivo é o formato, as seções por domínio e a lista canônica que o
> `scripts/verificar-fiacao-ipc.sh` confere. Ao subir a versão: a nota no topo
> do 03.1 e o contrato na seção do domínio, aqui.
>
> O protocolo-**alvo** completo está em
> `DocsPublic/especificacoes/arquitetura-interna-core-ipc-jobs.md`. Onde
> divergir, vale o que está implementado no código + o [`roadmaps/40`](../roadmaps/40-estado-e-continuidade.md).

## Objetivo

O protocolo IPC permite comunicação entre:

```text
Kinein Vectis UI  ←→  Kinein Vectis Core
```

A UI deve mandar comandos e receber respostas/eventos. O core deve executar lógica, chamar ferramentas externas e emitir eventos de estado.

## Os domínios

Cada domínio tem o seu arquivo em [`03-dominios/`](03-dominios/README.md)
desde 2026-10-16 ([61](../roadmaps/61-organizacao-da-documentacao.md) §3.7):
leia só o do contexto da mudança (00 §5.1). O formato base, as listas
canônicas e as regras ficam aqui.

| Domínio | Arquivo |
| --- | --- |
| `build.run`, `build.size` | [`03-dominios/build.md`](03-dominios/build.md) |
| `cargo.*` | [`03-dominios/cargo.md`](03-dominios/cargo.md) |
| `cmake.*` | [`03-dominios/cmake.md`](03-dominios/cmake.md) |
| `command.*` | [`03-dominios/command.md`](03-dominios/command.md) |
| `configAction.*` | [`03-dominios/configAction.md`](03-dominios/configAction.md) |
| `container.*` | [`03-dominios/container.md`](03-dominios/container.md) |
| `core.*` | [`03-dominios/core.md`](03-dominios/core.md) |
| `coverage.*` | [`03-dominios/coverage.md`](03-dominios/coverage.md) |
| `datasource.*` | [`03-dominios/datasource.md`](03-dominios/datasource.md) |
| `debug.*` | [`03-dominios/debug.md`](03-dominios/debug.md) |
| `dfu.*` | [`03-dominios/dfu.md`](03-dominios/dfu.md) |
| `draft.*` | [`03-dominios/draft.md`](03-dominios/draft.md) |
| `format.*` | [`03-dominios/format.md`](03-dominios/format.md) |
| `fs.*` (arquivos: ler, gravar, mover, lixeira) | [`03-dominios/fs.md`](03-dominios/fs.md) |
| `fs.findFiles`, `fs.search` | [`03-dominios/fs-busca.md`](03-dominios/fs-busca.md) |
| `git.*` | [`03-dominios/git.md`](03-dominios/git.md) |
| `grafana.*` | [`03-dominios/grafana.md`](03-dominios/grafana.md) |
| `index.*` | [`03-dominios/index.md`](03-dominios/index.md) |
| `job.*` | [`03-dominios/job.md`](03-dominios/job.md) |
| `library.*` | [`03-dominios/library.md`](03-dominios/library.md) |
| `lsp.*`, `event.lsp.documentsClosed` e `event.lsp.progress` | [`03-dominios/lsp.md`](03-dominios/lsp.md) |
| `platformio.*` | [`03-dominios/platformio.md`](03-dominios/platformio.md) |
| `probe.*` | [`03-dominios/probe.md`](03-dominios/probe.md) |
| `project.*` | [`03-dominios/project.md`](03-dominios/project.md) |
| `python.*` | [`03-dominios/python.md`](03-dominios/python.md) |
| `quality.*` | [`03-dominios/quality.md`](03-dominios/quality.md) |
| `remote.*` | [`03-dominios/remote.md`](03-dominios/remote.md) |
| `run.*` | [`03-dominios/run.md`](03-dominios/run.md) |
| `runConfig.*` | [`03-dominios/runConfig.md`](03-dominios/runConfig.md) |
| `serial.*` | [`03-dominios/serial.md`](03-dominios/serial.md) |
| `settings.*` | [`03-dominios/settings.md`](03-dominios/settings.md) |
| `setup.*` | [`03-dominios/setup.md`](03-dominios/setup.md) |
| `syntaxTree.*` | [`03-dominios/syntaxTree.md`](03-dominios/syntaxTree.md) |
| `terminal.*` | [`03-dominios/terminal.md`](03-dominios/terminal.md) |
| `test.*` | [`03-dominios/test.md`](03-dominios/test.md) |
| `toolchain.*` | [`03-dominios/toolchain.md`](03-dominios/toolchain.md) |
| `tools.detect`, `tools.status`, `environment.scan` | [`03-dominios/tools.md`](03-dominios/tools.md) |
| `workspace.*` | [`03-dominios/workspace.md`](03-dominios/workspace.md) |

## Transporte inicial

Para o MVP:

```text
stdin/stdout JSON-RPC
```

Depois:

```text
Unix domain socket
```

Futuro:

```text
gRPC/local socket
```

## Formato base

### Request

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "core.ping",
  "params": {}
}
```

### Response

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "status": "ok",
    "message": "pong"
  }
}
```

### Error

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": "TOOL_NOT_FOUND",
    "message": "clangd não foi encontrado",
    "details": {
      "tool": "clangd",
      "suggestedCommand": "sudo pacman -S clang"
    }
  }
}
```

### Event

```json
{
  "jsonrpc": "2.0",
  "method": "event.diagnostics.updated",
  "params": {
    "uri": "file:///home/vitor/dev/projeto/src/main.cpp",
    "errors": 1,
    "warnings": 0
  }
}
```

### Diagnóstico comum (`Diagnostic`)

Implementado no protocolo `0.20.0` como modelo comum para Problems. Os eventos
continuam sendo por domínio (`event.build.diagnostic`,
`event.quality.diagnostic`, `event.project.changed

event.lsp.diagnostics`), mas os itens de
diagnóstico usam campos comuns:

```text
Diagnostic {
  id?,
  source: "build|quality|lsp|toolchain|compiler",
  severity: "error|warning|note",
  category?,
  message,
  file?,
  line?,        // 1-based, início do range
  column?,      // 1-based, início do range
  endLine?,     // 1-based, fim do range (T6, para o sublinhado)
  endColumn?,   // 1-based, fim do range
  code?,        // código/regra: "E0425", "unused_variables", ...
  jobId?,
  command?,
  target?,
  logRef?
}
```

Estado atual: build usa `source: "build"` / `category: "compiler"`;
quality usa `source: "quality"` / `category: "lint"`; LSP usa
`source: "lsp"` / `category: "lsp"`; a checagem do compilador ao salvar
(`0.175.0`, a 0.4.0c do `roadmaps/52` §14.3) usa `source: "compiler"` e
chega no mesmo `event.lsp.diagnostics`, fundida com a do servidor da
linguagem por arquivo (a união; a lista vazia apaga só a parte dela). Ela
só roda em projeto confiável (`workspace.trust`): o compilador vem da CDB, e
a CDB pode ter vindo no repositório. `id`, `command`, `target` e `logRef` já
existem no tipo de protocolo, mas ainda só aparecem quando um produtor tiver
dado real para preencher.

`endLine`/`endColumn`/`code` entraram no protocolo `0.35.0` (fatia T6):
`event.lsp.diagnostics` passa a mandar o range completo (o core deriva de
`textDocument/publishDiagnostics`; `end` ausente cai de volta ao início) e o
`code` do servidor (string ou número → sempre string). A UI usa o range para
o sublinhado ondulado no editor e a marca na gutter, e o `code` no detalhe da
aba Problemas. Build/quality ainda não preenchem range (sublinhado deles fica
para o Problems 2.0).

### IA externa: sem domínio próprio (removido no `0.59.0`)

O domínio `aiBridge.*` **não existe mais**. Ele expunha `aiBridge.profiles` e
`aiBridge.terminal.open`, que abriam `claude`/`codex` por um caminho especial:
argumentos injetados pelo core (`--no-alt-screen`, `--ax-screen-reader`) e um
filtro que engolia `CSI 3 J` da própria aplicação.

Esse caminho era a IDE interferindo no terminal. Ele tratava um agente de CLI
como se fosse diferente de qualquer outro programa, o que produziu sintomas
atribuídos ao painel quando a causa estava no emulador raso (ver
`DocsPublic/decisoes-adr/ADR-0004-alacritty-terminal-emulator.md`).

O modelo agora é o de qualquer IDE profissional: **uma CLI de IA é um programa
como outro qualquer**. Abrir um terminal (`terminal.open`) e digitar `claude`
ou `codex` usa o mesmo PTY, o mesmo emulador e o mesmo contrato
`terminal.input/resize/scroll/close` de um `ls`. Não há allowlist de perfil,
argumento imposto, filtro de sequência nem preferência persistida.

Consequência de produto: rodar um agente dentro da Kinein passa a ser
indistinguível de rodá-lo fora dela — que era o requisito original. Uma
superfície visual de atalho (Assistente) pode voltar depois **como UI pura**,
abrindo uma sessão de terminal comum, sem regra de negócio própria no core.

## Os 191 métodos roteados — a lista inteira

> **Refeita por medição em 2026-09-24**, contando os braços `"dominio.metodo"`
> dos roteadores do core com o mesmo código do `verificar-fiacao-ipc.sh`. A
> lista dizia "inteira" e tinha 149 de 167: faltavam o domínio `remote.*`
> completo, `coverage.lines/run`, `debug.disassemble/readMemory/scopes`,
> `python.stubs` e `serial.files`. Lista escrita à mão divergindo em silêncio é
> o que este documento existe para impedir — daí a regeneração.
>
> **2026-09-17:** `serial.identify` (0.112.0), `runConfig.flashProposal`
> (0.113.0) e `serial.access` (0.114.0) entraram; eram 140.

> **Era "Métodos principais implementados", e listava 66 dos 128** — sem dizer
> que era parcial, o que fazia um domínio inteiro parecer inexistente.
> Refeita em 2026-09-06 pelo comando do cabeçalho, agrupada por domínio; os
> `sim.*` saíram dela em 2026-09-12 com o domínio.

```text
build.run
build.size

cargo.check
cargo.metadata

cmake.configure
cmake.presets.list
cmake.status
cmake.targets.list

command.list

configAction.apply
configAction.list
configAction.preview

container.action
container.compose
container.images
container.list
container.open
container.status
container.volumes

core.ping
core.shutdown

coverage.lines
coverage.run

datasource.odbc.sources
datasource.odbc.authorize
datasource.console
datasource.console.statement
datasource.create
datasource.destroy
datasource.discover
datasource.impact
datasource.introspect
datasource.list
datasource.query
datasource.preview.decide
datasource.disconnect
datasource.history
datasource.history.clear
datasource.remove
datasource.save
datasource.test

debug.continue
debug.disassemble
debug.evaluate
debug.next
debug.pause
debug.readMemory
debug.scopes
debug.setBreakpoints
debug.stackTrace
debug.start
debug.stepIn
debug.stepOut
debug.stop
debug.variables

draft.clear
draft.save

environment.scan

format.capabilities
format.text

fs.createDirectory
fs.createFile
fs.copy
fs.delete
fs.findFiles
fs.list
fs.read
fs.readExternal
fs.rename
fs.replace
fs.search
fs.trash
fs.transferBatch
fs.write

git.blame
git.branchCreate
git.branches
git.checkout
git.commit
git.commitDiff
git.discard
git.fileDiff
git.log
git.pull
git.push
git.stage
git.stash
git.status
git.unstage

grafana.forget
grafana.get
grafana.probe
grafana.save

index.context
index.status
index.symbols

job.cancel
job.list

library.list
library.plan

lsp.applyCodeAction
lsp.codeActions
lsp.completion
lsp.definition
lsp.didChange
lsp.documentSymbols
lsp.hover
lsp.references
lsp.rename
lsp.restart
lsp.semanticTokens
lsp.switchSourceHeader
lsp.workspaceEdit.apply
lsp.workspaceEdit.cancel
lsp.workspaceSymbols

platformio.homeRelease
platformio.homeStart
platformio.homeStop

dfu.list
probe.list

project.model

python.createEnvironment
python.status
python.stubs

quality.run

remote.command
remote.deploy
remote.directories
remote.discover
remote.hostKey
remote.list
remote.open
remote.parseCommand
remote.probe
remote.remove
remote.resolve
remote.save
remote.status
remote.sync
remote.trustHost

run.capabilities
run.script
run.start
run.stop

runConfig.delete
runConfig.flashProposal
runConfig.list
runConfig.save
runConfig.setActive

serial.access
serial.files
serial.identify
serial.list
serial.monitor

settings.get
settings.set

setup.list

syntaxTree.indent
syntaxTree.update

terminal.clearScrollback
terminal.close
terminal.copySelection
terminal.input
terminal.mouse
terminal.open
terminal.resize
terminal.scroll
terminal.selectAll

test.discover
test.run

toolchain.get
toolchain.importKit
toolchain.inspectSysroot
toolchain.install
toolchain.installable
toolchain.set
toolchain.setKit

tools.detect
tools.status

workspace.browse
workspace.close
workspace.createFolder
workspace.createProject
workspace.open
workspace.recent.clear
workspace.recent.list
workspace.recent.pin
workspace.recent.remove
workspace.saveSession
workspace.status
workspace.trust
```

## Os 63 eventos emitidos — a lista inteira

> **2026-10-03:** a lista estava atrás do código — sete eventos entraram sem
> chegar aqui (`coverage.finished`, `python.stubs`, `remote.deployed`,
> `remote.directories`, `remote.probed`, `remote.synced`, `serial.files`).
> Conferida contra o código (o comando está no `roadmaps/40` §1) e completada,
> junto com `datasource.impact` (0.150.0).

> **2026-09-17:** `event.lsp.log` (0.111.0) e `event.serial.identified`
> (0.112.0) entraram; eram 48.

> **Era "Eventos iniciais", e faltavam três** — os dois do `datasource` e o do
> `grafana`. Refeita em 2026-09-06. **Cinco não são literais no código**: eles
> nascem de `format!("event.{domain}.*")` em `handlers/build.rs`, com `domain`
> valendo `build` ou `quality`, e por isso um grep ingênuo não os encontra.

```text
event.build.diagnostic              <- so por format!
event.build.finished
event.build.output                  <- so por format!
event.build.started                 <- so por format!

event.cmake.finished
event.cmake.started

event.datasource.created
event.datasource.destroyed
event.datasource.impact
event.datasource.introspected
event.datasource.queried
event.datasource.previewed
event.datasource.disconnected
event.datasource.tested

event.debug.continued
event.debug.finished
event.debug.output
event.debug.started
event.debug.stopped

event.environment.finished
event.environment.started
event.container.finished
event.coverage.finished

event.environment.tool

event.fs.changed
event.fs.watchError

event.git.remoteFinished

event.grafana.probed

event.index.finished
event.index.progress

event.job.created
event.job.finished
event.job.output
event.job.progress

event.lsp.diagnostics
event.lsp.documentsClosed
event.lsp.log
event.lsp.progress
event.lsp.restarted
event.lsp.status

event.platformio.homeCommand
event.platformio.homeExited

event.project.changed
event.python.finished
event.python.stubs
event.quality.diagnostic
event.quality.finished
event.quality.output                <- so por format!
event.quality.started               <- so por format!
event.remote.deployed
event.remote.directories
event.remote.probed
event.remote.synced
event.serial.files

event.serial.identified

event.terminal.closed
event.terminal.render

event.test.case
event.test.discovered
event.test.finished
event.test.output
event.test.started

event.toolchain.installed
```

## Regras

1. Todo request deve ter resposta.
2. Eventos não têm `id`.
3. Erros devem ter `code`, `message` e `details`.
4. O protocolo deve ser versionado.
5. Toda alteração no protocolo exige atualização de docs e schemas.
6. A UI não deve interpretar logs brutos quando houver evento estruturado.

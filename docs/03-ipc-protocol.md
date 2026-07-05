# 03 — Protocolo IPC

> **Escopo:** este documento descreve o protocolo **implementado** hoje
> (JSON-RPC 0.19.0: `core.*`, `tools.*`, `workspace.*`, `fs.*`, `build/test/
> quality.run`, `lsp.*`, `run.*`, `terminal.*`). O protocolo-**alvo** completo
> (jobs, setup, cmake/cargo services, ai-bridge, targets, etc.) está em
> `docs/specs/KINEIN_VECTIS_INTERNAL_ARCHITECTURE_CORE_IPC_JOBS.md`. Onde
> divergir, vale o que está implementado no código + `ContextoIA.md`.

## Objetivo

O protocolo IPC permite comunicação entre:

```text
Kernwerk UI  ←→  Kernwerk Core
```

A UI deve mandar comandos e receber respostas/eventos. O core deve executar lógica, chamar ferramentas externas e emitir eventos de estado.

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

### Resultado de `tools.detect` / `tools.status`

Implementado no protocolo `0.2.0`. `tools.detect` sempre executa a detecção e
atualiza o registro interno; `tools.status` responde com o último resultado
conhecido (detectando na primeira chamada).

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "tools": [
      {
        "id": "cargo",
        "displayName": "Cargo",
        "status": "detected",
        "path": "/home/user/.cargo/bin/cargo",
        "version": "cargo 1.96.1"
      },
      {
        "id": "clangd",
        "displayName": "clangd",
        "status": "missing",
        "suggestedInstall": "sudo pacman -S clang",
        "message": "clangd nao foi encontrado no PATH."
      }
    ]
  }
}
```

Estados possíveis de ferramenta (`docs/07-tooling-lifecycle.md`):
`notConfigured`, `missing`, `detected`, `ready`, `running`, `failed`,
`disabled`. A detecção usa `missing`, `detected` e `failed`; os demais são
reservados para o gerenciamento de processos.

### Workspace (`workspace.browse` / `workspace.createFolder` / `workspace.createProject` / `workspace.open`)

`workspace.open` foi implementado no protocolo `0.3.0`. `workspace.browse`
foi adicionado no protocolo `0.5.0` para o seletor proprio de workspace da UI.
`workspace.createFolder` e `workspace.createProject` foram adicionados no
protocolo `0.9.0` para permitir o fluxo JetBrains-like de criar pasta/projeto
sem sair da IDE.

`workspace.open` recebe
`{ "path": "/dir" }`, canonicaliza o caminho, identifica o tipo de projeto por
marcadores (precedência: `Cargo.toml` > `CMakeLists.txt` > `pom.xml` >
Gradle > Python) e persiste `.kernwerk/workspace.json`
(`schemas/workspace.schema.json`).

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "name": "meu-projeto",
    "root": "/home/user/dev/meu-projeto",
    "kind": "rustCargo",
    "markers": ["Cargo.toml"]
  }
}
```

`workspace.browse` recebe `{ "path": "/dir" }`, canonicaliza o diretorio e
retorna apenas subdiretorios para a UI navegar sem depender de dialogo nativo do
desktop. A UI continua proibida de listar o filesystem diretamente.

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "path": "/home/user",
    "parent": "/home",
    "entries": [
      {
        "name": "dev",
        "path": "/home/user/dev"
      }
    ]
  }
}
```

`workspace.createFolder` recebe `{ "parent": "/dir", "name": "modulo" }`.
O core canonicaliza `parent`, valida que `name` é apenas um segmento de caminho
e cria o diretório filho. A resposta é `{ "path": "/dir/modulo" }`.

`workspace.createProject` recebe
`{ "parent": "/dir", "name": "demo", "template": "empty|cppCmake|rustCargo" }`.
O core cria o diretório do projeto, aplica o template e abre o projeto como
workspace, retornando o mesmo payload de `workspace.open`.

- `empty`: cria diretório vazio e persiste `.kernwerk/workspace.json`.
- `cppCmake`: cria projeto C++23/CMake strict inicial com `CMakeLists.txt`,
  `CMakePresets.json`, `src/main.cpp` e `README.md`.
- `rustCargo`: usa `cargo new --bin --vcs none`; se `cargo` não existir,
  retorna `TOOL_NOT_FOUND`.

`workspace.status` responde `{ "workspace": <objeto acima> | null }`.
`workspace.close` responde `{ "status": "ok", "closed": <root | null> }`.
Caminho inexistente ou sem `path` nos params retorna `INVALID_PARAMS`; falha de
IO ao persistir, listar ou criar diretorios retorna `INTERNAL_ERROR`.

### Arquivos (`fs.list` / `fs.read` / `fs.createFile` / `fs.createDirectory` / `fs.write` / `fs.rename` / `fs.delete`)

Implementado no protocolo `0.4.0`. Todos exigem workspace aberto
(`INVALID_REQUEST` caso contrário) e todo caminho é canonicalizado e
confinado à raiz do workspace (`INVALID_PARAMS` se escapar).

- `fs.list { path }` → `{ path, entries: [{ name, kind: file|directory|other, size? }] }`,
  ordenado diretórios primeiro, depois nome case-insensitive.
- `fs.read { path }` → `{ path, content }`. Limites: arquivo regular, até
  1 MiB, UTF-8 válido (senão `INVALID_PARAMS` com mensagem humana).
- `fs.createFile { path, content? }` → `{ path, bytesWritten }`. Criado no
  protocolo `0.15.0`; o diretório pai precisa existir dentro do workspace e a
  operação falha se o arquivo já existir.
- `fs.createDirectory { path }` → `{ path }`. Criado no protocolo `0.15.0`;
  o diretório pai precisa existir dentro do workspace e a operação falha se o
  caminho já existir.
- `fs.write { path, content }` → `{ path, bytesWritten }`. Sobrescreve apenas
  arquivos existentes.
- `fs.rename { from, to }` → `{ from, to }`. Criado no protocolo `0.19.0`;
  renomeia ou move um arquivo ou diretório dentro do workspace. `from` precisa
  existir; `to` não pode já existir e seu diretório pai precisa existir dentro
  do workspace. A raiz do workspace não pode ser renomeada (`INVALID_PARAMS`).
- `fs.delete { path }` → `{ path }`. Criado no protocolo `0.19.0`; remove um
  arquivo ou diretório (recursivo para diretórios) dentro do workspace. A raiz
  do workspace não pode ser removida (`INVALID_PARAMS`).

### Busca de arquivos (`fs.findFiles`)

Implementado no protocolo `0.16.0`. Requer workspace aberto. O core usa
`fd` para busca por nome de arquivo, respeitando ignores do projeto e evitando
um indexador próprio no MVP.

- `fs.findFiles { query }` →
  `{ matches: [{ path, name }], truncated }`.
- `query` não pode ser vazio (`INVALID_PARAMS`).
- `path` é relativo à raiz do workspace; `name` é o nome do arquivo.
- A busca usa `fd --type f --fixed-strings --hidden --color never`, com
  exclusões explícitas para `.git`, `.kernwerk`, `.idea`, `.cache`, `target`,
  `build` e `node_modules`.
- No máximo 100 arquivos são retornados; `truncated: true` indica que o limite
  cortou resultados.
- O core aceita o binário `fd` e também `fdfind` (nome usado por algumas
  distribuições). Se nenhum existir no `PATH`, retorna `TOOL_NOT_FOUND`.

### Busca no workspace (`fs.search`)

Implementado no protocolo `0.11.0` (Find in Files). Requer workspace aberto.

- `fs.search { query, caseSensitive? }` →
  `{ matches: [{ path, line, column, preview }], truncated }`.
- `query` é texto literal (não regex) e não pode ser vazio
  (`INVALID_PARAMS`). `caseSensitive` é opcional e por padrão `false`
  (comparação case-insensitive apenas ASCII, para manter offsets exatos).
- `path` é relativo à raiz do workspace; `line`/`column` são 1-based
  (coluna em caracteres) para consumo direto do editor.
- A caminhada é determinística (profundidade, nome case-insensitive),
  ignora silenciosamente symlinks, arquivos não UTF-8 ou maiores que 1 MiB e
  os diretórios `.git`, `.kernwerk`, `.idea`, `.cache`, `target`, `build` e
  `node_modules`.
- No máximo um match por linha e 500 matches no total; `truncated: true`
  indica que o limite cortou resultados. `preview` é a linha com trim,
  limitada a 200 caracteres.

### Execução (`run.start` / `run.stdin` / `run.stop`)

Implementado no protocolo `0.12.0`. Requer workspace aberto. O core executa
um comando via `sh -c` na raiz do workspace SEM bloquear o loop de IPC: a
saída chega como notificações assíncronas (mesmo canal dos eventos LSP) e o
processo aceita stdin e cancelamento enquanto roda. Um processo por vez.

- `run.start { command? }` → `{ command }`. Sem `command`, o core deriva o
  padrão do tipo de projeto: `cargo run` para Rust/Cargo; para CMake, o
  único executável em `.kernwerk/build` (erro claro se não houver ou houver
  mais de um). Outros tipos ainda não têm padrão (`INVALID_REQUEST` com
  mensagem orientando digitar o comando).
- `run.stdin { data }` → `{ status: "ok" }`. Encaminha `data` cru ao stdin
  do processo (a UI acrescenta o `\n`).
- `run.stop {}` → `{ status: "ok" }`. Mata o processo; o término é
  reportado pelo evento `finished`.
- Fechar o workspace mata o processo em execução automaticamente.

```text
event.run.started   { "command": "cargo run" }
event.run.output    { "stream": "stdout|stderr", "line": "..." }
event.run.finished  { "success": bool, "exitCode": int|null }
```

OBS: `run.*` não tem TTY (pipes simples) — é o executor do botão Run. Para
shell interativo com PTY, ver `terminal.*` abaixo.

### Terminal (`terminal.open` / `terminal.input` / `terminal.close`)

Implementado no protocolo `0.13.0`. Requer workspace aberto. O core NÃO
implementa emulador de terminal: ele orquestra o `script(1)` (util-linux)
para alocar um PTY real e rodar o shell do próprio usuário (`$SHELL`,
interativo — perfil, aliases e prompt carregados) na raiz do workspace.
Uma sessão por vez, persistente (cd, variáveis e histórico da sessão
sobrevivem entre comandos).

- `terminal.open {}` → `{ shell }`. Erro `INVALID_REQUEST` se já aberto.
- `terminal.input { data }` → `{ status: "ok" }`. Encaminha `data` cru ao
  PTY (a UI acrescenta o `\n`). O eco vem pela própria saída do PTY.
- `terminal.close {}` → `{ status: "ok" }`.
- Fechar o workspace fecha a sessão automaticamente.

```text
event.terminal.data    { "data": "chunk de texto ja sanitizado" }
event.terminal.closed  { "exitCode": int|null }
```

O core roda a sessão com `TERM=dumb` e remove sequências ANSI (CSI/OSC) e
`\r` com um sanitizador com estado que sobrevive a chunks divididos, para a
UI renderizar texto puro. LIMITAÇÃO REGISTRADA: como a UI renderiza texto
sanitizado, programas full-screen (vim, htop) não desenham corretamente
mesmo enxergando um TTY real; o caminho futuro é renderizar ANSI na UI.

### Build (`build.run`)

Implementado no protocolo `0.6.0`. Requer workspace aberto. O core executa a
ferramenta de build do tipo de projeto (`cargo build --message-format=json`
para Rust/Cargo; `cmake -S/-B` + `cmake --build` em `.kernwerk/build` para
CMake) e emite notificações durante a execução:

```text
event.build.started     { "command": "cargo build" }
event.build.output      { "stream": "stdout|stderr", "line": "..." }
event.build.diagnostic  { "severity": "error|warning|note", "message", "file"?, "line"?, "column"? }
event.build.finished    { "success", "exitCode", "diagnostics" }
```

A resposta final repete o resumo: `{ "success", "exitCode", "diagnostics" }`.
Diagnósticos vêm do JSON do cargo (span primário) ou do formato
`arquivo:linha:coluna: nivel: mensagem` de compiladores/CMake. Tipos sem
integração de build retornam `INVALID_REQUEST`; ferramenta ausente retorna
`TOOL_NOT_FOUND`. Cancelamento ainda não é suportado.

### Qualidade / lint (`quality.run`)

Implementado no protocolo `0.18.0`. Requer workspace aberto. Reusa
inteiramente o pipeline do `build.run`: para Rust/Cargo roda
`cargo clippy --all-targets --message-format=json`, cujo JSON é idêntico ao
do `cargo build`, então os lints viram diagnósticos estruturados sem parser
novo. Emite `event.quality.started/output/diagnostic/finished` (mesmos
formatos dos `event.build.*`); a UI adiciona os diagnósticos à aba Problemas
com origem `quality` (limpos a cada análise, distintos dos de build/LSP).
A resposta final repete `{ "success", "exitCode", "diagnostics" }`. Tipos
de projeto sem linter integrado (por ora, tudo que não é Rust/Cargo — CMake
via clang-tidy é o próximo passo) retornam `INVALID_REQUEST`; `cargo`
ausente retorna `TOOL_NOT_FOUND`.

### Testes (`test.run`)

Implementado no protocolo `0.17.0`. Requer workspace aberto. O core executa o
runner de testes do tipo de projeto (`cargo test` para Rust/Cargo; `ctest
--test-dir .kernwerk/build --output-on-failure` para CMake) e transmite cada
caso conforme sai da saída padrão do runner — nada de reimplementar framework
de teste. Aceita `{ "filter"? }` (posicional do cargo; `-R` do ctest).

```text
event.test.started   { "command": "cargo test" }
event.test.output    { "stream": "stdout|stderr", "line": "..." }
event.test.case      { "name": "modulo::caso", "status": "passed|failed|ignored" }
event.test.finished  { "success", "exitCode", "passed", "failed", "ignored" }
```

Os casos são extraídos das linhas `test <nome> ... ok|FAILED|ignored` (libtest)
e `... Test #N: <nome> ... Passed|***Failed` (ctest); a linha de resumo do
libtest é ignorada. A resposta final repete
`{ "success", "exitCode", "passed", "failed", "ignored" }`. Tipos sem
integração retornam `INVALID_REQUEST`; runner ausente retorna `TOOL_NOT_FOUND`.
O streaming reusa o mesmo mecanismo do `build.run` (módulo `process`);
cancelamento ainda não é suportado.

### LSP (`lsp.didChange` / `lsp.definition` / `lsp.hover` / `lsp.completion` / `lsp.references` / `lsp.rename`)

Implementado nos protocolos `0.7.0` e `0.8.0`. Requer workspace aberto. A UI envia
`lsp.didChange { "path": "/abs/file", "content": "..." }` depois de debounce
do editor; o core canonicaliza o caminho, confina à raiz do workspace e
sincroniza o texto completo com o servidor LSP gerenciado. A UI nunca fala LSP
diretamente.

O core sobe servidores de linguagem sob demanda quando um arquivo suportado é
aberto, alterado ou salvo:

- C/C++: `clangd --background-index`
- Rust: `rust-analyzer`

Se o servidor não estiver instalado ou falhar no `initialize`, o core emite
`event.lsp.status` com `status: "failed"` e mensagem humana. O MVP usa
sincronização de documento inteiro.

`lsp.definition` e `lsp.hover` foram adicionados no protocolo `0.8.0`. Ambos
recebem posição 1-based e o buffer atual para o core sincronizar o documento
antes de consultar o language server:

```json
{
  "path": "/abs/file",
  "content": "texto atual do editor",
  "line": 12,
  "column": 5
}
```

`lsp.definition` responde `{ "path"?, "line"?, "column"? }`; campos ausentes
significam que o servidor não encontrou alvo. `lsp.hover` responde
`{ "content"? }`, com markup LSP achatado em texto.

`lsp.completion`, `lsp.references` e `lsp.rename` foram adicionados no
protocolo `0.10.0` (Fase 5.2). Completion e references usam os mesmos
parâmetros posicionais de `lsp.definition`; rename recebe adicionalmente
`"newName"` (não vazio):

- `lsp.completion` responde `{ "items": [{ "label", "insertText",
  "detail"?, "kind"? }] }`. O core ordena por `sortText` e limita a 50 itens;
  `kind` é o `CompletionItemKind` numérico do LSP achatado em texto
  (`function`, `variable`, ...).
- `lsp.references` responde `{ "references": [{ "path", "line", "column" }] }`
  (1-based, limitado a 200 usos, incluindo a declaração).
- `lsp.rename` é fim a fim: o core consulta o servidor, valida que todos os
  arquivos afetados estão dentro do workspace, aplica os edits em memória
  (posições LSP 0-based/UTF-16 convertidas para offsets UTF-8) e só então
  reescreve os arquivos no disco, re-sincronizando os documentos abertos no
  servidor. Responde `{ "files": ["/abs/..."], "edits": N }`. Renames que
  criam/renomeiam/apagam arquivos (resource operations) ainda não são
  suportados e retornam erro estruturado sem tocar em nada.

`lsp.semanticTokens` foi adicionado no protocolo `0.14.0`. Recebe
`{ path, content }` (mesmos campos de `lsp.didChange`), sincroniza o buffer
e resolve `textDocument/semanticTokens/full`, decodificando os deltas com a
legend anunciada pelo servidor no `initialize`. Responde
`{ tokens: [{ line, start, length, kind }] }` com `line` 1-based e
`start`/`length` em unidades UTF-16 (0-based) — os mesmos índices de
`QString`, aplicados direto pelo highlighter da UI. `kind` é o nome da
legend (`variable`, `function`, `parameter`, `class`, ...). Servidores sem
suporte respondem lista vazia.

Code actions e restart/cancelamento de requests LSP ainda não fazem parte
deste contrato.

```text
event.lsp.status       { "language": "cpp|rust", "status": "running|failed|stopped|exited", "message"? }
event.lsp.diagnostics  { "path": "/abs/file", "diagnostics": [{ "severity": "error|warning|note", "message", "line", "column" }] }
```

`line` e `column` dos diagnósticos são 1-based para consumo direto da UI. A
UI integra esses eventos à aba Problemas com origem `lsp`.

## Métodos iniciais

```text
core.ping
core.shutdown
workspace.open
workspace.browse
workspace.createFolder
workspace.createProject
workspace.close
workspace.status
settings.get
settings.set
command.list
command.execute
tools.detect
tools.status
fs.list
fs.read
fs.createFile
fs.createDirectory
fs.write
fs.rename
fs.delete
fs.findFiles
fs.search
build.configure
build.run
test.run
quality.run
run.start
run.stdin
run.stop
terminal.open
terminal.input
terminal.close
lsp.didChange
lsp.semanticTokens
lsp.definition
lsp.hover
lsp.completion
lsp.references
lsp.rename
logs.tail
```

## Eventos iniciais

```text
event.core.ready
event.workspace.opened
event.workspace.closed
event.tool.statusChanged
event.build.started
event.build.output
event.build.finished
event.lsp.status
event.lsp.diagnostics
event.diagnostics.updated
event.notification.created
event.log.appended
```

## Regras

1. Todo request deve ter resposta.
2. Eventos não têm `id`.
3. Erros devem ter `code`, `message` e `details`.
4. O protocolo deve ser versionado.
5. Toda alteração no protocolo exige atualização de docs e schemas.
6. A UI não deve interpretar logs brutos quando houver evento estruturado.

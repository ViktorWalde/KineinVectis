# 03 — Protocolo IPC

> **Escopo:** este documento descreve o protocolo **implementado** hoje
> (JSON-RPC 0.55.0: `core.*`, `tools.*`, `workspace.*`, `fs.*`, `draft.*`,
> `format.*`, `cmake.*`, `cargo.*`, `runConfig.*`, `settings.*`, `debug.*`,
> `git.*`, `build/test/quality.run`,
> `lsp.*`, `syntaxTree.*`, `run.*`, `terminal.*`, `aiBridge.*`). O
> protocolo-**alvo** completo (setup, targets, contexto semântico profundo,
> etc.) está em
> `docs/specs/KINEIN_VECTIS_INTERNAL_ARCHITECTURE_CORE_IPC_JOBS.md`. Onde
> divergir, vale o que está implementado no código + `ContextoIA.md`.

## Objetivo

O protocolo IPC permite comunicação entre:

```text
Kinein Vectis UI  ←→  Kinein Vectis Core
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

### Diagnóstico comum (`Diagnostic`)

Implementado no protocolo `0.20.0` como modelo comum para Problems. Os eventos
continuam sendo por domínio (`event.build.diagnostic`,
`event.quality.diagnostic`, `event.lsp.diagnostics`), mas os itens de
diagnóstico usam campos comuns:

```text
Diagnostic {
  id?,
  source: "build|quality|lsp|toolchain",
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
`source: "lsp"` / `category: "lsp"`. `id`, `command`, `target` e `logRef` já
existem no tipo de protocolo, mas ainda só aparecem quando um produtor tiver
dado real para preencher.

`endLine`/`endColumn`/`code` entraram no protocolo `0.35.0` (fatia T6):
`event.lsp.diagnostics` passa a mandar o range completo (o core deriva de
`textDocument/publishDiagnostics`; `end` ausente cai de volta ao início) e o
`code` do servidor (string ou número → sempre string). A UI usa o range para
o sublinhado ondulado no editor e a marca na gutter, e o `code` no detalhe da
aba Problemas. Build/quality ainda não preenchem range (sublinhado deles fica
para o Problems 2.0).

### Resultado de `tools.detect` / `tools.status`

Implementado no protocolo `0.2.0`. `tools.detect` sempre executa a detecção e
atualiza o registro interno; `tools.status` responde com o último resultado
conhecido (detectando na primeira chamada).

Inventário atual: cargo, rustc, rustup, rust-analyzer, cmake, ninja, git,
clangd, clang, clang++ (`id: clangxx`), gcc, g++ (`id: gxx`), gdb, lldb,
ripgrep (`rg`) e fd/fdfind.

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

### Scan de ambiente (`environment.scan` — job assíncrono)

Implementado no protocolo `0.20.0`. Roda a mesma detecção de ferramentas de
`tools.detect`, mas como job assíncrono para o fluxo de First Run / Toolchain
Settings. Responde na hora com `{ "jobId" }`, não requer workspace aberto e,
ao concluir, atualiza o mesmo registry consultado por `tools.status`.

```text
event.environment.started   { "jobId", "tools" }
event.environment.tool      { "jobId", "tool": ToolInfo }
event.environment.finished  { "jobId", "success", "total", "detected", "missing", "failed", "tools": [ToolInfo] }
```

Além destes, emite `event.job.created/progress/output/finished`. A UI deve usar
`event.environment.*` para preencher telas de ambiente/toolchain e
`event.job.*` para status bar/lista de jobs. O core nunca instala ferramentas;
`suggestedInstall` continua sendo apenas uma sugestão para ação explícita do
usuário.

### Workspace (`workspace.browse` / `workspace.createFolder` / `workspace.createProject` / `workspace.open` / `workspace.recent.*`)

`workspace.open` foi implementado no protocolo `0.3.0`. `workspace.browse`
foi adicionado no protocolo `0.5.0` para o seletor proprio de workspace da UI.
`workspace.createFolder` e `workspace.createProject` foram adicionados no
protocolo `0.9.0` para permitir o fluxo JetBrains-like de criar pasta/projeto
sem sair da IDE.

`workspace.open` recebe
`{ "path": "/dir" }`, canonicaliza o caminho, identifica o tipo de projeto por
marcadores (precedência: `Cargo.toml` > `CMakeLists.txt` > `pom.xml` >
Gradle > Python) e persiste `.kinein/workspace.json`
(`schemas/workspace.schema.json`).

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "name": "meu-projeto",
    "root": "/home/user/dev/meu-projeto",
    "kind": "rustCargo",
    "markers": ["Cargo.toml", "CMakeLists.txt"],
    "capabilities": { "buildSystems": ["cargo", "cmake"] }
  }
}
```

Desde o protocolo `0.55.0`, `kind` continua sendo a classificação primária
compatível pela precedência de marcadores, enquanto `capabilities.buildSystems`
contém todos os sistemas reconhecidos na mesma varredura. Assim, um repositório
híbrido pode ser `rustCargo` e oferecer Cargo+CMake simultaneamente. O metadata
persistido usa `schemas/workspace.schema.json` 0.2.0; a UI consome o snapshot e
não repete detecção por arquivo.

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

- `empty`: cria diretório vazio e persiste `.kinein/workspace.json`.
- `cppCmake`: cria projeto C++23/CMake strict e target-based com
  `CMakeLists.txt`, presets Ninja Debug/Release, `src/main.cpp`, diretórios
  `include/` e `tests/`, `.gitignore` e `README.md`.
- `rustCargo`: usa `cargo new --bin --vcs none`; se `cargo` não existir,
  retorna `TOOL_NOT_FOUND`.

`workspace.status` responde `{ "workspace": <objeto acima> | null }`.
`workspace.close` responde `{ "status": "ok", "closed": <root | null> }`.

**Sessão por workspace** (protocolo `0.24.0`, fatia M1.5 de `docs/18`): a
resposta de `workspace.open` ganha o campo opcional
`session: { openFiles: ["/abs/..."], activeFile? }`, presente apenas quando
`.kinein/session.json` existe, tem `schemaVersion` conhecida (1) e ao menos
um arquivo ainda válido — arquivos apagados/fora do root são filtrados no
carregamento e schema desconhecido é ignorado sem erro. A escrita é feita por
`workspace.saveSession { openFiles: ["/abs/..."], activeFile? }` →
`{ files: N }` (N = entradas efetivamente salvas; caminhos inválidos são
pulados, nunca falham a sessão inteira). Em disco os caminhos são RELATIVOS
ao root (mover a pasta do projeto preserva a sessão); no contrato IPC são
sempre absolutos canônicos. A UI salva com debounce (~1.2s) a cada mudança de
abas/aba ativa e restaura pedindo `fs.read` na ordem da sessão, com a aba
ativa por último.
Caminho inexistente ou sem `path` nos params retorna `INVALID_PARAMS`; falha de
IO ao persistir, listar ou criar diretorios retorna `INTERNAL_ERROR`.

**Workspaces recentes globais** (protocolo `0.53.0`, fatia A1 de `docs/18`):
somente `workspace.open` e `workspace.createProject` concluídos com sucesso
registram a raiz canônica. O core persiste até 12 entradas em
`$XDG_CONFIG_HOME/kinein-vectis/recent-workspaces.json` (ou
`~/.config/kinein-vectis/`), conforme
`schemas/recent-workspaces.schema.json`; a UI nunca consulta o filesystem.

```text
workspace.recent.list {} → { workspaces: [RecentWorkspace] }
workspace.recent.pin { root, pinned } → mesmo snapshot completo
workspace.recent.remove { root } → mesmo snapshot completo
workspace.recent.clear {} → { workspaces: [] }

RecentWorkspace {
  name: string,
  root: string,
  lastOpenedAt: inteiro (Unix epoch em milissegundos),
  pinned: bool,
  available: bool
}
```

- entradas fixadas vêm primeiro; cada grupo é ordenado por
  `lastOpenedAt` decrescente e uma raiz canônica nunca é duplicada;
- `available` é recalculado pelo core em cada snapshot. Uma pasta removida
  continua listada, desabilitada e removível, sem ser aberta pela UI;
- abrir uma entrada reutiliza `workspace.open` e, portanto, restaura a sessão
  por workspace já existente;
- o formato legado `schemaVersion: 0` sem `pinned` é aceito e promovido na
  próxima escrita. JSON inválido ou schema futuro é tratado como lista vazia;
- pin/remover exigem uma raiz absoluta já registrada e retornam
  `INVALID_PARAMS` caso contrário. Erro de escrita retorna `INTERNAL_ERROR`;
  uma falha ao atualizar o histórico não desfaz uma abertura válida;
- o arquivo guarda apenas nome, raiz, último acesso e fixação: nunca conteúdo,
  credenciais ou contexto de IA.

### Arquivos (`fs.list` / `fs.read` / `fs.createFile` / `fs.createDirectory` / `fs.write` / `fs.rename` / `fs.delete` / `fs.replace`)

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
- `fs.write { path, content, expectedContent }` → `{ path, bytesWritten }`.
  Desde o protocolo `0.45.0`, sobrescreve apenas arquivos existentes cujo
  conteúdo atual ainda seja idêntico a `expectedContent` (o último snapshot
  lido pela UI). Divergência retorna `FILE_CHANGED` com `details.path` e não
  toca o disco. A substituição aceita continua atômica (temp irmão + `fsync` +
  `rename`).
- `fs.rename { from, to }` → `{ from, to }`. Criado no protocolo `0.19.0`;
  renomeia ou move um arquivo ou diretório dentro do workspace. `from` precisa
  existir; `to` não pode já existir e seu diretório pai precisa existir dentro
  do workspace. A raiz do workspace não pode ser renomeada (`INVALID_PARAMS`).
- `fs.delete { path }` → `{ path }`. Criado no protocolo `0.19.0`; remove um
  arquivo ou diretório (recursivo para diretórios) dentro do workspace. A raiz
  do workspace não pode ser removida (`INVALID_PARAMS`).

**Mudanças externas (protocolo `0.45.0`, T2):** ao abrir o workspace, o core
inicia `notify` com backend nativo (`inotify` no Linux) e fallback por polling.
O registro é lazy e não recursivo: raiz, diretórios expandidos por `fs.list` e
diretórios de arquivos abertos por `fs.read`. Build/caches e temporários de
save atômico são ignorados. Eventos brutos são deduplicados por uma janela de
180 ms:

```text
event.fs.changed {
  changes: [{ path: "/abs/file", kind: "created|modified|deleted" }]
}
event.fs.watchError { message }
```

A UI recarrega automaticamente uma aba limpa; se houver edição local, preserva
o buffer e exige a escolha explícita entre recarregar o disco ou manter o
local. `event.fs.watchError` é visível, mas a proteção compare-before-save
continua ativa mesmo sem watcher.

### Formatação de buffer (`format.text`)

Implementado no protocolo `0.21.0` (fatia M1.1 de
`docs/18-daily-driver-plan.md`). Requer workspace aberto. Formata o conteúdo
do editor com a ferramenta do projeto via stdin/stdout, sem tocar o disco —
salvar continua sendo decisão do usuário. O formatter roda com cwd na raiz do
workspace, então `rustfmt.toml`/`.clang-format` do projeto valem.

- `format.text { path, text }` →
  `{ path, text, changed, formatter }`.
- `path` precisa existir e estar confinado ao workspace (`INVALID_PARAMS` se
  escapar); é ecoado canonicalizado na resposta para a UI descartar respostas
  de uma aba que já mudou.
- Formatter por extensão: `.rs` → `rustfmt` (`--emit stdout`; acrescenta
  `--edition 2021` apenas quando o root não tem `rustfmt.toml`/
  `.rustfmt.toml`); `.c/.cc/.cpp/.cxx/.h/.hh/.hpp/.hxx` → `clang-format`
  (`--assume-filename=<path> --style=file --fallback-style=LLVM`). Extensão
  sem formatter → `INVALID_PARAMS`.
- `changed: false` quando a saída é idêntica ao texto enviado.
- Binário ausente no `PATH` → `TOOL_NOT_FOUND` (com `data.tool`); formatter
  com exit ≠ 0 → `INTERNAL_ERROR` com o stderr na mensagem.
- Operação síncrona por ser curta (um buffer); "formatar workspace inteiro"
  viraria job, e fica fora deste contrato.

### Busca de arquivos (`fs.findFiles`)

Implementado no protocolo `0.16.0`. Requer workspace aberto. O core usa
`fd` para busca por nome de arquivo, respeitando ignores do projeto e evitando
um indexador próprio no MVP.

- `fs.findFiles { query }` →
  `{ matches: [{ path, name }], truncated }`.
- `query` não pode ser vazio (`INVALID_PARAMS`).
- `path` é relativo à raiz do workspace; `name` é o nome do arquivo.
- A busca usa `fd --type f --fixed-strings --hidden --color never`, com
  exclusões explícitas para `.git`, `.kinein`, `.idea`, `.cache`, `target`,
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
  os diretórios `.git`, `.kinein`, `.idea`, `.cache`, `target`, `build` e
  `node_modules`.
- No máximo um match por linha e 500 matches no total; `truncated: true`
  indica que o limite cortou resultados. `preview` é a linha com trim,
  limitada a 200 caracteres.

`fs.replace { query, replacement, caseSensitive? }` foi adicionado no
protocolo `0.48.0` (T4). Ele executa substituição literal confirmada no
projeto, com a mesma política de confinamento/ignores e limites da busca:

- a UI exige confirmação e recusa iniciar enquanto houver editor sujo;
- o core lê e calcula todos os novos conteúdos antes de escrever;
- a gravação multi-arquivo usa a primitiva transacional comum, valida os
  snapshots novamente, escreve de forma atômica e faz rollback se qualquer
  arquivo falhar;
- responde `{ files: ["/abs/..."], replacements: N }`; arquivos binários,
  não UTF-8, grandes demais ou em diretórios ignorados não entram;
- `query` vazia retorna `INVALID_PARAMS`; zero ocorrências é sucesso com
  listas/contador vazios.

### Execução (`run.start` / `run.script` / `run.stdin` / `run.stop`)

Implementado no protocolo `0.12.0`. Requer workspace aberto. O core executa
um comando via `sh -c` na raiz do workspace SEM bloquear o loop de IPC: a
saída chega como notificações assíncronas (mesmo canal dos eventos LSP) e o
processo aceita stdin e cancelamento enquanto roda. Um processo por vez.

- `run.start { command? }` → `{ command }`. Sem `command`, o core deriva o
  padrão do tipo de projeto: `cargo run` para Rust/Cargo; para CMake, o
  único executável em `.kinein/build` (erro claro se não houver ou houver
  mais de um). Outros tipos ainda não têm padrão (`INVALID_REQUEST` com
  mensagem orientando digitar o comando).
- `run.script { path }` → `{ command }` (protocolo `0.55.0`). Aceita somente
  arquivo regular `.sh`, `.bash` ou `.zsh` dentro do workspace. O core
  canonicaliza/confina o caminho e chama `bash`/`zsh` com argv explícito
  (`--`, caminho), sem interpolação por `sh -c`; nomes com espaços ou aspas são
  dados, não sintaxe. Reutiliza os mesmos eventos e a mesma sessão única de
  `run.start`; extensão inválida é `INVALID_PARAMS`.
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

### Terminal (`terminal.open` / `terminal.input` / `terminal.resize` / `terminal.scroll` / `terminal.close`)

Terminal profissional (reescrito no protocolo `0.41.0`, fatia D2 de
`docs/24`). Requer workspace aberto. **PTY real** via `portable-pty` (do
wezterm) rodando o `$SHELL` interativo em `TERM=xterm-256color` na raiz.
Um **emulador VT** (`vt100`) no core mantém o GRID (células com cor/
atributos), cursor e scrollback; a UI recebe o grid PRONTO e só desenha —
sem interpretar ANSI. Suporta cores, prompts com `\r`, barra de progresso
do cargo e TUIs. Desde o protocolo `0.44.0` (D2.3), o manager mantém até 12
sessões simultâneas. Cada `open` cria um id monotônico (`t1`, `t2`, …); todos
os comandos e eventos seguintes carregam esse id.

- `terminal.open {}` → `{ id, shell }`. Cada chamada cria uma sessão nova;
  erro `INVALID_REQUEST` ao atingir o limite de 12 sessões.
- `terminal.input { id, data }` → `{ status: "ok" }`. Encaminha `data` **cru**
  ao PTY. A UI manda CADA tecla (char-a-char), incl. control chars
  (Enter=`\r`, Backspace=`\x7f`, setas=`\x1b[A..D`, Ctrl+letra, …) — não
  linha+Enter.
- `terminal.resize { id, cols, rows }` → `{ status: "ok" }`. Reflui o PTY e o
  grid. A UI calcula cols/rows do tamanho do painel ÷ métrica da fonte mono.
- `terminal.scroll { id, offset }` → `{ status: "ok" }` (D2.2, `0.42.0`).
  Rola o histórico: `offset` linhas acima do fundo (0 = ao vivo; o core
  **clampa** ao tamanho real do scrollback). A UI manda pela roda do mouse e
  faz snap-to-bottom ao digitar. Copiar/colar são 100% UI (singleton
  `Clipboard`), sem RPC.
  **Nunca confie no offset da UI:** até `0.43.0` um offset maior que o
  histórico **derrubava o core** (bug de overflow do `vt100` 0.15 —
  corrigido no 0.16, que satura a subtração). A verdade do offset volta no
  render (`scrollback`), não na resposta.
- `terminal.close { id }` → `{ status: "ok" }`. Fecha só a sessão indicada;
  fechar/trocar o workspace ou encerrar o core fecha todas.

```text
event.terminal.render {           (throttle ~30fps; substitui event.terminal.data)
  "id": string,                  (0.44.0 — sessão dona deste grid)
  "cols": u16, "rows": u16,
  "cursor": { "row": u16, "col": u16, "visible": bool },
  "alternateScreen": bool,       (0.51.0 — TUI em tela alternativa)
  "applicationCursor": bool,     (0.51.0 — setas SS3 quando solicitado)
  "bracketedPaste": bool,        (0.51.0 — paste delimitado e seguro)
  "scrollback": usize,            (0.43.0 — offset ATUAL, já clampado: a verdade)
  "scrollbackMax": usize,         (0.43.0 — quanto histórico existe; 0 = nenhum)
  "lines": [ [ { "text": str, "fg"?: idx|"#rrggbb", "bg"?: idx|"#rrggbb",
                 "bold"?: bool, "italic"?: bool, "underline"?: bool,
                 "inverse"?: bool } ... ] ... ]
}
event.terminal.closed  { "id": string, "exitCode": int|null }
```

`fg`/`bg`: índice 0–255 (paleta) ou `#rrggbb` (truecolor); ausência = cor
default do tema. Cada linha é uma lista de SPANS (runs de células de mesmo
estilo). O core coalesce runs e descarta o espaço final em estilo default.

`scrollback`/`scrollbackMax` (`0.43.0`, B1/B2 de docs/24) existem porque a UI
**não tem como saber sozinha** se há histórico nem onde a view está: o core é
quem clampa. Sem eles não dá pra desenhar barra de rolagem honesta, e a UI
acabava pedindo offsets impossíveis. A UI trata `scrollback` como fonte da
verdade (reconcilia o estado local a cada render).

Desde `0.51.0`, o render também expõe os modos VT que alteram a tradução de
entrada. A UI respeita application cursor, envolve colagens com bracketed
paste quando a aplicação o pede e continua sem interpretar a interface do
programa. Resize de painel é coalescido antes de `terminal.resize`, evitando
reserializar um grid por pixel durante o arrasto.

### AI CLI Bridge (`aiBridge.profiles` / `aiBridge.terminal.open`)

Implementado inicialmente no protocolo `0.50.0`; a paridade terminal-first foi
consolidada no `0.51.0` e o dimensionamento/scroll contínuos no `0.52.0`. O
**KV Context** é uma superfície separada
na UI, mas sua execução reutiliza o mesmo `TerminalManager`, PTY real,
`terminal.input/resize/scroll/close` e eventos `event.terminal.*`. Não existe
segundo emulador de terminal nem chamada de API de IA dentro da IDE.

```text
aiBridge.profiles {}
  → { profiles: [{ id: "claude"|"codex", name, command, available }],
      defaultProfile: "claude"|"codex" }

aiBridge.terminal.open { profileId: "claude"|"codex" }
  → { id, profileId, name, command }
```

- Os perfis iniciais são fixos e permitidos explicitamente: executáveis
  `claude` e `codex`. A detecção consulta o `PATH`; a IDE não instala nem
  autentica essas ferramentas.
- Claude é a preferência padrão. A seleção bem-sucedida é persistida em
  `settings.aiCliProfile`, e o usuário pode encerrar a sessão e trocar de
  perfil quando quiser.
- `aiBridge.terminal.open` exige workspace aberto e inicia diretamente o
  executável encontrado, na raiz do workspace, sem montar comando shell com
  entrada livre. CLI ausente retorna `TOOL_NOT_FOUND` com instrução de
  instalação.
- O perfil Codex acrescenta o argumento fixo e allowlisted
  `--no-alt-screen`, opção oficial da própria CLI. Isso mantém o TUI real em
  modo inline e preserva o histórico para a mesma barra/roda/arrasto do
  Terminal integrado. Como versões atuais ainda podem emitir `CSI 3 J` nesse
  modo, a sessão do bridge remove somente essa sequência de apagar scrollback;
  o Terminal comum continua honrando `clear` integralmente. Claude continua
  sendo iniciado sem argumento imposto.
- Com sessão ativa, a superfície tem divisor livre entre 300 e 720px, persiste
  a largura separadamente do seletor compacto, não fecha a árvore `Project` e
  pode ser ampliada para toda a área de trabalho. Ela reutiliza exatamente o
  mesmo renderer de grade, seleção, clipboard, teclado e scroll do Terminal
  integrado. Em janela estreita, a apresentação é clampada para preservar uma
  faixa editável central sem apagar a preferência do usuário.
- A IDE não envia código ou contexto automaticamente. Eventual rede,
  autenticação e política de dados pertencem à CLI externa iniciada pelo
  usuário.
- Uma terceira opção para comandos de outras IAs está deliberadamente adiada
  até a correção funcional atual ser validada; ela deverá encaminhar o usuário
  a um terminal com instrução explícita, sem ampliar a allowlist deste método.

### Build (`build.run` — job assíncrono)

Implementado no protocolo `0.6.0`; migrado para **job assíncrono** (ver seção
Jobs e `docs/ARCHITECTURE.md` §7). Requer workspace aberto. O core valida de
forma síncrona e responde **na hora** com `{ "jobId": "job_N" }`; o build roda
em background (`cargo build --message-format=json` para Rust/Cargo;
`cmake -S/-B` + `cmake --build` em `.kinein/build` para CMake) e é **cancelável**
via `job.cancel` (mata o processo de build).

Desde `0.55.0`, aceita `{ "buildSystem"?: "cargo"|"cmake"|... }`. Em workspace
híbrido, a seleção explícita precisa existir em
`workspace.capabilities.buildSystems`; sem o campo, o `workspace.kind` primário
preserva o comportamento anterior. Sistema ausente retorna `INVALID_PARAMS`
com a lista disponível, antes de criar job.

Enquanto roda, emite os eventos ricos que a UI consome, agora com `jobId`:

```text
event.build.started     { "jobId", "command": "cargo build" }
event.build.output      { "jobId", "stream": "stdout|stderr", "line": "..." }
event.build.diagnostic  { "jobId", "source": "build", "category": "compiler", "severity": "error|warning|note", "message", "file"?, "line"?, "column"? }
event.build.finished    { "jobId", "success", "exitCode", "diagnostics" }   // ou { "jobId", "success": false, "error" }
```

Além destes, o Job System emite `event.job.created` (ao iniciar),
`event.job.output` (fan-out da saida bruta) e `event.job.finished` (ao
encerrar) para a status bar / lista de jobs. O
resultado do build chega por `event.build.finished`, **não** mais na resposta.
Diagnósticos vêm do JSON do cargo (span primário) ou do formato
`arquivo:linha:coluna: nivel: mensagem` de compiladores/CMake e alimentam o
Problems. Validação síncrona antes de iniciar o job: tipos sem integração de
build retornam `INVALID_REQUEST`; sem workspace, `INVALID_REQUEST`. Falhas do
build (ferramenta ausente, erro de compilação) chegam por
`event.build.finished`/`event.job.finished`, não como erro da resposta.

### Qualidade / lint (`quality.run` — job assíncrono)

Implementado no protocolo `0.18.0`; migrado para **job assíncrono/cancelável**
como o `build.run`. Requer workspace aberto. Responde na hora com `{ "jobId" }`;
para Rust/Cargo roda `cargo clippy --all-targets --message-format=json` em
background, cujo JSON é idêntico ao do `cargo build`, então os lints viram
diagnósticos estruturados sem parser novo. Emite, com `jobId`,
`event.quality.started/output/diagnostic/finished` (mesmos formatos dos
`event.build.*`) + `event.job.*`; a saida bruta tambem faz fan-out para
`event.job.output`. Diagnosticos usam o mesmo payload de build, mas com
`source: "quality"` e `category: "lint"`. A UI adiciona os diagnósticos à aba Problemas
com origem `quality`. `job.cancel` mata o clippy. Validação síncrona: tudo que
não é Rust/Cargo retorna `INVALID_REQUEST` (CMake via clang-tidy é o próximo
passo). Falhas (`cargo` ausente etc.) chegam por `event.quality.finished`.
O parâmetro opcional `buildSystem` de `0.55.0` é tipado pelo mesmo enum; hoje a
única capacidade executável por `quality.run` continua sendo `cargo`.

### Testes (`test.run` — job assíncrono)

Implementado no protocolo `0.17.0`; migrado para **job assíncrono/cancelável**.
Requer workspace aberto. Responde na hora com `{ "jobId" }`; roda o runner do
tipo de projeto em background (`cargo test` para Rust/Cargo; `ctest --test-dir
.kinein/build --output-on-failure` para CMake) e transmite cada caso conforme
sai da saída do runner. Aceita `{ "filter"? }` (posicional do cargo; `-R` do
ctest) e, desde `0.55.0`, `{ "buildSystem"? }` com a mesma validação de
capacidade do build. `job.cancel` mata o runner.

```text
event.test.started   { "jobId", "command": "cargo test" }
event.test.output    { "jobId", "stream": "stdout|stderr", "line": "..." }
event.test.case      { "jobId", "name": "modulo::caso", "status": "passed|failed|ignored" }
event.test.finished  { "jobId", "success", "exitCode", "passed", "failed", "ignored" }
```

Além destes, `event.job.created`/`event.job.output`/`event.job.finished`. Cada
`event.test.output` tambem gera `event.job.output { "jobId", "line" }` para o
historico generico do job. Os casos são extraídos das linhas
`test <nome> ... ok|FAILED|ignored` (libtest) e `... Test #N: <nome> ...
Passed|***Failed` (ctest); a linha de resumo do libtest é ignorada. Tipos sem
integração retornam `INVALID_REQUEST` (síncrono, antes do job); o resultado vem
em `event.test.finished`, não na resposta.

> **Nota:** com build/quality/test todos como jobs assíncronos, não existe mais
> caminho de streaming síncrono no core — toda operação longa retorna `jobId` e
> emite eventos pelo canal assíncrono.

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
  "detail"?, "kind"? }], "isIncomplete": bool }`. O core ordena por `sortText`
  e limita a 100 itens; `kind` é o `CompletionItemKind` numérico do LSP
  achatado em texto (`function`, `variable`, ...). **`isIncomplete`
  (protocolo `0.37.0`)** é `true` quando o servidor marcou a lista incompleta
  OU o core truncou (ex.: `std::c` no clangd, centenas de candidatos): a UI
  então **repede** completion ao digitar mais (prefixo maior → itens que não
  couberam, como `cout`), em vez de filtrar só o cache. Sem isso, itens fora
  dos primeiros N nunca apareciam.
- `lsp.references` responde `{ "references": [{ "path", "line", "column" }] }`
  (1-based, limitado a 200 usos, incluindo a declaração).
- `lsp.rename` consulta o servidor e normaliza o `WorkspaceEdit` para a
  transação descrita abaixo. Renames que criam/renomeiam/apagam arquivos
  (resource operations) ainda não são suportados e retornam erro estruturado
  sem tocar em nada.

`lsp.semanticTokens` foi adicionado no protocolo `0.14.0`. Desde `0.54.0`,
recebe `{ path, content, version }`, sincroniza o buffer
e resolve `textDocument/semanticTokens/full`, decodificando os deltas com a
legend anunciada pelo servidor no `initialize`. Responde
`{ path, version, tokens: [{ line, start, length, kind }] }` com `line` 1-based e
`start`/`length` em unidades UTF-16 (0-based) — os mesmos índices de
`QString`, aplicados direto pelo highlighter da UI. `kind` é o nome da
legend (`variable`, `function`, `parameter`, `class`, ...). Servidores sem
suporte respondem lista vazia. A UI incrementa a versão no instante da edição,
limpa tokens anteriores e só aplica resposta cujo `path`+`version` ainda
corresponde ao buffer ativo; o tempo de chegada do LSP nunca substitui a base
Tree-sitter de uma versão mais nova.

`lsp.codeActions` e `lsp.applyCodeAction` foram adicionados no protocolo
`0.22.0` (fatia M1.3 de `docs/18-daily-driver-plan.md`):

- `lsp.codeActions { path, content, line, column }` →
  `{ actions: [{ title, kind? }] }`. O core sincroniza o buffer, envia
  `textDocument/codeAction` com range-ponto no cursor e compõe o `context`
  com os diagnostics que o próprio servidor publicou para a linha (cache da
  thread leitora — a UI não devolve diagnóstico). Só entram na lista ações
  `CodeAction` literais com `edit` inline e sem `disabled`; ações que
  dependem de `workspace/executeCommand` são filtradas (decisão registrada
  em docs/18). As ações cruas ficam guardadas como **consulta ativa**.
- `lsp.applyCodeAction { path, content, actionIndex }` cria a mesma transação
  confirmável de `lsp.rename`. A consulta é consumida na chamada; índice
  inválido, arquivo diferente ou consulta
  expirada (qualquer didChange real invalida) retornam `INVALID_PARAMS`
  pedindo nova consulta.

**Workspace edits confirmáveis (protocolo `0.47.0`):** rename e code action
respondem primeiro com um preview:

```text
{ transactionId, title, edits,
  files: [{ path, before, after }] }
lsp.workspaceEdit.apply  { transactionId }
  → { transactionId, files, edits }
lsp.workspaceEdit.cancel { transactionId }
  → { transactionId, cancelled: true }
```

O core confina todos os paths, rejeita versões LSP incompatíveis, mantém um
número limitado de planos pendentes e compara cada snapshot do disco outra
vez no `apply`. A escrita multi-arquivo compartilha a primitiva transacional
do `fs.replace`: arquivos atômicos e rollback de todos os já gravados em caso
de falha. Só após sucesso editor, watcher e LSP são ressincronizados. A UI
mostra `before`/`after` lado a lado e nunca aplica edits por conta própria.

### Sintaxe incremental (`syntaxTree.update`)

Implementado no protocolo `0.46.0` para C, C++ e Rust. Recebe
`{ path, content, version }` e responde:

```text
{ path, language, version, hasErrors,
  highlights: [{ line, start, length, scope }],
  foldingRanges: [{ startLine, endLine }],
  outline: [{ name, kind, line, column, endLine, children? }],
  locals: [{ kind, name?, line, column, endLine, endColumn }] }
```

`start`/`length` e colunas usam UTF-16 para casar com Qt/LSP. O core usa
Tree-sitter incremental com cache LRU de 32 buffers e limite de 4 MiB; versão
obsoleta é descartada na UI. A composição visual é regex fallback <
Tree-sitter < semantic tokens LSP < diagnósticos/busca. `locals` é índice
sintático local, nunca promovido a referência semântica.

`lsp.documentSymbols` e `lsp.workspaceSymbols` foram adicionados no
protocolo `0.23.0` (fatia M1.4 de `docs/18-daily-driver-plan.md`):

- `lsp.documentSymbols { path, content }` →
  `{ symbols: [{ name, kind, path, line, column, container? }] }`. Achata
  os dois shapes do LSP (`DocumentSymbol[]` hierárquico em pré-ordem, com o
  pai como `container`, ou `SymbolInformation[]` plano), preservando a
  ordem do documento; `kind` é o `SymbolKind` nomeado (`struct`, `method`,
  `function`, ...). Cap de 500 símbolos.
- `lsp.workspaceSymbols { path, content, query }` → mesmo shape. `query`
  não vazia (`INVALID_PARAMS` caso contrário), casada server-side via
  `workspace/symbol` **no servidor da linguagem do arquivo ativo** (`path`)
  — consulta multi-servidor está fora do contrato por ora. Cap de 100.
- Posições 1-based; `path` dos símbolos é canônico/absoluto (a UI abre e
  salta direto, como faz com diagnósticos). URIs fora de `file://` são
  ignoradas.
- Na UI, ambos alimentam o Search Everywhere: prefixo `@` lista/filtra a
  estrutura do arquivo atual; `#nome` busca no workspace.

`lsp.switchSourceHeader` foi adicionado no protocolo `0.34.0` (fatia T1
de `docs/18-daily-driver-plan.md`, trilha T de `docs/21`):

- `lsp.switchSourceHeader { path, content }` → `{ path: string | null }`.
  Alterna entre header e source do mesmo componente C/C++ via a
  **extensão do clangd** `textDocument/switchSourceHeader` (params é o
  `TextDocumentIdentifier` PLANO `{ uri }`, sem envelope; resposta é uma
  URI ou `null`). O core sincroniza o documento antes e converte a URI
  de volta ao caminho absoluto.
- Gate de linguagem no core: só o servidor C/C++ tem a extensão —
  arquivo de outra linguagem → `INVALID_PARAMS` (`UnsupportedFile`),
  nunca deixando o rust-analyzer responder "method not found" cru. Sem
  contraparte → `path` ausente (não é erro). clangd ausente →
  `TOOL_NOT_FOUND`. Na UI: atalho `Alt+O` + comando "C/C++: Alternar
  header/source" no Search Everywhere; a UI abre o contraparte reusando
  o caminho de abertura por diagnóstico.

`lsp.restart` foi adicionado no protocolo `0.39.0` (fatia M4.3b — robustez
do LSP travado):

- `lsp.restart { language? }` → `{ restarted: [string] }`. Reinicia o
  servidor de UMA linguagem (`"rust"`|`"cpp"`); sem `language`, reinicia
  TODOS os vivos. Reiniciar = matar o processo e removê-lo; sobe de novo
  (lazy) no próximo request. `restarted` lista as linguagens que tinham
  servidor rodando.
- **Auto-restart:** o core conta timeouts CONSECUTIVOS por servidor
  (`REQUEST_TIMEOUT` de 4s); ao chegar a 3 seguidos (~12s preso), reinicia
  aquele servidor sozinho. Qualquer resposta zera a contagem.
- Novo evento `event.lsp.restarted { language }` (auto OU comando): a UI
  re-sincroniza o arquivo ativo (o servidor novo não conhece os documentos
  abertos — `refreshSemanticTokens` refaz o `didOpen`, mesmo caminho do
  `recovered()` do crash). Na UI: comando "LSP: Reiniciar servidor" no
  Search Everywhere.

Restart/cancelamento de requests LSP ainda não fazem parte deste contrato.

```text
event.lsp.status       { "language": "cpp|rust", "status": "running|failed|stopped|exited", "message"? }
event.lsp.diagnostics  { "path": "/abs/file", "diagnostics": [{ "source": "lsp", "category": "lsp", "severity": "error|warning|note", "message", "line", "column" }] }
```

`line` e `column` dos diagnósticos são 1-based para consumo direto da UI. A
UI integra esses eventos à aba Problemas com origem `lsp`.

### CMake service (`cmake.configure` / `cmake.presets.list` / `cmake.targets.list` / `cmake.status`)

Implementado no protocolo `0.25.0` (fatia M2.2 de `docs/18`). Todos exigem
workspace aberto com kind `cmake` (`INVALID_PARAMS` para outros kinds). O
diretório de build é único e imutável: `<root>/.kinein/build` — o mesmo de
`build.run` e `run.start`; presets **não** mudam o diretório (o `-B`
explícito tem precedência).

- `cmake.configure { preset? }` → `{ jobId }` (job cancelável). Escreve a
  query `codemodel-v2` do file-api antes de rodar e sempre passa
  `-DCMAKE_EXPORT_COMPILE_COMMANDS=ON`. Eventos: `event.cmake.started
  { jobId, command }` e `event.cmake.finished { jobId, success, exitCode,
  hasCompileCommands }`; saída linha a linha vai por `event.job.output`.
- `cmake.presets.list {}` → `{ presets: [{ name, displayName? }] }` —
  configure presets não ocultos de `CMakePresets.json` +
  `CMakeUserPresets.json`, na ordem dos arquivos; JSON inválido → erro
  humano.
- `cmake.targets.list {}` → `{ targets: [{ name, kind }] }` — lidos da
  resposta codemodel-v2 do file-api do último configure (`kind`:
  `executable`, `staticLibrary`, ...); vazio antes do primeiro configure.
- `cmake.status {}` → `{ configured, hasCompileCommands, buildDir }` —
  stat de `CMakeCache.txt`/`compile_commands.json`.
- clangd: quando `compile_commands.json` existe no build dir, novos
  servidores cpp sobem com `--compile-commands-dir` apontando para ele.
  Servidor já em execução não recarrega flags de arquivos abertos —
  configure e reabra o arquivo (ou o workspace).

### Cargo service (`cargo.metadata` / `cargo.check`)

Implementado no protocolo `0.26.0` (fatia M2.3 de `docs/18`). Ambos exigem
workspace aberto com kind `rustCargo` (`INVALID_PARAMS` para outros kinds).

- `cargo.metadata {}` → `{ packages: [{ name, version, features: [...],
  targets: [{ name, kind }] }], workspaceMembers: [...] }`. Síncrono:
  `cargo metadata --format-version 1 --no-deps` é local e rápido (medido
  ~10ms num projeto pequeno); o core resume o JSON gigante do cargo.
  Falha vira erro humano (`INTERNAL_ERROR` com a última linha do stderr).
- `cargo.check {}` → `{ jobId }` (job cancelável). Roda `cargo check
  --workspace --all-targets --message-format=json` e **reusa o pipeline do
  quality**: diagnósticos chegam por `event.quality.diagnostic` e o fim
  por `event.quality.finished` — caem na aba Problemas sem parser novo.
  Aliasing consciente até o Problems 2.0 ter facetas por origem; o título
  do job ("Cargo Check") o distingue do clippy na aba Jobs.

### Run configurations (`runConfig.list` / `runConfig.save` / `runConfig.delete` / `runConfig.setActive`)

Implementado no protocolo `0.27.0` (fatia M2.4 de `docs/18`). Todos exigem
workspace aberto (qualquer kind). Persistência em `.kinein/runconfigs.json`
com `schemaVersion` (arquivo inválido/schema desconhecido = vazio, nunca
quebra). Uma config v1 é `{ id, name, command }` — comando shell executado
na raiz pelo `run.start`.

- Todas as quatro operações respondem o mesmo shape:
  `{ configs: [{ id, name, command }], activeId? }` — a UI nunca calcula
  estado derivado.
- `runConfig.save { id?, name, command }`: cria quando `id` está ausente
  (ids `cfg-N`); criar/editar torna a config a ATIVA. Nome/comando vazios →
  `INVALID_PARAMS`.
- `runConfig.delete { id }`: remove; se era a ativa, volta a "automático".
- `runConfig.setActive { id? }`: `id` ausente = automático; id inexistente →
  `INVALID_PARAMS`.
- `run.start {}` (sem `command`) resolve: comando explícito > config ativa >
  heurística (`cargo run` / executável único do CMake).

### Settings (`settings.get` / `settings.set`)

Implementado no protocolo `0.36.0` (fatia M4.1 de `docs/18`). Dois níveis:
GLOBAL (`$XDG_CONFIG_HOME` ou `~/.config`, então `kinein-vectis/
settings.json`, vale para todos os workspaces) e por-WORKSPACE
(`.kinein/settings.json`, sobrepõe o global campo a campo). Ambos com
`schemaVersion` (arquivo inválido/schema desconhecido = vazio, nunca
quebra). O efetivo = `default ← global ← workspace`.

```text
settings.get {} → SettingsResult
settings.set { scope: "global"|"workspace", values: SettingsValues }
             → SettingsResult
SettingsValues { formatOnSave?: bool, editorFontSize?: u32,
                 autoClosePairs?: bool,
                 rigorProfile?: "strict"|"balanced"|"relaxed",
                 explorerWidth?: u32, contextWidth?: u32,
                 assistantTerminalWidth?: u32,
                 bottomPanelHeight?: u32, outlineWidth?: u32,
                 outlineCollapsed?: bool,
                 aiCliProfile?: "claude"|"codex" }
                                          (campos ausentes = não setados)
EffectiveSettings { formatOnSave, editorFontSize, autoClosePairs,
                    rigorProfile, explorerWidth, contextWidth,
                    assistantTerminalWidth,
                    bottomPanelHeight, outlineWidth, outlineCollapsed,
                    aiCliProfile }
SettingsResult { settings: EffectiveSettings, global: SettingsValues,
                 workspace: SettingsValues }
```

- `settings.get` NÃO exige workspace (o global existe sempre; sem
  workspace, `workspace` vem vazio). Padrão de mutação do repo: `set`
  responde o estado COMPLETO novo.
- `settings.set` faz MERGE parcial de `values` no escopo (campo ausente
  mantém o valor gravado; reverter/limpar override é pós-v1). Defaults:
  `formatOnSave=false`, `editorFontSize=14`, `autoClosePairs=true`,
  `rigorProfile="strict"`, larguras `280/360/640/220` (Project, seletor KV,
  terminal KV ativo e Estrutura), painel inferior `260`, Estrutura expandida e
  `aiCliProfile="claude"`.
- Erros: `NO_WORKSPACE` (set `scope=workspace` sem workspace);
  `INVALID_PARAMS` (`editorFontSize` fora de 8..=40, ou `rigorProfile`
  fora do enum; largura/altura de painel fora dos limites, inclusive terminal
  KV ativo fora de 300..=720); `INTERNAL_ERROR`
  (falha de escrita).
- Consumidores atuais: `editorFontSize` → `Theme.fontSizeEditor`;
  `autoClosePairs` → auto-close da E1; `formatOnSave` → Ctrl+S formata e
  então salva; `rigorProfile` (M4.5) → flags de `quality.run`/`build.run`
  NO PROJETO DO USUÁRIO (clippy pedantic/nursery + `-D warnings` no
  strict; clippy default no balanced; só `clippy::correctness` +
  build sem `RUSTFLAGS` no relaxed). NUNCA regula o gate do repo Kinein.
  As dimensões persistem o layout que o usuário redimensionou;
  `outlineCollapsed` persiste o recolhimento da Estrutura; `aiCliProfile`
  define a preferência inicial do KV Context.
  `diffBase` (head|index) fica para uma micro-fatia futura (precisa de
  `base` no `git.fileDiff`).

### Rascunhos / autosave (`draft.save` / `draft.clear`)

Rede de segurança contra perda de dado (protocolo `0.40.0`, fatia S1 de
`docs/23`). O core persiste em **SQLite** (`.kinein/kinein.db`, WAL +
escrita atômica por transação) o buffer NÃO SALVO de arquivos sujos. Se a
UI cair (ou power loss), no próximo `workspace.open` as edições voltam.
Complementa a escrita atômica de `fs.write` (temp + `fsync` + `rename`, que
elimina o arquivo truncado/zerado em crash).

```text
draft.save { path, content } → { savedAt }   (autosave de buffer sujo)
draft.clear { path }          → { ok: true }  (save/close limpo)
workspace.open ... → { ..., drafts?: [{ path, content, savedAt }] }
```

- `path` é absoluto e confinado à raiz (como `fs.*`); a chave é o caminho
  absoluto. A UI autosalva com debounce (~1.5s) só quando o buffer está
  modificado; ao fechar a aba manda `draft.clear`.
- `fs.write` **também** limpa o rascunho do path salvo (o arquivo em disco
  passou a ser a verdade). Então rascunho **só sobrevive a um CRASH**.
- Recuperação: embutida na resposta de `workspace.open` (`drafts`), já
  FILTRADA — rascunho igual ao disco é obsoleto e apagado; só volta o que
  DIFERE. A UI abre a aba, sobrepõe o buffer e a marca MODIFICADA (o
  savedContent segue o disco: salvar/reverter continuam corretos).
- Erros: `NO_WORKSPACE` (sem workspace); `INVALID_PARAMS` (path fora da
  raiz / inexistente); `INTERNAL_ERROR` (store indisponível/falha).

### Debug (`debug.*` — sessao DAP via lldb-dap)

Implementado no protocolo `0.28.0` (fatia M2.5a de `docs/18`). O core
orquestra o `lldb-dap` (pacote `lldb`) pelo Debug Adapter Protocol; a UI
nunca fala DAP — recebe eventos `event.debug.*` ja mastigados. Uma sessao
por workspace.

- `debug.start { program? }` → `{ program }`. Sem `program`, resolve o
  alvo "Automatico" espelhando o run: cargo → unico executavel no topo de
  `target/debug`; cmake → unico executavel de `.kinein/build`; zero ou
  varios candidatos → erro claro com a acao a tomar. NAO compila antes
  (build e acao explicita, Ctrl+F9). Erros: `TOOL_NOT_FOUND` (lldb-dap
  ausente), `INVALID_REQUEST` (sem alvo/sessao ja viva), `INVALID_PARAMS`
  (program inexistente), `INTERNAL_ERROR` (falha do adapter).
- `debug.setBreakpoints { file, lines: [int] }` →
  `{ breakpoints: [{ line, verified }] }`. Conjunto COMPLETO por arquivo
  (lines vazio limpa); `file` confinado ao workspace. Sem sessao viva o
  conjunto e guardado (`verified: false`) e replayado no proximo launch;
  com sessao, o adapter responde o que de fato amarrou.
- `debug.continue` / `debug.next` / `debug.stepIn` / `debug.stepOut`
  exigem processo pausado; `debug.pause` pausa o processo em execucao;
  `debug.stop` desconecta educadamente e mata o adapter. Todos respondem
  `{ status: "ok" }`; exigem apenas o manager (como `run.stop`).

Inspeção (protocolo `0.29.0`, fatia M2.5c), sempre da thread pausada
(`INVALID_REQUEST` fora disso):

- `debug.stackTrace {}` → `{ frames: [{ id, name, file?, line? }] }` —
  topo primeiro, até 20 frames.
- `debug.variables { frameId }` → `{ frameId, variables }` — o core
  resolve os scopes DAP internamente e devolve as variáveis do primeiro
  escopo não-caro (Locals); Globals/Registers ficam pós-M2.
- `debug.variables { ref }` → `{ ref, variables }` — expande uma variável
  estruturada. Variável: `{ name, value, type?, ref }` (`ref` 0 = folha,
  > 0 = expansível). Exatamente um de `frameId`/`ref` → senão
  `INVALID_PARAMS`. A resposta ECOA a chave pedida para a UI correlacionar
  (mesmo padrão do `format.text`).

```text
event.debug.started   { program }
event.debug.output    { category, line }   (stdout|stderr|console)
event.debug.stopped   { reason, file?, line?, threadId }  (file/line podem
                        vir null; o core ja enriquece com o frame do topo)
event.debug.continued {}                   (um por retomada, deduplicado)
event.debug.finished  { exitCode? }        (exatamente um por sessao)
```

### Git (`git.status` e operações diárias)

Implementado no protocolo `0.30.0` (fatia M3.1 de `docs/18`). Orquestra o
binario `git` com saida ESTAVEL (`status --porcelain=v2 --branch
--untracked-files=all -z`); deteccao de repo por exit code
(`rev-parse --is-inside-work-tree`), nunca por mensagem (pode vir
localizada). Sincrono e stateless: a UI decide quando consultar
(open/save/fs-ops/manual).

```text
git.status {} → { repo: bool,
                  branch?, detached?, shortSha?,
                  upstream?, ahead?, behind?,
                  entries: [{ path, kind, staged }] }
kind ∈ modified|added|deleted|renamed|untracked|conflicted (o core
consolida o par XY do porcelain; a UI nunca decodifica porcelain).
```

- Paths das entries são relativos ao ROOT do workspace (convertidos do
  toplevel via `rev-parse --show-prefix`); entries fora do workspace e os
  metadados `.kinein/` são filtrados.
- Workspace sem git → `{ repo: false, entries: [] }` (nunca é erro).
  Erros reais: `NO_WORKSPACE`, `TOOL_NOT_FOUND` (git ausente),
  `INTERNAL_ERROR` (git falhou).

`git.fileDiff { path }` (protocolo `0.31.0`, fatia M3.2) → diff do
arquivo contra o HEAD, em duas granularidades na mesma resposta:

```text
git.fileDiff { path } → { path (canônico, ecoado p/ correlação),
                          repo, tracked,
                          hunks: [{ kind: added|modified|removed,
                                    startLine, lineCount }],
                          text }
```

- `hunks` vem de `--unified=0` (ranges exatos para a gutter; `removed` é
  ancorado na linha seguinte à remoção, lineCount 1); `text` vem de
  `--unified=3` (visão de diff). A UI nunca parseia diff.
- Untracked → `tracked: false` + um hunk `added` do arquivo inteiro e
  `text` vazio. `path` confinado ao workspace (`INVALID_PARAMS` fora).
- Diff é do arquivo EM DISCO (buffer não salvo não aparece — a gutter
  atualiza no save/troca de aba; attach de buffer é melhoria futura).

Mutações (protocolo `0.32.0`, fatia M3.3) — todas respondem o MESMO
shape do `git.status` (a UI atualiza tudo de uma vez):

```text
git.stage    { paths: [abs] } → GitStatusResult   (git add)
git.unstage  { paths: [abs] } → GitStatusResult   (git restore --staged)
git.discard  { paths: [abs] } → GitStatusResult   (DESTRUTIVO: restore
                                 p/ tracked, clean -f p/ untracked; a
                                 confirmação é responsabilidade da UI)
git.commit   { message }      → GitStatusResult   (commita SÓ o staged)
```

- Guarda do commit por exit code estável (`git diff --cached --quiet`):
  nada staged → `INVALID_REQUEST` com mensagem própria. `message` vazia
  e `paths` vazio/fora do root → `INVALID_PARAMS` (path deletado não
  canonicaliza: vale a checagem lexical dentro do root). Mutação em
  workspace sem git → `INVALID_REQUEST` (leitura responde `repo:false`).
  Falha real do git → `INTERNAL_ERROR` com o stderr.

Leitura de histórico (protocolo `0.33.0`, fatia M3.4) — todas com
formato estável (`--porcelain`, `%x1f`/NUL) e sem parsear saída
localizada:

```text
git.blame { path } → { path (ecoado), repo, tracked, groups: [
    { startLine, lineCount, sha, author, authorTime (epoch),
      summary, committed }] }
git.log { maxCount? } → { repo, entries: [
    { sha, shortSha, author, authorTime (epoch), summary }] }
git.commitDiff { sha } → { sha (ecoado), text (patch unificado) }
```

- `git.blame` usa `git blame --porcelain`; metadado de sha repetido é
  memoizado (o porcelain só o manda uma vez). Linha não commitada tem
  sha zerado → `committed:false` (detecção pelo sha, nunca pela string
  "Not Committed Yet"). Untracked/repo recém-init sem HEAD →
  `tracked:false` com `groups` vazio (não é erro). `path` confinado ao
  root e existente em disco → senão `INVALID_PARAMS`.
- `git.log` usa `--pretty=format:%H%x1f%h%x1f%an%x1f%at%x1f%s -z`;
  `maxCount` default 100, fora de `1..=500` → `INVALID_PARAMS`. Repo
  sem commits (sem HEAD, checado por `rev-parse --verify --quiet`) →
  `entries: []`. Fora de repo → `repo:false`.
- `git.commitDiff` usa `git show --pretty=format: --unified=3`; o `sha`
  é validado (4..=64 hex) ANTES de virar argv (barreira contra string
  arbitrária); merge trivial pode devolver `text` vazio. Fora de repo →
  `INVALID_REQUEST` (a chamada só nasce da lista do `git.log`).

Operações diárias adicionadas no protocolo `0.49.0`:

```text
git.branches {} → { repo, current?, detached, branches: [{ name, current }] }
git.checkout { branch } → GitStatusResult
git.branchCreate { name, checkout? } → GitStatusResult
git.pull {} → { jobId }
git.push {} → { jobId }
git.stash { action: "push|pop", message? } → GitStatusResult
```

- nomes de branch são validados antes de virar argumento; checkout/create e
  stash são recusados pela UI quando existem buffers sujos;
- pull/push são jobs canceláveis e publicam saída/resultado pelo Job System;
- stash push inclui untracked, limita-se ao workspace e exclui `.kinein`;
  pop restaura o stash mais recente;
- todas as mutações síncronas devolvem o status inteiro para não duplicar
  estado derivado na UI.

### Jobs (`job.list` / `job.cancel`)

Fundação do Job System (ver `docs/ARCHITECTURE.md` §7). Um **job** é uma
operação longa que o core executa de forma assíncrona: retorna um `id` na hora,
reporta progresso por eventos e pode ser cancelado. A infraestrutura atual
registra ciclo de vida/cancelamento e já é usada por `build.run`, `quality.run`
e `test.run`.

- `job.list` → `{ jobs: [{ id, kind, title, status, progress?, canCancel, risk }] }`.
  `status`: `queued|running|cancelRequested|success|warning|failed|cancelled`;
  `risk`: `low|medium|high|dangerous`. Ordem estável de criação. O core retém
  até 100 jobs em memória, preservando jobs ativos e removendo os finalizados
  mais antigos quando o limite é excedido.
- `job.cancel { jobId }` → `{ jobId, cancelled }`. `cancelled` é `true` só quando
  o job existe, expõe cancelamento e ainda está `running`; ao aceitar o pedido,
  o core muda o status para `cancelRequested` e emite `event.job.progress`.
  `jobId` ausente é `INVALID_PARAMS`. O cancelamento é cooperativo (o trabalho
  verifica o sinal) e o estado terminal chega depois como `cancelled`.

Eventos, todos com `jobId`:

```text
event.job.created   { "id", "kind", "title", "status", "progress"?, "canCancel", "risk" }
event.job.progress  { "jobId", "status": "running|cancelRequested", "progress"?, "message"? }
event.job.output    { "jobId", "line" }
event.job.finished  { "jobId", "status": "success|warning|failed|cancelled" }
```

Regra de UX (specs): `event.job.*` atualizam status bar / tool window; não abrem
pop-up automático. Job `high`/`dangerous` exige confirmação antes de iniciar.

## Métodos principais implementados

```text
core.ping
core.shutdown
workspace.open
workspace.browse
workspace.createFolder
workspace.createProject
workspace.saveSession
workspace.recent.list
workspace.recent.pin
workspace.recent.remove
workspace.recent.clear
workspace.close
workspace.status
command.list
tools.detect
tools.status
environment.scan
fs.list
fs.read
fs.createFile
fs.createDirectory
fs.write
fs.rename
fs.delete
fs.findFiles
fs.search
fs.replace
build.run
test.run
quality.run
run.start
run.script
run.stdin
run.stop
terminal.open
terminal.input
terminal.resize
terminal.scroll
terminal.close
aiBridge.profiles
aiBridge.terminal.open
lsp.didChange
lsp.semanticTokens
lsp.definition
lsp.hover
lsp.completion
lsp.references
lsp.rename
lsp.codeActions
lsp.applyCodeAction
lsp.workspaceEdit.apply
lsp.workspaceEdit.cancel
syntaxTree.update
git.status
git.branches
git.checkout
git.branchCreate
git.pull
git.push
git.stash
job.list
job.cancel
```

## Eventos iniciais

```text
event.build.started
event.build.output
event.build.diagnostic
event.build.finished
event.quality.started
event.quality.output
event.quality.diagnostic
event.quality.finished
event.environment.started
event.environment.tool
event.environment.finished
event.test.started
event.test.output
event.test.case
event.test.finished
event.job.created
event.job.progress
event.job.output
event.job.finished
event.run.started
event.run.output
event.run.finished
event.terminal.render
event.terminal.closed
event.lsp.status
event.lsp.diagnostics
event.fs.changed
event.fs.watchError
```

## Regras

1. Todo request deve ter resposta.
2. Eventos não têm `id`.
3. Erros devem ter `code`, `message` e `details`.
4. O protocolo deve ser versionado.
5. Toda alteração no protocolo exige atualização de docs e schemas.
6. A UI não deve interpretar logs brutos quando houver evento estruturado.

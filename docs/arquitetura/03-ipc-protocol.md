# 03 — Protocolo IPC

> **O `0.94.0` (2026-09-12) acrescentou o domínio `index`:** a IDE passa a
> ler o projeto INTEIRO que abre — todas as pastas, arquivos e declarações
> (funções, tipos) de C, C++, Rust e Python — por decisão do autor no mesmo
> dia, sem esperar language server. `index.status` dá os totais, `index.symbols`
> busca por nome, `event.index.progress`/`finished` acompanham o job que o
> `workspace.open` sobe. É a primeira forma concreta do "entender o projeto
> inteiro" da especificação do KSWE, com as gramáticas Tree-sitter do editor.
> Domínio novo sobe o minor.
>
> **O `0.93.0` (2026-09-12) acrescentou o domínio `project`:** o MODELO do
> projeto embarcado (pilar 0 do `roadmaps/42`). `project.model` diz o que o
> projeto É — framework com o arquivo que o prova, SDKs exigidos e se estão
> aqui, artefatos do último build, alvo deduzido com evidência — e
> `event.project.changed` o reemite ao abrir o workspace e ao fim de
> configure/build. Nada é adivinhado calado; o que não se decide vira `hint`.
> No mesmo minor, `serial.monitor` e o papel `serialMonitor` do kit.
>
> **O `0.92.0` (2026-09-12) acrescentou o domínio `container`:** Docker e
> Podman como domínio NATIVO (decisão do autor de 2026-07-17, `roadmaps/28`
> §0, priorizada em 2026-09-12). `container.status` é a tela de "ativar a
> ferramenta" (motor, versão, rootless, socket, responde, compose, passo
> oficial); `list`/`images` leem `ps`/`images` em JSON dos dois motores;
> `action` e `compose` são JOBS com `event.container.finished`; `open` abre
> logs ou um shell numa aba de terminal. A UI nunca chama `docker`. Domínio
> novo sobe o minor.
>
> **O `0.91.0` (2026-09-11) acrescentou o domínio `serial`:** `serial.list`
> enumera as portas seriais USB desta máquina pelo sysfs — `ttyUSB*` (ponte)
> e `ttyACM*` (CDC) — com VID:PID, driver, o que o VID:PID diz do **elo** (nunca
> do chip atrás de uma ponte), a permissão **medida** com `access(2)` e o estado
> do ModemManager via `udevadm`. Nunca abre a porta: abrir aciona DTR/RTS e
> reseta a placa. Domínio novo sobe o minor. É a E1 do
> [`../integracoes/38`](../integracoes/38-conectividade-bare-metal.md) §6.
>
> **O `0.90.0` (2026-09-11) acrescentou `build.size`:** o tamanho do ELF
> medido por `<prefix>size` do kit, com a fração usada de cada região do
> linker script (`BuildSizeParams` → `SizeReport`). Método e tipos novos sobem
> o minor. Detalhe na seção Build; a medição está no
> [`../roadmaps/35`](../roadmaps/35-ambiente-cpp-embarcados-simulacao.md) §5.7.
>
> **O `0.89.0` (2026-09-11) acrescentou o ALVO REMOTO do depurador:**
> `remoteTarget` e `debugServer` no `toolchain.setKit` e no `ToolchainResult`.
> Com eles o adaptador `gdb` (que fala DAP desde a v14) faz `attach` a um
> servidor GDB — QEMU, OpenOCD — que a IDE sobe e mata com a sessao, em vez de
> `launch`. Campos novos no contrato sobem o minor. A medicao esta' no
> [`../roadmaps/35`](../roadmaps/35-ambiente-cpp-embarcados-simulacao.md) §5.7
> e o ciclo completo e' provado no QEMU pelo `scripts/verificar-embarcado.sh`.
>
> **O `0.88.0` (2026-09-10) acrescentou o veredito de UNIDADE:**
> `SimDimensionCheck` e `SimDimensionVerdict`, no `dimensions` do `sim.run`
> (um) e do `sim.runSystem` (um por componente). Tipos novos no contrato sobem
> o minor. A medicao que os sustenta esta no
> [`../roadmaps/31`](../roadmaps/31-simulacao-fisica-matematica.md) §19.5 — e o
> que ela achou primeiro foram TRES armadilhas que dariam veredito errado em
> silencio.
>
> **RECONFERIDO em 2026-09-10**, com o gate completo verde: `0.87.0`, **130
> métodos**, **41 eventos**, **30 domínios**, e os 30 com seção aqui.
>
> **O `0.87.0` não trouxe método novo — trouxe PROCEDÊNCIA.** O `SimAccuracy` do
> `sim.run` ganhou `source` (`concept` ou `oracle`), `relativeError`,
> `solvedBy` e `closedForm`, e o `SimRunResult`/`SimRunSystemResult` ganharam
> `oracleNote`. O `source` é campo **obrigatório**, e é por isso que o minor
> sobe: quem ler a resposta antiga não o encontra. A razão de ele existir está
> medida no [`../roadmaps/31`](../roadmaps/31-simulacao-fisica-matematica.md)
> §19.0 — sem ele a coluna `exato` respondia por outra equação, e errava por
> 78.000x.
>
> Antes disso, a sincronização de 2026-09-06 e os dois métodos da forma vetorial
> (`sim.checkSystem` e `sim.runSystem`) — e **os comentários dentro dos
> comandos, que ainda diziam 128/130**. Comentário dentro de comando envelhece
> igual a número solto; a diferença é que o gate não o vê.
>
> **Escopo, SINCRONIZADO em 2026-09-06 — e a dívida que este cabeçalho
> declarava foi paga.** Em 2026-09-05 ele foi corrigido para parar de afirmar
> cobertura que não tinha: cinco domínios estavam roteados pelo core e ausentes
> daqui. **Os cinco agora têm seção**: `command.*`, `setup.*`, `datasource.*`,
> `grafana.*` e `sim.*`, no fim do documento.
>
> **E a sincronização achou dois números errados — nos comandos que os provam.**
> Ambos pela mesma causa: eles grepam literais sem saber o que os literais são.
>
> ```bash
> # METODOS: o braco de despacho nunca comeca com `event.`. Sem o filtro, a
> # contagem inclui dois nomes de EVENTO que aparecem num `match` dentro de
> # `#[cfg(test)]` em handlers/build.rs — e da' 132 em vez de 130.
> grep -rhoE '"[a-z][a-zA-Z]*\.[a-zA-Z][a-zA-Z.]*"\s*(\||=>)' \
>      crates/kinein-core/src/handlers/ crates/kinein-core/src/lib.rs \
>   | grep -oE '"[a-z][a-zA-Z]*\.[a-zA-Z][a-zA-Z.]*"' | tr -d '"' \
>   | grep -v '^event\.' | sort -u | wc -l          # 130
>
> # EVENTOS: cinco sao construidos por `format!("event.{domain}.*")` em
> # handlers/build.rs, com `domain` em {build, quality}. Um grep de literal
> # NAO OS VE, e da' 36 em vez de 41.
> { grep -rhoE '"event\.[a-zA-Z.]+"' crates/kinein-core/src/ | tr -d '"'
>   for d in build quality; do
>     for s in started output diagnostic finished; do echo "event.$d.$s"; done
>   done
> } | sort -u | wc -l                                # 41
> ```
>
> ```text
> protocolo   0.88.0
> metodos     131 roteados
> eventos     41 (36 literais + 5 construidos por format!)
> dominios    30, e os 30 tem secao neste documento
> ```
>
> **Os cinco que só existem por `format!`:** `event.build.started`,
> `event.build.output`, `event.build.diagnostic`, `event.quality.started` e
> `event.quality.output`. Os outros três da mesma família aparecem como literal
> em algum ponto do código e por isso o grep os via.
>
> O protocolo-**alvo** completo está em
> `docs/specs/KINEIN_VECTIS_INTERNAL_ARCHITECTURE_CORE_IPC_JOBS.md`. Onde
> divergir, vale o que está implementado no código + `docs-privada/ContextoIA.md`.

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
`event.quality.diagnostic`, `event.project.changed

event.lsp.diagnostics`), mas os itens de
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

**Sessão por workspace** (protocolo `0.24.0`, fatia M1.5 de `docs-privada/diario/18`): a
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

**Workspaces recentes globais** (protocolo `0.53.0`, fatia A1 de `docs-privada/diario/18`):
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
  do workspace não pode ser removida (`INVALID_PARAMS`). **Desde 2026-09-02**,
  se o arquivo estava aberto num language server, o core manda
  `textDocument/didClose`: documento apagado que continua aberto deixa
  diagnóstico de um arquivo que não existe mais na aba Problemas. A forma da
  mensagem IPC não mudou.

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

### Formatação de buffer (`format.text` / `format.capabilities`)

Implementado no protocolo `0.21.0` (fatia M1.1 de
`docs-privada/diario/18-daily-driver-plan.md`). Requer workspace aberto. Formata o conteúdo
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
- `format.capabilities {}` → `{ formatters: [ { id, extensions[] } ] }`
  (`0.61.0`). **Não** requer workspace: é o mapa estático de extensões, derivado
  da mesma constante que `formatter_for_path` usa para decidir — o que a UI
  recebe é, por construção, o que o `format.text` vai aceitar.

  **Invariante de camada.** Quem decide o que é formatável é o core; a UI
  consome. Até `0.60.0` o `EditorController.qml` mantinha duas listas escritas à
  mão (`formattableLanguage` por linguagem, `formattablePath` por extensão) que
  não concordavam entre si nem com o core, e adicionar linguagem exigia editar
  QML. Duas fontes para a mesma verdade divergem por construção.

  Capacidade e disponibilidade são perguntas **distintas**: `format.capabilities`
  responde "existe formatter registrado para esta extensão"; se o binário está no
  `PATH` só se descobre ao rodar `format.text` (`TOOL_NOT_FOUND`). O catálogo não
  muda com o workspace, então a UI pode pedi-lo uma vez por conexão.
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
- No máximo 500 matches no total; `truncated: true` indica que o limite cortou
  resultados. `preview` é a linha com trim, limitada a 200 caracteres.
- **`query` pode conter `\n`** (desde 2026-09-02, protocolo `0.65.0`). A busca
  varre o CONTEÚDO do arquivo, não uma linha de cada vez: `line`/`column`
  apontam para o início do match e o `preview` mostra o trecho inteiro com as
  quebras internas trocadas pela marca ` ⏎ `. Match que atravessa linhas continua
  sendo UM match — é isso que faz o contador do preview bater com o número de
  reescritas.
- **Matches não se sobrepõem**: a varredura retoma no fim do match anterior.
  `aa` em `aaa` são 1 match, não 2, e `fs.replace` reescreve exatamente esse 1.

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
- **`query` e `replacement` aceitam `\n`** (desde 2026-09-02, protocolo
  `0.65.0`). De 2026-08-29 até essa data o core RECUSAVA (`INVALID_PARAMS`), e a
  recusa estava certa para o que existia então: a busca casava **linha a linha**,
  então uma query multi-linha era invisível para o preview e ativa para a
  escrita — o usuário via "0 resultados" e arquivos eram reescritos mesmo assim.
  O que mudou não foi a guarda, foi a busca: ela passou a varrer o conteúdo
  inteiro e a devolver preview do trecho completo. **A guarda saiu porque a razão
  dela saiu** — a invariante que ela protegia ("o preview conta o que a escrita
  vai fazer") agora vale sozinha, e está travada pelos testes
  `multiline_search_previews_exactly_what_replace_will_rewrite` e
  `a_multiline_replacement_is_found_by_the_next_search`.
- **A sintaxe `\n` do painel é da UI, não do protocolo.** O campo de busca é um
  `TextInput` de uma linha, então `SearchController.expandLineBreaks()` traduz a
  sequência de dois caracteres `\n` digitada pelo usuário em quebra de verdade
  antes de chamar o core (e `\\n` devolve o literal barra-ene). Essa tradução
  **não pode descer para o core**: `query` é texto LITERAL, e um core que
  interpretasse escapes tornaria impossível procurar por um `\n` de verdade
  dentro do código.
- **`fs.search` reporta TODAS as ocorrências de cada linha** (desde 2026-08-29).
  Antes parava na primeira: "Alpha alpha" aparecia como 1 resultado e virava 2
  substituições. O preview de uma operação destrutiva tem de contar o que ela
  vai fazer.
- **paridade com `fs.search` (garantida desde 2026-08-29):** os dois percorrem o
  mesmo walk (`fsops::walk`), então `fs.replace` nunca toca arquivo que
  `fs.search` não mostrou. Consequência da unificação: um subdiretório ilegível é
  **pulado** (como sempre foi na busca), e não mais aborta a substituição inteira
  — só a raiz ilegível é erro. Travado pelo teste
  `replace_touches_exactly_the_files_search_reports`.

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

### Terminal (`terminal.open` / `terminal.input` / `terminal.resize` / `terminal.scroll` / `terminal.mouse` / `terminal.close`)

Terminal profissional (reescrito no protocolo `0.41.0`, fatia D2 de
`docs/roadmaps/24`). Requer workspace aberto. **PTY real** via `portable-pty` (do
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

- `terminal.open {}` → `{ id, shell }`. Cada chamada cria uma sessão nova;
  erro `INVALID_REQUEST` ao atingir o limite de 12 sessões.
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
  mouse. Copiar/colar são 100% UI (singleton `Clipboard`), sem RPC.
  **Nunca confie no offset da UI:** até `0.43.0` um offset maior que o
  histórico **derrubava o core** (bug de overflow do `vt100` 0.15 —
  corrigido no 0.16, que satura a subtração; o `alacritty_terminal` clampa por
  conta própria). A verdade do offset volta no render (`scrollback`), não na
  resposta.
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

`scrollback`/`scrollbackMax` (`0.43.0`, B1/B2 de docs/roadmaps/24) existem porque a UI
**não tem como saber sozinha** se há histórico nem onde a view está: o core é
quem clampa. Sem eles não dá pra desenhar barra de rolagem honesta, e a UI
acabava pedindo offsets impossíveis. A UI trata `scrollback` como fonte da
verdade (reconcilia o estado local a cada render).

Desde `0.51.0`, o render também expõe os modos VT que alteram a tradução de
entrada. A UI respeita application cursor, envolve colagens com bracketed
paste quando a aplicação o pede e continua sem interpretar a interface do
programa. Resize de painel é coalescido antes de `terminal.resize`, evitando
reserializar um grid por pixel durante o arrasto.

### IA externa: sem domínio próprio (removido no `0.59.0`)

O domínio `aiBridge.*` **não existe mais**. Ele expunha `aiBridge.profiles` e
`aiBridge.terminal.open`, que abriam `claude`/`codex` por um caminho especial:
argumentos injetados pelo core (`--no-alt-screen`, `--ax-screen-reader`) e um
filtro que engolia `CSI 3 J` da própria aplicação.

Esse caminho era a IDE interferindo no terminal. Ele tratava um agente de CLI
como se fosse diferente de qualquer outro programa, o que produziu sintomas
atribuídos ao painel quando a causa estava no emulador raso (ver
`docs/adr/ADR-0004-alacritty-terminal-emulator.md`).

O modelo agora é o de qualquer IDE profissional: **uma CLI de IA é um programa
como outro qualquer**. Abrir um terminal (`terminal.open`) e digitar `claude`
ou `codex` usa o mesmo PTY, o mesmo emulador e o mesmo contrato
`terminal.input/resize/scroll/close` de um `ls`. Não há allowlist de perfil,
argumento imposto, filtro de sequência nem preferência persistida.

Consequência de produto: rodar um agente dentro da Kinein passa a ser
indistinguível de rodá-lo fora dela — que era o requisito original. Uma
superfície visual de atalho (Assistente) pode voltar depois **como UI pura**,
abrindo uma sessão de terminal comum, sem regra de negócio própria no core.

### `build.size` — o tamanho do ELF (síncrono)

**Desde 2026-09-12 (pilar 0 do `roadmaps/42`)**, num projeto ESP-IDF a lista
de regiões ganha a partição `app` que o `flasher_args.json` aponta — usado =
tamanho da imagem, capacidade = a partição que **começa** naquele offset. O
`.ld` do IDF não declara a flash; a partição é a flash.

Implementado no protocolo `0.90.0`. **Síncrono**, ao contrário do `build.run`:
o `size` lê um arquivo e volta em milissegundos. `{ "program"? }` — ausente, o
core resolve o ELF como o `debug.start`. Roda `<prefix>size -A` (o prefixo vem
do compilador cross do kit: `arm-none-eabi-gcc` → `arm-none-eabi-size`; sem
cross, o `size` do sistema) e responde `SizeReport { toolAvailable, tool,
program, sections: [{ name, size, addr }], regions: [{ name, used, size }],
rawOutput }`. As `regions` saem do bloco `MEMORY` do único `.ld` do workspace,
com `used` = soma das seções **alocadas** cujo endereço cai na região —
`.comment`/`.ARM.attributes` (metadados do ELF, endereço 0) não contam. Sem
linker script legível, `regions` vem vazio e a UI mostra os totais por seção.

### Build (`build.run` — job assíncrono)

Implementado no protocolo `0.6.0`; migrado para **job assíncrono** (ver seção
Jobs e `docs/arquitetura/ARCHITECTURE.md` §7). Requer workspace aberto. O core valida de
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
`0.22.0` (fatia M1.3 de `docs-privada/diario/18-daily-driver-plan.md`):

- `lsp.codeActions { path, content, line, column }` →
  `{ actions: [{ title, kind? }] }`. O core sincroniza o buffer, envia
  `textDocument/codeAction` com range-ponto no cursor e compõe o `context`
  com os diagnostics que o próprio servidor publicou para a linha (cache da
  thread leitora — a UI não devolve diagnóstico). Só entram na lista ações
  `CodeAction` literais com `edit` inline e sem `disabled`; ações que
  dependem de `workspace/executeCommand` são filtradas (decisão registrada
  em docs-privada/diario/18). As ações cruas ficam guardadas como **consulta ativa**.
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

### Documentos fechados após um configure (`event.lsp.documentsClosed`)

Nasceu em 2026-09-02 (etapa 4 do `docs/roadmaps/30-caminho-para-o-mvp.md`).
`{ language, count }`, hoje sempre `language: "cpp"`.

**O que aconteceu:** um `cmake.configure` terminou com sucesso, e o core fechou
(`textDocument/didClose`) os documentos C/C++ que o language server conhecia.

**Por que o core faz isso.** O clangd recarrega a `compile_commands.json`
sozinho desde a v12 — reconfere a cada ~5 s
(<https://reviews.llvm.org/D92663>) —, mas **o documento já aberto fica com a
compilação em cache**. Reiniciar o servidor resolveria e jogaria o índice fora;
a ação certa é reabrir o documento. E reabrir exige **fechar antes**: o
curto-circuito por hash do core torna um `didOpen` repetido inerte, porque
depois do configure o texto não mudou.

**O que a UI deve fazer:** re-sincronizar o arquivo ativo, exatamente como já
faz em `event.lsp.restarted` e no `recovered()`. É esse `didOpen` seguinte que
leva o **buffer do editor** (não o disco) ao servidor, agora com as flags novas.
Sem reagir ao evento, o arquivo ativo fica sem diagnóstico até o usuário digitar.

**Onde a decisão mora, e por quê.** No core, e não na UI: a UI não consegue
fazê-lo sozinha (o `didOpen` seria inerte e o `didClose` não existia até
2026-09-02). O gatilho é o próprio `event.cmake.finished` do job, observado pelo
loop principal antes de ser repassado — o job roda em thread própria e não
alcança o `Core`, mas o evento dele volta ao dono do estado
(`docs/arquitetura/04-boot-e-comunicacao.md` §3).

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
protocolo `0.23.0` (fatia M1.4 de `docs-privada/diario/18-daily-driver-plan.md`):

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
de `docs-privada/diario/18-daily-driver-plan.md`, trilha T de `docs/roadmaps/21`):

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

Implementado no protocolo `0.25.0` (fatia M2.2 de `docs-privada/diario/18`). Todos exigem
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
- `cmake.status {}` → `{ configured, hasCompileCommands, buildDir,
  cdbDirectory?, cdbStale?, cdbStaleBecause? }` — stat de
  `CMakeCache.txt`/`compile_commands.json`, mais o **diagnóstico da compilation
  database** (protocolo `0.62.0`).
- **`hasCompileCommands` e `cdbDirectory` não são a mesma pergunta**, e confundi-
  las é a origem de "erro de include sem causa":

  ```text
  hasCompileCommands   existe CDB no build dir DA IDE (.kinein/build)?
  cdbDirectory         existe CDB que o clangd ALCANCA, e onde? Relativa ao
                       root; "." e' a propria raiz. AUSENTE quando nao ha.
  ```

  O clangd procura `compile_commands.json` **nos diretórios pai e em
  subdiretórios `build/`** por conta própria
  (<https://clangd.llvm.org/installation>), então um projeto **Meson ou `bear`
  funciona com `hasCompileCommands: false`**. Só `cdbDirectory` responde se o
  usuário tem inteligência de código ou não.
- `cdbStale` + `cdbStaleBecause`: a CDB alcançável é mais velha que um arquivo
  de build que a define (`CMakeLists.txt`, `CMakePresets.json`, `meson.build`,
  `Makefile`). O clangd então usa flags de um projeto que mudou. Os três campos
  são **omitidos quando não há o que reportar** — projeto sadio não carrega
  ruído, e a UI não precisa distinguir `false` de ausente.
- clangd: quando `compile_commands.json` existe no build dir da IDE, novos
  servidores cpp sobem com `--compile-commands-dir` apontando para ele — o
  `.kinein/build/` **não** é `$SRC/build/`, então o clangd não o acharia sozinho.
- **Recarga de flags, corrigido em 2026-08-30.** O clangd **tem** hot-reload da
  `compile_commands.json` desde a v12 (reconfere a cada ~5 s,
  [D92663](https://reviews.llvm.org/D92663)). O que não atualiza é o **documento
  já aberto**, que fica com a compilação em cache
  ([vscode-clangd #42](https://github.com/clangd/vscode-clangd/issues/42)).
  Logo a ação certa após um configure é **reabrir os documentos abertos**, e não
  reiniciar o servidor — reiniciar joga o índice fora.

### Cargo service (`cargo.metadata` / `cargo.check`)

Implementado no protocolo `0.26.0` (fatia M2.3 de `docs-privada/diario/18`). Ambos exigem
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

Implementado no protocolo `0.27.0` (fatia M2.4 de `docs-privada/diario/18`). Todos exigem
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

### Configuration Actions (`configAction.list` / `configAction.preview` / `configAction.apply`)

Implementado no protocolo `0.63.0` (etapa 2 de `docs/roadmaps/30-caminho-para-o-mvp.md`).
Os três exigem workspace aberto. O escopo de cada ação é o **build system
ativo** do workspace (spec 9.2 §1): projeto Cargo não vê ação `CMake`, e pedir
uma fora do escopo é `INVALID_PARAMS`, não silêncio.

**São três métodos porque o ciclo tem três passos**, e a spec 9.1 §10 exige os
três: *explicar, mostrar preview, aplicar com consentimento e validar*. Um
método só seria um botão que edita o `CMakeLists.txt` do usuário sem mostrar o
quê.

- `configAction.list { includeHiddenByScope? }` →
  `{ actions: [...], activeBuildSystems: [...] }`. Cada ação traz
  `{ id, title, description, scope, category, risk, affects, effect, state,
  reason?, params, docs }`. O `state` é medido no workspace agora
  (`available`, `recommended`, `partiallyAvailable`, `unavailable`,
  `hiddenByScope`) e o `reason` diz por quê — "o preset debug já existe",
  "nenhum target declarado". `includeHiddenByScope: true` traz também as ações
  do build system inativo, marcadas (spec 9.2 §25).
- `configAction.preview { id, params? }` →
  `{ id, title, summary, files: [{ path, before?, after }], report, notes }`.
  **Não escreve nada.** `before` ausente significa que o arquivo será CRIADO.
  `report` é o resultado das ações de leitura (o cache do `CMake`); `notes` são
  avisos que o usuário precisa ler antes de aplicar.
- `configAction.apply { id, params?, expected? }` →
  `{ id, message, files, jobId? }`. O `expected` é a lista
  `[{ path, content? }]` que veio do `before` do preview: é a **mesma barreira
  do `fs.write`** (`ARCHITECTURE.md` §7.1) aplicada ao consentimento — se o
  disco mudou entre o preview e o Apply, a resposta é `FILE_CHANGED` e nada é
  escrito. Lista vazia dispensa a comparação.

O `effect` de cada ação diz o que o `apply` faz, e por isso está no contrato:

```text
edit      reescreve um arquivo de texto; o preview mostra o diff exato
inspect   so le: o preview E o resultado, e o apply nao escreve
job       devolve `jobId` reusando um job que ja existe (hoje: cargo.check)
delegate  efeito nao textual dentro do core (remover build dir, salvar run config)
```

As 16 ações do MVP são as da §12 da spec de fechamento: 10 de `CMake`
(`enableCompileCommands`, `createDebugPreset`, `createReleasePreset`,
`addExecutable`, `addStaticLibrary`, `addSourceToTarget`, `addIncludeDirectory`,
`addTargetLinkLibraries`, `inspectCache`, `repairBuildDir`) e 6 de Cargo
(`addDependency`, `addDevDependency`, `addFeature`, `setEdition`, `check`,
`createRunConfig`).

**O que o core RECUSA em vez de adivinhar**, e é o comportamento certo: nome de
target fora do padrão da CMP0037, target inexistente, fonte com `..`,
dependência que já está no manifest, `Cargo.toml` com string multilinha ou
`dependencies` como tabela inline, `edition.workspace` herdada. Corromper o
manifest do usuário não tem desfazer; a recusa vem com o motivo, na lista e no
diálogo.

### Toolchain (`toolchain.get` / `toolchain.set`)

Implementado no protocolo `0.64.0` (etapa 5 de
`docs/roadmaps/30-caminho-para-o-mvp.md`; B2 do TR2 e §5d do `roadmaps/29`).
Ambos exigem workspace aberto. Persistência em `.kinein/toolchain.json` com
`schemaVersion` — arquivo inválido ou de schema desconhecido é tratado como
vazio, e toolchain quebrada nunca impede a IDE de abrir o projeto.

**A pergunta que o domínio responde:** *qual executável cumpre cada papel?*
Até 2026-09-02 todo processo externo do core nascia de `Command::new("<nome>")`
— 28 chamadas, todas resolvidas pelo `PATH` do processo. Trocar de compilador
significava editar `CMakeLists.txt` à mão ou exportar `CC`/`CXX` antes de abrir
a IDE.

Os **papéis** são vocabulário fechado (`cCompiler`, `cxxCompiler`, `generator`,
`cmake`, `cargo`). Papel novo é entrada nova no protocolo e no catálogo do
core, nunca string livre vinda da UI: um papel que o core não sabe usar daria
ao usuário um seletor sem efeito.

- `toolchain.get {}` e `toolchain.set { role, id? }` respondem o **mesmo**
  shape — como as run configs, a UI nunca calcula estado derivado:

```text
{ selections: [{ role, id?, resolvedPath? }],
  candidates: [{ role, id, label, path?, version? }] }
```

- `id` **ausente** em `selections` é AUTOMÁTICO, e é o padrão: nada é fixado e
  o `PATH` continua decidindo — exatamente o comportamento histórico. Quem
  nunca abrir o seletor não vê diferença nenhuma.
- `resolvedPath` no automático é **informação** (o primeiro candidato
  detectado), não fixação. É o que a UI mostra para dizer o que vai acontecer.
- `toolchain.set` com `id` ausente volta para automático. Com um `id` que não
  existe no catálogo, ou que existe mas **não foi detectado nesta máquina**,
  responde `INVALID_PARAMS`: oferecer um compilador ausente é oferecer um
  configure que vai falhar.

**A escolha passou a ser do KIT, não do workspace (protocolo `0.67.0`).** Um kit
é um preset do CMake mais o que ele precisa para compilar: os executáveis por
papel, o `sysroot` e o triple do alvo.

```text
toolchain.get  { preset? }                         -> ToolchainResult
toolchain.set  { role, id?, preset? }              -> ToolchainResult
toolchain.setKit { preset?, sysroot?, targetTriple?, chip?,               NOVO
                   remoteTarget?, debugServer? } -> ToolchainResult
```

`preset` ausente é **o kit padrão do workspace** — que é exatamente o que o
schema 1 do `.kinein/toolchain.json` guardava, e por isso a migração é direta:
as seleções antigas viram o kit padrão **na leitura**, não na escrita. Migrar na
leitura é o que impede a IDE de perder a escolha de quem abriu o projeto e não
mexeu na toolchain.

**Em `toolchain.setKit`, campo ausente NÃO é campo vazio.** Ausente preserva o
valor atual; string vazia (ou só de espaços) limpa. Sem essa distinção, mexer no
`sysroot` apagaria o `targetTriple` e o usuário só descobriria no próximo build.

**O papel `debugAdapter` entrou no protocolo `0.69.0`** (etapa 22 do
`roadmaps/35`). Antes, o adaptador DAP era uma **constante** no core
(`ADAPTER_BINARY = "lldb-dap"`), e por isso não havia debug de embarcado
nenhum: embarcado não debuga com `lldb-dap`.

```text
lldb-dap    desktop; sem argumento          (o PADRAO — nada muda sem escolha)
probe-rs    embarcado; `probe-rs dap-server` fala DAP por stdin/stdout
```

**O catálogo da toolchain NÃO sabe os argumentos, e a fronteira é deliberada.**
O cabeçalho dele diz que responde *"quais binários interessam a cada papel"* —
não *"com quais argumentos"*. Que o `probe-rs` precisa do subcomando
`dap-server` é conhecimento de quem **sobe o processo**, e isso mora no domínio
`dap/`.

**O kit ganhou `chip` no protocolo `0.70.0`** — o alvo do adaptador de
embarcado. Ele vai no **`launch` do DAP**, não na linha de comando: verificado
na documentação do probe-rs, onde `chip` é campo da configuração de launch e não
flag do `dap-server`. Supor o contrário daria um processo que sobe e falha no
primeiro request, com a causa longe do sintoma. **Campo ausente não é campo
nulo** — sem chip escolhido, o `launch` não carrega a chave, mesma regra da
condição de breakpoint (`0.66.0`).

**O kit ganhou `remoteTarget` e `debugServer` no `0.89.0`** — o caminho para
depurar o que não está na máquina. Quando o kit escolhe o adaptador `gdb` (que
fala DAP nativamente desde a v14, fonte `/usr/share/doc/gdb/NEWS`) e declara um
`remoteTarget` (`host:porta`), o `debug.start` faz **`attach`** com esse alvo —
*"passed to the `target remote` command"* (manual do GDB, capítulo Debugger
Adapter Protocol) — em vez de `launch`. O `debugServer` é o comando do servidor
(QEMU, OpenOCD) que a IDE sobe antes de conectar, com `{program}` trocado pelo
ELF; ela o mata com a sessão. **Ambos são declarados, nunca deduzidos** (a IDE
não sabe qual máquina do QEMU é a placa): `roadmaps/35` §5.1. Isso derruba o que
o `integracoes/36` §3 dizia — não há protocolo GDB-remote novo a escrever, é um
segundo candidato do papel `debugAdapter`.

**Cross-compilador NÃO virou papel novo**, e a medição corrigiu o esboço do
`roadmaps/35` §5.3 que dizia que sim: `arm-none-eabi-gcc` escreve a **mesma**
variável que o `gcc` — `CMAKE_C_COMPILER`. Dois papéis apontando para a mesma
variável seria duplicação. O cross é **candidato** dos papéis que já existem.

**Adaptador desconhecido não é recusado:** roda sem argumento, que é a forma da
maioria dos adaptadores DAP. Recusar o que não está na lista impediria o usuário
de apontar um adaptador que a IDE não conhece — e a lista é curta por ser nova,
não por ser completa.

O que o kit vira, no build:

```text
sysroot        -> -DCMAKE_SYSROOT=<caminho>
targetTriple   -> --target <triple>          (cargo: check, clippy e build)
               -> -DCMAKE_SYSTEM_NAME=<...>  quando o triple é inequívoco
               -> -DCMAKE_SYSTEM_PROCESSOR=<arch>
```

**Bare metal ganha `CMAKE_TRY_COMPILE_TARGET_TYPE=STATIC_LIBRARY`, e sem isso o
configure falha SEMPRE.** Medido em 2026-09-03 exercitando
`thumbv7em-none-eabihf` com o `arm-none-eabi-gcc` real: `CMAKE_SYSTEM_NAME=
Generic` sozinho não basta — o CMake ainda tenta **linkar** um executável no
teste de compilador, e bare metal não tem os stubs do newlib
(`undefined reference to _exit`). O projeto nem chega a ser configurado. A doc
do CMake diz que `STATIC_LIBRARY` existe exatamente para *"cross-compiling
toolchains that cannot link without custom flags or linker scripts"*.

Só entra em `Generic`. Em cross para Linux/Windows o link funciona, e forçar o
teste a virar biblioteca **esconderia** um toolchain de verdade quebrado.

`CMAKE_SYSTEM_NAME` é o que faz o CMake entrar em modo cross; o sysroot sozinho
não muda a decisão de compilador. **Triple que o mapa não conhece não vira
palpite:** fica sem `CMAKE_SYSTEM_NAME`, e o caminho oficial para alvos exóticos
continua sendo um `toolchainFile` do preset. O `--target` do cargo vale para
check e clippy também — compilar para o alvo e checar para o host daria
diagnóstico do host, que é pior que não ter porque parece certo.

O resultado carrega `presetToolchainFile` quando o preset declara um
`toolchainFile` (campo do schema de presets desde a versão 3 / CMake 3.21, com
precedência sobre `CMAKE_TOOLCHAIN_FILE`). **É informação, não escolha:** quando
ele existe, o compilador efetivo pode não ser o que o usuário escolheu aqui, e a
IDE diz isso em vez de deixar procurar no lugar errado.
- `candidates` traz só o que existe aqui. `Unix Makefiles` só aparece se o
  `make` existir — por isso ele entrou no `tools.detect` na mesma fatia.

**O que a escolha muda de fato**, e é o que faz dela uma entidade e não um
enfeite:

```text
cmake.configure   -DCMAKE_C_COMPILER / -DCMAKE_CXX_COMPILER / -G, e o
                  EXECUTAVEL do proprio cmake
build.run         o cmake (configure implicito + --build) e o cargo
quality.run       o cargo do clippy
cargo.check       o cargo
cargo.metadata    o cargo
```

Só o que o usuário **fixou** entra na linha de comando. Emitir
`-DCMAKE_CXX_COMPILER` com o que o `PATH` resolveria hoje congelaria no cache do
`CMake` uma escolha que o usuário não fez.

**O que esta fatia NÃO entrega, e está registrado:** sysroot, cross-compilação e
kit por preset. São o resto do B2 do TR2, e ficam para uma fatia própria — o
que existe hoje é a entidade e a rota até o comando.

### Settings (`settings.get` / `settings.set`)

Implementado no protocolo `0.36.0` (fatia M4.1 de `docs-privada/diario/18`). Dois níveis:
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
                 outlineCollapsed?: bool }
                                          (campos ausentes = não setados)
EffectiveSettings { formatOnSave, editorFontSize, autoClosePairs,
                    rigorProfile, explorerWidth, contextWidth,
                    assistantTerminalWidth,
                    bottomPanelHeight, outlineWidth, outlineCollapsed,
                    outlineCollapsed }
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
  terminal KV ativo e Estrutura), painel inferior `260` e Estrutura expandida.
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
  `outlineCollapsed` persiste o recolhimento da Estrutura.
  `diffBase` (head|index) fica para uma micro-fatia futura (precisa de
  `base` no `git.fileDiff`).

### Rascunhos / autosave (`draft.save` / `draft.clear`)

Rede de segurança contra perda de dado (protocolo `0.40.0`, fatia S1 de
`docs/seguranca/23`). O core persiste em **SQLite** (`.kinein/kinein.db`, WAL +
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

Implementado no protocolo `0.28.0` (fatia M2.5a de `docs-privada/diario/18`). O core
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
event.debug.started   { program }
event.debug.output    { category, line }   (stdout|stderr|console)
event.debug.stopped   { reason, file?, line?, threadId }  (file/line podem
                        vir null; o core ja enriquece com o frame do topo)
event.debug.continued {}                   (um por retomada, deduplicado)
event.debug.finished  { exitCode? }        (exatamente um por sessao)
```

### Git (`git.status` e operações diárias)

Implementado no protocolo `0.30.0` (fatia M3.1 de `docs-privada/diario/18`). Orquestra o
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
- pull/push são jobs canceláveis e publicam saída/resultado pelo Job System, e
  o desfecho sai em **`event.git.remoteFinished { operation, success, message }`**
  (documentado em 2026-08-29; o evento existia desde o `0.49.0` e era o único
  dos 32 eventos do core que não estava neste contrato). A UI usa `operation`
  para saber se foi `pull` ou `push` sem guardar o `jobId`;
- stash push inclui untracked, limita-se ao workspace e exclui `.kinein`;
  pop restaura o stash mais recente;
- todas as mutações síncronas devolvem o status inteiro para não duplicar
  estado derivado na UI.

### Jobs (`job.list` / `job.cancel`)

Fundação do Job System (ver `docs/arquitetura/ARCHITECTURE.md` §7). Um **job** é uma
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

## Os 131 métodos roteados — a lista inteira

> **Era "Métodos principais implementados", e listava 66 dos 128** — sem dizer
> que era parcial, o que fazia um domínio inteiro parecer inexistente.
> Refeita em 2026-09-06 pelo comando do cabeçalho, agrupada por domínio, e
> refeita de novo no mesmo dia quando a forma vetorial acrescentou
> `sim.checkSystem` e `sim.runSystem`.

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

core.ping
core.shutdown

datasource.introspect
datasource.list
datasource.remove
datasource.save
datasource.test

debug.continue
debug.evaluate
debug.next
debug.pause
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
fs.delete
fs.findFiles
fs.list
fs.read
fs.rename
fs.replace
fs.search
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

index.status
index.symbols
grafana.save

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

probe.list

project.model

quality.run

run.script
run.start
run.stdin
run.stop

runConfig.delete
runConfig.list
runConfig.save
runConfig.setActive

serial.list
serial.monitor

settings.get
settings.set

setup.list

sim.catalog
sim.checkFormula
sim.checkSystem
sim.estimate
sim.evaluate
sim.forget
sim.inspectFormula
sim.list
sim.run
sim.runSystem
sim.save

syntaxTree.update

terminal.close
terminal.input
terminal.mouse
terminal.open
terminal.resize
terminal.scroll

test.run

toolchain.get
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
```

## Os 41 eventos emitidos — a lista inteira

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

event.datasource.introspected
event.datasource.tested

event.debug.continued
event.debug.finished
event.debug.output
event.debug.started
event.debug.stopped

event.environment.finished
event.environment.started
event.container.finished

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
event.lsp.restarted
event.lsp.status

event.quality.diagnostic
event.quality.finished
event.quality.output                <- so por format!
event.quality.started               <- so por format!

event.run.finished
event.run.output
event.run.started

event.terminal.closed
event.terminal.render

event.test.case
event.test.finished
event.test.output
event.test.started
```

## Regras

1. Todo request deve ter resposta.
2. Eventos não têm `id`.
3. Erros devem ter `code`, `message` e `details`.
4. O protocolo deve ser versionado.
5. Toda alteração no protocolo exige atualização de docs e schemas.
6. A UI não deve interpretar logs brutos quando houver evento estruturado.

## `library.*` — o catálogo curado de bibliotecas C/C++

Domínio novo no protocolo `0.68.0` (etapa 19 do `roadmaps/35`). **A IDE endossa
o que oferece**: cada entrada carrega licença verificada, versão pinada e a
frase que explica o que a biblioteca faz.

```text
library.list {}                  -> { libraries: [LibraryInfo] }
library.plan { id, target }      -> LibraryPlan
```

**Nenhum dos dois exige workspace aberto**, e o domínio é **stateless** — o
catálogo é estático e a disponibilidade se lê do filesystem. Nada depende do
estado do `Core`, e o handler não recebe `self`.

`status` é `detected` | `notDetected`. **`notDetected` não é "não existe":** a
detecção olha os diretórios de config package conhecidos, e o usuário pode ter
um `CMAKE_PREFIX_PATH` próprio. Por isso, quando nada foi achado, o
`LibraryPlan` devolve `searchedPaths` — *"não achei, e olhei aqui"* é acionável;
*"não achei"* manda adivinhar.

O `LibraryInfo` carrega `standardLineage` quando existe o sinal FORTE — a
biblioteca ter sido adotada no padrão ISO, ou passado pela revisão formal do
Boost. **Não há certificação oficial de biblioteca C++:** o WG21 padroniza a
linguagem e a biblioteca padrão, e a Standard C++ Foundation declara que seu
objetivo é reduzir barreiras para *adotar* bibliotecas no próprio Standard — não
certificar as de terceiros. O campo registra o sinal que existe de verdade em
vez de inventar um selo que ninguém emite; hoje só `fmt` o tem
(virou `std::format` no C++20).

**O plano não escreve nada.** Ele devolve passos que nomeiam a Configuration
Action que os executaria, e quem escreve continua sendo o domínio
`configaction`, com o preview e o consentimento que ele já tem. Dois escritores
do mesmo arquivo de build é a forma de eles divergirem em silêncio — e o
critério de aceite do corte é mecânico: `grep -c "CMakeLists"
crates/kinein-core/src/library/` tem que voltar **0**, inclusive em comentário.

**`find_package` ou `FetchContent` sai da MEDIÇÃO, não de um campo do
catálogo.** Se o config package está no sistema, usa-se ele; baixar e compilar o
que já está instalado é desperdício que o usuário paga em tempo de build. A
primeira versão tinha um enum com três variantes para isso e as sete entradas
auditadas saíram todas iguais — os lints reprovaram as variantes nunca
construídas, e estavam certos.

## `probe.*` — as sondas de debug conectadas

Domínio novo no protocolo `0.71.0` (etapa 24 do `roadmaps/35`). É o **"plug"**
do plug and play: a IDE detecta em vez de pedir configuração.

```text
probe.list {} -> { probes, toolAvailable, rawOutput, hint? }
```

Executa `<adaptador> list`, onde o adaptador vem do kit (papel `debugAdapter`).
**Exige workspace**, porque sem kit não há qual ferramenta perguntar — e cair no
`PATH` em silêncio esconderia do usuário quem respondeu.

**O `rawOutput` vai SEMPRE, não só no erro.** O formato do `probe-rs list` foi
levantado de fontes da comunidade e **não** verificado contra o binário
instalado (ele não está nesta máquina, medido em 2026-09-03). Um parser rígido
contra formato não verificado quebraria na primeira mudança de espaçamento — e
quebraria dizendo *"nenhuma sonda"*, que é a pior mentira possível aqui.

Por isso: linha que casa vira sonda, linha que não casa é **ignorada**, e a
saída crua volta junto. *"Não entendi o que a ferramenta respondeu, e aqui está
o que ela disse"* é acionável; *"nenhuma sonda"* com uma plugada viola o item 4
do plug and play (`roadmaps/35` §5.1).

**A `hint` é onde o item 4 vira código.** O caso mais importante é o de **udev**:

```text
permissao negada  -> "falta regra de udev; rodar a IDE como root NAO e a solucao"
resposta vazia    -> "sonda desconectada, ou cabo de dados trocado por um de carga"
nao reconhecida   -> "o formato mudou e o parser precisa acompanhar"
```

Sem regra de udev a ferramenta roda, não acha nada, e o usuário conclui que a
placa está com defeito. **Plug and play morre exatamente aí**, e é a lacuna que
`integracoes/36` §5 já tinha nomeado como a mais subestimada.
## `container.*` — Docker e Podman como domínio nativo

Domínio novo no protocolo `0.92.0` (2026-09-12). A decisão é de 2026-07-17
(`roadmaps/28` §0: *"vão ser cidadãos nativos"*), e as invariantes registradas
lá em §4 são o desenho: **a UI nunca chama `docker`**, **toda ação é Job
cancelável**, **permissão é visível**, e **Podman é motor de primeira** — nesta
máquina `docker` é o shim `podman-docker`, medido em 2026-09-12.

```text
container.status  {}                       -> ContainerStatus
container.list    { all? = true }          -> { containers, engine, rawOutput, hint? }
container.images  {}                       -> { images, engine, rawOutput, hint? }
container.action  { id, action }           -> { jobId }            (job; start|stop|restart|remove)
container.open    { id, mode }             -> { id, command }      (aba de terminal; logs|shell)
container.compose { action, file? }        -> { jobId }            (job; up|down; exige workspace)

ContainerStatus   engine? (docker|podman), binary?, version?, emulated, rootless?,
                  socket?, reachable, compose?, hint?, rawOutput
ContainerInfo     id, names[], image, state, status, ports[], created
ImageInfo         id, repository, tag, size, created

event.container.finished { jobId, action, target, ok, message }
```

**A detecção pergunta ao binário, não ao nome.** `docker --version` do shim
escreve *"Emulate Docker CLI using podman"* em stderr; tratar isso como Docker
Engine erraria o formato de `ps` (o Podman devolve um **array** em `--format
json`; o Docker, **um objeto por linha** em `--format '{{json .}}'`) e o
diagnóstico (não há daemon nem grupo `docker` num Podman rootless). Quando o
`docker` é o shim, o core prefere o `podman` real e diz `emulated: true`.

**O parser aceita as duas formas e devolve a saída crua sempre** — a mesma
regra do `probe.list`. A forma do Docker foi lida da documentação e **não**
verificada contra um Docker Engine real (esta máquina não tem um); a do Podman
5.8.4 foi medida, inclusive o detalhe de que `images --format json` não traz
`repository`/`tag` e sim `Names ["repo:tag"]`.

**`status` é a tela de "ativar a ferramenta".** Sem motor: o passo de
instalação da distro. Motor que não responde: `permission denied` no socket →
grupo `docker` e relogar; daemon parado → `systemctl enable --now docker`;
Podman → a saída crua e o `podman.socket` do usuário. A IDE **imprime** o
comando; nunca roda `sudo`.

**`action` e `compose` são jobs** (`JobRisk::Medium`; `remove` é `High`): cada
linha do motor vai para `event.job.output`, cancelar mata o filho, e o
`event.container.finished` leva as últimas linhas como motivo. `rm` vai **sem
`-f`**: remover o que roda é dois gestos (parar, remover), de propósito.
`compose up` é `-d` — um job que nunca termina não é job; a saída viva mora
na aba de logs.

**`open` reaproveita o terminal.** `logs -f --tail 200 <id>` e `exec -it <id>
/bin/sh` sobem pelo mesmo `open_command` do domínio `terminal` e voltam como
uma sessão de terminal (`terminal.input`/`resize`/`close` a reconhecem). O
shell dentro do container é a semente do *contexto remoto* do `roadmaps/28`
§4 — o que falta para *dev containers* é o path mapping e o ciclo de vida,
não o transporte.

## `index.*` — o índice do projeto inteiro

Domínio novo no protocolo `0.94.0` (pilar 0 do `roadmaps/42`, 2026-09-12).
**Decisão do autor:** *"a IDE deve ler o projeto inteiro que for aberto… todas
as funções/arquivos/pastas"* — para Python, C, C++ e Rust.

```text
index.status  {}                        -> IndexStats   (responde tambem sem workspace: idle)
index.symbols { query, limit?, kind? }  -> { symbols[], total, state }   (exige workspace)

event.index.progress { files, symbols }     a cada ~200 arquivos
event.index.finished IndexStats             ao fim do build, e a cada incremento

IndexStats   state (idle|building|ready|failed), folders, files, sourceFiles,
             lines, bytes, symbols, functions, types, byLanguage[] { language,
             files, lines, symbols }, skipped[], elapsedMs, error?
IndexSymbol  name, kind, path (relativo a raiz), language, line, endLine,
             container?
```

**O que ele lê, e como.** Ao abrir o workspace, um job caminha a árvore
inteira com a **mesma lista de pastas ignoradas do watcher** (`.git`,
`.kinein`, `target`, `build`, `node_modules`…, para os dois verem o mesmo
projeto), conta **todo** arquivo, lê os de fonte (C/C++/Rust/Python) e extrai
as declarações de C/C++/Rust com as **gramáticas Tree-sitter do editor** (a
mesma query `tags` oficial de cada gramática, sem cache — `lang/extract.rs`).
Python é contado e medido; as declarações entram quando a gramática entrar
(bloco B do `roadmaps/41`). Arquivo acima de 4 MiB ou ilegível é contado e
**dito** em `skipped`, nunca sumido. Medido em 2026-09-12 neste repositório:
1.010 arquivos, 146 pastas, 70.030 linhas, 4.658 declarações em ~2 s (build
de depuração).

**O incremento.** Os caminhos de `event.fs.changed` são reindexados no loop
principal (um arquivo é milissegundos) e os totais reemitidos. **Limite
honesto:** o watcher só observa as pastas que a UI expandiu/abriu
(ADR-0001, não recursivo), então mudanças fora delas só entram no próximo
`workspace.open` — o watch recursivo é a próxima fatia do pilar 0.

**Sem duplicata e sem renomear a gramática.** A `tags` do Rust captura um `fn`
dentro de `impl` duas vezes (`function` e `method`); o índice fica com a mais
específica por (linha, nome). O que a gramática chama de `class` (a `struct`
do Rust) o índice **não** renomeia.

**A busca** ordena exato > prefixo > substring, sem diferenciar caixa, com
filtro por `kind` e `total` antes do limite. Na UI, `#nome` no Search
Everywhere pede ao índice **e** ao LSP: o índice responde primeiro e **sem
arquivo aberto**; o LSP, quando responde, substitui.

**O que este domínio NÃO é:** semântica (tipos, referências, rename continuam
no clangd/rust-analyzer) nem o contexto de compilador por arquivo (flags da
CDB, crate do cargo, interpretador do Python) — esse é o passo seguinte do
pilar 0.

## `project.*` — o modelo do projeto embarcado

Domínio novo no protocolo `0.93.0` (pilar 0 do `roadmaps/42`, 2026-09-12).
`workspace.open` diz *que build system* a raiz tem; este domínio diz *o que o
projeto é*.

```text
project.model {} -> ProjectModel        (exige workspace)
event.project.changed  ProjectModel     (ao abrir o workspace; ao fim de
                                         event.cmake.finished e event.build.finished)

ProjectModel   root, embedded, frameworks[], sdks[], artifacts, target, hints[]
FrameworkInfo  framework (espIdf|zephyr|picoSdk|platformIo|stm32Cube|cargoEmbedded|
               microPython|yocto|buildroot), evidence (caminho relativo do
               marcador), detail? (IDF_TARGET, PICO_BOARD, DeviceId do Cube,
               triple do cargo, MACHINE do Yocto, ambientes do PlatformIO)
SdkRequirement id, label, env?, path?, found, hint?
ProjectArtifacts elf[], bin[], hex[], uf2[], map[], flasherArgs?, partitionTable?,
               memoryX?, linkerScripts[], flashRecipe?, partitions?
FlashRecipe    chip?, flashMode?, flashSize?, flashSizeBytes?, flashFreq?, before?,
               after?, stub, files[] { offset, file (absoluto), name?, encrypted }
               — LIDO do flasher_args.json (forma do template
               components/esptool_py/flasher_args.json.in do ESP-IDF)
PartitionTable tableOffset, entries[] { name, kind, subtype, offset, size, flags? },
               end, unreadable[] — LIDA do partitions.csv com os offsets em
               branco resolvidos como o gen_esp32part.py (4 KB; app a 64 KB)
TargetModel    chip?, family?, triple?, flashEngine?, monitor?, debugAdapter?,
               evidence[]  — uma linha por dedução
```

**A detecção desce até 3 níveis e LÊ o marcador**: um `CMakeLists.txt` é
ESP-IDF se inclui o `project.cmake` do `IDF_PATH`, Zephyr se faz
`find_package(Zephyr)`, pico-sdk se chama `pico_sdk_init()`; um `main.py` é
MicroPython se importa `machine`/`board`. Pastas de saída (`build/`, `target/`,
`.pio/`, `.kinein/`) não contam. Entre dois achados do mesmo framework vence o
marcador que **decide** (`.cargo/config.toml` com o triple, não um `memory.x`
solto) e, empatando, o mais raso. Um `CMakeLists.txt` comum **não** é
embarcado; um `main.py` que não importa hardware **não** é MicroPython.

**O alvo tem evidência ou não tem alvo.** O chip do **kit** vence o do
framework (é a palavra do usuário); a família vem do chip ou do triple; os
motores vêm da família — e o ESP32 clássico (sem USB-JTAG) recebe `debugAdapter`
**ausente** com a evidência dizendo que depurar exige ESP-Prog, em vez de um
`probe-rs` que não funcionaria.

**"Achado" só vem de variável, pasta padrão ou binário.** `IDF_PATH` definido
mas apontando para pasta inexistente é `found: false`; a toolchain xtensa fora
do `PATH` mas em `~/.espressif/tools` é `found: true` com o caminho. O alvo
rustup não é medido aqui (é processo) e diz isso no `hint`. Nenhum comando roda.

**O que era só localizado passou a ser LIDO** (segunda fatia, 2026-09-12): a
receita de gravação com os caminhos resolvidos contra o `build/`, e a tabela
de partições — porque a "flash" de um ESP32 não é o `.ld`, é a partição
`app`, e é ela que o tamanho e o "Gravar" vão consumir. Linha de CSV que não
se lê vai em `unreadable`, nunca some.

**Fixtures reais e mínimas** de cada framework moram em
`scripts/fixtures/projetos/`; são elas que os testes leem.

## `serial.*` — as portas seriais USB

Domínio novo no protocolo `0.91.0` (E1 do `integracoes/38` §6, 2026-09-11). A
porta serial é o **canal que toda família bare metal compartilha** — bootloader
de ROM, console e as linhas DTR/RTS de reset — e por isso é a fundação do
monitor UART e do "Gravar".

```text
serial.list    {}                   -> { ports: [SerialPortInfo], hint? }
serial.monitor { device, baud? }    -> { id, command, tool }   (aba de terminal; 0.92.0)

SerialPortInfo  device, byId?, kind (usbUartBridge|usbCdc), vid, pid,
                manufacturer?, product?, serial?, interface?, driver?, family?,
                access { readableWritable, mode, group?, hint? },
                modemManager? { candidate, ignored, running }
```

**Não exige workspace**, ao contrário do `probe.list`: não há ferramenta vinda
do kit — é o sysfs desta máquina, o mesmo com ou sem projeto aberto.

**Nunca abre a porta.** Abrir um tty aciona DTR/RTS na maioria das pontes
(CP210x, CH340, FTDI) e isso **reseta a placa**; um `serial.list` que
resetasse o firmware a cada abertura do painel seria um defeito. A permissão é
medida com `access(2)`, que honra ACL — é assim que `TAG+="uaccess"` dá acesso
ao usuário da sessão sem grupo nenhum; um `stat` sozinho mentiria nesse caso.

**De onde vem cada campo:** VID:PID, `manufacturer`/`product`/`serial` e
`bInterfaceNumber` subindo de `/sys/class/tty/<n>/device` até o diretório USB
com `idVendor` (ABI documentada do kernel); `driver` do link `device/driver`;
`byId` de `/dev/serial/by-id`; `modemManager` de `udevadm info -q property`
(`ID_MM_CANDIDATE`, `ID_MM_DEVICE_IGNORE`) mais `/proc/*/comm` — e é `null`,
não `false`, quando o `udevadm` não existe. Medido em 2026-09-11: o
ModemManager examinou o ESP32 4 s depois do plug.

**`family` fala do ELO, nunca do chip.** `10c4:ea60` é *"ponte USB-UART
CP210x — o chip do outro lado não se lê pelo USB"*; `303a:1001` é *"Espressif
USB Serial/JTAG — o próprio chip"*. A identidade do chip vem **pelo canal**
(`esptool chip-id`, `probe-rs info`), que é a fatia E5. Fontes da tabela no
`integracoes/38` §5.

**`serial.monitor` é o monitor como PROCESSO numa aba de terminal** (E3 do
`integracoes/38` §6, decisão do autor em 2026-09-11: nunca código serial
nosso). A ferramenta vem do papel novo do kit, `serialMonitor` — candidatos
`tio`, `picocom`, `minicom`, `espflash` nessa ordem —, com uma regra a mais
que o catálogo não conhece: chip Espressif no kit **e** `espflash` detectado
→ `espflash monitor --elf <ELF>`, que decodifica o backtrace; escolha
**fixada** pelo autor vence tudo. Linhas de comando lidas na fonte: `tio -b`,
`picocom -b`, `minicom -D … -b`, e no espflash é `--monitor-baud` (o `--baud`
dele é o de **gravação**). Baud ausente = 115200. Sem nenhum monitor
instalado, `TOOL_NOT_FOUND` com o que instalar. Exige workspace (a aba nasce
no cwd do projeto) e volta como sessão de terminal, com o comando no título.

**A `hint` de acesso é o passo oficial, nunca um `sudo` que a IDE rodaria:**
*"`/dev/ttyUSB0` é `crw-rw----` do grupo `dialout` e você não está nele …
`sudo usermod -aG dialout $USER` e sair/entrar da sessão. A IDE não roda isso."*

## `command.*` — o catálogo de comandos que a UI mostra

Um método, e ele é a fonte única de **tudo que a IDE oferece por nome**: paleta,
menus, botões e atalhos leem daqui.

```text
command.list {}   ->  { commands: [CommandDescriptor] }
```

```json
{
  "id": "editor.save",
  "title": "Salvar arquivo",
  "category": "Editor",
  "description": "Grava o buffer atual no disco",
  "defaultShortcut": "Ctrl+S",
  "requiresWorkspace": true
}
```

**Não exige workspace aberto** — a paleta existe antes de haver projeto, e é o
`requiresWorkspace` de cada descritor que diz o que fica desabilitado.

**São 76 descritores, medidos em 2026-09-06**, em cinco grupos que são cinco
arquivos em `crates/kinein-core/src/commands/`:

```text
ide.rs      24    editor.rs   19    build.rs    16    git.rs       9    run.rs    8
```

**Por que este domínio existe em vez de a UI ter a lista:** porque o atalho que a
paleta **anuncia** tem de ser o que a IDE **obedece**, e isso é gate desde
2026-09-03 (`scripts/verificar-atalhos.sh`). Com a lista no core, o gate compara
uma fonte com o host; com a lista na UI, ele compararia a UI consigo mesma.

## `setup.*` — o passo a passo oficial de instalação, por distro

```text
setup.list {}  ->  { distroId, distroName, family, tools: [SetupToolInfo] }
```

```text
SetupToolInfo   id · name · summary · website · installed · guide?
SetupGuide      family · sourceUrl · checkedAt · steps: [SetupStep]
SetupStep       explanation · command
```

**Não exige workspace**: instalar o PostgreSQL não depende de projeto aberto, e
quem está começando abre a IDE antes de ter projeto — que é exatamente quando
este guia serve.

**A regra que governa o domínio inteiro: sem fonte oficial, a IDE não afirma.**
Cada `SetupGuide` carrega `sourceUrl` e `checkedAt`, e uma família de distro sem
fonte oficial **não recebe guia** — o `guide` volta `None` e a tela mostra o site
do projeto dizendo que não tem passo a passo. É a decisão registrada em
`../roadmaps/40` §5, e é por isso que Arch e openSUSE continuam sem passos: a
fonte dos três projetos não cobre essas famílias.

O `installed` vem do `ToolDetector` — detectar é capacidade, e a política de o
que fazer com a detecção fica na UI (`27-modulos-por-dominio.md` §6).

## `datasource.*` — os perfis de banco, e a senha que não mora em disco

Domínio da etapa 26/27 (`../roadmaps/35` §9). Cinco métodos, dois eventos.

```text
datasource.list       {}                      -> { profiles: [DataSourceProfile] }
datasource.save       { profile }             -> DataSourceWriteResult
datasource.remove     { name }                -> DataSourceWriteResult
datasource.test       { name, password? }     -> DataSourceTestAccepted   (job)
datasource.introspect { name, password? }     -> aceite + job
```

**Os quatro últimos exigem workspace aberto**; o perfil mora no projeto.

```text
event.datasource.tested        { jobId, ok, message, ... }
event.datasource.introspected  { jobId, schemas | collections, ... }
```

**A senha nunca entra no perfil.** O `DataSourceProfile` guarda motor, host,
porta, banco, usuário e um `SecretSource` — *de onde* o segredo vem —, e o
`password` viaja só no parâmetro do método que precisa dele, por chamada. É a
decisão de `../seguranca/40`, e a UI abre o diálogo de senha por
`secretRequired` no erro, **nunca casando texto de mensagem**.

**Duas formas de resultado, porque há dois tipos de banco.** O relacional
devolve `schemas → tables → columns` lido do `information_schema`; o MongoDB
devolve `collections → fields` com profundidade, tipo **plural** e presença em
%, e a origem do esquema declarada:

```text
DECLARADO   veio do validador `$jsonSchema` da colecao
INFERIDO    veio de `$sample` sobre a colecao
```

**A tela nunca deixa os dois parecidos**, e o custo da leitura vai junto:
quantos documentos foram lidos, e se a amostragem obrigou o servidor a varrer a
coleção inteira (`$sample` varre tudo quando N não é menor que 5% dela). Tetos
de RAM — 2.000 campos, 8 níveis, 10 elementos de array — aparecem como aviso na
coleção em vez de a deixarem com cara de completa.

## `grafana.*` — a observabilidade pela HTTP API, e só

Quatro métodos, um evento.

```text
grafana.get    {}          -> { profile: GrafanaProfile | null }
grafana.save   { profile }
grafana.forget {}
grafana.probe  { token? }  -> GrafanaProbeAccepted   (job)
```

```text
event.grafana.probed  { jobId, datasources, dashboards, matches, ... }
```

**Os quatro exigem workspace aberto.**

**O Grafana nunca é embutido** — licença AGPL, decisão registrada em
`../roadmaps/40` §5. A integração é HTTP, o cliente é o `ureq`, e os dashboards
**abrem no navegador do sistema**. O que justifica o domínio existir é o
`GrafanaMatch`: o cruzamento entre o datasource do Grafana e o perfil de banco
do projeto, que é a pergunta que nenhuma das duas ferramentas responde sozinha.

O token segue a mesma regra da senha: `GrafanaTokenSource` diz de onde ele vem,
e o valor viaja por chamada.

## `sim.*` — a simulação por conceito

Domínio da etapa 28. **Onze métodos**, nenhum evento — as corridas de hoje
terminam dentro da resposta. O desenho está em
[`34-simulacao-por-conceito.md`](34-simulacao-por-conceito.md), e os tipos em
`crates/kinein-protocol/src/sim.rs`.

```text
sim.catalog        { course? }                     -> { concepts: [SimConcept] }
sim.inspectFormula { formula }                     -> { variables: [String] }
sim.checkFormula   { concept, formula, bindings }  -> SimCheckResult
sim.evaluate       { concept, formula, bindings, values }        -> SimEvaluateResult
sim.estimate       { duration, step, samples }     -> SimEstimateResult
sim.run            { concept, formula, bindings, values, initial,
                     duration, step, method, samples }           -> SimRunResult
sim.list           {}                              -> { simulations: [SimSaved] }
sim.save           { simulation }
sim.forget         { name }

sim.checkSystem    { concept, equations }          -> SimCheckSystemResult
sim.runSystem      { concept, equations, values, initial,
                     duration, step, method, samples }  -> SimRunSystemResult
```

**Os dois últimos são a forma VETORIAL** (`dY/dt = F(t, Y)`), entrada em
2026-09-06 e desenhada em [`34`](34-simulacao-por-conceito.md) §13. O
`equations` traz **uma fórmula por componente**, cada uma com a ligação dela, e
a ORDEM da lista não importa: o core casa pelo campo `component`, porque supor
que a n-ésima fórmula é do n-ésimo componente seria adivinhar.

**O método `eulerSymplectic` é recusado quando o conceito não declara o
pareamento posição/velocidade**, com `reason: "noPairing"`. Não é limitação a
contornar: sem o par, o método não está definido. E ele importa — medido numa
órbita circular de raio verdadeiro 1 com `dt=0,01` por dez voltas, o Euler
explícito termina com raio `1,647957` e o simplético com `1,000024`.

**O `SimRunSystemResult` traz DOIS sinais de exatidão**, e o segundo não existia
na forma escalar:

```text
accuracy     o erro contra a solucao fechada, quando ela existe. Na orbita ela
             vale so' no caso CIRCULAR — a eliptica exige a equacao de Kepler,
             que e' transcendental, e o oraculo recusa em vez de aproximar
invariants   a DERIVA de cada grandeza que a fisica conserva. Existe mesmo sem
             solucao fechada, e e' o unico sinal que um sistema caotico admite.
             A tela chama de DERIVA e nunca de erro: invariante conservado nao
             significa resultado certo
```

**Os seis primeiros não exigem workspace** — montar e conferir uma fórmula não
depende de projeto aberto. Os três últimos exigem, porque a persistência mora em
`.kinein/simulacoes/`.

**A decisão que governa cada tipo deste domínio: nada é adivinhado.** Todo campo
que decide um resultado é **obrigatório** — não há método padrão, passo padrão
nem amostragem padrão. A IDE calcula e MOSTRA; quem escolhe é o usuário.

**A ligação é dado do usuário, nunca casamento por nome.** O `SimBinding` diz
qual grandeza cada variável da fórmula é. Isso não é rigor: o avaliador devolve
as variáveis em ordem **alfabética**, e montar o vetor de avaliação pela ordem de
leitura da fórmula produz um número com a física errada e **sem erro nenhum**
(ADR-0006, armadilha 1).

**O `sim.estimate` existe porque contar passos é regra de negócio.** A IDE mostra
o custo antes de rodar — quantos passos, quanto a trilha ocuparia inteira e
amostrada, e se compilar valeria a pena nesta escala. A UI pergunta e desenha;
ela não faz a conta (`ARCHITECTURE.md` §2).

**O `SimCheckResult` devolve TODOS os problemas, não o primeiro**, cada um como
uma variante tipada — `parseFailed`, `missingQuantity`, `unboundVariable`,
`unknownQuantity`, `duplicateQuantity`, `variableNotInFormula` — para a UI nunca
casar por texto de mensagem. E o erro de parse carrega a mensagem **da IDE**: o
texto do avaliador tem endereço de ponteiro dentro e nunca é repassado.

**O `SimRunResult.accuracy` só existe quando o conceito tem solução fechada**, e
a tela **diz** quando não tem, em vez de omitir a coluna e deixar parecer que o
número é exato.

> **Defeito conhecido, medido em 2026-09-06 e registrado em `../roadmaps/31`
> §19.0:** o `accuracy` vem do CONCEITO e não olha a fórmula digitada. Quando as
> duas divergem — o que o `sim.checkFormula` permite, porque ele confere ligação
> e não física — o `absoluteError` é calculado contra a solução de outra equação.
> Reproduz com `python3 scripts/exercitar-sim-oraculo.py`. O conserto depende de
> decisão de desenho e está na fila do `../roadmaps/40` §4.

## `core.*` — o handshake e o encerramento

Dois métodos, e eles são os únicos que **nunca** dependem de nada: não exigem
workspace, não tocam o filesystem e respondem mesmo com o resto do core inerte.

```text
core.ping     {}  ->  { protocolVersion, ... }
core.shutdown {}  ->  { message: "shutdown requested" }
```

**O `core.ping` é como a UI descobre com que protocolo está falando** — é o par
do `PROTOCOL_VERSION` em `crates/kinein-protocol/src/lib.rs`, e é o primeiro
pedido que a UI faz depois de subir o processo (`04-boot-e-comunicacao.md`).

**O `core.shutdown` pede, não mata.** Ele responde e deixa o encerramento
acontecer com o drain dos jobs em andamento, que é o que impede um build a meio
caminho de virar processo órfão.

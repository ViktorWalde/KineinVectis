# `syntaxTree.*` — o domínio no protocolo IPC

> Parte do [03](../03-protocolo-ipc.md), que tem o formato base, as listas
> canônicas de métodos e eventos e as regras. Movido sem mudar o texto em
> 2026-10-16 ([61](../../roadmaps/61-organizacao-da-documentacao.md) §3.7, O6).

### Indentação pela gramática (`syntaxTree.indent`, `0.135.0`)

Recebe `{ path, version, line, column, trigger }` — `line` 1-based, `column`
0-based em unidades UTF-16, `trigger` em `newline` ou `closeDelimiter` — e
responde `{ path, version, language, level }` ou **`null`**.

Três coisas que o formato diz de propósito:

- **`version` é a da ÁRVORE**, não a que foi pedida. O core responde a partir da
  árvore que tem, que pode ser anterior ao que já foi digitado; a UI descarta a
  resposta quando as duas não batem. É por isso que o `ParsedDocument` guarda
  versão;
- **`level` é um número de níveis**, e o texto de um nível é decisão do editor
  (espaço ou tabulação). Devolver texto obrigaria os dois lados a concordar
  sobre a configuração. **Qual linha** recebe esse nível vem do `trigger` que
  foi enviado, e não de um segundo campo: `newline` corrige a linha que está
  sendo aberta, `closeDelimiter` corrige aquela em que se acabou de digitar. Um
  `dedentTo` existiu aqui por algumas horas e saiu assim que foi medido — ele
  repetia o `level` e ninguém o lia;
- **`null` não é erro.** Significa "a gramática não sabe aqui" — árvore com
  `ERROR` em volta do cursor, ou arquivo sem gramática. O editor já aplicou o
  fallback local antes de perguntar, e ele continua valendo. Transformar isso em
  falha encheria o log de algo que está funcionando como desenhado.

Nada disso está no caminho síncrono da tecla: o fallback local decide na hora, e
esta resposta só corrige quando diverge.

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
protocolo `0.23.0` (fatia M1.4):

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

`lsp.switchSourceHeader` foi adicionado no protocolo `0.34.0` (fatia T1,
trilha T de `DocsPublic/roadmaps/21-roadmap-de-longo-prazo.md`):

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
  principal e os companheiros de UMA linguagem (`"rust"`|`"cpp"`|`"python"`);
  emite um `event.lsp.restarted` por linguagem. Sem `language`, reinicia
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

Cancelamento de requests LSP ainda não faz parte deste contrato.

```text
event.lsp.status       { "language": "cpp|rust|python|python-ruff", "status": "running|failed|stopped|exited|restarting", "message"? }
event.lsp.diagnostics  { "path": "/abs/file", "diagnostics": [{ "source": "lsp", "category": "lsp", "severity": "error|warning|note", "message", "line", "column" }] }
event.lsp.log          { "language": "cpp|rust|python|python-ruff", "line": "..." }   (0.111.0)
```

`line` e `column` dos diagnósticos são 1-based para consumo direto da UI. A
UI integra esses eventos à aba Problemas com origem `lsp`.

**`exited` e `failed` são só do servidor** (2026-10-01, sem mudança de
versão: o formato é o mesmo, muda o que o core deixa de emitir). Quando o
próprio core encerra o servidor — workspace fechado (`stopped`), `lsp.restart`
(`restarting`), companheiro desligado (`stopped`) —, nenhum `exited` ou
`failed` segue, nem um `running` atrasado do handshake. Antes, a thread leitora
via o fim do stdout depois do `stopped` e anunciava `exited`, e a barra de
status mostrava "LSP ✗" depois de fechar o projeto.

**`event.lsp.log` (`0.111.0`)** é o stderr do servidor, uma linha por evento,
sem interpretação: o `clangd` anunciando a versão e onde procurou o
`compile_commands.json`, o rust-analyzer contando o índice, o traceback de um
basedpyright que morreu no `import`. Linhas são UTF-8 com perda e cortadas em
4 KiB com marcador; o core guarda as últimas 64 (a cauda). A UI as escreve na
aba IDE. A cauda também vai no **`message`** do `event.lsp.status` quando
`status` é `failed` (dentro do texto do erro, sob `--- stderr do servidor ---`)
ou `exited` (o `message` inteiro; ausente quando o servidor não disse nada).
Um servidor que falha o `initialize` é morto pelo core — antes ficava vivo e
mudo. Sem agrupamento: medido em 2026-09-17 com os servidores reais nesta
máquina, o pico foi 18 linhas/s (clangd indexando), longe de justificar lote.

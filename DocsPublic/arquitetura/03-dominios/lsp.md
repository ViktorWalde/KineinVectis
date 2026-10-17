# `lsp.*`, `event.lsp.documentsClosed` e `event.lsp.progress` — o domínio no protocolo IPC

> Parte do [03](../03-protocolo-ipc.md), que tem o formato base, as listas
> canônicas de métodos e eventos e as regras. Movido sem mudar o texto em
> 2026-10-16 ([61](../../roadmaps/61-organizacao-da-documentacao.md) §3.7, O6).

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
- Python (`0.99.0`, 2026-09-13): `basedpyright-langserver --stdio` — o binário
  **detectado** (`~/.local/bin` do pipx/npm entra pelo detector; o `PATH` do
  processo da IDE pode não o ter), `languageId: "python"` para `.py/.pyi/.pyw`.

Se o servidor não estiver instalado ou falhar no `initialize`, o core emite
`event.lsp.status` com `status: "failed"` e mensagem humana. O MVP usa
sincronização de documento inteiro.

**O progresso do servidor (`0.176.0`, a 0.4.0d do `roadmaps/52` §14.4).** O
cliente anuncia `window.workDoneProgress` e responde `null` ao
`window/workDoneProgress/create`; cada `$/progress` vira `event.lsp.progress
{ language, title, message?, percentage?, done }`. O começo e o fim de cada
frente sempre passam; os relatos de uma mesma frente, no máximo 4 por
segundo (o rust-analyzer manda uns 7, medido em 2026-10-16). Um servidor já
parado não manda mais nada. O cliente NÃO anuncia `didChangeWatchedFiles`:
medido na mesma data, o rust-analyzer vigia os arquivos sozinho quando o
cliente não se oferece, e o clangd não pede vigilância.

**Configuração do servidor (`0.99.0`).** Um `ServerSpec` pode carregar
`settings` (JSON). Quando os há, o core envia `workspace/didChangeConfiguration
{ settings }` **logo após o `initialized`** e responde ao
`workspace/configuration` que o servidor pergunta (LSP 3.17): um valor por
`ConfigurationItem`, navegando `section` com pontos (`python.analysis` →
`settings.python.analysis`); seção inexistente → `null`; item sem `section`
(ou vazia) → a configuração inteira. Servidor sem `settings` (clangd,
rust-analyzer) não recebe configuração alguma — o wire é o mesmo de antes.

Para Python os `settings` são montados pelo core a partir do **interpretador
que o projeto resolve** (`python/env.rs`, a precedência do `roadmaps/29`
§4.1 — o mesmo que `python.status` e `index.context` mostram):
`{ python: { pythonPath, analysis: { autoSearchPaths, diagnosticMode:
"openFilesOnly" } }, basedpyright: { analysis: {…} } }`. Sem interpretador
nenhum (workspace sem `.venv`, sem `python3`) não se empurra configuração — a
IDE não inventa `pythonPath`. `diagnosticMode: "openFilesOnly"` é decisão: o
modo `workspace` faria o basedpyright analisar o projeto inteiro a cada
mudança. O `event.python.finished` com `success: true` (o `.venv` nasceu)
reconfigura e, se o servidor de Python estiver vivo, **reinicia-o** — sai
`event.lsp.restarted { language: "python" }` e a UI reabre os documentos; um
`success: false` não reinicia nada. Sem isto o basedpyright indexaria a stdlib
do Python do `PATH` e o completar mentiria sobre os pacotes do projeto.

**Dois servidores Python (validado em 2026-09-15, sem mudar o IPC).** O
`lsp/registry.rs` mantém o principal `python` (basedpyright) e o companheiro
`python-ruff` (`ruff server`), registrado quando o detector encontra Ruff.
`didOpen`/`didChange`/`didSave`/`didClose` alcançam os dois, com versões
independentes. Navegação, hover e completion continuam no principal; as code
actions consultam ambos. Cada `event.lsp.diagnostics` leva a união das listas
por arquivo, e limpar/remover um servidor preserva a parte do outro. Ruff
ausente não impede o principal; falha de subida do companheiro gera status e
o retira do registro para não repetir a falha a cada sincronização.

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
`0.22.0` (fatia M1.3):

- `lsp.codeActions { path, content, line, column }` →
  `{ actions: [{ title, kind? }] }`. O core sincroniza o buffer, envia
  `textDocument/codeAction` com range-ponto no cursor e compõe o `context`
  com os diagnostics que o próprio servidor publicou para a linha (cache da
  thread leitora — a UI não devolve diagnóstico). Só entram na lista ações
  `CodeAction` literais com `edit` inline e sem `disabled`; ações que
  dependem de `workspace/executeCommand` são filtradas (decisão da fatia
  M1.3: a IDE não executa comando arbitrário do servidor). As ações cruas ficam guardadas como **consulta ativa**,
  junto da chave do servidor que as produziu. A lista reúne principal e
  companheiros; o contexto enviado a cada servidor contém seus próprios
  diagnósticos. O preview valida versões no servidor de origem da ação.
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

Nasceu em 2026-09-02 (etapa 4 do `DocsPublic/roadmaps/30-caminho-para-o-mvp.md`).
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
(`DocsPublic/arquitetura/04-boot-e-comunicacao.md` §3).

**Quando o clangd subiu sem CDB, ele é reiniciado** (0.4.0e, `0.177.0`). O
recarregar sozinho só vale para a CDB que o clangd JÁ achou. Medido em
2026-10-16 (clangd 21.1.8, sonda LSP direta): sem nenhuma ao subir, a que
nasce depois não vale com o documento reaberto — na raiz, só uns 30 s depois
(o cache do "não achei"); em `.kinein/build`, que ele só vê pelo
`--compile-commands-dir` dado ao subir, nunca. O caso é comum: o `.cpp`
aberto antes do primeiro configure (o projeto ainda não confiável) ou antes
do `compiledb` do PlatformIO. O core lembra se o clangd subiu vendo uma CDB
(a do configure, ou uma na raiz ou no `build/` dela); se não, o fim do
configure — e o do job `platformio.compiledb` — o reinicia
(`event.lsp.restarted`, que a UI já trata re-sincronizando) em vez de fechar
os documentos.

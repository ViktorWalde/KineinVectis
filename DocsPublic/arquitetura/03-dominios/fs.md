# `fs.*` (arquivos: ler, gravar, mover, lixeira) — o domínio no protocolo IPC

> Parte do [03](../03-protocolo-ipc.md), que tem o formato base, as listas
> canônicas de métodos e eventos e as regras. Movido sem mudar o texto em
> 2026-10-16 ([61](../../roadmaps/61-organizacao-da-documentacao.md) §3.7, O6).

### Arquivos (`fs.list` / `fs.read` / `fs.readExternal` / `fs.createFile` / `fs.createDirectory` / `fs.write` / `fs.rename` / `fs.copy` / `fs.transferBatch` / `fs.trash` / `fs.delete` / `fs.replace`)

Implementado no protocolo `0.4.0`. Todos exigem workspace aberto
(`INVALID_REQUEST` caso contrário) e todo caminho é canonicalizado e
confinado à raiz do workspace (`INVALID_PARAMS` se escapar).

- `fs.list { path }` → `{ path, entries: [{ name, kind: file|directory|other, size? }] }`,
  ordenado diretórios primeiro, depois nome case-insensitive.
- `fs.read { path }` → `{ path, content }`. Limites: arquivo regular, até
  1 MiB, UTF-8 válido (senão `INVALID_PARAMS` com mensagem humana).
- `fs.readExternal { path }` → `{ path, content }`. Criado em `0.142.0` para
  uma URL local única solta no editor. Exige workspace aberto, mas aceita path
  absoluto fora da raiz; lê arquivo regular UTF-8 até 1 MiB sem seguir
  symlinks nos componentes do caminho. Não registra watcher/LSP e não tem
  operação de escrita correspondente para arquivos externos. A UI mantém a
  aba somente leitura e fora da sessão/rascunhos.
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
  Desde `0.137.0`, a publicação usa `RENAME_NOREPLACE` também para impedir
  sobrescrita se surgir uma colisão entre a validação e a operação.
- `fs.copy { from, to }` → `{ from, to }`. Criado em `0.137.0`; copia um
  arquivo regular ou diretório, inclusive conteúdo binário, no mesmo
  workspace. O pai de `to` precisa existir; `to` não pode existir. Recusa
  links simbólicos, arquivos especiais, a raiz e cópia de pasta para si ou
  descendente. A publicação do destino é atômica e não substitui colisões.
  No aplicativo, usa um Job cancelável `fs.copy`: `event.job.*` informa o
  progresso por bytes; `job.cancel` interrompe a cópia entre blocos e remove
  o staging, preservando a origem. O resultado final continua na resposta
  original de `fs.copy`; não há lote neste método.
- `fs.transferBatch { operation: "copy"|"move"|"import", items: [{ from, to }] }` →
  `{ operation, items: [{ from, to, status, error? }], cancelled }`. Criado em
  `0.138.0`; `import` desde `0.139.0`. Aceita de 1 a 128 pares. Cópia e
  movimento requerem origens no workspace; importação aceita origem local
  absoluta explicitamente selecionada e mantém seu conteúdo original. O
  destino de todas as operações fica no workspace aberto. Valida antes de
  escrever: confinamento, origem existente, destino novo, raiz protegida,
  fontes iguais/ancestrais conflitantes, destinos duplicados, destino dentro
  de outra fonte selecionada e, para cópia, symlinks/entradas especiais em
  toda a árvore. Para importação, também recusa symlink nos componentes do
  caminho da origem e entradas especiais; URLs remotas são rejeitadas na UI.
  Falha dessa etapa é um erro RPC sem mutação. Na execução,
  reutiliza `fs.copy`/`fs.rename` em ordem; `status` é
  `success|failed|cancelled|notStarted`. Cada item conserva o destino do
  pedido para atribuir falhas corretamente. Uma falha de execução não desfaz
  itens anteriores nem impede os posteriores. Cancelar interrompe a cópia
  corrente entre blocos e marca os restantes como `notStarted`; em movimento,
  só interrompe entre itens, pois um rename individual é atômico. No
  aplicativo, o lote é um Job cancelável com progresso; a resposta final traz
  todos os resultados. A interface de seleção múltipla ainda está em ligação.
- `fs.delete { path }` → `{ path }`. Criado no protocolo `0.19.0`; remove um
  arquivo ou diretório (recursivo para diretórios) dentro do workspace. A raiz
  do workspace não pode ser removida (`INVALID_PARAMS`). **Desde 2026-09-02**,
  se o arquivo estava aberto num language server, o core manda
  `textDocument/didClose`: documento apagado que continua aberto deixa
  diagnóstico de um arquivo que não existe mais na aba Problemas. A forma da
  mensagem IPC não mudou.
- `fs.trash { path }` → `{ path }`. Criado em `0.141.0`; move um arquivo ou
  diretório do workspace para a lixeira recuperável do sistema, seguindo a
  especificação FreeDesktop no Linux. Recusa a raiz e paths fora do workspace.
  Se não houver lixeira utilizável, retorna erro e conserva a origem; não cai
  silenciosamente em exclusão permanente. Após sucesso, a UI atualiza árvore
  e abas pelo mesmo evento de remoção já usado por `fs.delete`.

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

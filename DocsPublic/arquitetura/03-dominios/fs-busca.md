# `fs.findFiles`, `fs.search` — o domínio no protocolo IPC

> Parte do [03](../03-protocolo-ipc.md), que tem o formato base, as listas
> canônicas de métodos e eventos e as regras. Movido sem mudar o texto em
> 2026-10-16 ([61](../../roadmaps/61-organizacao-da-documentacao.md) §3.7, O6).

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

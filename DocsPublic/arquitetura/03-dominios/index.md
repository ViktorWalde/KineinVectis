# `index.*` — o domínio no protocolo IPC

> Parte do [03](../03-protocolo-ipc.md), que tem o formato base, as listas
> canônicas de métodos e eventos e as regras. Movido sem mudar o texto em
> 2026-10-16 ([61](../../roadmaps/61-organizacao-da-documentacao.md) §3.7, O6).

## `index.*` — o índice do projeto inteiro

Domínio novo no protocolo `0.94.0` (pilar 0 do `roadmaps/42`, 2026-09-12).
**Decisão do autor:** *"a IDE deve ler o projeto inteiro que for aberto… todas
as funções/arquivos/pastas"* — para Python, C, C++ e Rust.

```text
index.status  {}                        -> IndexStats   (responde tambem sem workspace: idle)
index.symbols { query, limit?, kind? }  -> { symbols[], total, state }   (exige workspace)
index.context { path }                  -> FileContext  (exige workspace; 0.95.0)

event.index.progress { files, symbols }     a cada ~200 arquivos
event.index.finished IndexStats             ao fim do build, a cada incremento e
                                            a cada recarga do contexto

IndexStats   state (idle|building|ready|failed), folders, files, sourceFiles,
             lines, bytes, symbols, functions, types, byLanguage[] { language,
             files, lines, symbols }, skipped[], elapsedMs, error?, context?
ContextSummary cdbDirectory?, cdbEntries, cdbStale, cdbStaleBecause?,
             cargoPackages, cargoTargets, cmakeTargets (0.96.0),
             pythonInterpreter?, pythonOrigin?
IndexSymbol  name, kind, path (relativo a raiz), language, line, endLine,
             container?
FileContext  path (absoluto), language (c|cpp|rust|python|other), unit?, crate?,
             python?, targets[] (0.96.0: os targets do CMake que compilam ou
             listam o arquivo), source?, hint?
CompileUnit  compiler, directory, standard?, includes[] (absolutos), defines[],
             output?, arguments[]
CargoUnit    package, target, kind (lib|bin|test|bench|example|custom-build…),
             edition, manifest, srcPath, features[]
PythonEnv    interpreter, version?, origin (VIRTUAL_ENV|.venv|venv|env|poetry|
             sistema), warning?
```

**O contexto de compilador (0.95.0).** O índice diz *o que há* em cada
arquivo; o contexto diz *com que* ele é compilado — e é carregado no mesmo
job, logo depois dos arquivos:

- **C/C++:** a `compile_commands.json` que o `cdb::status` encontra
  (`.kinein/build`, raiz, `build`, `builddir`, `out/build`), nas **duas
  formas** do padrão do clang (`arguments[]` e `command` dividido como um
  shell: aspas agrupam, `\` escapa); `file` relativo é resolvido contra o
  `directory` e a chave é o caminho **real** (`canonicalize`), porque a CDB do
  Ninja/Meson escreve `../src/a.cpp` e o editor pergunta pelo absoluto.
  `-I`/`-isystem`/`-iquote` colados ou separados viram `includes` absolutos;
  `-D` viram `defines`; `-std=` vira `standard`; `output` vence `-o`. O resto
  segue inteiro em `arguments`. Cabeçalho não tem unidade própria e a `hint`
  diz isso (o clangd deduz pela unidade que o inclui); fonte fora da CDB diz
  "nenhum alvo o compila" — ou, se a CDB envelheceu, qual arquivo a envelheceu.
- **A CDB envelhecida por SUBPASTA.** Medido em 2026-09-12 neste repositório:
  a CDB de 04/09 sem cinco fontes que o `ui/CMakeLists.txt` de 12/09
  acrescentou, e o `cdb::status` dizendo "não envelheceu" porque só compara
  com os arquivos de build da **raiz**. O contexto, com as unidades em mãos,
  sobe de cada diretório de fonte até a raiz procurando um `CMakeLists.txt`
  mais novo que a CDB; acha → `cdbStale` com `cdbStaleBecause`
  (`ui/CMakeLists.txt`) e a `hint` da unidade pede reconfigure. A subida
  **para na raiz do workspace**.
- **Rust:** `cargo metadata --format-version 1 --no-deps` com o `cargo` do kit
  efetivo (se houver `Cargo.toml` na raiz e o cargo existir — nos testes o
  PATH é vazio e nada roda). O arquivo pertence ao alvo pelo `src_path`
  **exato**; senão pelo alvo cujo diretório de `src_path` é o prefixo **mais
  longo** do arquivo; `lib` vence o empate (um `src/x.rs` pertence ao lib e
  ao bin do mesmo pacote, como o rust-analyzer prefere). O `build.rs`
  (`custom-build`) só casa exato: mora na raiz do pacote e, por prefixo, seria
  dono de tudo.
- **Python:** a precedência do `roadmaps/29` §4.1 — `$VIRTUAL_ENV` (só se o
  interpretador existir), `.venv/`, `venv/`, `env/`, `poetry.lock` + `poetry
  env info -p`, e por último o `python3` do PATH **com aviso** (instalar
  pacote nele quebra a distro). `version` é o `--version` do interpretador
  achado.
- **O modelo por alvo do `CMake` (0.96.0):** o `codemodel-v2` do build dir da
  IDE (`.kinein/build`) entra no mesmo contexto. Todo arquivo C/C++ ganha
  `targets` — cabeçalho incluso, porque o target o *lista* mesmo sem o
  compilar. E quando **não há** `compile_commands.json`, a unidade vem do
  grupo de compilação do target (includes, defines, `compileCommandFragments`
  inteiros em `arguments`, `-std=` do fragmento antes do `languageStandard`)
  com o compilador da `toolchains-v1` — ou `"(CXX do kit)"` quando ela não
  foi respondida (CMake < 3.20 ou query antiga), dito em vez de inventado;
  `source` diz `file-api codemodel-v2 (target X), sem compile_commands.json`.
  Com a CDB presente, a CDB vence a unidade e os `targets` ficam.
- **Quando recarrega:** `event.cmake.finished` (o configure reescreve a CDB) e
  um `Cargo.toml` em `event.fs.changed` (um alvo novo muda a que pacote cada
  arquivo pertence) recarregam **só o contexto**, num job curto; os arquivos
  ficam. O `event.index.finished` sai de novo com o `context` novo, e a UI
  pergunta de novo pelo arquivo ativo.

Medido neste repositório pelo core real (2026-09-12): 288 unidades na CDB de
`.kinein/build`, 4 pacotes e 6 alvos do cargo, Python do sistema
(`/usr/bin/python3`, 3.14.7). `ui/src/core_client_index.cpp` → `c++`,
`gnu++23`, 12 `-I`, 9 `-D`; `crates/kinein-core/src/index/context.rs` →
`kinein-core`/`kinein_core` (lib, 2024); `crates/kinein-core/src/main.rs` →
`kinein-core` (bin).

**Na UI:** a barra de status mostra o contexto do arquivo ativo —
`contexto: c++ · gnu++23 · 12 -I · 9 -D`, `cargo · kinein-core (lib, 2024)`,
`python · sistema · 3.14.7 ⚠` — e o detalhe (diretório, origem, dica) ao
pairar. Resposta atrasada de outro arquivo é descartada; trocar de arquivo
limpa até a resposta chegar.

**O que ele lê, e como.** Ao abrir o workspace, um job caminha a árvore
inteira com a **mesma lista de pastas ignoradas do watcher** (`.git`,
`.kinein`, `target`, `build`, `node_modules`…, para os dois verem o mesmo
projeto), conta **todo** arquivo, lê os de fonte (C/C++/Rust/Python) e extrai
as declarações com as **gramáticas Tree-sitter do editor** (a mesma query
`tags` oficial de cada gramática, sem cache — `lang/extract.rs`). Python
entrou na fundação em 2026-09-12 à tarde (`tree-sitter-python` 0.25.0, MIT;
bloco B do `roadmaps/41`): a `tags` oficial dá `function` e `class` (o método
vem como `function` com `container`). Arquivo acima de 4 MiB ou ilegível é
contado e **dito** em `skipped`, nunca sumido. Medido em 2026-09-12 à tarde
neste repositório: 1.022 arquivos, 148 pastas, 72.000 linhas, 3.939
declarações (217 delas em 18 `.py`) em ~2,1 s (build de depuração). O
"4.658" da manhã foi medido antes de o dedup do `fn` em `impl` entrar no
mesmo commit — remedido sem Python, o mesmo código dá 3.720.

**O incremento — e o índice segue o disco INTEIRO (2026-09-12 à tarde).**
Os caminhos de `event.fs.changed` são reindexados no loop principal (um
arquivo é milissegundos) e os totais reemitidos. Uma **pasta nova** é
caminhada inteira (com a mesma lista de pastas ignoradas); uma pasta apagada
leva os arquivos e as subpastas dela. E o que fecha o buraco que existia até
então: a cada `event.index.finished` o Core **registra no watcher todas as
pastas que o índice caminhou** — uma a uma, `NonRecursive`, como o ADR-0001
manda —, então um arquivo criado pelo terminal numa pasta que nenhuma tela
listou chega ao índice pelo mesmo caminho. Antes, só as pastas que a UI
expandiu eram observadas e o arquivo ficava fora até o próximo
`workspace.open`, em silêncio. Medido neste repositório: **148 inotify
watches** (as 148 pastas), contra o limite de 186.243 desta máquina
(`fs.inotify.max_user_watches`); num projeto grande o registro **para no
primeiro erro** e o relata uma vez (`event.fs.watchError`). Provado pelo
core real na exercitação: um arquivo nascido em `src/tarde/` depois do índice
pronto aparece no `index.symbols` sem reabrir nada.

**Sem duplicata e sem renomear a gramática.** A `tags` do Rust captura um `fn`
dentro de `impl` duas vezes (`function` e `method`); o índice fica com a mais
específica por (linha, nome). O que a gramática chama de `class` (a `struct`
do Rust) o índice **não** renomeia.

**A busca** ordena exato > prefixo > substring, sem diferenciar caixa, com
filtro por `kind` e `total` antes do limite. Na UI, `#nome` no Search
Everywhere pede ao índice **e** ao LSP: o índice responde primeiro e **sem
arquivo aberto**; o LSP, quando responde, substitui.

**O que este domínio NÃO é:** semântica (tipos, referências, rename continuam
no clangd/rust-analyzer). O contexto de compilador por arquivo entrou no
`0.95.0` (acima); o que ainda falta no pilar 0 está no `roadmaps/42` §3 P0.

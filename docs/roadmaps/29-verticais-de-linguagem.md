# 29 — Verticais de linguagem: C/C++, Rust e Python sem atrito

> **Classe: PLANO** (`docs/README.md`). Diverge da implementação por natureza.
> **Estado do código medido em 2026-08-29**; **afirmações sobre ferramenta de
> terceiro pesquisadas na fonte na mesma data**, com URL citada (`AGENTS.md`:
> "ao afirmar que uma API se comporta de tal forma, cite fonte e versão").
> **Pedido do autor:** integração nativa ao contexto de uso de C/C++, Rust e
> Python; atrito zero; 100% open source; abstrair a configuração de ambiente de
> C/C++ para o usuário.

## 1. O estado medido, antes de qualquer plano

| Camada | Rust | C/C++ | Python |
| --- | --- | --- | --- |
| Detecção de projeto | ✅ `Cargo.toml` | ✅ `CMakeLists.txt` | ✅ `pyproject.toml`, `setup.py`, `requirements.txt` |
| Tree-sitter (realce, outline) | ✅ | ✅ (C e C++) | ❌ **ausente** |
| Language server | ✅ `rust-analyzer` | ✅ `clangd` | ❌ **ausente** |
| Build | ✅ `cargo build` | ✅ `cmake --build` | ❌ **ausente** |
| Test | ✅ `cargo test` | ✅ `ctest` | ❌ **ausente** |
| Run | ✅ | ✅ | ❌ cai no ramo "não suportado" |
| Qualidade | ✅ `clippy` | ✅ `clang-tidy` | ❌ **ausente** |
| Debug | ✅ `lldb-dap` | ✅ `lldb-dap` | ❌ **ausente** |

**O achado que importa: Python é RECONHECIDO e depois IGNORADO.** O
`workspace/detect.rs` devolve `ProjectKind::Python`, a IDE informa isso ao
usuário — e nenhum subsistema faz nada com a informação. `LanguageId` tem três
variantes (`C`, `Cpp`, `Rust`); a tabela `SERVERS` do `lsp/server.rs` tem duas
entradas; `build.rs` e `test.rs` não contêm a palavra "Python".

Isso é pior que não detectar: a IDE **promete e não entrega**. Quem abre um
projeto Python vê o rótulo certo e um editor sem realce, sem completar e sem
rodar teste.

## 2. A tensão que este documento cria, dita antes de propor

A ordem decidida em 2026-07-17 (`docs/roadmaps/28`) é **profundidade antes de
superfície**: L2–L4 são C/C++/Rust sólidos, e só depois vem superfície nova.

**Python é superfície nova.** Colocá-lo antes de L2–L4 contraria a ordem que o
autor definiu, e o argumento dele continua válido: *"IDE com Docker e sem
cobertura de teste é demo"* — vale igual para Python.

O contra-argumento honesto é que Python **já está prometido na tela**: não é
superfície nova, é dívida de uma promessa existente. As duas leituras são
defensáveis; a escolha é do autor (§6).

## 3. Onde está o risco proprietário — e ele é estreito e específico

Esta seção existe porque a escolha "óbvia" para Python é a errada para este
projeto.

### 3.1 O language server de Python

| Ferramenta | Licença | Serve à Kinein? |
| --- | --- | --- |
| **Pylance** | **Proprietária (Microsoft)** | ❌ **Não.** Licenciada para uso **somente em produtos Microsoft**; restrita aos builds oficiais do VS Code |
| **Pyright** | MIT | ✅ É o motor de tipos aberto sobre o qual o Pylance é construído |
| **basedpyright** | MIT | ✅ Fork comunitário do Pyright que **reimplementa em código aberto** recursos que a Microsoft mantém exclusivos do Pylance (inlay hints, realce semântico, docstrings) |
| **Ruff** (`ruff server`) | MIT | ✅ Complementa: diagnóstico, code actions e formatação |

Fontes: [licença do Pylance](https://marketplace.visualstudio.com/items/ms-python.vscode-pylance/license) ·
[FAQ do pylance-release](https://github.com/microsoft/pylance-release/blob/main/FAQ.md) ·
[basedpyright](https://github.com/DetachHead/basedpyright) ·
[Ruff: o servidor nativo](https://astral.sh/blog/ruff-v0.4.5).

**Recomendação: `basedpyright`, com `pyright` como alternativa.** Dois motivos
concretos além da licença: ele é instalável pelo **PyPI sem exigir Node.js** (o
`pyright` upstream é pacote npm), e traz no servidor aberto os recursos de editor
que a maioria associa a "pyright" mas que na verdade moram no Pylance fechado.

**Armadilha registrada:** a partir do Ruff 0.14.0 há relato de que o servidor
nativo passou a exigir `--preview`
([issue do Zed](https://github.com/zed-industries/zed/issues/46894)). **Medir na
versão instalada antes de fixar a linha de comando** — não deduzir.

### 3.2 O resto da cadeia

Tudo aberto, e nada é linkado: a IDE **executa como processo**, o que também
mantém a GPL do GDB fora de qualquer questão de derivação.

```text
clangd          Apache-2.0 com excecao LLVM      debugpy   MIT
rust-analyzer   MIT / Apache-2.0                 uv        MIT / Apache-2.0
CMake           BSD-3                            Ruff      MIT
Ninja           Apache-2.0                       pytest    MIT
LLDB            Apache-2.0 com excecao LLVM      GDB       GPL-3 (processo)
tree-sitter-python  MIT
```

Fontes: [debugpy](https://github.com/microsoft/debugpy) · [uv](https://github.com/astral-sh/uv) ·
[tree-sitter-python](https://github.com/tree-sitter/tree-sitter-python).

### 3.3 O risco que NÃO é de terceiro: a política que ninguém executa

O `deny.toml` restringe as dependências Rust a `MIT`, `Apache-2.0` e
`Unicode-3.0` — política estreita e correta. **Medido em 2026-08-29:
`cargo-deny` não está no gate e não está instalado.**

A única política de licença do projeto é uma regra que mora num arquivo que
ninguém executa. É a mesma forma da regra de split antes da catraca, e do
`shellcheck` antes de 2026-08-29. **Ligar o `cargo-deny` no gate é a correção
mais barata deste documento.**

## 4. Python: o que falta, e o que cada peça custa

O mecanismo já existe nas três camadas; falta preencher.

```text
BARATO   Tree-sitter Python. `lang/registry.rs` ja tem o padrao para C/Cpp/Rust:
         crate + queries highlights/tags/locals. Realce e outline saem juntos.
         NAO depende de nada instalado na maquina do usuario — e' a unica peca
         com essa propriedade.

BARATO   Language server. `lsp/server.rs` e' uma tabela de 2 entradas;
         acrescentar `basedpyright` e' um `ServerSpec`. Mas so funciona depois
         do item CARO abaixo: sem o interpretador certo, ele indexa a stdlib
         errada e o completar mente.

MEDIO    build / test / run. `cargo` e `cmake` tem comando unico e previsivel;
         Python nao: e' `pytest`, ou `python -m pytest`, ou `uv run pytest` —
         e a escolha depende do INTERPRETADOR.

CARO     O interpretador. E' o verdadeiro trabalho, e e' o analogo Python do
         `compile_commands.json` de C/C++.
```

### 4.1 O interpretador é o problema real

Um projeto Python não diz qual Python usa. Precedência proposta, com o que a
pesquisa confirmou:

```text
1. $VIRTUAL_ENV                 ativado na sessao; o uv trata como prioritario
2. .venv/ no root ou acima      convencao que o uv cria por padrao e que os
                                editores esperam
3. venv/ ou env/                mesma ideia, outro nome
4. uv.lock presente             `uv run` resolve o ambiente sozinho
5. poetry.lock presente         `poetry env info -p` da o caminho — o Poetry
                                cria o venv FORA do projeto (cache do sistema),
                                entao a busca por pasta NAO acha
6. Python do sistema            ultimo recurso, e AVISAR: instalar pacote no
                                Python da distro quebra a distro
```

Fontes: [uv — versões de Python](https://docs.astral.sh/uv/concepts/python-versions/) ·
[uv × Poetry](https://pydevtools.com/handbook/explanation/how-do-uv-and-poetry-compare/) ·
[ambientes Python no VS Code](https://code.visualstudio.com/docs/python/environments).

**O item 5 é o que separa uma detecção que funciona de uma que decepciona:**
quem procura `.venv/` na árvore acha uv e venv, e falha em Poetry justamente
porque o Poetry guarda o ambiente no cache do sistema. Uma detecção que só olha
pasta vai errar num dos gerenciadores mais usados.

Desenho, seguindo `ARCHITECTURE.md` §5:

```text
kinein-protocol/src/python.rs   PythonInterpreter { path, version, origem, venv }
kinein-core/src/python.rs       precedencia acima + probe de versao
handlers/python.rs              python.interpreters / python.select
```

A precedência vira lista ordenada e testável — o mesmo formato de `KNOWN_TOOLS`
em `tools.rs`, e pelo mesmo motivo: **detectar é agnóstico, e a decisão nunca se
deduz do PATH.**

## 5. C/C++ sem atrito: o que já existe e o que falta

Mais adiantado do que parece. Já implementado:

```text
✅ configure automatico UMA vez ao abrir (ProjectHealthController), com aviso
   acionavel na falha e sem loop
✅ `-DCMAKE_EXPORT_COMPILE_COMMANDS=ON` no configure do core
✅ clangd recebe `--compile-commands-dir` quando a CDB existe
✅ leitura de `CMakePresets.json` + `CMakeUserPresets.json`
```

O que ainda cobra atrito:

**a) Só CMake tem caminho feliz.** Projeto C/C++ com Makefile, autotools ou
script próprio é detectado como `Unknown`, não recebe CDB, e o clangd sobe sem
flags — errando include de tudo. Saídas, em ordem de custo:

```text
Meson      JA gera compile_commands.json no build dir. Basta DETECTAR
           meson.build e apontar o clangd para la. E' o mais barato.
Bear       (MIT) gera a CDB interceptando o build: `bear -- make`. E' a
           ferramenta de escolha quando o build system nao produz CDB.
compile_flags.txt   ultimo recurso: um argumento por linha, MESMAS flags para
           todo arquivo. Serve para "abrir e ler", nao para projeto real.
```

Fontes: [Bear](https://github.com/rizsotto/bear) ·
[especificação da JSON Compilation Database](https://clang.llvm.org/docs/JSONCompilationDatabase.html) ·
[clangd — troubleshooting](https://clangd.llvm.org/troubleshooting).

**b) Arquivo já aberto não recebe flags novas.** O clangd **tem** hot-reload da
`compile_commands.json` desde a versão 12 — ele reconfere a cada ~5 s —, mas
**arquivos já abertos continuam com a compilação em cache**: é preciso reabrir o
arquivo ou reiniciar o servidor.

Fonte: [D92663 — hot-reload no clangd](https://reviews.llvm.org/D92663) ·
[vscode-clangd #42](https://github.com/clangd/vscode-clangd/issues/42).

Isso corrige o comentário que hoje está em `lsp/server.rs` ("servidor já em
execução não recarrega flags"): a afirmação é verdadeira para o **documento
aberto**, e falsa para a CDB. **A ação da IDE é reabrir os documentos abertos
após um `cmake.configure` bem-sucedido** — não reiniciar o servidor, que joga o
índice fora.

**c) CDB desatualizada não avisa.** Se o `CMakeLists.txt` muda e ninguém
reconfigura, o clangd usa flags velhas e o usuário vê erro sem causa. Detectável
por mtime (`CMakeLists.txt` × `compile_commands.json`) e endereçável com o mesmo
aviso acionável que o auto-configure já usa.

**d) Toolchain não é entidade.** Compilador, gerador, sysroot e flags são
implícitos: o que estiver no PATH. Não existe o "kit" que CLion e Qt Creator
expõem, então trocar de compilador ou cruzar-compilar é editar arquivo à mão.
É o **B2 do TR2** ("targets, perfis e toolchains como entidades"), e é o item
mais caro desta seção.

Os quatro juntos são o que separa "funciona quando você sabe o que fazer" de
"funciona".

## 6. Sequência proposta, e a decisão que ela exige

**Passo 0, sem decisão nenhuma — ligar o `cargo-deny` no gate.** Minutos. Fecha
a única política de licença do projeto que hoje não é verificada (§3.3).

Depois, duas ordens defensáveis:

```text
ORDEM A (mantem a decisao de 2026-07-17: profundidade antes de superficie)
  1. C/C++ sem atrito: Meson, aviso de CDB velha, reabrir documentos apos
     configure  (itens a, b, c da §5 — os tres sao pequenos)
  2. Toolchain como entidade (B2 do TR2 — grande)
  3. Python completo

ORDEM B (paga a promessa que ja esta na tela)
  1. Python minimo: Tree-sitter + basedpyright + deteccao de interpretador
  2. C/C++ sem atrito
  3. Toolchain como entidade
```

**Recomendação: ORDEM A, com uma exceção.** A gramática Tree-sitter de Python é
o único item que **não depende de nada instalado na máquina do usuário** e tira
o pior sintoma — arquivo `.py` sem realce nenhum. Entregar só isso, e deixar o
resto do Python para depois de C/C++, honra as duas leituras: a IDE para de
mentir na tela e a ordem de profundidade continua de pé.

O que **não** fazer nessa exceção: **anunciar suporte a Python.** Realce sem LSP
e sem `pytest` é realce, não vertical — e a §1 existe justamente porque prometer
mais do que se entrega é o defeito atual.

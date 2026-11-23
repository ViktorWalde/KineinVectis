# Regras de C++23

Dono: os métodos de C++ do core e o mecanismo que impõe cada um. A defesa
contra cada defeito clássico da linguagem está em
[erros-conhecidos-de-cpp-e-defesas.md](erros-conhecidos-de-cpp-e-defesas.md).
As flags estão em [perfis-de-compilacao-e-hardening.md](perfis-de-compilacao-e-hardening.md).

"Revisão" na última coluna quer dizer que nenhuma ferramenta impõe a regra:
quem revisa confere. Toda regra marcada assim é candidata a mecanismo.

## Linguagem

| Regra | Por quê | Quem impõe |
| --- | --- | --- |
| C++23 sem extensões (`CMAKE_CXX_EXTENSIONS OFF`) | Código portável entre GCC, Clang e MSVC, que o clangd entende | `build.contract` reprova `-std=gnu++` |
| C++26 só por ADR, um recurso por vez, quando GCC, clangd e MSVC o suportarem sem a etiqueta de experimental, protegido por macro de teste de recurso (`__cpp_*`) | O C++26 não foi publicado (a votação DIS de 2026-10-23 reprovou); o GCC 16 marca o suporte como experimental; o Clang oficial 23 não tem reflexão (P2996) | Revisão; a sondagem `cxx26-probe` só detecta incompatibilidade |
| Identificadores em inglês; comentários e documentação em português | ENG-37 da 0.3: 14 mil ocorrências de identificador em português viraram dívida | Revisão |

## Recursos e posse

| Regra | Por quê | Quem impõe |
| --- | --- | --- |
| RAII para todo recurso; nada de `new` e `delete` soltos | Vazamento e liberação dupla | `cppcoreguidelines-owning-memory`, `cppcoreguidelines-no-malloc`; ASan e LSan |
| `QObject` vive na árvore de pais do Qt; todo o resto em RAII; nunca os dois no mesmo objeto | Dois donos para o mesmo objeto é liberação dupla | Revisão; só existe em `core/qt` |
| `std::string_view` e `std::span` só como parâmetro, nunca guardados em membro nem devolvidos de função que recebeu temporário | Referência pendurada | `bugprone-dangling-handle`, `-Wdangling-reference` (GCC), `-Wdangling` (Clang); ASan `stack-use-after-return` |
| Regra do zero; se uma operação especial é declarada, as cinco são | Cópia rasa e liberação dupla | `cppcoreguidelines-special-member-functions` |

## Erros

| Regra | Por quê | Quem impõe |
| --- | --- | --- |
| `std::expected` para erro esperado (arquivo ausente, ferramenta que falhou, prazo vencido) | O erro faz parte do tipo e não é esquecido | Resultado descartado reprova: `bugprone-unused-return-value` com `std::expected` (`core/.clang-tidy`) |
| Exceção só para o que não é esperado, e nunca atravessa thread nem o laço de eventos do Qt | Exceção que escapa de thread chama `std::terminate`; pelo laço do Qt, o comportamento não é definido | `bugprone-exception-escape` em função `noexcept`; corpo de thread e slot do adaptador são `noexcept` |
| Cada causa de falha tem nome próprio (`DeadlineExceeded`, `RequestCancelled`, `WorkspaceUntrusted`...) | L19: o prazo vencido saía como "obsoleto" | Teste de cada caminho de falha |

## Concorrência e execução

As invariantes E1 a E10 estão em
[execucao-e-concorrencia.md](../arquitetura/execucao-e-concorrencia.md). Em
termos de código:

| Regra | Quem impõe |
| --- | --- |
| `std::jthread` com `std::stop_token` para trabalho cancelável | Revisão |
| Thread nunca solta: sem `detach()` | clang-tidy (`CustomFunctions`), `gate.detach` |
| Um dono por dado; threads trocam mensagens | TSan; revisão |
| Variável global não constante é proibida | `cppcoreguidelines-avoid-non-const-global-variables`; `-Wglobal-constructors` e `-Wexit-time-destructors` em `core/lib` (Clang) |
| Função da biblioteca C que não é segura em thread é proibida | `concurrency-mt-unsafe` |
| O laço nunca bloqueia; toda chamada externa tem prazo e cancelamento | Guarda de E/S no laço (a projetar) |

## Fronteiras

| Regra | Quem impõe |
| --- | --- |
| `core/lib` sem Qt, testável sem laço de eventos | O preset `dev` não procura o Qt |
| O moc nunca vê `core/lib` | AUTOMOC só nos alvos de `core/qt` |
| Processo só pelo lançador | clang-tidy (`CustomFunctions`), ativo e medido ([processos](../arquitetura/processos-e-confianca.md)) |
| Dependência entre módulos só para camada de baixo | `kinein_add_module`, proposto ([módulos](../arquitetura/modulos-e-camadas.md)) |
| Nada executa antes da confiança no workspace | Teste do projeto hostil (RNF-SEG-01) |
| Dependência externa nova só com aprovação do Viktor, com licença conferida | Revisão |
| Uma responsabilidade, um módulo dono. Variação nova entra como adaptador de uma porta que já existe (aberto/fechado), nunca como cópia ajustada de código existente | `gate.duplication` (T1 a T3); a mesma lógica escrita de outro jeito (T4), na revisão (P20) |

## Supressões

- `NOLINT` só na forma `NOLINT(<checagem>): <motivo>`, na linha, nunca em bloco.
- Aviso do compilador não se desliga por arquivo; se uma flag não serve, ela
  sai do perfil, com o motivo no `core/cmake/`.
- O `-Werror` dos presets de desenvolvimento não se desliga para passar um build.

## Comments

One standard covers every comment in `core/` and `.github/`: C++, CMake, Python, YAML and
`.clang-tidy`. It is the same in every file and for every author, and a change to it is a
decision of the maintainer. A comment that breaks it fails review, the same as a compiler
warning. No tool enforces these rules in the CI.

| # | Rule | Reason |
| --- | --- | --- |
| C1 | American English, complete sentences that end with a period, present tense, impersonal technical voice | One voice across the codebase; a reader never has to tell an opinion from a rule |
| C2 | ASCII only | Keeps the text readable in every terminal, diff and log, and leaves no room for look-alike characters |
| C3 | `//` in C++ (`/* */` only as corpus data); `#` in CMake, YAML, `.clang-tidy` and shell; docstrings in Python ([PEP 257](https://peps.python.org/pep-0257/)), with the function summary in the imperative (`Return ...`) | One syntax per language |
| C4 | At most 100 columns, the `ColumnLimit` of `.clang-format`. Only a URL, a path, a regular expression or a corpus line that cannot break may exceed it | The width of the code |
| C5 | A comment sits on its own line, above the code it explains. On a code line, only `NOLINT(<check>): <reason>`, `} // namespace <name>`, `#endif // <condition>`, a corpus mark, and `# vX.Y.Z` after an action pinned by SHA | Each allowed form is read by a tool or closes a block |
| C6 | Every file that accepts comments opens with the header below | The reader finds the owning document from any file |
| C7 | A comment states what the code cannot: the reason, the contract, the source, the limit. It never paraphrases the code | A paraphrase goes stale as soon as the code changes |
| C8 | State, not history: no "now", "new", "old", "still", "already", "changed", "used to" | History is in git |
| C9 | A technical claim (version, flag, API behavior) carries its source in one of three forms: `(<document title>, <section>)`; `Measured on YYYY-MM-DD with <tool> <version>.`; `Not verified: <reason>.` | A claim without a source cannot be checked |
| C10 | A decision is cited as `the maintainer's decision of YYYY-MM-DD (<ID>)`; a pending item by its ID in [decisoes-pendentes.md](../decisoes-pendentes.md). No `TODO`, `FIXME` or `XXX` | Pending work has one owner, not one per file |
| C11 | A file is cited by its path from the repository root (`docs/...`, `core/...`); a 0.3 lesson by its ID in [licoes-da-0.3.md](../licoes-da-0.3.md) | The path works from any directory |
| C12 | Never a person's name or user name ("the maintainer"), a mention of AI tools or assistants, or commented-out code outside a corpus | The code describes the product, not who wrote it |

The header (C6) uses `//` or `#`. In Python it is the module docstring with the same lines,
and its first line is in the imperative because it is also the `--help` text.

```text
# <One sentence: what the file is or does.>
#
# Documented in: <path from the repository root>[, section "<heading>"]
#
# <Optional paragraphs: rules, sources, limits.>
```

Corpus marks are `// reject: <what the defect is>` and `// accept: <what the form is>`, with no
final period. In the duplication corpus the mark is `// reject: <name of the original function>`.
Corpus code is data: `clang-format` does not reformat it, but its comments follow this standard.

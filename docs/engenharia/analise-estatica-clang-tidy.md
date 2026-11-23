# Análise estática: clang-tidy

Dono: o que o clang-tidy confere no core, o que está desligado e como roda. A
configuração é `core/.clang-tidy`, independente do `.clang-tidy` da raiz, que
segue valendo para `ui/`.

## Como roda

- **No build:** o preset `lint` compila com o Clang e roda o clang-tidy em cada
  unidade (`CMAKE_CXX_CLANG_TIDY`). Qualquer achado reprova a compilação
  (`WarningsAsErrors: '*'`).
- **No editor:** o clangd lê o mesmo `core/.clang-tidy`.
- **Versão:** clang-tidy 23.1.5. Com o arquivo atual são 394 checagens ativas
  (`clang-tidy --list-checks`, 2026-11-22).

## Categorias

`bugprone-`, `cert-`, `concurrency-`, `cppcoreguidelines-`, `modernize-` e
`performance-`, que são as do plano, mais o `clang-analyzer-`. O analisador
segue caminhos de execução e pega o que a checagem local não pega:
desreferência de nulo, uso depois de liberar, vazamento, uso depois de
`std::move` e bloqueio dentro de seção crítica
([erros conhecidos](erros-conhecidos-de-cpp-e-defesas.md)). Não está nas
categorias pedidas; sai se o Viktor preferir.

`readability-`, `misc-`, `hicpp-` e `portability-`, que a raiz usa, ficam de
fora: estilo não é defeito, e o `hicpp-` repete o `cppcoreguidelines-`.

## Desligadas, com o motivo

Todas vêm do `.clang-tidy` da 0.3 sem medida nova e voltam a ser avaliadas
com o primeiro código do core. O motivo de cada uma está comentado no próprio
`core/.clang-tidy`.

| Checagem | Motivo |
| --- | --- |
| `bugprone-easily-swappable-parameters` | Ruído alto na 0.3; a defesa planejada é tipo próprio para parâmetro do domínio |
| `cppcoreguidelines-avoid-magic-numbers` | Estilo, sem defeito associado |
| `cppcoreguidelines-non-private-member-variables-in-classes` | Struct de dados com campos públicos é a forma das Core Guidelines (C.2, C.131) |
| `modernize-use-trailing-return-type` | Estilo |

Desligar mais uma exige o achado concreto que a motivou, registrado na tabela.

## Opções fixadas

- `bugprone-unused-return-value.CheckedReturnTypes` inclui `::std::expected`.
  É o padrão do clang-tidy 23, fixado para não mudar com a versão. Com
  `AllowCastToVoid: false`, nem o cast para `void` descarta o erro.
- `bugprone-unsafe-functions.CustomFunctions` lista as funções de criar
  processo: o gate "processo só pelo lançador"
  ([processos](../arquitetura/processos-e-confianca.md)). A lista padrão da
  checagem (`gets`, as do Anexo K) continua ligada.

## Gates sobre o clang-tidy

Uma opção do clang-tidy usada como gate tem corpus em `core/tests/gates/` e
um teste no preset `lint` que mede a eficácia a cada build
([gates](gates-e-medida-de-eficacia.md)). O teste roda só a checagem do gate,
com as opções do `core/.clang-tidy`: mudar a opção sem atualizar o corpus
reprova.

## Supressão

`NOLINT(<checagem>): <motivo>`, na linha. Nunca `NOLINT` sem checagem e
nunca `NOLINTBEGIN`/`NOLINTEND`.

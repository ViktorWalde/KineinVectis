# Processos e confiança

Dono: como um processo externo nasce e o que um projeto não confiável pode
fazer. Fecha as lições L05, L06 e L18 ([lições](../lessons-from-0.3.md)).

## O lançador

O lançador é o único caminho para criar processo no core: `fork`, `exec*`,
`posix_spawn*`, `system`, `popen`, PTY e `QProcess`. O desenho é do Viktor; o
pedido de lançamento tem de carregar, por tipo, o que a 0.3 deixava a cargo de
cada local de chamada (64:460-470):

| Campo | Por quê |
| --- | --- |
| Origem do programa: da IDE (detectado), do projeto, ou ferramenta configurada pelo projeto | A confiança é decidida pela origem (L05) |
| Argumentos, diretório e ambiente (lista permitida) | Ambiente herdado inteiro vaza segredo e configuração |
| Prazo, obrigatório | L08 |
| Teto de saída, obrigatório | Saída sem limite enche a memória |
| Cancelamento | E2 e E8 da [execução](execution-and-concurrency.md) |

E as regras:

- **Argumento que nomeia programa** (`--query-driver` do clangd, por exemplo)
  passa pela mesma regra de origem. Foi o vetor E5 do INF-23 (64:122).
- **O git é ferramenta configurada pelo projeto**: o `.git/config` define
  `fsmonitor`, filtros e hooks (64:476-477).
- **O compilador fixado no kit** é do projeto enquanto o caminho vier de um
  arquivo da árvore do projeto (64:480-482).
- **Falha de `exec`** volta como erro tipado, com o erro do sistema, sem
  aborto nem core dump (L18).
- **Todo lançamento** registra programa, origem, decisão, duração, código de
  saída e se o prazo venceu (RNF-SEG-02).
- **Os testes** também passam pelo lançador, com a API de ferramentas falsas.

**Quem impõe "só o lançador cria processo".** No Rust era o compilador
(`disallowed_methods = "forbid"`, 64:248-265). Em C++ não há equivalente
direto. Desde 2026-11-22 é o `bugprone-unsafe-functions.CustomFunctions` do
`core/.clang-tidy`, que acusa toda função de criar processo (POSIX, glibc,
`QProcess` e Windows) em qualquer lugar do core. Ele casa chamada direta,
qualificada, via `std::`, ponteiro, macro, `using` e método de classe; não
casa construtor. Eficácia: 40 de 40 defeitos e 16 de 16 formas corretas, pelo
teste `gate.launcher` ([gates](../engineering/gates-and-effectiveness.md#registro)).
Quando `sys.process` existir, a pasta dele ganha um `.clang-tidy` próprio que
libera só ela, e o `VISIBLE_TO` deixa só o `launch` ligá-lo
([módulos](modules-and-layers.md)).

## O modo restrito

Do 64 §5.5 (64:508-528), restrito a C e C++, com o Neovim (65 §21).

| Capacidade | Sem confiança | Com confiança |
| --- | --- | --- |
| Ler, navegar, editar, buscar | sim | sim |
| Editor novo (`nvim`) | sobe com Landlock; sem Landlock, o projeto abre sem o editor novo | sobe com Landlock |
| clangd | sobe com `--query-driver` só de compilador detectado pela IDE | sobe com o kit inteiro |
| Git | desligado, com aviso | sim |
| CMake configure, compilar, executar, testar, depurar, gravar | pede confiança no gesto | sim |
| `--version` de ferramenta do projeto | não | sim |

- **Nada roda antes da confiança**, inclusive o configure do CMake e o
  `--query-driver` do clangd.
- **A regra mora no core**, no lançador. A QML mostra o estado e pede o gesto.
  Na 0.3, a QML segurava o configure automático (64:306-309); no core novo, o
  core recusa.
- **A confiança** continua no estado global da pessoa, pela raiz canônica do
  projeto, com as pastas confiáveis da DP-02 (64:525-528).
- **Aceite:** o projeto hostil do 64 §2.1 mais os vetores do Neovim, com zero
  marcas (RNF-SEG-01) e um mutante que tira a guarda e precisa reprovar.

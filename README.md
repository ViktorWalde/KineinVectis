# A prova de conceito do Neovim como motor de edição (ENG-60, fase B)

Este ramo guarda a prova de conceito do 65 da Kinein Vectis
(`DocsPublic/roadmaps/65-avaliacao-do-neovim-como-motor-de-edicao.md`, no
`main`): o código, os testes e os resultados que respondem aos
critérios R1 a R13 (65 §1) e aos quinze itens (65 §12.2). **Não é código do
produto**: não compila com ele, não passa pelos gates dele e não tem
histórico em comum com o `main`. A integração no produto é a fase D do 65
§14, feita por partes, cada uma com os gates do produto.

Os dois primeiros commits deste ramo são o código: a prova (`2d1ac38d`) e o
`XDG_RUNTIME_DIR` próprio de cada rodada (`6e828799`). O terceiro traz os
resultados da rodada de 2026-11-15, feita a partir de uma cópia limpa do
`6e828799` (`resultados/`).

## A pilha

```text
ui/ (Qt Quick, offscreen)  --socket-->  nvim --embed  <--stdio--  supervisor/ (o papel do core)
                                                                    |-- Landlock no nvim (sem execve)
                                                                    |-- relé LSP: clangd, rust-analyzer
                                                                    |-- espelho do buffer e rascunho
runtime/ : o init.lua e o Lua da Kinein (edição sem modo, atalhos, LSP pelo relé)
driver/  : os testes, em Python, que dirigem a pilha pelo caminho de uma pessoa
base/    : a instrumentação do editor atual, para a comparação
```

## O ambiente das medidas

Linux 7.3 com Landlock ABI 11; Neovim 0.12.6 (pacote oficial); Qt 6.12.0;
GCC 16.2.1; Rust 1.96.1; clangd 23.1.4; rust-analyzer 1.96.1; Python 3.14;
Xvfb e at-spi2 para o R10. A data e o commit de cada rodada estão no commit
dos resultados.

## A linha de base

O editor atual é o produto no commit `308c1561` com o
`base/instrumentacao.patch`, que mexe só no harness de digitação que o
produto já tem (`ui/src/typing_perf_harness.cpp`), e só atrás de variável
de ambiente; nada muda no caminho da tecla:

- `KINEIN_PERF_OPEN_PROBE=1`: da abertura do arquivo ao primeiro quadro com
  o texto (R2), pelo mesmo critério de sincronia e troca de quadro da prova;
- `KINEIN_PERF_DUMP_LINE=1`: imprime a linha digitada no fim (R4);
- `KINEIN_PERF_CRASH_AFTER_MS=N`: `SIGKILL` em si mesmo N ms depois da
  última tecla (a queda do editor, R4);
- `KINEIN_PERF_HOLD=1`: não sai no fim, para ler memória, CPU e a árvore
  de acessibilidade (R10, R12);
- a linha `KINEIN_PERF typing_window=` marca o início da digitação no log,
  para o custo de quadro (`driver/render.py`).

A base usa o build Release do produto (com o endurecimento do
`KINEIN_ENABLE_HARDENING`, sem sanitizer, que só entra no Debug); a prova
não usa endurecimento.

## Como reproduzir

Pela raiz deste ramo, com o `nvim` 0.12.6 no `PATH`:

```sh
cmake -S ui -B ui/build -G Ninja -DCMAKE_BUILD_TYPE=Release && cmake --build ui/build
cargo build --release --locked --manifest-path supervisor/Cargo.toml

mkdir -p base/arvore
git -C <checkout da Kinein Vectis> archive 308c1561 | tar -x -C base/arvore
patch -d base/arvore -p1 < base/instrumentacao.patch
(cd base/arvore && cargo build --release --locked -p kinein-core)
cmake -S base/arvore -B base/arvore/build/release -G Ninja -DCMAKE_BUILD_TYPE=Release
cmake --build base/arvore/build/release
```

Os testes (cada um grava `resultados/<nome>.json`):

```sh
python3 -I driver/provas.py            # os testes funcionais; um nome roda só ele
sh driver/a11y.sh                      # R10, num Xvfb e numa sessão D-Bus próprios
python3 -I driver/desempenho.py        # item 15: R1, R2, R12 e CPU, 5 rodadas alternadas
python3 -I driver/render.py            # o custo de desenhar um quadro, nos dois lados
python3 -I driver/recuperacao_base.py  # R4 no editor atual; `2500` roda o controle positivo
python3 -I driver/normalizar.py        # tira dos resultados o caminho da máquina
```

A rodada de `resultados/` rodou uma etapa por vez, sem outra medida em
paralelo, nesta ordem: `provas.py`, `a11y.sh`, `recuperacao_base.py`,
`recuperacao_base.py 2500 2500 2500` (o controle positivo três vezes),
`desempenho.py` e `render.py`; depois, o `normalizar.py`. O `rodada.log`
traz o commit, o início, o fim e o código de saída de cada etapa; o
`<etapa>.log`, a saída dela. O `runtime-de-quem-roda.diff` compara a
listagem do `XDG_RUNTIME_DIR` de quem roda antes e depois da rodada (`diff
rc=0`: nada sobrou nele).

## O isolamento

- Tudo roda offscreen, salvo o R10, que sobe um Xvfb e uma sessão D-Bus
  próprios: nada abre na sessão gráfica de quem roda. A sessão D-Bus não
  ativa serviço nenhum (`driver/a11y-sessao.conf`).
- Cada rodada usa `HOME` e `XDG_*` próprios, dentro de `tmp/`: nada lê nem
  grava a configuração de quem roda. O `XDG_RUNTIME_DIR` também é próprio,
  numa pasta curta em `/tmp` apagada no fim: os sockets dos dois lados não
  vão ao de quem roda, nem depois de uma queda provocada.
- O log do Qt vai para o log da rodada (`QT_FORCE_STDERR_LOGGING=1`), não
  para o journal do sistema.
- Os resultados passam pelo `driver/normalizar.py` antes de entrar no git:
  a raiz da prova vira `$PROVA` e a pasta pessoal vira `~`.

## Licença

A mesma do repositório da Kinein Vectis: GNU AGPL 3.0 para o código.

# ADR-0006 — `exmex` para a fórmula que o usuário digita

- Status: aceito
- Data: 2026-09-05
- Escopo: domínio `sim` / etapa 28 (`../roadmaps/31`, `../arquitetura/34`)

## Contexto

A etapa 28 põe o usuário digitando a equação dentro de um conceito escolhido, e
a IDE tem de **alertar na hora da digitação** quando a fórmula não é daquele
conceito (decisão do autor, `../roadmaps/31` §9.1). Isso exige saber **quais
variáveis a fórmula usa** antes de calcular qualquer coisa — ou seja, a árvore
da expressão.

Escrever parser e avaliador próprios seria reimplementar o que já existe
consolidado, que é a regra de uma linha que define esta IDE
(`LEITURA_TECNICA` §1). E o caminho alternativo — gerar código e compilar com o
toolchain do usuário, que é o idioma da casa — **não serve para esta função**:
um compilador diz `error:` ou nada, nunca *"esta fórmula não é de oscilador"*.
Medido, ele também sai caro: 490 ms de `cargo` incremental por edição de fórmula
para economizar 3 ms de conta num pêndulo de 10⁵ passos, com ponto de virada em
21,5 milhões de avaliações (`../roadmaps/31` §8.3, §9.3).

## Decisão

Adotar `exmex` `=0.21.0` em **Modo A**, com a feature `partial`, ligado somente
pelo `kinein-core`.

Cinco alternativas foram medidas contra o `cargo-deny` 0.20.2 **deste
repositório**, num projeto isolado, antes da escolha (`../roadmaps/31` §8.1):

```text
crate       licenca             ultimo release   deny.toml deste repo
exmex       MIT OR Apache-2.0     2026-05-23     PASSA     <- escolhido
fasteval    MIT                   2020-01-25     PASSA
rsc         MIT                   2024-03-31     PASSA
meval       Unlicense/MIT         2018-09-30     PASSA
mexprp      MPL-2.0               2022-11-25     REPROVA
evalexpr    AGPL-3.0-only         2025-11-26     REPROVA
```

**O `exmex` é o único MANTIDO que passa.** Os outros três que passam pararam em
2020, 2018 e 2024.

**As duas reprovações merecem registro, porque ensinam:**

- **`evalexpr` é o mais usado do ecossistema** (9,96 M de downloads, 2,11 M
  recentes — mais que os outros cinco somados) e **trocou de MIT para
  AGPL-3.0-only na v12.0.0, em 2024-10-17**. Quem o adotou antes e rodou
  `cargo update` passou a distribuir AGPL sem tomar decisão nenhuma. É a lição
  do Grafana repetida dentro de um crate, e a prova de que licença se verifica a
  cada build e na transitiva — fixá-la uma vez, na adoção, não teria pegado.
- **`mexprp` reprova por onde ninguém olharia:** a licença dele é MPL-2.0, que o
  `docs/integracoes/README.md` admite após revisão. Quem reprova são as
  transitivas `rug` e `gmp-mpfr-sys` (GMP/MPFR), **LGPL-3.0-or-later**.

## Auditoria

Medida em 2026-09-05, nesta máquina, contra o crate publicado:

- upstream oficial: `bertiqwerty/exmex`;
- licença do crate: SPDX `MIT OR Apache-2.0`, do manifest publicado;
- MSRV publicado: Rust **1.80.1**, abaixo do MSRV 1.85 do workspace;
- runtime: **zero** ocorrências de `std::net`, `std::process`, `std::fs`,
  `std::env`, `unsafe` ou cliente HTTP em `src/` — é cálculo puro;
- **custo real nesta árvore: +1 crate.** Os seis transitivos (`regex`,
  `regex-automata`, `regex-syntax`, `aho-corasick`, `memchr`, `smallvec`) já
  estavam todos no workspace. A medição isolada de +7 valia para um projeto
  vazio, não para este;
- a feature `partial` (derivada parcial) é `partial = []`: não traz dependência;
- versão fixada em `=0.21.0`, checksum
  `e9a53dccfc387af263e69c1eb53d13a4e9b26a7134ffdd38398a208374cbf851`;
- `cargo deny check` completo verde no workspace real com ele dentro.

## As quatro armadilhas medidas, e o que o desenho faz com cada uma

Exercitar o crate — em vez de ler a documentação dele — achou quatro coisas
(`../roadmaps/31` §8.2). **Três delas o desenho resolve; a quarta a IDE absorve.**

1. **A ordem das variáveis é ALFABÉTICA, não a da fórmula.** `"a*t + v0"` e
   `"v0 + a*t"` devolvem os dois `["a","t","v0"]`, e `eval()` recebe um slice
   **posicional**. Quem montar esse slice na ordem de leitura calcula a física
   errada **sem erro nenhum** — falha silenciosa, a categoria que este
   repositório já pagou três vezes.
   **Resolvida por desenho:** a ligação de variáveis é explícita (decisão do
   autor, `../arquitetura/34` §5.0), então o vetor se monta pela ligação e a
   posição deixa de importar. Com gate de mutação: trocar duas variáveis de
   papel tem de mudar o resultado.
2. **`pi` minúsculo é variável livre**, não constante (`PI` maiúsculo é que é).
   `sin(2*pi*t)` devolve `vars ["pi","t"]`.
   **Resolvida pela mesma ligação:** vira variável sem papel, que pede ligação e
   não tem a que ligar — aparece como pergunta na tela.
3. **Divisão por zero é aceita e vira `inf`/`NaN` calados:** `1/x` com `x=0` dá
   `Ok(inf)`; `x/x` dá `Ok(NaN)`; `log(0)` dá `-inf`; `sqrt(-1)` dá `NaN`.
   **Absorvida pela IDE:** resultado não-finito é erro do domínio, não número.
4. **A mensagem de erro do parser não é mostrável:** 356 caracteres com
   **endereço de ponteiro** (`0x...`) dentro.
   **Absorvida pela IDE:** ela classifica o erro e escreve a própria mensagem —
   como já faz com `secretRequired` e com o `code` do `requestFailed`.

Do lado bom, medido: derivada parcial confere numericamente
(`d/dx (x²y + sin x)` em (2,3) = 11.583853163452858) e o vocabulário está
completo para física — `sin cos tan asin acos atan sinh cosh tanh exp log log2
log10 sqrt abs signum floor ceil round`.

## Consequências

- o `sim` ganha a árvore da fórmula, que é o que a checagem de conceito precisa;
- o desempenho medido é 7,6× o nativo — 33 ms por 1.000.000 de avaliações, e os
  quatro avaliadores testados concordam **até a nona casa** com o código
  compilado, o que dá o determinismo que a reprodução de janela exige
  (`../arquitetura/34` §7.1);
- compilar continua sendo caminho previsto para escala grande, mas **por cima**
  do parser, nunca no lugar dele (`../arquitetura/34` §6);
- as quatro armadilhas viram teste, não comentário.

## Alternativa recusada, e por quê

**Escrever o parser na Kinein.** Recusada pela regra de não reimplementar
consolidado, e sem ganho: nenhuma das quatro armadilhas acima desapareceria por
ser código nosso — três delas são decisões de desenho que teríamos de tomar
igual, e a quarta (a mensagem) já é nossa de qualquer forma.

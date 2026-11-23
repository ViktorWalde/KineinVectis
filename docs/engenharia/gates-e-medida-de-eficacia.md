# Gates e medida de eficácia

Dono: em que ordem os gates do core rodam, como um gate próprio prova que
funciona e o registro das medidas.

## Ordem

1. **Determinísticos:** compilador com `-Werror`, sanitizers, clang-tidy,
   testes (contrato do build, unidade, complexidade, diferencial, queda) e
   validação do contrato do protocolo.
2. **Gate próprio**, só com a eficácia medida pelo método abaixo e registrada
   neste arquivo.
3. **Modelo de IA** ajuda a revisar o diff, propor caso de teste e montar o
   corpus de defeitos. Quem decide são os números.

Gate de heurística de texto sobre código não entra (L15: na 0.3, esses gates
detectaram 36 de 120 defeitos e rejeitaram 14 de 15 formas corretas).

## O método (63 §D.5 e 66 §2.1)

- **N defeitos plantados** que o gate precisa reprovar, cada um aplicado
  sozinho numa cópia da árvore, nunca no checkout.
- **M formas corretas** que o gate precisa aceitar, incluindo as formas
  difíceis (comentário, texto literal, macro, código gerado).
- **Mutante equivalente** (a mudança não altera a propriedade) não conta como
  defeito e é registrado como equivalente, com o motivo.
- O gate entra no CI só com N de N reprovados e M de M aceitos. Abaixo disso,
  a medida fica registrada com o veredito "não entra".
- O corpus fica versionado junto do gate, e a medida volta a rodar quando o
  gate muda.

## Registro

### `build.contract` e `build.elf_hardening` (2026-11-22)

Medido numa cópia de `core/`, um defeito por vez, com GCC 16.2.1, Clang 23.1.5
e GNU ld 2.47.

| # | Defeito plantado | Preset | Reprovado por |
| --- | --- | --- | --- |
| D01 | `CMAKE_CXX_EXTENSIONS ON` | dev | `build.contract` |
| D02 | Sem `-D_FORTIFY_SOURCE=3` | dev | `build.contract` |
| D03 | `_FORTIFY_SOURCE=2` | production | `build.contract` (nível efetivo da glibc) |
| D04 | FORTIFY ligado no ASan | asan-ubsan | `build.contract` |
| D05 | Sem `_GLIBCXX_ASSERTIONS` | dev | `build.contract` |
| D06 | `-fno-stack-protector` | dev | `build.contract` |
| D07 | `-fcf-protection=none` | dev | `build.contract` |
| D08 | Sem PIE na compilação e no link | dev | `build.contract` |
| D09 | `-Wl,-z,lazy` | dev | `build.elf_hardening` |
| D10 | `-Wl,-z,norelro` | dev | `build.elf_hardening` |
| D11 | `-Wl,-z,execstack` | dev | `build.elf_hardening` |
| D12 | Perfil `tsan` sem `-fsanitize=thread` | tsan | `build.contract` |
| D13 | Perfil `debug-stl` sem `_GLIBCXX_DEBUG` | debug-stl | `build.contract` |
| D14 | `CMAKE_CXX_STANDARD 20` | dev | `build.contract` |
| D15 | Perfil `asan-ubsan` sem ASan | asan-ubsan | `build.contract` |
| D16 | Compilação com PIE, link com `-no-pie` | dev | `build.elf_hardening` |

- **Defeitos:** 16 de 16 reprovados.
- **Equivalente:** um `-no-pie` acrescentado ao link sem desligar o PIE do
  CMake. O `-pie` do CMake vem depois na linha de comando e vence, e o binário
  continua PIE (conferido com `readelf -h`).
- **Formas corretas:** 12 de 12 aceitas: os oito presets (dev, asan-ubsan,
  tsan, debug-stl, production, lint, cxx26-probe, dev-qt) e os perfis
  asan-ubsan, tsan, production e debug-stl compilados com o Clang.
- **Veredito:** entram. **Limite:** nenhuma forma correta difícil, porque o
  alvo é um arquivo só. Não cobrem MSVC nem aarch64.

### `gate.launcher`: processo só pelo lançador (2026-11-22)

O `bugprone-unsafe-functions.CustomFunctions` do `core/.clang-tidy`
([processos](../arquitetura/processos-e-confianca.md)). O corpus está em
`core/tests/gates/launcher/`, e o teste `gate.launcher` (preset `lint`) mede a
eficácia a cada build, com o `tidy_corpus.py`: cada linha marcada como defeito
tem de ser acusada, e nenhuma outra.

- **Defeitos:** 40 de 40 reprovados. São 24 funções POSIX e glibc (fork,
  vfork, `_Fork`, a família exec, `posix_spawn`, `pidfd_spawn`, system,
  popen, syscall, clone, forkpty, daemon, wordexp), 5 formas (ponteiro, macro,
  `using`, função passada como argumento, lambda), 5 métodos do `QProcess` e
  6 funções do Windows.
- **Formas corretas:** 16 de 16 aceitas, entre elas as difíceis: função
  própria com o mesmo nome em outro namespace, método `start` de classe
  própria, texto literal, comentário, macro com "system" no nome,
  `posix_spawn_file_actions_init` e `std::chrono::system_clock`.
- **Mutantes do próprio gate**, todos reprovados pelo teste: lista sem
  `popen` (38 de 40), sem o `QProcess` (35 de 40), regex sem âncora (3 formas
  corretas reprovadas), sem a opção (2 de 40, os que a lista padrão da
  checagem já pega) e corpus que não compila (diagnóstico estranho).
- **Limite medido:** a checagem não casa construtor; por isso
  `std::thread` não pode ser barrado por ela.
- **Veredito:** entra.

### `unit.TestFramework`: o GoogleTest no modo da biblioteca padrão (2026-11-22)

- **Defeito:** o GoogleTest compilado sem o perfil no `debug-stl`, e portanto
  sem o `_GLIBCXX_DEBUG`. Reprovado no link (`undefined reference`).
- **A primeira versão não pegou:** ela chamava uma função que só devolvia
  `std::vector`, e o tipo de retorno não entra no nome mangled. Corrigida para
  uma função que recebe `std::vector`.
- **Forma correta:** os oito presets, verdes.
- **Veredito:** entra.

### `gate.detach`: thread nunca solta (2026-11-23)

A mesma opção do clang-tidy, com `std::thread::detach` e
`std::jthread::detach`. O corpus está em `core/tests/gates/threads/`.

- **Defeitos:** 10 de 10 reprovados: `std::thread` e `std::jthread`, por
  ponteiro, por referência, ponteiro para o método, método passado a
  `std::invoke`, dentro de lambda, herdado por classe derivada, dentro de
  macro e em temporário.
- **Formas corretas:** 10 de 10 aceitas, entre elas `detach` de classe
  própria, função própria chamada `detach`, `join`, `request_stop`, texto,
  comentário e variável chamada `detach`.
- **Mutantes do próprio gate**, todos reprovados pelo teste: sem o `jthread`
  (9 de 10), regex de qualquer classe (2 formas corretas reprovadas) e sem a
  entrada (0 de 10).
- **Limite:** medido só com a libstdc++.
- **Veredito:** entra.

### `-Wbidi-chars=any`: Trojan Source (2026-11-23)

- **Defeito:** um par de caracteres bidirecionais (U+202E e U+202C) num
  comentário. O padrão do GCC não acusa o par fechado; com `=any`, acusa. Numa
  cópia do core, o defeito plantado num teste reprovou o `dev`.
- **O lado do Clang não pega o par:** o compilador não acusa nada, e o
  `misc-misleading-bidirectional` do clang-tidy só acusa o par aberto, que o
  GCC já pega. Por não acrescentar nada à CI, que sempre roda o GCC, ele não
  foi ligado.
- **Forma correta:** os oito presets, verdes.
- **Veredito:** entra, só no GCC.

### `alpha.clone.CloneChecker` do Clang: duplicação (2026-11-23)

Candidato ao gate de duplicação (P20), medido com o Clang 23.1.5 num corpus
de quatro funções em dois arquivos.

- **Pega** a cópia com os nomes trocados no mesmo `.cpp`.
- **Não pega** a mesma cópia em outro `.cpp`: a análise é por unidade de
  tradução.
- **Acusa** o laço de uma função de lógica diferente (contar `\t` em vez de
  `\n`), porque ignora os literais. Pode ser duplicação real (é um
  `std::ranges::count`); acusar ou não é decisão de política.
- É checagem `alpha`, instável, do analisador estático.
- O AST em JSON do Clang também não serve de base para um detector próprio:
  144 MiB, 1,9 s e 410 MB de memória para um arquivo de 30 linhas com dois
  cabeçalhos da biblioteca padrão.
- **Veredito:** não entra.

## A CI

`.github/workflows/core.yml`, que roda os presets do `core/CMakePresets.json`
no runner `ubuntu-26.04` do GitHub, sem contêiner, a cada push e pull request
que toca `core/` ou o próprio workflow.

| Jobs | Compilador | Para quê |
| --- | --- | --- |
| `dev` | g++-14 e clang++-20 | O piso declarado |
| `dev`, `production`, `asan-ubsan`, `tsan`, `debug-stl`, `cxx26-probe` | g++-15 | Todos os perfis, no GCC mais novo do runner |
| `lint` (com o clang-tidy 22), `asan-ubsan`, `tsan` | clang++-22 | O segundo compilador e a análise estática |

- **Segurança do workflow:** token só de leitura, a action do checkout fixada
  por SHA (v6.1.0), credencial não persistida, prazo por job e cancelamento
  da rodada obsoleta. A matriz entra nos scripts por variável de ambiente, e
  nunca colada no texto com `${{ }}`: o `shellcheck` lê os passos `run:` de
  verdade (limpo em 2026-11-22), e os passos rodaram nesta máquina no shell do
  runner (`bash --noprofile --norc -eo pipefail`), com e sem clang-tidy.
- **Hipótese não medida:** que o passo que baixa o `vm.mmap_rnd_bits` para
  28 nos jobs de sanitizer seja necessário. Kernel com ASLR de alta entropia
  derrubava os sanitizers de LLVM antigo (actions/runner-images#9491). O passo
  sai se uma rodada sem ele passar.
- **Fora da CI por ora:** o `dev-qt`. O Ubuntu não traz o Qt 6.12, e o
  `core/qt` ainda não tem código.

**Estado (2026-11-23):** verde. A primeira rodada, no push do
`reescrita/core-cpp23` (`7648308d`), passou nos 11 jobs, de 0,3 a 0,7 min cada
(rodada 45098547884, lida pela API pública). O log de cada job só abre com
login de admin; o passo de teste verde prova que houve teste, porque o preset
tem `noTestsAction: error`.

- **Custo:** zero. Runner padrão em repositório público é gratuito e não gasta
  a cota do plano
  ([cobrança do Actions](https://docs.github.com/en/billing/concepts/product-billing/github-actions)).
  O plano Free roda até 20 jobs ao mesmo tempo, e um push abre 12: os 11
  daqui e o `ci.yml` da 0.3, que roda em push de qualquer ramo
  ([limites](https://docs.github.com/en/actions/reference/limits)).
- **Conta bloqueada por cobrança**, como de 2026-10-19 a 2026-11-22: o job
  não começa e a anotação diz "account is locked due to a billing issue". A
  saída é do Viktor, pela cobrança da conta ou pelo suporte do GitHub
  ([conta bloqueada](https://docs.github.com/en/billing/how-tos/troubleshooting/locked-account)).
- **Proteção da `main`** (P18): só os jobs deste workflow. O `ci.yml` da 0.3
  reprovou na primeira rodada dele (2026-11-23, saída 101 do `verificar.sh`).

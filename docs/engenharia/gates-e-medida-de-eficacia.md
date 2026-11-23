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
- **Mecanismo:** a prova recebe `std::vector`; o retorno não entra no nome
  mangled.
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

### `gate.duplication`: cópia de lógica, de T1 a T3 (2026-11-23)

`core/tests/gates/duplication/clones.py`, sobre a libclang: cada função vira
a sequência dos nós da árvore, com local e parâmetro sem nome e o resto pelo
USR. Acusa 44 nós iguais em dois lugares (janela) e função de 40 nós ou mais
com semelhança de 0,85 ou mais (`difflib`). Função gerada por macro não entra.

- **Corpus:** 15 de 15 defeitos (T1; T2 de nomes, literais e tipos; T3 com
  comando a mais, a menos e trocado; no arquivo, entre arquivos e módulos;
  lambda, template, método, bloco em outra função), 14 de 14 formas corretas
  (acessores, tabelas, serializador irmão, mesmo laço com outro algoritmo,
  macro, `std::expected`, código como texto, cabeçalho duas vezes), base limpa.
- **Margem:** semelhança de 0,87 no menor defeito e 0,82 na maior forma
  correta; janela de 45 e 36 nós. Passam a janela de 38 a 50 e o limiar de
  0,83 a 0,86. **Mutantes:** os 8 reprovam (sem renomear local, sem janela,
  sem semelhança, macro entra, cabeçalho contado duas vezes, nome dependente
  vazio, literal sem valor, janela sem USR).
- **GoogleTest 1.17** (41 mil linhas): 32 s, 506 MB, 101 acusações. São 55
  cópias (cinco lidas, o resto pelo nome) e 46 nas 23 sobrecargas de
  `UnpackStructImpl`, repetição que o C++23 impõe.
- **Limites:** ferramenta e corpus do mesmo autor, e o d13 (template) só
  passou depois de ler o nome dependente pelo token. Só a libclang 23 e a
  libstdc++. Sem medida em 85, 2.500 e 20.000 linhas. T4 é indecidível
  (teorema de Rice): fica na P20.
- **Descartados:** o `alpha.clone.CloneChecker` (não vê outro arquivo) e o AST
  em JSON (144 MiB para 30 linhas). Falta medir o PMD CPD.
- **Veredito:** entra no `lint`, medindo o corpus a cada build (6 s). Lê o
  `core/lib` a partir do primeiro módulo (P15).

## A CI

O workflow `CoreCpp 0.1` (`.github/workflows/core.yml`) é a única CI do ramo
`reescrita/core-cpp23`, a frente nova que vai substituir o core da `main`. A
CI da 0.3 saiu do ramo em 2026-11-23 (decisão do Viktor): os testes e gates
dela reprovam aqui por construção e não são critério. A CI da 0.1 se refaz
por partes, cada gate com eficácia medida, como nas seções acima.

Todo push e pull request no ramo roda inteiro, sem filtro de caminho e sem
cancelar o anterior: todo commit tem veredito. A referência é a máquina
local, e o runner `ubuntu-26.04` a imita, sem contêiner; todo passo roda
com `bash -euo pipefail` (o padrão do runner é `bash -e`: medido na rodada
45188723983). Os jobs rodam os presets do `core/CMakePresets.json`:

| Jobs | Compilador | Para quê |
| --- | --- | --- |
| `dev` | g++-14 e clang++-20 | O piso declarado |
| `dev`, `production`, `asan-ubsan`, `tsan`, `debug-stl`, `cxx26-probe` | g++-15 | Todos os perfis, no GCC mais novo do runner |
| `lint` (com o clang-tidy 22 e o `python3-clang-22`), `asan-ubsan`, `tsan` | clang++-22 | O segundo compilador e a análise estática |

- **Segurança:** token só de leitura, checkout v7.0.1 fixado por SHA
  (última release em 2026-11-23, Node 24), credencial não persistida, prazo
  por job, matriz por variável de ambiente (o `shellcheck` lê os passos
  `run:` de verdade).
- **Sem passo de ambiente sem fonte:** a imagem já grava `vm.mmap_rnd_bits=28`
  (actions/runner-images#9491).
- **Máquina local × runner:** a imagem não traz o GCC 16, o Clang 23 nem o
  Qt 6.12 (o `dev-qt` fica fora); trazê-los é dependência nova, que o Viktor
  decide. O CMake 4.3 local lê os presets até a versão 11, e o 4.5 do runner
  até a 13 (o `schema.json` de cada tag do CMake). O piso 3.25, nenhum.
- **Sem mecanismo na CI** desde a saída da 0.3: o idioma dos identificadores
  e a privacidade das docs (P17), e as regras 6 a 9 do README (P19).
- **No GitHub, pelo Viktor** (o token daqui não administra): ações só do
  GitHub e por SHA, token padrão só de leitura, aprovação de todo
  contribuidor externo, ramo sem apagar nem forçar push (P18).

**Estado (2026-11-23):** verde nos 11 jobs, ainda como `Core C++` (rodada
45188723983, `5acb917c`; `gate.duplication` em 19,5 s), sem custo em
repositório público ([cobrança](https://docs.github.com/en/billing/concepts/product-billing/github-actions)).
O nome `CoreCpp 0.1`, o checkout v7.0.1 e o gatilho em todo push não rodaram.

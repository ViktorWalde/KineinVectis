# Perfis de compilação e hardening

Dono: os perfis de build do core, o que cada um liga, as fontes de cada
escolha e como rodar. A lista exata de flags está em `core/cmake/`: este
documento explica, e não repete.

## Como rodar

Da pasta `core/`:

```sh
cmake --workflow --preset dev          # configura, compila e testa
cmake --workflow --preset asan-ubsan
cmake --workflow --preset tsan
cmake --workflow --preset lint         # Clang + clang-tidy
```

Os builds vão para `build/core-<preset>/`, na raiz do repositório, que o git
ignora. O preset `dev-qt` precisa de `KINEIN_QT_PREFIX` apontando para o Qt da
UI (por exemplo `~/Qt/6.12.1/gcc_64`). O clangd lê `build/core-lint/` pelo
`core/.clangd`.

## Os perfis

| Preset | Perfil | Compilador | O que muda | Para quê |
| --- | --- | --- | --- | --- |
| `dev` | dev | GCC | baseline, semântica de produção e `-Werror`; `-O2 -g` | O build do dia a dia: o que se testa é o que se entrega |
| `production` | production | GCC | o mesmo, **sem `-Werror`** | O que se distribui: aviso de um compilador mais novo não quebra o build de quem empacota |
| `asan-ubsan` | asan-ubsan | GCC | ASan e UBSan, sem FORTIFY e sem a semântica de produção; `-O1 -g` | Achar erro de memória e comportamento indefinido |
| `tsan` | tsan | GCC | TSan, sem FORTIFY; `-O1 -g` | Achar corrida de dados |
| `debug-stl` | debug-stl | GCC | `_GLIBCXX_DEBUG`; `-O0 -g3` | Iteradores inválidos e precondições da biblioteca; depuração passo a passo |
| `lint` | dev | Clang | clang-tidy em cada unidade | Análise estática; e o segundo compilador com `-Werror` |
| `cxx26-probe` | dev | GCC | `-std=c++26` | Só detectar incompatibilidade com o C++26 |
| `dev-qt` | dev | GCC | liga `core/qt` | O adaptador Qt |

Versões mínimas, decisão do Viktor de 2026-11-22: **GCC 14 e Clang 20**,
o piso que o CI testa no `ubuntu-26.04`. Abaixo disso o configure falha. O
Clang 18 não serve: até o Clang 18, o `__cpp_concepts` do Clang era 201907L,
e a libstdc++ só declara o `std::expected` com 202002L
(`bits/version.h` da libstdc++ 16, lido; o Clang 19 subiu o valor para
habilitar o `<expected>` da libstdc++, [PR #87998 do LLVM](https://lists.llvm.org/pipermail/cfe-commits/Week-of-Mon-20240408/565418.html)).

O código de terceiros compilado junto (o GoogleTest) recebe o perfil como
`THIRD_PARTY`: otimização, modo da biblioteca padrão, sanitizers e proteções,
sem os avisos e sem o clang-tidy (`core/cmake/KineinProfile.cmake`).

## O que vale em todos os perfis

- **Avisos:** a lista do `KineinStrictOptions.cmake` da 0.3, que já compilava
  a UI com `-Werror` (A10 das [lições](../lessons-from-0.3.md)), mais o TL;DR do
  OpenSSF. A lista é fixa por compilador: flag desconhecida quebra o build em
  vez de sumir (falha fechado).
- **Proteções:** `-fstack-protector-strong`, `-fstack-clash-protection`,
  `-fstrict-flex-arrays=3`; no x86_64, `-fcf-protection=full` e
  `-fzero-call-used-regs=used-gpr`; no aarch64, `-mbranch-protection=standard`.
- **Biblioteca padrão:** `_GLIBCXX_ASSERTIONS` e o modo `FAST` da libc++; no
  `debug-stl`, `_GLIBCXX_DEBUG` e o modo `DEBUG` da libc++.
- **Link do executável e dos testes:** `-z nodlopen`, `-z noexecstack`,
  `-z relro`, `-z now`, `--as-needed`, `--no-copy-dt-needed-entries` e PIE.

## As escolhas e as fontes

| Escolha | Fonte |
| --- | --- |
| O baseline de flags | OpenSSF, *Compiler Options Hardening Guide for C and C++*, versão de 2026-08-20, TL;DR; linhas condicionais "for x86_64", "for aarch64" e "for production code" |
| `-Werror` só em desenvolvimento, nunca no fonte distribuído; `-Werror=format-security` em todos | O mesmo guia, TL;DR |
| `_FORTIFY_SOURCE` só com `-O1` ou mais, precedido de `-U_FORTIFY_SOURCE` | O mesmo guia, Tabela 2 |
| Sem `_FORTIFY_SOURCE` nos sanitizers | FAQ do AddressSanitizer: "ASan (and other sanitizers) doesn't support source fortification" |
| TSan em build separado | Medido: o GCC 16.2.1 recusa `-fsanitize=thread` junto de `-fsanitize=address` |
| `_GLIBCXX_DEBUG` só no `debug-stl`, com tudo compilado junto | Manual da libstdc++ (Debug Mode): a flag muda o tamanho dos containers, e o código só se liga com o de fora do modo se nenhum container passar entre os dois |
| A semântica de produção também no `dev` | Decisão deste documento: o build do dia a dia roda o que se entrega. Os sanitizers ficam sem ela, para o UBSan ver o UB que ela define |
| O `-z nodlopen` não marca executável | Medido em 2026-11-22: o GNU ld 2.47 marca `NOOPEN` em biblioteca compartilhada e não em executável PIE. A flag fica, como o guia pede, e a conferência vale para biblioteca compartilhada quando houver uma |
| `CMAKE_BUILD_TYPE` não muda flag nenhuma | As flags padrão do CMake por configuração são zeradas no escopo do core: o perfil é o único dono |

## Conferido pelo próprio build

- `build.contract` (`core/tests/build/build_contract_test.cpp`): o compilador
  confere o padrão da linguagem, a ausência de extensões, PIE, SSP, CET,
  `_GLIBCXX_ASSERTIONS`, o FORTIFY de nível 3 efetivo na glibc, a ausência de
  FORTIFY nos sanitizers e o sanitizer certo em cada perfil.
- `build.elf_hardening` (`core/cmake/CheckElfHardening.cmake`): o `readelf`
  confere no binário da rodada `BIND_NOW`, `PIE`, `GNU_RELRO` e a pilha não
  executável.
- A eficácia dos dois, medida em 2026-11-22, está em
  [gates-and-effectiveness.md](gates-and-effectiveness.md#registro).

## NÃO VERIFICADO

- MSVC e Windows: o ramo MSVC dos módulos foi herdado do
  `KineinStrictOptions.cmake` e não compilou nesta máquina.
- GCC 14 e Clang 20: o piso compilou e passou no CI em 2026-11-23
  ([gates](gates-and-effectiveness.md#a-ci)), mas a versão exata que o
  runner usou está no log, que não foi lido. Nesta máquina há só o GCC 16.2.1
  e o Clang 23.1.5.
- O ruído do `-Wconversion` no código gerado pelo moc: `core/qt` ainda não tem
  código. Os cabeçalhos do Qt já entram como `-isystem`.
- aarch64.

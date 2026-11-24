# Erros conhecidos de C++ e as defesas

Dono: os defeitos clássicos da linguagem e a defesa contra cada um, por
camada. A ideia é que nenhum desses defeitos dependa só de atenção: cada um
tem pelo menos uma ferramenta que o pega.

Camadas: **C** compilador (aviso vira erro nos presets de desenvolvimento),
**T** clang-tidy (preset `lint`), **S** sanitizer em tempo de execução
(`asan-ubsan`, `tsan`), **B** verificação da biblioteca padrão
(`_GLIBCXX_ASSERTIONS`; `_GLIBCXX_DEBUG` no `debug-stl`), **P** endurecimento
do binário entregue, **R** regra de projeto
([cpp23-rules.md](cpp23-rules.md)).

Os nomes das checagens foram conferidos no clang-tidy 23.1.5 e as flags no
GCC 16.2.1 e no Clang 23.1.5, em 2026-11-22.

## Memória e vida dos objetos

| Defeito | Defesa |
| --- | --- |
| Uso depois de liberar, liberação dupla, vazamento | **S** ASan e LSan; **T** `clang-analyzer-cplusplus.NewDelete`, `cppcoreguidelines-owning-memory`, `cppcoreguidelines-no-malloc`; **R** RAII |
| Referência ou view pendurada (`string_view`, `span`, referência para temporário) | **C** `-Wdangling-reference` (GCC), `-Wdangling` (Clang); **T** `bugprone-dangling-handle`, `bugprone-return-const-ref-from-parameter`; **S** ASan `detect_stack_use_after_return`; **R** view só como parâmetro |
| Lambda que captura por referência e sobrevive ao escopo (trabalho em outra thread) | **T** `bugprone-capturing-this-in-member-variable`, `cppcoreguidelines-avoid-capturing-lambda-coroutines`; **S** ASan e TSan; **R** trabalho recebe cópia imutável |
| Uso depois de `std::move` | **T** `bugprone-use-after-move`, `clang-analyzer-cplusplus.Move` |
| Acesso fora dos limites | **B** `_GLIBCXX_ASSERTIONS` confere `operator[]` dos containers; **S** ASan; **C** `-fstrict-flex-arrays=3`; **T** `cppcoreguidelines-pro-bounds-pointer-arithmetic`, `cppcoreguidelines-pro-bounds-constant-array-index` |
| Variável lida sem inicialização | **C** `-Wuninitialized` e `-Wmaybe-uninitialized` (no perfil `dev`, com `-O2`); **T** `cppcoreguidelines-init-variables`, `cppcoreguidelines-pro-type-member-init`; **P** `-ftrivial-auto-var-init=zero` |
| Fatiamento de objeto | **T** `cppcoreguidelines-slicing` |
| Classe base sem destrutor virtual | **C** `-Wnon-virtual-dtor`; **T** `cppcoreguidelines-virtual-class-destructor` |
| Cópia rasa por operação especial faltando | **T** `cppcoreguidelines-special-member-functions`; **C** `-Wdeprecated` |
| Ordem de inicialização estática entre unidades | **C** `-Wglobal-constructors`, `-Wexit-time-destructors` (Clang, em `core/lib`); **T** `cppcoreguidelines-avoid-non-const-global-variables`, `cert-err58-cpp`; **S** ASan `check_initialization_order` |
| Acesso a `std::optional` vazio | **T** `bugprone-unchecked-optional-access`, `bugprone-optional-value-conversion` |

## Aritmética e tipos

| Defeito | Defesa |
| --- | --- |
| Conversão que perde valor ou troca o sinal | **C** `-Wconversion`, `-Wsign-conversion`, `-Wdouble-promotion`; **T** `bugprone-narrowing-conversions` |
| Estouro de inteiro com sinal (comportamento indefinido) | **S** UBSan, que reprova o teste (`-fno-sanitize-recover=all`); **P** `-fno-strict-overflow` torna o estouro definido no que se entrega |
| Multiplicação estreita alargada depois | **T** `bugprone-implicit-widening-of-multiplication-result` |
| Divisão inteira onde se queria real | **T** `bugprone-integer-division` |
| Laço com variável menor que o limite | **T** `bugprone-too-small-loop-variable` |
| Conversão estilo C e `reinterpret_cast` | **C** `-Wold-style-cast`, `-Wuseless-cast` (GCC); **T** `cppcoreguidelines-pro-type-cstyle-cast`, `cppcoreguidelines-pro-type-reinterpret-cast` |
| Aliasing estrito violado | **P** `-fno-strict-aliasing`; **T** `cppcoreguidelines-pro-type-reinterpret-cast` |
| Ponteiro nulo removido pelo otimizador depois de uma desreferência | **P** `-fno-delete-null-pointer-checks`; **C** `-Wnull-dereference`; **T** `clang-analyzer-core.NullDereference` |

## Controle de fluxo e formato

| Defeito | Defesa |
| --- | --- |
| `case` que cai no seguinte sem querer | **C** `-Wimplicit-fallthrough=5` (GCC: só aceita `[[fallthrough]]`), `-Wimplicit-fallthrough` |
| `switch` de enum sem um valor | **C** `-Wswitch-enum` |
| Formato de `printf` com argumento errado ou vindo de fora | **C** `-Wformat=2`, `-Werror=format-security`; **T** `cppcoreguidelines-pro-type-vararg`; **R** `std::format` e `std::print`, conferidos em compilação |
| Resultado de `std::expected` descartado | **T** `bugprone-unused-return-value` com `std::expected` |
| Laço infinito por condição que não muda | **T** `bugprone-infinite-loop` |
| Sombra de variável | **C** `-Wshadow` |
| Caractere bidirecional do Unicode que muda o que o código parece dizer (Trojan Source, CVE-2021-42574) | **C** `-Wbidi-chars=any` (GCC), que pega também o par fechado; o padrão do GCC e o clang-tidy só pegam o par aberto (medido) |

## Concorrência

| Defeito | Defesa |
| --- | --- |
| Corrida de dados | **S** TSan (build separado: o compilador recusa TSan junto de ASan); **R** um dono por dado |
| Função da biblioteca C que não é segura em thread | **T** `concurrency-mt-unsafe` |
| Espera em variável de condição sem predicado (despertar espúrio) | **T** `bugprone-spuriously-wake-up-functions` |
| Bloqueio dentro de seção crítica | **T** `clang-analyzer-unix.BlockInCriticalSection` |
| Exceção escapando de thread ou de slot | **T** `bugprone-exception-escape`; **R** `noexcept` no corpo de thread e no slot |
| Thread esquecida sem `join` | **R** `std::jthread`; sem checagem automática para `std::thread` sem `join` (o clang-tidy não casa construtor) |
| Thread solta com `detach()` | **T** `bugprone-unsafe-functions.CustomFunctions`, medido pelo `gate.detach` |
| Travas pedidas em ordens diferentes (deadlock em potencial) | **S** TSan, que reporta `lock-order-inversion` |
| Dado acessado sem a trava que o protege | **C** `-Wthread-safety` (Clang), que só age com as anotações da análise de segurança de threads. Usar ou não é decisão do desenho |

## O binário entregue

| Ameaça | Defesa (**P**) | Conferido por |
| --- | --- | --- |
| Estouro de pilha e de buffer | `-fstack-protector-strong`, `-fstack-clash-protection`, `_FORTIFY_SOURCE=3` | `build.contract` |
| Desvio de fluxo (ROP e JOP) | `-fcf-protection=full` no x86_64, `-mbranch-protection=standard` no aarch64 | `build.contract` (`__CET__`) |
| Tabela de realocação gravável | `-Wl,-z,relro -Wl,-z,now` | `build.elf_hardening` |
| Pilha executável | `-Wl,-z,noexecstack` | `build.elf_hardening` |
| Endereço fixo | `-fPIE -pie` | `build.contract` e `build.elf_hardening` |

## Sem defesa automática ainda

- Thread esquecida, captura por referência em trabalho assíncrono e a
  mistura de árvore do Qt com RAII dependem de revisão. Candidatas: o
  `-Wunsafe-buffer-usage` do Clang e uma checagem própria, cada uma só depois
  da medida de eficácia ([gates](gates-and-effectiveness.md)).
- O MemorySanitizer (leitura não inicializada em tempo de execução) só existe
  no Clang e exige a biblioteca padrão instrumentada. Fica fora.

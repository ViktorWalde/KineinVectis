# Estratégia de testes

Dono: as camadas de teste do core, as regras de todo teste, o teste
diferencial contra o core em Rust e a medida de desempenho.

## Regras de todo teste

| Regra | Origem |
| --- | --- |
| A prova usa a ferramenta real; ferramenta ausente é NÃO PROVADO, nunca aprovado | R-TEST-01 do 63 |
| Teste não sincroniza por tempo fixo: espera um evento, ou a ferramenta falsa tem trava controlada pelo teste | R-TEST-03 do 63; L16 |
| Prazo é parâmetro do teste, nunca constante escondida no código de produção | INF-06 e INF-30 da 0.3 |
| Teste não lê nem grava o diretório pessoal: `HOME` e `XDG_*` apontam para pasta temporária | R-TEST-04 do 63 |
| Refatoração não edita os testes que a provam | R-TEST-07 do 63 |
| Toda guarda nova tem um mutante que a desliga e precisa reprovar | R-TEST-02 do 63 |
| O teste confere o artefato da própria rodada, nunca um binário velho | L17 (INF-31) |
| A suíte passa nos perfis `dev`, `asan-ubsan` e `tsan` | RNF-QUA-03 |

## Camadas

| Camada | O que prova | Onde | Estado |
| --- | --- | --- | --- |
| Contrato do build | As flags prometidas chegaram ao compilador e ao binário | `core/tests/build/` | **Existe**: 2 testes |
| Unidade | Regra e algoritmo de `core/lib`, sem laço de eventos e sem processo | `core/tests/unit/` | Framework ligado; o primeiro teste do core nasce com o primeiro código |
| Eficácia dos gates | Cada gate próprio reprova os defeitos plantados e aceita as formas corretas do corpus dele | `core/tests/gates/` | **Existe**: `gate.launcher` e `gate.detach`, no preset `lint` |
| Complexidade | Nenhum caminho cresce mais que linearmente (RNF-PERF-01) | `core/tests/complexity/` | A criar |
| Diferencial | O core em C++ responde como o core em Rust | `core/tests/diff/` | Pasta criada, vazia |
| Ordem e concorrência | Intercalações forçadas: geração velha descartada, cancelamento, mesmo recurso em série | `core/tests/unit/` | A criar |
| Queda | `kill -9` em cada processo, com a perda medida (RNF-DUR-01) e nenhum órfão | `core/tests/crash/` | A criar |
| Projeto hostil | Zero marcas sem confiança (RNF-SEG-01) | `core/tests/hostile/` | A criar |

## Diferencial

- **O oráculo** é o binário do core em Rust, pelo fio JSON-RPC, como a UI o
  usa. Os 622 testes IPC da 0.3 chamam `handle_request` dentro do processo
  (64:310) e não servem para comparar dois processos.
- **A comparação** acontece depois de normalizar ids, caminhos absolutos,
  horários, ids de job e a ordem dos eventos que não têm ordem garantida.
- **As divergências intencionais** ficam numa lista versionada junto dos
  testes, cada uma com o defeito do oráculo que ela evita. Sem essa lista, o
  teste premia quem reproduz o defeito. Entram de início:
  - INF-23: o oráculo executa código do projeto sem confiança;
  - INF-22: carga antiga do contexto por cima da nova;
  - INF-29: falha de `exec` com aborto e core dump;
  - INF-30: prazo vencido reportado como obsoleto;
  - ENG-56: o oráculo grava estado da pessoa no `.kinein`.
- **O critério de troca** (ADR-0012): cada método do N0 e do N1 passa no
  diferencial, salvo a lista.

## Complexidade

Cada caminho que depende do tamanho do arquivo ou do projeto roda nos quatro
tamanhos de referência (85, 2.500, 20.000 e 40.000 linhas) e reprova se a
razão de tempo entre dois tamanhos passar do limite do RNF-PERF-01. A medida é
o tempo de CPU do processo, não o de parede, para não depender da carga da
máquina.

## Desempenho

- **Antes e depois**, sempre com o mesmo arquivo, o mesmo método e o mesmo
  perfil (`production`).
- **As fixtures C/C++ não existem.** As linhas de base da 0.3 usaram arquivos
  Rust da geradora do `medir-core.py` (65:670-673). O primeiro passo é gerar
  fixtures C/C++ nos quatro tamanhos e medir nelas o produto atual (UI e core
  em Rust), para haver base de comparação.
- **O método do editor** é o do 65 §20.3: tecla→quadro, abertura, CPU ao
  digitar e memória, com as rodadas alternadas.

## Framework de teste

GoogleTest 1.17.0, decisão do Viktor de 2026-11-22. O módulo é
`core/cmake/KineinTesting.cmake`, e um teste de unidade nasce com
`kinein_add_unit_test(<nome> SOURCES ... LIBRARIES ...)`. Cada `TEST` vira um
teste do CTest com o prefixo `unit.`.

- **Compilado com o perfil, não o do sistema.** No `debug-stl`, o
  `_GLIBCXX_DEBUG` muda o ABI dos containers, e a biblioteca do framework
  precisa estar no mesmo modo. O `unit.TestFramework.*` confere isso no link:
  ele chama uma função da biblioteca que recebe `std::vector` como parâmetro.
  O tipo de parâmetro entra no nome mangled; o de retorno, não, e a primeira
  versão do teste, com uma função que só devolvia `std::vector`, não pegou o
  mutante (medido em 2026-11-22). Nos sanitizers, o framework também fica
  instrumentado.
- **O tarball tem hash fixado** e é baixado no configure. Sem rede:
  `-DFETCHCONTENT_SOURCE_DIR_GOOGLETEST=<pasta>`.
- O teste de contrato do build não usa framework: é um executável que
  compila ou não.

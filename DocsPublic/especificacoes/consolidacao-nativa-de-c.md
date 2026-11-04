# Consolidação nativa de C na base 0.4

> **Classe: DECISÃO / ALVO, 2026-11-03.** Etapa profunda e obrigatória antes
> da consolidação C++. Define os aceites de C na base pública 0.4.0; a primeira
> jornada C11 é uma prova parcial. Estado observado em §7; fronteira da release
> no [62](../roadmaps/62-base-publica-0.4.md).

## 1. Finalidade e significado de integração nativa

C é uma linguagem fundamental do desenvolvimento embarcado. Deve ter uma
jornada própria e contínua de criar/abrir, configurar, editar, analisar,
compilar, ligar, executar, testar e depurar, com recuperação de falhas.
Os fluxos visuais precisam permitir uso diário em projetos pequenos e médios,
incluindo projetos existentes e bibliotecas, com contexto explícito por alvo.

Integração nativa significa que esses fluxos pertencem aos modelos, comandos,
Jobs e superfícies usuais da IDE. GCC/Clang, clangd, CMake/Make, analisadores,
formatadores e depuradores continuam executando suas funções. A 0.4 aproveita
e fortalece o que já foi construído; os donos compartilhados C/C++ são estendidos
para respeitar as particularidades de C. Cada capacidade exige inventário,
integração e prova de uso; presença de um executável no catálogo é só uma entrada.

Esta etapa integra B2/B6/B7 do 62. As lacunas essenciais abaixo permanecem na
0.4.0. Versões públicas 0.4.1 em diante aprofundam e estabilizam a base aceita.
O verificador existente acompanha as fatias; sua fase transversal dedicada
permanece posterior à orquestração das ferramentas, ainda dentro da 0.4.

## 2. C moderno e compatibilidade

**C23 é o padrão moderno de referência desta etapa**, publicado como
ISO/IEC 9899:2024 segundo o [WG14](https://www.open-std.org/jtc1/sc22/wg14/www/projects).
C17 e C11 têm regressões obrigatórias; preservar C99/C90 e dialetos GNU quando
exigidos por projetos e SDKs. A escolha pertence ao projeto/alvo e requer
prévia para qualquer alteração. Projetos existentes conservam suas opções.
C2y permanece experimental, com capacidade e alcance identificados.

| Dimensão | Requisito de integração |
| --- | --- |
| Compilador e dialeto | Identificar driver C, versão, alvo e modo efetivo: ISO ou extensões, incluindo `c23`/`gnu23` quando suportados |
| Padrão solicitado e efetivo | Mostrar a origem no projeto, toolchain ou comando; comunicar divergência e recusar redução silenciosa nas configurações geradas pela IDE |
| Capacidade da ferramenta | Conferir recursos por versão/alvo; opção aceita pelo driver, suporte de sintaxe e disponibilidade de biblioteca são evidências distintas |
| CMake | Configurar `C_STANDARD`, `C_STANDARD_REQUIRED` e `C_EXTENSIONS` no escopo correto, preservando propriedades existentes e opções por alvo |
| Editor e análise | Tree-sitter C, clangd, diagnósticos, formatador e verificador precisam aceitar o corpus moderno no contexto observado |
| Biblioteca e runtime | Identificar libc, headers, biblioteca de suporte e ambiente hosted/freestanding; explicar recursos indisponíveis no alvo |

O suporte por recurso está nas tabelas oficiais de
[GCC](https://gcc.gnu.org/projects/c-status.html) e
[Clang](https://clang.llvm.org/c_status.html). A IDE deve comunicar limitações
da toolchain escolhida e conservar a evidência produzida por ela. Isso exige
tratar também libc e alvo: selecionar `-std=c23` sozinho não demonstra todo
o ecossistema C23. No CMake, o comportamento de redução de padrão é controlado
por [`C_STANDARD_REQUIRED`](https://cmake.org/cmake/help/latest/prop_tgt/C_STANDARD.html).

O corpus moderno inclui `bool`/`true`/`false`, `nullptr`, `static_assert`,
`typeof`/`typeof_unqual`, inferência com `auto`, `constexpr`, atributos,
literais binários/separadores e `_BitInt`, conforme as capacidades das ferramentas.
Também exercita biblioteca C23, por exemplo `<stdckdint.h>` e `<stdbit.h>`,
quando fornecida pela libc/implementação. Registrar cada recurso aceito,
recusado ou indisponível. Falha conhecida de fornecedor exige delimitação e
diagnóstico; lacuna do consumidor da IDE exige correção no seu dono.

## 3. Aceites obrigatórios da etapa C

| Aceite | Entrega e prova exigidas |
| --- | --- |
| C1 — Projeto e ambiente | Criar projeto apenas C pela UI e abrir projeto C existente; escolher toolchain/padrão e configuração com prévia; detectar ausência/incompatibilidade e recuperar sem perder arquivos |
| C2 — Build e contexto | GCC e Clang reais em host; CMake e Make com CDB obtida pelo fluxo existente, dependências, várias unidades, biblioteca estática/dinâmica e executável; flags, compilador, diretório, macros, includes e gerados coerentes em build/editor/análise |
| C3 — Linguagem e editor | C11/C17/C23, header C e header compartilhado com C++, navegação, completion, hover, ajuda de parâmetros, rename, correções e formatação de arquivo/seleção; languageId coerente no LSP e gramática C onde aplicável |
| C4 — Análise e memória | Avisos configuráveis, clang-tidy/analisador estático pertinente e ASan/UBSan em alvo de host compatível, com diagnósticos navegáveis, execução controlada e caso defeituoso que a ferramenta realmente detecta |
| C5 — Executar, testar e depurar | Artefato/configuração corretos, CTest/testes do projeto, GDB/DAP com breakpoints, steps, pilha, variáveis, expressão e memória; falha, cancelamento e reinício exercitados pela UI |
| C6 — Embarcados | Contexto freestanding ARM e RISC-V, toolchain, sysroot/libc, headers de SDK, startup/linker script e ELF identificados; build, relatório Flash/RAM e ciclo QEMU/DAP dos mínimos F1–F5, com regressões de SDKs já oferecidos |
| C7 — Bibliotecas e fronteiras | API pública C, visibilidade de símbolos, linkage, tipos/layout/alinhamento, includes transitivos e consumo de dependência; biblioteca C preparada e testada para consumo posterior em C++/Rust |
| C8 — Robustez e plataformas | Projetos pequenos/médios, novos/existentes, configurações distintas, caminhos com espaços/Unicode, reabertura, troca de workspace, ferramenta ausente, falha/cancelamento e recuperação; matriz Linux e Windows nativo, integridade e latência/recursos medidos |

As funções do editor dependem das capacidades negociadas do servidor; a UI
explica uma função indisponível e conserva as operações independentes. A
prontidão de C exige integrar e provar essas funções na toolchain de referência.
Drivers, versões, bibliotecas e alvos suportados ficam explícitos na matriz.

Pré-processamento tem corpus próprio: macro condicional, header privado/público,
header gerado, mudanças de include/define, unidades que compartilham header
com configurações diferentes e associação de `.h` ao contexto de compilação.
Invalidação deve alcançar diagnóstico, contexto e prévia de correção dependentes.
Editar um header C precisa conservar a linguagem e a unidade de referência.

A análise de memória inclui duração de objetos, ponteiros, alinhamento,
aliasing, conversões e comportamento indefinido. Casos positivos e negativos
usam ferramentas reais e explicam cobertura. Sanitizers de host e análise
estática não comprovam execução no microcontrolador. Configuração incompatível
gera motivo explícito; opções de instrumentação respeitam alvo e toolchain.

C6 conserva os aceites da [infraestrutura fundamental](infraestrutura-fundamental-0.4.md)
e da trilha [52](../roadmaps/52-arquitetura-executavel-da-0.4.md). Emulação requer
máquina/firmware compatíveis. Provas físicas permanecem separadas e vinculadas
à placa/sonda efetivamente exercitada. Projetos de host não aceitam o recorte
freestanding e uma máquina emulada não representa todos os MCUs de uma arquitetura.

O produtor/API C é aceito nesta etapa. O consumo completo C++/Rust e a
invalidação bilateral pertencem às etapas seguintes de integração, usando
essa mesma biblioteca; essa dependência mantém uma sequência executável.

## 4. Sequência de execução

1. **Inventário e contexto moderno:** mapear o que existe, reusar a jornada
   C11, medir C23 nas toolchains presentes e localizar lacunas de criação,
   padrão, headers, LSP e análise. Distinguir implementado, integrado e provado.
2. **Projeto e configuração:** completar o fluxo C próprio e a seleção de
   padrão; provar GCC/Clang e coerência de contexto nos fluxos CMake/Make.
3. **Editor C e ferramentas de qualidade:** consolidar C23, headers e
   pré-processamento, ações LSP, formatação, diagnósticos e análise de memória.
4. **Ciclo diário de host:** bibliotecas/dependências, build/ligação, executar,
   testes e depuração; provas visuais e recuperação na mesma jornada.
5. **C embarcado e portabilidade:** ARM/RISC-V e mínimos F1–F5, Linux/Windows
   e consumidores VFS; conservar a mesma identidade de projeto/configuração.
6. **Fechamento C:** repetir matriz positiva/negativa, mutações das guardas,
   medições e gates; registrar cobertura e lacunas. Aceitar a etapa somente
   após os requisitos obrigatórios, então aprofundar C++ e seguir para Rust.

As fatias corrigem falhas confirmadas nos donos existentes. Enquanto se
constrói outro ambiente, o endurecimento do host pode avançar; o ambiente
pendente conserva seu aceite aberto. Regressões das outras linguagens e dos
recursos embarcados oferecidos continuam necessárias durante a sequência.

## 5. Corpus e registro de prontidão

O conjunto deve conter a fixture C11 atual, corpus C17/C23, projeto Make,
projeto com biblioteca compartilhada/dependência e projeto C de porte médio
com licença/origem e revisão identificadas. Cargas ARM/RISC-V têm contexto e
artefatos próprios. Preferir ampliar fixtures e runners existentes, limitando
recursos e conservando os exemplos úteis para reprodução externa.

Para cada caso, registrar ferramenta/versão, plataforma/alvo, dialeto efetivo,
libc, configuração, entradas, operação pelo produtor real, resultado observado,
limite e duração. Prova em processo, prova por IPC e gesto na UI conservam
seus alcances. Sugestão LSP simulada permite uma regressão controlada; a
integração real do editor exige servidor real e interação na UI.

Falhas introduzidas incluem padrão incompatível, include/define divergente,
header alterado, símbolo ausente, diagnóstico obsoleto, resposta fora de ordem,
ferramenta ausente, timeout e cancelamento. Conferir preservação dos originais,
aplicação explícita, recuperação e nova execução bem-sucedida.

O fechamento exige evidência por C1–C8, regressões e gates pertinentes, com
prova na UI para os fluxos visuais. Uma pendência obrigatória permanece aberta
mesmo quando a ferramenta/plataforma necessária está indisponível. O algoritmo
transversal mantém depois sua fase dedicada e seus próprios aceites; encerrar
C não equivale ao fechamento da vertical inteira da 0.4.

## 6. Donos existentes a fortalecer

| Responsabilidade | Dono de implementação ou contrato |
| --- | --- |
| Criação e configuração de projeto | `workspace/create.rs`, `configaction/` e domínio IPC `workspace`/`configAction` |
| Toolchain, CDB e contexto efetivo | `toolchain/`, `cmake/`, `build/`, `cdb/` e `index/context/` |
| Gramática C, consultas e sincronização LSP | `lang/`, `lsp/` e `handlers/lsp/` |
| Diagnósticos, análise e formatação | `build/syntax_check.rs`, `build/tidy.rs`, `format.rs` e consumidores atuais |
| Testes, execução e depuração | `test/`, `runconfig.rs`, `dap/` e seus handlers/Jobs |
| Verificador complementar | `build/edit_verification/` e prévia/aplicação LSP existentes |
| Fluxos visuais | Wizard, editor, configuração e docks de build/test/debug atuais, pela ponte C++ |

Esta tabela indica responsabilidades; localizar arquivos pelo
[mapa de módulos](../arquitetura/01-mapa-de-modulos.md) antes da mudança.
Alteração de schema segue o contrato IPC primeiro. Correção interna que
preserva o contrato conserva a versão do protocolo.

## 7. Estado inicial e lacunas observadas — 2026-11-03

**Etapa C aberta.** A jornada C11 de host tem prova parcial no
[40.7 §7.376](../roadmaps/40.7-registro-das-entregas.md#7376--jornada-c-de-host-e-sincronização-do-contexto-após-build-2026-11-03):
biblioteca/executável, contexto após build, prévia/verificação/aplicação,
CTest, clangd e breakpoint GDB/DAP. Tem alcance Linux/host e provas por
despacho/IPC; não constitui o aceite C1–C8 nem a prova visual completa.

| Inspeção do código nesta data | Consequência para a etapa |
| --- | --- |
| `WorkspaceProjectTemplate`/`workspace/create.rs` oferecem template C++ CMake, Rust, Python e pasta vazia | Projeto C próprio pela UI permanece pendente em C1 |
| `configaction/rigor.rs` e catálogo CMake oferecem `CMAKE_CXX_STANDARD`, sem ação equivalente de padrão C | Seleção visual de padrão C, incluindo C23, permanece pendente em C1/C2 |
| `lang/registry.rs` possui gramática C para `.c`, mas associa `.h` a C++ por extensão | Contexto de headers C e corpus moderno precisam de consolidação em C3 |
| `lsp/registry.rs` compartilha clangd e `languageId` `cpp` para `.c`/headers | Preservar o servidor compartilhado e corrigir/provar a identidade documental C no wire em C3 |
| CMake, CDB/contexto, análise ao salvar, formatador, CTest, DAP e verificador já têm donos | Ampliar e provar os mecanismos existentes; a presença deles não encerra as jornadas |

As lacunas de template/padrão já constavam da
[auditoria de C puro no 52 §14.14](../roadmaps/52-arquitetura-executavel-da-0.4.md#1414-auditoria-de-c-puro-antes-do-fechamento--2026-10-20).
Esta decisão consolida sua execução na etapa C, com C23 e aceites profundos.
O inventário não declara novas capacidades implementadas nem certificação
de compiladores, bibliotecas, plataformas ou da conformidade integral ao padrão.

**Auditoria direta das toolchains, nesta data:** GCC 16.2.1 e Clang 23.1.4
em Linux/host compilaram e executaram sete pequenos casos cada, com
`-std=c23 -pedantic-errors -Wall -Wextra -Werror`: palavras-chave/assertion,
`typeof`/`typeof_unqual`, `auto`/`constexpr`, atributo/literal, `_BitInt`,
inteiros com overflow conferido por `<stdckdint.h>` e contagem de bits com
`<stdbit.h>`. As 14 execuções terminaram com código zero. Isso demonstra
capacidade desses recursos nas ferramentas/biblioteca presentes. A integração
desses mesmos casos no editor, contexto, verificador e UI continua pendente;
a auditoria direta não aceita C23 na IDE nem a matriz C1–C8.

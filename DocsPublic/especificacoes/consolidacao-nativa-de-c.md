# Consolidação nativa de C na base 0.4

> **Classe: DECISÃO / ALVO, 2026-11-03.** Etapa profunda e obrigatória antes
> da consolidação C++. Define os aceites de C na base pública 0.4.0; a primeira
> jornada C11 é uma prova parcial. Inventário inicial em §7 e evolução em §§8–10;
> fronteira da release
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

Esta etapa integra B2/B6 do 62; desde 2026-11-08 o Windows (B7) entra só na
validação básica em VM, perto do fim da 0.4, e o porte completo é colaboração. Conforme a ordem do autor (62 §3), a depuração de C5 e o
ciclo QEMU de C6 fecham na etapa de DAP e QEMU, depois de Python, e os
problemas C-03–C-05 do inventário (CDB 1:N, diretório de build por preset e
localização da CDB) entram nesta etapa. As lacunas essenciais abaixo permanecem na
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

**Reconhecimento aberto de dialetos:** a identidade C não depende de uma lista
de revisões na IDE. Projetos antigos, aliases ISO/GNU, prévias e valores futuros
conservam o padrão e os argumentos observados. A ausência de `-std` mantém o
padrão como não informado; não se inventa o default da toolchain. Reconhecer
o contexto não certifica conformidade, suporte de sintaxe ou biblioteca: essas
capacidades pertencem ao compilador, clangd e demais ferramentas efetivos.
As opções sugeridas pela IDE são exemplos, sem restringir a leitura do projeto.

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
| C8 — Robustez e plataformas | Projetos pequenos/médios, novos/existentes, configurações distintas, caminhos com espaços/Unicode, reabertura, troca de workspace, ferramenta ausente, falha/cancelamento e recuperação; matriz Linux (Windows nativo só no recorte básico em VM desde 2026-11-08), integridade e latência/recursos medidos |

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
5. **C embarcado e portabilidade:** ARM/RISC-V e mínimos F1–F5, Linux e
   consumidores VFS (Windows no recorte básico em VM); conservar a mesma
   identidade de projeto/configuração.
6. **Fechamento C:** repetir matriz positiva/negativa, mutações das guardas,
   medições e gates; registrar cobertura e lacunas. Aceitar a etapa somente
   após os requisitos obrigatórios, então aprofundar C++ e seguir para Rust.

As fatias corrigem falhas confirmadas nos donos existentes. Enquanto se
constrói outro ambiente, o endurecimento do host pode avançar; o ambiente
pendente conserva seu aceite aberto. Regressões das outras linguagens e dos
recursos embarcados oferecidos continuam necessárias durante a sequência.

### 4.1 C1 — Projeto e ambiente: recorte prioritário

**Prioridade de 2026-11-04: consolidar C1 antes de ampliar C2–C8.** O template
C próprio e a escolha de instalação/padrão já integram a criação, conforme a
[especificação de projetos §§4.0–4.1](projetos-arquivos-e-integracao-desktop-0.3.md).
São a base a fortalecer. C1 permanece aberto até a jornada abaixo estar
integrada e comprovada, com cobertura e limitações explícitas.

Versão do compilador e padrão da linguagem são escolhas distintas. A criação
permite selecionar uma instalação local de GCC/Clang, inclusive por caminho
alternativo, e solicitar um padrão/dialeto compatível com o projeto e as
ferramentas. Sugestões não formam uma lista fechada de revisões. C23 continua
referência de prova, sem imposição ao projeto. Escolha automática não inventa
uma revisão; escolha explícita persiste e não admite substituição silenciosa
de ferramenta ou redução de padrão. Obter outra instalação é uma operação
explícita, separada da criação.

| Jornada C1 | Evidência de aceite |
| --- | --- |
| Criar projeto apenas C | Selecionar instalação, padrão opcional, destino e configuração pela UI; conferir prévia e arquivos gerados, sem impor revisão da linguagem |
| Abrir projeto C existente | Reconhecer configuração/toolchain do projeto, respeitando presets, CDB, flags e arquivos manuais; propor mudanças com prévia e aplicação explícita |
| Persistir e retomar | Reabrir o projeto e alternar configuração/workspace mantendo a escolha efetiva; tornar origem, seleção automática e divergências compreensíveis |
| Conferir a seleção real | GCC e Clang reais, com instalações/versões identificadas; configurar e compilar um caso mínimo, conferindo compilador, padrão e configuração na CDB/contexto consumidos pela IDE |
| Falhar e recuperar | Caminho inválido, ferramenta ausente ou removida após a seleção, padrão incompatível, destino conflitante e cancelamento; preservar arquivos e buffers não salvos e permitir corrigir/repetir |
| Provar o fluxo visual | Projeto novo e existente, espaços/Unicode, prévia/aplicação, reabertura e recuperação na UI; regressões e mutações pertinentes, sem atribuir a ferramentas simuladas uma prova real |

Reusar os donos de criação, configuração, ambiente/toolchain, CMake e contexto.
A prova mínima de configure/build confirma a escolha de C1; a matriz completa
de Make, dependências, bibliotecas e múltiplas unidades pertence a C2.
`typeof(expr)`, gramática/corpus C23 e ampliação do editor permanecem em C3;
análise, depuração e embarcados conservam C4–C6. QEMU integra C6/F3, sem ser
pré-condição para concluir o recorte de projeto/ambiente de host. Cada aceite
pendente mantém seu alcance aberto, inclusive plataformas não exercitadas.

Conferência de 2026-11-08 ([62 §3.4](../roadmaps/62-base-publica-0.4.md#34-inventário-sistemático-de-dívida-técnica)):
abrir um projeto existente ainda não respeita o `binaryDir` dos presets (C-04)
nem localiza uma CDB fora dos diretórios conhecidos (C-05); os dois pertencem
a este recorte. A CDB com uma unidade por arquivo (C-03) pertence a C2.
O inventário e o desenho de C-05 estão em §16.

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

Após C1–C8 implementados, integrados e comprovados, a base C entra em maturação
por uso real: corrigir falhas reproduzíveis, ampliar cobertura, melhorar UX e
otimizar com medições. O polimento já acompanha cada jornada e continua nessa
maturação; não substitui requisitos obrigatórios ainda abertos. As frentes C++,
Rust/interoperabilidade, Python e F1–F9 mantêm seus próprios aceites da 0.4.

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

## 8. Configuração do padrão C por alvo — desenho de 2026-11-03

A ação `cmake.setCStandard` amplia o fluxo existente de configuração:
listagem contextual → parâmetros → prévia sem escrita → aplicação explícita.
O alvo é obrigatório; o padrão oferece C90/C99/C11/C17/C23, com C23 como
referência inicial, e as extensões oferecem OFF/ON. A receita escreve
`C_STANDARD`, `C_STANDARD_REQUIRED ON` e `C_EXTENSIONS` no alvo escolhido.
As variáveis `CMAKE_C_*`, os presets e os outros alvos conservam seu conteúdo.
A prévia explica que a escolha substitui os defaults herdados nesse alvo e
que configurar/build e consultar o contexto efetivo são passos posteriores.
O CMake continua responsável por validar a capacidade da toolchain.

O recorte de edição é o `CMakeLists.txt` da raiz, com alvo literal que lista
fonte `.c`. A ação não avalia a linguagem CMake: recusa fluxo condicional,
scripts externos, nomes dinâmicos e sintaxe que não consegue delimitar.
Propriedades manuais de padrão/extensões e flags explícitas de dialeto no
alvo escolhido exigem edição manual, com motivo na prévia. Propriedades de
outros alvos são preservadas. Somente o bloco exato gerado por esta ação pode
ser atualizado por ela; bloco alterado manualmente também é recusado.

Os parâmetros e snapshots usam o contrato existente de `configAction.*`;
o catálogo passa a distinguir sugestões de padrão C das de C++. O schema
dos envelopes não ganha campos. A adição da receita e de seus parâmetros
é versionada no [domínio IPC](../arquitetura/03-dominios/configAction.md).
Template C e identidade documental de headers/LSP permanecem fatias próprias.

### 8.1 Integração e provas — 2026-11-04

A receita está integrada ao catálogo e aos três métodos existentes, no
protocolo 0.199.0. A UI revoga a prévia quando os parâmetros mudam ou uma
consulta falha; a ponte descarta respostas de prévias anteriores pelo id da
requisição. O botão só aplica uma prévia vigente, com o snapshot exibido.

Sete regressões Rust novas reusam a fixture C11, CMake/Ninja, CDB/file-api,
contexto, CTest e verificador. A matriz C11/C17/C23 passou com GCC 16.2.1 e
Clang 23.1.4 em Linux x86_64. Só o alvo escolhido muda; biblioteca e teste
conservam C11. Troca GNU/ISO, snapshot obsoleto, configuração manual, alvo
incorreto e recuperação foram exercitados. Dois mutantes compiláveis foram
eliminados: ignorar propriedades manuais e editar outro alvo.

O caso C23 acrescenta `static_assert` e confere `__STDC_VERSION__ >= 202311L`;
verifica uma correção textual com compilador real, recusa sua aplicação após
mudar o padrão, provoca falha em C17 e recupera build/CTest em C23. As
sugestões LSP desse teste são controladas. O CMake 4.4.5 produziu `c2x` para
GCC e `c23` para Clang; o contexto conserva o dialeto efetivamente observado.
Isso não transfere o aceite para todo compilador que reconhece `c2x`.

Interação automatizada de mouse/teclado na UI Qt 6.12: ação C encontrada,
alvo selecionado, prévia C23 sem escrita, padrão inválido recusado, recuperação
e aplicação explícita. Build/CTest pela UI passaram, e o painel de contexto
mostrou Clang, `c23` e alvo `c_host`, coerentes com a CDB gerada. A comparação
visual usa a interface existente; não representa aprovação de um novo desenho.

A regressão Qt `configaction-response-identity` usa a ponte, os roteadores e
o controller de produção contra um core controlado: sucesso/falha antigos
fora de ordem são descartados, falha vigente bloqueia aplicação e uma nova
consulta recupera o fluxo. Cinco fases passaram; a aplicação levou os
parâmetros e o snapshot da última prévia. Reprodução sobre a UI compilada:

```bash
python3 scripts/verificar_probe_response_identity.py --build-dir build/dev-local \
    --domain configaction \
    --artifacts-dir "${XDG_CACHE_HOME:-$HOME/.cache}/kinein-vectis/qa-response-identity"
```

Gate rápido de 2026-11-04 aprovado: 1437 testes Rust, Clippy, análise de 50
arquivos C++, 8 CTest e 178 harnesses QML. Permanecem dois NÃO PROVADOS ARM
por ausência de `arm-none-eabi-gcc` e `arm-none-eabi-g++`. Não equivale ao
gate completo estrito de release.

Alcance parcial de C1/C2. C90/C99 são opções da receita, sem novo corpus
nativo nesta fatia. Template C, identidade documental C/headers, corpus C23
completo no editor, análise/memória, Make, cross e Windows continuam abertos,
assim como os demais aceites C1–C8. Não houve nova prova DAP nesta fatia.

## 9. Projeto C próprio e configuração aberta — desenho de 2026-11-04

`workspace.createProject` ganha o template `cCmake` no fluxo existente.
O seletor distingue C e C++; a prévia anuncia C23 inicial, arquivos e presets.
A criação interna não executa compilador nem modifica diretório existente.
Gera `src/main.c` compatível com C90, diretórios `include/` e `tests/`, presets
Ninja Debug/Release, `.gitignore`, README e teste de execução CTest. O padrão
inicial C23 é exigido por alvo, sem extensões, usando o mesmo serializador da
ação `cmake.setCStandard`; a escolha posterior usa sua prévia/aplicação.
Configurar sem suporte conserva a falha da ferramenta, sem reduzir o padrão.

A ação passa a aceitar um identificador decimal positivo de `C_STANDARD`,
sem enumeração fechada na IDE. Os valores sugeridos continuam exemplos; o
CMake efetivo valida o pedido, incluindo valores que só uma versão futura
dele venha a aceitar. A ação pode atualizar seu bloco exato também nesses
valores. Flags e aliases específicos do compilador, como `iso9899:199409` e
`c2y`, permanecem no projeto/CDB e não são traduzidos em número CMake por
adivinhação. `-ansi`/`--ansi` são exibidos como seletores de dialeto observados.
O reconhecimento conserva também grafias futuras desconhecidas da IDE.

O contrato vem antes dos produtores no protocolo 0.200.0, nos domínios
`workspace` e `configAction`. Template C++ e demais linguagens conservam seus
arquivos e fluxos. Identidade contextual de headers e languageId C no clangd
seguem como fatia própria; esta entrega não fecha C1–C8.

### 9.1 Integração e provas — 2026-11-04

Template `cCmake` integrado ao gerador de projetos, protocolo 0.200.0 e
catálogo visual. C e C++ têm cartões distintos; a prévia anuncia C23 inicial,
fonte C90, presets e geração interna. A gravação C/C++ compartilha o dono de
templates; arquivos e opções do template C++ são conservados. A criação C
reusa o bloco da ação de padrão, com alteração posterior explícita por alvo.

Interação na UI nativa comprovou seleção C separada de C++, prévia antes da
escrita, escolha do diretório pai e recusa de nome inválido com recuperação.
O projeto criado foi aberto; configuração, build e CTest de execução (1/1)
passaram pela interface. O contexto do arquivo conservou o seletor efetivo da
CDB. A prova usa os componentes visuais existentes, sem novo aceite de desenho.

Regressões pelo despacho do core criam o projeto em caminho com espaços e
Unicode, configuram e exercitam build/contexto/CTest de C90, C99, C11, C17 e
C23 com GCC 16.2.1 e Clang 23.1.4. O pedido `2099` passa pela prévia/aplicação,
é recusado pelo CMake 4.4.5 e recupera ao voltar para C23; a IDE não reduz o
padrão nem sobrescreve projeto existente em colisão.

Outras 48 compilações básicas, com `-pedantic-errors -fsyntax-only`, passaram
nos dois drivers: 24 modos/aliases por ferramenta, incluindo C89, a emenda
C90 de 1994, aliases ISO/GNU, C23 e C2y. A CDB conserva o argumento e a grafia
do modo. Casos controlados conservam valores futuros/vendor, default ausente,
último seletor e `/std:clatest`; não são prova nativa MSVC. Esta matriz mede
aceitação do seletor e uma fonte mínima, sem certificar toda a linguagem,
biblioteca, gramática do editor ou recursos da revisão futura.

Três mutantes compiláveis eliminados: voltar à lista fechada na ação,
filtrar dialetos na CDB e criar C++ ao selecionar C. Gate rápido aprovado
em 314 s: 1440 Rust, Clippy, 50 arquivos C++, 8 CTest e 178 harnesses QML.
Os dois NÃO PROVADOS ARM persistem por compiladores cross ausentes.

## 10. Identidade documental C/C++ — desenho de 2026-11-04

A família da linguagem do documento deve ser independente da chave do servidor:
C e C++ continuam no mesmo clangd, sob a chave histórica `cpp`. Fontes C usam
`languageId: c`; fontes C++ usam `cpp`. O padrão/dialeto permanece nos argumentos
originais, sem lista de revisões nem inserção de flags pela seleção de gramática.
`.C` e `.H` conservam a convenção C++; `.c` conserva C. A seleção local usa o mesmo
contexto do índice: comando explícito CDB (`-x`, `/TC`, `/TP`) e driver C++
conhecido precedem a extensão, sem executar compiladores na consulta.
O recorte respeita a posição da entrada: `-x` posterior ao próprio arquivo
não muda sua família, conforme a [semântica do driver GCC](https://gcc.gnu.org/onlinedocs/gcc/Overall-Options.html).

Para `.h` sem comando próprio, os alvos que o declaram no codemodel podem
determinar uma única família C/C++. Sem dono explícito, uma CDB homogênea pode
orientar a gramática local. Contexto ausente, conflitante ou misto conserva o
fallback C++ de compatibilidade, com alcance explicado em `index.context`.
Não se deduz um includer por proximidade de nome nem se reimplementa o
pré-processador. A inferência semântica de includes/flags continua no clangd,
conforme sua [documentação de comandos](https://clangd.llvm.org/design/compile-commands).
Um header compartilhado não ganha uma unidade fictícia na CDB.

Editor, extração estrutural e `index.context` devem consumir a mesma escolha.
Recarregar contexto refaz a estrutura dos documentos cuja família mudou;
Tree-sitter descarta a árvore anterior de outra gramática. No LSP, uma mudança
de família exige `didClose`/`didOpen`, mesmo com texto idêntico, e invalida ações
baseadas na identidade anterior. Troca de workspace não reutiliza contexto antigo.
Os envelopes IPC existentes já comportam `c` e `cpp`; esta correção não adiciona
campo/método nem altera a versão 0.200.0. A prova deve separar wire controlado,
gramática local, clangd real e interação na UI. C3 permanece parcial até seus
demais aceites; headers ambíguos não equivalem a resolução completa de inclusões.

### 10.1 Integração parcial e limites observados — 2026-11-04

Identidade documental integrada nos donos de contexto, Tree-sitter, índice
estrutural e sincronização LSP. O índice carrega contexto antes da extração e
refaz somente os arquivos cuja gramática mudou ao recarregá-lo. O editor pede
novos snapshots após `event.index.finished`, inclusive sem edição do buffer.
O watcher conserva o mesmo canal de eventos usando o tipo `mpsc::Sender`
diretamente; a dependência de alias no LSP foi retirada para manter o grafo
sem novo ciclo ao compartilhar o contexto do índice com o manager.

Quatro regressões novas: seletores CDB sem enumeração de revisões; `.C`, `-x`
e `/TC`/`/TP`; headers em contexto homogêneo/ambíguo; wire de C/C++ com um
único servidor, reabertura e invalidação com texto igual, troca de workspace
e rejeição de contexto da raiz anterior; projeto misto pelos produtores reais.
A fixture `scripts/fixtures/c-identity/` usa biblioteca C23, executável C++17,
headers C/C++ próprios e uma API compartilhada. Configuração/build/CTest,
identidade contextual e gramáticas dos headers passaram com GCC 16.2.1 e
Clang 23.1.4; hover do header
C passou com clangd 23.1.4. Seletores MSVC são casos controlados, sem prova Windows.
Três mutantes compiláveis eliminados: `languageId` C convertido em C++, dono
C do header convertido em C++ e reabertura por mudança de família desativada.

Na UI nativa foram comprovados realce e inspeção dos headers C e C++, a dica
de ambiguidade do header compartilhado, build misto e CTest 1/1. Para o header
C, o compilador de fallback do inspetor mudou de C++ para C, mantendo unidade
ausente. A prova funcional usa os componentes existentes.

**Limites que mantêm C3 aberto:** a gramática `tree-sitter-c` 0.24.2 sinaliza
erro em `typeof(1)` usado como tipo, embora GCC/Clang aceitem a fonte C23.
Prova isolada por IPC conservou `language: c`; `static_assert`, `_Static_assert`
e `_Generic` passaram sem erro sintático local. As quatro fontes passaram
também em oito compilações básicas GCC/Clang; isso não certifica o corpus C23.

O clangd real, ao abrir primeiro o header C++ da fixture mista, escolheu um
comando C por inferência e publicou erro em `namespace`, apesar da identidade
local C++ correta. O caso foi observado tanto por IPC quanto na UI. A seleção
local entregue não corrige essa inferência semântica: associar uma unidade de
referência confiável e invalidá-la continua pendente em C2/C3. O mecanismo de
[comandos via LSP do clangd](https://clangd.llvm.org/extensions#compilation-commands)
é uma possibilidade técnica a avaliar, sem escrever CDB ou flags inventadas.
Não há aceite completo de headers mistos, gramática C23 ou fechamento C1–C8.

Gate rápido final aprovado em 283 s: 1444 Rust, Clippy sem avisos, análise de
50 arquivos C++, 8 CTest e 178 harnesses QML. Dois NÃO PROVADOS ARM por
`arm-none-eabi-gcc`/`arm-none-eabi-g++` ausentes. Revisão editorial e gates de
documentação, nomenclatura, mapa e arquitetura aprovados. Não equivale ao
gate completo estrito de release nem aceita os limites funcionais acima.

## 11. Referência semântica de headers — desenho de 2026-11-04

O dono `index::context` pode associar um header sem entrada própria na CDB
a uma unidade quando o codemodel declara exatamente um alvo proprietário e
esse alvo compila exatamente uma fonte com comando CDB não duplicado. Mais de
uma fonte/alvo, CDB obsoleta, ausência de comando ou configuração manual da
base em `.clangd` conservam a inferência do servidor, com motivo em `hint`.
A associação prova propriedade no build; não afirma que a fonte inclui o header.
Headers compartilhados continuam ambíguos. Não escolher pela ordem de abertura,
nome próximo ou primeira entrada da CDB. A unidade do header permanece ausente.

O LSP envia o argv e diretório originais, sem alterar qualquer flag, pela
[extensão `compilationDatabaseChanges`](https://clangd.llvm.org/extensions#compilation-commands).
O [CommandMangler do clangd](https://github.com/llvm/llvm-project/blob/main/clang-tools-extra/clangd/CompileCommands.cpp)
transfere o comando da fonte para o header e aplica depois os ajustes `.clangd`.
Essa responsabilidade permanece na ferramenta; não há parser de opções ou
pré-processador novo na IDE. Comando explícito do header na CDB prevalece.

O manager compara a referência atual antes de sincronizar ou consultar. Mudança
de comando reabre com o buffer recebido e invalida code actions, mesmo com
conteúdo/família iguais. A extensão não oferece remoção de uma entrada; perder
uma associação exige reiniciar o clangd pelo ciclo de vida existente. O evento
de reinício solicita o buffer vivo à UI, sem reler disco ou salvar edições.
Trocar raiz descarta o processo e suas associações. Sem novo schema IPC: a
explicação usa `index.context.hint` e os eventos LSP existentes.

Provar abertura do header C++ antes de fontes, flags por alvo, ambiguidade,
comando explícito, configuração manual, troca de contexto sem edição e buffer
não salvo. Essa associação conservadora não encerra C2/C3 nem resolve headers
de múltiplas unidades com configurações diferentes.

### 11.1 Integração conservadora e provas — 2026-11-04

A referência está integrada ao contexto e à sincronização documental existentes.
O header mantém `unit` ausente; a dica identifica alvo/fonte ou explica ausência
ou ambiguidade. O LSP envia o comando original antes de abrir o header, compara
argv/diretório e invalida ações mesmo sem mudança de texto/família. A atualização
do índice revalida também referências de headers inativos. Retirar um override
reinicia pelo ciclo de vida existente, pois a extensão não oferece remoção.

Três regressões novas complementam as quatro de identidade: comando explícito
do header, donos compartilhados/ausentes, CDB duplicada, configuração manual,
argv integral no wire, troca de flags com buffer não salvo e retirada de override.
Clangd 23.1.4 real interpretou o header C++ antes das fontes, manteve uma definição
adicionada por `.clangd`, diagnosticou erro introduzido somente no buffer e
recuperou após a edição. Três mutantes compiláveis eliminados: seleção de primeiro
alvo em ambiguidade, envio do comando desativado e comparação de contexto ignorada.

A associação atual exige alvo e fonte únicos; múltiplas fontes, mesmo com flags
aparentemente semelhantes, conservam inferência do clangd. Não se afirma conhecer
includes transitivos ou aceitar todas as configurações de um header compartilhado.
A gramática `typeof(expr)` e os demais aceites C1–C8 continuam abertos.

Na UI nativa, o header C++ aberto antes das fontes deixou de apresentar erro
em `namespace`. A prova em fixture isolada preservou a configuração manual
herdada por diretórios no caso separado. Não houve aceite de release nem
fechamento da etapa C.

## 12. C1: dialeto na criação e validação do driver — desenho de 2026-11-04

Inventário localizado de C-01, antes da implementação:

| ID / dono | Evidência e causa confirmada | Severidade / impacto | Testes e decisão |
| --- | --- | --- | --- |
| C1-01 / `workspace/project_options.rs`, `cmake_template.rs` | `workspace.createProject`, template C e `standard: gnu17`, retorna `INVALID_PARAMS` antes de gerar. O validador só aceita dígitos e o template só emite `C_STANDARD` ISO | Alta: a criação não expressa dialetos e aliases que o contexto já reconhece | Corrigir nesses donos: aceitar token de dialeto sem enum de revisões; GCC/Clang reais, alias ISO, GNU e prévia, CDB/contexto, reabertura e mutação da persistência |
| C1-02 / `workspace/project_options.rs` | Selecionar `g++` com padrão 17 cria projeto apenas C. A validação só confere o texto de `--version`, compartilhado entre drivers | Alta: seleção aceita pode configurar o projeto como C++ | Corrigir: testar uma unidade `.c` que recusa `__cplusplus` com o executável selecionado, sem forçar `-x c`; incompatibilidade/ausência não cria destino. Provar erro, repetição e mutação da guarda |
| C1-03 / `handlers/workspace_creation.rs` | A validação roda no handler síncrono; uma ferramenta controlada que não termina ocupa o laço durante o prazo da sonda | Média: eventos e consultas independentes aguardam a ferramenta | Corrigir pelo `defer_then` existente: validar fora do laço e gerar/ativar na continuação. Se a raiz ativa mudar durante a espera, recusar antes de gerar; regressão de resposta adiada e guarda de workspace |
| C1-04 / regressão `c_identity/headers.rs` | O gate observou comando anterior após `reload_index_context`; caso isolado passou. O teste drena eventos uma vez, embora a recarga rode em job e a identidade da família permaneça igual | Média: falso negativo na prova de preservação de headers durante configuração | Corrigir a sincronização do teste: aguardar a flag nova no contexto antes de conferir invalidação/wire, conservando as asserções e mutações das guardas |

Estado inicial dos registros: defeitos de escolhas reproduzidos pelo despacho IPC;
correção e provas ainda pendentes. C1 completo permanece aberto em §4.1.

Desenho: números positivos continuam como propriedades CMake por alvo,
`C_STANDARD_REQUIRED ON` e extensões OFF. Um token literal de dialeto, como
`gnu17`, `c2y` ou `iso9899:1990`, gera `target_compile_options` com o argumento
`-std=` exato. Aceitar sintaxe de token não certifica suporte da ferramenta;
não traduzir aliases/prévias em uma revisão diferente. Recusar metacaracteres
de CMake e espaços; não manter uma lista de dialetos válidos. A receita numérica
existente continua conservadora diante de flags manuais ou desse bloco literal.

Uma seleção explícita de driver C passa por consulta de versão e compilação
sintática limitada de uma fonte própria, em temporário privado, antes da geração.
O teste usa o dialeto pedido quando literal. Para números, confere somente o
modo C do driver; traduzir a propriedade e exigir o padrão cabe ao CMake no
configure. Não deduzir `-std=c<número>` na sonda: o CMake pode usar um alias
do compilador para aquela propriedade.
Consultas externas rodam fora do laço por `defer_then`; a continuação confere
a raiz ativa antes de gerar/ativar o projeto. Mudança de workspace recusa a
operação antes da criação, sem assumir que uma resposta adiada ainda é atual.
Não liga nem executa o programa e não testa libc, SDK ou conformidade completa.
Sem executável explícito, a criação conserva a seleção automática: configure/build
continuam responsáveis pela disponibilidade e compatibilidade efetivas. O padrão
vazio não emite flag ou propriedade. Padrão, caminho e motivo da recusa aparecem
no fluxo atual, permitindo corrigir/repetir sem trocar de workspace no erro.

O contrato de `options.standard` em C deve ser ampliado antes do código; C++,
Rust e Python conservam seus formatos. A UI distingue versão da instalação,
propriedade numérica e token literal na prévia. As escolhas persistem nos arquivos
do projeto, lidos pelos donos existentes na abertura/configuração; nenhuma nova
base de configurações, download ou serviço de toolchain é introduzido.

### 12.1 Integração e alcance das provas — 2026-11-04

Protocolo 0.202.0: números e tokens literais compartilham a criação existente,
com validação antes de gerar arquivos do projeto. A consulta de instalações e
a criação reconhecem o marcador de resposta adiada no despacho. A continuação
retoma somente se a raiz ativa for a mesma. A sonda usa o dono de processos,
prazo de três segundos e limpeza do temporário privado em sucesso/erro.

Cinco regressões de C1 exercitam GCC 16.2.1, Clang 23.1.4 e CMake 4.4.5 reais
em Linux x86_64. GNU17, `iso9899:1990`, C2y e padrão automático configuram,
compilam e passam em CTest; CDB e `index.context` conservam compilador, dialeto
exato e origem. Reabertura com core novo, troca de workspace e Debug/Release
preservam fonte, presets e `.clangd` manual. Alias futuro é conservado sem
driver explícito, sem afirmar suporte da ferramenta. Drivers C++, ferramenta
ausente/removida, dialeto literal recusado, token inválido e destino conflitante não
substituem conteúdo; corrigir/repetir funciona. A prova de caminho alternativo
com espaços/Unicode usa encaminhador para GCC real; somente a sonda de prazo
é uma ferramenta controlada, sem atribuir a ela capacidade de compilador.
Padrão numérico 2099 com driver real cria a propriedade exigida e é recusado
pelo CMake efetivo, sem redução silenciosa. O encaminhador registra argv para
conferir que a sonda não inventou uma grafia de `-std` para a propriedade 17;
o build e a CDB comprovam a tradução feita pelo CMake para C17.

A regressão de headers aguarda as flags recarregadas pelo job antes de conferir
invalidação e wire. A família C++ e o estado Ready, isoladamente, não identificam
essa atualização. As asserções de buffer, argv e geração permanecem exigidas.

Na UI nativa isolada, driver C++, caminho inexistente e dialeto recusado mantêm
as escolhas para correção. A recuperação com GCC/GNU17 cria apenas C em destino
com espaços/Unicode; a prévia mostra `-std=gnu17` literal. Debug, build e CTest
1/1 foram acionados pela UI; a CDB contém o driver selecionado e a opção exata.
Reabrir a fixture recupera abas e contexto GCC/GNU17. Com salvamento automático
desativado somente na fixture, erro de criação e cancelamento mantêm texto
adicionado apenas ao buffer, indicador de alteração e arquivo em disco intacto.

Gate rápido final aprovado em 311 s: 1455 Rust, Clippy, 51 arquivos C++, 8 CTest
e 178 harnesses QML. Dois NÃO PROVADOS ARM por compiladores cross ausentes.
Quatro mutantes compiláveis de C1 eliminados: erro do driver ignorado, dialeto
persistido substituído, sintaxe de token liberada e guarda de workspace removida.
Três mutantes de headers também eliminados após a correção da sincronização:
ambiguidade, envio de comando e identidade de contexto. C1-01–C1-04 corrigidos
e validados no recorte descrito; o aceite global C1 permanece aberto.

Esta fatia não encerra C1. Abertura/configuração visual de projetos manuais
arbitrários, divergências de ambiente, remoção após configure, cancelamento de
sonda em andamento e a matriz nativa Windows continuam sem aceite completo.
O diálogo pode ser cancelado antes de submeter; a sonda submetida termina pelo
resultado ou prazo. Libc, SDK, conformidade completa de C2y/C23 e C2–C8 conservam
suas provas próprias. `typeof(expr)` e QEMU permanecem em C3 e C6/F3.

## 13. C1: validade do ambiente de projetos existentes — desenho de 2026-11-05

Inventário localizado antes da correção, nos donos de CMake e contexto:

| ID / capacidade | Evidência e causa | Severidade / impacto | Decisão e prova necessária |
| --- | --- | --- | --- |
| C1-05 / configurar e repetir | `cmake::status` só verifica artefatos; um configure que falha conserva cache/CDB anteriores. `choose_default` retoma o preset do último sucesso, mesmo depois de outra escolha falhar. Causa confirmada no fluxo | Alta: evidência anterior parece atual e a repetição pode abandonar a configuração escolhida | Corrigir nos donos existentes: registrar tentativa incompleta antes de executar, invalidar sua evidência e repetir a mesma escolha. Provar falha real, reabertura, recuperação e cancelamento do configure |
| C1-06 / ambiente efetivo | `CompileContext` conserva unidades e `effective` após remoção do executável; a atualidade compara somente origem/bytes da CDB | Alta: instalação indisponível aparece efetiva; mudanças externas podem deixar a resposta anterior na UI | Corrigir a consulta: distinguir comando histórico de contexto atual, sem substituir compilador. Provar GCC/Clang reais, remoção após configure, mudança de CDB, prévia/aplicação e buffer não salvo |
| C1-07 / cancelamento da sonda de criação | `defer_then` não fornece um job cancelável ao diálogo; Cancelar antes de submeter não interrompe a sonda submetida | Lacuna de aceite, alta no fechamento C1; não é prova de cancelamento em andamento | Manter aberto: esta fatia trata configure cancelável pelo dono de jobs; integrar cancelamento da sonda exige recorte próprio no fluxo de criação |
| C1-08 / troca de workspace durante indexação/recarga | As continuações da indexação inicial e de `reload_index_context` aplicam resultado sem conferir a raiz do índice; uma consulta lenta pode terminar depois da abertura de outro projeto. Causa confirmada por inspeção | Alta: contexto da raiz anterior contamina os totais/contexto da nova | Corrigir as guardas no dono existente e exercitar os dois trabalhos bloqueados por produtor controlado, troca de raiz e liberação do resultado anterior |

Estado inicial: C1-05/C1-06/C1-08 confirmados por inspeção; regressões de reprodução
e provas dos consumidores pendentes. C1-07 conserva a lacuna documentada.

O CMake registra no build dir somente a tentativa ainda não concluída, com
o preset solicitado e o kit usado, antes de executar o configure explícito ou
automático.
Sucesso remove o registro; falha, cancelamento ou interrupção o conservam.
A tentativa mantém lock nativo do arquivo no build dir durante o produtor;
outro configure da IDE é recusado enquanto esse lock estiver ocupado. Isso
impede que o sucesso de uma tentativa limpe o registro de outra ainda ativa.
A repetição sem escolha nova usa esse preset, inclusive a escolha sem preset;
preset removido ou registro ilegível exigem correção explícita. O preset do
último sucesso permanece distinguível do solicitado no diagnóstico. Não
reescrever presets, toolchain files, flags ou fontes para recuperar.

`index.context` deve identificar dados históricos com `staleBecause`, conservar
o comando para inspeção e omitir `effective`/deriva enquanto obsoleto. Conferir
origem/bytes atuais da CDB, fontes de invalidação já conhecidas e existência do
compilador com caminho explícito; nome simples continua dependente do ambiente
do produtor, sem resolução inventada. Configure incompleto invalida somente
os artefatos do build dir da IDE; uma CDB manual externa conserva seu dono.
A UI apresenta o contexto como desatualizado e permite inspecionar o comando
anterior. Isso não certifica disponibilidade de SDK/libc nem interpreta `.clangd`.

### 13.1 Integração e alcance das provas — 2026-11-05

Protocolo 0.203.0, produto 0.3.9. Sete regressões de projetos existentes usam
GCC 16.2.1, Clang 23.1.4 e CMake 4.4.5: configure/build, GNU17 na CDB/contexto,
Debug/Release, caminho com espaços/Unicode, toolchain file e arquivos manuais.
Falha conserva o último sucesso separado da tentativa pendente; repetição,
reabertura, preset removido e registro inválido não retornam ao preset anterior.
A escolha automática conserva o kit padrão e a ausência de preset quando aplicável.

Remoção do compilador após configure/build torna a consulta histórica antes de
nova execução; forçar recompilação comprova falha, e restaurar o mesmo caminho
permite recuperar. Trocar os bytes da CDB invalida o snapshot carregado.
CDB gerada externamente continua sendo lida em `build/`, mesmo com tentativa
incompleta no build dir da IDE. `.clangd`, `compile_flags.txt`, toolchain file,
fontes e outros alvos permanecem intactos. Essa preservação não prova que as
opções adicionais do clangd estejam representadas em `index.context`.

Cancelamento é exercitado depois de uma mensagem do CMake real confirmar o
início do produtor; ele é interrompido, o registro permanece obsoleto e a repetição
recupera. Outro configure no mesmo build dir é recusado durante a tentativa.
Uma consulta Cargo controlada bloqueia indexação inicial e recarga para provar
a troca de workspace: seu resultado tardio não substitui o contexto da nova raiz. Essa
ferramenta controlada prova sincronização, sem certificar capacidades do Cargo.

Na UI nativa isolada, prévia não grava; aplicação explícita preserva as demais
configurações. Com `autoSave=false` efetivamente recebido pela UI, o marcador
no buffer e o indicador de alteração permanecem durante aplicação, falha de
configure e recuperação. O marcador continua ausente do disco. A instalação
GCC escolhida e GNU17 reaparecem na CDB/contexto após recuperar.

Oito mutantes compiláveis eliminados com baseline de sete casos aprovado:
CDB incompleta, seleção da repetição, compilador ausente, efetivo obsoleto,
guardas de raiz da indexação/recarga, kit automático e lock do configure.

Gate rápido integrado aprovado em 318 s: 1462 Rust, Clippy, 51 arquivos C++,
8 CTest e 178 harnesses QML. Dois NÃO PROVADOS ARM por compiladores cross ausentes.
Formatação e gates documentais aprovados; sem alteração de baselines/allowances.

C1-05/C1-06/C1-08 têm correção e provas localizadas; C1 permanece aberto.
A matriz nativa Windows não foi executada. Cancelamento da sonda de criação
submetida (C1-07), seleção visual unificada de instalações/dialetos e troca de
configuração em projetos manuais arbitrários, divergências e arquivos de build transitivos conservam
aceites próprios. O diagnóstico não verifica conteúdo/versão do binário no
mesmo caminho, disponibilidade de SDK/libc ou toda alteração externa do file-api.
O cancelamento provado é do processo de configure em andamento; não abrange
uma consulta de versão auxiliar bloqueada antes dele. Não encerra C2–C8.

## 14. C1: seleção visual de opções conhecidas — desenho de 2026-11-05

| ID / capacidade | Evidência e reprodução | Severidade, impacto e causa | Decisão e provas necessárias |
| --- | --- | --- | --- |
| C1-09 / escolha na criação e configuração | `ProjectEnvironmentFields` oferece instalações em uma lista de duas linhas e exige texto para padrão; `ConfigActionField` mantém entrada de texto mesmo quando `suggestions` já contém alvos e valores. Abrir a configuração do padrão C reproduz o fluxo | Média: transcrição desnecessária de opções conhecidas e baixa descoberta das instalações. Causa confirmada na apresentação dos dados já fornecidos pelos donos | Corrigir nos consumidores existentes com um seletor compartilhado, opção vazia explícita e entrada personalizada acessível. Provar seleção por teclado/clique, cancelamento do menu, atualização sem substituição da escolha, opções futuras, prévia/aplicação e preservação na UI |
| C1-10 / buffer ao retornar ao arquivo | Com `autoSave=false`, editar `main.c`, configurar/compilar, abrir `other.c` pela árvore e retornar a `main.c` elimina o texto não salvo e seu indicador; marcador continua ausente do disco. Reprodução confirmada na UI e nas respostas `fs.read` | Alta: perda de alterações durante a conferência do ambiente. Causa confirmada: a carga ordinária de arquivo já aberto chama `applyDiskContent` e substitui também a superfície | Corrigir em `EditorDocumentController`: carga ordinária ativa o documento existente por identidade; recarga explícita e conflito externo continuam nos seus fluxos. Provar buffer ativo/inativo, resposta repetida, identidade, baseline salvo e recarga explícita; mutação da proteção e repetição nativa da troca de arquivo |

O seletor apresenta as instalações medidas por `workspace.projectEnvironments`
com versão e caminho. Sugestões de padrão são exemplos, sem enum fechado:
o valor personalizado conserva o contrato numérico/literal existente. Vazio
continua sem impor padrão na criação. Em Configuration Actions, as sugestões
continuam sendo valores do core; ausência de sugestões conserva a entrada livre.
Escolher uma opção solicita a mesma prévia; somente a aplicação explícita grava.
O menu precisa escapar da área rolável dos parâmetros, caber na janela e
devolver o foco ao fechar. Ocultar/destruir o formulário fecha o menu.

Não há mudança de schema ou semântica IPC; produto 0.3.9 e protocolo 0.203.0.
Esta fatia não altera as receitas nem transforma seus exemplos em limites.
A seleção livre de instalações em projetos existentes, alterações de presets
manuais arbitrários, origem/divergência completa, cancelamento da sonda
submetida e matriz Windows continuam com seus aceites abertos.

### 14.1 Integração e alcance das provas — 2026-11-05

`KvSelect` apresenta instalações, padrões/edições e sugestões das ações, com
automático/vazio e “Personalizado…”. A lista de instalações mostra versão
medida e caminho; atualizar não troca a escolha. Ocultar o formulário fecha
o menu e limpa apenas seu modo transitório de edição. Não há instalação ou
tradução de padrão no frontend. Ações sem sugestões conservam entrada livre.

A abertura ordinária de arquivo já aberto passa a ativar seu documento por
identidade, conservando texto, baseline salvo e indicador. Recarga explícita,
prévia externa somente leitura e tratamento de conflito conservam seus donos.
Dois harnesses adicionais exercitam os seletores e a preservação de buffer
ativo/inativo, resposta de disco repetida e recarga explícita. Oito mutações
QML válidas eliminadas com ambos os baselines verdes: caminho selecionado,
personalizado, vazio, ocultação, busca em andamento, atualização da escolha,
modelo do documento e texto da superfície.

Na UI nativa, um projeto C existente recusa a receita sobre alvo com flags
manuais. Outro alvo recebe a prévia e aplicação explícita; padrão numérico
indisponível falha no CMake, e escolher C17 recupera configure/build. A CDB e
`index.context` confirmam C17 nesse alvo e GNU17 no alvo manual. A remoção do
driver após configure produz falha na tentativa seguinte; restaurar o mesmo
caminho recupera, sem substituir a instalação. Fontes, presets, toolchain file,
`.clangd` e `compile_flags.txt` conservam seus bytes; somente o bloco aplicado
no `CMakeLists.txt` muda. Com `autoSave=false` confirmado pelo IPC, o texto e
indicador não salvos permanecem após troca pela árvore e configure/build;
marcador ausente do disco. C1-09/C1-10 têm correção localizada; C1 segue aberto.

A criação nativa por seletores usa GCC 16.2.1 e Clang 23.1.4, com CMake 4.4.5:
GNU17 literal e C17 numérico, respectivamente. Prévia, criação, seleção de
preset Debug, configure/build e reabertura conferem os caminhos escolhidos e
`-std=gnu17`/`-std=c17` na CDB e em `index.context`. O destino contém
espaços/Unicode. Esta prova não amplia a configuração de instalações em
projetos existentes arbitrários.

Gate rápido aprovado em 286 s: 1462 Rust, Clippy, 51 arquivos C++, 8 CTest e
180 harnesses QML. Dois NÃO PROVADOS ARM por compiladores cross ausentes.
Formatação, veracidade, links/privacidade e revisão editorial acompanham o
fechamento. Baselines, allowances e exclusões Windows não foram ampliados.
Sem prova nativa Windows, cancelamento de sonda submetida ou fechamento de C1.

## 15. C1: instalação do compilador em projetos existentes — desenho de 2026-11-08

Inventário localizado antes da mudança, nos donos `toolchain/`, `cmake/`,
`build/` e `configaction/`. Reproduções pelo despacho IPC com GCC 16.2.1,
Clang 23.1.4 e CMake 4.4.5 reais, encaminhados por caminhos alternativos:

| ID / capacidade | Evidência e causa | Severidade / impacto | Decisão e prova necessária |
| --- | --- | --- | --- |
| C1-11 / escolher instalação em projeto existente | `toolchain.set` aceita só ids do catálogo detectados (`gcc`, `clang` primeiro no `PATH`). Instalações paralelas enumeradas por `workspace.projectEnvironments` e caminho alternativo, aceitos na criação, não podem ser fixados no kit; a escolha é aplicada no clique, sem prévia. Lacuna confirmada por inspeção | Alta: aceite C1 de abertura/configuração sem escolha livre de instalação nem prévia | Corrigir reutilizando kit, Configuration Actions e a sonda C da criação: receita com prévia/aplicação, validação fora do laço, detectado/selecionado/efetivo, origem e divergências. Provar GCC/Clang, caminho com espaços/Unicode, inválido/C++ recusados, persistência, reabertura e mutações |
| C1-12 / compilador do kit removido | Com Clang fixado e removido, `toolchain.get` ainda informa `effectiveId: clang`; `cmake_definitions` deixa de emitir `CMAKE_C_COMPILER` e um configure novo usa `/usr/bin/cc` (GCC). Reproduzido | Alta: escolha explícita trocada em silêncio | Corrigir: escolha indisponível fica sem efetivo e configure explícito/automático recusa antes de produzir, nomeando a correção. Provar remoção, recusa, restauração/repetição e mutação |
| C1-13 / kit do configure sem preset explícito | Configure sem preset pedido usa o preset anotado (`buildDir`) com o kit padrão; o kit desse preset é ignorado. Com kits distintos, o CMake troca de Clang para GCC, enquanto `index.context` informa o kit do preset. Reproduzido | Alta: troca silenciosa de compilador e kit divergente entre consumidores | Corrigir: configure e build automático usam o kit ativo (`cmake::active_kit`), já usado por contexto e execução. Preset vindo dos arquivos sem kit próprio continua no kit padrão |
| C1-14 / CDB após troca de compilador | Ao mudar `CMAKE_C_COMPILER` num build dir configurado, o CMake 4.4.5 apaga o cache e repete o configure sem as definições da linha de comando: `CMAKE_EXPORT_COMPILE_COMMANDS` fica vazio, `build.ninja` usa o compilador novo e a CDB anterior permanece. Reproduzido por kit e por preset | Crítica: contexto do compilador anterior apresentado como atual | Corrigir no produtor de configure: conferir no cache as definições da IDE após sucesso; ausentes, repetir uma vez o mesmo configure e recusar se persistirem. Provar troca por kit/preset, CDB/contexto atuais e mutação |

Estado inicial: C1-12/C1-13/C1-14 reproduzidos; C1-11 confirmado por inspeção.
C1-07, matriz Windows e consulta de versão bloqueada continuam abertos.

A receita grava a instalação no kit (`.kinein/toolchain.json`) usado pelo
configure em uso ou no kit do preset informado, sem editar presets, toolchain files,
`CMakeLists.txt`, flags ou `.clangd`. Vazio volta ao automático. A prévia lista
instalações detectadas, escolha atual, compilador do último configure, origem
sem fixação (preset, toolchain file ou regra do CMake) e divergências: toolchain
file que pode prevalecer, compilador do preset sobreposto pela linha de comando
e cache que será recriado. Nada é instalado e o padrão C não muda.

### 15.1 Integração e alcance das provas — 2026-11-08

Protocolo 0.204.0, produto 0.3.9. A receita `cmake.selectCCompiler` usa o
kit existente: o `toolchain` planeja o texto de `.kinein/toolchain.json` com o
mesmo serializador da gravação, e a Configuration Action mostra diff, notas e
aplica com snapshot. Versão e sonda C da criação conferem o executável fora do
laço; mudar de workspace antes da continuação recusa a resposta. Sugestões
listam drivers C das pastas do detector sem consultar versão; a prévia lista
as instalações C com versão. O kit padrão descreve a origem pelo preset que o
configure sem preset usaria. `presetToolchainFile` passa a seguir `inherits`.

O configure registra em `.kinein-kit` o kit efetivamente usado. Tentativa
pendente, preset anotado e build automático usam esse kit, e a receita sem kit
informado edita o mesmo. Voltar um kit nomeado ao automático não entrega o
configure seguinte ao kit padrão. Compilador fixado indisponível, por id ou
caminho, perde `resolvedPath`/`effectiveId` e recusa `cmake.configure` e o
configure do `build.run` antes do CMake. Quando o CMake recria o cache ao trocar
de compilador e perde `CMAKE_EXPORT_COMPILE_COMMANDS`, os dois produtores repetem
uma vez o configure; a CDB e `index.context` passam a descrever o compilador
novo. Cache sem essa entrada, como o de ferramentas controladas, não dispara a
repetição.

Quatro regressões de projeto existente usam GCC 16.2.1, Clang 23.1.4 e CMake
4.4.5 reais, por caminho direto e encaminhadores em caminhos com espaços e
Unicode. Cobrem sugestões, recusa de caminho relativo, ausente e driver C++
sem gravar o kit; prévia sem escrita, notas de detectado/selecionado/efetivo,
divergência com o compilador do preset; aplicação, configure/build, troca
GCC→Clang com repetição e CDB/contexto atuais; reabertura; troca pelo build
automático; retorno ao automático sem trocar o cache; remoção e restauração por
id e por caminho; kit do preset anotado; e conferência adiada com troca de
workspace. Fontes, presets, `.clangd` e `compile_flags.txt` conservam seus bytes.
Antes do configure, `index.context` já informa a escolha com origem `kit`;
depois, origem `cdb`.

Doze mutantes compiláveis eliminados com baseline verde: recusa no configure
explícito e no build, repetição nos dois produtores, kit anotado no configure e
no build, leitura do kit registrado, kit padrão da receita, instalação ausente,
efetivo de id ausente, sonda C e guarda de workspace. Um mutante que removia a
gravação do registro não compilou e não conta como prova. No harness QML, o
seletor mostra o caminho fixado e “(ausente)” quando o core não o resolve; a
mutação desse rótulo foi eliminada.

Na UI nativa isolada, com `autoSave=false` confirmado pelo IPC, um projeto C
existente em caminho com espaços/Unicode recebeu Clang pelo seletor e depois
GCC por “Personalizado…” em `instalações C/gcc 16/bin/gcc`. Caminho inexistente
foi recusado na prévia, sem gravar o kit; a prévia não gravou e “Ativar” aplicou.
Antes do configure, o Project Health apontou a divergência entre kit e build;
configurar e compilar pelo menu Build repetiram o configure após recriar o
cache, e a CDB, `index.context` e a barra de status passaram a Clang e depois ao
GCC escolhido, com `-std=gnu17`. O menu da toolchain mostra o caminho fixado e
“(ausente)” depois da remoção. A prova expôs que a recusa síncrona de
`cmake.configure` só chegava ao log da UI; agora o Project Health mostra o motivo
e “Tentar de novo”, e o aviso sai no configure seguinte bem-sucedido. Restaurar
o mesmo caminho recuperou configure, CDB e contexto. Trocar de arquivo pela árvore
e reabrir a IDE conservaram o texto não salvo e o indicador; o marcador seguiu
ausente do disco, e fontes, `CMakeLists.txt`, `.clangd` e `compile_flags.txt`
mantiveram seus bytes. Duas mutações QML dessa recusa foram eliminadas.

Limites: o seletor de toolchain continua aceitando só ids do catálogo; o
caminho entra pela receita com prévia. A lista sugere o primeiro caminho de cada
binário real (`/usr/bin/cc` para o GCC do sistema). O kit padrão exibido no menu pode
diferir do kit registrado até a tela escolher o preset. A receita trata o
compilador C; C++ e geradores conservam o comportamento anterior, e o
executável do papel `cmake` ausente continua resolvido pelo `PATH`. Não há
verificação do conteúdo do binário no mesmo caminho, de SDK/libc ou de flags
de toolchain files além da divergência anunciada. C1-07, consulta de versão
auxiliar bloqueada antes do configure e a matriz nativa Windows seguem abertos.
Gate rápido aprovado em 289 s: 1468 Rust, Clippy, 51 arquivos C++, 8 CTest e
180 harnesses QML; um NÃO PROVADO (debugpy ausente). Formatação e gates
documentais acompanham o fechamento, sem ampliar baselines, allowances ou
exclusões Windows. C1-11–C1-14 têm correção e provas localizadas; C1 permanece aberto.

## 16. C1: CDB de configurações existentes e coerência com o clangd — desenho de 2026-11-09

Inventário localizado de C-05 ([62 §3.4](../roadmaps/62-base-publica-0.4.md#34-inventário-sistemático-de-dívida-técnica))
antes da mudança, nos donos `cdb.rs`, `cmake/`, `lsp/server.rs` e
`index/context/`. As reproduções usam CMake 4.4.5, GCC 16.2.1, Clang 23.1.4 e
clangd 23.1.4 reais, com o core do commit b1d22bdb consultado por IPC:

| ID / capacidade | Evidência e causa | Severidade / impacto | Decisão e prova necessária |
| --- | --- | --- | --- |
| C1-15 / CDB de preset configurado fora da IDE | Fixture com presets `debug` e `release` (`binaryDir` `${sourceDir}/build/<nome>`), configurada por `cmake --preset debug`: `cmake.status` sem `cdbDirectory`, `index.context` orienta "configure o projeto (cmake)" e `clangd --check` não acha base. Neste checkout as três CDBs ficam em `build/<preset>/`, e nenhuma está nas cinco pastas da lista. Causa confirmada: `cdb::DIRETORIOS` é fixa e não lê o `binaryDir` dos presets | Alta: um projeto configurado por presets, CLI ou outra IDE perde o contexto C/C++ e é levado a configurar de novo; o configure automático ao abrir reconfigura em `.kinein/build` | Corrigir em `cdb.rs`, com o `binaryDir` resolvido pelo dono dos presets. Uma candidata é usada e mostra a origem. Com várias e nenhuma escolha registrada, a IDE pergunta e não escolhe (autor, 2026-11-09). Provar com GCC e Clang reais: uma e várias candidatas, escolha, troca, retorno ao automático e à pasta da IDE, CDB obsoleta ou removida, preset removido, macros, caminho com espaços/Unicode e mutações |
| C1-16 / CDB em uso divergente do clangd | Com CDB só em `builddir/`, `cmake.status` responde `cdbDirectory: builddir`, e `clangd --check` não a acha: `point_clangd_at_cdb` só passa `--compile-commands-dir` para `.kinein/build`. Por leitura, um servidor que subiu com a CDB da raiz ou de `build/` não reinicia quando `.kinein/build` passa à frente, porque `cdb_at_start` é booleano. Medido: `--compile-commands-dir` prevalece sobre o `CompileFlags.CompilationDatabase` do `.clangd`, que `cdb::status` ignora | Alta: o contexto da IDE e o clangd descrevem bases diferentes sem aviso | Corrigir: um dono decide a CDB em uso, e o clangd recebe exatamente essa pasta e reinicia quando ela muda. O `.clangd` que fixa a base vence (autor, 2026-11-09). Provar o argv do clangd e um diagnóstico real que dependa da CDB |
| C1-17 / codemodel de outro build dir | `CompileContext` lê o codemodel de `.kinein/build` sem conferir a pasta da CDB: a referência de header e as unidades do file-api podem juntar alvos de uma configuração e comandos de outra. Confirmada por inspeção | Média: hoje só com CDB externa e configure anterior da IDE; C1-15 torna o caso comum | Corrigir: o codemodel vem da pasta da CDB em uso; sem CDB, de `.kinein/build` como antes. Provar com as duas pastas configuradas, com mutação |
| C1-18 / `include` dos presets | `list_presets`, `default_preset` e a leitura de campos com herança não seguem `include` (schema 4+): um preset de arquivo incluído não é listado, escolhido nem resolvido. Confirmada por inspeção | Média: um projeto que divide presets por `include` aparece sem eles | Adiar para C-04, que reescreve a leitura de presets do configure; a dependência é a mesma leitura. Até lá, limite explícito: candidata de arquivo incluído não aparece. Retomar no C-04, com prova de `include` aninhado, ciclo e `${fileDir}` |

Estado inicial: C1-15 e C1-16 reproduzidos; C1-17 e C1-18 confirmados por
inspeção. C-04 (configure no `binaryDir`, multi-config) é a fatia seguinte, e
o autor decidiu em 2026-11-09 que o configure da IDE com preset que declara
`binaryDir` usa essa pasta. Esta fatia não muda o diretório do configure.

**CDB em uso.** Um só dono, `cdb::status`, responde qual CDB vale, com origem,
nesta ordem:

1. `.clangd` da raiz que declara `CompileFlags.CompilationDatabase`. Na forma
   simples (um só fragmento com a chave, sem `If`, valor escalar diferente de
   `Ancestors` e `None`), a pasta declarada, relativa à raiz, é a CDB em uso,
   com origem `clangd`, e a IDE não passa `--compile-commands-dir`. Nas outras
   formas a base é manual: a IDE não passa a opção, não atribui CDB e explica o
   motivo, como a referência de header já faz.
2. A configuração escolhida pela pessoa (`.kinein/configuration.json`, com
   `{ "preset": "<nome>" }`). Vale `.kinein/build` se o último configure da IDE
   usou esse preset e deixou CDB; senão, o `binaryDir` resolvido dele. Sem CDB
   ali, preset removido ou registro ilegível, não há CDB em uso e o motivo
   aparece; outra candidata nunca a substitui.
3. `.kinein/build`, do configure da IDE.
4. A raiz e `build/`, que o clangd também acharia sozinho.
5. O `binaryDir` dos configure presets não ocultos que a `condition` não exclui
   no host. Uma pasta com CDB é usada, com origem `preset`. Com várias, nenhuma
   é usada: `cdbAbsence` com `kind: "choiceRequired"` e a lista vão para a UI.
6. `builddir/` e `out/build/`.

Uma pasta das listas 3, 4 e 6 que coincide com o `binaryDir` de um preset leva o
nome dele. Todas as pastas com CDB entram em `cdbCandidates`, sem duplicar pelo
caminho canônico. Obsolescência e configure incompleto seguem as regras
anteriores; o configure incompleto vale só para `.kinein/build`.

**clangd.** O servidor cpp sobe com `--compile-commands-dir` da CDB em uso, também
para a raiz e `build/`: com isso a busca do clangd não acha, num subdiretório,
uma base que a IDE não mostra. Com `.clangd` decidindo, ou sem CDB em uso, a opção
não é passada. O handle guarda a pasta com que subiu. Se a CDB em uso passa a
outra pasta (escolha, configure, CDB que aparece), o servidor reinicia: a opção
só vale na partida. Reiniciar descarta o índice do clangd, e os documentos
reabrem com o buffer do editor pelo fluxo `restarted` existente.

**Escolha.** `cmake.selectConfiguration { preset }` grava ou apaga o registro.
Um nome precisa estar entre os configure presets não ocultos; `null` volta ao
automático. Responde com o `cmake.status` atual, recarrega o contexto e reinicia
o clangd quando a pasta muda. O configure sem preset pedido usa a configuração
escolhida antes do preset anotado no build dir (`presetSource:
"configuration"`), com o kit de mesmo nome, como um preset pedido. Um configure
bem-sucedido da IDE, explícito ou do build, apaga o registro, e `.kinein/build`
volta a valer. No C-04, esse registro passa a ser a configuração ativa
do configure.

**Presets.** A leitura de campos com `inherits` sai de `toolchain/selection.rs`
para `cmake_presets.rs`, que serve o `toolchainFile` e o `binaryDir`. O
`binaryDir` expande as macros de cmake-presets(7): `${sourceDir}`,
`${sourceParentDir}`, `${sourceDirName}`, `${presetName}`, `${generator}`,
`${hostSystemName}`, `${fileDir}`, `${dollar}`, `${pathListSep}`,
`$env{…}` (campo `environment` do preset com herança, depois o ambiente do
core; ausente expande vazio) e `$penv{…}`. `$vendor{…}`, macro desconhecida,
ciclo e `generator` ausente deixam o preset sem pasta resolvida, sem
candidata. Caminho relativo vale a partir da raiz.

**Interface.** O Project Health pergunta quando há várias candidatas, com um item
por pasta e preset, e escolher não reconfigura. O configure automático ao abrir
não roda quando já há CDB em uso ou escolha pendente. A origem aparece no
contexto do arquivo (`compile_commands.json em build/debug (preset debug)`), e
o motivo da ausência substitui a dica de configurar.

**Limites declarados.** O `binaryDir` é resolvido com o ambiente do core, que
pode diferir do terminal ou da IDE que configurou. O `.clangd` lido é o da raiz;
o de um subdiretório é sobreposto pela opção que a IDE passa. A configuração do
usuário do clangd não é lida. `include` de presets fica em C1-18. A CDB com uma
unidade por arquivo continua em C-03 (C2), e o diretório do configure em C-04.

### 16.1 Integração e alcance das provas — 2026-11-09

Protocolo 0.206.0, produto 0.3.9. `cdb::status` decide a CDB em uso na ordem
acima, e `cdb/clangd.rs` lê a forma simples do `.clangd` da raiz. A leitura
dos presets virou o módulo-folha `cmake_presets`: lista, preset padrão,
`condition`, campos com `inherits` e `binaryDir`. Saiu de
`toolchain/selection.rs` e de `cmake/mod.rs`, e assim `cmake`, `toolchain` e
`cdb` a leem sem ciclo entre `cmake` e `toolchain`. Corrigido também: um pai
de `inherits` que desfaz o campo (`null`) vence os pais seguintes, como o
CMake 4.4.5 faz. Antes, o leitor pulava para o pai seguinte. `cmake/configuration.rs`
grava `.kinein/configuration.json`. O configure guarda a escolha do início e,
no sucesso, só apaga um registro que ninguém trocou durante o produtor. O
servidor cpp guarda a pasta com que subiu e reinicia quando a pasta em uso
muda. O contexto lê o codemodel da pasta da CDB e mostra a origem. A ausência
vai no fio como `cdbAbsence { kind, reason }`, com `kind` `choiceRequired`,
`chosenConfiguration` ou `clangd`; só o primeiro pede escolha. Na UI, o
sinal `cmakeStatusResolved` leva o mapa `compileDatabase`. O Project Health
pergunta pelas candidatas, e o item de menu (`cmake.configuration:<preset>`)
chama `cmake.selectConfiguration` e põe o kit no mesmo preset, sem
reconfigurar. O configure automático e o aviso de modo degradado não aparecem
com CDB em uso. A tabela de ferramentas exigidas saiu para
`ProjectToolRequirements.qml`, sem estado, porque um objeto filho com
bindings lia a lista atrasada no `onToolsListChanged`.

Quatro regressões de dispatch (`tests/c_project/existing_cdb.rs`) usam CMake
4.4.5, GCC 16.2.1, Clang 23.1.4 e clangd 23.1.4 reais. Os presets são
configurados por `cmake --preset` fora da IDE, com `binaryDir` por
`${presetName}` e fora da raiz por `${sourceParentDir}`, `${sourceDirName}` e
`$env{}` do preset, com espaços e Unicode. O clangd real roda atrás de um
encaminhador que grava o argv. Cada CDB tem assinatura positiva: Debug dá um
aviso e um erro, Release só o aviso dela. Publicação vazia não prova nada,
porque o core limpa os diagnósticos do clangd que mata; a primeira versão do
teste aceitava essa limpeza, e a contagem de partidas acusou. Cobrem:

- uma candidata, com origem, compilador, defines, texto da fonte e argv;
- duas candidatas: escolha pedida, dica com o motivo e lista ordenada;
- escolha sem configure: reinício do servidor vivo, uma partida nova, contexto
  e diagnósticos de Release, e codemodel da mesma pasta (C1-17);
- recusa de preset inexistente sem gravar, `null` e nova pergunta;
- CDB escolhida obsoleta, com o culpado;
- configure da IDE: `presetSource: "configuration"`, registro apagado,
  `.kinein/build` de volta e clangd reiniciado nela;
- `.clangd` simples vence sem a opção, com escolha registrada; `If` passa a
  base manual com motivo;
- CDB de `builddir/` que chega ao clangd (C1-16);
- preset escolhido removido dos presets: o motivo aparece e o configure é
  recusado, em vez de trocar a escolha pelo único preset restante.

Testes unitários cobrem `inherits` com `null` medido, macros, `$env`/`$penv`,
caminhos absolutos e `CMakeUserPresets.json`, as formas do `.clangd`, a
precedência, o registro e o instantâneo do configure (escolha feita durante o
produtor sobrevive; registro ilegível fica). Dezesseis mutantes compiláveis
do core e seis do QML foram eliminados com baseline verde; dois deles trocam o
`kind` da ausência. O harness
`tst_cdb_choice` cobre a faixa, o menu e o configure automático.

Na UI nativa isolada (Xvfb, `autoSave=false` confirmado pelo IPC), o projeto
`projeto presets ação/` foi configurado pelo CLI nos dois presets. A faixa
pediu confiança e depois mostrou o motivo do core com "Escolher
configuração…", sem configure automático. O menu listou as duas pastas, e
escolher clang-release enviou `cmake.selectConfiguration` e `toolchain.get`
com o preset, sem `cmake.configure`. O clangd vivo recebeu
`--compile-commands-dir` com a pasta de fora da raiz. `main.c` mostrou só o
aviso de Release e o bloco de Debug inativo, e a barra de status passou a
`contexto: clang · 0 -I · 2 -D`. Reaberta, a IDE conservou a escolha, e a
confiança não disparou configure. Fontes, `CMakeLists.txt` e
`CMakePresets.json` mantiveram seus bytes, e `.kinein/build` não foi criado.
Repetida com os binários finais e `main.c` já aberto: o clangd vivo, sem a
opção, mostrava só o `#error`. A escolha o reiniciou com a pasta de Release, e
o documento aberto passou ao aviso de Release sem configure. Na reabertura, o
clangd já subiu nessa pasta.

Limites desta fatia: o Project Health só oferece a escolha quando ela é
obrigatória. Trocar uma configuração já escolhida ou voltar ao automático
pela UI fica para o C-04, junto do seletor da configuração ativa. No menu, a
pasta longa fora da raiz é cortada no fim. Valem também os limites declarados
em §16 e C1-18. C-05 tem correção e provas localizadas; C1 permanece aberto
com C-04, C1-07, origem por toolchain file e `CC` do ambiente e consulta de
versão bloqueada.

## 17. C1: configure no `binaryDir`, configuração ativa e `include` de presets — desenho de 2026-11-09

Inventário localizado de C-04 ([62 §3.4](../roadmaps/62-base-publica-0.4.md#34-inventário-sistemático-de-dívida-técnica))
e de C1-18 (§16) antes da mudança, nos donos `cmake/`, `cmake_presets.rs`,
`cdb.rs` e nos consumidores da pasta de build. As medições usam CMake 4.4.5 e
GCC 16.2.1 reais, com o core do commit 3c224e9b:

| ID / capacidade | Evidência e causa | Severidade / impacto | Decisão e prova necessária |
| --- | --- | --- | --- |
| C1-18 / `include` dos presets | Como em §16. Medido: o CMake segue `include` aninhado, relativo ao arquivo que inclui, aceita o mesmo arquivo duas vezes e recusa **todos** os presets com ciclo ("Cyclic include among presets files"), arquivo ausente ("File not found"), `include` em versão menor que 4 e nome repetido ("Duplicate preset"). Macros no caminho: nenhuma até a versão 6, só `$penv{}` nas 7–8, todas menos `$env{}` e as do preset (`presetName`, `generator`) da 9 em diante; outra macro recusa ("Invalid \"include\" field"). `${fileDir}` no `binaryDir` herdado de outro arquivo vale a pasta do arquivo que define o preset em uso, nas versões 6 e 12 | Média: um projeto que divide presets por `include` aparece sem eles, e o configure o reprovaria com outra mensagem | Corrigir em `cmake_presets.rs`, com as mesmas recusas e o motivo. Provar com aninhamento, inclusão repetida, ciclo, ausência, versão, duplicata, macros por versão e `${fileDir}`, e com `cmake --preset` real resolvendo a mesma pasta que a IDE |
| C1-19 / configure fora do `binaryDir` | `configure_command` (`cmake/mod.rs`) passa `-B .kinein/build` também com `--preset`. Medido: `cmake --preset top -B ide` configura em `ide/`, e o `binaryDir` do preset fica vazio | Alta: a IDE e o terminal configuram pastas diferentes para o mesmo preset, e trocar de preset reconfigura a mesma pasta | Configurar na pasta da configuração (decisões abaixo). Provar com dois presets de `binaryDir` distinto, troca e retorno sem reconfigurar o outro, e o argv do CMake gravado |
| C1-20 / registros da IDE numa pasta só | A tentativa pendente, a trava, o kit, o preset anotado e o perfil de rigor (`cmake/attempt.rs`, `cmake/rigor.rs`, `cmake/mod.rs`) moram em `.kinein/build`; o sucesso apaga `.kinein/configuration.json` (C-05) | Alta: o configure de uma configuração invalidaria os registros de outra | Os registros vão para a pasta de cada configuração, e o registro de `.kinein/configuration.json` passa a ser a configuração ativa. Provar a tentativa incompleta de uma configuração sem afetar a outra, com mutações |
| C1-21 / consumidores fixos em `.kinein/build` | Build (`build/cmake.rs`), run (`run.rs`), debug (`dap/target.rs`), ctest (`test/mod.rs`), gravação (`project/flash_origin.rs`), qmlls (`handlers/qml.rs`), contexto (`index/context/mod.rs`), ações de configuração (`configaction/builddir.rs`) e `toolchain/project.rs` usam `cmake::build_dir` ou o caminho literal. Confirmada por leitura | Alta: com o C-19 corrigido, compilar e executar sairiam de outra pasta que a do contexto | Um dono, a configuração ativa, para todos. Provar build, run, debug e ctest reais na pasta do preset, e o contexto e o Project Health na mesma pasta |
| C1-22 / multi-config | `CmakeModel` lê só `/configurations/0` (`cmake/model.rs`); `cmake --build` vai sem `--config`; `collect_executables` percorre a pasta inteira. Medido com Ninja Multi-Config: três configurações no codemodel, três entradas da CDB para o mesmo arquivo, o clangd usa a primeira (Debug), e `cmake --build` sem `--config` compila `CMAKE_DEFAULT_BUILD_TYPE`, senão a primeira; após dois builds há `Debug/app` e `Release/app` | Média: com `CMAKE_DEFAULT_BUILD_TYPE` diferente, o build compila uma configuração e o contexto mostra outra; o run recusa por achar dois executáveis | Usar a configuração que o clangd vê (decisão abaixo). Provar com Ninja Multi-Config real, `CMAKE_DEFAULT_BUILD_TYPE=Release` e mutações |
| C1-23 / UI sem troca da configuração | O Project Health só oferece "Escolher configuração…" quando a escolha é obrigatória; o menu corta no fim a pasta longa fora da raiz (§16.1) | Média: trocar a configuração ativa ou voltar ao automático exige editar `.kinein/configuration.json` | Menu de configuração sempre que há presets, com a ativa marcada e "Automático". Provar na UI nativa isolada |
| C1-24 / macros dos presets só sob demanda | Achado na prova do C1-23 (2026-11-09). `cmake_presets` expande só o `binaryDir`, e só do preset usado. Medido com `cmake --list-presets` e `cmake --preset` do CMake 4.4.5 (§17.4): cada preset de cada tipo, mesmo oculto e sem uso, é expandido ao ler os arquivos, com a herança e na versão do arquivo que o define. Uma macro desconhecida, sem fechamento, `$env{}` ou `$penv{}` sem nome, uma macro de versão maior (`${hostSystemName}` 3, `${fileDir}` 4, `${pathListSep}` 5) ou um ciclo no `environment` recusa o arquivo inteiro ("Invalid macro expansion"). Nos configure presets valem `binaryDir`, `installDir`, `toolchainFile`, `graphviz`, os valores de `cacheVariables` e de `environment` e as cadeias da `condition`; nos build presets, `environment`, `targets`, `nativeToolOptions` e `condition`, com o `environment` do configure preset; nos test presets, `environment`, `overwriteConfigurationFile`, `output`, `filter`, `execution` e `condition`; nos package presets, `environment`. `$vendor{}` tira da lista só o preset que o usa. Divergências vizinhas: `${generator}` sem `generator` expande vazio, e a IDE recusava a pasta; no build e no test preset, `${generator}` é o do configure preset, e no package preset é sempre vazio (achado pelos mutantes); na versão 12, `${fileDir}` é a pasta do arquivo que escreveu o texto quando todas as macros dele são `${fileDir}`, e a IDE usava a do preset em uso, então o `-B` da IDE e o `cmake --preset` configuravam em pastas diferentes. A `condition` é herdada, e o CMake a avalia ao listar: o preset de condição falsa some do `--list-presets`. A IDE lia só a do próprio preset e dava por verdadeira toda forma que não fosse do host. `environment: null` tira a chave, e `$env{}` cai no ambiente do processo; a IDE expandia vazio. `$vendor{}` interrompe a expansão do preset: o que vem depois, na ordem do CMake, não é conferido. A ordem é `environment`, `condition` e os campos do tipo, com as chaves dos mapas em ordem. A toolchain mostrava `CMAKE_C_COMPILER` e `toolchainFile` crus, com as macros | Média: a IDE lista, escolhe e oferece no menu presets de um arquivo que o CMake recusa, e o configure falha no CMake com a mensagem dele, não em silêncio; com `${fileDir}` na versão 12, a IDE configura fora da pasta do preset | Corrigido em 2026-11-09 (§17.4). Decisões do autor de 2026-11-09: corrigir antes do C1-22, e em todos os tipos de preset. Corrigir em `cmake_presets`, com o motivo do CMake. A `condition` segue o CMake em todas as formas menos `matches` e `notMatches`, que dependem da expressão regular do CMake (C1-26) e continuam valendo como verdadeiras. Provar com o CMake real cada campo, o preset oculto, a versão por arquivo, o ciclo e as pastas do `${fileDir}` |
| C1-25 / esquema dos presets | Achado na medida do C1-24 (2026-11-09, CMake 4.4.5). O CMake recusa o arquivo inteiro por um campo que o esquema não tem ("Invalid extra field") e por um campo de versão maior (`installDir` e `toolchainFile` 3, `graphviz` 10: "File version must be N or higher"); um `generator` desconhecido tira o preset da lista, e o configure dele falha. Também recusam o arquivo um `configurePreset` inexistente (`Invalid "configurePreset"`), uma `condition` sem campo obrigatório (`Missing required field "rhs"`) e uma de tipo desconhecido (`Invalid condition`). A IDE lê só os campos que usa e lista esses presets; na `condition`, o campo ausente vale vazio, e o tipo desconhecido, verdadeiro | Média: o mesmo efeito do C1-24 por outra causa | Corrigido em 2026-11-10 (§18.1). Decisão do autor de 2026-11-09: corrigir depois do C1-24 e antes do C1-22. Decisões do autor de 2026-11-10: perguntar ao CMake do projeto, o que o kit padrão do workspace fixa, senão o do `PATH` (desenho no §18). A IDE roda o `cmake --list-presets` desse CMake, com cache pela data dos arquivos de presets e do executável, e a recusa e a lista dele valem: o esquema muda com a versão do CMake. A leitura nativa do C1-24 fica para os valores e para quando não há CMake. Com isso a lista do CMake já reflete `matches` e `notMatches`, e ao C1-26 resta o caso sem CMake. Provar com o CMake real cada recusa medida, o cache e a ausência do CMake |
| C1-26 / expressão regular da `condition` | Achado na medida do C1-24 (2026-11-09, CMake 4.4.5). `matches` e `notMatches` usam a expressão regular do próprio CMake: `\d` não é classe, e `^a\d$` não casa `a1`. Uma expressão que não compila recusa o arquivo inteiro ("RegularExpression::compile(): Error in compile"). A IDE dá essas formas por verdadeiras | Baixa: um preset de `matches` falsa aparece na lista, e uma expressão inválida não recusa o arquivo | Aberto. Decisão do autor de 2026-11-10: perguntar ao CMake. Só quando uma `condition` usa `matches` ou `notMatches`, a IDE roda o `cmake --list-presets` do projeto, com cache pela data dos arquivos, e usa a lista ou a recusa dele. Sem CMake, essas formas continuam verdadeiras, e a IDE diz isso. Vem depois do C1-25 e antes do C1-22 (autor, 2026-11-10). Provar com o CMake real a expressão que casa, a que não casa, a que não compila e a ausência do CMake |
| C1-27 / preset oculto pedido pelo nome | Achado pelos mutantes do C1-25 (2026-11-10): o mutante que dava aos presets ocultos o motivo de fora da lista sobreviveu, porque nenhum teste decidia o caso. Medido com o CMake 4.4.5: `cmake --preset h`, com `h` oculto, sai com 1 e "Cannot use hidden configure preset", e o `cmake --list-presets` não o lista. A IDE resolve a pasta, o configure pelo nome começa o job e o CMake recusa depois. Causa: `Reading::unusable` não conhece o oculto, e só a lista o filtra | Baixa: só quem pede o preset pelo nome, pelo IPC ou por uma configuração registrada que ficou oculta; o configure falha com a mensagem do CMake | Corrigido em 2026-11-10 (§18.1). Decisão do autor de 2026-11-10: corrigir dentro do C1-25, no mesmo protocolo 0.212.0. A leitura nativa anota os configure presets ocultos, e o preset oculto pedido pelo nome é recusado antes do job, com ou sem CMake: "o preset h é oculto: o CMake não o usa pelo nome". Vale para a pasta, o configure e os valores da toolchain; um filho visível continua herdando dele. Provar pela leitura nativa e com o CMake real, de ponta a ponta sem job, e com o mutante da guarda |

**Decisões do autor em 2026-11-09.** O configure de um preset que declara
`binaryDir` usa essa pasta (sessão anterior). A configuração pedida passa a ser
a ativa quando o configure começa. Com gerador multi-config, a IDE usa a
configuração que o clangd lê. O cache da pasta do preset recebe as definições da
IDE, e a IDE mostra isso. Um preset sem `binaryDir` configura em
`.kinein/presets/<preset>`. Reparar o build dir remove a pasta da configuração
ativa: a da IDE como antes, e a de um preset só quando o `CMakeCache.txt` dela
tem a raiz do projeto em `CMAKE_HOME_DIRECTORY` e ela não é a raiz nem a
contém, com o caminho absoluto na prévia.

**Presets (C1-18).** A leitura segue `include` como o CMake: o caminho
relativo vale a partir do arquivo que inclui. Um arquivo incluído também inclui,
e um arquivo já lido não é lido de novo. `CMakeUserPresets.json` inclui
`CMakePresets.json` mesmo sem `include`. As macros do caminho seguem a versão do
arquivo que inclui. Ciclo, arquivo ausente ou inválido, `include` antes da
versão 4, macro não aceita, nome repetido, pai inexistente ou fora do alcance
do arquivo do filho e herança cíclica recusam todos os presets com o motivo
("Could not find inherited preset" e "Cyclic preset inheritance" no CMake): `cmake.presets.list` falha, e não há padrão nem candidata de preset.
A ordem da lista não muda: `CMakePresets.json` e o que ele inclui, depois
`CMakeUserPresets.json` e o que só ele inclui; os presets de cada arquivo vêm
antes dos que ele inclui. O padrão (52 §14.2) considera do lado do usuário os
presets alcançados só por `CMakeUserPresets.json`. `${fileDir}` vale a pasta do
arquivo que define o preset em uso.

**Pasta da configuração (C1-19).** `cmake::configuration_dir` responde:
- sem preset, `.kinein/build`;
- preset com `binaryDir`, a pasta resolvida, passada ao CMake com `-B`. A query
  do file-api precisa estar na pasta antes do configure, e o `-B` com o mesmo
  caminho garante que o CMake configure onde a IDE lê;
- preset sem `binaryDir`, `.kinein/presets/<preset>`. Nome vazio, `.`, `..` ou
  com separador de caminho é recusado com o motivo;
- `binaryDir` não resolvido (§16) recusa antes do CMake, que também recusaria.

Uma pasta fora da raiz vale, como no C-05. Antes do comando, a saída do
configure diz em que pasta ele roda e, numa pasta de preset, que o cache recebe
as definições da IDE e que o terminal as herda.

**Configuração ativa (C1-20).** `.kinein/configuration.json` passa a registrar
a configuração ativa. O configure grava o preset pedido antes do produtor, ou
apaga o registro quando roda sem preset; o sucesso não o apaga mais. A
tentativa pendente, a trava, o kit, o preset anotado e o perfil de rigor moram
na pasta da configuração. Sem registro, vale o automático:
1. `.kinein/build` já configurado pela IDE, com o preset anotado nele, inclusive
   o de antes do C-04;
2. senão, o preset padrão único;
3. senão, nenhum preset, em `.kinein/build`.

Com vários presets e nenhuma escolha, a IDE pergunta, como no configure.
Escolher pela UI grava o registro sem configurar (C-05), e `null` volta ao
automático. Na regra 2 do §16, a CDB em uso é a da pasta da configuração
escolhida. A exceção do `.kinein/build` anotado com o mesmo preset deixa de
existir: esse `.kinein/build`, de antes do C-04, continua valendo no automático.

**Consumidores (C1-21).** Build, run, debug, ctest, gravação, qmlls, contexto,
ações de configuração e `toolchain/project.rs` leem a configuração ativa. Sem
configuração ativa, cada um recusa com o motivo do configure.

**Multi-config (C1-22).** A configuração usada é a primeira de
`CMAKE_CONFIGURATION_TYPES` no cache, a mesma que o clangd lê da CDB. Build e
ctest passam `--config`, e o codemodel é lido por nome. O run e o debug procuram
o executável nos artefatos dessa configuração. Quando `CMAKE_DEFAULT_BUILD_TYPE`
é outra, o status traz as duas, e a faixa mostra a diferença. Trocar de
configuração numa pasta multi-config fica para o C-03, que trata da CDB 1:N.

**Interface (C1-23).** O menu de configuração tem as configurações, com a
ativa marcada, e "Automático". A pasta longa é cortada no meio, e o caminho
inteiro fica na dica. Escolher não configura, como no C-05. A faixa do Project
Health some quando está tudo bem. Por isso o menu mora num chip do cabeçalho,
ao lado do da toolchain, por decisão do autor de 2026-11-09. O chip aparece
num projeto CMake com configure presets ou com configuração registrada e
mostra a configuração ativa. Sem espaço, ele vai para o "⋯", como os outros
chips. A faixa usa o mesmo menu em dois casos: quando a escolha é obrigatória
e quando a configuração ativa é recusada sem presets a escolher. O contrato
do IPC é escrito no [03 cmake](../arquitetura/03-dominios/cmake.md) antes da
mudança, com nova versão do protocolo.

Fatias, cada uma com commit próprio: C1-18; C1-19–C1-21 com o IPC; C1-23;
C1-24; C1-25 com o C1-27; C1-26; C1-22. O autor pôs o C1-23 antes do C1-22 em 2026-11-09, porque sem
presets e com o registro órfão a UI não tinha saída (§17.2). No mesmo dia pôs o
C1-24, achado na prova do C1-23, também antes do C1-22: o menu novo listava
presets de um arquivo que o CMake recusa. O C1-25, achado na medida do C1-24,
vem logo depois dele. Em 2026-11-10 o autor pôs o C1-26 depois do C1-25 e antes
do C1-22: o menu não deve oferecer um preset que o CMake tira da lista. No
mesmo dia pôs o C1-27, achado pelos mutantes do C1-25, dentro do C1-25.

### 17.1 C1-18: integração e provas — 2026-11-09

`cmake_presets.rs` foi dividido por responsabilidade. `files.rs` lê os arquivos
com `include` e recusa o que o CMake recusa. `macros.rs` expande as macros com
duas políticas, `binaryDir` e `include`. O módulo-raiz guarda a lista, o
padrão, a herança e o `binaryDir`. A lista, o padrão, os campos herdados
(`toolchainFile` e o compilador do preset) e as pastas candidatas da CDB vêm da
mesma leitura. O padrão ganhou a recusa com o motivo: o `cmake.status` não
oferece escolhas nesse caso, o configure sem preset pedido é recusado, e uma
configuração escolhida mostra o motivo em `cdbAbsence`. Protocolo 0.207.0, sem
campo novo.

Além do `include`, muda o seguinte: um `CMakeUserPresets.json` inválido, um pai
inexistente e a herança cíclica recusam todos os presets, como no CMake. Antes,
o padrão ignorava o arquivo inválido, e o pai inexistente e o ciclo deixavam o
campo ausente.

Provas:
- testes do dono: aninhamento com caminho relativo, `./` e `../`, arquivo
  incluído duas vezes e lido uma, e `${fileDir}` do arquivo do preset em uso;
  doze recusas com o motivo e sem lista, padrão, pasta nem campo; macros por
  versão; lado do usuário, com o `include` implícito só no arquivo do usuário;
- regressão com `cmake --preset` real, GCC 16.2.1 e Clang 23.1.4: dois presets
  configurados fora da IDE, um do arquivo incluído e outro da raiz, herdando o
  `binaryDir` com `${fileDir}`. A pasta vem da saída do próprio CMake. A IDE
  lista os dois, acha as duas CDBs nessas pastas e aceita a escolha. Sem o
  arquivo incluído, a lista falha com o motivo, o CMake também recusa, e a
  configuração escolhida mostra o motivo;
- vinte mutantes reprovam, entre eles o `include` ignorado, o caminho a partir
  da raiz, o ciclo, a versão, a duplicata, o pai fora do alcance, o pai
  inexistente, a herança cíclica, o lado do usuário, o `${fileDir}`, as macros
  por versão e o motivo escondido no configure e na CDB.

Limites: a ordem da lista não é a do `cmake --list-presets`, que põe o arquivo
do usuário primeiro. Os build e test presets não são lidos, então um nome
repetido entre eles não recusa. Campos de tipo errado e campos obrigatórios por
versão seguem sem validação própria: o CMake recusa no configure, com a
mensagem dele. O teste de serial que reprovou sem o `TMPDIR` do gate é de outra
causa, registrada como INF-05 no 62 §3.4.

### 17.2 C1-19–C1-21: integração e provas — 2026-11-09

O dono novo é `cmake_configuration.rs`, uma folha ao lado de `cmake_presets`.
Ele guarda a pasta de cada configuração, a configuração ativa e os registros
que a IDE deixa em cada pasta. Fica fora de `cmake` porque `toolchain` lê a
configuração ativa, e `cmake` depende de `toolchain`. O `cmake/configuration.rs`
saiu, e o registro `.kinein/configuration.json` do C-05 passou a ser o da
configuração ativa. O `handlers/cmake.rs` passou do limite de tamanho e foi
dividido: o `cmake.status` e o `cmake.selectConfiguration` foram para
`handlers/cmake_configuration.rs`. Protocolo 0.209.0, com o contrato no
[03 cmake](../arquitetura/03-dominios/cmake.md).

O configure da IDE passa `-B` com a pasta da configuração. Essa pasta é o
`binaryDir` resolvido, que o `-B` antes sobrepunha, ou `.kinein/presets/<preset>`
para um preset sem `binaryDir`. Um nome que não serve de pasta é recusado antes
de qualquer processo. A trava, a tentativa pendente, o kit, o preset e o rigor
ficam na pasta, e cada configuração conserva o seu estado. Duas configurações
diferentes podem configurar ao mesmo tempo; a mesma, não. O configure grava a
configuração ativa ao começar, depois da trava, e o sucesso não a apaga mais. A
saída diz em que pasta o configure roda. Numa pasta de preset, diz também que o
cache recebe as definições da IDE e que `cmake --preset` no terminal as herda.

Build, run, debug, ctest, flash, `qmlls`, o contexto do índice, o compilador do
projeto, as ações do Project Health e os artefatos leem a pasta da configuração
ativa. Sem configuração ativa, recusam com o motivo, sem voltar a
`.kinein/build`. O build sem cache configura na pasta do preset. Um
`.kinein/build` de antes do C-04 vale no automático até o próximo configure. Na
CDB, a configuração escolhida usa a pasta dela, sem a exceção do `.kinein/build`
anotado, e uma tentativa pendente deixa obsoleta a CDB da pasta em que está.
`cmake.selectConfiguration` recarrega o contexto, o clangd e o `qmlls` quando a
CDB em uso ou a pasta ativa muda.

Três achados da integração foram corrigidos no dono:
- um registro órfão (o preset saiu dos presets, ou o registro está inválido)
  recusava sem nada a escolher. Agora o `cmake.status` oferece as escolhas do
  automático em `presetChoices`;
- o contexto de uma pasta sem CDB dizia "Comando anterior apenas para inspeção",
  sem comando anterior;
- presets com o mesmo `binaryDir` dividem a pasta. A tentativa pendente e o kit
  do último sucesso de um passavam a valer para o outro. Agora valem só para o
  preset que os gravou.

Provas:
- testes do dono:
  - a pasta por preset;
  - os nomes recusados (vazio, `.`, `..`, `/`, `\`);
  - um `binaryDir` fora da raiz e uma macro aberta;
  - a ordem da configuração ativa, sem volta silenciosa;
  - o registro de ida e volta;
  - a tentativa ativa desde o começo, com registros e rigor por pasta;
  - a pasta comum de dois presets: a trava recusa o segundo sem tocar o
    registro, e a tentativa e o kit de um não valem para o outro;
  - a tentativa pendente deixa obsoleta só a CDB da pasta dela;
- três regressões com CMake 4.4.5, GCC 16.2.1, Clang 23.1.4, Ninja e `ctest`
  reais:
  - os presets: dois com `binaryDir`, um deles fora da raiz com espaço e acento
    no caminho, e um sem `binaryDir`;
  - as agulhas: o executável imprime o tipo de build e o compilador, e o caso
    do `ctest` tem o tipo no nome;
  - cada configuração configura, compila, roda e testa na pasta dela. O alvo
    de debug sai da mesma pasta, e o contexto vem da CDB dela;
  - voltar a uma configuração não reconfigura;
  - a pasta que sumiu é refeita no mesmo lugar;
  - o nome inválido é recusado sem processo;
  - um configure que falha deixa a outra configuração atual;
  - um `.kinein/build` de antes do C-04 segue ativo no automático, e o próximo
    configure vai para a pasta do preset;
  - reparar recusa o cache de outro projeto e o build in-source, e a prévia
    mostra o caminho absoluto;
- o `qmlls` recebe `-b` da configuração ativa e reinicia com a escolhida;
- as regressões C anteriores passaram a provar a pasta do preset. Entre elas
  estão a falha do configure com a configuração anterior intacta e duas
  configurações configurando ao mesmo tempo, com a mesma recusada;
- vinte e quatro mutantes reprovam, todos contra baseline verde:
  - o nome inválido e a barra invertida aceitos;
  - o registro órfão aceito e a pasta da IDE frouxa;
  - a tentativa e o kit de outro preset;
  - o registro gravado antes da trava ou não gravado;
  - o configure sem `-B`, e o build que configura na pasta ativa;
  - ctest, debug, run e `qmlls` em `.kinein/build`;
  - as escolhas do registro escondidas, e o comando anterior sempre anunciado;
  - a nota do `binaryDir` ausente, e as pastas da IDE fora das candidatas;
  - a tentativa que não marca a CDB, e a exceção antiga do `.kinein/build`;
  - o in-source e o cache de outro projeto removíveis;
  - a escolha que não recarrega.

  Na primeira rodada, quatro sobreviveram. O M9 e o M21 sobreviveram por
  falta de prova, coberta pela pasta comum e pela CDB obsoleta. O M4 e o M5
  eram mutantes equivalentes para a prova nova, e foram refeitos. O mutante
  sem `-B` configurou em `crates/kinein-core`, o diretório do processo de
  teste: o preset sem `binaryDir` caiu ali, como cai no terminal. Os arquivos
  gerados foram removidos, e o `compile_commands.json` versionado foi
  restaurado.

Limites:
- sem presets no projeto e com um registro órfão, a UI não oferece saída até o
  C1-23 ("Automático"). O IPC já aceita `cmake.selectConfiguration { preset:
  null }`;
- o `qmlls` monta o `-b` ao abrir, ao configurar e ao escolher. Uma edição do
  `CMakePresets.json` que muda a configuração ativa só chega a ele no próximo
  desses momentos;
- `configured` continua sendo o stat do `CMakeCache.txt`. O CMake grava o cache
  mesmo quando o configure falha; quem diz que ele não vale é a tentativa
  pendente;
- o flash e os artefatos leem a pasta ativa sem regressão própria;
- o multi-config (C1-22) e o menu (C1-23) seguem abertos.

### 17.3 C1-23: integração e provas — 2026-11-09

O `cmake.status` ganhou `configurations`: cada configure preset não oculto, com
a pasta da configuração dele ou o motivo da recusa, e se ela tem
`CMakeCache.txt`. Quem monta a lista é `cmake_configuration::configurations`,
pelas mesmas regras da pasta do configure (§17). Protocolo 0.210.0, com o
contrato no [03 cmake](../arquitetura/03-dominios/cmake.md). Na ponte,
`cmakeStatusResolved` ganhou o argumento `configuration`.

O menu tem um dono próprio, `CmakeConfigurationChoices.qml`, porque o
`ProjectHealthController` estava perto do limite de tamanho. O controller
decide o aviso da faixa, e o objeto novo diz o que há para escolher: o texto e
a dica do chip e os itens do menu. Cada item mostra a pasta à direita, cortada
no meio, e a dica tem o caminho inteiro. A pasta desconhecida não se escolhe,
e a dica dela diz por quê. O `AppMenuPopup` ganhou o detalhe e a dica, e o item
desabilitado passou a receber o pairar. O `TopHeaderBar` estava a quatro
linhas do limite, e o grupo da esquerda foi para `HeaderLeftGroup.qml`. O chip
da configuração entra antes do da toolchain e pega a largura antes dela. Uma
ordem salva antes dele o recebe no fim.

A faixa usa o mesmo menu quando a escolha é obrigatória (o menu do C-05 por
candidata saiu) e quando não há configuração ativa sem presets a configurar.
Nesse segundo caso, que inclui o registro órfão e o ilegível, o configure
automático espera. Escolher no menu não configura.

O menu Ambiente ganhou **Configuração do CMake…**, ao lado de **Configurar
CMake**, habilitado só em projeto CMake. O item do "⋯" usa a mesma ação, e o
gate dos atalhos exige que toda ação tratada pelo `ShellHeaderHost` tenha um
item na barra de título. A toolchain e o Python já seguem esse padrão. Decisão
do autor de 2026-11-09, quando o gate reprovou o item do "⋯" sozinho.

Um achado da integração foi corrigido no dono. Num projeto confiável que
ainda não tinha tido o configure automático, escolher uma pasta sem cache o
liberava: a escolha de `semPasta` mandou `cmake.configure` na UI nativa. No
C-05 isso não acontecia, porque o menu só tinha pastas com CDB. Agora a escolha
gasta o configure automático, e a faixa oferece o **Configurar**.

Outro achado ficou aberto como C1-24 (inventário acima). Ao montar a prova, o
CMake 4.4.5 recusou o `CMakePresets.json` inteiro por uma macro inválida no
`binaryDir` de um preset. A IDE expande só o `binaryDir` do preset usado e
listava os outros.

Provas:
- testes do dono: a lista com `displayName`, pasta fora da raiz com espaço e
  acento, preset sem `binaryDir`, nome que não serve de pasta, macro sem
  fechamento, preset oculto e dois presets na mesma pasta; a lista vazia sem
  presets e com presets recusados; o `cmake.status` no fio e o protocolo;
- harnesses com o `ProjectHealthController`, a faixa, o `AppMenuPopup` e o
  `TopHeaderBar` reais: a marca, o "Automático", o corte no meio, a dica do
  item e do desabilitado, a faixa do órfão e do ilegível, o configure
  automático que espera, a escolha que não configura, a ordem, a prioridade de
  largura e o "⋯" com o aviso;
- UI nativa isolada, em quatro sessões com CMake 4.4.5, GCC 16.2.1 e Ninja reais.
  A fixture tem quatro presets: `debug` com `displayName`, `release` num
  caminho longo fora da raiz, `semPasta` sem `binaryDir` e `ci/linux`, cujo
  nome não serve de pasta. Escolher `release` e voltar ao automático gravou e
  apagou o registro, sem configure. O menu da faixa escolheu `debug`. Com a
  janela estreita, o chip foi para o "⋯", e o item dele abriu o menu. No
  projeto com registro órfão e sem presets, a faixa deu a volta ao automático,
  e o **Configurar** configurou `.kinein/build` de verdade. Escolher
  `semPasta` não configurou, e o **Configurar** configurou
  `.kinein/presets/semPasta`. Na quarta sessão, **Ambiente → Configuração do
  CMake…** abriu o mesmo menu, com `semPasta` marcada. Fontes e presets
  mantiveram seus bytes;
- 24 mutantes que compilam ou carregam, todos reprovados: 6 no core (a lista
  vazia, a pasta existente tomada por configurada, a recusa sem motivo, a pasta
  sem rótulo, o `cmake.status` sem a lista e o `displayName` perdido) e 18 na
  UI (a marca sem olhar a origem, o "Automático" marcado na recusa, a pasta
  desconhecida escolhível, o corte no fim, o desabilitado sem pairar, o
  configure automático na recusa e depois da escolha, a escolha herdada por
  outro projeto, a faixa sem a volta ao automático, o ilegível sem chip, a
  toolchain antes da configuração, o "⋯" sem ela, o aviso escondido sem ponto,
  o menu estreito, o roteador sem o argumento, o automático sem o preset, o
  chip sem `displayName` e o item do Ambiente fora de projeto CMake). O
  mutante do desabilitado sobreviveu na primeira rodada: o harness emitia o
  `entered` direto, o que pula o `enabled` da área. O harness passou a
  conferir a área ligada.

Limites:
- a ponte C++ e o tratamento das ações do "⋯" e do Ambiente no
  `ShellHeaderHost` não têm teste automatizado; ficaram provados na UI nativa;
- abaixo de uns 760 px, o projeto e o executar ocupam o cabeçalho, e o "⋯" fica
  por baixo do executar, como antes desta fatia;
- C1-22 e C1-24 seguem abertos.

### 17.4 C1-24: integração e provas — 2026-11-09

A leitura dos presets passou a expandir todos os presets como o CMake 4.4.5,
medido em treze rodadas de `cmake --list-presets` e `cmake --preset` sobre
projetos gerados. O dono continua `cmake_presets`, dividido por
responsabilidade:
- `files.rs` lê os quatro tipos de preset (configure, build, test e package) e
  guarda a versão de cada arquivo;
- `macros.rs` tem um scanner só. No contexto do preset, valem as macros que a
  versão do arquivo conhece, `${generator}` do configure preset (vazio sem
  gerador e sempre vazio no package preset), `$env{}` do `environment` depois
  da herança, com ciclo recusado e `null` caindo no ambiente do core, e
  `$vendor{}` interrompendo o preset. O `include` segue a política do C1-18;
- `expansion.rs` é novo: expande cada preset na ordem do CMake (`environment`
  com as chaves em ordem, `condition` e os campos do tipo), avalia a
  `condition` herdada e diz quais configure presets o CMake ignora
  (`$vendor{}`) ou desabilita (`condition` falsa).

Uma macro inválida em qualquer preset recusa todos, com o tipo, o nome e o
motivo. A lista, o padrão, a pasta da configuração, o menu e os valores da
toolchain usam a mesma leitura. Um preset ignorado ou desabilitado sai da
lista, e o configure pedido pelo nome dele é recusado antes do job. Protocolo
0.211.0, sem campo novo, com o contrato no
[03 cmake](../arquitetura/03-dominios/cmake.md).

A regra do `${fileDir}` da versão 12 foi medida com dez combinações de macros:
o texto usa a pasta do arquivo que o escreveu só quando todas as macros dele são
`${fileDir}`. Um `$` que não abre macro não conta, e qualquer outra macro
(`${presetName}`, `${hostSystemName}`, `$env{}`, `${dollar}`) devolve o texto ao
preset em uso. A versão que decide é a do arquivo que escreveu o texto. Na
`condition` e no `environment` a regra não vale. A IDE passa `-B` com a pasta
que ela resolve (C1-19), e antes desta fatia a IDE e o `cmake --preset`
configuravam em pastas diferentes.

`anyOf` para no primeiro verdadeiro e `allOf` no primeiro falso, sem expandir o
resto (medido com uma macro inválida depois). Um `anyOf` vazio é falso, e um
`allOf` vazio, verdadeiro. `matches` e `notMatches` ficam indecisos e contam
como verdadeiros (C1-26).

Os mutantes acharam uma divergência, corrigida no dono. O que tirava o
`${generator}` do build preset sobreviveu à primeira rodada. Medido com um
`anyOf` que compara o gerador antes de uma macro inválida, o CMake dá o
gerador do configure preset ao build e ao test preset, herdado também, e nunca
ao package preset; a IDE dava também ao package. O `environment` do configure
preset chega aos três tipos, e `inheritConfigureEnvironment: false` o corta
(medido). Outro sobrevivente, a regra da versão 12 tirada da leitura, era
equivalente: se um texto é recusado ou para no `$vendor{}` não depende da pasta
que o `${fileDir}` nomeia. A leitura passou a expandir sem a regra, que ficou só
no valor do campo.

Provas:
- testes do dono, em `expansion/tests.rs`: macro inválida em cada campo de
  cada tipo, cada motivo de recusa, a versão por arquivo, o ciclo e o `null`,
  a ordem do `$vendor{}`, a `condition` herdada, curta e indecisa, o
  `${fileDir}` da versão 12, o `${generator}` por tipo e os valores da
  toolchain; no `cmake_presets`, os presets ignorados ou desabilitados ficam
  sem pasta;
- diferencial com o CMake real, em 347 projetos: 17 campos × 9 macros
  (inválidas, ignoradas e válidas) × oculto ou não, os gates de versão de 2 a
  5, os gates herdados entre arquivos, 21 casos de `environment`, ordem,
  `condition` e `${generator}` por tipo, e o `CMakeUserPresets.json`. Em cada
  um, a IDE recusa ou lista exatamente o que `cmake --list-presets` recusa ou
  lista. O teste exige mais de cem de cada lado;
- regressão de ponta a ponta com `cmake --preset`, CMake 4.4.5 e GCC 16.2.1
  reais. Na versão 12, o CMake configura `kid` na pasta do arquivo incluído e
  `named` (com `${presetName}`) na pasta do preset. O configure da IDE vai à
  mesma pasta, e a CDB tem o GCC. Depois, um build preset oculto com macro
  inválida recusa o configure com o motivo, sem `event.cmake.started` e sem
  `configurations`;
- os dois testes de regressão reprovaram contra o dono do commit 3e27ffe2;
- 33 mutantes que compilam, todos reprovados: só os configure presets lidos,
  os ocultos pulados, cada um dos três gates de versão, a versão do arquivo
  mais novo, o `${generator}` vazio recusado, a regra da versão 12 desligada,
  tirada do campo ou aplicada sem olhar as macros, o `null` expandindo vazio,
  o `$vendor{}` sem interromper, as chaves fora de ordem, a `condition` antes
  do `environment` ou sem herança, o `anyOf` sem curto-circuito, o `const`
  indeciso, o `inList` sem a lista, o preset desabilitado listado, o build sem
  o `environment` ou o gerador do configure, o package com o gerador, o
  `inheritConfigureEnvironment` ignorado, a pasta e o valor do preset que o
  CMake não usa, `graphviz` e `fixtures.cleanup` fora da tabela, o mapa sem o
  `value` do objeto ou com o último vencendo, os pais na ordem inversa, o
  rótulo do build, o `$env{}` sem nome aceito e o ciclo sem detecção, que
  derruba o teste por estouro de pilha. Na primeira rodada, dois mutantes não
  compilaram (uma função e uma variável sem uso) e foram refeitos, e dois
  sobreviveram, como descrito acima.

Limites:
- o esquema dos presets (C1-25) segue aberto: campo extra, campo de versão
  maior, gerador desconhecido, `configurePreset` inexistente e `condition`
  incompleta ou de tipo desconhecido;
- `matches` e `notMatches` (C1-26) seguem indecisos. Um `anyOf` ou `allOf` com
  uma forma indecisa antes de uma macro inválida segue expandindo, e o CMake
  pode parar antes;
- a regra do `${fileDir}` foi medida em `binaryDir`, `installDir`,
  `toolchainFile`, `graphviz` e `cacheVariables`, e a IDE usa o valor de
  `binaryDir`, `toolchainFile` e `cacheVariables`. Os campos de build, test e
  package presets ela só confere (recusa e `$vendor{}`), sem usar o valor;
- seguem abertos o C1-25 e, depois dele, o C1-22.

## 18. C1-25: os presets pelo CMake do projeto — desenho de 2026-11-10

O esquema dos presets muda com a versão do CMake: os campos de cada versão de
arquivo, os tipos, os campos obrigatórios e os geradores que ele tem. A leitura
nativa do C1-24 segue o CMake 4.4.5 e não confere o esquema (C1-25, inventário
do §17). Decisões do autor de 2026-11-10: a IDE pergunta ao CMake do projeto, e
quem responde é o CMake que o kit padrão do workspace fixa, senão o `cmake` do
`PATH`, a mesma regra do configure sem kit.

Medido com o CMake 4.4.5:
- `cmake --list-presets` lista só os configure presets, um por linha, como
  `  "nome"` ou `  "nome"  - displayName`, sem escapar as aspas do nome, e
  sai com 0. Sem preset visível, não imprime nada;
- numa recusa, sai com 1, e a linha depois de
  `CMake Error: Could not read presets from <pasta>:` é o motivo, às vezes com
  o arquivo e a linha (`CMakePresets.json:1: Invalid extra field "nope" in
  Preset`). Seguem o trecho do JSON e um `^`. Recusam assim o campo extra, o
  campo de versão maior, o tipo errado, o JSON inválido, o `configurePreset`
  inexistente e a `condition` incompleta ou de tipo desconhecido;
- um preset cujo gerador este CMake não tem sai da lista, também herdado. Isso
  inclui os geradores do Visual Studio no Linux e a macro no `generator`, que o
  CMake não expande. Um filho que troca o gerador por um que existe volta à
  lista. `cmake --list-presets=all` lista esses presets, e por isso a pergunta
  usa a forma sem `=all`, que é a do `cmake.presets.list`;
- `matches` com `\d` não casa `a1`, e o preset sai da lista (C1-26);
- a pergunta leva cerca de 9 ms num projeto de 80 presets.

Desenho:
- `cmake_presets/verdict.rs` roda `<cmake> --list-presets` na raiz, com o
  ambiente do core, só quando há arquivo de presets. A saída 0 dá os nomes
  listados: cada linha é conferida contra os nomes da leitura nativa e, sem
  ela, lida até o `"  - ` ou o fim. Outra saída dá o motivo da linha depois de
  `Could not read presets from`, ou a primeira linha de erro. Um executável
  ausente, ou que não roda, não dá veredito;
- o programa vem do toolchain: o `cmake` que o kit padrão fixa, achado pelo
  detector do core, senão `cmake` pelo `PATH`. O exec repete no `ETXTBSY`, com
  a regra que hoje só a sonda de versão tem;
- o veredito fica em cache por raiz. A chave tem o caminho, o tamanho e a data
  do executável resolvido e de cada arquivo de presets que a leitura nativa
  abriu, também os abertos antes de uma recusa dela. Mudou um, a IDE pergunta
  de novo;
- quando o CMake recusa, todos os presets são recusados com
  `o CMake recusa os presets: <motivo>`: a lista, o padrão, o menu, a pasta e o
  configure, como as recusas do C1-18;
- quando o CMake lista e a leitura nativa lê, a lista é a do CMake. Um preset
  não oculto fora dela não serve: com o motivo nativo (`$vendor{}`, condition
  falsa), com `a condition do preset X é falsa para o CMake` quando a leitura
  nativa não a decidia, senão com `o CMake não tem o gerador "G" do preset X`
  ou `o CMake não lista o preset X`;
- quando o CMake lista e a leitura nativa recusa, os presets são recusados com
  `o CMake aceita os presets, mas a IDE não os lê: <motivo>`. A IDE precisa dos
  valores (`binaryDir`, toolchain) para configurar, e a divergência fica à
  vista em vez de virar uma pasta adivinhada;
- sem veredito, vale a leitura nativa, como no C1-24;
- o diferencial do C1-24 continua comparando a leitura nativa com o CMake
  real, para não virar a comparação do CMake consigo mesmo.

O contrato muda sem campo novo: protocolo 0.212.0, escrito no
[03 cmake](../arquitetura/03-dominios/cmake.md) antes da mudança.

Provas necessárias:
- cada recusa medida, pelo CMake real, com o motivo dele na lista, no padrão,
  no menu e no configure, sem job;
- o gerador desconhecido, o do Visual Studio e o herdado fora da lista, o filho
  que troca o gerador dentro, e o configure pedido pelo nome recusado antes do
  job;
- com um CMake falso que registra cada chamada: a segunda leitura não pergunta;
  uma edição num arquivo incluído, ou um executável trocado, pergunta de novo;
  o executável ausente cai na leitura nativa; a divergência das duas leituras
  aparece com os dois motivos;
- o programa do kit padrão e o do `PATH`;
- mutantes que compilem e reprovem pela guarda.

Limites previstos:
- a pergunta não tem prazo, como a sonda de `--version`; um CMake que trava
  trava a leitura;
- o ambiente é o do core. Um preset cujo kit fixa outro CMake configura com
  ele, e a recusa desse CMake aparece na saída do configure;
- sem CMake, `matches` e `notMatches` continuam verdadeiros, e a IDE ainda não
  diz isso (C1-26).

### 18.1 C1-25: integração e provas — 2026-11-10

A lista, o padrão, o menu, a pasta da configuração e o configure seguem o
`cmake --list-presets` do CMake do projeto, como o §18 desenhou. O dono continua
`cmake_presets`:
- `files.rs` guarda cada arquivo que a leitura abriu, com um hash do texto,
  também os abertos antes de uma recusa. Diz também a ordem em que o CMake
  percorre os arquivos;
- `reading.rs` é novo e guarda por que um preset não serve: ignorado,
  desabilitado, com `condition` que a IDE não decide, ou fora da lista do
  CMake, com o motivo;
- `verdict.rs` é novo: pergunta, guarda a resposta em cache e combina as duas
  leituras.

O programa vem do módulo novo `kits`, e não do `toolchain`, como o §18 previa.
O `toolchain` lê os presets, e os presets passaram a ler o kit: o ciclo
reprovava o mapa de módulos. Decisão do autor de 2026-11-10: descer para `kits`
o arquivo dos kits (`store.rs`, o `.kinein/toolchain.json`), o catálogo de
candidatos por papel (`catalog.rs`) e a busca do executável que um kit fixa. O
`toolchain` passou a usá-los de lá, sem mudança de comportamento. Os dois
identificadores em português do `store.rs` foram traduzidos na mudança, e a
linha de base deles saiu.

A medida da implementação achou a ordem. O CMake lista primeiro os presets do
`CMakeUserPresets.json`, depois os que ele inclui e por último os do
`CMakePresets.json` e dos seus includes, cada arquivo uma vez, onde aparece
primeiro. A IDE listava antes os do `CMakePresets.json`. Agora a lista e o menu
seguem a ordem que o CMake imprime e, sem ele, a dos arquivos como ele os
percorre. O contrato do 0.212.0 registra a mudança.

O cache segue o desenho com uma diferença: a chave tem o caminho, o tamanho e
a data do executável, mas dos arquivos de presets guarda o hash do texto, e não
a data. Uma edição de mesmo tamanho no mesmo segundo também pergunta de novo.
Um arquivo que aparece ou some muda a lista de arquivos abertos, e por isso o
arquivo ausente não precisa de entrada própria.

Os mutantes acharam o C1-27, registrado no inventário do §17. O que dava aos
presets ocultos o motivo de fora da lista sobreviveu, porque nenhum teste
decidia o caso. Medido com o 4.4.5: `cmake --preset h`, com `h` oculto, sai
com "Cannot use hidden configure preset". A IDE começava o job. Decisão do
autor de 2026-11-10: corrigir dentro do C1-25. A leitura nativa anota os
configure presets ocultos, e o oculto pedido pelo nome é recusado antes do
job, com ou sem CMake, com `o preset X é oculto: o CMake não o usa pelo nome`.
O filho visível continua herdando dele. Com isso, os filtros de oculto da lista
e do fora da lista ficaram redundantes e saíram.

A linha da lista também foi medida: o CMake alinha o `- displayName` pelo nome
mais longo, com espaços. A IDE reconhece o nome entre aspas seguido do fim da
linha, ou de espaços e `- `. Um nome que é prefixo de outro não se confunde,
nem `x` com um preset chamado `x - y`, que parece um `displayName`. As aspas
dentro do nome, que o CMake não escapa, também não confundem.

Provas:
- testes do dono, em `verdict/tests.rs`, contra o CMake 4.4.5 real: as dez
  recusas do esquema medidas (campo extra na raiz e no preset, `installDir`,
  `toolchainFile` e `graphviz` em versão menor, tipo errado, `configurePreset`
  inexistente, `condition` sem `rhs` e de tipo desconhecido, nome vazio), cada
  uma na lista, no padrão e na pasta, depois de a leitura nativa aceitar o
  arquivo; os geradores do Visual Studio, desconhecido e herdado fora da
  lista, o filho que troca o gerador dentro, o `matches` falso para o CMake e o
  padrão entre os que sobram;
- com um CMake falso que registra cada chamada: a segunda leitura não pergunta,
  e perguntam de novo uma edição de mesmo tamanho num include, um arquivo de
  usuário que aparece, um executável trocado no mesmo caminho e um include que
  some, cuja recusa também fica em cache; dois executáveis de mesmo texto,
  tamanho e data em pastas diferentes têm cada um a sua resposta; a lista com
  alinhamento, aspas, um nome prefixo de outro e `x` ao lado de `x - y`, na
  ordem impressa, também em `cmake.presets.list`, com o preset que a leitura
  nativa desabilitava e o CMake lista; o motivo da linha certa; o CMake que aceita o
  que a IDE não lê; o executável ausente, uma pasta e um arquivo que não
  executa no lugar dele; sem arquivo de presets, nenhuma pergunta;
- o kit padrão fixa o CMake que responde, e outro kit não; com o `PATH` dado,
  o fixado vence o do `PATH` e, sem fixação, o do `PATH` responde. No `kits`,
  cada ramo da busca: o kit e o papel pela chave, a instalação pelo caminho, o
  binário antes do alternativo, o fixado que sumiu e o id fora do catálogo;
- regressão de ponta a ponta pelo core, com CMake 4.4.5, GCC 16.2.1 e Ninja
  reais: a lista e o menu têm só o preset que o CMake lista; o configure do
  preset do Visual Studio, do gerador herdado e do `matches` falso é recusado
  com o motivo, sem `event.cmake.started` e sem pasta; o configure sem preset
  pedido escreve a CDB; um campo fora do esquema recusa a lista e o configure
  com a linha que o CMake dá, sem job e sem `configurations`; corrigido o
  arquivo, a lista volta sem reabrir nada. O configure do preset oculto pelo
  nome é recusado com o motivo do C1-27, sem job; pela leitura nativa e com o
  CMake, a pasta e os valores do oculto são recusados, e o filho herda dele;
- os testes da leitura nativa (C1-18, C1-24) rodam sem perguntar ao CMake, e o
  diferencial do C1-24 compara a leitura nativa com o CMake real, nos mesmos
  347 projetos;
- 40 mutantes que compilam, todos reprovados na terceira rodada: o cache
  sempre vencido, sem a data, o caminho ou o texto da chave, ou sem os
  arquivos abertos; a pergunta sem arquivo de presets, ou só com todos; a
  recusa caindo na leitura nativa; a divergência sem o prefixo; a lista com
  `=all` ou fora da raiz; o motivo da linha errada, com o `CMake Error:` ou
  sempre a primeira linha; o nome sem o alinhamento, com qualquer resto ou sem
  a aspa final; o desabilitado nativo que o CMake lista; sem os de fora da
  lista, sem a ordem do CMake ou a dos arquivos, e a ordem a partir do
  projeto; sem o motivo da `condition` ou do gerador, e o gerador sem herança;
  o `matches` não anotado; sem o kit, sem o `PATH`, e o `PATH` do ambiente no
  lugar do dado; qualquer kit, qualquer papel; a instalação procurada no
  catálogo ou valendo depois de sumir; sem o binário alternativo, ou com ele
  antes; o id fora do catálogo; o oculto sem motivo, não anotado ou herdado.

Na primeira rodada, dez mutantes não compilaram (variável ou import sem uso,
que o teste nega) e foram refeitos, e sete sobreviveram. Três pediam teste: o
caminho do executável na chave, o nome sem a aspa final e a ordem do CMake na
lista. Quatro eram equivalentes, e o código saiu: a entrada do arquivo ausente
na chave, em três lugares, e a deduplicação da ordem dos arquivos, porque quem
ordena já pega o primeiro lugar; e o filtro dos já recusados no fora da lista,
porque o motivo nativo vem antes. Na segunda rodada sobreviveram dois: o nome
com qualquer resto, fechado com um preset `q` ao lado de `q"uote`, e o que dava
motivo ao oculto, que achou o C1-27.

Limites:
- a pergunta não tem prazo, como a sonda de `--version`: um CMake que trava
  trava a leitura;
- o ambiente é o do core. Um preset cujo kit fixa outro CMake configura com
  ele, e a recusa desse CMake aparece na saída do configure;
- os motivos do CMake são mais secos que os da leitura nativa: "Invalid macro
  expansion in "all"" não diz qual macro. O motivo nativo fica à vista só sem
  CMake;
- sem CMake, `matches` e `notMatches` continuam verdadeiros, e a IDE ainda não
  diz isso (C1-26);
- a ordem dos motivos de um preset oculto que também usa `$vendor{}` ou tem a
  `condition` falsa não foi medida: a IDE dá o do oculto;
- segue aberto o C1-26 e, depois dele, o C1-22.

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

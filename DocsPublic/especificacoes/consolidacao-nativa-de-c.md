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
[40.7 §7.376](../roadmaps/registro/40.7.9-a-base-publica-e-o-c-da-0.4.md#7376--jornada-c-de-host-e-sincronização-do-contexto-após-build-2026-11-03):
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
| C1-22 / multi-config | `CmakeModel` lê só `/configurations/0` (`cmake/model.rs`); `cmake --build` vai sem `--config`; `collect_executables` percorre a pasta inteira. Medido com Ninja Multi-Config: três configurações no codemodel, três entradas da CDB para o mesmo arquivo, o clangd usa a primeira (Debug), e `cmake --build` sem `--config` compila `CMAKE_DEFAULT_BUILD_TYPE`, senão a primeira; após dois builds há `Debug/app` e `Release/app`. Medido de novo em 2026-11-10 (CMake 4.4.5, clangd 23.1.4, §20): a CDB e o codemodel seguem a ordem de `CMAKE_CONFIGURATION_TYPES`, e o clangd usa a primeira entrada; o `ctest` sem `-C` dá todo teste como "Not Run" e sai com 8, e o `--show-only=json-v1` sem `-C` não lista nenhum teste. A regressão do C1-22 achou mais um: o `index.context` mostra a última entrada repetida da CDB (Debug), e o clangd, a primeira (Release); `parse_cdb` substitui a unidade e só a marca `ambiguous` (a causa do C-03). Causa: o build, o ctest, o run e o debug não sabem qual configuração usar, e o contexto fica com outra entrada | Média: com `CMAKE_DEFAULT_BUILD_TYPE` diferente, o build compila uma configuração e o contexto mostra outra; o run recusa por achar dois executáveis; os testes da IDE não rodam num build multi-config, e a descoberta por framework fica vazia | Corrigido em 2026-11-10 ([§20.1](#201-c1-22-integração-e-provas--2026-11-10), protocolo 0.214.0): o build, o `ctest`, o run, o debug e o contexto usam a configuração que o clangd lê, e a faixa informa a padrão diferente. Decisões do autor de 2026-11-10 (desenho no §20): a faixa mostra a diferença do `CMAKE_DEFAULT_BUILD_TYPE` como informativa, sem ação; a gravação nativa mantém a recusa do build multi-config; o run e o debug num build multi-config sem o modelo do file-api recusam com o motivo; o contexto fica com a primeira entrada repetida, como o clangd, e o resto do C-03 (guardar todas as unidades) segue no C2. Provar com Ninja Multi-Config real, `CMAKE_DEFAULT_BUILD_TYPE=Release` e mutações |
| C1-23 / UI sem troca da configuração | O Project Health só oferece "Escolher configuração…" quando a escolha é obrigatória; o menu corta no fim a pasta longa fora da raiz (§16.1) | Média: trocar a configuração ativa ou voltar ao automático exige editar `.kinein/configuration.json` | Menu de configuração sempre que há presets, com a ativa marcada e "Automático". Provar na UI nativa isolada |
| C1-24 / macros dos presets só sob demanda | Achado na prova do C1-23 (2026-11-09). `cmake_presets` expande só o `binaryDir`, e só do preset usado. Medido com `cmake --list-presets` e `cmake --preset` do CMake 4.4.5 (§17.4): cada preset de cada tipo, mesmo oculto e sem uso, é expandido ao ler os arquivos, com a herança e na versão do arquivo que o define. Uma macro desconhecida, sem fechamento, `$env{}` ou `$penv{}` sem nome, uma macro de versão maior (`${hostSystemName}` 3, `${fileDir}` 4, `${pathListSep}` 5) ou um ciclo no `environment` recusa o arquivo inteiro ("Invalid macro expansion"). Nos configure presets valem `binaryDir`, `installDir`, `toolchainFile`, `graphviz`, os valores de `cacheVariables` e de `environment` e as cadeias da `condition`; nos build presets, `environment`, `targets`, `nativeToolOptions` e `condition`, com o `environment` do configure preset; nos test presets, `environment`, `overwriteConfigurationFile`, `output`, `filter`, `execution` e `condition`; nos package presets, `environment`. `$vendor{}` tira da lista só o preset que o usa. Divergências vizinhas: `${generator}` sem `generator` expande vazio, e a IDE recusava a pasta; no build e no test preset, `${generator}` é o do configure preset, e no package preset é sempre vazio (achado pelos mutantes); na versão 12, `${fileDir}` é a pasta do arquivo que escreveu o texto quando todas as macros dele são `${fileDir}`, e a IDE usava a do preset em uso, então o `-B` da IDE e o `cmake --preset` configuravam em pastas diferentes. A `condition` é herdada, e o CMake a avalia ao listar: o preset de condição falsa some do `--list-presets`. A IDE lia só a do próprio preset e dava por verdadeira toda forma que não fosse do host. `environment: null` tira a chave, e `$env{}` cai no ambiente do processo; a IDE expandia vazio. `$vendor{}` interrompe a expansão do preset: o que vem depois, na ordem do CMake, não é conferido. A ordem é `environment`, `condition` e os campos do tipo, com as chaves dos mapas em ordem. A toolchain mostrava `CMAKE_C_COMPILER` e `toolchainFile` crus, com as macros | Média: a IDE lista, escolhe e oferece no menu presets de um arquivo que o CMake recusa, e o configure falha no CMake com a mensagem dele, não em silêncio; com `${fileDir}` na versão 12, a IDE configura fora da pasta do preset | Corrigido em 2026-11-09 (§17.4). Decisões do autor de 2026-11-09: corrigir antes do C1-22, e em todos os tipos de preset. Corrigir em `cmake_presets`, com o motivo do CMake. A `condition` segue o CMake em todas as formas menos `matches` e `notMatches`, que dependem da expressão regular do CMake (C1-26) e continuam valendo como verdadeiras. Provar com o CMake real cada campo, o preset oculto, a versão por arquivo, o ciclo e as pastas do `${fileDir}` |
| C1-25 / esquema dos presets | Achado na medida do C1-24 (2026-11-09, CMake 4.4.5). O CMake recusa o arquivo inteiro por um campo que o esquema não tem ("Invalid extra field") e por um campo de versão maior (`installDir` e `toolchainFile` 3, `graphviz` 10: "File version must be N or higher"); um `generator` desconhecido tira o preset da lista, e o configure dele falha. Também recusam o arquivo um `configurePreset` inexistente (`Invalid "configurePreset"`), uma `condition` sem campo obrigatório (`Missing required field "rhs"`) e uma de tipo desconhecido (`Invalid condition`). A IDE lê só os campos que usa e lista esses presets; na `condition`, o campo ausente vale vazio, e o tipo desconhecido, verdadeiro | Média: o mesmo efeito do C1-24 por outra causa | Corrigido em 2026-11-10 (§18.1). Decisão do autor de 2026-11-09: corrigir depois do C1-24 e antes do C1-22. Decisões do autor de 2026-11-10: perguntar ao CMake do projeto, o que o kit padrão do workspace fixa, senão o do `PATH` (desenho no §18). A IDE roda o `cmake --list-presets` desse CMake, com cache pela data dos arquivos de presets e do executável, e a recusa e a lista dele valem: o esquema muda com a versão do CMake. A leitura nativa do C1-24 fica para os valores e para quando não há CMake. Com isso a lista do CMake já reflete `matches` e `notMatches`, e ao C1-26 resta o caso sem CMake. Provar com o CMake real cada recusa medida, o cache e a ausência do CMake |
| C1-26 / expressão regular da `condition` | Achado na medida do C1-24 (2026-11-09, CMake 4.4.5). `matches` e `notMatches` usam a expressão regular do próprio CMake: `\d` não é classe, e `^a\d$` não casa `a1`. Uma expressão que não compila recusa o arquivo inteiro ("RegularExpression::compile(): Error in compile"). A IDE dá essas formas por verdadeiras | Baixa: um preset de `matches` falsa aparece na lista, e uma expressão inválida não recusa o arquivo | Corrigido em 2026-11-10 (§19.1). Decisão do autor de 2026-11-10: perguntar ao CMake. Só quando uma `condition` usa `matches` ou `notMatches`, a IDE roda o `cmake --list-presets` do projeto, com cache pela data dos arquivos, e usa a lista ou a recusa dele. Sem CMake, essas formas continuam verdadeiras, e a IDE diz isso. Vem depois do C1-25 e antes do C1-22 (autor, 2026-11-10). Com o C1-25, a lista do CMake já decide essas formas. Decisões do autor de 2026-11-10 para o resto (desenho no §19): sem CMake, o item do preset no menu diz na dica que a `condition` fica verdadeira; uma macro ou expressão inválida depois de uma forma indecisa, num `anyOf` ou `allOf`, deixa o preset indeciso, em vez de recusar. Provar com o CMake real a expressão que casa, a que não casa, a que não compila e a ausência do CMake |
| C1-27 / preset oculto pedido pelo nome | Achado pelos mutantes do C1-25 (2026-11-10): o mutante que dava aos presets ocultos o motivo de fora da lista sobreviveu, porque nenhum teste decidia o caso. Medido com o CMake 4.4.5: `cmake --preset h`, com `h` oculto, sai com 1 e "Cannot use hidden configure preset", e o `cmake --list-presets` não o lista. A IDE resolve a pasta, o configure pelo nome começa o job e o CMake recusa depois. Causa: `Reading::unusable` não conhece o oculto, e só a lista o filtra | Baixa: só quem pede o preset pelo nome, pelo IPC ou por uma configuração registrada que ficou oculta; o configure falha com a mensagem do CMake | Corrigido em 2026-11-10 (§18.1). Decisão do autor de 2026-11-10: corrigir dentro do C1-25, no mesmo protocolo 0.212.0. A leitura nativa anota os configure presets ocultos, e o preset oculto pedido pelo nome é recusado antes do job, com ou sem CMake: "o preset h é oculto: o CMake não o usa pelo nome". Vale para a pasta, o configure e os valores da toolchain; um filho visível continua herdando dele. Provar pela leitura nativa e com o CMake real, de ponta a ponta sem job, e com o mutante da guarda |
| C1-28 / Catch2 por qualquer saída | Achado na regressão do C1-22 (2026-11-10). `test::frameworks::list_cases` toma por Catch2 todo executável de teste que sai com 0 e imprime uma linha para `--list-tests --verbosity quiet`: um teste do `ctest` que imprime `kv-run-Release` para qualquer argumento virou dois casos Catch2 na descoberta, e rodar um deles roda `exe "kv-run-Release" -r compact`. O gtest exige a forma `Suite.` com casos indentados e não confunde esse binário. Causa: `parse_catch2_list` aceita qualquer linha não vazia | Média: a árvore de testes mostra casos falsos para um teste comum que imprime algo, e o caso falso roda o binário com um argumento que ele ignora | Corrigido em 2026-11-10 ([§20.1](#201-c1-22-integração-e-provas--2026-11-10)): a lista do Catch2 só vem do binário que responde `framework: Catch2` ao `--libidentify`, provado com o Catch2 3.16.1 e o gtest 1.18 reais. Decisão do autor de 2026-11-10: corrigir agora, com o Catch2 real que ele instala nesta máquina (`catch2` 3.16.1 da CachyOS). Medir o que o Catch2 imprime em `--libidentify` e em `--list-tests`, e exigir a identidade antes de aceitar casos. Provar com Catch2 e gtest reais, o executável comum que imprime, e mutações |
| C1-29 / caso gtest contado duas vezes | Achado na prova do C1-28 com o gtest 1.18 real (2026-11-10). Ao rodar um caso de dentro (`test::frameworks::run_inner_case`), o gtest imprime `[  FAILED  ] Math.Fails (0 ms)` e, no resumo do fim, `[  FAILED  ] Math.Fails` de novo; o mesmo com `[  SKIPPED ]`. Um caso que falha vira `failed: 2`. Num parametrizado, a linha é `[  FAILED  ] Some/P.Odd/1, where GetParam() = 2 (0 ms)`, e o nome que sai não é o `Some/P.Odd/1` da lista, então `JobsController.handleTestCase` não marca a linha do caso. Com `GTEST_PRINT_TIME=0` no ambiente, nem a linha do caso tem o tempo; `--gtest_print_time=1` vence o ambiente (medido). Causa: `parse_gtest_case` aceita as linhas do resumo e corta o nome só no ` (` | Baixa: o resumo do teste conta em dobro as falhas e os ignorados de um caso, e o caso parametrizado que falha não fica vermelho na árvore | Corrigido em 2026-11-10 ([§20.1](#201-c1-22-integração-e-provas--2026-11-10)), provado com o gtest 1.18 real. Decisão do autor de 2026-11-10: corrigir agora, junto do C1-28. O caso roda com `--gtest_print_time=1`, e só conta a linha que termina no tempo; o comentário `, where …` sai do nome. Provar com gtest real, inclusive um `main` que desliga o tempo, e mutações |
| C1-30 / biblioteca compartilhada como executável | Achado nos mutantes do C1-22 (2026-11-10), medido com `CMake` 4.4.5 e Ninja. Num build single-config, `run::cmake_executables` varre a pasta e conta todo arquivo com o bit de execução: um projeto com `add_library(... SHARED)`, `MODULE` e uma versionada tem `app`, `libshared.so`, `libplugin.so` e `libversioned.so.1.2.3`, e o run e o debug respondem "mais de um executável". O codemodel do file-api diz `EXECUTABLE` só para o `app`. Causa: o single-config não lê o codemodel, que o multi-config já lê desde o C1-22 | Média: o run e o debug sem comando recusam todo projeto C com biblioteca compartilhada | Corrigido em 2026-11-10 ([§20.1](#201-c1-22-integração-e-provas--2026-11-10)), provado com o CMake 4.4.5 real. Decisão do autor de 2026-11-10: corrigir agora, junto do C1-22. Com o modelo do file-api, só os artefatos dos alvos `EXECUTABLE`, como no multi-config; sem o modelo, a varredura deixa de fora `.so`, `.so.N` e `.dylib` |
| C1-31 / nome do caso Catch2 como filtro | Achado na prova do C1-28 com o Catch2 3.16.1 real (2026-11-10). `run_inner_case` passa o nome do caso como o filtro do Catch2, que dá sentido a alguns caracteres. Medido: `~tilde` e `exclude:this` excluem e rodam os 9 casos do binário; `a, b`, `x [y]` e `back\slash` não acham nada e saem com 2. A barra invertida antes de `\`, `,`, `[`, `]`, `*`, `~` e `"` roda só o caso; o prefixo `exclude:` não se escapa, e o Catch2 compara sem caixa (`case` roda `Case` também). O `--list-tests` do Catch2 v3 sai em ordem aleatória, e a árvore muda de ordem a cada descoberta; `--order decl` dá a ordem da declaração. Causa: o nome vai sem escape, e a lista, sem ordem | Média: "Rodar só este teste" roda todos os casos ou nenhum para esses nomes, e a falta de caso vira falha | Corrigido em 2026-11-10 ([§20.1](#201-c1-22-integração-e-provas--2026-11-10)), provado com o Catch2 3.16.1 real. Decisão do autor de 2026-11-10: corrigir agora, junto do C1-22. O nome vai com a barra invertida antes de `\`, `,`, `[`, `]`, `*`, `~` e `"`, e a lista sai com `--order decl`. Quando o Catch2 não isola o caso (`exclude:` no começo, ou outro caso da lista com o mesmo nome sem caixa), "Rodar só este teste" recusa com o motivo, sem rodar o binário. Provar com o Catch2 3.16.1 real e mutações |
| C1-32 / casos de dentro de um teste que já é um caso | Achado na prova do C1-31 (2026-11-10), por leitura de `test::frameworks::discover_inner_cases` e medido com o CTest do CMake 4.4.5, o gtest 1.18 e o Catch2 3.16.1. Com `gtest_discover_tests`, o `--show-only=json-v1` lista um teste por caso, com `--gtest_filter=Math.Adds` no comando; com `catch_discover_tests`, um por caso, com o nome do caso no comando. A descoberta pergunta ao binário todos os casos para cada teste: três casos gtest viram nove ids `teste::caso` falsos, e dois Catch2, quatro. O teste `Widget::draw works` do Catch2 tem `::` no nome, e `run_inner_case` corta o id no primeiro `::`. Causa: a descoberta usa só o primeiro item do comando, e o id não diz onde acaba o nome do teste do ctest | Média: num projeto com `gtest_discover_tests` ou `catch_discover_tests`, comuns no CMake, a árvore de testes repete cada caso uma vez por caso, e rodar um caso de um teste com `::` no nome diz que o ctest não o lista | Corrigido em 2026-11-10 ([§21.1](#211-c1-32-integração-e-provas--2026-11-10), protocolo 0.215.0). Decisão do autor de 2026-11-10: corrigir logo depois do C1-22, num commit próprio. Um teste do ctest cujo comando já filtra um caso não é expandido, e o id acha o nome do teste do ctest conhecido, em vez de cortar no primeiro `::`. Segunda decisão do autor, de 2026-11-10: o id que já não acha um caso só é recusado com o motivo, seja o caso que o binário não lista mais (o gtest 1.18 diz `PASSED 0 tests` e sai com 0), seja o id que dois testes dão; a descoberta deixa de fora o id repetido |
| C1-33 / faixa cortada sem como ler | Achado na prova na UI nativa do C1-22 (2026-11-10), num Xvfb a 900x600. `ui/qml/shell/ProjectHealthBanner.qml` corta a mensagem com `Text.ElideRight` e não tem dica: a faixa do multi-config termina em "o cmake --buil…", e pairar não mostra o resto. Vale para toda mensagem longa da faixa, como a da CDB velha com o motivo. Causa: o texto cortado não tem outro lugar | Baixa: numa janela estreita, a faixa esconde o fim da mensagem, que às vezes é o que diz o que fazer | Corrigido em 2026-11-10 ([§20.1](#201-c1-22-integração-e-provas--2026-11-10)), provado no harness e na UI nativa. Decisão do autor de 2026-11-10: corrigir agora, junto do C1-22. Quando a mensagem é cortada, pairar sobre ela mostra a mensagem inteira numa dica. Provar no harness e na UI nativa |
| C1-34 / caso de dentro sem o ambiente e os argumentos do teste | Achado no desenho do C1-32 (2026-11-10), por leitura de `test::frameworks`: `parse_ctest_json` lê do `--show-only=json-v1` só o `WORKING_DIRECTORY`, e `run_inner_case` roda o binário com o filtro do caso e nada mais. O CTest do CMake 4.4.5 dá no json-v1 o `ENVIRONMENT` (`["K=V", …]`) e o `ENVIRONMENT_MODIFICATION` (`["K=set:V", …]`) do teste, que ele aplica ao rodar, e o comando leva os argumentos do `add_test`. Causa: o caso de dentro é rodado pelo binário, fora do CTest, e só a pasta passava | Média: um teste que lê o ambiente ou um argumento do `add_test`, como a pasta dos dados, passa na linha do ctest e falha, ou faz outra coisa, ao rodar um caso dele pela árvore | Corrigido em 2026-11-10 ([§22.1](#221-c1-34-integração-e-provas--2026-11-10), protocolo 0.216.0). Decisão do autor de 2026-11-10: corrigir num commit próprio, logo depois do C1-32. Medido o `ENVIRONMENT_MODIFICATION` ([§22](#22-c1-34-o-caso-como-o-ctest-roda-o-teste--desenho-de-2026-11-10)), o autor decidiu em 2026-11-10: todos os argumentos do `add_test`, e as flags da IDE por último; no Catch2, o `-r compact` só quando os argumentos não escolhem um reporter; o ambiente que o CTest dá, também na listagem, que não leva os argumentos |
| C1-35 / caso gtest desativado que diz que passou | Achado na medida do C1-32 com o gtest 1.18 real (2026-11-10). O `--gtest_list_tests` lista `Text.DISABLED_Off`, e a árvore mostra o caso. Rodá-lo com `--gtest_filter=Text.DISABLED_Off` não roda nada, imprime `YOU HAVE 1 DISABLED TEST` e sai com 0; a IDE não vê linha de caso e diz sucesso, com zero aprovados. O `gtest_discover_tests` dá a esse caso a propriedade `DISABLED`, e o CTest escreve `***Not Run (Disabled)` e sai com 0; `test::parse::parse_ctest_case` lê todo `***Not Run` como falha, e a linha do ctest sai reprovada num job que diz sucesso. Com `GTEST_BRIEF=1` no ambiente, o gtest não escreve a linha do caso que passou, e a IDE também conta zero. Causa: o gtest pula o desativado sem `--gtest_also_run_disabled_tests`, a IDE lê o código de saída, e o `Not Run` do desativado não é o de uma falha | Baixa: o caso desativado aparece como aprovado sem ter rodado, ou como reprovado na linha do ctest | Corrigido em 2026-11-10 ([§21.1](#211-c1-32-integração-e-provas--2026-11-10), protocolo 0.215.0). Decisão do autor de 2026-11-10: contar como ignorado, com o motivo, como o gtest e o CTest fazem; corrigir junto do C1-32 |
| C1-36 / testes do 1 ao 9 fora da árvore | Achado na prova do C1-32 com o CTest do CMake 4.4.5 (2026-11-10). Com dez testes ou mais, o CTest alinha o número: o `ctest -N` escreve `  Test  #1: t1` e a execução, ` 1/11 Test  #1: t1 ...   Passed`. `test::discover::parse_ctest_list_line` pede `Test #` no começo da linha, e `test::parse::parse_ctest_case` procura `Test #`. O `-N` também escreve `t2 (Disabled)` para o desativado. Causa: a leitura do texto do CTest seguia a forma de menos de dez testes, e o nome exato já vinha no `--show-only=json-v1` | Alta: num projeto CMake com dez testes ou mais, os nove primeiros somem da árvore, e rodar todos não conta o desfecho deles; com cem ou mais, os 99 primeiros | Corrigido em 2026-11-10 ([§21.1](#211-c1-32-integração-e-provas--2026-11-10), protocolo 0.215.0). Achado com a decisão do autor de 2026-11-10 de incluir as correções no commit do C1-32: os nomes da árvore saem do `json-v1`, e a leitura do `Test #N:` aceita o número alinhado |
| C1-37 / desfecho do caso de dentro fora da linha dele | Achado no preparo da prova na UI do C1-32 (2026-11-10), por leitura. `ui/qml/jobs/JobsController.qml::handleTestCase` pinta a linha da árvore cujo **id** é o `name` do `event.test.case`, como no cargo, no pytest e no ctest, em que o nome do caso é o id. `test::frameworks::run_inner_case` manda o nome de dentro do binário, `Math.Adds`, e o id da linha é `ns::unit::Math.Adds`. Causa: o caso de dentro tem um id que não é o nome dele, e o evento levava o nome | Média: o desfecho do caso de dentro não pinta a linha dele e vai para a lista de casos soltos; com o `gtest_discover_tests` ao lado, pinta a linha do ctest de mesmo nome, que não rodou | Corrigido em 2026-11-10 ([§21.1](#211-c1-32-integração-e-provas--2026-11-10), protocolo 0.215.0). Achado com a decisão do autor de 2026-11-10 de incluir as correções no commit do C1-32: o `event.test.case` do caso de dentro leva o id da árvore, `teste::caso` |
| C1-38 / comando do caso gtest que não é o que roda | Achado na prova na UI nativa do C1-32 (2026-11-10). `test::frameworks::run_inner_case` roda o gtest com `--gtest_filter=Math.Adds --gtest_color=no --gtest_print_time=1 --gtest_brief=0`, e o `event.test.started` diz só `…/gt --gtest_filter=Math.Adds`; o caminho do executável vai sem aspas, e o do projeto da prova tem espaços. Já faltavam duas flags antes do C1-32, que pôs a terceira. Causa: o texto mostrado é montado à parte dos argumentos | Baixa: colado num terminal, o comando mostrado não roda, ou roda outra coisa: com o `brief` ligado no `main`, esconde a linha do caso que passou | Corrigido em 2026-11-10 ([§22.1](#221-c1-34-integração-e-provas--2026-11-10), protocolo 0.216.0), junto do C1-34, que decide quais argumentos acompanham o filtro do caso: o comando mostrado sai dos mesmos argumentos, com aspas onde o shell pede ([§22](#22-c1-34-o-caso-como-o-ctest-roda-o-teste--desenho-de-2026-11-10)) |
| C1-39 / árvore que não distingue o caso de dentro nem o ignorado | Achado na prova na UI nativa do C1-32 (2026-11-10). `ui/qml/panels/bottom/TestsPanel.qml` mostra o `name` da linha: o caso `ns::unit::Math.Adds` aparece como `Math.Adds`, igual à linha do ctest do `gtest_discover_tests`, e os casos de `ns::unit` e de `gt_color` aparecem repetidos. `statusColor` pinta `ignored` com a cor do que não rodou | Baixa: na árvore, não se sabe de que teste é um caso de dentro, e o caso ignorado parece não ter rodado; o resumo diz `ignorado: 1` | Corrigido em 2026-11-10 ([§23.1](#231-c1-39-e-c1-44-integração-e-provas--2026-11-10), protocolo 0.217.0). Decisão do autor de 2026-11-10: a linha mostra o id inteiro, `ns::unit::Math.Adds`, o texto que o "Rodar só este" usa, sem campo novo no protocolo; o ponto do ignorado é âmbar, o `warningSoft`. Corrigir num commit próprio, depois do C1-34, e provar no harness e na UI nativa |
| C1-40 / caso de dentro de um teste cujo desfecho o CTest decide de outro jeito | Achado na medida do C1-34 com o CTest do CMake 4.4.5 (2026-11-10). Num teste escrito à mão que roda o binário gtest inteiro, com um caso que falha: o `WILL_FAIL` faz o CTest dizer `Passed`; o `PASS_REGULAR_EXPRESSION` que não aparece reprova; o `SKIP_RETURN_CODE 1` dá `Skipped`; o `DISABLED` dá `Not Run (Disabled)`; o `FIXTURES_REQUIRED` roda antes o teste do `FIXTURES_SETUP`, também com `-R`; e o `REQUIRED_FILES` que falta dá `Not Run`. O `json-v1` traz cada uma só quando ligada: o `WILL_FAIL FALSE` e o `DISABLED OFF` não aparecem. O `test.discover` abre esses testes em casos, e o caso roda pelo binário e sai pelo código de saída dele. Causa: o desfecho do CTest não é só o código de saída, e a regra do C1-32 olhava só o que escolhe os casos | Média: o caso de um teste `WILL_FAIL` sai reprovado quando a linha do ctest passa; o de um teste com fixture falha por falta do preparo; o de um teste desativado roda | Corrigido em 2026-11-10 ([§22.1](#221-c1-34-integração-e-provas--2026-11-10), protocolo 0.216.0). Decisão do autor de 2026-11-10: não abrir em casos, com a mesma regra do C1-32, junto do C1-34; o `TIMEOUT` do teste passa a valer para o caso ([§22](#22-c1-34-o-caso-como-o-ctest-roda-o-teste--desenho-de-2026-11-10)) |
| C1-41 / comando mostrado da linha do ctest que não roda | Achado no desenho do C1-34 (2026-11-10), por leitura de `test::runners::ctest_command`. O `event.test.started` diz `ctest --output-on-failure -R ^Widget::draw works$`: sem o `--test-dir` da pasta da configuração, que o `ctest` recebe, e sem aspas no nome com espaço do `catch_discover_tests`; o `ctest` roda na raiz do projeto. Causa: o texto é montado à parte dos argumentos, como no C1-38 | Baixa: colado num terminal na raiz, o comando mostrado não acha os testes, ou roda outros | Corrigido em 2026-11-10 ([§22.1](#221-c1-34-integração-e-provas--2026-11-10), protocolo 0.216.0). Decisão do autor de 2026-11-10: corrigir junto do C1-34. O comando mostrado sai dos mesmos argumentos, com o `--test-dir` e as aspas do C1-38; o `ctest -N` da descoberta, que também vinha sem o `--test-dir`, entra junto ([§22](#22-c1-34-o-caso-como-o-ctest-roda-o-teste--desenho-de-2026-11-10)) |
| C1-42 / linha do ctest que cai ou passa do prazo sem contar | Achado na regressão do C1-34 com o CTest do CMake 4.4.5 (2026-11-10). A linha do teste que cai é `***Exception: SegFault`, `***Exception: Numerical`, `***Exception: Illegal` ou `Subprocess aborted***Exception:`, com `terminated` e `killed` no lugar de `aborted` para o `SIGTERM` e o `SIGKILL`; a do que passa do `TIMEOUT` é `***Timeout`. `test::parse::parse_ctest_case` só lê `Passed`, `***Failed`, `***Not Run`, `***Skipped` e `***Disabled`, e essas linhas não viram `event.test.case`. Causa: as formas de falha do CTest que não são `***Failed` ficaram fora da leitura | Alta: num projeto C, o teste que cai por `SegFault` ou trava até o prazo não fica vermelho na árvore nem conta como reprovado; só o código de saída do job diz falha | Corrigido em 2026-11-10 ([§22.1](#221-c1-34-integração-e-provas--2026-11-10), protocolo 0.216.0). Decisão do autor de 2026-11-10: corrigir junto do C1-34; as linhas medidas viram `failed` |
| C1-43 / listagem de um binário que não termina | Achado no desenho da prova do C1-34 (2026-11-10), por leitura de `test::frameworks::list_cases`: a listagem roda o binário com `Command::output()`, que espera o processo sem prazo e sem olhar o `cancel` do job. Um programa comum posto como teste, como um servidor que ignora `--gtest_list_tests` e `--libidentify` e fica rodando, não termina. Causa: a listagem é uma chamada direta, fora do `process` que os jobs usam | Média: o `test.discover` desse projeto não termina, e cancelar não o para | Corrigido em 2026-11-11 ([§24.1](#241-c1-43-integração-e-provas--2026-11-11), protocolo 0.218.0). Decisão do autor de 2026-11-10: corrigir num commit próprio, depois do C1-39: a listagem pelo mesmo processo dos jobs, com o `cancel` e um prazo; o cancel dá "listagem cancelada"; 5 s e 64 MiB por chamada ([§24](#24-c1-43-a-listagem-dos-casos-com-prazo--desenho-de-2026-11-10)) |
| C1-44 / árvore dos testes que volta ao topo ao rodar | Achado na prova na UI nativa do C1-34 (2026-11-10), com 17 linhas na árvore e o painel alto: rolada até os casos, a árvore volta ao topo depois do ▶, nas duas vezes medidas, e o caso que rodou sai da vista. Causa provável, por leitura: `ui/qml/jobs/JobsController.qml::prepareTestRun` zera a saída, o `hasOutput` de `ui/qml/panels/bottom/TestsPanel.qml` vira falso, a lista cresce até caber inteira e o `contentY` vai a 0; quando a saída volta, a lista encolhe e fica no topo | Baixa: para ver o desfecho do caso, é preciso rolar de novo a cada execução | Corrigido em 2026-11-10 ([§23.1](#231-c1-39-e-c1-44-integração-e-provas--2026-11-10), protocolo 0.217.0). Decisão do autor de 2026-11-10: corrigir junto do C1-39, que mexe no mesmo painel: a árvore fica onde estava ao rodar. Provar no harness e na UI nativa |
| C1-45 / motivo da listagem que não se vê na UI | Achado na prova na UI nativa do C1-43 (2026-11-11). O motivo do binário que não respondeu no prazo ou passou do teto sai só no `event.job.output` do `test.discover`. A aba Testes mostra a árvore e o resumo, sem a saída da listagem, e a aba Jobs mostra só a última linha de cada job, que é a do `ctest -N` ou a do fim. Causa: a saída da descoberta só vai para o log do job, e nenhuma vista da UI mostra esse log inteiro | Baixa: a linha do servidor aparece sem casos, e a UI não diz por quê; o motivo está no log do job e, no run do caso, no erro | Aberto. Registrado; fora do C1-43 por decisão do autor de 2026-11-11, que mandou fechar o escopo atual. Decisão do autor de 2026-11-11: mostrar as linhas da listagem na aba Testes |

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

## 19. C1-26: `matches` e `notMatches` sem o CMake — desenho de 2026-11-10

Com o C1-25, a lista do CMake do projeto já decide `matches` e `notMatches`: um
preset de `condition` falsa para ele sai da lista com
`a condition do preset X é falsa para o CMake`. Restam dois casos.

Medido com o CMake 4.4.5, em 16 projetos de um preset `ok` e um `c` com a
`condition` do caso:
- o `anyOf` para no primeiro verdadeiro e o `allOf` no primeiro falso, também
  quando quem decide é um `matches` ou um `notMatches`. Depois disso, uma macro
  inválida ou uma expressão que não compila não recusa nada: `anyOf` de um
  `matches` verdadeiro e de `${nope}` lista `c`; `allOf` de um `matches` falso
  e de `${nope}` lista só `ok`;
- quando o CMake chega na forma inválida, recusa o arquivo com
  `Invalid configure preset: "c": Invalid condition`, também a macro inválida
  dentro do `string` ou da `regex`, e também num preset oculto;
- a expressão só é compilada quando o CMake chega nela;
- a macro dentro da `regex` expande antes: `^${hostSystemName}$` casa `Linux`.

A leitura nativa não decide essas formas, porque a expressão regular é a do
CMake, e a IDE não cria uma própria. Hoje ela segue avaliando
depois de uma forma indecisa e recusa os presets pela macro inválida que o
CMake talvez nem leia. Com o C1-25, isso aparece como
`o CMake aceita os presets, mas a IDE não os lê`, mesmo quando o CMake aceita.

Decisões do autor de 2026-11-10:
- num `anyOf` ou `allOf`, depois de uma forma indecisa, uma macro inválida ou
  um `$vendor{}` deixam a `condition` indecisa, e não recusam nem ignoram o
  preset: o CMake só chega neles se a forma indecisa não decidir. Uma forma
  depois dela que decide sozinha (verdadeira no `anyOf`, falsa no `allOf`)
  continua decidindo. A `condition` de tipo desconhecido, que a leitura nativa
  já dá por verdadeira (C1-25), não muda;
- sem resposta do CMake, o preset de `condition` indecisa continua na lista,
  como verdadeiro, e a IDE diz isso na dica do item dele no menu de
  configurações.

Desenho:
- `expansion.rs`: a avaliação do `anyOf` e do `allOf` passa a parar indecisa no
  primeiro erro ou `$vendor{}` depois de uma forma indecisa;
- `verdict.rs`: quando o CMake lista, as formas indecisas ficam decididas por
  ele, e a anotação sai. Quando não há resposta, ela fica;
- `cmake.presets.list` e `cmake.status.configurations` ganham `conditionNote`,
  só para um preset de `condition` indecisa sem resposta do CMake:
  `a condition usa matches ou notMatches, que só o CMake decide; sem ele, a IDE
  a dá por verdadeira`. Protocolo 0.213.0, com o contrato no
  [03 cmake](../arquitetura/03-dominios/cmake.md) antes da mudança;
- `CmakeConfigurationChoices.qml` acrescenta a nota à dica do item.

Provas necessárias:
- os casos medidos, pela leitura nativa e com o CMake real: o erro depois da
  forma indecisa sem recusa, a forma que decide depois dela, e o erro antes de
  qualquer forma indecisa ainda recusando;
- a nota só sem resposta do CMake, na lista e no menu, de ponta a ponta;
- o harness da dica e a prova na UI nativa, com o projeto sem CMake no `PATH`;
- mutantes que compilem e reprovem pela guarda.

Limites previstos:
- sem o CMake, a expressão que não compila não é recusada: a IDE não compila
  a expressão do CMake;
- um `$vendor{}` depois da forma indecisa deixa a `condition` indecisa. Com o
  CMake, a lista dele decide; sem ele, o preset aparece com a nota.

### 19.1 C1-26: integração e provas — 2026-11-10

O desenho do §19 entrou como previsto, no mesmo dono, `cmake_presets`:
- `expansion.rs`: num `anyOf` ou `allOf`, depois de uma forma indecisa, a
  avaliação para indecisa no primeiro erro ou `$vendor{}`. Uma forma que decide
  sozinha (verdadeira no `anyOf`, falsa no `allOf`) continua decidindo;
- `verdict.rs`: quando o CMake lista, a anotação das formas indecisas sai,
  depois de dar o motivo dos presets que ele não lista;
- `reading.rs` dá o texto da nota, só ao preset de `condition` indecisa;
- `cmake_presets.rs` e `cmake_configuration.rs` passam a nota a
  `cmake.presets.list` e a `cmake.status.configurations`, no protocolo 0.213.0;
- `CmakeConfigurationChoices.qml` acrescenta a nota à dica do item, na linha
  depois da pasta ou do motivo dela.

A ponte C++ já passava os itens de `configurations` como mapas, e o campo novo
chega ao QML sem mudança nela.

Provas:
- testes do dono, em `expansion/tests.rs`, pela leitura nativa. Depois de uma
  forma indecisa, a macro inválida no `anyOf` e no `allOf`, a macro inválida
  dentro da `regex`, o `$vendor{}` e o caso aninhado deixam o preset na lista,
  com a nota. Depois dela, o `true` no `anyOf` decide, sem nota, e o `false` no
  `allOf` desabilita. O `$vendor{}` antes da forma indecisa ainda ignora o
  preset, e a macro inválida antes dela ainda recusa os presets;
- em `verdict/tests.rs`, contra o CMake 4.4.5 real, os 16 casos medidos no §19:
  o CMake decide cada um, com a recusa dele, o motivo de fora da lista ou a
  lista, e nenhum preset leva a nota;
- regressão de ponta a ponta pelo core, com CMake 4.4.5, GCC 16.2.1 e Ninja
  reais, em `tests/c_project/preset_condition.rs`. Com o CMake, a lista e o
  menu têm `ok` e o `anyOf` que ele para no `matches` verdadeiro, sem nota. O
  configure desse preset escreve a CDB, e o do `matches` falso é recusado com o
  motivo, sem job. Sem resposta do CMake, a lista e o menu têm os três, os dois
  indecisos com a nota. Quando o CMake volta a responder, voltam à lista dele;
- o harness `tst_configuration_menu` confere a nota depois da pasta e depois do
  motivo da pasta desconhecida, o detalhe sem mudança e a dica na tela ao
  pairar;
- prova na UI nativa, num Xvfb com HOME isolado, o projeto com `autoSave`
  desligado e um `PATH` sem `cmake`. O Project Health acusa o `cmake` ausente,
  o menu tem `ok`, `rx` e `anybad`, e a dica de `rx` e de `anybad` tem a pasta
  e, na linha seguinte, a nota; a de `ok`, só a pasta. Numa janela de 900 px, o
  chip vai para o ⋯, e a dica se desloca para caber. Com o `cmake` no `PATH`, o
  menu tem só `ok` e `anybad`, sem nota. Saída 0 e nenhum aviso QML;
- 18 mutantes que compilam, todos reprovados na primeira rodada: sem o
  curto-circuito, só no erro, só no `$vendor{}`, também antes da forma
  indecisa e para qualquer valor depois dela; a anotação que fica com o CMake,
  ou que sai antes do motivo de fora da lista; a nota nunca, sempre ou de
  qualquer indeciso; a lista ou o menu sem a nota; a nota nula gravada na
  lista ou no menu; e, no QML, a dica sem a nota, com ela antes da pasta, na
  mesma linha ou no lugar da pasta.

Limites:
- sem o CMake, a expressão que não compila não é recusada: a IDE não compila a
  expressão do CMake;
- sem o CMake, o preset com `$vendor{}` depois da forma indecisa aparece com a
  nota, e o CMake talvez o ignore;
- a nota fica na dica do menu. A lista de presets do Project Health e a saída
  do configure não a repetem;
- segue aberto o C1-22.

## 20. C1-22: multi-config — desenho de 2026-11-10

Decisão do autor de 2026-11-09 (§17): com gerador multi-config, a IDE usa a
configuração que o clangd lê.

Medido com o CMake 4.4.5, o Ninja 1.13.2 e o clangd 23.1.4, em oito projetos
Ninja Multi-Config e um Ninja de configuração única:
- a CDB tem, para cada arquivo, uma entrada por configuração, na ordem de
  `CMAKE_CONFIGURATION_TYPES`, e o clangd usa a primeira. O codemodel tem as
  configurações na mesma ordem. Com `Release;Debug`, o clangd lê a de
  Release; com `Fast;Debug`, a de Fast;
- `cmake --build` sem `--config` compila `CMAKE_DEFAULT_BUILD_TYPE`, senão a
  primeira configuração; com `CMAKE_DEFAULT_CONFIGS=all`, todas;
- o `ctest` sem `-C` dá todo teste como "Not Run" e sai com 8, também com uma
  configuração só. O `--show-only=json-v1` sem `-C` não lista nenhum teste, e
  o `-N` lista só os que não pedem configuração. Com `-C Debug`, o teste roda,
  e o comando dele é o executável de `bin/Debug`;
- o codemodel traz os artefatos de cada configuração (`bin/Debug/app`,
  `bin/Release/app`);
- um Ninja de configuração única aceita `CMAKE_CONFIGURATION_TYPES` no cache e
  o ignora: o build e o ctest seguem o `CMAKE_BUILD_TYPE`. Quem diz se o
  gerador é multi-config é o `CMAKE_GENERATOR` do cache (o index do file-api
  traz `multiConfig`);
- com `CMAKE_CONFIGURATION_TYPES` vazio, o build falha no próprio CMake.

Na IDE, hoje, o build compila a configuração padrão do CMake, que pode não ser
a que o clangd lê. Os testes não rodam, e a descoberta por framework fica
vazia. O run e o debug acham um executável por configuração compilada e
recusam por haver mais de um.

Decisões do autor de 2026-11-10:
- quando o `CMAKE_DEFAULT_BUILD_TYPE` difere da configuração que o clangd lê,
  a faixa mostra isso como informativa, sem ação: o `cmake --build` sem
  `--config` do terminal compila outra;
- a gravação nativa mantém a recusa do build multi-config: ela fecha na etapa
  5 (C6), e o motivo atual já diz o que fazer;
- o run e o debug num build multi-config sem o modelo do file-api (configurado
  fora da IDE) recusam com o motivo e pedem o configure pela IDE;
- achado na regressão: o `index.context` mostra a última entrada repetida da
  CDB (Debug), e o clangd, a primeira (Release). Corrigir agora: na entrada
  repetida, a IDE fica com a primeira, como o clangd. O resto do C-03, guardar
  todas as unidades com a identidade delas, segue no C2;
- também achado na regressão, o C1-28 (Catch2 por qualquer saída) é corrigido
  agora, com o Catch2 real que o autor instala.

Desenho:
- `cmake::build_configuration` lê o cache da pasta: com gerador multi-config
  (`Ninja Multi-Config`, `Xcode` ou `Visual Studio …`), a configuração é o
  primeiro item não vazio de `CMAKE_CONFIGURATION_TYPES`, e a padrão é
  `CMAKE_DEFAULT_BUILD_TYPE`. Sem gerador multi-config, nada muda;
- o build passa `--config` com ela; o `ctest -N`, o `--show-only=json-v1` e a
  execução passam `-C`;
- o run e o debug procuram o executável nos artefatos dos alvos
  `EXECUTABLE` do codemodel. O modelo continua lido na primeira configuração,
  que o CMake escreve na ordem de `CMAKE_CONFIGURATION_TYPES` (medido): é a
  mesma, e o "lido por nome" do §17 não muda o resultado;
- `cmake.status` ganha `buildConfiguration`, só com gerador multi-config, e
  `defaultBuildConfiguration`, só quando ela difere. Protocolo 0.214.0, com o
  contrato no [03 cmake](../arquitetura/03-dominios/cmake.md) antes da
  mudança. A ponte leva os dois no mapa `configuration`, e a faixa mostra a
  diferença como informativa;
- `index/context/cdb.rs::parse_cdb`: a primeira unidade de um arquivo
  repetido fica, marcada `ambiguous`, como antes; os headers e o verificador
  continuam recusando a unidade ambígua.

Provas necessárias:
- testes do dono para cada gerador, a lista com itens vazios, a padrão igual e
  diferente e o Ninja de configuração única com `CMAKE_CONFIGURATION_TYPES`;
- regressão de ponta a ponta pelo core, com Ninja Multi-Config, GCC e o CTest
  reais, `CMAKE_CONFIGURATION_TYPES=Release;Debug` e
  `CMAKE_DEFAULT_BUILD_TYPE=Debug`: o status, o build só de Release, o teste
  que passa, o run e o debug no executável de Release com o de Debug também
  compilado, e a recusa sem o modelo; o contexto com a definição de Release;
  o Ninja de configuração única sem `--config`;
- o harness da faixa e a prova na UI nativa;
- mutantes que compilem e reprovem pela guarda.

Limites previstos:
- trocar de configuração numa pasta multi-config fica para o C-03 (CDB 1:N);
- a faixa compara só o `CMAKE_DEFAULT_BUILD_TYPE`. Com `CMAKE_DEFAULT_CONFIGS`,
  o terminal compila outras também;
- os build presets com `configuration` não são lidos: a IDE não usa build
  presets;
- `Xcode` e `Visual Studio` não foram medidos nesta máquina.

### 20.1 C1-22: integração e provas — 2026-11-10

O desenho do §20 entrou como previsto:
- `cmake/build_configuration.rs` lê o cache como o CMake: sem os brancos do
  fim do valor e com os do começo. Medido com o CMake 4.4.5: o
  `-DCMAKE_CONFIGURATION_TYPES=" Release;Debug "` grava ` Release;Debug`, e o
  codemodel e a pasta se chamam ` Release`. O primeiro desenho cortava os dois
  lados, e um mutante que não cortava nada sobreviveu; o build (`build/cmake.rs`), o
  `ctest -N` (`test/discover.rs`), o `--show-only=json-v1`
  (`test/frameworks.rs`) e a execução (`test/runners.rs`) passam a
  configuração dele;
- `run::cmake_executables`, que o run e o debug usam, lê os artefatos dos
  alvos `EXECUTABLE` do codemodel e recusa o multi-config sem o modelo;
- `index/context/cdb.rs::parse_cdb` fica com a primeira entrada repetida. O
  teste antigo que pedia a última, de 2026-10-19, passa a pedir a primeira,
  com o mesmo caminho escapado e o mesmo prefixo não ASCII. O contrato do
  `index.context` diz isso desde o 0.214.0;
- `handlers/cmake_configuration.rs` dá os dois campos ao `cmake.status`, a
  ponte os leva no mapa `configuration`, e
  `CmakeConfigurationChoices.qml::buildMismatch` dá o texto da faixa
  informativa ao `ProjectHealthController`, depois dos avisos da CDB e do
  Python;
- `is_executable` passou de `run` para `platform`: o `run` agora lê o `cmake`,
  e o `toolchain`, que usava o do `run`, fecharia um ciclo de módulos.

Achados na regressão, registrados antes da correção e corrigidos com decisão
do autor:
- C1-28: o Catch2 se identifica. `list_cases` só pede a lista ao binário que
  responde `framework: Catch2` a `--libidentify`, na forma `chave: valor`
  alinhada. O gtest real responde ao `--libidentify` rodando todos os testes
  (gtest 1.18, medido); como ele é perguntado antes, não chega a isso;
- C1-29: o caso gtest roda com `--gtest_print_time=1`, e
  `parse_gtest_case` só aceita a linha que termina no tempo, sem o
  `, where …`. Medido com o gtest 1.18: o resumo repete o `[  FAILED  ]` e o
  `[  SKIPPED ]` sem o tempo, o `GTEST_PRINT_TIME=0` tira o tempo de todas as
  linhas, e o `--gtest_print_time=1` vence o ambiente;
- C1-30: o single-config também lê o codemodel. Sem o modelo, a varredura da
  pasta deixa de fora `.so`, `.so.N` e `.dylib` pelo nome;
- C1-31: o caso Catch2 roda com a barra invertida antes de `\`, `,`, `[`,
  `]`, `*`, `~` e `"`, e a lista sai com `--order decl`. Medido com o Catch2
  3.16.1: sem escape, `~tilde` e `exclude:this` rodavam os nove casos, e
  `a, b`, `x [y]` e `back\slash`, nenhum; com o escape, cada nome roda só o
  caso dele. O prefixo `exclude:`, só em minúsculas, não se escapa, e o
  Catch2 compara os nomes sem a caixa ASCII: `case` roda `Case`, e `Ação` não
  roda `aÇão`. Nesses dois casos, o `test.run` recusa com o motivo
  (`TestError::Refused`), sem rodar o binário;
- C1-33: a `ProjectHealthBanner` mostra a mensagem inteira numa dica, ao
  pairar, quando a corta. A área só paira, e o clique segue para baixo.

Achado na prova do C1-31 e registrado, o C1-32 fica para o commit seguinte,
por decisão do autor.

Provas:
- testes do dono: os geradores, a lista com item vazio, a padrão igual,
  diferente e vazia, o Ninja de configuração única, a chave de prefixo igual
  (`CMAKE_GENERATOR_INSTANCE`) e o cache ausente; a entrada repetida da CDB; o
  `--libidentify` alinhado, o programa que imprime uma linha para qualquer
  argumento e o que se diz `framework: Catch`, com `description: Catch2`; as
  linhas do gtest, com o resumo sem o tempo, os parametrizados e tipados, o
  tempo que não é número e o parêntese no parâmetro; os nomes de biblioteca
  compartilhada; o argv exato do caso gtest; o filtro do Catch2 para cada
  caractere especial e para um nome não ASCII, e as recusas, com o
  `EXCLUDE:` maiúsculo e o `Ação` ao lado de `aÇão` sem recusa; o falso Catch2
  só lista quando recebe `--order decl`; os brancos do cache;
- regressão de ponta a ponta pelo core, com CMake 4.4.5, Ninja Multi-Config,
  GCC e CTest reais, em `tests/c_project/configuration_types.rs`. Com
  `Release;Debug` e a padrão `Debug`: o status traz Release e Debug; o
  contexto tem `KV_KIND="Release"`; o build é `cmake --build --config Release`
  e só gera o de Release; o `ctest` roda os dois testes, um só de Release; a
  descoberta mostra `ctest -N -C Release` e os dois nomes; o `json-v1` aponta
  os dois para o de Release. Com o Debug também compilado pelo terminal, o run
  imprime `kv-run-Release` e o debug resolve o de Release; sem o modelo, os
  dois recusam com o motivo exato. No Ninja de configuração única, os campos
  não vêm, o build vai sem `--config`, e o run e o debug acham só o
  executável, com duas bibliotecas compartilhadas e um script do configure
  com o bit de execução na pasta. Sem o modelo, a recusa lista o executável e
  o script, e não as bibliotecas;
- em `tests/c_project/inner_cases.rs`, com o gtest 1.18 e o Catch2 3.16.1
  reais: o programa que ecoa os argumentos fica só a linha do ctest; o gtest
  dá os cinco casos, os dois parametrizados com o nome da lista; rodar um
  caso conta um aprovado, um reprovado ou um ignorado, e o parametrizado que
  falha sai com o nome da lista. O `main` desse gtest desliga o tempo, como o
  `GTEST_PRINT_TIME=0` faria. O Catch2 dá os oito casos na ordem da
  declaração, sem o oculto. Cada caso imprime o próprio nome, e rodar
  `a, b [x]`, `~tilde` e `back\slash` roda só ele; `exclude:this` e `case`
  são recusados com o motivo exato;
- o harness `tst_project_health` confere a faixa informativa sem ação com o
  texto exato, a CDB velha antes dela, a falta da padrão e a troca de pasta.
  O `tst_project_health_banner`, novo, confere a dica da mensagem cortada, a
  falta dela com a mensagem inteira, o corte com o botão de ação e a área que
  não pega o clique;
- prova na UI nativa, em duas sessões num Xvfb com HOME isolado e o projeto
  Ninja Multi-Config com `autoSave` desligado, `Release;Debug`, a padrão
  `Debug` e uma biblioteca compartilhada. Confiar no projeto configura, e a
  faixa fica azul, com o texto exato e sem botão. O ▶ antes do build pede o
  build; o Ctrl+F9 roda `cmake --build --config Release` e só gera o de
  Release; o ▶ roda `build/multi/Release/multi`, que imprime
  `kv-run-Release`, com a biblioteca ao lado. Numa janela de 900 px, a faixa
  corta a mensagem, e pairar mostra a mensagem inteira numa dica, que some ao
  sair; a 1366 px, sem dica. O × fecha com saída 0, e nenhuma das duas sessões
  registrou aviso QML;
- 69 mutantes que compilam, todos reprovados. Na primeira rodada, 47 de 49:
  o mutante que tirava o `--libidentify` não compilava, e o que lia o cache
  sem cortar os brancos sobreviveu, o que mostrou que o `trim()` não lia como
  o CMake. Na segunda, os dois, o corte dos dois lados e 13 do C1-31; na
  terceira, os três da extensão da biblioteca sem caixa; e 5 da dica do
  C1-33. São 56 no core e no protocolo e 13 no QML: o gerador, a primeira
  configuração, a padrão, a chave e os brancos do cache; o `--config` e o
  `-C` em cada um dos três lugares; o modelo, a recusa sem ele, o filtro
  `EXECUTABLE` e os nomes de biblioteca; o debug; os campos do status e do
  contrato; a entrada repetida da CDB; o `--libidentify`; o tempo, o resumo e
  o `, where` do gtest; o escape de cada caractere, a ordem e as recusas do
  Catch2; a faixa, a troca de pasta e a dica.

Limites:
- os do §20;
- o Catch2 não roda sozinho um caso de nome com `exclude:` no começo, nem um
  de dois nomes iguais sem a caixa ASCII: a IDE recusa, e a linha do ctest
  roda o binário inteiro;
- o escape do `]` não muda o que o Catch2 3.16.1 roda (medido), e não tem
  mutante;
- o C1-32 segue aberto: com `gtest_discover_tests` ou `catch_discover_tests`,
  a árvore repete os casos;
- sem o modelo, a varredura do single-config não distingue um script com o
  bit de execução de um programa;
- o `.dylib` e o `.exe` do Windows não foram medidos nesta máquina;
- a ponte C++ é provada pela UI nativa, sem mutante próprio.

## 21. C1-32: testes que já são casos — desenho de 2026-11-10

Decisões do autor de 2026-11-10: o teste do ctest cujo comando já filtra um
caso não é expandido, e o id acha o nome do teste do ctest conhecido em vez de
cortar no primeiro `::`, num commit próprio. O id que já não acha um caso só é
recusado com o motivo. O C1-35 entra junto, e o caso desativado conta como
ignorado. O C1-34 fica para o commit seguinte.

Medido com o CMake 4.4.5, o CTest dele, o gtest 1.18 e o Catch2 3.16.1:
- o `gtest_discover_tests` escreve um teste do ctest por caso, com o comando
  `<exe> --gtest_filter=<caso> --gtest_also_run_disabled_tests`, mais o
  `EXTRA_ARGS`. O nome do caso desativado perde o `DISABLED_` e o teste ganha
  a propriedade `DISABLED`;
- o `catch_discover_tests` escreve um teste por caso, com o comando `<exe>
  <nome escapado> [EXTRA_ARGS] [reporter] "--order decl"`, este último num
  argumento só. O nome do teste leva o `TEST_PREFIX` e o `TEST_SUFFIX`, e o
  escape põe a barra antes de `\`, `,`, `[`, `]` e `;`;
- o gtest escolhe os casos pelo `--gtest_filter=`, pelo `--gtest_flagfile=` e
  pelo `--gtest_list_tests`. O `--gtest_filter Math.Adds`, com espaço, e o
  `-gtest_filter=` imprimem a ajuda e não rodam nada. No ambiente, lê o
  `GTEST_FILTER` também na listagem, e o `GTEST_TOTAL_SHARDS` com o
  `GTEST_SHARD_INDEX` só ao rodar; o `GTEST_FLAGFILE` ele não lê;
- o Catch2 lê os argumentos com o Clara. Um argumento que não começa com `-`
  é um test spec; `--order decl`, `--order=decl` e `--order:decl` são a opção
  com o valor; `-sa` é `-s -a`, e `-rcompact` é recusado. Uma opção
  desconhecida e o `--` são recusados (`Unrecognised token`). O `-f`, o
  `--shard-count` e o `--shard-index` escolhem casos; o `-c`, o `-p` e o `-g`
  escolhem seções e geradores, e todos os casos rodam. São 23 opções com
  valor no `--help`;
- o caso gtest que não existe roda zero testes, escreve `PASSED 0 tests` e
  sai com 0. O do Catch2 sai com 2;
- o caso desativado rodado pelo filtro não roda, e o gtest escreve `YOU HAVE
  1 DISABLED TEST`. A contagem é a dos desativados que o filtro escolhe: com
  outro filtro, a linha não aparece. Com `GTEST_ALSO_RUN_DISABLED_TESTS=1`, o
  caso roda, e a linha some. Na linha do ctest, o CTest escreve `***Not Run
  (Disabled)` e sai com 0; o executável ausente dá `***Not Run` e sai com 8;
- com `GTEST_BRIEF=1` no ambiente, o gtest não escreve a linha do caso que
  passou, e `--gtest_brief=0` a traz de volta;
- o `json-v1` traz `ENVIRONMENT` e `ENVIRONMENT_MODIFICATION` como listas de
  texto (`K=V` e `K=op:valor`).

Desenho, em `test::entries`, novo:
- a descoberta só abre em casos o teste que roda todos os casos do binário.
  No gtest, nenhum argumento começa com `--gtest_filter`, `--gtest_flagfile`
  ou `--gtest_list_tests`, e nem o `ENVIRONMENT` nem o
  `ENVIRONMENT_MODIFICATION` têm `GTEST_FILTER`, `GTEST_TOTAL_SHARDS` ou
  `GTEST_SHARD_INDEX`, com o nome comparado sem caixa, como no Windows. No
  Catch2, todo argumento é uma opção que não escolhe casos, lido como o Clara
  lê: 20 das 23 com valor e 9 sem valor, nas formas curta e longa. Uma opção que a IDE não conhece
  deixa o teste fechado, e ele continua rodando inteiro pela linha dele;
- a listagem é pedida uma vez por binário e pasta: o `gtest_discover_tests`
  dá um teste por caso, e um binário pode listar outros casos noutra pasta;
- a descoberta deixa de fora o id de caso igual ao nome de um teste do ctest
  ou ao de outro caso;
- no `test.run`, o id igual ao nome de um teste do ctest é esse teste. Os
  outros acham os testes cujo nome, seguido de `::`, começa o id, e o caso que
  o binário lista. Um só roda. Sem nenhum, a recusa diz o motivo do teste de
  nome mais longo: ele já escolhe os casos, ou o binário não lista mais o
  caso; sem teste, o ctest não lista um teste para o id. Com dois, a recusa
  diz os dois;
- o caso gtest roda também com `--gtest_brief=0`. Quando o gtest escreve
  `YOU HAVE N DISABLED TEST(S)`, o caso conta como ignorado, com uma linha que
  diz por quê. Na linha do ctest, `***Not Run (Disabled)` é ignorado, e o
  outro `***Not Run` segue falha;
- C1-36, achado na prova: os nomes da árvore saem do `json-v1`, exatos, e a
  leitura do `Test #N:` aceita o número alinhado, na execução e no `-N`, que
  fica para quando o `json-v1` não lista nada;
- C1-37, achado no preparo da prova na UI: o `event.test.case` de um caso de
  dentro leva o id da árvore, `teste::caso`, que é o que o painel procura;
- protocolo 0.215.0, sem campo novo, com o contrato no
  [03 test](../arquitetura/03-dominios/test.md) antes da mudança.

Provas necessárias:
- testes do dono com cada forma medida do gtest e do Catch2, o id repetido, a
  resolução e o motivo, as linhas alinhadas e as do desativado;
- regressão de ponta a ponta com o CMake, o gtest e o Catch2 reais, com o
  `gtest_discover_tests`, o `catch_discover_tests` e testes escritos à mão:
  a árvore, cada recusa com o texto exato, o caso de um teste com `::` no
  nome, a linha com `::` pelo ctest, o desativado e a suíte inteira com mais
  de dez testes; e a mesma regressão reprovando no código de antes;
- mutantes que compilem e reprovem pela guarda.

Limites previstos:
- o caso rodado pela árvore não leva o ambiente nem os argumentos do teste do
  ctest: é o C1-34;
- o prefixo `/` das opções do Catch2 no Windows não foi medido;
- o nome do teste que termina em ` (Disabled)` só é lido certo pelo
  `json-v1`;
- com o `TEST_PREFIX`, o nome da linha do `catch_discover_tests` difere do
  nome do caso, e as duas não são ligadas na árvore.

### 21.1 C1-32: integração e provas — 2026-11-10

Integrado no desenho do §21, protocolo 0.215.0:
- `test/entries.rs`, novo: `runs_every_case`, com as tabelas medidas do gtest
  e do Catch2; `unique`, que deixa de fora o id repetido; `resolve`, que acha
  o teste e o caso de um id;
- `test/frameworks.rs`: o `json-v1` dá o `ENVIRONMENT` e o
  `ENVIRONMENT_MODIFICATION`; a descoberta pede a listagem uma vez por binário
  e pasta e só abre o teste que roda todos os casos; o `run_inner_case` usa o
  `resolve`, passa `--gtest_brief=0`, conta o desativado como ignorado e
  manda o `event.test.case` com o id da árvore (C1-37);
- `test/discover.rs`: num CMake, os nomes saem do `json-v1`, e o `-N` fica
  para quando ele não lista nada (C1-36);
- `test/parse.rs`: o `Test #N:` alinhado e o `***Not Run (Disabled)`.

Achados e corrigidos na prova, com a decisão do autor de incluir as
correções: o C1-36, quando a regressão nova, com 17 testes do ctest, mostrou
os testes do 1 ao 9 fora da árvore; e o C1-37, ao ler como o painel acha a
linha de um caso para preparar a prova na UI.

Provas:
- testes do dono: cada forma medida do gtest (filtro com e sem `=`, arquivo
  de flags, listagem, as três variáveis no `ENVIRONMENT` e no
  `ENVIRONMENT_MODIFICATION`, o nome sem caixa) e do Catch2 (o nome escapado
  do `catch_discover_tests`, a opção com valor junto e separado, as curtas
  juntas, o valor que falta, a flag com valor, cada opção que escolhe casos,
  a desconhecida, o `-` e o `--`); o id repetido e o de uma linha do ctest; a
  resolução, com o motivo do teste de nome mais longo e o que cada id lista;
  o mesmo binário que lista outro caso noutra pasta, perguntado uma vez por
  pasta; o evento do caso com o id da árvore; as linhas alinhadas do
  `-N` e da execução, o desativado e o executável ausente;
- em `tests/c_project/discovered_cases.rs`, com o CMake 4.4.5, o gtest 1.18 e
  o Catch2 3.16.1 reais, o `gtest_discover_tests`, o `catch_discover_tests` e
  dez testes escritos à mão: a árvore tem as 17 linhas e os 15 casos dos
  testes que rodam o binário inteiro, uma vez cada, sem o `dup::x::y` dos dois
  testes. O caso de `ns::unit` roda pelo binário, e o evento leva o id da
  linha, `ns::unit::Math.Adds`, e não `Math.Adds`, o nome da linha do ctest ao
  lado (C1-37). O `main` desse gtest liga o `brief`, como o `GTEST_BRIEF=1`
  faria. O desativado conta como ignorado,
  com a linha que diz por quê, e a linha `Text.Off` do ctest também. As
  linhas `gt_color::Math.Adds` e `Widget::draw works` rodam pelo ctest. Cada
  caso Catch2 com `::` roda só ele. Cada recusa sai com o texto exato. A
  suíte inteira conta 10 aprovados, 6 reprovados e 1 ignorado, os 17;
- a mesma regressão, posta no código do commit anterior, reprova: a árvore
  perde os testes do 1 ao 9 e tem `Math.Adds::Math.Adds`,
  `Math.Adds::Math.Fails` e os outros casos repetidos;
- na UI nativa isolada, num Xvfb a 1366x768, com o core 0.215.0 atrás de um
  `tee` do IPC e o projeto da regressão numa pasta com espaço e acento,
  `autoSave` desligado: confiar, configurar e compilar; rodar tudo dá 10
  aprovados, 6 reprovados e 1 ignorado; "Listar testes" mostra 32 linhas,
  as 17 do ctest e os 15 casos. O ▶ do caso `ns::unit::Math.Adds` manda o
  `test.run` com esse id, o `event.test.case` volta com ele, a linha do
  caso fica verde, e a `Math.Adds` do ctest, no topo, fica cinza; a saída
  tem o `[       OK ]`, com o `brief` ligado no `main`. O ▶ de
  `ns::unit::Text.DISABLED_Off` dá `ignorado: 1`, com a linha do motivo. A
  IDE sai com 0, sem aviso de QML no log. A prova achou o C1-38 e o C1-39;
- 53 mutantes que compilam, todos reprovados por um teste do dono ou pela
  regressão; os que não compilavam foram reescritos até compilar. Três
  guardas saíram por não terem prova: a do zero caso rodado, porque o gtest
  não imprime o aviso quando o caso roda (medido), e as duas que aceitavam
  outras formas do aviso, que um filtro de um caso não produz; ficou a
  linha exata, `YOU HAVE 1 DISABLED TEST`.

Limites:
- os do §21;
- o `-N` só é lido quando o `json-v1` não lista nada, e ali o desativado
  continua com ` (Disabled)` no nome;
- o comando mostrado do caso gtest não é o que roda (C1-38), e a árvore não
  distingue o caso de dentro nem o ignorado (C1-39).

## 22. C1-34: o caso como o ctest roda o teste — desenho de 2026-11-10

Decisões do autor de 2026-11-10: o caso de dentro roda com todos os
argumentos do `add_test`, e as flags da IDE vêm por último. No Catch2, o
`-r compact` só vai quando os argumentos não escolhem um reporter. O ambiente
é o que o CTest dá, também na listagem, que não leva os argumentos, como o
`gtest_discover_tests`. O C1-38 e o C1-40 entram junto. No C1-40, o teste
cujo desfecho o CTest decide de outro jeito não se abre em casos, e o
`TIMEOUT` do teste passa a valer para o caso.

Medido com o CMake 4.4.5, o CTest dele, o gtest 1.18 e o Catch2 3.16.1, com o
`env` como teste:
- o CTest põe `CTEST_INTERACTIVE_DEBUG_MODE=1` no ambiente de todo teste,
  com ou sem `-V` e `-j`, antes do `ENVIRONMENT`: um `ENVIRONMENT` com `=0`
  vence, e um `unset:` tira a variável;
- o `ENVIRONMENT` vale na ordem: com `R=1;R=2`, fica `2`. O texto é cortado
  no primeiro `=`, e `Z=a=b` dá `a=b`. Uma entrada sem `=`, como `S`, tira a
  variável, e `=x` põe uma de nome vazio. Depois dela, `S=string_append:x`
  dá `x`, e não o valor do processo com `x`, e `S=reset:` deixa a variável
  fora;
- o `ENVIRONMENT_MODIFICATION` vale na ordem, depois do `ENVIRONMENT`. O
  texto é cortado no primeiro `=` e, depois dele, no primeiro `:`: `N=set:a=b`
  dá `a=b`, e `O=set:a:b` dá `a:b`. O `set` põe o valor, também vazio. O
  `unset` tira a variável, e a operação seguinte parte do vazio. O `reset`
  volta ao valor de antes das modificações, o do `ENVIRONMENT` ou o do
  processo, e sem nenhum a variável fica fora; o valor do `unset` e do
  `reset` não conta;
- o `string_append` e o `string_prepend` juntam o valor ao atual, ou ao
  vazio. O `path_list_append` e o `path_list_prepend` põem o `:` só quando o
  atual não é vazio: `/a` mais o vazio dá `/a:`, e o vazio mais `/x` dá
  `/x`. O `cmake_list_append` e o `cmake_list_prepend` fazem o mesmo com o
  `;`. Uma operação de juntar sobre a variável ausente a põe, também vazia;
- a entrada sem `=`, sem `:` ou com outra operação faz o CTest não rodar o
  teste: `***Not Run` e `BAD_COMMAND`, com ``Error: Missing `=` after the
  variable name``, ``Error: Missing `:` after the operation`` ou `Error:
  Unrecognized environment manipulation argument: bogus`;
- o `json-v1` traz o `ENVIRONMENT_MODIFICATION` com o `;` escapado num
  elemento só (`V=set:a;b`), e o `WORKING_DIRECTORY`, sempre: sem ele
  declarado, é a pasta de build do `CMakeLists.txt`;
- o gtest usa a última de duas flags iguais: `--gtest_brief=1` antes de
  `--gtest_brief=0` dá a linha do caso, e o contrário não. Um argumento que
  não é flag dele fica para o `main` depois do `InitGoogleTest`. Com
  `--gtest_repeat=2`, o caso escreve duas linhas `[       OK ]`. Com
  `--gtest_also_run_disabled_tests`, o caso desativado roda e passa, sem o
  `YOU HAVE 1 DISABLED TEST`;
- no Catch2, um segundo reporter na saída padrão aborta o binário: `-r junit`
  ou `-r compact` com mais um `-r compact` dão `cannot use default output for
  multiple reporters` e o código 134. O `-r junit::out=j.xml` com o
  `-r compact` roda os dois. Com `-o arquivo`, o `-r compact` escreve no
  arquivo, e o código de saída segue o desfecho;
- o `json-v1` traz o `WILL_FAIL`, o `PASS_REGULAR_EXPRESSION`, o
  `FAIL_REGULAR_EXPRESSION`, o `SKIP_REGULAR_EXPRESSION`, o
  `SKIP_RETURN_CODE`, o `DISABLED`, o `FIXTURES_REQUIRED`, o
  `REQUIRED_FILES` e o `TIMEOUT_AFTER_MATCH` só quando ligados; o
  `WILL_FAIL FALSE` e o `DISABLED OFF` não aparecem;
- o `TIMEOUT` vem em segundos, como número. Com `1`, um `sleep 3` dá
  `***Timeout`; com `0` ou `-1`, não há limite. Com `include(CTest)` e
  `DART_TESTING_TIMEOUT` 1, o `DartConfiguration.tcl` diz `TimeOut: 1`, o
  CTest para em 1 s o teste sem `TIMEOUT`, e o `json-v1` não traz o prazo.
  Sem um arquivo de recursos, o `RESOURCE_GROUPS` não põe variável nenhuma;
- o `TEST_LAUNCHER` vem no começo do `command`: `/usr/bin/env LAUNCHED=1
  <exe>`. A listagem pergunta ao lançador, que não responde, e o teste fica
  fechado;
- o teste que cai sai como `***Exception: SegFault`, `***Exception:
  Numerical`, `***Exception: Illegal`, `Subprocess aborted***Exception:`,
  `Subprocess terminated***Exception:` ou `Subprocess killed***Exception:`, e o
  que passa do prazo como `***Timeout` (C1-42).

Desenho, em `test::launch`, novo:
- o ambiente do caso e da listagem é o do processo, mais o do Visual Studio
  que o `ctest` recebe, o `CTEST_INTERACTIVE_DEBUG_MODE=1`, o `ENVIRONMENT` e
  o `ENVIRONMENT_MODIFICATION`, com as regras medidas. No Windows, o nome é
  comparado sem caixa e a lista de caminhos usa o `;`;
- o caso gtest roda com `<exe> <argumentos> --gtest_filter=<caso>
  --gtest_color=no --gtest_print_time=1 --gtest_brief=0`. O caso Catch2 roda
  com `<exe> <argumentos> <filtro>`, mais o `-r compact` quando nenhum
  argumento é `-r` ou `--reporter`, também nas formas juntas que o Clara lê;
- o comando mostrado sai dos mesmos argumentos, cada um com aspas quando o
  shell do sistema pede: no Unix, as simples do POSIX; no Windows, o `& '…'`
  do PowerShell (C1-38);
- com `TIMEOUT` maior que zero, o caso é parado no prazo, conta como
  reprovado e ganha uma linha que diz por quê. Um prazo grande demais para
  uma duração fica sem limite;
- C1-40: em `test::entries`, o teste com uma das propriedades que mudam o
  desfecho não se abre em casos, e nem pede a listagem. O
  `ENVIRONMENT_MODIFICATION` que o CTest recusa também deixa o teste fechado.
  No `test.run`, o id de um caso desse teste é recusado com o motivo;
- C1-41, com a decisão do autor de 2026-11-10 de corrigir junto: o comando
  mostrado do `ctest` da linha e do `ctest -N` da descoberta sai dos mesmos
  argumentos, com o `--test-dir` e as aspas do C1-38;
- C1-42, achado na regressão e com a decisão do autor de 2026-11-10 de
  corrigir junto: as linhas `***Exception` e `***Timeout` do CTest contam
  como reprovadas;
- protocolo 0.216.0, sem campo novo, com o contrato no
  [03 test](../arquitetura/03-dominios/test.md) antes da mudança.

Provas necessárias:
- testes do dono com cada operação, a ordem, o vazio, o ausente, o `reset`
  depois do `unset`, a entrada que o CTest recusa e o nome sem caixa; os
  argumentos e as aspas do comando mostrado; cada propriedade que fecha o
  teste e a que não fecha; o prazo;
- regressão de ponta a ponta com o CMake, o gtest e o Catch2 reais: o caso
  vê o ambiente e os argumentos que a linha do ctest vê, comparados pelo
  próprio binário; o reporter do `add_test`; o desfecho de cada propriedade
  pelo ctest e o teste fechado; o caso parado no `TIMEOUT`; e a mesma
  regressão reprovando no código de antes;
- mutantes que compilem e reprovem pela guarda.

Limites previstos:
- o comando mostrado não leva o ambiente do teste;
- a listagem de um binário que não termina prende a descoberta (C1-43);
- o `TIMEOUT_SIGNAL_NAME` não vale: no prazo, o caso é parado como um job
  cancelado;
- o prazo padrão do CTest, o `TimeOut` do `DartConfiguration.tcl`, que o
  `json-v1` não traz, não vale para o caso;
- o Windows não foi medido: a lista de caminhos com `;`, o nome sem caixa,
  a entrada sem `=` e as aspas do PowerShell seguem a documentação.

### 22.1 C1-34: integração e provas — 2026-11-10

Integrado no desenho do §22, protocolo 0.216.0:
- `test/launch.rs`, novo: o ambiente do caso e da listagem, pelas regras
  medidas, e o comando do caso, com os argumentos do `add_test`, as flags da
  IDE por último e o comando mostrado com as aspas do shell (C1-38);
- `test/entries.rs`: `closed`, com as nove propriedades que mudam o desfecho
  e o `ENVIRONMENT_MODIFICATION` recusado, antes da listagem e no `resolve`;
  o reporter do Catch2 nas formas que o Clara lê (C1-40);
- `test/frameworks.rs`: o `json-v1` dá as propriedades ligadas e o `TIMEOUT`;
  a listagem roda com o ambiente e é pedida uma vez por binário, pasta e
  ambiente; o caso parado no prazo conta só como reprovado;
- `test/mod.rs`: o `stream_command` com prazo, que diz se parou; o comando
  mostrado do `ctest` com o `--test-dir` e as aspas, na linha e no `-N`
  (C1-41);
- `test/parse.rs`: `***Exception` e `***Timeout` contam como reprovados
  (C1-42);
- `platform`: `shell_argument`, as aspas de um argumento no Unix e no
  PowerShell.

Provas:
- testes do dono: cada operação do `ENVIRONMENT_MODIFICATION`, a ordem, o
  vazio, o ausente, o `reset` depois do `unset`, a variável que o
  `ENVIRONMENT` tira, o `CTEST_INTERACTIVE_DEBUG_MODE` antes do
  `ENVIRONMENT`, o nome comparado como o sistema compara e as três entradas
  que o CTest recusa, com o texto exato e o comando intocado; os argumentos e
  as aspas do comando mostrado, no Unix e no Windows; cada propriedade que
  fecha o teste e o valor desligado que não fecha, lidos do `json-v1`; o
  reporter do Catch2 em cada forma; a listagem pedida uma vez por ambiente; o
  caso parado no `TIMEOUT` e a linha que diz por quê; as linhas de queda e de
  prazo do CTest;
- em `tests/c_project/case_environment.rs`, com o CMake 4.4.5, o gtest 1.18 e
  o Catch2 3.16.1 reais, num projeto com espaço e acento: o binário escreve o
  ambiente e os argumentos que recebe, e o caso de `env_unit` vê os mesmos
  que a linha do ctest vê, com o `ENVIRONMENT` que tira o `HOME`, o
  `path_list_prepend` no `PATH`, o `set:` vazio e uma pasta com espaço nos
  argumentos; o comando mostrado é o que roda, com as aspas. Os testes com
  `WILL_FAIL`, `DISABLED`, fixtures e o `ENVIRONMENT_MODIFICATION` recusado
  não se abrem em casos, o id de um caso deles sai recusado com o texto
  exato, e a linha do ctest dá o desfecho do CTest: o `wf` passa, o `off`
  fica ignorado, o `bad` reprova. O caso de um teste com `TIMEOUT 1` é parado
  e reprova com a linha do motivo, e a linha do ctest dele também reprova. O
  caso Catch2 de um teste com `-r console` roda sem o `-r compact` e passa;
- a mesma regressão, posta no código do commit anterior, reprova em cada
  parte: o caso sem o ambiente e sem os argumentos, o comando sem aspas, os
  casos dos testes fechados, o caso que espera 30 s e passa, e a linha do
  ctest que passa do prazo sem contar;
- na UI nativa isolada, num Xvfb a 1366x768, com o core 0.216.0 atrás de um
  `tee` do IPC, o projeto da regressão mais um teste que cai, numa pasta com
  espaço e acento, e o `autoSave` desligado, confirmado pelo IPC: compilar e
  rodar tudo dá 3 aprovados, 6 reprovados e 1 ignorado, com o `crash`
  (`***Exception: SegFault`), o `slow` e o `slow_end` (`***Timeout`) em
  vermelho, e o comando mostrado `ctest --test-dir '<pasta>'
  --output-on-failure` (C1-41, C1-42). "Listar testes" mostra 17 linhas, os
  10 testes do ctest e 7 casos, sem caso do `wf`, do `off`, do `fx_user` e do
  `bad`. O ▶ de `env_unit::Env.Reads` roda o binário com a pasta dos dados, o
  `--gtest_brief=1` e as flags da IDE por último, o comando mostrado é o que
  roda, com as aspas, e as seis linhas que o binário escreve são iguais às da
  linha `env_unit` na suíte; a linha do caso fica verde. O ▶ de
  `slow_end::Slow.Passes` passa no gtest, é parado em 1 s, ganha a linha do
  motivo e fica vermelho, com `passou: 0 falhou: 1`. O ▶ de `c2_rep::reads`
  roda `c2 -r console reads` e passa. O ▶ da linha `slow_end` dá
  `***Timeout` e fica vermelho. A IDE sai com 0, e o log dela fica vazio, sem
  aviso de QML. A prova achou o C1-44, a árvore que volta ao topo ao rodar, e
  o INF-07 (62 §3.4), o `settings.json` de esquema desconhecido que a IDE
  sobrescreve;
- nenhum teste novo fica de fora do Windows: a catraca dos adiados parou a
  segunda rodada do gate com três `cfg(unix)` e um adiado da W6 a mais, e o
  esperado de cada sistema passou para dentro do teste, com o JSON do ctest
  falso montado pelo `serde_json`, que escapa a `\` dos caminhos;
- 72 mutantes que compilam: 71 reprovados por um teste do dono ou pela
  regressão; os quatro que não compilavam foram reescritos até compilar. Um
  sobreviveu e é equivalente: tirar a recusa do `apply_environment` no
  `run_inner_case`, porque o `resolve` já recusa o teste pelo `closed` antes
  de chegar ali; a guarda fica, porque o tipo pede o `Err`, e o mutante que
  tira o `closed` do `resolve` reprova.

Limites:
- os do §22;
- o Windows segue sem medida: as aspas do PowerShell, o nome sem caixa e a
  lista de caminhos com `;` seguem a documentação. Os testes do dono rodam
  nos dois sistemas, com o esperado de cada um, e o do prazo e do ambiente
  usa o executável falso da W2b, mas só rodaram no Linux.

## 23. C1-39 e C1-44: a árvore dos casos — desenho de 2026-11-10

Decisões do autor de 2026-11-10: no C1-39, a linha mostra o id inteiro,
`ns::unit::Math.Adds`, o texto que o "Rodar só este" usa, sem campo novo no
protocolo, e o ponto do ignorado é âmbar, o `warningSoft`; no C1-44, achado
na prova na UI do C1-34, corrigir junto, no mesmo painel.

Medido no harness novo `tst_tests_tree`, com o `JobsController` e o
`TestsPanel` reais, 17 linhas e um painel de 420 px: com saída, a árvore tem
186 px; o `prepareTestRun` zera a saída, o `hasOutput` cai, e a árvore passa
a 384 px, onde as 17 linhas cabem. O `fixup` do `Flickable` leva então o
`contentY` a 0, e a árvore fica no topo quando a saída volta. Na UI do C1-34,
foi o que se viu nas duas vezes medidas.

Desenho:
- C1-39, no core: o `name` do `TestCaseInfo` do caso de dentro é o id, como
  o contrato do `TestCaseInfo` já diz para todo runner que não é o pytest;
  protocolo 0.217.0, sem campo novo, com o contrato no
  [03 test](../arquitetura/03-dominios/test.md) antes da mudança;
- C1-39, na UI: `TestsPanel.statusColor` pinta `ignored` com o
  `Theme.warningSoft`; o que não rodou segue com o `textMuted`;
- C1-44, na UI: a área da saída fica enquanto houver árvore, depois que a
  saída apareceu com ela na tela. A árvore que chega sem saída usa o painel
  inteiro, como hoje, e sem árvore a área volta a seguir as linhas. O
  `JobsController` está no limite de 400 linhas e não muda.

Provas necessárias:
- testes do dono no core: o `name` do caso de dentro é o id, na descoberta
  e na regressão com o gtest e o Catch2 reais;
- o harness `tst_tests_tree`: a árvore não muda de tamanho no run, na volta
  da saída, ao relistar e no run seguinte; sem árvore, o run usa o painel
  inteiro; depois do `clear`, a listagem usa o painel inteiro; a cor de cada
  status. Ele reprova no código de antes;
- mutantes que compilem e reprovem pela guarda;
- na UI nativa: a linha do caso com o id inteiro, o ponto âmbar do ignorado,
  e a árvore que fica onde estava ao rodar um caso do fim da lista.

### 23.1 C1-39 e C1-44: integração e provas — 2026-11-10

Integrado no desenho do §23, protocolo 0.217.0:
- `test/frameworks.rs`: o `name` do caso de dentro é o id;
- `ui/qml/panels/bottom/TestsPanel.qml`: `statusColor` pinta `ignored` com o
  `warningSoft`; a área da saída fica enquanto houver árvore, depois que a
  saída apareceu com ela (`keepsOutput`), e a árvore que chega sem saída
  decide de novo pela saída que há;
- `scripts/qml-harness/tst_tests_tree.qml`, novo.

Provas:
- testes do dono no core: todo caso da descoberta pelo ctest falso tem o
  `name` igual ao id; na regressão `discovered_cases.rs`, com o CMake 4.4.5,
  o gtest 1.18 e o Catch2 3.16.1 reais, toda linha da árvore também;
- o harness `tst_tests_tree`, com o `JobsController` e o `TestsPanel` reais:
  a árvore fica com 186 px no run, na volta da saída, ao relistar e no run
  seguinte; sem árvore, o run usa os 384 px; depois do `clear`, a listagem
  também; a cor de cada status. Com o `TestsPanel` do commit anterior, ele
  reprova no run, no run depois de relistar e na cor do ignorado;
- 9 mutantes que reprovam: um no core, pelo teste do dono e pela regressão,
  e oito no painel, pelo harness: sem a trava, a trava sem a árvore, a trava
  que nunca liga, a que nunca solta, a árvore nova que sempre desliga ou
  sempre liga, e o ignorado apagado ou verde;
- na UI nativa isolada, com o core 0.217.0 e o `autoSave` desligado,
  confirmados pelo IPC, e a fixture do §22.1 mais um gtest com `GTEST_SKIP`:
  "Listar testes" dá 20 linhas, todas com o `name` igual ao id, e a árvore
  mostra `slow::Slow.Passes` e `slow_end::Slow.Passes`, que antes apareciam
  iguais. Rolada até os casos e parada (duas fotos iguais), o ▶ de
  `skip::Skip.Pulado` dá `ignorado: 1`; a área de texto da árvore fica igual
  pixel a pixel à de antes do ▶, e o ponto da linha é `#e6b84a`, o
  `warningSoft`, com o da linha ao lado no cinza do que não rodou. A IDE sai
  com 0, e o log dela fica vazio, sem aviso de QML.

Limites:
- a cor do ignorado na lista de casos soltos, sem árvore, é a mesma, pela
  mesma função;
- o harness prova o tamanho da árvore, que é a causa; a posição da rolagem,
  que depende do `fixup` animado do `Flickable`, está provada só na UI.

## 24. C1-43: a listagem dos casos com prazo — desenho de 2026-11-10

Hoje `test::frameworks::list_cases` roda o binário até três vezes
(`--gtest_list_tests`, `--libidentify` e `--list-tests`) com
`Command::output()`, sem prazo e sem o `cancel` do job. O `inner_cases` só
olha o `cancel` entre um teste e outro.

Referência medida no CMake 4.4.5: o `GoogleTest.cmake` dá 5 s à listagem
(`DISCOVERY_TIMEOUT`, padrão 5), no `execute_process` que roda o
`--gtest_list_tests`, e o build falha quando ela passa disso. O
`Catch.cmake` do Catch2 3.16.1 não tem prazo.

Desenho, pela decisão do autor:
- cada chamada da listagem roda pelo
  `process::stream_command_lines_with_timeout_cancelable`, com o `cancel` do
  job e o prazo de 5 s do `DISCOVERY_TIMEOUT` padrão; o processo nasce num
  grupo, e o fim do prazo ou o cancel matam a árvore dele;
- o binário que passa do prazo fica só com a linha do ctest, e a saída da
  listagem diz qual e por quê, em vez de falhar a descoberta inteira como o
  CMake, porque a linha do ctest ainda roda;
- a saída guardada de uma chamada tem um teto, e passar dele conta como não
  ter respondido, com o motivo na saída, porque em 5 s um programa pode
  escrever gigabytes;
- o `run_inner_case`, que lista para achar o caso, usa a mesma listagem e diz
  o mesmo motivo.

Depende do INF-09 (62 §3.4): pelo `process`, uma linha que não é UTF-8
cortaria a listagem, e o `from_utf8_lossy` de hoje a mantém.

Decisões do autor de 2026-11-10: o INF-09 vem antes, num commit próprio; o
cancel no meio da listagem dá o erro "listagem cancelada", pelo caminho de
falha que já existe, em vez da árvore pela metade contada como completa; o
prazo de 5 s e o teto de 64 MiB valem para cada chamada, e um binário que
trava pode levar até 15 s.

Provas exigidas: o binário que não termina, o que ignora o `SIGTERM`, o que
deixa um filho com o pipe aberto, o cancel no meio de cada chamada e a
saída sem fim, cada um com o tempo medido e sem processo sobrando; a
regressão com o gtest e o Catch2 reais segue igual; mutantes no prazo, no
cancel, no teto e no motivo.

### 24.1 C1-43: integração e provas — 2026-11-11

Integrado no desenho do §24, protocolo 0.218.0:
- `test/listing.rs`, novo, com a listagem que saiu do `test/frameworks.rs`:
  as três chamadas, cada uma pelo
  `process::stream_command_lines_with_timeout_cancelable`, com o `cancel` do
  job e os `LIMITS` (5 s e 64 MiB); o motivo do binário que passa do prazo ou
  do teto vai para a saída; a chamada que vem depois do cancel não roda;
- `test/frameworks.rs`: o `discover_inner_cases` recebe a saída do job para o
  motivo, e o `run_inner_case` lista do mesmo jeito: o motivo do prazo vira o
  erro, em vez de "não lista mais o caso", e o cancel vira "listagem
  cancelada";
- `test/discover.rs` e `TestError::ListingCancelled`: o cancel durante o
  comando do runner ou durante a listagem dos casos dá o erro "listagem
  cancelada", e o `event.test.discovered` sai com `success: false`.

Provas:
- testes do dono no `listing.rs`, com binários falsos e limites curtos: o
  servidor que ignora as flags e o `SIGTERM` e tem um filho para em cada
  chamada no prazo, com dois motivos e sem processo sobrando; a saída no teto
  exato conta, um byte a mais não conta, a saída sem fim para no prazo, e o
  `stderr` não entra na lista nem no teto; o cancel no meio de cada uma das
  três chamadas para a chamada e as seguintes; o binário que responde e
  deixa um filho com o pipe aberto conta, sem esperar o filho; os `LIMITS`
  são 5 s e 64 MiB;
- o cancel no meio do comando do runner, com um Python falso, dá
  `ListingCancelled`, e não "saiu com signal";
- a regressão `listing_deadline`, com o CMake 4.4.5, o `ctest` e o GCC 16
  reais: um servidor posto como teste, que grava o pid a cada chamada, e um
  binário que lista como o gtest. O `test.discover` termina em
  10 s a 30 s com `server`, `fake` e `fake::Math.Adds` e os dois motivos do
  servidor, sem os processos dele; o `test.run` de `server::Math.Adds` é
  recusado pelo prazo; o cancel no meio do run e no meio da listagem dá
  "listagem cancelada" em menos de 3 s, o job termina `cancelled`, e o
  servidor não sobra. Com a listagem de antes, ela reprova;
- 14 mutantes que reprovam: a listagem de antes, pelo
  `Command::output()`; sem o prazo; sem o `cancel` na chamada; sem olhar o
  `cancel` antes dela; sem o teto, com `>=` no teto, e o `stderr` contado
  nele; o teto e o prazo sem o motivo; a descoberta sem o motivo; o run sem
  o motivo e sem o erro do cancel; a descoberta sem o cancel do runner e sem
  o da listagem. Sem o prazo, o teste da saída sem fim não termina, e o
  mutante foi medido pelo teste do binário que não responde;
- na UI nativa isolada, com o core 0.218.0 e o `autoSave` desligado,
  confirmados pelo IPC, e a fixture do §23.1 mais um servidor posto como
  teste: "Listar testes" termina em 11,5 s com 21 linhas, a do servidor sem
  casos, e os dois processos do servidor terminam. Clicado o × do job na
  barra de status no meio da listagem, o `event.test.discovered` chega em
  0,75 s com `success: false` e "listagem cancelada", o job termina
  `cancelled`, a aba Testes diz "não listou: listagem cancelada", e o
  servidor não sobra. A IDE sai com 0, e o log dela fica sem aviso de QML.

Limites:
- o binário que trava em todas as chamadas leva 10 s, e o Catch2 que
  responde ao `--libidentify` e trava na lista leva 15 s; o cancel para
  antes;
- o prazo é fixo: o `DISCOVERY_TIMEOUT` que o projeto passa ao
  `gtest_discover_tests` não é lido, porque é argumento da função, que o usa
  no build, e não propriedade do teste, e o `json-v1` não o tem;
- o filho que o binário deixa vivo depois de responder segue vivo, como em
  todo job que termina sozinho;
- na UI, o motivo do prazo sai só no log do job, que nenhuma vista mostra
  inteiro (C1-45).

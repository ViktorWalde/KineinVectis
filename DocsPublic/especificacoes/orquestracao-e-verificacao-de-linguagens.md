# Orquestração e verificação das linguagens

> **Classe: DECISÃO / ALVO, 2026-11-02.** Define a evolução da base pública
> 0.4: consolidar C e C++, introduzir o provedor Rust e aprofundar a integração
> Rust/C/C++, depois introduzir o provedor Python. O núcleo comum e os provedores
> adicionais descritos aqui são alvo; a implementação comprovada é a primeira
> fatia host do [verificador de correções LSP](verificador-de-correcoes-lsp.md).
> Fronteira da entrega no [62](../roadmaps/62-base-publica-0.4.md).

## 1. Finalidade e sequência

A base deve permitir desenvolver projetos pequenos e médios com uso contínuo
de editor, LSP, build, testes, depuração, Git e terminal. A Kinein Vectis deve
ser um caso de aceitação dessa jornada: core Rust, ponte C++, frontend QML e
comunicação IPC precisam permanecer coerentes ao editar e reconstruir o projeto.
Quantidade de arquivos ou linhas, isoladamente, não comprova essa capacidade.

A ordem de consolidação é vinculante:

1. C como linguagem com corpus e aceites próprios; C++ e seu ecossistema de host.
2. Rust como terceira linguagem do verificador, integração profunda com C e C++,
   e jornada de desenvolvimento da própria IDE.
3. Python, reutilizando o núcleo comum demonstrado pelos provedores anteriores.
4. Perfis especializados de plataforma e domínio, conforme dependências e provas.

Os LSPs e recursos de Rust e Python já existentes continuam sendo mantidos
durante essa sequência. A ordem acima rege o aprofundamento e a introdução dos
provedores de verificação; não exige remover capacidades atuais.
Bare metal, Linux embarcado, ROS 2, computação científica e IoT compõem a
evolução especializada. A trilha do [52](../roadmaps/52-arquitetura-executavel-da-0.4.md)
permanece aberta, com seus compromissos e provas independentes.

## 2. Arquitetura comum e donos

O motor coordena evidências produzidas pelas ferramentas reais do projeto.
As decisões comuns ficam compartilhadas; cada linguagem define como preparar,
executar e interpretar sua verificação. Especializações acrescentam condições
de aplicação e regras versionadas.

```mermaid
flowchart TB
    consumer["Consumidores: editor, configuração, build e qualidade"]
    common["Núcleo comum: identidade, execução, evidência e invalidação"]
    providers["Provedores: C, C++, Rust e Python"]
    profiles["Perfis combináveis: plataforma, domínio, framework e SDK"]
    tools["Ferramentas e donos existentes do projeto"]
    consumer --> common
    common --> providers
    providers --> tools
    profiles --> providers
```

| Responsabilidade | Dono existente a estender |
| --- | --- |
| Buffers, versões, ações e respostas dos servidores | `crates/kinein-core/src/lsp/` e `handlers/lsp/` |
| Conteúdo antes/depois, prévia e confirmação atômica | `crates/kinein-core/src/lsp/transaction.rs` |
| Contexto efetivo C/C++, CDB e identidade observada | `crates/kinein-core/src/index/context/` e `cdb/` |
| Primeira comparação com compilador | `crates/kinein-core/src/build/edit_verification/` |
| Build, Cargo e ambiente Python | `crates/kinein-core/src/build/`, `cargo.rs` e `python/` |
| Processos, limites e encerramento | `crates/kinein-core/src/process.rs`, Jobs e donos de execução |
| Contratos e apresentação | `crates/kinein-protocol/`, ponte C++ e consumidores QML atuais |

Essa divisão é conceitual, sem novos módulos ou API pública definidos por esta
especificação. Extrair mecanismos comuns ao adicionar um segundo consumidor
real, preservando os donos. Parsing, resolução de tipos e regras da linguagem
continuam nas ferramentas especializadas. O motor semântico profundo do
[56](../roadmaps/56-motor-semantico-profundo-cpp-rust.md) conserva seu escopo.

## 3. Identidade e contrato conceitual da verificação

Cada operação deve identificar o conjunto de entradas que efetivamente observou.

| Entrada | Identidade exigida |
| --- | --- |
| Documento e edição | Workspace, caminhos resolvidos, versão, conteúdo antes/depois, ranges e geração da consulta |
| Ferramentas | Executável efetivo, versão identificada, argumentos, diretório e ambiente relevante |
| Build | Sistema, configuração, alvo, toolchain, dependências, manifests e arquivos gerados pertinentes |
| Linguagem | Padrão ou edição, extensões, opções e unidade/pacote selecionado |
| Regras | Identificador, versão, origem, pré-condições e evidência de aplicabilidade |
| Execução | Escopo solicitado, orçamento, cancelamento e completude da saída |

Buffers modificados, headers, manifests, lockfiles, flags, bibliotecas ou alvo
alterados invalidam as evidências que dependem deles. Resposta de outro workspace,
consulta ou configuração não pode substituir a atual. O provedor deve declarar
as dependências observadas e as que não conseguiu capturar.

O resultado conserva origem, etapa executada, duração, diagnóstico, motivo de
incompletude e identidade das entradas. O contrato atual usa `newError`,
`noNewError` e `notVerified`; ele permanece o contrato da primeira fatia.
Uma evolução para várias etapas exige contrato IPC próprio antes do produtor.
Este documento não acrescenta campos ao protocolo nem altera sua versão.

O fluxo de cada operação deve capturar o snapshot, resolver contexto e escopo,
selecionar provedores/regras aplicáveis, executar em área controlada, comparar
evidências equivalentes e revalidar identidade antes de publicar e aplicar.
Cada passo conserva cancelamento e prazo. Mudança de entrada encerra ou invalida
o trabalho dependente; falha parcial preserva a informação da etapa afetada,
sem transformar o sucesso de outro provedor em verificação completa.

## 4. Evidências e alcance

| Etapa | Evidência que pode produzir | Prova separada necessária |
| --- | --- | --- |
| Estrutura/sintaxe | Edição bem formada e análise sintática da entrada observada | Resolução de tipos e dependências |
| Análise estática/compilação de verificação | Diagnósticos com o contexto e as ferramentas selecionadas | Geração de código, ligação e execução |
| Build e ligação | Artefatos e símbolos resolvidos na configuração exercitada | Comportamento em execução |
| Testes e depuração | Resultado dos casos executados no ambiente identificado | Propriedades e ambientes fora desses casos |
| Regra de domínio | Violação ou atendimento de uma condição versionada | Condições não cobertas pela regra |

Nenhuma etapa eleva automaticamente o resultado para uma garantia geral de
equivalência ou correção. Por exemplo, `cargo check` omite a etapa final de
geração de código e pode deixar de emitir erros dessa etapa, conforme a
[documentação do Cargo](https://doc.rust-lang.org/cargo/commands/cargo-check.html).
Resultados devem explicar o que foi executado, inclusive quando apenas parte
do projeto foi verificada. Ausência de ferramenta ou evidência fica explícita.

## 5. C como frente própria

O corpus C deve conter programas C puros e bibliotecas consumidas por outras
linguagens. O driver e o padrão efetivos devem ser preservados; sucesso ao
compilar um exemplo como C++ não aceita o caso C.

- **Linguagem:** padrões usados pelo projeto, extensões do compilador, tipos,
  conversões, declarações, linkage e diferenças entre configurações.
- **Pré-processamento:** macros, includes condicionais, headers públicos e
  privados, arquivos gerados, includes relativos e unidades que compartilham
  headers com flags diferentes.
- **Ecossistema de build:** CMake e Make nos donos existentes, CDB real,
  bibliotecas estáticas/dinâmicas, caminhos de busca e símbolos na ligação.
- **Memória e análise:** ponteiros, ownership convencionado pela API, duração
  dos objetos, alinhamento, aliasing e comportamento indefinido. Reusar
  analisadores e sanitizers quando aplicáveis, com execução e alcance declarados.
- **Plataforma:** ABI, arquitetura, sysroot, libc e distinção entre ambiente
  hospedado e freestanding. Este último afeta bibliotecas e inicialização;
  a prova de host não comprova um alvo sem sistema operacional.

A distinção hosted/freestanding e a seleção de dialetos estão descritas na
[documentação do GCC](https://gcc.gnu.org/onlinedocs/gcc/Standards.html).
Aceitar a frente por projetos C reais, incluindo biblioteca e executável em
múltiplas unidades, erro de header compartilhado, símbolo ausente, edição
válida e contexto divergente. Testes negativos devem demonstrar a guarda.

## 6. Consolidação C++

Preservar padrão, biblioteca padrão, ABI, macros, include paths, opções de
exceções/RTTI e configuração efetiva. O corpus deve cobrir templates, overloads,
headers compartilhados, dependências e falhas de ligação entre unidades.
Módulos, PCH, wrappers e response files só recebem suporte declarado com
interpretação, isolamento e provas próprios; não eliminar essas opções para
obter um resultado positivo.

A ampliação da comparação para headers e edições em vários arquivos deve
identificar as unidades afetadas, usar snapshots coerentes e relatar cobertura
parcial. Bibliotecas e dependências precisam ser tratadas no contexto do projeto,
incluindo a configuração selecionada e artefatos gerados.

## 7. Rust antes de Python

O provedor Rust deve resolver workspace/pacote, toolchain, edição, target,
profile, features, configuração `cfg`, manifests e lockfile efetivos.
Rust-analyzer, Cargo, compilador e ferramentas de teste devem observar o mesmo
contexto. Escolher um pacote ou conjunto de features diferente não aceita a
verificação solicitada.

Usar os diagnósticos estruturados existentes quando disponíveis, preservando
código, severidade, spans, expansões e relações entre diagnósticos. Conversões
entre bytes, Unicode e posições LSP precisam de testes próprios; o formato
[JSON do rustc](https://doc.rust-lang.org/rustc/json.html) não deve ser tratado
como texto já pronto para os ranges UTF-16 do editor.

Build scripts, macros procedurais e geradores podem participar da análise e
devem seguir a confiança e o isolamento do projeto. Preparar a verificação em
área controlada, preservar originais e impedir alteração implícita de manifests,
lockfiles ou dependências. Separar `check`, build, testes, Clippy e depuração
como evidências distintas, com orçamento e cancelamento próprios.

## 8. Integração profunda Rust/C/C++

O grafo do projeto deve representar a origem dos headers/bindings, bibliotecas
nativas, unidades produtoras e consumidores Rust. Alterar uma interface deve
invalidar e verificar os dois lados pertinentes.

| Fronteira | Casos obrigatórios de integração |
| --- | --- |
| Rust/C | Convenção de chamada, símbolos, tamanhos/layout, alinhamento, nulidade, buffers e ownership |
| Rust/C++ | Mecanismo de interoperabilidade declarado pelo projeto, headers/bindings gerados, runtime C++ e ligação |
| Execução | Callbacks, tempo de vida, threads, destruição, propagação de erros, panic e exceções |
| Build | Relação Cargo/CMake/Make, geradores, ordem/dependências e configurações compatíveis |

A compatibilidade precisa de fixtures positivas e negativas, build/ligação e
testes de execução nas fronteiras exercitadas. Rust não consegue conferir por
si só que toda declaração estrangeira corresponde à biblioteca real; esse
limite da FFI é descrito no [Rustonomicon](https://doc.rust-lang.org/nomicon/ffi.html).
O provedor deve diferenciar verificações estáticas de contratos que dependem
de teste. Nenhum mecanismo de bindings específico é imposto por este desenho.

Na própria IDE, Rust e C++ comunicam-se por IPC entre processos. Quando exercitada,
essa jornada comprova contrato, mensagens e sincronização das duas implementações; a prova FFI
precisa de um projeto próprio. Manter ambas na matriz, sem confundir seus aceites.
QML também participa da jornada de build, navegação e diagnóstico da IDE.

## 9. Python e projetos mistos

Depois da consolidação Rust/C/C++, o provedor Python deve preservar interpretador,
versão, ambiente virtual, dependências, caminhos de importação e configuração
dos analisadores do projeto. Reusar os donos de Python, basedpyright e Ruff.
Compilar bytecode é uma evidência limitada, conforme o escopo de
[`py_compile`](https://docs.python.org/3/library/py_compile.html); análise de tipos,
resolução de imports e testes têm aceites separados.

A descoberta não deve importar módulos do projeto para inferir sua estrutura.
Executar testes ou ferramentas que carreguem código deve respeitar confiança,
isolamento e gesto correspondente. CPython, MicroPython e CircuitPython têm
contextos distintos. Extensões Python em C/C++/Rust acrescentam casos de build,
ABI, ambiente e execução ao corpus de projetos mistos.

## 10. Especializações combináveis

Um perfil de domínio deve declarar linguagem, plataforma, framework/SDK,
versões, alvo, dependências, regras e ferramentas exigidas. Perfis não são
categorias mutuamente exclusivas: um projeto pode combinar C, bare metal e IoT,
ou Python, C++ e computação científica.

A seleção usa fatos observados do projeto; extensão do arquivo ou presença de
uma pasta não bastam para habilitar uma regra de SDK. Regras precisam de origem,
versão, pré-condição e corpus. Incompatibilidade, conflito ou contexto insuficiente
devem ser explicados. SDKs, ROS 2 e novas cargas científicas recebem provas reais
próprias, depois da base das linguagens; testes de host não demonstram suporte
nesses ambientes. Funcionalidades especializadas já oferecidas conservam regressões.

## 11. Orquestração e eficiência

Sincronizar conteúdo realmente alterado; vincular consultas e respostas a sua
geração; cancelar trabalho substituído; manter diagnóstico, navegação e correções
no contexto efetivo. O fechamento exige testar servidor principal e companheiros,
reinício/crash, ferramenta ausente, troca de configuração e workspace, edição
rápida, Unicode e retorno fora de ordem.

Exercitar `initialize`/prontidão, `didOpen`, `didChange`, `didSave` e `didClose`
com as capacidades negociadas de cada servidor. Sincronização incremental ou
completa, encoding de posições e conteúdo devem manter a mesma identidade.
Reinício ou troca de configuração retira diagnósticos antigos antes de aceitar
os novos. A fusão dos diagnósticos de principal/companheiros conserva origem,
severidade e retirada de resultados; uma falha de transporte não é uma lista
vazia bem-sucedida. Controlar concorrência e invalidação por documento/contexto,
preservando operações independentes.

Coalescer consultas equivalentes sem perder pedidos relevantes. Cache, quando
introduzido, deve usar a identidade completa das entradas e dependências;
resultado de outra configuração não é reutilizável. Limitar filas, processos,
saída, memória e temporários; manter a UI e o IPC responsivos sob carga.

Medir latência de edição/diagnóstico/navegação, consultas enviadas, cancelamentos,
CPU e memória em repouso e sob carga. Comparar execuções com a mesma fixture,
toolchain e configuração, incluindo cache frio/quente. A supressão atual de
`didChange` redundante é uma melhoria localizada; não constitui medição de ganho
global nem aceite de toda a orquestração.
Os limites de 1 MiB/5 s pertencem ao verificador inicial; provedores adicionais
precisam de orçamento justificado por medição, sem bloqueio síncrono do editor.

## 12. Fatias e aceite da base

| Ordem | Entrega técnica | Evidência de saída |
| --- | --- | --- |
| 1a | Contexto e jornada C de host | Projeto C puro: biblioteca + executável, edição, includes, build, testes e depuração |
| 1b | C++ e comparação ampliada conforme o corpus | Projeto C++ com dependências e várias unidades; correção válida e falhas introduzidas |
| 2a | Rust e núcleo compartilhado com consumidor real | Workspace Cargo, features/target, diagnósticos, verificação, build/testes e depuração |
| 2b | Integração Rust/C e Rust/C++ | Fixtures de interface positivas/negativas, invalidação bilateral, ligação e execução |
| 2c | Desenvolvimento da própria IDE | Jornada Rust/C++/QML, IPC, reconstrução, testes, recuperação e contexto externo conforme 62 §5 |
| 3 | Python e integração com componentes nativos | Ambiente correto, análise, imports, testes/depuração e extensão nativa exercitada |
| 4 | Especializações | Corpus e dependências reais por perfil; aceites dos roadmaps correspondentes |

Polimento de UI/HUD/UX e otimização acompanham a jornada implementada; integridade
e recuperação têm precedência. A matriz deve incluir projeto novo e existente,
dependência ausente, falha/cancelamento, troca de workspace e reabertura, sempre
com preservação de conteúdo e prévia das alterações.
Registrar por capacidade se está implementada, integrada e comprovada, com
fixture, versões e alcance. Cada fatia fecha regressões, mutações das guardas,
falsos resultados e tempo. Compilar a IDE ou passar um gate isolado não aceita
todo o ciclo de uso. Este plano não altera o estado de implementação por si só.

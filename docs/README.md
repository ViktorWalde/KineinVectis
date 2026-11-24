# Documentação do core em C++23

Esta árvore documenta só o core (backend) da Kinein Vectis reescrito em C++23,
na pasta `core/`. A UI em QML, o core em Rust e a documentação da série 0.3
continuam em `ui/`, `crates/` e `DocsPublic/`. Nenhum arquivo daqui manda
neles, e nenhum deles manda no core novo. Quando um fato da 0.3 serve de
evidência, ele é citado pela origem, não copiado.

Estado: rascunho de 2026-11-22, para revisão do Viktor.

## O mapa

Cada arquivo responde a uma pergunta e é o único dono dos fatos dessa
pergunta. Os outros apontam para ele.

| Arquivo | Pergunta que responde |
| --- | --- |
| [lessons-from-0.3.md](lessons-from-0.3.md) | O que a 0.3 errou e acertou, e o que impede a repetição de cada erro? |
| [pending-decisions.md](pending-decisions.md) | O que ainda é decisão do Viktor, com a recomendação? |
| [adr/ADR-0012-cpp23-backend-with-neovim.md](adr/ADR-0012-cpp23-backend-with-neovim.md) | Por que C++23 e Neovim, e o que essa decisão substitui? |
| [adr/ADR-0013-single-source-of-the-contract.md](adr/ADR-0013-single-source-of-the-contract.md) | De onde vem o contrato entre a UI e os dois cores, e como ele muda? |
| **Requisitos** | |
| [requirements/functional-requirements.md](requirements/functional-requirements.md) | O que o core faz, em que ordem, e o teste de aceite de cada capacidade |
| [requirements/non-functional-requirements.md](requirements/non-functional-requirements.md) | Com que limite medido: tempo, memória, segurança, durabilidade, qualidade |
| **Arquitetura** | |
| [architecture/boundaries-and-responsibilities.md](architecture/boundaries-and-responsibilities.md) | Quais processos e componentes existem, quem decide o quê e quem depende de quem |
| [architecture/modules-and-layers.md](architecture/modules-and-layers.md) | Quais módulos `core/lib` tem, em que camada, quem usa quem e o que impõe isso (com os diagramas) |
| [architecture/execution-and-concurrency.md](architecture/execution-and-concurrency.md) | Onde o trabalho roda, quem é dono de cada dado, prazo e cancelamento |
| [architecture/processes-and-trust.md](architecture/processes-and-trust.md) | Como um processo nasce e o que o projeto não confiável pode fazer |
| [architecture/persistence-and-durability.md](architecture/persistence-and-durability.md) | Onde cada arquivo próprio mora e quanto texto uma queda pode levar |
| [architecture/protocol-and-contract.md](architecture/protocol-and-contract.md) | O fio entre a UI e o core, a troca de core e a mudança de contrato |
| [architecture/neovim-editing-engine.md](architecture/neovim-editing-engine.md) | O que o Neovim é dono, o que o core é dono e as condições da troca |
| **Engenharia** | |
| [engineering/cpp23-rules.md](engineering/cpp23-rules.md) | Os métodos de C++ do projeto e o mecanismo que impõe cada um |
| [engineering/known-cpp-defects-and-defenses.md](engineering/known-cpp-defects-and-defenses.md) | Os defeitos clássicos da linguagem e a defesa contra cada um |
| [engineering/build-profiles-and-hardening.md](engineering/build-profiles-and-hardening.md) | Os perfis de build, as flags, as fontes e como rodar |
| [engineering/static-analysis-clang-tidy.md](engineering/static-analysis-clang-tidy.md) | O que o clang-tidy confere, o que está desligado e por quê |
| [engineering/test-strategy.md](engineering/test-strategy.md) | As camadas de teste, o diferencial contra o Rust e as medidas de desempenho |
| [engineering/gates-and-effectiveness.md](engineering/gates-and-effectiveness.md) | Em que ordem os gates rodam e como um gate novo prova que funciona |
| [engineering/definition-of-done.md](engineering/definition-of-done.md) | Quando uma mudança ou um marco pode ser chamado de pronto |

Ordem de leitura para quem chega: este arquivo, `lessons-from-0.3.md`,
`architecture/boundaries-and-responsibilities.md`, `architecture/modules-and-layers.md`,
`engineering/cpp23-rules.md` e `engineering/definition-of-done.md`. O resto se lê quando o assunto aparece.

## As regras desta árvore

1. **Um fato, um dono.** Se o fato já tem dono aqui, os outros arquivos
   apontam para a seção (R-DOC-03 da 0.3).
2. **A regra que uma ferramenta impõe mora na ferramenta.** O documento diz
   qual ferramenta impõe e por quê; ele não repete a lista de flags nem de
   checagens. O CMake, o `.clang-tidy` e os testes são a verdade.
3. **Cada afirmação técnica diz de onde veio:** medido (com data e método),
   lido (com arquivo e linha), fonte primária (com link) ou **hipótese**. O
   que ninguém conferiu é marcado como NÃO VERIFICADO.
4. **Arquivo novo só com pedido do Viktor.** Assunto novo entra primeiro como
   seção do dono mais próximo.
5. **ADRs** ficam em `adr/`. A numeração continua a de
   `DocsPublic/decisoes-adr/` (a última é a ADR-0011), para nenhum número
   apontar para duas decisões.
6. **Até 200 linhas por arquivo**, e cada assunto numa seção `##` de nome
   explícito, que se entende sozinha. Passou disso, o arquivo se divide por
   responsabilidade (regra 4).
7. **As primeiras linhas dizem o dono.** Quem lê decide por elas se precisa
   do arquivo, e então lê só a seção de que precisa.
8. **Estado, não história.** O arquivo diz o que vale agora e de onde veio.
   Como se chegou lá fica na mensagem do commit. A exceção é o registro de
   medidas dos gates, que é a prova de cada gate.
9. **A 0.3 se consulta pelo identificador** (`INF-26`, `ENG-51`) ou pela
   linha (`62:1371`), e nunca lendo um roadmap inteiro: o 62 sozinho tem
   2.183 linhas. O resumo é o `lessons-from-0.3.md`.

As regras 6 a 9 ainda são convenção: o mecanismo é a P19
([decisões](pending-decisions.md)).

## Como citar a 0.3

`62:1371` quer dizer `DocsPublic/roadmaps/62-base-publica-0.4.md`, linha 1371,
no commit `a9f01d51`. A mesma forma vale para o 19
(`DocsPublic/arquitetura/19-compromissos-de-arquitetura.md`) e para os
roadmaps 63 a 66. Arquivos de código são citados pelo caminho a partir da
raiz do repositório.

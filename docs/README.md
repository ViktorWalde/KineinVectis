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
| [licoes-da-0.3.md](licoes-da-0.3.md) | O que a 0.3 errou e acertou, e o que impede a repetição de cada erro? |
| [decisoes-pendentes.md](decisoes-pendentes.md) | O que ainda é decisão do Viktor, com a recomendação? |
| [adr/ADR-0012-backend-em-cpp23-com-neovim.md](adr/ADR-0012-backend-em-cpp23-com-neovim.md) | Por que C++23 e Neovim, e o que essa decisão substitui? |
| [adr/ADR-0013-fonte-unica-do-contrato.md](adr/ADR-0013-fonte-unica-do-contrato.md) | De onde vem o contrato entre a UI e os dois cores, e como ele muda? |
| **Requisitos** | |
| [requisitos/requisitos-funcionais.md](requisitos/requisitos-funcionais.md) | O que o core faz, em que ordem, e o teste de aceite de cada capacidade |
| [requisitos/requisitos-nao-funcionais.md](requisitos/requisitos-nao-funcionais.md) | Com que limite medido: tempo, memória, segurança, durabilidade, qualidade |
| **Arquitetura** | |
| [arquitetura/fronteiras-e-responsabilidades.md](arquitetura/fronteiras-e-responsabilidades.md) | Quais processos e componentes existem, quem decide o quê e quem depende de quem |
| [arquitetura/modulos-e-camadas.md](arquitetura/modulos-e-camadas.md) | Quais módulos `core/lib` tem, em que camada, quem usa quem e o que impõe isso (com os diagramas) |
| [arquitetura/execucao-e-concorrencia.md](arquitetura/execucao-e-concorrencia.md) | Onde o trabalho roda, quem é dono de cada dado, prazo e cancelamento |
| [arquitetura/processos-e-confianca.md](arquitetura/processos-e-confianca.md) | Como um processo nasce e o que o projeto não confiável pode fazer |
| [arquitetura/persistencia-e-durabilidade.md](arquitetura/persistencia-e-durabilidade.md) | Onde cada arquivo próprio mora e quanto texto uma queda pode levar |
| [arquitetura/protocolo-e-contrato.md](arquitetura/protocolo-e-contrato.md) | O fio entre a UI e o core, a troca de core e a mudança de contrato |
| [arquitetura/motor-de-edicao-neovim.md](arquitetura/motor-de-edicao-neovim.md) | O que o Neovim é dono, o que o core é dono e as condições da troca |
| **Engenharia** | |
| [engenharia/regras-de-cpp23.md](engenharia/regras-de-cpp23.md) | Os métodos de C++ do projeto e o mecanismo que impõe cada um |
| [engenharia/erros-conhecidos-de-cpp-e-defesas.md](engenharia/erros-conhecidos-de-cpp-e-defesas.md) | Os defeitos clássicos da linguagem e a defesa contra cada um |
| [engenharia/perfis-de-compilacao-e-hardening.md](engenharia/perfis-de-compilacao-e-hardening.md) | Os perfis de build, as flags, as fontes e como rodar |
| [engenharia/analise-estatica-clang-tidy.md](engenharia/analise-estatica-clang-tidy.md) | O que o clang-tidy confere, o que está desligado e por quê |
| [engenharia/estrategia-de-testes.md](engenharia/estrategia-de-testes.md) | As camadas de teste, o diferencial contra o Rust e as medidas de desempenho |
| [engenharia/gates-e-medida-de-eficacia.md](engenharia/gates-e-medida-de-eficacia.md) | Em que ordem os gates rodam e como um gate novo prova que funciona |
| [engenharia/definicao-de-pronto.md](engenharia/definicao-de-pronto.md) | Quando uma mudança ou um marco pode ser chamado de pronto |

Ordem de leitura para quem chega: este arquivo, `licoes-da-0.3.md`,
`arquitetura/fronteiras-e-responsabilidades.md`, `arquitetura/modulos-e-camadas.md`,
`engenharia/regras-de-cpp23.md` e `engenharia/definicao-de-pronto.md`. O resto se lê quando o assunto aparece.

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

## Como citar a 0.3

`62:1371` quer dizer `DocsPublic/roadmaps/62-base-publica-0.4.md`, linha 1371,
no commit `a9f01d51`. A mesma forma vale para o 19
(`DocsPublic/arquitetura/19-compromissos-de-arquitetura.md`) e para os
roadmaps 63 a 66. Arquivos de código são citados pelo caminho a partir da
raiz do repositório.

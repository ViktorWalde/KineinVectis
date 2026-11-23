# ADR-0012: o backend em C++23, com o Neovim como motor de edição

> **Estado:** rascunho de 2026-11-22, para revisão do Viktor.
> **Decisão do autor:** 2026-11-22 (backend em C++23); 2026-11-15 (Neovim, DP-10 do 64).
> A numeração continua a de `DocsPublic/decisoes-adr/`.

## Contexto

Até a 0.3.9, a Kinein Vectis foi construída para uso próprio, e a arquitetura
seguiu a funcionalidade. A 0.4 precisa atender terceiros. A revisão de
2026-11-14 (64) e a avaliação do motor de edição (65) mediram defeitos que vêm
do desenho, e não de um ponto isolado:

- um projeto não confiável executava código dele por cinco caminhos (INF-23);
- o laço do core bloqueava, e nada tinha prazo nem cancelamento (ENG-50);
- processo criado em cerca de 122 lugares (ENG-51);
- o texto viajava inteiro a cada tecla, e abrir 20 mil linhas levava até 47 s
  (ENG-55, INF-25);
- o rascunho não tinha prazo máximo de gravação (INF-26);
- os gates de texto detectavam 36 de 120 defeitos (66).

A lista completa, com a evidência, está nas [lições](../licoes-da-0.3.md).

## Decisão

1. **O core é reescrito em C++23**, na pasta `core/`, com uma biblioteca sem
   Qt (`core/lib`) e um adaptador fino (`core/qt`). O core em Rust não recebe
   mais evolução.
2. **O motor de edição é o Neovim** embutido (`nvim --embed`, msgpack-RPC,
   `ext_linegrid`), nas condições do 65 §21
   ([motor de edição](../arquitetura/motor-de-edicao-neovim.md)).
3. **O frontend QML fica.** As views e o tema são reaproveitados; os
   controllers passam pelo teste de fronteira
   ([fronteiras](../arquitetura/fronteiras-e-responsabilidades.md)).
4. **O escopo é só C e C++** até o núcleo estar sólido. Rust e Python voltam
   depois.
5. **A migração é por estrangulamento**, no mesmo repositório:
   - o core em C++ fala o mesmo protocolo, e a UI escolhe o core por
     `KINEIN_CORE_BIN`;
   - o core em Rust é o oráculo do teste diferencial;
   - sempre existe uma IDE funcionando, para o dogfooding.
6. **O core em Rust só é apagado** quando o core em C++ passar nos requisitos
   não funcionais do N0 e do N1 e no teste diferencial, salvo a lista de
   divergências intencionais. Antes de apagar, cria-se a tag
   `v0.3-rust-final`.

## Por quê

**A CONFIRMAR PELO VIKTOR.** Os motivos abaixo saem do plano de 2026-11-22;
nenhum documento anterior os registra:

- quem escreve a lógica do core quer dominar o mecanismo, e a biblioteca de
  algoritmos (ambiente C/C++, embarcados, semântica) é o diferencial do
  produto;
- o produto é uma IDE para C e C++, desenvolvida nela mesma: o código do core
  passa pelo mesmo clangd, pelo mesmo CMake e pelos mesmos sanitizers que a
  IDE oferece a quem a usa;
- a UI já é Qt e C++: uma linguagem a menos no produto.

## O que esta decisão substitui

- **A conclusão do 64 §1** (64:71-77), que descartou a reconstrução ampla e
  escolheu a estratégia B: trocar quatro costuras dentro do core em Rust.
- **As fases B1 a B11 do 64 §8**, como plano de código. As soluções que elas
  descreviam (lançador com confiança, classes de execução, prazo e
  cancelamento, contrato com fonte única, três classes de persistência) passam
  a ser requisitos do core em C++.
- **A proposta do 66** (os gates antes do B1), que aguardava decisão: os gates
  de texto da 0.3 não entram no core em C++.
- **A frase do 62:2079-2080**, de fechar dívida "pelo comportamento comprovado,
  não pela conclusão de um rewrite". A troca de core só acontece pelo
  comportamento comprovado: o critério do item 6 é medida, não conclusão.

**Itens que fecham pela remoção do core em Rust**, sem correção própria:
INF-24, INF-29, INF-30, INF-31 e ENG-57. **Itens que viram requisito do core
em C++**: INF-23, ENG-50 a ENG-56 e INF-22. **Itens da UI** (INF-25 a INF-28)
fecham com a remoção do editor atual na fase E do 65, e não com o backend.

## Alternativas consideradas

| Alternativa | Por que não |
| --- | --- |
| A estratégia B do 64, no Rust | Era o plano decidido em 2026-11-14 (DP-01 a DP-09). A decisão de 2026-11-22 o revê, pelos motivos da seção "Por quê" (a confirmar) |
| C++26 | Não publicado (a votação DIS de 2026-10-23 reprovou); o GCC 16 marca o suporte como experimental; o Clang oficial 23 não tem reflexão, e o clangd não entenderia o código. Recursos do C++26 entram um por vez, por ADR |
| Reescrever também a UI | A UI em QML funciona e é a IDE do dogfooding; os defeitos medidos estão no core, na ponte e no editor |

## Consequências

- Durante a migração há dois cores, e todo comportamento novo do core em C++
  é conferido contra o oráculo ou contra a lista de divergências.
- O contrato precisa de uma fonte que não dependa do Rust antes de o Rust sair
  ([decisões](../decisoes-pendentes.md), P02).
- Os defeitos da 0.3 seguem no produto em uso até a troca: o INF-26 e o INF-23
  pedem tratamento próprio (P10).
- A regra "só o lançador cria processo" perde a imposição pelo compilador que
  o Rust dava (`disallowed_methods = "forbid"`) e precisa de outro mecanismo,
  com eficácia medida.

## O que não foi verificado

- A reação da UI a `MethodNotFound` nos pedidos que ela manda sozinha.
- O esforço da reescrita: nenhuma estimativa foi medida.
- Windows e MSVC.

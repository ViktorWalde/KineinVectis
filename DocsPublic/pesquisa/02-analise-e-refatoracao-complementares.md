# Análise e refatoração complementares — o estudo

> **Classe: PESQUISA.** Estudo autorizado pelo autor em 2026-10-19 para o
> longo prazo. Autoriza investigar algoritmos próprios que complementem os
> motores existentes; não promete recurso, versão ou equivalência à
> JetBrains. A direção do produto e o estado atual têm dono no
> [56 §1.1](../roadmaps/56-motor-semantico-profundo-cpp-rust.md#11-direção-de-longo-prazo-recuperada-em-2026-10-19).

## 1. O que falsearia uma hipótese

Uma proposta não avança se produzir transformação incorreta, usar fatos de
um contexto desatualizado ou não demonstrar melhoria frente ao fluxo atual.
Se exigir duplicar um compilador ou language server para funcionar, está
fora deste recorte. Se consumir mais CPU, RAM ou tempo do que o ganho
medido, sua alegação de otimização falha. Cada experimento precisa dizer
antes de rodar qual é a medida e qual resultado o reprova.

## 2. O objetivo e a fronteira

Pedido do autor: análises e refatorações próprias como algoritmo
**complementar**, uma melhoria sobre a composição da IDE, sem substituir
clangd, rust-analyzer ou o compilador. A restrição absoluta antiga de
"nenhum refactor próprio" foi refinada para permitir este estudo; não
se trocou a autoridade semântica do produto.

Entradas possíveis: contexto efetivo e sua origem, grafo de projeto,
artefatos gerados, diagnóstico do compilador, informações dos servidores e
sintaxe local. Cada fato mantém origem, versão e contexto. Uma estrutura
sintática não ganha informação de tipos só por ser chamada de semântica.

## 3. Linhas candidatas, não implementações aprovadas

| Linha | Hipótese a testar | Comparação necessária |
| --- | --- | --- |
| Correlação de contexto | Relacionar target, flags, código gerado e diagnósticos pode explicar inconsistências que hoje exigem investigação manual | Mesmo defeito, projeto e toolchain, com e sem a análise complementar; acertos, falsos alertas e tempo para corrigir |
| Agendamento e invalidação | Conhecer dependências/contextos pode evitar consultas e análises redundantes mantendo as respostas corretas | Latência, CPU/RAM, quantidade de trabalho e resultados após edição, troca de target e rebuild |
| Assistências entre código e projeto | Combinar fatos dos motores com o modelo do workspace pode propor mudanças que cruzam configuração, arquivo gerado e código | Propostas corretas e recusas explícitas diante de contexto incompleto; comparação com o fluxo atual e a ferramenta upstream |
| Refatorações complementares | Transformações delimitadas podem acrescentar assistências sem reimplementar resolução de tipos | Casos positivos e negativos, preservação do comportamento, interação com macros/templates/condicionais e versões dos arquivos |

Esta pesquisa não autoriza pular compilações: a hipótese específica de
evitar recompilação tem seus próprios critérios e dono no
[01](01-evitar-recompilacao.md). Também não interpreta "otimização" como
autorização para alterar o código gerado pelo compilador.

## 4. Como conduzir cada experimento

1. Escolher uma dor e um recorte pequeno em projeto real; conferir primeiro
   se o servidor/ferramenta já resolve o problema e qual parte falta na IDE.
2. Registrar baseline, versões, conjunto de casos, orçamento de recursos,
   ganho esperado e critérios de fracasso. Não começar por um engine geral.
3. Provar fora do caminho principal, inicialmente observando e produzindo
   propostas, sem aplicar edições no projeto do usuário.
4. Para transformação, validar contexto e versões; usar a prévia e a
   transação de edição existentes. Testes/build são evidências necessárias
   quando aplicáveis, junto de casos de preservação de comportamento e de
   recusa; não bastam sozinhos para concluir correção geral da transformação.
5. Medir resultado, custo, limitações e alternativa de não adotar. Publicar
   também o resultado negativo, em vez de promovê-lo por tempo investido.

Promoção a produto exige desenho e decisão registrados, escopo explícito,
integração aos serviços existentes, gates e saída reversível. Até lá,
esta página é pesquisa autorizada, não uma fila de implementação da 0.4.

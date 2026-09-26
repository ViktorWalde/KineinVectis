# Evitar recompilação por entender o código — a hipótese

> **Classe: PESQUISA** (`README.md`). Não é alvo. Em 2026-09-26 nada disto
> existe, nada está decidido, e o autor está preparando documentação técnica
> mais detalhada sobre o assunto.
>
> **Origem:** relatório de pesquisa do autor, 2026-09-26. A parte
> implementável dele foi extraída para
> [`../especificacoes/cache-de-compilacao.md`](../especificacoes/cache-de-compilacao.md);
> o que sobrou é isto.

## 1. A pergunta

Um cache de compilação responde *"já compilei exatamente isto antes?"*. A
pergunta desta pesquisa é outra, e bem mais difícil:

> *"esta mudança **precisa** recompilar aquilo?"*

Um comentário alterado num cabeçalho recompila tudo que o inclui. Um `private:`
novo numa classe muda o layout e recompila com razão. Entre os dois extremos há
uma faixa enorme de mudanças cujo efeito real é menor do que o grafo de
inclusão sugere — e é essa faixa que a hipótese persegue.

## 2. O que falsearia a hipótese

Esta seção vem antes das ideias, de propósito.

```text
UM ÚNICO FALSO-NEGATIVO INVALIDA A LINHA INTEIRA.
```

Um falso-negativo aqui é: *"não precisa recompilar"* quando precisava. O
resultado não é lentidão — é um binário **errado**, montado de partes que não
combinam, que compila, linka e roda. Numa IDE que também grava firmware em
placa, isso vira uma placa com um binário que ninguém consegue explicar.

Portanto:

1. Se o modo sombra (§4) acusar **um** falso-negativo em uso real, a linha não
   avança — e o registro dessa medição vale mais que o resto deste documento.
2. Se a análise custar mais tempo do que a recompilação que ela evita, a linha
   não avança. Medir isso é parte do experimento, não um detalhe.
3. Se a resposta só for confiável em C++ "bem-comportado" — sem macro, sem
   `#ifdef` agressivo, sem template pesado —, ela não serve para o código real.

## 3. As etapas, na ordem em que fazem sentido

Cada uma só começa se a anterior tiver medida favorável.

**P1 — contexto.** Entender o que muda entre duas versões de um arquivo usando
o que a IDE **já tem**: a árvore do tree-sitter e o índice. Sem promessa de
pular nada; só medir quantas mudanças são localmente irrelevantes.

**P2 — diff de AST.** Classificar a mudança: corpo de função, assinatura,
layout de tipo, macro. A literatura e as ferramentas de build incremental de
outras linguagens (Rust com o seu próprio modelo de dependência fina, por
exemplo) mostram que a classificação é possível; o que não está mostrado é que
ela é **segura** em C++ com pré-processador.

**P3 — *fingerprints* semânticas.** Uma identidade por símbolo que não mude
quando a mudança não puder afetar quem o usa. É onde a hipótese vira difícil de
verdade.

**P4 — modo sombra.** O único jeito honesto de ganhar confiança: a IDE **prevê**
o que poderia pular e **recompila tudo assim mesmo**, comparando. Métricas
opt-in, nada ligado por padrão, e a previsão nunca altera o build.

Pular de fato — se um dia — só depois de o modo sombra acumular **zero**
falso-negativos em uso real prolongado. E ainda assim, atrás de uma opção que a
pessoa liga.

## 4. Por que o modo sombra é a peça central

Porque ele transforma a hipótese em medição sem arriscar nada de quem usa a
IDE. Enquanto ele estiver ligado, o pior caso é gastar um pouco de CPU
prevendo o que não se usa — e o melhor caso é uma tabela com o número que
decide a pergunta.

## 5. O que esta pesquisa NÃO é

- Não é o cache de compilação. Aquilo é pronto, é de terceiros, é alvo, e está
  em [`../especificacoes/cache-de-compilacao.md`](../especificacoes/cache-de-compilacao.md).
- Não é compilação distribuída. Distribuir é o que `sccache` e `icecc` já
  fazem; a IDE orquestraria, não reinventaria.
- Não é um compilador próprio. A IDE **não compila**: ela chama o compilador do
  projeto, e isso não está em discussão.

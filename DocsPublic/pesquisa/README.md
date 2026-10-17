# Pesquisa — hipóteses de longo prazo, e por que ficam separadas

> **Classe: PESQUISA.** Criada em 2026-09-26. Não é alvo, não é estado, e
> **nada aqui é promessa de produto.**

## Por que esta pasta existe

O [ADR-0005](../decisoes-adr/ADR-0005-tres-arvores-de-documentacao.md) descreve por que documento cancelado sai do repositório:
documentos **grandes, completos e persuasivos** de coisas não decididas,
guardados dentro de `especificacoes/`, são lidos como **alvo** pela próxima
sessão. Deletar perderia o registro; deixar junto das especificações é
convidar a implementação.

O `legado/` resolve o caso do que foi **cancelado**. Faltava o caso oposto: o
que ainda **não foi decidido** — uma linha de pesquisa que o autor quer
perseguir, com risco real de não dar em nada, e que não pode ser confundida
com o que a IDE se comprometeu a fazer.

```text
especificacoes/   O ALVO. Alguém decidiu fazer. Diverge do código por
                  natureza, e reconcilia-se ao retomar.
pesquisa/         A HIPÓTESE. Ninguém decidiu fazer. Pode nunca sair daqui,
                  e sair daqui exige uma decisão registrada do autor.
```

## As regras desta pasta

1. **Todo documento começa dizendo o que falsearia a hipótese.** Pesquisa sem
   critério de fracasso é marketing.
2. **Nenhum número sem data e sem fonte**, como em toda a documentação.
3. **Sair daqui é um gesto explícito**: o conteúdo é extraído para uma
   especificação, com decisão datada no `roadmaps/40`. Nada "vira alvo" por
   envelhecer aqui dentro.
4. **Uma sessão que encontrar esta pasta não deve implementar nada dela.** Se
   parecer urgente, o caminho é perguntar ao autor — não começar.

## O que há aqui

- [`01-evitar-recompilacao.md`](01-evitar-recompilacao.md) — pular trabalho de
  compilação por análise própria do código: diff de AST, *fingerprints*
  semânticas, modo sombra. A parte **já pronta no mercado** deste assunto
  (ccache, sccache) não está aqui: ela é alvo, e mora em
  [`../especificacoes/cache-de-compilacao.md`](../especificacoes/cache-de-compilacao.md).

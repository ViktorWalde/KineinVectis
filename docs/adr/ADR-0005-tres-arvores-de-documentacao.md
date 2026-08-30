# ADR-0005 — Três árvores de documentação: `docs/`, `docs-privada/`, `docs-legada/`

> **Status:** **implementado** em 2026-08-29, com gate.
> **Data:** 2026-08-29.
> **Contraria:** a decisão de 2026-07-05 que removeu `docs/archive/` com a regra
> *"não recriar uma pasta de arquivo só para guardar"*. É por isso que este ADR
> existe: neste repositório, regra só muda por decisão explícita e registrada.

## Contexto

A documentação tem **61.671 linhas em 82 arquivos** — mais do que o core Rust e
a UI QML somados. Isso é uma escolha consciente (o projeto é conduzido por
sessões que trocam de contexto e precisam se reorientar do zero), mas cobra um
preço que ficou concreto.

Dentro de `docs/specs/` moravam **2.380 linhas em duas specs completas** —
`KINEIN_VECTIS_ASSISTANT_AI_ASSISTANCE.md` e
`KINEIN_VECTIS_AI_CLI_BRIDGE_EXTERNAL_TERMINAL.md` — descrevendo painel de IA,
chat, seletor de agente, aba dedicada e atalho no rail. Tudo **cancelado pelo
autor em 2026-07-17**, com o `aiBridge` removido do código no protocolo 0.59.0.

As duas tinham um aviso `⛔ FORA DE ESCOPO` no topo. **O aviso não segura.** Quem
chega pelo `SPEC_INDEX` vê o título; quem chega por `grep` cai no meio do
arquivo. Uma spec cancelada é grande, completa, bem escrita e persuasiva — é
exatamente o que uma sessão nova procura ao perguntar "para onde este projeto
vai?". O repositório já pagou por confiar em documento velho: em 2026-07-17 uma
IA reimplementou um seletor que o autor mandou remover **no mesmo dia**.

Havia uma segunda pressão, medida na mesma data: o `ContextoIA.md` (2.414 linhas
de log datado) esteve em **primeiro** na ordem de precedência, descrito como
"estado real". Foi rebaixado em 2026-07-17, mas continuava no caminho de leitura
de quem só queria trabalhar.

## Decisão

Três árvores, e uma regra: **uma sessão de trabalho lê `docs/` e mais nada.**

```text
docs/            LIDA EM TODA SESSAO. Contrato, estado, plano, specs, build.
                 Auto-suficiente para trabalhar.
docs-privada/    Continuidade interna: ContextoIA (log datado), diario de
                 sessoes, prompts de bootstrap. Consulta SOB DEMANDA, para
                 "por que isto ficou assim?" — nunca para "o que existe".
docs-legada/     Superado ou CANCELADO. Nunca e' alvo.
```

**Critério de entrada em `docs-legada/`, deliberadamente estreito: só entra o
que uma sessão poderia confundir com alvo.** O que for descontinuado e
inofensivo continua seguindo a regra de 2026-07-05 — extrair o que tem valor e
**remover**. Legar é para o que engana.

**Mão única: entra e não volta.** Se algo lá dentro voltar a valer, o *conteúdo*
é extraído para o documento vivo; o arquivo legado permanece, como registro.

## Por que não as alternativas

```text
DELETAR    perde o registro da decisao de produto. O "por que NAO" e' a parte
           cara de descobrir; o "o que" se reescreve numa tarde.
DEIXAR     convida a reimplementacao, e foi o estado que ja mordeu o projeto.
           O aviso no topo do arquivo nao alcanca quem chega pelo indice ou
           pelo grep.
SEPARAR    custa uma pasta e uma regra. E' o que foi feito.
```

## Consequências

- `AGENTS.md` ganhou o passo 0: ler `docs/` e só `docs/`. O log deixou de ser
  o item 1 da ordem de leitura.
- `docs/LEITURA_TECNICA.md` passa a ser a primeira leitura de quem é novo.
- `scripts/exportar-copia-limpa.sh` nega as três árvores (todas internas).
- `scripts/verificar-docs.sh` trata `docs-legada/` como registro: número lá
  dentro descreve o que era verdade quando o documento valia.
- **Gate novo, `scripts/verificar-links-docs.sh`.** `git mv` não atualiza link
  nenhum, e este repositório já moveu documentação assim uma vez (2026-07-16:
  410 referências reescritas à mão). Link morto em Markdown não tem compilador:
  o gate fica verde e o próximo a ler conclui que o documento não existe. Foi
  escrito e testado por mutação **antes** de a reorganização começar, e foi ele
  que apontou os 5 links que a mudança quebrou.

## O risco que este ADR aceita

`docs-legada/` pode virar depósito — que é exatamente o que a decisão de
2026-07-05 matou. A defesa é o critério estreito escrito acima e o
`docs-legada/README.md`, que exige **motivo e data** por documento. Se um dia a
pasta crescer com material que ninguém confundiria com alvo, a regra foi
violada, não revista.

# docs-legada — superado ou cancelado

> **Nada aqui é alvo.** Não implemente a partir destes documentos, não os cite
> como plano e não os traga de volta para `docs/`. Uma sessão de trabalho lê
> `docs/` e mais nada (`docs/README.md`, "As três árvores").

## Por que esta pasta existe

Documento cancelado que continua dentro de `docs/specs/` é uma armadilha: ele é
grande, completo, bem escrito e persuasivo — exatamente o que uma sessão nova
procura quando quer saber para onde o projeto vai. Foi o que aconteceu com a
linha de IA na IDE, cancelada em 2026-07-17: **2380 linhas em duas specs**
descrevendo painel, chat, seletor de agente e aba dedicada, tudo morto.

Havia três saídas e as três têm custo:

```text
DELETAR      perde o registro da decisao de produto. O "por que nao" e' a
             parte cara de descobrir; o "o que" se reescreve em uma tarde.
DEIXAR       convida a reimplementacao. O aviso no topo do arquivo nao segura:
             quem chega pelo indice ou pelo grep nao le o topo.
SEPARAR      custa uma pasta e uma regra. E' o que este diretorio e'.
```

## A regra

**Mão única: entra e não volta.** Se algo aqui voltar a valer, o *conteúdo* é
extraído para o documento vivo relevante em `docs/`; o arquivo legado
permanece, como registro do que foi decidido e quando.

Isto **não** é depósito. `docs/archive/` foi removida em 2026-07-05 justamente
por ser depósito, e essa decisão continua de pé. O critério de entrada aqui é
estreito: **só entra o que uma sessão poderia confundir com alvo.** O que for
descontinuado e inofensivo se extrai e se remove.

## O que está aqui e por quê

| Documento | Motivo | Data da decisão |
| --- | --- | --- |
| `KINEIN_VECTIS_ASSISTANT_AI_ASSISTANCE.md` | Linha de IA na IDE cancelada pelo autor | 2026-07-17 |
| `KINEIN_VECTIS_AI_CLI_BRIDGE_EXTERNAL_TERMINAL.md` | Idem; o `aiBridge` saiu do código no protocolo 0.59.0 | 2026-07-17 |
| `KINEIN_VECTIS_KV_CONTEXT_AI_ASSISTANCE.svg` | Diagrama do KV Context, construído e removido no mesmo dia | 2026-07-17 |
| `17-architecture-hygiene-plan.md` | Fase concluída; os números envelheceram 3,4x e enganaram uma sessão. Guardrails vivos em `docs/arquitetura/ARCHITECTURE.md` §4 | 2026-07-06 |
| `PLANO_ORGANIZACAO_E_HANDOFF.md` | Descreve o estado **anterior** à reorganização de 2026-07-16, já executada. As faixas P/T/X foram extraídas para `docs/README.md` | 2026-07-16 |

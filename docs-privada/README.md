# docs-privada — continuidade interna

> **Não é leitura de sessão.** Uma sessão de trabalho lê `docs/` e mais nada
> (`docs/README.md`, "As três árvores"). Esta pasta se consulta **sob demanda**,
> para uma pergunta específica — nunca para descobrir o estado do projeto.

## O que mora aqui, e para qual pergunta

```text
ContextoIA.md    LOG datado, append-only. Responde "POR QUE isto ficou assim?".
                 NAO responde "o que existe hoje?" — isso se mede no codigo.
diario/          Registro de sessoes: marcos de dogfooding, decisoes por sessao
                 e a escada de rigor. Processo, nao contrato.
  18-daily-driver-plan.md    as fatias e a escada de rigor.
  19-registro-de-saidas.md   append-only: cada SAIDA da Kinein para outra
                 ferramenta, com reproducao minima. E' o que ordena a frente C
                 do roadmaps/34 por dor real. Entrada sem reproducao nao conta.
prompts/         Bootstrap de retomada em terminal.
```

## Por que saiu de `docs/`

O `ContextoIA.md` já esteve em **primeiro** na ordem de precedência, descrito
como "estado real e decisões vigentes". Ele é um log de 44+ entradas datadas, e
log em primeiro é o mecanismo que faz uma sessão nova confiar num registro
velho: em 2026-07-17 uma IA reimplementou um seletor que o autor mandou remover
no mesmo dia, e listou como pendente um harness entregue havia 24 horas.

Ele foi rebaixado na ordem de precedência naquela data. Movê-lo para fora da
árvore de leitura, em 2026-08-29, é a mesma decisão levada até o fim: **o log
deixa de estar no caminho de quem só quer trabalhar.** Quem precisa do "por quê"
vem aqui de propósito, e sabe que está lendo um registro datado.

O mesmo vale para o `diario/`: 2694 linhas de processo que o `docs/README.md` já
classificava como "material interno; não publicado".

## Onde está o estado, então

```text
o que existe        -> o CODIGO e os gates. Mede-se.
a fila de trabalho  -> PONTO_ATUAL.md (raiz)
o mapa              -> GUIAIA.md (raiz)
as regras           -> AGENTS.md (raiz) + docs/arquitetura/ARCHITECTURE.md
o alvo              -> docs/specs/ e docs/roadmaps/
```

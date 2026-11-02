# ADR-0005 — Três árvores de documentação: a pública, a interna e o legado

> **Status:** implementado em 2026-08-29; organização atualizada em
> 2026-09-12 e política de referências públicas consolidada em 2026-10-01.
> **Data:** 2026-08-29.

## Contexto

Especificações canceladas no índice de desenvolvimento podem ser confundidas
com requisitos vigentes. Em 2026-07-17, o painel de IA e a ponte dedicada de
CLI foram retirados do escopo e o `aiBridge` foi removido no protocolo 0.59.0.
Essas especificações precisavam deixar de orientar a implementação.

Registros históricos também não definem o estado atual: números e provas
valem para a data declarada. O código e os gates determinam o que existe.

## Decisão

A árvore `DocsPublic/` reúne documentação pública autossuficiente: contratos,
estado, planos, especificações vigentes, requisitos de build e registros
estritamente técnicos. Documentos públicos só apontam para fontes públicas.

A organização inicial de três árvores, de 2026-08-29, separava documentação
vigente, notas locais e material superado. Em 2026-09-12, as duas últimas
passaram a uma área local ignorada pelo Git. Instruções de ferramentas,
conversas e dados pessoais não integram a distribuição.

Especificações canceladas deixam o índice vigente. Uma decisão recuperada
exige novo registro técnico datado no documento responsável; o conteúdo
histórico não volta automaticamente a ser requisito.

## Por que não as alternativas

- Manter uma especificação cancelada junto das vigentes, apenas com aviso,
  permite que índices ou buscas apresentem seu conteúdo como alvo.
- Apagar a justificativa técnica de uma decisão elimina contexto útil para
  revisão. A justificativa pública deve ser curta, datada e independente de
  relatos pessoais.
- Misturar registro histórico e estado atual torna contagens e pendências
  ambíguas. Cada classe deve ter precedência e finalidade explícitas.

## Consequências

- `DocsPublic/00-comece-aqui.md` organiza a leitura por contexto técnico.
- `DocsPublic/leitura-tecnica.md` apresenta o estado implementado.
- `scripts/exportar-copia-limpa.sh` exclui notas e orientações locais.
- `scripts/verificar-docs.sh` confere números sem data contra o código;
  medições históricas permanecem identificadas pela data.
- `scripts/verificar-links-docs.sh` verifica referências, índices e a
  separação entre material público e local.

## O risco que este ADR aceita

A separação exige revisão editorial contínua. Gates detectam padrões e links
inválidos, mas não substituem a leitura para identificar relatos pessoais ou
requisitos superados. Publicar somente o fato técnico verificável.

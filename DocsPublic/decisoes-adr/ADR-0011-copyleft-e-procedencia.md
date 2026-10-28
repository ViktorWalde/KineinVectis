<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# ADR-0011 — Copyleft, atribuição e procedência das contribuições

**Data:** 2026-10-28. **Estado:** aceito.

## Contexto

O licenciamento `MIT OR Apache-2.0` permite derivados proprietários. O
projeto passa a exigir reciprocidade sobre o código abrangido nas hipóteses
de distribuição e de interação remota previstas por uma licença livre
padronizada, preservando as permissões concedidas anteriormente.

## Decisão

- Código próprio: **GNU AGPL versão 3 somente**, `AGPL-3.0-only`, com texto
  integral e inalterado no arquivo `LICENSE` da raiz.
- Documentação própria: **CC BY-SA 4.0**, ressalvadas licenças específicas,
  trechos de código e materiais de terceiros.
- Atribuição em `NOTICE` e `AUTHORS.md`; identidade oficial tratada em
  `TRADEMARKS.md`, sem restringir os direitos de modificar ou redistribuir.
- Contribuições novas com **DCO 1.1**, sem cessão obrigatória de direitos;
  autorização do titular competente quando a contribuição a exigir.
- Manifestos Cargo e AppStream e documentos instalados devem refletir a
  licença do software. A auditoria das dependências continua restrita às
  licenças já aprovadas; a AGPL é autorizada especificamente nos crates próprios.

O escopo, as exceções e as obrigações têm um único dono:
[`../contribuindo/12-licenciamento-e-procedencia.md`](../contribuindo/12-licenciamento-e-procedencia.md).
As condições para contribuições institucionais estão no
[13](../contribuindo/13-colaboracao-institucional.md), acompanhadas de
[minuta de acordo](../contribuindo/14-minuta-de-colaboracao-institucional.md).

## Alternativas e consequências

A GPLv3 oferece copyleft na distribuição, mas não inclui a condição de rede
da AGPLv3. A GPLv2-only não é adequada à combinação com componentes
Apache-2.0. GNU/Linux reúne componentes com licenças distintas; a licença do
kernel não é uma licença universal de aplicações Linux.

A migração não revoga licenças anteriores, não transfere titularidade e não
obriga o envio de alterações ao repositório oficial. Uso comercial e forks
continuam permitidos. Regras de aceitação e acordos de colaboração não são
restrições adicionais à licença pública. Materiais com licença específica
conservam seus termos e avisos.

## Verificação

Conferir os textos oficiais, os metadados Cargo/AppStream, a política do
`cargo-deny`, a instalação dos avisos e os gates documentais e de distribuição.
Esta decisão não altera protocolo, funcionalidades ou dependências da IDE.

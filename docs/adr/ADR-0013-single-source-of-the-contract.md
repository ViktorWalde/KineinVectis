# ADR-0013: a fonte única do contrato entre a UI e os cores

> **Estado:** rascunho de 2026-11-22, para aprovação do Viktor. Nada do que
> esta ADR descreve foi implementado: o `schemas/ipc.schema.json` não muda
> antes da aprovação.
> **Decisão do autor:** 2026-11-22 (P02: gerar a partir do `kinein-protocol`).

## Contexto

- **A verdade do contrato hoje são os tipos Rust do `kinein-protocol`**, na
  versão 0.223.0 (`crates/kinein-protocol/src/lib.rs:121`). O
  `schemas/ipc.schema.json` parou no 0.62.0 e só descreve o envelope
  (`schemas/ipc.schema.json:4`; ENG-54, L12).
- **Durante o estrangulamento há dois cores** falando com a mesma UI
  (ADR-0012). Sem uma fonte que os dois sigam, o core em C++ copiaria o
  contrato à mão, e a cópia divergiria como o schema divergiu.
- **O 64 §5.7** (64:556-577) já previa gerar o schema dos tipos Rust com o
  `schemars` e conferir o gerado contra o versionado. O `kinein-protocol` não
  usa o `schemars` hoje (lido em 2026-11-22: nenhuma ocorrência no crate).

## Decisão

1. **Os tipos do `kinein-protocol` geram o `schemas/ipc.schema.json`.** Cada
   método declara o nome, o tipo dos parâmetros e o tipo do resultado uma vez;
   o schema sai deles.
2. **O schema gerado é versionado.** Um teste do lado Rust gera de novo e
   reprova se o resultado for diferente do arquivo versionado. O schema nunca
   mais para.
3. **Os dois cores são conferidos contra o schema.** O teste de contrato do
   core em C++ valida cada pedido e cada resposta dos métodos implementados
   contra o schema (RNF-CON-01). O teste diferencial (ADR-0012) confere o
   comportamento; o schema confere a forma.
4. **Quando o core em Rust sair**, o schema versionado vira a fonte e passa a
   ser mantido à mão, sempre com ADR. A geração sai junto com o Rust.
5. **A versão do protocolo** continua uma só, a do `kinein-protocol`
   enquanto ele existir.

## Alternativas consideradas

| Alternativa | Por que não |
| --- | --- |
| Escrever um schema novo à mão, só do subconjunto N0 | Cópia manual de um contrato que ainda muda no Rust; repete a causa do ENG-54 |
| Decidir no primeiro marco, com o diferencial como única referência | O diferencial compara comportamento, não forma; um campo opcional que os dois omitem passa despercebido |
| Gerar cabeçalho C++ dos tipos Rust | O 64 §5.7 previa para a ponte da UI. Para o core novo, o desenho dos tipos do protocolo é do Viktor (`protocol`, [módulos](../architecture/modules-and-layers.md)); a geração pode entrar depois, por ADR própria |

## Consequências

- **Primeiro passo:** o `schemars` entra como dependência do `kinein-protocol`.
  É código novo em `crates/`, sem apagar nem mover nada.
- **O schema gerado substitui o atual inteiro.** Qualquer consumidor do
  schema 0.62.0 precisa ser conferido antes. NÃO VERIFICADO: quem lê o schema
  hoje além dos gates.
- **O core em C++ precisa de um validador de JSON Schema nos testes**, o que
  é uma dependência a aprovar (junto da P16).

## O que não foi verificado

- Se o `schemars` representa todos os tipos do `kinein-protocol` sem
  anotação manual (enums com dados, campos renomeados pelo `serde`).
- O tamanho e o tempo de geração do schema dos cerca de 250 métodos.

# Protocolo e contrato

Dono: o fio entre a UI e o core, o que o core em C++ implementa dele, como a
UI troca de core e como o contrato muda.

## O fio

- **JSON-RPC 2.0**, uma mensagem por linha, no stdin e no stdout do core
  (64:95-96; `ui/src/core_client_process.cpp:74` e `:220`).
- **Versão:** a do core em Rust, hoje 0.223.0
  (`crates/kinein-protocol/src/lib.rs:121`). O core em C++ informa a versão
  que implementa; a mudança de versão segue a regra da última seção.
- **A fonte do contrato hoje são os tipos Rust do `kinein-protocol`.** O
  `schemas/ipc.schema.json` parou no 0.62.0 e só descreve o envelope
  (`schemas/ipc.schema.json:4`; ENG-54). Não serve de referência para o core
  em C++ (L12).

## O que o core em C++ implementa

- **Um subconjunto**, pelos níveis dos
  [requisitos funcionais](../requirements/functional-requirements.md). Cada
  método implementado tem a mesma forma de pedido, resposta e erro do core em
  Rust, conferida pelo teste diferencial.
- **Método fora do subconjunto** responde com o erro `MethodNotFound`, que é o
  que o core em Rust já devolve para método desconhecido
  (`crates/kinein-protocol/src/rpc.rs:132` e `:195`).
- **Pedidos que a UI manda sozinha** ao abrir um workspace (`cmakeStatus`,
  `cmakePresetsList`, `cargoMetadata`, `runConfigList`, `gitStatus`; 64:110-114)
  têm de ter resposta definida no N0, mesmo que seja `MethodNotFound`. Como a
  UI reage a esse erro é NÃO VERIFICADO; o primeiro marco confere.

## A troca de core

```sh
KINEIN_CORE_BIN=/caminho/do/core-em-cpp ./kinein-vectis
```

A UI já lê essa variável (`ui/src/core_client_process.cpp:240`). Sem ela, a
UI sobe o core em Rust como hoje.

## Mudança de contrato

1. **Só por ADR** em [`docs/adr/`](../adr/), aprovada pelo Viktor. Isso inclui
   o `schemas/ipc.schema.json` e métodos novos que o 64 previa e a 0.223.0 não
   tem, como o `core.cancel` (64:431-432).
2. **O contrato vem antes do código** (R-IPC-01 do 63): o teste de contrato
   muda no mesmo commit.
3. **Uma fonte única** (RNF-CON-01): o schema gerado a partir dos tipos do
   `kinein-protocol`, com os dois cores conferidos contra ele, decisão do
   Viktor de 2026-11-22. Os detalhes estão na
   [ADR-0013](../adr/ADR-0013-single-source-of-the-contract.md), em rascunho:
   nada muda no schema antes da aprovação.

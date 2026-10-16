# `core.*` — o domínio no protocolo IPC

> Parte do [03](../03-protocolo-ipc.md), que tem o formato base, as listas
> canônicas de métodos e eventos e as regras. Movido sem mudar o texto em
> 2026-10-16 ([61](../../roadmaps/61-organizacao-da-documentacao.md) §3.7, O6).

## `core.*` — o handshake e o encerramento

Dois métodos, e eles são os únicos que **nunca** dependem de nada: não exigem
workspace, não tocam o filesystem e respondem mesmo com o resto do core inerte.

```text
core.ping     {}  ->  { protocolVersion, ... }
core.shutdown {}  ->  { message: "shutdown requested" }
```

**O `core.ping` é como a UI descobre com que protocolo está falando** — é o par
do `PROTOCOL_VERSION` em `crates/kinein-protocol/src/lib.rs`, e é o primeiro
pedido que a UI faz depois de subir o processo (`04-boot-e-comunicacao.md`).

**O `core.shutdown` pede, não mata.** Ele responde e deixa o encerramento
acontecer com o drain dos jobs em andamento, que é o que impede um build a meio
caminho de virar processo órfão.

# `cargo.*` — o domínio no protocolo IPC

> Parte do [03](../03-protocolo-ipc.md), que tem o formato base, as listas
> canônicas de métodos e eventos e as regras. Movido sem mudar o texto em
> 2026-10-16 ([61](../../roadmaps/61-organizacao-da-documentacao.md) §3.7, O6).

### Cargo service (`cargo.metadata` / `cargo.check`)

Implementado no protocolo `0.26.0` (fatia M2.3). Ambos exigem
workspace aberto com kind `rustCargo` (`INVALID_PARAMS` para outros kinds).

- `cargo.metadata {}` → `{ packages: [{ name, version, features: [...],
  targets: [{ name, kind }] }], workspaceMembers: [...] }`. Síncrono:
  `cargo metadata --format-version 1 --no-deps` é local e rápido (medido
  ~10ms num projeto pequeno); o core resume o JSON gigante do cargo.
  Falha vira erro humano (`INTERNAL_ERROR` com a última linha do stderr).
- `cargo.check {}` → `{ jobId }` (job cancelável). Roda `cargo check
  --workspace --all-targets --message-format=json` e **reusa o pipeline do
  quality**: diagnósticos chegam por `event.quality.diagnostic` e o fim
  por `event.quality.finished` — caem na aba Problemas sem parser novo.
  Aliasing consciente até o Problems 2.0 ter facetas por origem; o título
  do job ("Cargo Check") o distingue do clippy na aba Jobs.

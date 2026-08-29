# OBSOLETO — prompt de bootstrap inicial

> **Status:** histórico. Não use este arquivo.
> **Entrada atual:** [`AGENTS.md`](../../AGENTS.md).

Este prompt existia para criar a **base inicial** do workspace Rust: `core.ping`,
tipos básicos do protocolo, uma CLI de ping e a configuração estrita. Esse
estágio terminou há muito — hoje a Kinein tem core, protocolo, UI Qt/QML,
editor, LSP/DAP, build/run/debug, Git, terminal e empacotamento. O arquivo
chegava a instruir "não implementar Qt/LSP/CMake ainda".

Ele também apontava para documentos que não existem mais
(`docs/00-product-vision.md`, `docs/01-architecture.md`,
`docs/04-command-system.md`, `docs/10-mvp-plan.md`), removidos junto com
`docs/archive/` em 2026-07-05. Seguir este arquivo hoje levaria a caminhos
mortos e a reimplementar o que já existe.

## Por onde começar de verdade

```text
AGENTS.md          regras obrigatórias e ordem de leitura
docs-privada/ContextoIA.md      estado real e decisões vigentes
GUIAIA.md          mapa: domínio → documentos → arquivos → gate
PONTO_ATUAL.md     fila viva: a próxima ação executável
docs/README.md     índice da documentação técnica
```

O conteúdo anterior permanece recuperável pelo histórico Git.

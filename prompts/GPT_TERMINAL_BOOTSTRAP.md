# Prompt inicial para GPT/Claude no terminal

Você está trabalhando no projeto **Kernwerk Studio**.

Leia estes arquivos antes de responder ou alterar código:

1. `AGENTS.md`
2. `docs/00-product-vision.md`
3. `docs/01-architecture.md`
4. `docs/02-repository-structure.md`
5. `docs/03-ipc-protocol.md`
6. `docs/04-command-system.md`
7. `docs/06-strict-mode.md`
8. `docs/10-mvp-plan.md`

## Contexto

Kernwerk Studio é uma IDE open source, Linux-first, visualmente plug and play e rígida por padrão.

Arquitetura decidida:

```text
Qt/QML Frontend  ← IPC/JSON-RPC local → Rust Core
```

O projeto deve começar como Rust workspace. A UI Qt/QML será adicionada depois.

## Objetivo imediato

Criar a base inicial do projeto Rust com boas práticas modernas e máximo rigor.

## Tarefa inicial sugerida

Crie ou ajuste a estrutura:

```text
Cargo.toml
rust-toolchain.toml
crates/kernwerk-core/
crates/kernwerk-protocol/
crates/kernwerk-config/
crates/kernwerk-cli/
```

Implemente apenas:

- `core.ping`;
- tipos básicos do protocolo;
- CLI simples para enviar ping;
- testes unitários;
- configuração strict com fmt/clippy/test.

## Regras

- Não usar `unsafe`.
- Não usar `unwrap`, `expect` ou `panic` fora de testes.
- Não adicionar dependências desnecessárias.
- Não implementar Qt ainda.
- Não implementar LSP ainda.
- Não implementar CMake ainda.
- Não misturar UI com core.
- Atualizar documentação se mudar contrato.

## Comandos de validação

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo check --workspace --all-targets --all-features
```

Explique antes de alterar arquivos, mantenha o escopo pequeno e prefira passos incrementais.

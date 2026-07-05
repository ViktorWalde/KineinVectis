# 10 — Plano de MVP

> **Status:** implemented / histórico — O MVP descrito aqui foi entregue (core
> Rust rígido respondendo comandos, workspace, fs, build/test/quality, LSP,
> run/terminal via IPC 0.19.0). Mantido como registro do plano original; o
> estado atual e as próximas prioridades ficam em `ContextoIA.md`.

## MVP 0.1 — Core mínimo

### Objetivo

Ter um core Rust compilando com rigor máximo e respondendo a comandos simples.

### Escopo

- Criar workspace Rust.
- Criar `kernwerk-core`.
- Criar `kernwerk-protocol`.
- Criar `kernwerk-config`.
- Criar `kernwerk-cli`.
- Implementar `core.ping`.
- Implementar logging básico.
- Implementar resposta JSON.
- Implementar testes.
- Configurar fmt/clippy/test.

### Fora do escopo

- Qt/QML.
- Editor.
- LSP.
- CMake.
- Git.
- IA.
- Embedded.

## MVP 0.2 — Tool detection

- Detectar `cargo`, `rustc`, `cmake`, `ninja`, `git`, `clangd`.
- Retornar status estruturado.
- Sugerir instalação no CachyOS/Arch.
- Mostrar logs.

## MVP 0.3 — Workspace

- Abrir pasta.
- Identificar tipo de projeto:
  - Rust/Cargo;
  - CMake;
  - Maven/Gradle;
  - Python;
  - desconhecido.
- Salvar workspace básico em `.kernwerk/workspace.json`.

## MVP 0.4 — UI mínima

- Criar `ui/`.
- Qt/QML dark window.
- Conectar no core.
- Mostrar status.
- Botão Ping.
- Painel Logs.

## Primeiro comando para IA no terminal

Usar `prompts/GPT_TERMINAL_BOOTSTRAP.md`.

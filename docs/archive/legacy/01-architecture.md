# 01 — Arquitetura

## Visão geral

A arquitetura do Kernwerk Studio é separada em frontend visual e core/backend.

```text
┌──────────────────────────────────────────┐
│              Kernwerk UI                 │
│              Qt / QML                    │
│                                          │
│  - janelas                               │
│  - painéis                               │
│  - editor visual                         │
│  - command palette                       │
│  - configurações                         │
│  - temas                                 │
└─────────────────────┬────────────────────┘
                      │ IPC / JSON-RPC local
┌─────────────────────▼────────────────────┐
│              Kernwerk Core               │
│                 Rust                     │
│                                          │
│  - workspace                             │
│  - command system                        │
│  - settings                              │
│  - tooling lifecycle                     │
│  - LSP process manager                   │
│  - build manager                         │
│  - Git manager                           │
│  - AI provider manager                   │
│  - diagnostics                           │
│  - cache                                 │
└─────────────────────┬────────────────────┘
                      │ processos externos
┌─────────────────────▼────────────────────┐
│          Ferramentas externas            │
│ clangd, jdtls, pyright, cmake, ninja,    │
│ git, gdb, lldb, gradle, maven, uv, etc.  │
└──────────────────────────────────────────┘
```

## Camadas

### 1. UI Layer

Responsável apenas por apresentação e interação:

- renderizar janelas;
- receber cliques/atalhos;
- abrir menus;
- mostrar painéis;
- enviar comandos ao core;
- escutar eventos do core;
- nunca chamar `cmake`, `git`, `clangd` etc. diretamente.

### 2. Application/Core Layer

Responsável pela lógica da IDE:

- workspace;
- projetos;
- comandos;
- eventos;
- settings;
- toolchains;
- strict mode;
- validação;
- roteamento para serviços.

### 3. Adapter Layer

Responsável por falar com ferramentas externas:

- processo do clangd;
- processo do jdtls;
- processo do pyright;
- comandos CMake/Ninja;
- comandos Git;
- processos de terminal;
- providers de IA.

### 4. Protocol Layer

Responsável por contrato UI ↔ Core:

- JSON-RPC;
- schemas;
- tipos de request/response/event;
- versionamento do protocolo.

## Regras arquiteturais

1. UI não conhece detalhes de ferramentas externas.
2. Core não depende de Qt.
3. Serviços devem ser pequenos e testáveis.
4. Toda ação da IDE deve ser um comando.
5. Toda mudança assíncrona relevante deve emitir evento.
6. Erros devem ter mensagem humana e dados técnicos.
7. Configurações devem ser serializáveis.
8. Não bloquear thread de UI.
9. Não executar trabalho pesado no processo de UI.
10. Não adicionar dependência sem motivo documentado.

## Processos

No futuro, a aplicação pode ter:

```text
kernwerk-studio    # UI Qt/QML
kernwerk-core      # daemon/core Rust
kernwerk-cli       # CLI para automação e debug
```

No MVP inicial, pode existir apenas:

```text
kernwerk-core
kernwerk-cli
```

para validar protocolo, comandos, settings e logs antes da UI definitiva.

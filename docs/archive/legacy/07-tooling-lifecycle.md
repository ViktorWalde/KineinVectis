# 07 — Ciclo de Vida das Ferramentas

## Objetivo

O Kernwerk Studio deve detectar, configurar, iniciar, monitorar e parar ferramentas externas de forma previsível.

## Ferramentas iniciais

```text
Rust: cargo, rustc, rustfmt, clippy
C++: clangd, clang++, cmake, ninja, clang-format, clang-tidy, gdb/lldb
Java: java, javac, jdtls, maven, gradle
Python: uv, python, pyright/basedpyright, ruff, pytest
Git: git
Busca: ripgrep, fd
IA: gpt cli, claude cli, ollama
```

## Componentes do core

```text
ToolManager
├── ToolDetector
├── ToolRegistry
├── ToolHealthChecker
├── ProcessManager
├── LogCollector
└── ToolSuggestionProvider
```

## Estados de ferramenta

```text
NotConfigured
Missing
Detected
Ready
Running
Failed
Disabled
```

## Exemplo de erro amigável

```text
clangd não encontrado.

O Kernwerk Studio usa clangd para recursos inteligentes de C++.

Ações:
[Instalar via pacman] [Selecionar binário] [Desativar C++ inteligente]
```

## Linux/CachyOS sugestões

Comandos sugeridos pela IDE quando aplicável:

```bash
sudo pacman -S rustup cmake ninja clang lldb gdb qt6-base qt6-tools git ripgrep fd
```

O core nunca deve executar instalação automaticamente sem confirmação explícita.

## Logs

Logs devem ir para:

```text
~/.cache/kernwerk-studio/logs/
```

Configurações para:

```text
~/.config/kernwerk-studio/
```

Dados persistentes:

```text
~/.local/share/kernwerk-studio/
```

## Regras

1. UI não chama ferramenta externa diretamente.
2. Core gerencia processos.
3. Toda ferramenta tem status.
4. Toda falha tem mensagem humana.
5. Toda sugestão de instalação deve mostrar comando antes de executar.
6. Logs técnicos devem ser acessíveis no painel Diagnostics.

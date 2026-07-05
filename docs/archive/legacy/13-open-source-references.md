# 13 — Atenção: referências open source consolidadas

Este documento existe para manter o Kernwerk Studio alinhado com projetos
abertos, maduros e auditáveis. A regra é estudar arquitetura, contratos,
organização e integração de ferramentas; não copiar identidade visual,
assets proprietários ou trechos incompatíveis de licença.

## Regra principal

Antes de criar uma solução própria, verificar se uma ferramenta aberta,
madura e gratuita já resolve o problema.

O Kernwerk Studio deve orquestrar ferramentas consolidadas. Ele não deve
reimplementar compiladores, debugadores, language servers, formatadores,
linters ou build systems.

## Referências prioritárias

### Rust, LSP e arquitetura de core

- rust-analyzer: https://github.com/rust-lang/rust-analyzer
  - Referência para LSP, organização em crates, análise incremental e integração
    com `rustc`, `cargo`, `rustfmt` e `clippy`.
- Helix: https://github.com/helix-editor/helix
  - Referência para editor moderno em Rust, integração LSP e arquitetura de
    comandos/estado sem depender de uma UI proprietária.
- Language Server Protocol: https://microsoft.github.io/language-server-protocol/
  - Referência de contrato para recursos inteligentes de linguagem.

### C++/Qt/QML e IDEs visuais

- Qt Creator: https://github.com/qt-creator/qt-creator
  - Referência para IDE Qt/QML, gerenciamento de kits, projetos, debug e UX
    nativa do ecossistema Qt.
- Qt QML: https://doc.qt.io/qt-6/qtqml-index.html
  - Referência oficial para integração QML/C++ e módulos QML.
- Qt com CMake: https://doc.qt.io/qt-6/cmake-get-started.html
  - Referência oficial para estrutura CMake de aplicações Qt 6.
- KDevelop: https://invent.kde.org/kdevelop/kdevelop
  - Referência para IDE KDE/Qt orientada a plugins, CMake e ferramentas externas.
- Kate/KTextEditor: https://invent.kde.org/frameworks/ktexteditor
  - Referência para editor maduro integrado ao ecossistema KDE.

### C++ tooling

- clangd: https://clangd.llvm.org/
  - Referência para recursos inteligentes C++ via LSP.
- LLVM/Clang: https://github.com/llvm/llvm-project
  - Referência para compilador, tooling, clang-format, clang-tidy e sanitizers.
- CMake: https://gitlab.kitware.com/cmake/cmake
  - Referência para build system e presets.
- Ninja: https://github.com/ninja-build/ninja
  - Referência para execução de builds rápida e previsível.

## Modo de segurança máximo

O padrão do projeto é `Strict`. Qualquer modo menos rigoroso precisa de
justificativa registrada no projeto.

### Rust

- `unsafe_code = "forbid"`.
- Warnings quebram build.
- `cargo fmt --all --check`.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
- `cargo test --workspace --all-features`.
- Sem `unwrap`, `expect`, `panic`, `dbg!` ou `todo!` fora de testes.
- Dependências novas precisam de justificativa técnica.

### C++/Qt/QML

- C++23 obrigatório.
- CMakePresets e Ninja como caminho padrão.
- `CMAKE_EXPORT_COMPILE_COMMANDS=ON`.
- `clang-format` obrigatório.
- `clang-tidy` com warnings como erro.
- Warnings do compilador como erro.
- Sanitizers em Debug: AddressSanitizer e UndefinedBehaviorSanitizer.
- Release com LTO e hardening.
- QML validado por tooling Qt antes de release.

## Decisões proibidas por padrão

- Parser próprio quando `clangd`, `jdtls`, `pyright` ou outro LSP maduro atende.
- Debugador próprio quando GDB/LLDB atendem.
- Build system próprio quando CMake/Ninja, Maven/Gradle ou uv atendem.
- Formato de config sem schema.
- UI chamando ferramenta externa diretamente.
- Core Rust dependendo de Qt.
- Envio de código do usuário para IA externa sem confirmação explícita.

## Como usar estas referências

1. Abrir issue/tarefa com o problema real.
2. Verificar se uma referência acima já resolve ou orienta a solução.
3. Preferir integração por processo, IPC, LSP, JSON-RPC, arquivo de configuração
   ou schema.
4. Criar código próprio apenas na camada que orquestra, valida, registra estado
   ou apresenta resultado.
5. Registrar no documento técnico quando uma solução própria for inevitável.

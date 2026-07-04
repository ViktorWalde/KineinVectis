# UI Qt/QML

Este diretório é o processo visual do Kernwerk Studio.

A UI sobe o `kernwerk-core` como processo filho e conversa por JSON-RPC
line-delimited via stdin/stdout. Nenhuma lógica de negócio mora aqui.

## Estrutura

```text
ui/
├── CMakeLists.txt        # alvo kernwerk-studio (qt_add_qml_module)
├── src/
│   ├── main.cpp          # entrypoint QGuiApplication
│   ├── core_client.h     # CoreClient: QProcess + JSON-RPC (exposto ao QML)
│   ├── core_client.cpp
│   ├── editor_highlighter.h
│   └── editor_highlighter.cpp
├── qml/
│   ├── FolderPickerDialog.qml
│   ├── Main.qml          # janela principal: workspace, explorer, editor e paineis
│   └── Theme.qml         # singleton com a paleta de docs/05-design-system.md
└── assets/
    └── app-icon.png      # ícone KW (copiado de imagens/app-icon.png)
```

## Build

A partir da raiz do repositório:

```bash
cargo build -p kernwerk-core          # binário que a UI executa
cmake --preset dev-local
cmake --build --preset dev-local
./build/linux-clang-debug-strict/ui/kernwerk-studio
```

A UI procura o `kernwerk-core` nesta ordem: variável `KERNWERK_CORE_BIN`,
diretório do executável, `target/debug/kernwerk-core` relativo ao diretório
atual, e por fim o `PATH`.

## Dependências

Arch/CachyOS:

```bash
sudo pacman -S cmake ninja clang qt6-base qt6-declarative qt6-tools
```

Debian/Ubuntu/Pop!_OS:

```bash
sudo apt install cmake ninja-build clang qt6-base-dev qt6-declarative-dev \
    qml6-module-qtquick qml6-module-qtquick-window
```

## Regras permanentes

- UI Qt/QML não chama `cmake`, `git`, `clangd`, `gdb`, `lldb` ou IA externa diretamente.
- Toda ação visual deve virar comando enviado ao `kernwerk-core` por IPC.
- QML fica responsável por apresentação e interação; lógica de negócio fica no core Rust.
- O seletor próprio de workspace usa `workspace.browse`; não voltar para dialog nativo.
- Criação de pasta/projeto no seletor usa `workspace.createFolder` e
  `workspace.createProject`; a UI não cria diretórios diretamente.
- Diagnósticos de build/LSP devem passar pelos eventos do core e pela aba Problemas existente.
- Navegação semântica (`lsp.definition`, `lsp.hover`) deve passar por `CoreClient`
  e pelo `LspManager`; a UI não fala LSP diretamente.
- C++/Qt usa CMake, Ninja, C++23, clang-format, clang-tidy e as opções de `cmake/KernwerkStrictOptions.cmake`.
- Debug nativo usa sanitizers; release nativo usa hardening e LTO.

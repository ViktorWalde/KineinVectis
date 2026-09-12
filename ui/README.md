# UI Qt/QML

Este diretório é o processo visual do Kinein Vectis.

A UI sobe o `kinein-core` como processo filho e conversa por JSON-RPC
line-delimited via stdin/stdout. Nenhuma lógica de negócio mora aqui.

## Estrutura

```text
ui/
├── CMakeLists.txt        # alvo kinein-vectis (qt_add_qml_module)
├── src/
│   ├── main.cpp          # entrypoint QGuiApplication
│   ├── core_client.h     # CoreClient: QProcess + JSON-RPC (exposto ao QML)
│   ├── core_client.cpp
│   ├── editor_highlighter.h
│   ├── editor_highlighter.cpp
│   ├── documentation.h  # singleton somente leitura para recursos da UI
│   └── documentation.cpp
├── qml/
│   ├── Main.qml          # composição da janela e dos hosts
│   ├── Theme.qml         # singleton com a paleta de DocsPublic/05-design-system.md
│   ├── shell/            # cabeçalho, layout, overlays e visualizador do manual
│   ├── workspace/        # seletor, tela inicial e saúde do projeto
│   ├── editor/           # superfície, controladores e popups do editor
│   └── panels/           # janelas de ferramentas inferiores
└── assets/
    └── app-icon.png      # ícone KW (copiado de imagens/app-icon.png)
```

## Build

A partir da raiz do repositório:

```bash
cargo build -p kinein-core          # binário que a UI executa
cmake --preset dev-local
cmake --build --preset dev-local
./build/linux-clang-debug-strict/ui/kinein-vectis
```

A UI procura o `kinein-core` nesta ordem: variável `KINEIN_CORE_BIN`,
diretório do executável, `target/debug/kinein-core` relativo ao diretório
atual, e por fim o `PATH`.

O `DocsPublic/manual.md` da raiz é empacotado como recurso somente leitura e exibido em
Markdown por **Ajuda > Manual da IDE**; isso não acessa arquivos do workspace
nem introduz lógica de negócio na UI.

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
- Toda ação visual deve virar comando enviado ao `kinein-core` por IPC.
- QML fica responsável por apresentação e interação; lógica de negócio fica no core Rust.
- O seletor próprio de workspace usa `workspace.browse`; não voltar para dialog nativo.
- Criação de pasta/projeto no seletor usa `workspace.createFolder` e
  `workspace.createProject`; a UI não cria diretórios diretamente.
- Diagnósticos de build/LSP devem passar pelos eventos do core e pela aba Problemas existente.
- Navegação semântica (`lsp.definition`, `lsp.hover`) deve passar por `CoreClient`
  e pelo `LspManager`; a UI não fala LSP diretamente.
- C++/Qt usa CMake, Ninja, C++23, clang-format, clang-tidy e as opções de `cmake/KineinStrictOptions.cmake`.
- Debug nativo usa sanitizers; release nativo usa hardening e LTO.

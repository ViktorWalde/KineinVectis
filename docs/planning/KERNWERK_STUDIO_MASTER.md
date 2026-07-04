# README.md

# Kernwerk Studio

**Kernwerk Studio** é uma IDE open source, Linux-first, rígida por padrão e visualmente plug and play, criada para desenvolvimento moderno em C++, Java, Python, backend convencional e sistemas embarcados.

A arquitetura do projeto separa claramente:

- **Frontend visual:** Qt/QML.
- **Core/backend:** Rust.
- **Ferramentas externas:** clangd, jdtls, pyright, CMake, Ninja, Maven, Gradle, uv, Git, GDB, LLDB, QEMU, OpenOCD, Ollama/GPT/Claude CLI.

O Kernwerk Studio não tenta reimplementar compiladores, parsers, debugadores ou language servers. Ele atua como uma camada visual profissional, rigorosa e integrada sobre ferramentas open source consolidadas.

## Objetivo

Criar uma IDE com experiência visual familiar para usuários acostumados ao ecossistema JetBrains, mas com filosofia:

- open source;
- sem telemetria obrigatória;
- Linux-first;
- alta performance;
- baixo consumo de memória;
- qualidade máxima por padrão;
- configuração visual em vez de configuração manual excessiva;
- suporte profissional a C++, Java, Python, backend e embarcados;
- integração opcional com IA local e/ou externa.

## Nome

- Nome do produto: **Kernwerk Studio**
- Diretório do projeto: `kernwerk-studio`
- Binário principal futuro: `kernwerk-studio`
- Core Rust/daemon: `kernwerk-core`
- Nome interno de crate no código Rust: `kernwerk_core`

## Decisão técnica inicial

O projeto deve começar como **Rust workspace** no CLion.

Motivo:

1. O core/backend é o cérebro da IDE.
2. A UI Qt/QML pode ser adicionada depois como subprojeto `ui/`.
3. O core Rust pode ser testado desde o primeiro dia sem interface gráfica.
4. O protocolo IPC entre UI e core pode ser definido antes da interface definitiva.
5. A arquitetura evita um monólito C++/Qt difícil de manter.

A UI Qt/QML será adicionada posteriormente como processo separado ou como subprojeto CMake que se comunica com o core Rust via IPC local.

## Primeira meta técnica

Antes de editor, LSP, Git ou CMake, o primeiro marco técnico é:

```text
Qt/QML UI ou cliente CLI
        ↓
IPC local
        ↓
Rust Core
        ↓
Resposta: core.pong
```

O MVP inicial pode começar até sem Qt: primeiro um `kernwerk-core` e um `kernwerk-cli` para validar protocolo, comandos, settings, logs e strict mode.


# AGENTS.md

# AGENTS.md — Instruções para GPT/Claude no terminal

Este arquivo orienta agentes de IA trabalhando no repositório **Kernwerk Studio**.

## Identidade do projeto

Kernwerk Studio é uma IDE open source, Linux-first, rígida por padrão, visualmente plug and play e orientada a performance.

Arquitetura principal:

```text
Qt/QML Frontend  ← IPC/JSON-RPC local →  Rust Core
```

O projeto deve evitar reimplementar ferramentas já existentes. Sempre que possível, integrar ferramentas consolidadas:

- C++: clangd, CMake, Ninja, clang-format, clang-tidy, GDB/LLDB.
- Java: JDK 25 LTS, jdtls, Maven, Gradle, JUnit, Checkstyle, SpotBugs, PMD.
- Python: uv, venv, Pyright/basedpyright, Ruff, pytest, mypy opcional.
- Embedded: CMake toolchains, QEMU, OpenOCD, pyOCD, GDB remote, serial monitor, Yocto/Buildroot SDKs.
- Backend: Docker/Podman Compose, HTTP client, OpenAPI, PostgreSQL, Redis, logs.
- IA: GPT CLI, Claude CLI, Ollama, OpenAI/Anthropic/OpenRouter via providers configuráveis.

## Regra principal

Não criar soluções improvisadas quando existir ferramenta aberta, madura e gratuita que resolva o problema.

A IDE deve orquestrar ferramentas, não substituir compiladores, servidores LSP ou debugadores.

## Rigor obrigatório

Todo código Rust deve seguir o máximo rigor possível:

- `unsafe` proibido por padrão.
- Warnings devem quebrar build.
- `cargo fmt --check` obrigatório.
- `cargo clippy --all-targets --all-features -- -D warnings` obrigatório.
- Testes devem ser criados para módulos de core.
- Erros devem usar tipos explícitos e contexto.
- Evitar `unwrap`, `expect` e panics fora de testes.
- Preferir APIs pequenas, testáveis e desacopladas.
- Evitar acoplamento entre UI e core.
- Não bloquear UI com trabalho pesado.
- Não adicionar dependências sem justificativa.

## Como agir ao implementar

Antes de criar código:

1. Ler `docs/00-product-vision.md`.
2. Ler `docs/01-architecture.md`.
3. Ler `docs/02-repository-structure.md`.
4. Ler `docs/03-ipc-protocol.md`.
5. Ler `docs/06-strict-mode.md`.
6. Verificar se a tarefa pertence ao core, UI, tooling, docs ou protocolo.

Ao propor implementação:

- explicar arquivos que serão criados/alterados;
- manter escopo pequeno;
- não misturar muitas camadas;
- escrever testes quando aplicável;
- atualizar docs se mudar contrato ou arquitetura.

## Padrão de commits sugerido

Usar Conventional Commits:

```text
feat: adiciona protocolo inicial de IPC
fix: corrige serialização de eventos do core
docs: documenta strict mode
refactor: separa workspace service do command registry
test: cobre validação de comandos
chore: atualiza configuração de lint
```

## Nomes

- Produto: `Kernwerk Studio`
- Repositório/pasta: `kernwerk-studio`
- Core daemon/package: `kernwerk-core`
- Crate importável: `kernwerk_core`
- CLI futura: `kernwerk-cli`
- Protocolo: `kernwerk-protocol`
- Configuração: `kernwerk-config`

## Não fazer

- Não transformar o core Rust em código dependente de Qt.
- Não colocar lógica de negócio na UI.
- Não chamar ferramentas externas diretamente da UI.
- Não criar formato de configuração sem schema/documentação.
- Não adicionar telemetria.
- Não enviar código do usuário para IA externa sem confirmação explícita.
- Não relaxar strict mode sem registrar motivo.


# docs/00-product-vision.md

# 00 — Visão do Produto

## Nome

**Kernwerk Studio**

## Frase curta

Uma IDE open source, Linux-first, rígida por padrão e visualmente plug and play para engenharia de software moderna.

## Público inicial

O primeiro usuário é o próprio autor do projeto:

- usa Linux/CachyOS/KDE;
- gosta da filosofia open source;
- está acostumado ao ecossistema JetBrains;
- quer conforto visual para longas sessões;
- quer memória muscular preservada;
- trabalha/estuda C++, Java, Python, backend, integração OT/TI, Linux embarcado e robótica/simulação;
- prefere uma IDE completa e visual, com menos configuração manual do que VS Code.

## Filosofia

Kernwerk Studio deve ser:

- familiar como JetBrains;
- modular como VS Code;
- rápido como editores modernos;
- auditável como software open source;
- rígido como CI profissional;
- confortável para uso diário;
- visualmente plug and play;
- sem telemetria obrigatória;
- IA opcional e controlada.

## O que a IDE não é

Kernwerk Studio não é:

- um compilador;
- um parser universal;
- um novo debugger;
- um novo build system;
- uma cópia de código ou assets da JetBrains;
- um clone literal de identidade visual proprietária.

Ela pode ser visualmente familiar, mas deve ter identidade própria.

## Pilares

1. **Visual plug and play**
   - Novo projeto deve gerar estrutura, build, lint, format, testes e docs.
   - Configurações complexas devem ter UI.
   - Linhas de comando devem existir, mas não serem obrigatórias para o fluxo principal.

2. **Strict by default**
   - Projetos novos começam no modo mais rígido.
   - Warnings devem virar erros quando apropriado.
   - Sanitizers, linters, formatadores e testes devem estar integrados.
   - Relaxar regras deve ser escolha explícita.

3. **Linux-first**
   - CachyOS/Arch como ambiente principal inicial.
   - Boa integração com KDE/Wayland.
   - Configs em diretórios Linux padrão.
   - Empacotamento futuro via PKGBUILD/AUR, AppImage e Flatpak.

4. **Core em Rust**
   - Segurança, concorrência e robustez.
   - Processo separado da UI.
   - Testável sem interface.
   - Sem `unsafe` por padrão.

5. **UI Qt/QML**
   - Interface moderna e eficiente.
   - Painéis previsíveis.
   - Memória muscular inspirada em JetBrains.
   - Tema escuro confortável por padrão.

6. **Ferramentas existentes**
   - clangd, jdtls, pyright, CMake, Ninja, Gradle, Maven, uv, Git, GDB/LLDB etc.
   - A IDE integra, organiza e visualiza essas ferramentas.


# docs/01-architecture.md

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


# docs/02-repository-structure.md

# 02 — Estrutura do Repositório

## Estrutura inicial recomendada

```text
kernwerk-studio/
├── README.md
├── AGENTS.md
├── Cargo.toml
├── rust-toolchain.toml
├── deny.toml
├── .gitignore
├── .editorconfig
│
├── crates/
│   ├── kernwerk-core/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── main.rs
│   │
│   ├── kernwerk-protocol/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs
│   │
│   ├── kernwerk-config/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs
│   │
│   └── kernwerk-cli/
│       ├── Cargo.toml
│       └── src/
│           └── main.rs
│
├── ui/
│   ├── README.md
│   └── placeholder.md
│
├── schemas/
│   ├── ipc.schema.json
│   ├── settings.schema.json
│   └── project.schema.json
│
├── templates/
│   ├── cpp-console-strict/
│   ├── cpp-qt-qml-strict/
│   ├── java-backend-strict/
│   ├── python-backend-strict/
│   └── embedded-linux-strict/
│
├── docs/
│   ├── 00-product-vision.md
│   ├── 01-architecture.md
│   ├── 02-repository-structure.md
│   ├── 03-ipc-protocol.md
│   ├── 04-command-system.md
│   ├── 05-design-system.md
│   ├── 06-strict-mode.md
│   ├── 07-tooling-lifecycle.md
│   ├── 08-performance-budget.md
│   ├── 09-roadmap.md
│   └── 10-mvp-plan.md
│
└── prompts/
    └── GPT_TERMINAL_BOOTSTRAP.md
```

## Por que começar com Rust workspace

O core é o cérebro do projeto. Começar com Rust evita que a lógica da IDE fique presa ao Qt/C++ cedo demais.

A UI pode ser adicionada depois em `ui/` como:

```text
ui/
├── CMakeLists.txt
├── src/
└── qml/
```

ou integrada via CXX-Qt futuramente, se fizer sentido.

## Nomes de packages e crates

Usar hífen no nome do pacote:

```toml
[package]
name = "kernwerk-core"
```

No código Rust, o crate será referenciado como:

```rust
use kernwerk_core::...
```

Convenção prática:

- diretórios/packages: `kebab-case`;
- crates/imports/módulos Rust: `snake_case`;
- tipos Rust: `PascalCase`;
- funções: `snake_case`.

## Arquivo Cargo.toml raiz sugerido

```toml
[workspace]
resolver = "2"
members = [
    "crates/kernwerk-core",
    "crates/kernwerk-protocol",
    "crates/kernwerk-config",
    "crates/kernwerk-cli",
]

[workspace.package]
edition = "2024"
license = "MIT OR Apache-2.0"
repository = "https://github.com/SEU_USUARIO/kernwerk-studio"
homepage = "https://github.com/SEU_USUARIO/kernwerk-studio"
readme = "README.md"
rust-version = "1.85"

[workspace.lints.rust]
unsafe_code = "forbid"
warnings = "deny"
missing_docs = "warn"

[workspace.lints.clippy]
all = "deny"
pedantic = "deny"
nursery = "deny"
cargo = "warn"
unwrap_used = "deny"
expect_used = "deny"
panic = "deny"
todo = "deny"
dbg_macro = "deny"
print_stdout = "warn"
print_stderr = "warn"
module_name_repetitions = "allow"
missing_errors_doc = "allow"
missing_panics_doc = "allow"

[profile.dev]
debug = true
incremental = true

[profile.release]
lto = "fat"
codegen-units = 1
strip = true
panic = "abort"
```

A versão exata de `rust-version` deve ser ajustada ao toolchain instalado.


# docs/03-ipc-protocol.md

# 03 — Protocolo IPC

## Objetivo

O protocolo IPC permite comunicação entre:

```text
Kernwerk UI  ←→  Kernwerk Core
```

A UI deve mandar comandos e receber respostas/eventos. O core deve executar lógica, chamar ferramentas externas e emitir eventos de estado.

## Transporte inicial

Para o MVP:

```text
stdin/stdout JSON-RPC
```

Depois:

```text
Unix domain socket
```

Futuro:

```text
gRPC/local socket
```

## Formato base

### Request

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "core.ping",
  "params": {}
}
```

### Response

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "status": "ok",
    "message": "pong"
  }
}
```

### Error

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": "TOOL_NOT_FOUND",
    "message": "clangd não foi encontrado",
    "details": {
      "tool": "clangd",
      "suggestedCommand": "sudo pacman -S clang"
    }
  }
}
```

### Event

```json
{
  "jsonrpc": "2.0",
  "method": "event.diagnostics.updated",
  "params": {
    "uri": "file:///home/vitor/dev/projeto/src/main.cpp",
    "errors": 1,
    "warnings": 0
  }
}
```

## Métodos iniciais

```text
core.ping
core.shutdown
workspace.open
workspace.close
workspace.status
settings.get
settings.set
command.list
command.execute
tools.detect
tools.status
build.configure
build.run
logs.tail
```

## Eventos iniciais

```text
event.core.ready
event.workspace.opened
event.workspace.closed
event.tool.statusChanged
event.build.started
event.build.output
event.build.finished
event.diagnostics.updated
event.notification.created
event.log.appended
```

## Regras

1. Todo request deve ter resposta.
2. Eventos não têm `id`.
3. Erros devem ter `code`, `message` e `details`.
4. O protocolo deve ser versionado.
5. Toda alteração no protocolo exige atualização de docs e schemas.
6. A UI não deve interpretar logs brutos quando houver evento estruturado.


# docs/04-command-system.md

# 04 — Sistema de Comandos

## Ideia central

Tudo na IDE deve ser um comando.

Menus, atalhos, command palette, botões, ações de IA e plugins devem chamar o mesmo sistema de comandos.

## Exemplos de comandos

```text
core.ping
workspace.open
workspace.reload
project.create
project.index
editor.openFile
editor.saveFile
editor.formatFile
editor.quickFix
symbol.rename
build.configure
build.run
build.clean
debug.start
debug.stop
git.status
git.commit
git.stageFile
ai.explainSelection
ai.reviewDiff
tools.detect
lsp.restart
settings.open
```

## Estrutura de comando

```json
{
  "id": "build.run",
  "title": "Executar build",
  "category": "Build",
  "description": "Executa o build usando a configuração ativa",
  "defaultShortcut": "Ctrl+F9",
  "requiresWorkspace": true
}
```

## Benefícios

- Menus ficam simples.
- Atalhos ficam consistentes.
- Command palette fica poderosa.
- Plugins podem registrar comandos.
- IA pode sugerir comandos com segurança.
- Logs podem registrar ações de forma rastreável.

## Command Palette

Atalhos-alvo:

```text
Shift Shift       Search Everywhere
Ctrl+Shift+A     Find Action
Ctrl+Shift+P     Command Palette alternativa
```

## Presets de atalhos

Começar com:

```text
JetBrains Compatible
```

Futuro:

```text
Kernwerk Default
VS Code Compatible
Vim Mode
```

## Regras

1. Não criar botão que não chame comando.
2. Não criar atalho direto para função interna.
3. Comandos devem validar contexto.
4. Comandos devem retornar resultado estruturado.
5. Comandos longos devem emitir eventos de progresso.


# docs/05-design-system.md

# 05 — Design System

## Identidade

Nome: **Kernwerk Studio**

Símbolo: **KW**

Estilo: técnico, escuro, minimalista, Linux-first, inspirado em estética de distros Arch Linux, porém com identidade própria.

## Paleta principal

```text
Accent escuro:    #6e5c01
Texto claro:      #eae6e1
Accent forte:     #ffbb00
Neutro oliva:     #6e6c58
```

## Paleta auxiliar sugerida

```text
Background 0:     #0d0e0e
Background 1:     #121313
Background 2:     #191a18
Surface 1:        #1f201d
Surface 2:        #25261f
Border soft:      #2a2922
Text primary:     #eae6e1
Text secondary:   #b9b3a5
Text muted:       #8f8a7c
Accent:           #ffbb00
Accent dim:       #6e5c01
Neutral olive:    #6e6c58
Error soft:       #d16d6d
Warning soft:     #ffbb00
Success soft:     #7fbf7f
Info soft:        #7aa2d8
```

## Tema padrão

O tema padrão é escuro, confortável para longas horas.

Princípios:

- baixo brilho geral;
- contraste suficiente sem agredir os olhos;
- amarelo usado como destaque, não como cor dominante;
- painéis com bordas sutis;
- código no centro com distrações reduzidas;
- elementos previsíveis para memória muscular.

## Densidade

Padrão:

```text
Confortável
```

Opções futuras:

```text
Compacta
Confortável
Espaçosa
```

## Tipografia

Sugestões:

- Editor: JetBrains Mono, Fira Code, Cascadia Code ou monospace do sistema.
- UI: Inter, Noto Sans, ou fonte do sistema KDE.
- Terminal: JetBrains Mono ou monospace do sistema.

## Layout principal

```text
┌──────────────────────────────────────────────────────────────┐
│ Top Bar: Projeto | Branch | Run Config | Build | Debug | IA  │
├──────┬───────────────────────────────┬───────────────────────┤
│ Left │ Editor Tabs                   │ Right Tool Window     │
│ Bar  │                               │ AI / Docs / Inspector │
│      │ Código                        │                       │
├──────┴───────────────────────────────┴───────────────────────┤
│ Bottom Tool Window: Terminal | Problems | Build | Git | Debug│
├──────────────────────────────────────────────────────────────┤
│ Status Bar: Git | CMake | clangd | Java | Python | Encoding  │
└──────────────────────────────────────────────────────────────┘
```

## Memória muscular

Atalhos e localização de painéis devem ser familiares para usuários JetBrains:

```text
Alt+1        Project
Alt+4        Run/Build
Alt+5        Debug
Alt+9        Git
Alt+Enter    Quick Fix
Shift+F6     Rename
Ctrl+B       Go to Definition
Ctrl+Alt+L   Format Code
Shift Shift  Search Everywhere
```

## Regra visual

Não mover automaticamente painéis de lugar. A IDE pode sugerir, mas nunca reorganizar o workspace sem ação explícita do usuário.


# docs/06-strict-mode.md

# 06 — Strict Mode

## Filosofia

Kernwerk Studio é rígido por padrão.

Projetos novos devem nascer com qualidade profissional, e o usuário pode relaxar regras depois se quiser.

## Regra geral

```text
Strict primeiro.
Relaxado apenas por decisão explícita.
```

## Rust — core do Kernwerk

Regras:

- `unsafe_code = "forbid"`.
- warnings como erro.
- clippy agressivo.
- sem `unwrap`, `expect`, `panic` fora de testes.
- `cargo fmt --check`.
- `cargo test`.
- `cargo clippy --all-targets --all-features -- -D warnings`.
- auditoria de dependências com `cargo-deny`/`cargo-audit` futuramente.

Comandos:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo check --workspace --all-targets --all-features
```

## C++ Strict

Padrão para projetos C++ gerados:

- C++23;
- CMakePresets;
- Ninja;
- `CMAKE_EXPORT_COMPILE_COMMANDS=ON`;
- clangd;
- clang-format;
- clang-tidy;
- warnings as errors;
- sanitizers em Debug;
- LTO em Release Hardened;
- CTest;
- Catch2 ou GoogleTest.

Flags sugeridas para Clang/GCC:

```text
-Wall
-Wextra
-Wpedantic
-Werror
-Wconversion
-Wsign-conversion
-Wshadow
-Wnon-virtual-dtor
-Wold-style-cast
-Wcast-align
-Wunused
-Woverloaded-virtual
-Wnull-dereference
-Wdouble-promotion
-Wformat=2
-Wimplicit-fallthrough
-Wundef
```

Sanitizers Debug:

```text
-fsanitize=address,undefined
-fno-omit-frame-pointer
```

## Java Strict

Padrão para projetos Java:

- Java 25 LTS como alvo desejado do usuário;
- Maven ou Gradle;
- JUnit;
- Checkstyle;
- SpotBugs;
- PMD;
- Error Prone opcional;
- logs estruturados;
- profile dev/prod;
- testes por padrão.

## Python Strict

Padrão para projetos Python:

- uv;
- `.venv` obrigatório;
- Ruff lint;
- Ruff format;
- Pyright/basedpyright strict;
- pytest;
- mypy opcional;
- `.env.example`;
- estrutura clara para FastAPI/Django/CLI.

## Backend Strict

Padrão:

- healthcheck;
- logs estruturados;
- `.env.example`;
- Docker/Podman Compose opcional;
- OpenAPI quando aplicável;
- testes de endpoint;
- configuração dev/prod separada.

## Embedded Strict

Padrão:

- target explícito;
- toolchain explícita;
- sysroot explícito quando aplicável;
- CMake toolchain file;
- debug profile;
- flash profile;
- serial monitor;
- QEMU quando aplicável;
- logs de build e deploy.

## Níveis futuros

```text
Strict       padrão
Balanced     menos agressivo
Relaxed      para legado
```

Mesmo que o autor use apenas Strict, os níveis devem existir no modelo para futura distribuição.


# docs/07-tooling-lifecycle.md

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


# docs/08-performance-budget.md

# 08 — Performance Budget

## Objetivo

Kernwerk Studio deve ser rápido e leve por padrão.

## Metas iniciais

```text
Abrir janela inicial:          < 1s
Iniciar core:                  < 500ms
Ping UI ↔ Core:                < 50ms
Abrir projeto pequeno:         < 2s
UI travada por trabalho pesado: nunca
Uso de RAM em projeto pequeno: < 300 MB como meta aspiracional
```

## Princípios

1. UI nunca executa trabalho pesado.
2. Builds, LSPs, IA e busca rodam fora da thread de UI.
3. Core deve emitir eventos incrementais.
4. Logs não devem bloquear comandos.
5. File watching deve ser eficiente.
6. Cache deve acelerar abertura sem esconder erros.
7. Evitar dependências pesadas no core.
8. Carregamento preguiçoso de módulos.
9. Painéis só carregam dados quando abertos, quando possível.
10. IA nunca bloqueia editor.

## Métricas a coletar localmente

Sem telemetria externa. Apenas diagnóstico local:

```text
tempo de start
tempo de abertura de workspace
tempo de resposta IPC
número de processos externos
memória do core
memória da UI
tempo de build
tempo de indexação
```

## Painel Diagnostics

O painel deve mostrar:

```text
Core: running
UI: connected
IPC latency: 12ms
clangd: running
cmake: ready
git: ready
memory core: 42 MB
memory ui: 180 MB
```

## Regra

O projeto deve falhar em desenvolvimento se uma mudança introduzir bloqueio óbvio na UI ou acoplamento indevido entre UI e ferramenta externa.


# docs/09-roadmap.md

# 09 — Roadmap

## Fase 0 — Fundamento Rust

Objetivo: criar base robusta antes da UI.

- Rust workspace.
- Crates iniciais:
  - `kernwerk-core`
  - `kernwerk-protocol`
  - `kernwerk-config`
  - `kernwerk-cli`
- Strict mode do próprio projeto.
- Logs.
- Settings básicos.
- IPC simulado ou stdin/stdout.
- Comando `core.ping`.
- Testes iniciais.

## Fase 1 — Core + CLI

Objetivo: validar arquitetura sem interface gráfica.

- `kernwerk-core` roda como processo.
- `kernwerk-cli ping`.
- `kernwerk-cli tools detect`.
- `kernwerk-cli workspace open <path>`.
- JSON-RPC básico.
- Schemas iniciais.

## Fase 2 — UI Qt/QML mínima

Objetivo: janela conecta no core.

- Janela escura.
- Splash/local status.
- IPC client.
- Status: Core conectado.
- Painel de logs.
- Settings básico.

## Fase 3 — Workspace e Explorer

- Abrir pasta/projeto.
- Explorer visual.
- Abas.
- Abrir/salvar arquivo.
- Estado de layout.
- Arquivos recentes.

## Fase 4 — Build CMake

- Detectar CMake.
- Detectar CMakePresets.
- Configure.
- Build.
- Run.
- Painel Build.
- Erros estruturados.

## Fase 5 — LSP C++

- Iniciar clangd.
- Enviar abertura de documento.
- Diagnostics.
- Hover.
- Go to definition.
- Completion simples.
- Quick fixes.

## Fase 6 — Git

- Status.
- Diff.
- Stage/unstage.
- Commit.
- Branch atual.
- Histórico básico.

## Fase 7 — IA

- Painel Assistente KW.
- Provider local/Ollama.
- GPT CLI/Claude CLI.
- Política de contexto.
- Explicar erro de build.
- Gerar commit message.
- Revisar diff.

## Fase 8 — Java

- JDK detection.
- Maven/Gradle.
- jdtls.
- Run Spring Boot.
- Testes JUnit.
- Painel Services.

## Fase 9 — Python

- uv/venv.
- Pyright.
- Ruff.
- pytest.
- FastAPI/Django templates.
- Painel Services.

## Fase 10 — Embedded

- Toolchain profiles.
- CMake toolchain files.
- QEMU.
- Serial monitor.
- GDB remote.
- Flash profiles.
- Yocto/Buildroot SDK support.

## Regra do roadmap

Não avançar para features avançadas antes de validar o contrato UI ↔ Core e o sistema de comandos.


# docs/10-mvp-plan.md

# 10 — Plano de MVP

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


# docs/11-layout-interactions.md

# 11 — Layout e Interações

## Layout principal

```text
┌──────────────────────────────────────────────────────────────┐
│ KW | Projeto | Branch | Run Config | Build | Run | Debug | IA │
├────┬───────────────────────────────┬─────────────────────────┤
│    │ Tabs                          │ Assistente / Inspector  │
│Bar │ Editor                        │ Docs / Outline          │
│    │                               │                         │
├────┴───────────────────────────────┴─────────────────────────┤
│ Terminal | Problems | Build | Git | Debug | Tests             │
├──────────────────────────────────────────────────────────────┤
│ Status: Git | Toolchain | LSP | Build | Encoding | Line/Col   │
└──────────────────────────────────────────────────────────────┘
```

## Sidebar esquerda

Ícones fixos:

```text
Projeto
Buscar
Git
Run
Debug
Services
Embedded
IA
Settings
```

Regra:

- clique abre/recolhe painel;
- botão direito abre opções;
- posição não muda automaticamente.

## Top Bar — Projeto

Menu:

```text
Abrir projeto
Abrir recente
Novo projeto
Fechar projeto
Adicionar projeto ao workspace
Configurações do workspace
Reindexar workspace
Executar verificação completa
```

## Top Bar — Branch

Menu:

```text
Criar branch
Trocar branch
Fetch
Pull
Push
Merge
Rebase
Abrir painel Git
```

## Top Bar — Run Config

Menu:

```text
Debug Strict
Release Strict
Sanitized Debug
Release Hardened
Editar configurações
Nova configuração
Executar testes
CMake Configure
CMake Build
CMake Clean
```

## Painel Project

Modos:

```text
Projeto
Arquivos
Estrutura
Pacotes
CMake Targets
Services
Embedded Targets
```

## Clique direito em arquivo

```text
Abrir
Abrir ao lado
Renomear
Mover
Duplicar
Excluir
Formatar arquivo
Analisar arquivo
Gerar teste
Explicar com IA
Ver alterações Git
Histórico do arquivo
```

## Editor — clique direito

```text
Quick Fix
Refatorar
Renomear símbolo
Extrair função
Extrair variável
Ir para definição
Encontrar usos
Mostrar documentação
Formatar seleção
Explicar seleção com IA
Gerar teste
Revisar performance
```

## Painel inferior

Tabs:

```text
Terminal
Problems
Build
Output
Git
Debug
Tests
```

## Painel direito

Tabs:

```text
Assistente KW
Inspector
Docs
Outline
```

## Settings

Seções:

```text
Geral
Aparência
Editor
Atalhos
Projetos
C++
Java
Python
Embedded
Backend
Git
IA
Terminal
Build
Debug
Plugins
Privacidade
```

## Privacidade padrão

```text
Telemetria: desligada
IA local: permitida
IA externa: perguntar antes de enviar contexto
Nunca enviar projeto inteiro sem confirmação
```


# docs/12-ai-policy.md

# 12 — Política de IA

## Objetivo

Kernwerk Studio deve integrar IA sem quebrar privacidade, controle e filosofia open source.

## Providers previstos

```text
Ollama
GPT CLI
Claude CLI
OpenAI API
Anthropic API
OpenRouter
```

## Modos

```text
Local only
Ask before external context
External allowed for selected files
```

Padrão:

```text
Ask before external context
```

## Regras

1. Nunca enviar projeto inteiro sem confirmação.
2. Mostrar arquivos/trechos que serão usados como contexto.
3. Permitir IA local sem confirmação extra.
4. Permitir desativar IA completamente.
5. Logs de prompts devem ser locais e opcionais.
6. Chaves/API tokens nunca devem ir para logs.
7. IA não executa comando destrutivo sem confirmação.

## Ações de IA

```text
ai.explainFile
ai.explainSelection
ai.reviewDiff
ai.generateCommitMessage
ai.fixBuildError
ai.generateTests
ai.createDocs
ai.suggestRefactor
ai.explainArchitecture
```

## Painel Assistente KW

Deve mostrar:

```text
Contexto atual
Arquivos incluídos
Provider ativo
Política de privacidade
Ações rápidas
Chat
```


# prompts/GPT_TERMINAL_BOOTSTRAP.md

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

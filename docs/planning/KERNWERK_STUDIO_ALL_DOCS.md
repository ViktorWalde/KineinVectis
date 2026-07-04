# FILE: README.md

# Kernwerk Studio — Documentation Pack

Este pacote contém documentação especializada para orientar o desenvolvimento do **Kernwerk Studio**, uma IDE open source, Linux-first, visualmente plug and play, rígida por padrão e focada em performance.

## Nome do projeto

```text
kernwerk-studio
```

## Arquitetura decidida

```text
Qt/QML Frontend  ← IPC/JSON-RPC local →  Rust Core
```

## Estratégia inicial

Começar com **Rust workspace** no CLion.

A UI Qt/QML será adicionada depois como módulo separado em `ui/`.

## Como usar com GPT/Claude no terminal

Antes de pedir implementação, peça para o agente ler:

```text
AGENTS.md
docs/00_INDEX.md
docs/01_PRODUCT_VISION.md
docs/02_ARCHITECTURE.md
docs/03_ENGINEERING_PRACTICES.md
docs/04_STRICT_TOOLCHAINS.md
```

Para tarefas específicas:

- Layout/UI: `docs/05_LAYOUT_AND_UX.md`
- Performance: `docs/06_PERFORMANCE_RULES.md`
- IPC: `docs/07_IPC_PROTOCOL.md`
- Comandos: `docs/08_COMMAND_SYSTEM.md`
- Ferramentas externas: `docs/09_TOOLING_LIFECYCLE.md`
- IA: `docs/10_AI_POLICY.md`
- Roadmap/MVP: `docs/11_ROADMAP_AND_MVP.md`


---

# FILE: AGENTS.md

# AGENTS.md — Regras para agentes de IA

Este repositório é o **Kernwerk Studio**.

Kernwerk Studio é uma IDE open source, Linux-first, visualmente plug and play, rígida por padrão, sem telemetria obrigatória e focada em performance.

## Arquitetura obrigatória

```text
Qt/QML Frontend  ← IPC/JSON-RPC local → Rust Core
```

## Regra central

A UI não executa lógica pesada.  
A UI não chama compiladores, Git, CMake, LSPs ou ferramentas externas diretamente.  
Tudo passa pelo **Rust Core**.

## Antes de implementar qualquer coisa

Leia:

```text
docs/00_INDEX.md
docs/01_PRODUCT_VISION.md
docs/02_ARCHITECTURE.md
docs/03_ENGINEERING_PRACTICES.md
docs/04_STRICT_TOOLCHAINS.md
```

Depois leia o arquivo especializado da tarefa.

## Filosofia

- Não reimplementar compiladores, parsers, debugadores ou language servers.
- Integrar ferramentas open source existentes.
- Garantir previsibilidade visual.
- Priorizar performance.
- Priorizar segurança.
- Priorizar documentação.
- Manter o projeto modular.
- Evitar acoplamento entre UI e core.
- Evitar dependências desnecessárias.

## Rigor Rust obrigatório

- `unsafe` proibido por padrão.
- Warnings quebram build.
- `unwrap`, `expect`, `panic`, `todo!`, `dbg!` proibidos fora de testes.
- `cargo fmt --check` obrigatório.
- `cargo clippy --all-targets --all-features -- -D warnings` obrigatório.
- `cargo test --all-features` obrigatório.
- Erros devem ser explícitos e possuir contexto.
- APIs devem ser pequenas e testáveis.

## Proibido

- Não adicionar telemetria.
- Não enviar contexto de usuário para IA externa sem confirmação.
- Não misturar lógica de negócio na UI.
- Não criar comandos ocultos sem documentação.
- Não criar comportamento mágico que mude layout automaticamente.
- Não adicionar dependência sem justificar.
- Não relaxar strict mode sem registrar motivo.
- Não implementar recursos avançados antes de validar o MVP.

## Convenção de commits

Usar Conventional Commits:

```text
feat: adiciona protocolo inicial de IPC
fix: corrige parsing de settings
docs: detalha strict mode
refactor: separa command registry
test: cobre detector de ferramentas
chore: ajusta clippy
```


---

# FILE: docs/00_INDEX.md

# 00 — Índice da Documentação

Este diretório contém a documentação especializada do **Kernwerk Studio**.

## Ordem recomendada de leitura

```text
01_PRODUCT_VISION.md
02_ARCHITECTURE.md
03_ENGINEERING_PRACTICES.md
04_STRICT_TOOLCHAINS.md
05_LAYOUT_AND_UX.md
06_PERFORMANCE_RULES.md
07_IPC_PROTOCOL.md
08_COMMAND_SYSTEM.md
09_TOOLING_LIFECYCLE.md
10_AI_POLICY.md
11_ROADMAP_AND_MVP.md
```

## Documentos

### `01_PRODUCT_VISION.md`

Define a visão de produto, identidade, público, filosofia e limites do projeto.

### `02_ARCHITECTURE.md`

Define arquitetura geral, camadas, limites entre UI/Core, modularidade e estrutura do repositório.

### `03_ENGINEERING_PRACTICES.md`

Define regras de engenharia de software, boas práticas, padrões de código, testes, documentação, erros e manutenção.

### `04_STRICT_TOOLCHAINS.md`

Define a configuração rígida de compiladores, linters, formatadores, build systems e ferramentas para Rust, C++, Java, Python, backend e embarcados.

### `05_LAYOUT_AND_UX.md`

Define layout, UX, memória muscular, menus, painéis, atalhos, estados vazios, tema escuro e comportamento visual.

### `06_PERFORMANCE_RULES.md`

Define regras de performance, budgets, concorrência, uso de memória, limites de responsabilidade e prevenção de conflitos entre módulos.

### `07_IPC_PROTOCOL.md`

Define o protocolo entre Qt/QML UI e Rust Core.

### `08_COMMAND_SYSTEM.md`

Define o sistema centralizado de comandos da IDE.

### `09_TOOLING_LIFECYCLE.md`

Define como detectar, iniciar, monitorar, reiniciar e parar ferramentas externas.

### `10_AI_POLICY.md`

Define integração com IA local/externa, GPT CLI, Claude CLI, Ollama e política de privacidade.

### `11_ROADMAP_AND_MVP.md`

Define etapas de implementação, MVPs e ordem segura de desenvolvimento.

## Regra para agentes

Ao receber uma tarefa, o agente deve identificar qual documento especializado rege a tarefa antes de alterar código.


---

# FILE: docs/01_PRODUCT_VISION.md

# 01 — Visão do Produto

## Nome

**Kernwerk Studio**

## Diretório do projeto

```text
kernwerk-studio
```

## Descrição curta

Kernwerk Studio é uma IDE open source, Linux-first, visualmente plug and play, rígida por padrão e focada em desenvolvimento moderno de sistemas, backend e embarcados.

## Frase de posicionamento

```text
A strict, open-source, Linux-first IDE for modern engineering.
```

Em português:

```text
Uma IDE open source, Linux-first e rígida por padrão para engenharia de software moderna.
```

## Público inicial

O usuário inicial é o próprio autor do projeto:

- usa CachyOS/KDE;
- prefere software open source;
- valoriza liberdade e controle;
- está acostumado ao ecossistema JetBrains;
- quer conforto visual para longas sessões;
- quer desenvolvimento C++, Java, Python, backend convencional e embarcado;
- quer uma IDE visualmente plug and play;
- quer menos configuração manual do que VS Code;
- quer máximo rigor técnico por padrão;
- pretende usar GPT/Claude no terminal durante o desenvolvimento.

## Filosofia

Kernwerk Studio deve ser:

- familiar como JetBrains;
- modular como VS Code;
- rápido como Zed;
- auditável como KDE/open source;
- rígido como CI profissional;
- confortável como ambiente de trabalho diário;
- visualmente plug and play;
- livre de telemetria obrigatória;
- preparado para IA local e externa com controle explícito.

## O que a IDE deve resolver

A IDE deve reduzir atrito em:

- criação de projetos;
- configuração de CMake;
- configuração de toolchains;
- integração com LSP;
- integração com Git;
- execução de testes;
- build/debug;
- perfis de backend;
- perfis embarcados;
- documentação;
- diagnósticos;
- uso de IA contextual.

## O que a IDE não deve tentar fazer

Kernwerk Studio não deve reimplementar:

- compiladores;
- language servers;
- debugadores;
- build systems;
- Git;
- gerenciadores de pacotes;
- parsers completos de linguagens;
- frameworks de backend.

A IDE deve orquestrar ferramentas existentes.

## Ecossistemas de primeira classe

### C++

- C++23/26 futuramente;
- CMake;
- Ninja;
- clangd;
- clang-format;
- clang-tidy;
- GDB/LLDB;
- CTest;
- Catch2/GoogleTest;
- Qt/QML;
- Linux embarcado.

### Java

- Java 25 LTS como alvo desejado;
- Maven;
- Gradle;
- JDT LS;
- JUnit;
- Spring Boot;
- Checkstyle;
- SpotBugs;
- PMD.

### Python

- uv;
- venv;
- Ruff;
- Pyright/basedpyright;
- pytest;
- FastAPI;
- Django;
- scripts técnicos.

### Backend convencional

- Spring Boot;
- FastAPI;
- Django;
- PostgreSQL;
- Redis;
- Docker/Podman Compose;
- OpenAPI;
- HTTP client;
- logs estruturados.

### Embarcados

- CMake toolchains;
- cross-compilation;
- QEMU;
- OpenOCD;
- pyOCD;
- GDB remote;
- serial monitor;
- Zephyr;
- ESP-IDF;
- STM32;
- Yocto;
- Buildroot;
- ROS 2/Gazebo.

## Identidade visual

Nome: Kernwerk Studio  
Símbolo: KW  
Estilo: técnico, escuro, minimalista, Linux-first, inspirado em estética de distros Arch Linux sem copiar identidade visual de terceiros.

## Paleta

```text
#6e5c01
#eae6e1
#ffbb00
#6e6c58
```


---

# FILE: docs/02_ARCHITECTURE.md

# 02 — Arquitetura

## Decisão principal

O Kernwerk Studio deve começar como **Rust workspace**.

A UI Qt/QML deve ser adicionada depois como módulo separado.

```text
Qt/QML Frontend  ← IPC/JSON-RPC local → Rust Core
```

## Por que Rust primeiro

O Rust Core é o cérebro da IDE:

- workspace;
- configurações;
- comandos;
- ferramentas;
- LSP;
- build;
- Git;
- IA;
- diagnóstico;
- cache;
- políticas de strict mode.

Começar pelo core evita prender a arquitetura à UI.

## Componentes principais

```text
kernwerk-studio/
├── crates/
│   ├── kernwerk-core/
│   ├── kernwerk-protocol/
│   ├── kernwerk-config/
│   ├── kernwerk-command/
│   ├── kernwerk-tooling/
│   ├── kernwerk-workspace/
│   ├── kernwerk-build/
│   ├── kernwerk-lsp/
│   ├── kernwerk-vcs/
│   ├── kernwerk-ai/
│   └── kernwerk-cli/
│
├── ui/
│   ├── qt/
│   └── qml/
│
├── schemas/
├── templates/
├── docs/
└── prompts/
```

No MVP, não é necessário criar todos os crates. Começar com:

```text
kernwerk-core
kernwerk-protocol
kernwerk-config
kernwerk-cli
```

## Camadas

### 1. UI Layer

Responsável por:

- renderizar interface;
- mostrar menus;
- mostrar painéis;
- receber input;
- enviar comandos;
- mostrar eventos do core;
- preservar estado visual.

Não deve:

- chamar `git` diretamente;
- chamar `cmake` diretamente;
- iniciar `clangd` diretamente;
- executar build diretamente;
- decidir regras de strict mode.

### 2. Protocol Layer

Responsável por:

- tipos de mensagens;
- JSON-RPC;
- schemas;
- versionamento de protocolo;
- serialização/desserialização.

### 3. Core Application Layer

Responsável por:

- orquestrar comandos;
- gerenciar workspace;
- validar configurações;
- aplicar políticas;
- chamar serviços internos;
- emitir eventos.

### 4. Domain/Service Layer

Responsável por:

- workspace service;
- command registry;
- tool manager;
- build manager;
- diagnostics manager;
- settings manager;
- AI provider manager;
- LSP process manager.

### 5. Adapter Layer

Responsável por integrar ferramentas externas:

- processos do sistema;
- arquivos;
- sockets;
- JSON-RPC LSP;
- Git CLI;
- CMake CLI;
- Gradle/Maven;
- Python tools;
- GPT/Claude/Ollama.

## Regra de dependência

Fluxo permitido:

```text
UI → Protocol → Core → Services → Adapters → External Tools
```

Fluxo proibido:

```text
UI → External Tools
UI → Filesystem complexo
UI → Build system
UI → Git
UI → LSP direto
```

## Modelo mental

```text
Workspace
└── Projects
    └── Modules
        └── Targets
            └── Run Configurations
```

## Configurações locais

A IDE deve seguir padrão Linux:

```text
~/.config/kernwerk-studio/
~/.cache/kernwerk-studio/
~/.local/share/kernwerk-studio/
```

Por projeto:

```text
.kernwerk/
├── workspace.json
├── settings.json
├── run-configs.json
├── toolchains.json
├── layout.json
└── cache/
```

## Arquitetura de processo

Inicialmente:

```text
kernwerk-core
kernwerk-cli
```

Depois:

```text
kernwerk-studio  # UI Qt/QML
kernwerk-core    # daemon Rust
kernwerk-cli     # CLI para automação/debug
```

## Regra de evolução

Não adicionar recurso visual complexo antes de validar:

1. protocolo;
2. sistema de comandos;
3. settings;
4. logs;
5. tool detection;
6. workspace básico.


---

# FILE: docs/03_ENGINEERING_PRACTICES.md

# 03 — Regras de Engenharia de Software

Este documento define boas práticas obrigatórias para desenvolvimento do Kernwerk Studio.

## Princípios

1. Simplicidade antes de abstração.
2. Modularidade antes de conveniência.
3. Testabilidade antes de velocidade de implementação.
4. Contratos explícitos antes de comportamento mágico.
5. Erros claros antes de panics.
6. Logs úteis antes de silêncio.
7. Documentação junto com código.
8. Performance como requisito, não otimização tardia.
9. UI previsível antes de UI impressionante.
10. Ferramentas abertas existentes antes de soluções próprias.

## Regras de código Rust

### Proibido por padrão

- `unsafe`;
- `unwrap`;
- `expect`;
- `panic!`;
- `todo!`;
- `unimplemented!`;
- `dbg!`;
- lógica de negócio dentro de `main`;
- dependência circular;
- estado global mutável;
- strings mágicas espalhadas;
- erros como `String` sem tipo.

### Permitido com justificativa

- `Arc<Mutex<T>>`;
- channels;
- tasks assíncronas;
- crates externas;
- cache persistente;
- lazy initialization.

Sempre justificar quando aumentar complexidade.

## Tratamento de erros

Usar erros explícitos.

Exemplo conceitual:

```rust
pub enum ToolError {
    NotFound { tool: String },
    FailedToStart { tool: String, reason: String },
    InvalidOutput { tool: String },
}
```

Todo erro enviado à UI precisa ter:

```text
code
message
details
suggestedAction opcional
```

## Logs

Logs devem ter níveis:

```text
trace
debug
info
warn
error
```

Logs técnicos devem ir para arquivo.  
Mensagens de usuário devem ser eventos estruturados.

Nunca logar:

- tokens;
- chaves de API;
- conteúdo de arquivos privados sem permissão;
- prompts completos sem opção explícita.

## Testes

Obrigatório para:

- parsing de config;
- protocolo IPC;
- command registry;
- detecção de ferramentas;
- validação de settings;
- strict mode;
- resolução de caminhos;
- schemas;
- serialização;
- tratamento de erro.

Tipos:

```text
unit tests
integration tests
snapshot tests futuros
golden tests para templates futuros
```

## Documentação

Toda mudança que afeta arquitetura deve atualizar docs.

Mudanças que exigem atualização:

- novo comando;
- novo evento IPC;
- novo setting;
- novo arquivo de configuração;
- nova dependência;
- mudança de strict mode;
- mudança de layout;
- mudança em performance budget.

## Dependências

Antes de adicionar uma dependência, responder:

1. O problema justifica dependência?
2. A crate é mantida?
3. A licença é compatível?
4. A dependência é pesada?
5. Existe alternativa na std?
6. Ela afeta build time?
7. Ela introduz risco de segurança?

## Convenções de nomes

Packages:

```text
kernwerk-core
kernwerk-protocol
kernwerk-config
```

Crates/imports:

```text
kernwerk_core
kernwerk_protocol
kernwerk_config
```

Tipos:

```text
PascalCase
```

Funções/módulos:

```text
snake_case
```

Comandos:

```text
workspace.open
build.run
git.status
ai.explainSelection
```

Eventos:

```text
event.workspace.opened
event.build.finished
event.tool.statusChanged
```

## Arquitetura limpa

Evitar que módulos saibam demais uns dos outros.

Exemplo correto:

```text
CommandRegistry → BuildService → CMakeAdapter
```

Exemplo errado:

```text
UI Button → CMakeAdapter → Terminal Output Widget
```

## API interna

Toda API pública deve ser pequena.

Preferir:

```rust
open_workspace(path)
```

a APIs que exigem múltiplas chamadas em ordem específica.

## Concorrência

- Core pode executar tarefas paralelas.
- Toda tarefa longa deve emitir progresso.
- Nenhuma tarefa longa deve bloquear outro subsistema sem necessidade.
- Usar cancelamento quando possível.
- Build, LSP, busca e IA devem ser independentes.

## Estado

Estado deve ser:

- serializável quando persistente;
- observável por eventos;
- protegido contra corrupção;
- validado ao carregar.

## Configuração

Configuração deve ter:

- default seguro;
- schema;
- versão;
- migração futura;
- validação;
- mensagens amigáveis.

## Regra final

Se uma implementação dificulta testes, logs, documentação ou performance, provavelmente a arquitetura está errada.


---

# FILE: docs/04_STRICT_TOOLCHAINS.md

# 04 — Configuração Rígida de Compiladores, Linters e Ferramentas

Este documento define o modo mais exigente possível para o Kernwerk Studio e para projetos gerados por ele.

## Filosofia

```text
Strict by default.
Relaxed only by explicit choice.
```

Projetos novos devem nascer em qualidade profissional.

## Regras globais

- Warnings devem quebrar build quando possível.
- Formatador obrigatório.
- Linter obrigatório.
- Testes configurados por padrão.
- Configurações devem ser explícitas.
- Dependências externas não devem quebrar build por warnings do projeto.
- Código de terceiros deve ser isolado de `-Werror`.

## Rust — Kernwerk Core

### Cargo workspace

`Cargo.toml` raiz recomendado:

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

### `rust-toolchain.toml`

```toml
[toolchain]
channel = "stable"
components = [
    "rustfmt",
    "clippy",
]
```

### Comandos obrigatórios

```bash
cargo fmt --all --check
cargo check --workspace --all-targets --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

### Ferramentas futuras

```bash
cargo install cargo-deny
cargo install cargo-audit
cargo install cargo-nextest
```

Validações futuras:

```bash
cargo deny check
cargo audit
cargo nextest run --workspace
```

## C++ Strict

### Padrão

- C++23;
- CMake;
- Ninja;
- `CMAKE_EXPORT_COMPILE_COMMANDS=ON`;
- clangd;
- clang-format;
- clang-tidy;
- CTest;
- sanitizers em Debug;
- LTO em Release Hardened.

### CMakePresets exemplo

```json
{
  "version": 6,
  "configurePresets": [
    {
      "name": "debug-strict",
      "displayName": "Debug Strict",
      "generator": "Ninja",
      "binaryDir": "${sourceDir}/build/debug-strict",
      "cacheVariables": {
        "CMAKE_BUILD_TYPE": "Debug",
        "CMAKE_EXPORT_COMPILE_COMMANDS": "ON",
        "ENABLE_WARNINGS_AS_ERRORS": "ON",
        "ENABLE_SANITIZERS": "ON"
      }
    },
    {
      "name": "release-hardened",
      "displayName": "Release Hardened",
      "generator": "Ninja",
      "binaryDir": "${sourceDir}/build/release-hardened",
      "cacheVariables": {
        "CMAKE_BUILD_TYPE": "Release",
        "CMAKE_EXPORT_COMPILE_COMMANDS": "ON",
        "ENABLE_WARNINGS_AS_ERRORS": "ON",
        "ENABLE_LTO": "ON"
      }
    }
  ],
  "buildPresets": [
    {
      "name": "debug-strict",
      "configurePreset": "debug-strict"
    },
    {
      "name": "release-hardened",
      "configurePreset": "release-hardened"
    }
  ]
}
```

### Warnings Clang/GCC

```cmake
target_compile_options(project_options INTERFACE
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
)
```

### Sanitizers

```cmake
target_compile_options(project_options INTERFACE
    -fsanitize=address,undefined
    -fno-omit-frame-pointer
)

target_link_options(project_options INTERFACE
    -fsanitize=address,undefined
)
```

### Regra para dependências

Não aplicar `-Werror` a código de terceiros.

Dependências devem ser adicionadas como `SYSTEM` quando apropriado.

## Java Strict

### Alvo

Java 25 LTS como alvo desejado do usuário.

### Build systems

- Gradle;
- Maven.

### Ferramentas

- JDT LS;
- JUnit;
- Checkstyle;
- SpotBugs;
- PMD;
- Error Prone opcional;
- JaCoCo opcional.

### Regras

- warnings visíveis;
- testes configurados;
- estilo consistente;
- dependências auditáveis;
- logs estruturados em backend;
- profiles dev/prod;
- `.env.example` quando backend.

### Gradle ideia base

```kotlin
tasks.withType<JavaCompile>().configureEach {
    options.compilerArgs.addAll(listOf(
        "-Xlint:all"
    ))
}
```

Warnings como erro podem ser ativados em perfil strict/CI quando viável.

## Python Strict

### Ferramentas

- uv;
- Ruff;
- Pyright ou basedpyright;
- pytest;
- mypy opcional.

### `pyproject.toml` base

```toml
[tool.ruff]
line-length = 100
target-version = "py312"

[tool.ruff.lint]
select = ["ALL"]
ignore = [
    "D100",
    "D104"
]

[tool.pyright]
typeCheckingMode = "strict"

[tool.pytest.ini_options]
addopts = "-ra --strict-markers --strict-config"
testpaths = ["tests"]
```

### Regras

- `.venv` obrigatório;
- imports organizados;
- formatador obrigatório;
- type checking strict;
- testes por padrão;
- `.env.example` em backend;
- nunca instalar pacotes globalmente por padrão.

## Backend Strict

### Regras

- `.env.example`;
- healthcheck;
- logs estruturados;
- OpenAPI quando aplicável;
- testes de endpoint;
- Docker/Podman Compose opcional;
- separação dev/prod;
- configuração explícita de portas;
- nenhum segredo commitado.

## Embedded Strict

### Regras

- target explícito;
- toolchain file explícito;
- sysroot explícito;
- perfil de debug;
- perfil de flash/deploy;
- serial monitor;
- GDB remote;
- QEMU quando aplicável;
- logs de build/deploy.

### CMake toolchain esperado

```text
toolchains/
├── arm64-linux-gnu.cmake
├── stm32.cmake
└── esp32.cmake
```

## Níveis

```text
Strict       padrão
Balanced     futuro
Relaxed      futuro/legado
```

## Regra final

Strict mode não pode ser uma opção estética. Ele deve afetar compilação, lint, testes, templates, CI e diagnósticos.


---

# FILE: docs/05_LAYOUT_AND_UX.md

# 05 — Layout e UX

Este documento define layout, experiência visual, menus, atalhos e comportamento da interface do Kernwerk Studio.

## Objetivo visual

Criar uma IDE escura, confortável e previsível para longas horas de desenvolvimento, com memória muscular próxima do ecossistema JetBrains.

## Tema padrão

O tema padrão deve ser escuro e confortável.

Paleta principal:

```text
#6e5c01
#eae6e1
#ffbb00
#6e6c58
```

Paleta auxiliar sugerida:

```text
Background 0:   #0d0e0e
Background 1:   #121313
Background 2:   #191a18
Surface 1:      #1f201d
Surface 2:      #25261f
Border soft:    #2a2922
Text primary:   #eae6e1
Text secondary: #b9b3a5
Text muted:     #8f8a7c
Accent strong:  #ffbb00
Accent dim:     #6e5c01
Neutral olive:  #6e6c58
```

## Layout principal

```text
┌──────────────────────────────────────────────────────────────┐
│ KW | Projeto | Branch | Run Config | Build | Run | Debug | IA │
├────┬───────────────────────────────┬─────────────────────────┤
│    │ Editor Tabs                   │ Assistente / Inspector  │
│Bar │ Código                        │ Docs / Outline          │
│    │                               │                         │
├────┴───────────────────────────────┴─────────────────────────┤
│ Terminal | Problems | Build | Git | Debug | Tests             │
├──────────────────────────────────────────────────────────────┤
│ Status: Git | Toolchain | LSP | Build | Encoding | Line/Col   │
└──────────────────────────────────────────────────────────────┘
```

## Regra de memória muscular

A IDE não deve mover painéis automaticamente.

Painéis devem ser previsíveis:

```text
Alt+1   Project
Alt+4   Run/Build
Alt+5   Debug
Alt+9   Git
```

Atalhos JetBrains-like:

```text
Shift Shift       Search Everywhere
Ctrl+Shift+A     Find Action
Alt+Enter        Quick Fix
Ctrl+B           Go to Definition
Ctrl+Alt+B       Go to Implementation
Ctrl+Alt+L       Format Code
Shift+F6         Rename
Ctrl+Shift+F     Search in Files
Ctrl+E           Recent Files
```

## Top Bar

Itens:

```text
KW
Projeto atual
Branch atual
Run Configuration
Build
Run
Debug
Search
Settings
```

### Menu Projeto

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

### Menu Branch

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

### Menu Run Config

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

Para embarcados:

```text
Debug ARM64 Remote
Flash STM32
Run QEMU ARM
Attach GDB Remote
Serial Monitor
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

Comportamento:

- clique abre/recolhe painel;
- botão direito abre opções;
- tooltip no hover;
- posição fixa;
- sem reorganização automática.

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

### Clique direito em arquivo

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

### Clique direito em pasta

```text
Novo arquivo
Nova classe C++
Novo header
Novo source
Novo pacote Java
Novo módulo Python
Novo teste
Marcar como Sources
Marcar como Includes
Marcar como Tests
Excluir do projeto
```

## Editor

### Clique direito no código

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
Revisar segurança
```

### Hover sobre erro

Card esperado:

```text
Erro: conversão implícita pode perder dados
Arquivo: app.cpp:42
Regra: -Wconversion

[Aplicar correção] [Ver documentação] [Explicar com IA]
```

Em strict mode, ações de "ignorar" devem ser discretas.

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

### Problems

Deve agrupar por:

```text
Todos
C++
Java
Python
CMake
Security
```

Cada problema deve mostrar:

```text
arquivo
linha
severidade
origem
ação sugerida
```

### Build

Deve mostrar fases:

```text
Configure
Generate
Compile
Link
Test
Package
```

Com botões:

```text
Ver erro
Copiar log
Explicar com IA
Abrir arquivo
```

## Painel direito

Tabs:

```text
Assistente KW
Inspector
Docs
Outline
```

### Assistente KW

Deve mostrar:

```text
Contexto atual
Arquivos incluídos
Provider ativo
Política de privacidade
Ações rápidas
Chat
```

Ações rápidas:

```text
Explicar arquivo
Sugerir refatoração
Gerar teste
Revisar performance
Revisar segurança
Gerar commit
```

## Novo Projeto

Tela visual:

```text
Linguagem/Ecossistema
[C++] [Java] [Python] [Embedded] [Backend] [Rust]

Template
C++ Console Strict
Qt/QML App
Java Spring Boot
Python FastAPI
Embedded Linux
ROS 2/Gazebo

Qualidade
Strict / Professional
Balanced
Relaxed

Ferramentas
Git
Tests
Formatter
Linter
CI config
Documentation template
```

Padrão:

```text
Strict / Professional
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

## Estados vazios

Toda tela deve ter estado vazio útil.

Exemplos:

```text
Nenhum projeto aberto.
[Abrir projeto] [Novo projeto]

clangd não encontrado.
[Instalar via pacman] [Selecionar binário] [Desativar C++ inteligente]

Nenhum provider de IA configurado.
[Usar Ollama] [Usar GPT CLI] [Usar Claude CLI]
```

## Regra final

A UI deve ser bonita, mas nunca imprevisível.  
A previsibilidade vale mais que animação ou efeito visual.


---

# FILE: docs/06_PERFORMANCE_RULES.md

# 06 — Regras de Performance e Prevenção de Conflitos

Este documento define limites de performance, responsabilidades de módulos e regras para evitar conflito entre UI, Core e ferramentas externas.

## Objetivo

Kernwerk Studio deve ser rápido, leve e responsivo.

A interface nunca deve parecer travada.

## Performance budgets iniciais

```text
Início do core:                  < 500 ms
Ping UI ↔ Core:                  < 50 ms
Abrir janela inicial:            < 1 s
Abrir projeto pequeno:           < 2 s
Resposta visual a clique:        imediata
Build bloquear UI:               proibido
IA bloquear editor:              proibido
LSP bloquear editor:             proibido
Uso de RAM projeto pequeno:      meta aspiracional < 300 MB
```

## Regra absoluta

A UI não executa trabalho pesado.

Trabalho pesado inclui:

- build;
- indexação;
- busca global;
- LSP;
- parsing grande;
- Git status em repositório grande;
- IA;
- leitura massiva de arquivos;
- geração de documentação;
- análise estática.

## Separação de responsabilidades

### UI

Pode:

- renderizar;
- enviar comandos;
- receber eventos;
- mostrar loading;
- mostrar resultados;
- preservar layout.

Não pode:

- executar `cmake`;
- executar `git`;
- iniciar `clangd`;
- rodar IA;
- ler árvore inteira de projeto sozinha;
- bloquear esperando processo.

### Core

Pode:

- orquestrar comandos;
- chamar ferramentas externas;
- gerenciar cache;
- emitir eventos;
- controlar processos;
- validar settings.

Não deve:

- depender de Qt;
- renderizar UI;
- assumir detalhes visuais;
- manter estado visual que pertence à UI.

### Ferramentas externas

Devem ser isoladas:

- processo próprio;
- logs próprios;
- timeout quando aplicável;
- cancelamento quando possível;
- status reportado ao core.

## Concorrência

Regras:

1. Toda operação longa deve ser assíncrona.
2. Toda operação longa deve emitir progresso.
3. Toda operação longa deve poder falhar sem derrubar a IDE.
4. Toda operação longa deve ter logs.
5. Operações independentes não devem bloquear umas às outras.
6. Cancelamento deve ser suportado em build, busca, IA e tasks longas.

## Eventos de progresso

Exemplo:

```json
{
  "method": "event.build.progress",
  "params": {
    "phase": "compile",
    "current": 12,
    "total": 40,
    "message": "Compilando app.cpp"
  }
}
```

## Evitar conflitos entre módulos

### Build vs LSP

- Build não deve matar LSP.
- LSP pode usar `compile_commands.json`.
- Quando CMake regenera `compile_commands.json`, core avisa LSP.
- Reiniciar LSP deve ser ação controlada.

### Git vs File Watcher

- File watcher não deve disparar reindexação completa por qualquer alteração.
- Alterações em `.git/` devem ser tratadas com debounce.
- Git status deve ter throttling.

### IA vs Privacidade

- IA externa não pode ler arquivos sem permissão.
- IA local pode ter política mais flexível.
- UI deve mostrar contexto incluído.

### Build vs Terminal

- Terminal manual não deve ser confundido com build gerenciado.
- Build gerenciado tem painel próprio e eventos próprios.
- Terminal é livre, mas não substitui o Build Manager.

### Settings vs Runtime

- Alterar settings deve emitir evento.
- Serviços afetados podem reiniciar com confirmação.
- Nada deve reiniciar silenciosamente sem feedback.

## Debounce e throttling

Usar debounce para:

- file watcher;
- Git status;
- busca incremental;
- diagnostics agregados;
- atualização de outline;
- autosave futuro.

Exemplos conceituais:

```text
Git status: no máximo a cada 1-2 segundos durante mudanças intensas.
File watcher: agrupar eventos em janelas curtas.
Search: não disparar busca global a cada tecla sem debounce.
```

## Cache

Cache permitido para:

- arquivos recentes;
- projetos recentes;
- layout;
- diagnostics recentes;
- histórico de comandos;
- status de ferramentas;
- metadados de workspace.

Cache não deve esconder erro.

Se cache falhar, a IDE deve continuar com degradação controlada.

## Logs de performance

O Kernwerk deve medir localmente:

```text
tempo de start do core
tempo de abertura do workspace
latência IPC
tempo de detecção de ferramentas
tempo de build
tempo de busca
uso de memória aproximado
número de processos externos
```

Sem telemetria externa.

## Falhas

Falha de módulo não deve derrubar a IDE inteira.

Exemplos:

- clangd caiu → mostrar status e opção de reiniciar.
- Git falhou → mostrar erro no painel Git.
- IA falhou → manter editor funcionando.
- Build falhou → mostrar diagnóstico.
- Core caiu → UI mostra reconectar/reiniciar core.

## Regra final

Performance não é etapa futura.  
Todo módulo novo deve declarar:

```text
tempo esperado
se roda em background
como cancela
que eventos emite
que logs produz
que módulo pode afetar
```


---

# FILE: docs/07_IPC_PROTOCOL.md

# 07 — Protocolo IPC

## Objetivo

O protocolo IPC conecta:

```text
Qt/QML UI  ←→  Rust Core
```

## Transporte inicial

MVP:

```text
stdin/stdout JSON-RPC
```

Futuro:

```text
Unix domain socket
```

## Formato

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
  "method": "event.core.ready",
  "params": {
    "version": "0.1.0"
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
logs.tail
```

## Eventos iniciais

```text
event.core.ready
event.workspace.opened
event.workspace.closed
event.tool.statusChanged
event.notification.created
event.log.appended
```

## Regras

1. Todo request tem resposta.
2. Evento não tem `id`.
3. Erro sempre tem `code`, `message`, `details`.
4. Protocolo deve ter versão.
5. Mudança no protocolo exige atualização de schema.
6. UI não deve depender de texto de log para lógica.


---

# FILE: docs/08_COMMAND_SYSTEM.md

# 08 — Sistema de Comandos

## Ideia central

Tudo na IDE deve ser um comando.

Menus, atalhos, command palette, botões, plugins e IA devem chamar o mesmo sistema.

## Exemplos

```text
core.ping
workspace.open
workspace.reload
project.create
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

## Regras

1. Botões chamam comandos.
2. Menus chamam comandos.
3. Atalhos chamam comandos.
4. Command palette chama comandos.
5. Plugins registram comandos.
6. IA pode sugerir comandos, mas não executar comandos destrutivos sem confirmação.
7. Comandos longos emitem eventos.
8. Comandos validam contexto.
9. Comandos retornam resultado estruturado.

## Command Palette

Atalhos:

```text
Shift Shift
Ctrl+Shift+A
Ctrl+Shift+P
```

## Presets de atalhos

Inicial:

```text
JetBrains Compatible
```

Futuros:

```text
Kernwerk Default
VS Code Compatible
Vim Mode
```


---

# FILE: docs/09_TOOLING_LIFECYCLE.md

# 09 — Ciclo de Vida das Ferramentas Externas

## Objetivo

O Kernwerk Studio deve detectar, iniciar, monitorar, reiniciar e parar ferramentas externas de forma estruturada.

## Ferramentas previstas

```text
Rust: cargo, rustc, rustfmt, clippy
C++: clangd, clang++, cmake, ninja, clang-format, clang-tidy, gdb, lldb
Java: java, javac, jdtls, maven, gradle
Python: uv, python, pyright, basedpyright, ruff, pytest
Git: git
Busca: ripgrep, fd
IA: gpt cli, claude cli, ollama
Embedded: qemu, openocd, pyocd, gdb-multiarch
```

## Estados

```text
NotConfigured
Missing
Detected
Ready
Running
Failed
Disabled
```

## ToolManager

```text
ToolManager
├── ToolDetector
├── ToolRegistry
├── ToolHealthChecker
├── ProcessManager
├── LogCollector
└── ToolSuggestionProvider
```

## Exemplo de mensagem

```text
clangd não encontrado.

O Kernwerk Studio usa clangd para recursos inteligentes de C++.

[Instalar via pacman] [Selecionar binário] [Desativar C++ inteligente]
```

## Instalação

O core nunca instala algo sem confirmação explícita.

No CachyOS/Arch, sugestões podem incluir:

```bash
sudo pacman -S rustup cmake ninja clang lldb gdb qt6-base qt6-tools git ripgrep fd
```

## Logs

Logs de ferramentas:

```text
~/.cache/kernwerk-studio/logs/tools/
```

## Regras

1. UI não chama ferramenta diretamente.
2. Core gerencia processos.
3. Toda ferramenta tem status.
4. Toda falha tem mensagem amigável.
5. Logs técnicos ficam acessíveis.
6. Ferramentas podem ser reiniciadas pelo usuário.


---

# FILE: docs/10_AI_POLICY.md

# 10 — Política de IA

## Objetivo

Integrar IA sem comprometer privacidade, liberdade e controle do usuário.

## Providers

```text
Ollama
GPT CLI
Claude CLI
OpenAI API
Anthropic API
OpenRouter
```

## Padrão

```text
IA local: permitida
IA externa: perguntar antes de enviar contexto
Telemetria: desligada
```

## Regras

1. Nunca enviar projeto inteiro sem confirmação.
2. Mostrar arquivos/trechos enviados como contexto.
3. Nunca logar API keys.
4. Nunca executar comando destrutivo sugerido por IA sem confirmação.
5. Permitir desativar IA completamente.
6. Permitir modo local-only.
7. Prompts salvos localmente devem ser opção explícita.

## Ações

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
Provider ativo
Política de privacidade
Contexto atual
Arquivos incluídos
Ações rápidas
Chat
```


---

# FILE: docs/11_ROADMAP_AND_MVP.md

# 11 — Roadmap e MVP

## Decisão inicial

Criar primeiro o projeto como Rust workspace no CLion.

Nome:

```text
kernwerk-studio
```

## MVP 0.1 — Rust Core mínimo

Escopo:

- `Cargo.toml` workspace.
- `rust-toolchain.toml`.
- `crates/kernwerk-core`.
- `crates/kernwerk-protocol`.
- `crates/kernwerk-config`.
- `crates/kernwerk-cli`.
- `core.ping`.
- logging básico.
- testes.
- fmt/clippy/test/check.

Fora do escopo:

- Qt/QML;
- editor;
- LSP;
- Git;
- CMake;
- IA;
- Embedded.

## MVP 0.2 — Tool detection

- detectar cargo/rustc/rustfmt/clippy;
- detectar cmake/ninja/git/clangd;
- retornar status estruturado;
- sugerir instalação no CachyOS/Arch.

## MVP 0.3 — Workspace

- abrir pasta;
- detectar tipo de projeto;
- criar `.kernwerk/workspace.json`;
- listar arquivos;
- salvar settings básicos.

## MVP 0.4 — IPC real

- core como processo;
- CLI comunica via JSON-RPC;
- eventos básicos;
- logs.

## MVP 0.5 — UI Qt/QML mínima

- janela escura;
- conecta no core;
- botão Ping;
- status Core conectado;
- painel Logs.

## MVP 0.6 — Explorer e editor básico

- abrir pasta;
- explorer;
- abrir arquivo;
- salvar arquivo;
- abas;
- tema escuro.

## MVP 0.7 — Build CMake

- detectar CMake;
- detectar presets;
- configure;
- build;
- painel Build.

## MVP 0.8 — LSP C++

- iniciar clangd;
- diagnostics;
- hover;
- go to definition;
- completion simples.

## Regra

Não avançar para recursos grandes antes de validar core, protocolo e comandos.


---

# FILE: prompts/GPT_TERMINAL_BOOTSTRAP.md

# Prompt inicial para GPT/Claude no terminal

Você está no projeto **Kernwerk Studio**.

Leia antes de agir:

```text
AGENTS.md
docs/00_INDEX.md
docs/01_PRODUCT_VISION.md
docs/02_ARCHITECTURE.md
docs/03_ENGINEERING_PRACTICES.md
docs/04_STRICT_TOOLCHAINS.md
docs/11_ROADMAP_AND_MVP.md
```

## Contexto

Kernwerk Studio é uma IDE open source, Linux-first, visualmente plug and play, rígida por padrão e focada em performance.

Arquitetura:

```text
Qt/QML Frontend  ← IPC/JSON-RPC local → Rust Core
```

## Tarefa inicial

Crie a base Rust workspace do projeto:

```text
Cargo.toml
rust-toolchain.toml
crates/kernwerk-core/
crates/kernwerk-protocol/
crates/kernwerk-config/
crates/kernwerk-cli/
```

Implemente apenas:

- protocolo mínimo;
- comando `core.ping`;
- CLI para testar ping;
- testes;
- strict mode;
- fmt/clippy/check/test.

## Restrições

- Não implementar Qt ainda.
- Não implementar LSP ainda.
- Não implementar CMake ainda.
- Não implementar Git ainda.
- Não adicionar dependências desnecessárias.
- Não usar `unsafe`.
- Não usar `unwrap`, `expect`, `panic`, `todo!`, `dbg!` fora de testes.
- Não misturar UI com core.

## Comandos obrigatórios

```bash
cargo fmt --all --check
cargo check --workspace --all-targets --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

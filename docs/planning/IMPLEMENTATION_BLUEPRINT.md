# Kernwerk Studio — Implementation Blueprint

> Documento final de execução para iniciar o desenvolvimento do **Kernwerk Studio** de forma pragmática, rígida e alinhada aos padrões profissionais definidos nos documentos anteriores.

---

## 1. Objetivo deste documento

Este documento transforma a visão arquitetural do Kernwerk Studio em um plano inicial de implementação.

Ele define:

```text
estrutura inicial de pastas;
crates Rust;
módulos;
ordem de implementação;
contratos internos;
schemas JSON;
comandos IPC;
testes obrigatórios;
milestones;
critérios de aceite;
regras para agentes de IA;
o que fazer e o que não fazer no MVP.
```

Este arquivo deve ser usado como guia prático para iniciar o projeto sem improviso.

---

## 2. Documentos que devem ser lidos antes

Antes de implementar, ler:

```text
docs/ARCHITECTURE_OVERVIEW.md
docs/TOOLCHAIN_LAYER_POLICY.md
docs/CPP_RUST_NATIVE_TOOLING.md
docs/QUALITY_CENTER_STRICT_MODES.md
docs/PROFESSIONAL_IDE_STANDARDS.md
docs/IMPLEMENTATION_BLUEPRINT.md
```

Se houver conflito entre documentos, usar esta ordem de prioridade:

```text
1. PROFESSIONAL_IDE_STANDARDS.md
2. IMPLEMENTATION_BLUEPRINT.md
3. TOOLCHAIN_LAYER_POLICY.md
4. QUALITY_CENTER_STRICT_MODES.md
5. CPP_RUST_NATIVE_TOOLING.md
6. ARCHITECTURE_OVERVIEW.md
```

Motivo:

```text
PROFESSIONAL_IDE_STANDARDS.md define regras globais.
IMPLEMENTATION_BLUEPRINT.md define execução prática.
Os demais documentos definem visão, políticas e domínio.
```

---

## 3. Filosofia de implementação

O Kernwerk Studio deve começar pequeno, mas correto.

A regra é:

```text
MVP pequeno.
Arquitetura séria.
Sem atalhos perigosos.
Sem telemetria.
Sem ferramentas pesadas no boot.
Sem sobrescrever arquivos sem confirmação.
Sem tentar ser universal cedo demais.
```

O foco inicial é:

```text
C
C++
Rust
CMake
Cargo
clangd
rust-analyzer
Quality Center
toolchains locais
Qt/QML UI futura
```

---

## 4. Escopo oficial do MVP

## 4.1 Dentro do MVP inicial

```text
Rust workspace;
CLI funcional;
core.ping;
command registry;
config manager;
tool detection básica;
workspace open;
project detection básica;
Cargo detection;
CMake detection;
quality-profile.json;
quality.runAll inicial;
logs locais;
testes;
strict mode do próprio Kernwerk.
```

## 4.2 Fora do MVP inicial

```text
Qt/QML UI completa;
editor avançado;
debug visual;
DAP;
plugin system;
Java;
Python;
backend services;
IA profunda;
marketplace;
embedded completo;
profiler visual;
indexador próprio completo.
```

Regra:

```text
Não implementar UI antes de o Core, Protocol, Commands, Config e Tooling estarem testáveis.
```

---

## 5. Estrutura inicial de pastas

Estrutura recomendada:

```text
kernwerk-studio/
├── Cargo.toml
├── rust-toolchain.toml
├── README.md
├── AGENTS.md
├── LICENSES/
├── docs/
│   ├── ARCHITECTURE_OVERVIEW.md
│   ├── TOOLCHAIN_LAYER_POLICY.md
│   ├── CPP_RUST_NATIVE_TOOLING.md
│   ├── QUALITY_CENTER_STRICT_MODES.md
│   ├── PROFESSIONAL_IDE_STANDARDS.md
│   └── IMPLEMENTATION_BLUEPRINT.md
│
├── crates/
│   ├── kernwerk-cli/
│   ├── kernwerk-core/
│   ├── kernwerk-protocol/
│   ├── kernwerk-config/
│   ├── kernwerk-command/
│   ├── kernwerk-tooling/
│   ├── kernwerk-project/
│   ├── kernwerk-quality/
│   ├── kernwerk-diagnostics/
│   └── kernwerk-test-support/
│
├── schemas/
│   ├── quality-profile.schema.json
│   ├── toolchains.schema.json
│   ├── run-configs.schema.json
│   └── workspace.schema.json
│
├── templates/
│   ├── cpp-console-strict/
│   ├── cpp-library-strict/
│   ├── rust-cli-strict/
│   └── rust-workspace-strict/
│
├── cmake/
│   └── templates/
│
├── ui/
│   └── qt/
│
├── xtask/
│   └── src/
│
└── .github/
    └── workflows/
```

---

## 6. Cargo workspace inicial

O `Cargo.toml` raiz deve ser simples e rígido.

```toml
[workspace]
resolver = "2"
members = [
    "crates/kernwerk-cli",
    "crates/kernwerk-core",
    "crates/kernwerk-protocol",
    "crates/kernwerk-config",
    "crates/kernwerk-command",
    "crates/kernwerk-tooling",
    "crates/kernwerk-project",
    "crates/kernwerk-quality",
    "crates/kernwerk-diagnostics",
    "crates/kernwerk-test-support",
    "xtask"
]

[workspace.package]
edition = "2024"
license = "MIT OR Apache-2.0"
repository = "https://github.com/<owner>/kernwerk-studio"
rust-version = "1.85"

[workspace.lints.rust]
unsafe_code = "forbid"
warnings = "deny"

[workspace.lints.clippy]
unwrap_used = "deny"
expect_used = "deny"
panic = "deny"
todo = "deny"
dbg_macro = "deny"
unimplemented = "deny"
```

Observação:

```text
A versão exata do Rust deve ser ajustada conforme a toolchain real usada no início.
Se edition 2024 ainda gerar atrito em alguma dependência, usar edition 2021 temporariamente.
```

---

## 7. rust-toolchain.toml

```toml
[toolchain]
channel = "stable"
components = [
    "rustfmt",
    "clippy",
    "rust-src"
]
```

Regra:

```text
O projeto deve compilar em stable.
Nightly só pode ser usado com justificativa explícita e nunca no MVP.
```

---

## 8. Crates e responsabilidades

## 8.1 kernwerk-cli

Responsável por:

```text
entrada via terminal;
subcomandos;
chamar o core;
testar funcionalidades sem UI;
rodar quality checks;
detectar ferramentas;
abrir workspace;
emitir JSON opcional.
```

Comandos iniciais:

```text
kernwerk ping
kernwerk tools detect
kernwerk workspace open <path>
kernwerk quality run
kernwerk config validate
```

---

## 8.2 kernwerk-core

Responsável por orquestração.

```text
inicialização;
shutdown;
event bus interno;
coordenação entre comandos, config, tooling, project e quality;
controle de estado global mínimo;
serviços principais.
```

Não deve conter:

```text
lógica específica de CMake profunda;
lógica específica de Cargo profunda;
UI;
código Qt;
execução direta desorganizada de processos.
```

---

## 8.3 kernwerk-protocol

Responsável por contratos.

```text
tipos de request;
tipos de response;
tipos de eventos;
erros padronizados;
serialização JSON;
futuros contratos IPC.
```

Deve ser o crate mais estável possível.

---

## 8.4 kernwerk-config

Responsável por:

```text
ler configurações;
validar schemaVersion;
aplicar defaults;
migrar versões futuras;
salvar configurações;
evitar sobrescrita perigosa;
normalizar caminhos.
```

Arquivos:

```text
.kernwerk/workspace.json
.kernwerk/quality-profile.json
.kernwerk/toolchains.json
.kernwerk/run-configs.json
```

---

## 8.5 kernwerk-command

Responsável por:

```text
Command Registry;
metadados dos comandos;
permissões;
categorias;
execução;
validação;
cancelamento futuro;
eventos.
```

Todo recurso da IDE deve passar pelo command system.

---

## 8.6 kernwerk-tooling

Responsável por:

```text
detecção de ferramentas;
status de ferramentas;
execução controlada de processos;
captura de stdout/stderr;
timeouts;
logs;
sugestões de instalação;
validação de versão.
```

Ferramentas iniciais:

```text
git
cargo
rustc
rustfmt
clippy
rust-analyzer
cmake
ninja
clang
clang++
clangd
clang-format
clang-tidy
gdb
lldb
```

---

## 8.7 kernwerk-project

Responsável por:

```text
abrir workspace;
detectar tipo de projeto;
detectar Cargo.toml;
detectar CMakeLists.txt;
detectar CMakePresets.json;
ler informações iniciais;
montar Project Model básico.
```

Não deve implementar análise semântica própria de C++ ou Rust.

---

## 8.8 kernwerk-quality

Responsável por:

```text
Quality Center;
perfis;
biblioteca de regras;
quality-profile.json;
pipeline de verificação;
execução de checks;
relatório estruturado.
```

Checks iniciais:

```text
cargo fmt --check;
cargo check;
cargo clippy;
cargo test;
cmake configure;
cmake build;
ctest;
clang-format check futuramente;
```

---

## 8.9 kernwerk-diagnostics

Responsável por:

```text
tipos de diagnóstico;
severidade;
origem;
mensagens;
localização arquivo/linha/coluna;
relatórios;
logs estruturados.
```

---

## 8.10 kernwerk-test-support

Responsável por:

```text
fixtures;
temp dirs;
mock toolchains;
helpers de teste;
golden tests;
asserts reutilizáveis.
```

---

## 9. Dependências Rust iniciais

Usar poucas dependências no MVP.

Recomendadas:

```toml
anyhow = "1"
thiserror = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
clap = { version = "4", features = ["derive"] }
tracing = "0.1"
tracing-subscriber = "0.3"
camino = "1"
schemars = "0.8"
```

Observações:

```text
anyhow pode ser usado na CLI.
thiserror deve ser preferido em crates de biblioteca.
camino ajuda com UTF-8 paths.
schemars pode gerar schemas JSON a partir dos tipos Rust.
```

Evitar no MVP:

```text
framework async complexo sem necessidade;
banco de dados;
plugin runtime;
dependências de UI;
dependências experimentais;
crates abandonados.
```

---

## 10. Tipos centrais

## 10.1 ToolStatus

```rust
pub enum ToolStatus {
    NotConfigured,
    Missing,
    Detected,
    Ready,
    Running,
    Failed,
    Disabled,
}
```

## 10.2 ToolKind

```rust
pub enum ToolKind {
    Git,
    Cargo,
    Rustc,
    Rustfmt,
    Clippy,
    RustAnalyzer,
    CMake,
    Ninja,
    Clang,
    Clangxx,
    Clangd,
    ClangFormat,
    ClangTidy,
    Gdb,
    Lldb,
}
```

## 10.3 CommandSafety

```rust
pub enum CommandSafety {
    Safe,
    ModifiesFiles,
    Dangerous,
    ForbiddenByDefault,
}
```

## 10.4 ProjectKind

```rust
pub enum ProjectKind {
    Unknown,
    CMake,
    Cargo,
    MixedCMakeCargo,
}
```

## 10.5 DiagnosticSeverity

```rust
pub enum DiagnosticSeverity {
    Info,
    Warning,
    Error,
    Fatal,
}
```

---

## 11. Command Registry

Todo comando deve ter metadados.

Exemplo conceitual:

```json
{
  "id": "quality.runAll",
  "title": "Rodar verificação completa",
  "category": "Quality",
  "description": "Executa a pipeline de qualidade configurada para o workspace atual.",
  "requiresWorkspace": true,
  "safety": "Safe",
  "runsInBackground": true,
  "canBeCalledByAI": true,
  "requiresConfirmation": false
}
```

Comandos iniciais obrigatórios:

```text
core.ping
core.shutdown
tools.detect
tools.status
workspace.open
workspace.status
config.validate
quality.profile.show
quality.profile.apply
quality.runAll
quality.report.show
```

---

## 12. IPC / JSON-RPC futuro

Mesmo que o MVP use CLI primeiro, os tipos devem já ser compatíveis com IPC.

Request:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "core.ping",
  "params": {}
}
```

Response:

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

Error:

```json
{
  "jsonrpc": "2.0",
  "id": 2,
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

Evento:

```json
{
  "jsonrpc": "2.0",
  "method": "event.tool.statusChanged",
  "params": {
    "tool": "clangd",
    "status": "Ready"
  }
}
```

---

## 13. Schemas JSON obrigatórios

## 13.1 quality-profile.json

Local:

```text
.kernwerk/quality-profile.json
```

Exemplo:

```json
{
  "schemaVersion": 1,
  "profile": "strict-iso-pedantic",
  "languages": {
    "cpp": {
      "standard": "c++23",
      "compilerExtensions": false,
      "warningsAsErrors": true,
      "pedanticErrors": true,
      "sanitizers": {
        "address": true,
        "undefinedBehavior": true,
        "thread": false,
        "leak": false
      },
      "staticAnalysis": {
        "clangTidy": "recommended",
        "cppcheck": false,
        "includeWhatYouUse": false
      },
      "formatting": {
        "clangFormat": true
      },
      "testing": {
        "ctest": true,
        "framework": "catch2"
      }
    },
    "rust": {
      "denyWarnings": true,
      "forbidUnsafe": true,
      "fmtCheck": true,
      "clippy": true,
      "tests": true,
      "cargoDeny": false,
      "cargoAudit": false
    }
  }
}
```

---

## 13.2 toolchains.json

Local:

```text
.kernwerk/toolchains.json
```

Exemplo:

```json
{
  "schemaVersion": 1,
  "active": "system-clang",
  "toolchains": [
    {
      "id": "system-clang",
      "name": "System Clang",
      "type": "cpp",
      "cCompiler": "/usr/bin/clang",
      "cppCompiler": "/usr/bin/clang++",
      "cmake": "/usr/bin/cmake",
      "ninja": "/usr/bin/ninja",
      "debugger": "/usr/bin/lldb",
      "languageServer": "/usr/bin/clangd"
    },
    {
      "id": "system-rust",
      "name": "System Rust",
      "type": "rust",
      "cargo": "/usr/bin/cargo",
      "rustc": "/usr/bin/rustc",
      "languageServer": "/usr/bin/rust-analyzer"
    }
  ]
}
```

---

## 13.3 workspace.json

Local:

```text
.kernwerk/workspace.json
```

Exemplo:

```json
{
  "schemaVersion": 1,
  "name": "kernwerk-studio",
  "root": ".",
  "projectKind": "MixedCMakeCargo",
  "activeQualityProfile": "strict-iso-pedantic",
  "activeRunConfig": "cargo-check",
  "uiStateFile": ".kernwerk/ui-state.json"
}
```

---

## 13.4 run-configs.json

Local:

```text
.kernwerk/run-configs.json
```

Exemplo:

```json
{
  "schemaVersion": 1,
  "active": "cargo-check",
  "configs": [
    {
      "id": "cargo-check",
      "name": "Cargo Check",
      "type": "cargo",
      "command": "check",
      "args": ["--workspace", "--all-targets", "--all-features"],
      "workingDirectory": "."
    },
    {
      "id": "cmake-debug-strict",
      "name": "CMake Debug Strict",
      "type": "cmake",
      "preset": "debug-strict",
      "workingDirectory": "."
    }
  ]
}
```

---

## 14. Tool Detection MVP

O MVP deve detectar ferramentas usando:

```text
PATH;
caminhos conhecidos;
configuração manual futura;
execução de --version.
```

Comportamento:

```text
se ferramenta existe → Detected;
se versão responde corretamente → Ready;
se não existe → Missing;
se usuário desativou → Disabled.
```

Comando:

```bash
kernwerk tools detect
```

Saída esperada em modo texto:

```text
git              Ready      /usr/bin/git
cargo            Ready      /usr/bin/cargo
rust-analyzer    Ready      /usr/bin/rust-analyzer
cmake            Ready      /usr/bin/cmake
ninja            Ready      /usr/bin/ninja
clangd           Ready      /usr/bin/clangd
gdb              Missing    -
```

Saída opcional JSON:

```json
{
  "tools": [
    {
      "kind": "cargo",
      "status": "Ready",
      "path": "/usr/bin/cargo",
      "version": "cargo 1.85.0"
    }
  ]
}
```

---

## 15. Quality Center MVP

O MVP do Quality Center não precisa ter UI.

Deve existir via CLI/Core.

Comando:

```bash
kernwerk quality run
```

Para projeto Rust:

```text
cargo fmt --all --check
cargo check --workspace --all-targets --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

Para projeto CMake:

```text
cmake --preset debug-strict
cmake --build --preset debug-strict
ctest --preset debug-strict
```

Se um comando não puder rodar porque a ferramenta está ausente:

```text
não entrar em pânico;
marcar check como Skipped ou Failed;
mostrar ferramenta ausente;
sugerir instalação;
gerar relatório.
```

---

## 16. Relatório de qualidade

Estrutura conceitual:

```json
{
  "schemaVersion": 1,
  "workspace": "kernwerk-studio",
  "profile": "strict-iso-pedantic",
  "status": "failed",
  "checks": [
    {
      "id": "rust.fmt",
      "name": "cargo fmt",
      "status": "passed",
      "durationMs": 1200
    },
    {
      "id": "rust.clippy",
      "name": "cargo clippy",
      "status": "failed",
      "durationMs": 8300,
      "diagnostics": [
        {
          "severity": "error",
          "message": "unwrap_used is denied",
          "file": "crates/kernwerk-core/src/lib.rs",
          "line": 42,
          "column": 10
        }
      ]
    }
  ]
}
```

Status possíveis:

```text
passed
failed
skipped
cancelled
```

---

## 17. Política de arquivos gerados no MVP

O MVP pode criar arquivos novos, mas deve ter cuidado com sobrescrita.

Regra obrigatória:

```text
Se o arquivo já existe, não sobrescrever sem flag explícita ou confirmação futura.
```

No CLI, usar:

```text
--dry-run
--write
--force
```

Comportamento:

```text
--dry-run: mostra o que faria;
--write: escreve arquivos novos, não sobrescreve existentes;
--force: pode sobrescrever, mas deve ser usado conscientemente.
```

---

## 18. Logs locais

Diretórios:

```text
~/.cache/kernwerk-studio/logs/
```

Logs mínimos:

```text
core.log
tools.log
quality.log
commands.log
```

Regras:

```text
não logar API keys;
não logar tokens;
não logar conteúdo completo de arquivos sem necessidade;
não enviar logs para lugar nenhum.
```

---

## 19. Testes obrigatórios por crate

## 19.1 kernwerk-protocol

```text
serialização de requests;
serialização de responses;
serialização de erros;
compatibilidade de eventos.
```

## 19.2 kernwerk-config

```text
parse de quality-profile;
parse de toolchains;
schemaVersion obrigatório;
config inválida gera erro claro;
defaults funcionam;
```

## 19.3 kernwerk-command

```text
registro de comando;
comando duplicado falha;
permissão correta;
comando inexistente gera erro;
```

## 19.4 kernwerk-tooling

```text
detecção de binário inexistente;
detecção de binário existente;
parse de versão;
status Missing/Ready;
não entrar em pânico quando comando falha.
```

## 19.5 kernwerk-project

```text
detecta Cargo.toml;
detecta CMakeLists.txt;
detecta projeto misto;
ignora diretórios build/ target/ .git/;
```

## 19.6 kernwerk-quality

```text
monta pipeline Rust;
monta pipeline CMake;
lida com ferramenta ausente;
gera relatório;
status final correto;
```

---

## 20. Golden tests

Templates devem ter golden tests.

Estrutura:

```text
tests/golden/
├── cpp-console-strict/
│   ├── expected/
│   └── actual/
└── rust-cli-strict/
    ├── expected/
    └── actual/
```

Regra:

```text
Toda alteração em template deve ser intencional e revisável.
```

---

## 21. Milestones

## Milestone 0 — Repositório base

Objetivo:

```text
criar workspace Rust rígido;
criar docs;
criar crates vazios;
configurar fmt/clippy/test.
```

Critérios de aceite:

```text
cargo fmt --all --check passa;
cargo check --workspace --all-targets --all-features passa;
cargo clippy --workspace --all-targets --all-features -- -D warnings passa;
cargo test --workspace --all-features passa;
README existe;
AGENTS.md existe;
docs principais existem.
```

---

## Milestone 1 — Protocol + CLI ping

Objetivo:

```text
implementar kernwerk-cli;
implementar kernwerk-protocol;
implementar core.ping.
```

Comando:

```bash
kernwerk ping
```

Saída esperada:

```text
kernwerk: pong
```

Critérios de aceite:

```text
CLI compila;
ping funciona;
testes de serialização passam;
nenhum unwrap/expect/panic fora de testes;
logs básicos funcionam.
```

---

## Milestone 2 — Command System

Objetivo:

```text
implementar command registry;
registrar comandos básicos;
validar permissões.
```

Comandos:

```text
core.ping
tools.detect
workspace.open
quality.runAll
config.validate
```

Critérios de aceite:

```text
comandos registrados;
comando inexistente retorna erro estruturado;
comando duplicado falha em teste;
metadados de segurança existem.
```

---

## Milestone 3 — Config Manager

Objetivo:

```text
ler e validar configs .kernwerk;
schemaVersion;
quality-profile básico;
toolchains básico.
```

Critérios de aceite:

```text
config válida carrega;
config inválida gera erro claro;
schemaVersion ausente gera erro claro;
defaults documentados;
testes passam.
```

---

## Milestone 4 — Tool Detection

Objetivo:

```text
detectar ferramentas locais;
mostrar status;
validar --version.
```

Comando:

```bash
kernwerk tools detect
```

Critérios de aceite:

```text
detecta pelo menos git, cargo, rustc, cmake, ninja, clangd quando presentes;
ferramenta ausente não quebra;
saída texto legível;
saída JSON opcional;
logs locais.
```

---

## Milestone 5 — Workspace + Project Detection

Objetivo:

```text
abrir workspace;
detectar Cargo/CMake;
montar ProjectKind.
```

Comando:

```bash
kernwerk workspace open .
```

Critérios de aceite:

```text
detecta Cargo.toml;
detecta CMakeLists.txt;
detecta projeto misto;
ignora target/, build/, .git/;
gera workspace status.
```

---

## Milestone 6 — Quality Runner Rust

Objetivo:

```text
rodar pipeline Rust inicial.
```

Comando:

```bash
kernwerk quality run
```

Checks:

```text
cargo fmt --all --check
cargo check --workspace --all-targets --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

Critérios de aceite:

```text
relatório estruturado;
status passed/failed/skipped;
diagnóstico mínimo;
ferramenta ausente tratada corretamente.
```

---

## Milestone 7 — Quality Runner CMake/C++

Objetivo:

```text
rodar pipeline CMake inicial.
```

Checks:

```text
cmake configure;
cmake build;
ctest;
clang-format check futuro;
```

Critérios de aceite:

```text
usa CMakePresets quando disponíveis;
mostra erro claro quando preset não existe;
não assume estrutura inexistente;
relatório estruturado.
```

---

## Milestone 8 — Templates Strict iniciais

Objetivo:

```text
gerar templates C++ e Rust strict.
```

Templates:

```text
cpp-console-strict;
cpp-library-strict;
rust-cli-strict;
rust-workspace-strict.
```

Critérios de aceite:

```text
templates geram arquivos;
não sobrescrevem sem --force;
golden tests passam;
projetos gerados passam quality run.
```

---

## Milestone 9 — IPC local mínimo

Objetivo:

```text
preparar JSON-RPC local para futura UI.
```

Critérios de aceite:

```text
core.ping via JSON-RPC;
tools.detect via JSON-RPC;
quality.runAll via JSON-RPC;
erros estruturados;
eventos básicos.
```

---

## Milestone 10 — UI Qt/QML mínima

Objetivo:

```text
janela inicial;
status do core;
botão tools detect;
botão quality run;
painel de output.
```

Critérios de aceite:

```text
UI não trava durante comando;
core roda separado;
output aparece;
erro aparece;
sem editor avançado ainda.
```

---

## 22. Regras para agentes de IA

Ao implementar com Claude/Codex/GPT, seguir:

```text
Não usar Electron.
Não criar telemetria.
Não implementar compilador próprio.
Não implementar debugger próprio.
Não implementar LSP próprio.
Não iniciar ferramentas pesadas no boot.
Não adicionar Java/Python agora.
Não criar plugin marketplace agora.
Não sobrescrever arquivos sem confirmação.
Não usar crates experimentais sem necessidade.
Não usar unwrap/expect/panic fora de testes.
Não usar unsafe.
Não misturar UI no core.
Não criar arquitetura gigante antes do MVP.
```

Foco do agente:

```text
implementar uma etapa por vez;
rodar testes;
manter código pequeno;
manter docs atualizadas;
não antecipar recursos avançados.
```

---

## 23. Prompt recomendado para iniciar no terminal

```text
Você está no projeto Kernwerk Studio.

Leia primeiro:

AGENTS.md
docs/PROFESSIONAL_IDE_STANDARDS.md
docs/IMPLEMENTATION_BLUEPRINT.md
docs/TOOLCHAIN_LAYER_POLICY.md
docs/QUALITY_CENTER_STRICT_MODES.md
docs/CPP_RUST_NATIVE_TOOLING.md

Implemente apenas o Milestone 0 e Milestone 1.

Objetivo:
- criar Rust workspace;
- criar crates iniciais;
- configurar rust-toolchain;
- configurar lint strict;
- implementar kernwerk-cli;
- implementar kernwerk-protocol mínimo;
- implementar core.ping;
- adicionar testes.

Não implemente Qt ainda.
Não implemente LSP ainda.
Não implemente CMake ainda.
Não implemente Git profundo ainda.
Não implemente IA ainda.
Não adicione dependências desnecessárias.
Não use unsafe.
Não use unwrap/expect/panic fora de testes.

Ao final, rode:

cargo fmt --all --check
cargo check --workspace --all-targets --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

---

## 24. Critério para considerar o Core inicial saudável

O core inicial está saudável quando:

```text
compila rápido;
tem poucos crates claros;
tem testes;
tem command registry;
tem config parsing;
tem tool detection;
tem quality runner inicial;
não depende de UI;
não depende de IA;
não tem telemetria;
não executa comandos perigosos;
não usa unsafe;
não usa unwrap/expect/panic fora de testes;
gera erros claros.
```

---

## 25. Critério para avançar para UI

Só avançar para Qt/QML quando:

```text
core.ping funciona;
command registry funciona;
tools.detect funciona;
workspace.open funciona;
quality.run funciona;
configs são validadas;
testes passam;
logs locais existem;
erros são estruturados;
JSON-RPC mínimo está definido ou preparado.
```

---

## 26. Critério para dogfooding

O Kernwerk começa a fazer dogfooding quando consegue:

```text
abrir o próprio repositório;
detectar Cargo workspace;
rodar cargo check;
rodar cargo test;
rodar cargo clippy;
mostrar relatório de qualidade;
editar pelo menos arquivos simples futuramente;
executar comandos sem terminal externo futuramente.
```

Marco final da primeira grande fase:

```text
Kernwerk Studio can build Kernwerk Studio.
```

---

## 27. Conclusão

Este blueprint deve guiar a implementação inicial.

A regra principal é:

```text
Começar pequeno, mas com fundação profissional.
```

O Kernwerk Studio deve nascer como uma IDE pessoal, livre, séria e extensível, focada inicialmente em C/C++/Rust, usando ferramentas abertas profissionais, com strict mode visual, toolchains locais, zero telemetria e capacidade futura de desenvolver a si própria.

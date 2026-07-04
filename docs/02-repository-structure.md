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

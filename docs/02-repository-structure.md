# 02 — Estrutura do Repositório

## Estrutura inicial recomendada

```text
kinein-vectis/
├── README.md
├── AGENTS.md
├── Cargo.toml
├── rust-toolchain.toml
├── deny.toml
├── .gitignore
├── .editorconfig
│
├── crates/
│   ├── kinein-core/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs          # binary entry (calls run_stdio)
│   │       ├── lib.rs           # Core struct + handle_request dispatch + re-exports
│   │       ├── runtime.rs       # stdio JSON-RPC loop (run_stdio / run_json_lines)
│   │       ├── rpc.rs           # JSON-RPC error responses + param parsing
│   │       ├── commands.rs      # command.list descriptors
│   │       ├── tools.rs process.rs build.rs test.rs run.rs terminal.rs  # domain services
│   │       ├── handlers/        # request routers by domain (impl Core blocks)
│   │       │   └── workspace.rs fs.rs lsp.rs run.rs terminal.rs build.rs
│   │       ├── lsp/             # LSP client subsystem
│   │       │   └── mod.rs types.rs manager.rs server.rs framing.rs parse.rs edit.rs uri.rs
│   │       ├── fsops/           # workspace-confined filesystem operations
│   │       │   └── mod.rs error.rs confine.rs ops.rs search.rs find.rs
│   │       ├── workspace/       # open / detect / browse / create + persistence
│   │       │   └── mod.rs error.rs detect.rs open.rs create.rs
│   │       └── tests/           # integration tests grouped by domain
│   │           └── mod.rs dispatch.rs tools.rs workspace.rs fs.rs run.rs build.rs lsp.rs
│   │
│   ├── kinein-protocol/
│   │   ├── Cargo.toml
│   │   └── src/                 # per-domain modules re-exported flat from lib.rs
│   │       └── lib.rs rpc.rs command.rs core.rs tools.rs workspace.rs fs.rs run.rs terminal.rs lsp.rs build.rs
│   │
│   ├── kinein-config/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs
│   │
│   └── kinein-cli/
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs          # thin binary shim over kinein_cli::run
│           ├── lib.rs           # library target: re-exports run + CliError
│           ├── commands.rs      # argv → JSON-RPC request dispatch
│           └── error.rs         # CliError
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
name = "kinein-core"
```

No código Rust, o crate será referenciado como:

```rust
use kinein_core::...
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
    "crates/kinein-core",
    "crates/kinein-protocol",
    "crates/kinein-config",
    "crates/kinein-cli",
]

[workspace.package]
edition = "2024"
license = "MIT OR Apache-2.0"
repository = "https://github.com/SEU_USUARIO/kinein-vectis"
homepage = "https://github.com/SEU_USUARIO/kinein-vectis"
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

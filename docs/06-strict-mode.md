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

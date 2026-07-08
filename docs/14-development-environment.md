# 14 — Ambiente de desenvolvimento

O ambiente local deve refletir o modo de segurança máximo do projeto.

## Rust

Toolchain definido em `rust-toolchain.toml`:

```bash
rustup run stable cargo --version
rustup run stable cargo kw-fmt
rustup run stable cargo kw-check
rustup run stable cargo kw-clippy
rustup run stable cargo kw-test
```

Os aliases `kw-*` estão em `.cargo/config.toml`.

## C++/Qt/QML

Dependências base no Arch/CachyOS:

```bash
sudo pacman -S cmake ninja clang lldb gdb qt6-base qt6-declarative qt6-tools
```

Presets previstos:

```bash
cmake --preset linux-clang-debug-strict
cmake --build --preset debug-strict
ctest --preset debug-strict

cmake --preset linux-clang-release-hardened
cmake --build --preset release-hardened
```

Todo alvo C++/Qt deve chamar:

```cmake
include(cmake/KineinStrictOptions.cmake)
kinein_enable_strict_compiler_options(nome_do_target)
```

## Estado observado no ambiente atual (Arch, 2026-07-08)

- Rust 1.96.1, rustfmt e clippy disponíveis via `rustup run stable`.
- CMake 4.3.4 e Ninja disponíveis.
- GCC 16.1.1; `clang++`, `clang-format`, `clang-tidy` e `clangd` 22.1.6
  disponíveis no PATH.
- Qt 6.11.1 em `/usr/lib` (qt6-base, qt6-declarative, qt6-tools).
- Gate completo `scripts/verificar.sh` verde nesta máquina em 2026-07-08.

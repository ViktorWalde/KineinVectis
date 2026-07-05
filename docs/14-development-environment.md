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

## Estado observado no ambiente atual

- Rust 1.96.1, rustfmt e clippy disponíveis via `rustup run stable`.
- CMake e Ninja disponíveis.
- `clang++`, `clang-format` e `clang-tidy` disponíveis.
- Qt 6 CLI e `clangd` ainda não apareceram no PATH pelos nomes testados.

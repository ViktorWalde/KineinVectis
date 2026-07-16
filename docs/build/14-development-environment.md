# 14 — Ambiente de desenvolvimento

O ambiente local deve refletir o modo de segurança máximo do projeto.

## Bootstrap ao trocar de distro/máquina

`scripts/instalar-ambiente.sh` instala todas as dependências abaixo com
detecção de distro (pacman no Arch/CachyOS — alvo principal —, apt no
Debian/Ubuntu, dnf no Fedora), aceita `--dry-run` (só mostra os comandos) e
`--extras` (shellcheck + cargo-deny, degraus da escada de rigor de
`docs/diario/18-daily-driver-plan.md`), e termina verificando ferramenta por
ferramenta o mesmo conjunto que o ToolDetector do core reporta. Depois dele:
`cmake --preset dev-local && cmake --preset dev-local-release` e
`scripts/verificar.sh`. Lembrete: build dir de outra distro é lixo — apagar e
reconfigurar (ver `ContextoIA.md`, seção "Ambiente revalidado").

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

## Medição de performance (`scripts/medir-performance.sh`)

Mede, 100% local (offscreen + stdio + `/proc`, **zero telemetria/rede**),
startup da UI, `workspace.open`, `fs.read` de arquivo grande e RSS
(UI + core + LSP). Cada métrica roda `KINEIN_PERF_N` vezes (default 5) e
reporta a mediana; imprime a tabela e sai. Precisa dos binários compilados
(`cmake --build build/dev-local` + `cargo build -p kinein-core`). O
ORÇAMENTO (números-alvo) e as métricas manuais (latência de digitação)
vivem em `docs/roadmaps/21`, seção "M4.2". Detalhe do gancho `KINEIN_PERF_MARKER`
(marker env-gated na `main.cpp`, sem efeito no uso normal): `docs/diario/18`,
"Fatia M4.2".

## Estado observado no ambiente atual (Arch, 2026-07-08)

- Rust 1.96.1, rustfmt e clippy disponíveis via `rustup run stable`.
- CMake 4.3.4 e Ninja disponíveis.
- GCC 16.1.1; `clang++`, `clang-format`, `clang-tidy` e `clangd` 22.1.6
  disponíveis no PATH.
- Qt 6.11.1 em `/usr/lib` (qt6-base, qt6-declarative, qt6-tools).
- Gate completo `scripts/verificar.sh` verde nesta máquina em 2026-07-08.

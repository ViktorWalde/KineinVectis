# 14 — Ambiente de desenvolvimento

O ambiente local deve refletir o modo de segurança máximo do projeto.

## Bootstrap ao trocar de distro/máquina

`scripts/instalar-ambiente.sh` instala todas as dependências abaixo com
detecção de distro (pacman no Arch/CachyOS — alvo principal —, apt no
Debian/Ubuntu, dnf no Fedora), aceita `--dry-run` (só mostra os comandos) e
`--extras` (shellcheck, degrau da escada de rigor de
`docs-privada/diario/18-daily-driver-plan.md`; o `cargo-deny` saiu de `--extras`
em 2026-08-30 e virou dependência BASE, porque o gate passou a depender dele), e termina verificando ferramenta por
ferramenta o mesmo conjunto que o ToolDetector do core reporta. Depois dele:
`cmake --preset dev-local && cmake --preset dev-local-release` e
`scripts/verificar.sh`. Lembrete: build dir de outra distro é lixo — apagar e
reconfigurar (ver `docs-privada/ContextoIA.md`, seção "Ambiente revalidado").

## Rust

Toolchain definido em `rust-toolchain.toml`:

```bash
cargo --version
cargo kw-fmt
cargo kw-check
cargo kw-clippy
cargo kw-test
```

Os aliases `kw-*` estão em `.cargo/config.toml`.

> **Não use `rustup run stable`** (corrigido em 2026-08-29). O
> `rust-toolchain.toml` fixa uma versão exata, e `rustup run stable` **força** o
> canal `stable`, ignorando o pin: você rodaria o gate num compilador diferente
> do que o projeto declara. Numa máquina onde só a toolchain fixada está
> instalada — o caso depois do `instalar-ambiente.sh` — o comando simplesmente
> falha com *"toolchain 'stable' is not installed"*. `cargo` puro já respeita o
> arquivo: é o rustup que resolve o pin por você.

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
(marker env-gated na `main.cpp`, sem efeito no uso normal): `docs-privada/diario/18`,
"Fatia M4.2".

## Estado observado no ambiente atual (Fedora 44, 2026-08-29)

A máquina trocou de distro de novo — era Arch em 2026-07-08. O registro
anterior fica em `docs-privada/ContextoIA.md`; **este bloco descreve o ambiente
de hoje**, e é o único que vale.

- Fedora Linux 44 (Workstation), kernel 7.1.
- Rust 1.96.1 (a toolchain fixada), rustfmt e clippy via `cargo` direto.
- CMake e Ninja disponíveis; Clang 22 (`clang++`, `clang-format`, `clang-tidy`,
  `clangd`), GCC, GDB, LLDB e `lldb-dap` no PATH.
- `ripgrep` e `fd` no PATH; ShellCheck instalado.
- Qt 6.11.1 **runtime** presente; os pacotes **`-devel` não estavam
  instalados** — sem eles não há `qmllint` nem `qmake6`, e os gates
  `verificar-cpp.sh`, `verificar-qml.sh` e `verificar-qml-logica.sh` não rodam.
  É o que `scripts/instalar-ambiente.sh` resolve.
- **`scripts/verificar.sh` completo: VERDE em 2026-09-04**, depois de
  `scripts/instalar-ambiente.sh` — inclui `verificar-cpp.sh` (clang-format +
  clang-tidy), `verificar-qml.sh` (qmllint estrito) e `verificar-qml-logica.sh`
  (24 harnesses headless, medido em 2026-09-04; `ls scripts/qml-harness/ | wc -l`).
  Desde 2026-09-04 o runner monta um **espelho plano** do módulo
  `KineinVectis` a partir das fontes, então o harness também alcança
  componente **visual** — o que antes era impossível, porque o módulo só
  existia dentro do `qrc` do binário compilado.

> **Armadilha do `qmllint`, medida em 2026-08-29.** O response file do lint
> nasce na **configuração**; o `kinein-vectis.qmltypes` do módulo só na
> **compilação**. Um build dir configurado e não compilado faz o qmllint
> despejar uma parede de *"QML types file does not exist"*, que parece defeito
> no QML e é build faltando. O `verificar-qml.sh` passou a exigir os dois e a
> dizer qual `cmake --build` rodar.

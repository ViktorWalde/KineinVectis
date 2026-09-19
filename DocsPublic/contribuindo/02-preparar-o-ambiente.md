# 02 — Preparar o ambiente

A referência completa é [`../build/14-development-environment.md`](../build/14-development-environment.md)
e [`../build/como-executar.md`](../build/como-executar.md); os comandos
oficiais, [`../build/comandos-de-build-e-verificacao.md`](../build/comandos-de-build-e-verificacao.md).
Este capítulo é o caminho mais curto até "a IDE abre pelo meu checkout e o
gate está verde".

## O que precisa estar na máquina

- **Rust** pela `rust-toolchain.toml` do repositório (rustup respeita o
  arquivo; a versão exata está lá — não instale outra à mão).
- **Qt 6.10+** (Quick, Qml, QuickControls2) e **CMake ≥ 3.28** com Ninja.
- **Clang/LLVM** (clang, clang++, clang-tidy, clang-format, lldb) — os
  presets oficiais são clang; o gate C++ é clang-tidy.
- Python 3 (para os scripts dos gates), `shellcheck`, `qmllint` (vem com o
  Qt), `cargo-deny`.
- Para exercitar integrações: clangd, rust-analyzer, basedpyright, ruff,
  git, docker ou podman, qemu-system-arm, gdb com DAP. **Nenhuma é
  obrigatória para compilar** — os gates que dependem delas dizem o que
  falta e pulam com motivo.

`bash scripts/instalar-ambiente.sh` conhece as distros suportadas e
**imprime** os comandos por distro; leia e execute você. (É a mesma regra
da IDE: nada de `sudo` escondido.)

## Compilar e abrir

```bash
cargo build -p kinein-core                  # o core (debug)
cmake --preset dev-local && cmake --build --preset dev-local
./scripts/kinein-vectis                     # abre a IDE pelo checkout
./scripts/kinein-vectis /caminho/do/projeto # já com um workspace aberto
bash scripts/instalar-atalho.sh             # atalho "Kinein Vectis (Desenvolvimento)"
```

Os presets, e quando usar cada um:

| Preset (configure / build) | Para quê |
| --- | --- |
| `dev-local` / `dev-local` | o dia a dia: debug, **sem** sanitizers, compila rápido |
| `dev-local-release` / `dev-local-release` | medir desempenho no seu ambiente |
| `linux-clang-debug-strict` / `debug-strict` | o que o gate compila: `-Werror`, sanitizers, o qmllint lê o módulo daqui |
| `linux-clang-release-hardened` / `release-hardened` | o binário que vai no AppImage; os números oficiais de tempo |

O nome do preset de **configure** e o de **build** diferem nos dois
últimos (`cmake --preset linux-clang-debug-strict` e depois
`cmake --build --preset debug-strict`). O erro mais comum de quem chega é
usar o nome de configure no `--build`.

O core de release para medir a IDE de verdade:
`cargo build --release -p kinein-core` e
`KINEIN_CORE_BIN=$PWD/target/release/kinein-core ./build/<preset>/ui/kinein-vectis`.

## Rodar o gate

```bash
cargo fmt --all
bash scripts/verificar.sh --rapido   # lint + testes + C++ (sem os builds)
bash scripts/verificar.sh            # completo: + builds debug/release + o binário abre
```

O gate **para no primeiro erro** — de propósito. O `verificar-cpp.sh`
(clang-tidy) leva **~1 h** numa máquina comum; rode-o em segundo plano
com a saída num arquivo (`bash scripts/verificar-cpp.sh > /tmp/cpp.log
2>&1 &`) e **não compile a UI enquanto ele roda** (os dois disputam o
mesmo build dir). Os demais gates levam segundos a poucos minutos; a
lista, um por um, está em [04](04-os-gates-que-dizem-nao.md).

## As variáveis de ambiente que ajudam a medir

| Variável | Efeito |
| --- | --- |
| `QT_QPA_PLATFORM=offscreen` | abre sem tela (fotos, gates) |
| `KINEIN_SCREENSHOT=<png>` `KINEIN_SCREENSHOT_DELAY_MS=<ms>` `KINEIN_SCREENSHOT_SIZE=WxH` | fotografa a janela e sai |
| `KINEIN_STARTUP_COMMANDS=<ids da paleta>` | executa comandos depois do workspace abrir (`git.log`, `probe.list=kit`, `index.symbols=parse_`, `container.list`…) |
| `KINEIN_PERF_EXIT=1` `KINEIN_PERF_MARKER` | tempo até o primeiro frame e sai |
| `KINEIN_PERF_TYPING_WORKSPACE/_FILE/_KEYS` | o harness de latência da tecla |
| `KINEIN_CORE_BIN=<caminho>` | qual core a UI sobe (o release, ou um logger de IPC) |
| `XDG_CONFIG_HOME=<pasta>` | isola as configurações (`settings.json` é achatado: `{"schemaVersion":1,"railExpanded":true}`) |

## O que nunca fazer no checkout

- `git checkout -- <arquivo>` para "desfazer": se o arquivo tinha
  trabalho não commitado, ele some sem aviso. Use `git stash`, `git diff`
  e edição — e leia o `40` §7.74 antes de discordar.
- Commitar sem `cargo fmt --all` e sem o gate rápido verde.
- Rodar a UI enquanto o `verificar-cpp.sh` compila no mesmo build dir.
- Mexer em `/dev/ttyUSB*` de placa que não é sua: os gates de embarcado
  rodam no QEMU justamente para não precisar de placa.

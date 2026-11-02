# 14 — Ambiente de desenvolvimento

> **Classe: ESTADO.** Os requisitos efetivos são definidos por
> `rust-toolchain.toml`, `Cargo.toml`, `CMakeLists.txt` e `cmake/KineinQt.cmake`.
> Comandos de validação: [build e verificação](comandos-de-build-e-verificacao.md).
> Execução da IDE: [como executar](como-executar.md).

## Dependências

- Rust pelo rustup, respeitando o pin e os componentes de `rust-toolchain.toml`.
  Não force `rustup run stable`: isso ignora a toolchain do projeto.
- CMake ≥ 3.25, Ninja e Clang/LLVM, incluindo clang-tidy, clang-format e
  compiler-rt para ASan/UBSan nos builds estritos.
- Qt **6.12**, com Core, Gui, Qml, Quick, QuickControls2, Test e suporte SVG.
  Outra série é recusada por `cmake/KineinQt.cmake`.
- unixODBC de desenvolvimento: `unixodbc` no Arch/CachyOS,
  `unixodbc-dev` no Debian/Ubuntu ou `unixODBC-devel` no Fedora.
  Drivers de banco não são necessários para compilar.
- Git, Python 3, ShellCheck, qmllint e cargo-deny para os gates.
- Xvfb, xdotool, libXtst e Pillow para automação visual. QtWebEngine é
  opcional, usado pela integração web local.

O bootstrap reconhece Arch/CachyOS, Debian/Ubuntu e Fedora. Inspecione o plano
antes de executar a instalação; sem `--dry-run`, ele instala pacotes:

```bash
bash scripts/instalar-ambiente.sh --dry-run
bash scripts/instalar-ambiente.sh
```

O script também prepara o PATH do Rust e os presets locais ausentes. Instalar
pacotes de sistema exige os privilégios correspondentes. O pin do auditor da
CI pública pode ser reproduzido com:

```bash
cargo install cargo-deny --version 0.20.2 --locked
```

## Escolher o Qt

No Arch/CachyOS, o Qt do sistema pode ser usado quando sua série for 6.12.
Em outras instalações, passe `-DKINEIN_QT_PREFIX=/caminho/do/qt` ou instale em
`~/Qt/6.12.x/gcc_64`; o CMake procura o prefixo mais recente dessa série.
Não misture bibliotecas do sistema com ferramentas de outro prefixo Qt.
Cada build grava `kinein-qt-bins.txt`, usado pelos gates de QML.

Para uma instalação oficial com navegador embutido, consulte os módulos do
instalador antes do download. Exemplo com aqtinstall:

```bash
aqt list-qt linux desktop --modules 6.12.0 linux_gcc_64
aqt install-qt linux desktop 6.12.0 linux_gcc_64 \
  -m qtwebengine qtwebchannel qtpositioning -O "$HOME/Qt"
```

Ao trocar distro, compilador ou Qt, reconfigure os builds. Quando houver
objetos compilados com bibliotecas anteriores, use uma reconstrução limpa;
a conferência de objetos obsoletos está em `scripts/verificar-binario-abre.sh`.

## Presets locais

`CMakeUserPresets.json` é ignorado pelo Git e não vem no clone. O bootstrap
o gera quando ausente. Para criá-lo manualmente, use o exemplo abaixo;
ele herda Clang dos presets versionados. Preserve configurações já existentes.

```json
{
  "version": 6,
  "configurePresets": [
    {
      "name": "dev-local",
      "inherits": "linux-clang-debug-strict",
      "binaryDir": "${sourceDir}/build/dev-local",
      "cacheVariables": { "KINEIN_ENABLE_SANITIZERS": "OFF" }
    },
    {
      "name": "dev-local-release",
      "inherits": "linux-clang-release-hardened",
      "binaryDir": "${sourceDir}/build/dev-local-release"
    }
  ],
  "buildPresets": [
    { "name": "dev-local", "configurePreset": "dev-local" },
    { "name": "dev-local-release", "configurePreset": "dev-local-release" }
  ]
}
```

```bash
cmake --preset dev-local
cmake --preset dev-local-release
cmake --preset linux-clang-debug-strict
cargo build --locked -p kinein-core -p kinein-adapter-sqlite --jobs 2
cmake --build --preset dev-local --parallel 2
ctest --test-dir build/dev-local --output-on-failure
./scripts/kinein-vectis
```

O limite de dois jobs é um exemplo conservador; ajuste-o à RAM disponível.
O gate calcula o paralelismo automaticamente. Os aliases Rust de formatação,
check, Clippy e testes estão em `.cargo/config.toml` (`cargo kw-*`).

## Recursos de memória e temporários

O gate tenta criar um escopo systemd de usuário com `MemorySwapMax=0` e teto
pela RAM disponível menos 2 GiB de reserva. Confira o cabeçalho da rodada:
sem escopo, o isolamento não está comprovado. O limite vale para o escopo;
não altera a configuração de swap de outros programas.

Para um build fora do gate, é possível aplicar a mesma política. Este exemplo
usa teto de 8 GiB: escolha o teto pela RAM disponível, preservando a reserva
para o desktop, e ajuste o paralelismo antes de executar.

```bash
systemd-run --user --scope --collect -p MemorySwapMax=0 -p MemoryMax=8G \
  cmake --build --preset dev-local --parallel 2
```

Use `TMPDIR` em disco para provas grandes, especialmente se `/tmp` for tmpfs.
Cada rodada e cada binário de teste limpam seus temporários. Para QA isolado,
defina um `CARGO_TARGET_DIR` próprio. O contrato completo de recursos e as
variáveis de controle estão em
[comandos de verificação](comandos-de-build-e-verificacao.md#recursos-da-rodada-só-ram-temporários-e-paralelismo-2026-10-24).

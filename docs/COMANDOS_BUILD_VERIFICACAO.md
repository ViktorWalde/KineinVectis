# Comandos de build e verificacao

## Gate unico (recomendado)

Um comando so, que roda tudo em sequencia e **para no primeiro erro**. Isso
evita o caso em que um passo falha (ex.: `clang-format`) mas os seguintes
continuam e dao falsa sensacao de "tudo passou".

```bash
cargo fmt --all          # formate primeiro (o gate apenas CHECA a formatacao)
scripts/verificar.sh     # completo: lint + testes + C++ + builds debug/release
```

Modo rapido para iteracao (sem builds):

```bash
scripts/verificar.sh --rapido
```

O modo **completo** e o que bloqueia release: os binarios do icone
(`build/linux-clang-release-hardened/ui/kinein-vectis` e
`target/release/kinein-core`) so sao atualizados se lint, testes, C++ e
builds passarem. Presets CMake podem ser sobrescritos por ambiente:
`KINEIN_PRESET_DEBUG` / `KINEIN_PRESET_RELEASE` (padrao `dev-local*`).

## Sequencia manual (referencia)

O gate acima equivale a rodar, nesta ordem, parando no primeiro erro:

```bash
cargo fmt --all --check
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
scripts/verificar-cpp.sh
scripts/verificar-qml.sh
cmake --build --preset dev-local
cargo build --release -p kinein-core
cmake --build --preset dev-local-release
```

`scripts/verificar-qml.sh` roda o qmllint em modo estrito (zero warnings)
com o contexto de modulo do build debug; se um `.qml` novo nao aparecer no
lint, reconfigure o preset debug para regenerar o response file.

Execucao manual da IDE:

```bash
./scripts/kinein-vectis
```

Quando o preset local ainda nao existir/configurar:

```bash
cmake --preset dev-local
cmake --preset dev-local-release
```

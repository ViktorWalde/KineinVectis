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

## Atualização integral sem cache antigo

Quando houver dúvida de que UI, core e cache CMake pertencem ao mesmo estado,
use o fluxo transacional único:

```bash
scripts/atualizar-tudo.sh              # = --ide
scripts/atualizar-tudo.sh --appimage   # so o AppImage portatil em dist/
scripts/atualizar-tudo.sh --tudo       # IDE primeiro; so empacota se ela ficar verde
```

**A limpeza do cache é o padrão.** Cache velho aqui não falha barulhento: ele
entrega uma IDE que parece atual e não é. Para reaproveitar os caches (mais
rápido, menos seguro), use `--sem-limpeza`.

O alvo `--ide` executa, nesta ordem:

1. lock contra duas atualizações concorrentes;
2. fingerprint das fontes e backup dos quatro executáveis atuais;
3. remoção dos diretórios de build da IDE e dos órfãos — todo `build/*` que
   preset nenhum reivindica, já resolvendo `inherits`;
4. configuração `--fresh` dos presets Clang Debug/Release oficiais;
5. rebuild `--clean-first` das duas UIs;
6. limpeza dirigida de `kinein-protocol`/`kinein-core`;
7. gate completo com `debug-strict`/`release-hardened`;
8. materialização explícita do `kinein-core` Debug, que `cargo test` não
   garante no caminho estável usado pelo fallback do launcher;
9. rejeição se as fontes mudarem durante o build;
10. smoke pelo `scripts/kinein-vectis` real;
11. manifesto com commit, fingerprint e hashes dos quatro executáveis em
   `build/kinein-build-manifest.env`.

O alvo `--appimage` limpa o staging, empacota no container Debian 12 fixado e
roda os dois smokes de entrega (host e Debian mínimo sem rede). O cache de
ferramentas (`build/appimage/cache/tools`) é preservado: ele é verificado por
SHA256 fixado a cada uso e o smoke portátil roda sem rede de propósito.

Se qualquer etapa falhar, os executáveis anteriores são restaurados e o
manifesto não é regravado. O script não faz `git pull`, `cargo update`, push
nem acesso de rede por conta própria; ele sincroniza somente o checkout local
já existente.

> Ao rodar o script por um pipe (`| tee`, `| grep`), o código de saída passa a
> ser o do último comando do pipe e a falha some. Use `set -o pipefail`, ou
> confira `build/kinein-build-manifest.env`: manifesto não regravado = não
> passou.

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

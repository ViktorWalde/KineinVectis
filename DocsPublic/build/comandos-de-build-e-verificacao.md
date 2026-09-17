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

## Requisitos que não são do Rust

`cargo test --workspace` exige **`python3` no PATH** desde 2026-09-02: os testes
do subsistema LSP sobem `scripts/fake_lsp_server.py`, um language server falso e
determinístico, e verificam o que o core FALA com ele (`didOpen`, `didChange`,
`didClose`, versão do documento). Sem ele, os testes **falham** — nunca são
pulados: teste que pula não prova nada, e essa era exatamente a lacuna que a
etapa 3 do `DocsPublic/roadmaps/30-caminho-para-o-mvp.md` fechou.

`python3` já era requisito de 9 das 22 verificações do gate de então (veracidade
dos `.md`, links, catraca de arquitetura, duplicação e alcance QML, o binário
que abre, mais as sondas; medido em 2026-09-11 — a 23ª, o ciclo de depurar
Python de 2026-09-13, também o usa), então isto não acrescenta
dependência ao ambiente — só a torna explícita para quem roda `cargo test`
sozinho. O `scripts/instalar-ambiente.sh` continua sendo o bootstrap.

## Atualização integral sem cache antigo

Quando houver dúvida de que UI, core e cache CMake pertencem ao mesmo estado,
use o fluxo transacional único:

```bash
scripts/atualizar-tudo.sh
```

Ele executa, nesta ordem:

1. lock contra duas atualizações concorrentes;
2. fingerprint das fontes e backup dos quatro executáveis atuais;
3. configuração `--fresh` dos presets Clang Debug/Release oficiais;
4. rebuild `--clean-first` das duas UIs;
5. limpeza dirigida de `kinein-protocol`/`kinein-core`;
6. gate completo com `debug-strict`/`release-hardened`;
7. materialização explícita do `kinein-core` Debug, que `cargo test` não
   garante no caminho estável usado pelo fallback do launcher;
8. rejeição se as fontes mudarem durante o build;
9. smoke pelo `scripts/kinein-vectis` real;
10. manifesto com commit, fingerprint e hashes dos quatro executáveis em
   `build/kinein-build-manifest.env`.

Se qualquer etapa falhar, os executáveis anteriores são restaurados. O script
não faz `git pull`, `cargo update`, AppImage, push nem acesso de rede por conta
própria; ele sincroniza somente o checkout local já existente.

## Sequencia manual (referencia)

O gate acima equivale a rodar, nesta ordem, parando no primeiro erro:

```bash
cargo fmt --all --check
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
scripts/verificar-deny.sh                    # licencas e advisories das deps Rust
scripts/verificar-shell.sh                   # shellcheck nos scripts do gate
scripts/verificar-appimage.sh                # invariantes da frente de distribuicao
scripts/verificar-cpp.sh                     # clang-format + clang-tidy
scripts/verificar-qml.sh                     # qmllint estrito
scripts/verificar-qml-fiacao.sh              # binding auto-referente
scripts/verificar-qml-propriedades.sh        # binding para propriedade inexistente
scripts/verificar-qml-duplicacao.sh          # mesma derivacao em dois arquivos
scripts/verificar-qml-alcance.sh             # componente entregue que nenhuma tela abre
scripts/verificar-exercitacao.sh             # o core contra ferramenta real
scripts/verificar-embarcado.sh               # ciclo de embarcado no QEMU, sem placa
scripts/verificar-python-debug.sh            # (1) a porta escolhida chega ao mpremote — run.start/
                                             # run.script { device } contra o core REAL com um mpremote
                                             # falso que ecoa os argv; sem placa, sem interpretador
                                             # (scripts/verificar_micropython_porta.py, 0.110.0);
                                             # (2) launch de arquivo/modulo e attach TCP com debugpy REAL
                                             # (python3 que importa debugpy, ou
                                             # KINEIN_PYTHON_DEBUGPY=<venv>/bin/python;
                                             # senao "nao provado", sem falhar)
scripts/verificar-clangd-cross.sh            # clangd enxerga o cross do kit
scripts/verificar-atalhos.sh                 # a paleta promete o que a IDE faz
scripts/verificar-docs.sh                    # numero sem data que mente
scripts/verificar-links-docs.sh              # link de documentacao morto
scripts/verificar-arquitetura.sh             # catraca da regra de split
scripts/verificar-transicao-workspace.sh     # estado por-workspace com um dono
scripts/verificar-qml-logica.sh              # controllers QML headless
cmake --build --preset dev-local             # UI debug (KINEIN_PRESET_DEBUG)
scripts/verificar-binario-abre.sh --preset dev-local          # o binario que saiu do build ABRE
cargo build --release -p kinein-core
cmake --build --preset dev-local-release     # UI release (KINEIN_PRESET_RELEASE)
scripts/verificar-binario-abre.sh --preset dev-local-release  # idem, release
```

Esta lista tem de bater com `scripts/verificar.sh` — o script é a fonte, e a
razão de existir de cada gate está em `DocsPublic/arquitetura/ARCHITECTURE.md` §4
regra 11 (todo gate aqui nasceu de uma falha que passou verde por todos os
outros).

`scripts/verificar-qml.sh` roda o qmllint em modo estrito (zero warnings)
com o contexto do módulo do build debug. Em Qt recente usa `.rsp` e `-W 0`;
sem `.rsp` (como no Qt 6.4), `scripts/verificar_qml.py` executa o alvo
`kinein-vectis_qmllint_json` gerado pelo Qt e valida o relatório novo. Avisos
reprovam mesmo com exit zero; JSON vazio/inválido e falha do CMake também.
Não há lista paralela de fontes ou imports. Se um `.qml` novo não aparecer,
reconfigure o preset debug. O build precisa ter gerado o `.qmltypes`.

`KINEIN_QML_RSP` e `KINEIN_QMLLINT` continuam disponíveis para o caminho
explícito de response file. Sem `.rsp`, o alvo CMake usa o binário da mesma
instalação Qt do build; um `KINEIN_QMLLINT` isolado é rejeitado.
No Ubuntu, os testes de lógica precisam do pacote `qml-qt6`: o executável
é `/usr/lib/qt6/bin/qml`, não o wrapper de Qt 5 que pode existir no PATH.

Execucao manual da IDE:

```bash
./scripts/kinein-vectis
```

Quando o preset local ainda nao existir/configurar:

```bash
cmake --preset dev-local
cmake --preset dev-local-release
```

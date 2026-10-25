# Comandos de build e verificacao

## Gate unico (recomendado)

Um comando so, que roda tudo em sequencia e **para no primeiro erro**. Isso
evita o caso em que um passo falha (ex.: `clang-format`) mas os seguintes
continuam e dao falsa sensacao de "tudo passou".

```bash
cargo fmt --all          # formate primeiro (o gate apenas CHECA a formatacao)
scripts/verificar.sh     # completo: lint + testes + C++ + builds debug/release
```

Modo rapido para iteracao (sem build release e sem os smokes; a UI de debug
compila mesmo assim, antes dos lints que leem o build):

```bash
scripts/verificar.sh --rapido
```

A camada rápida da CI pública usa o mesmo orquestrador, sem precisar de Qt
ou de presets CMake:

```bash
bash scripts/verificar.sh --ci-rapida --estrito
```

O perfil, os pré-requisitos e a cobertura estão no
[`contribuindo/07` §2.1](../contribuindo/07-fluxo-e-responsabilidades-dos-gates.md#21-a-camada-rápida-da-ci-2026-10-19).
Ele não substitui o gate local rápido estrito de cada commit.

Antes de release e em CI, acrescente `--estrito` (combina com os três perfis):

```bash
scripts/verificar.sh --estrito
```

Sem ele, gate de **ambiente** que nao tem a ferramenta nesta maquina (QEMU,
debugpy, kit cross, Podman/Docker) nao reprova: aparece na lista
`NAO PROVADO` do fim, e o veredito vira "VERDE no que esta maquina prova" em
vez de "TUDO VERDE". Com ele, a lista nao vazia reprova. O contrato inteiro
esta' em `DocsPublic/contribuindo/07-fluxo-e-responsabilidades-dos-gates.md`
§1.1, e cada desfecho em `04-os-gates-que-dizem-nao.md`.

O modo **completo** e o que bloqueia release: os binarios do icone
(`build/linux-clang-release-hardened/ui/kinein-vectis` e
`target/release/kinein-core`) so sao atualizados se lint, testes, C++ e
builds passarem. Presets CMake podem ser sobrescritos por ambiente:
`KINEIN_PRESET_DEBUG` / `KINEIN_PRESET_RELEASE` (padrao `dev-local*`).

### Recursos da rodada: só RAM, temporários e paralelismo (2026-10-24)

O gate se protege sozinho ([`contribuindo/07` §2](../contribuindo/07-fluxo-e-responsabilidades-dos-gates.md#2-o-que-o-orquestrador-possui)):

- **Só RAM.** Roda num escopo do systemd do usuário com swap zero e teto de
  memória disponível menos 2 GiB. O começo da rodada imprime a memória, o
  teto, o paralelismo e a pasta dos temporários. `KINEIN_GATE_SCOPE=0`
  desliga o escopo.
- **Paralelismo.** Sem `CMAKE_BUILD_PARALLEL_LEVEL`/`CARGO_BUILD_JOBS` no
  ambiente, cada um sai da memória; com eles, vale o que você pôs.
- **Temporários.** Cada rodada usa `${TMPDIR:-/tmp}/kinein-gate.<pid>` e o
  apaga no fim. Fora do gate, o runner do cargo
  (`scripts/isolar-temporarios.sh`, no `.cargo/config.toml`) dá a cada
  binário de teste uma pasta própria e a apaga quando ele termina. Para
  inspecionar o que um teste deixou:

```bash
KINEIN_KEEP_TEST_TMP=1 cargo test -p kinein-core nome_do_teste
```

Quando /tmp for tmpfs com quota insuficiente para ligar release com LTO,
use temporários em disco, sem desativar otimizações (o `kinein-gate.<pid>`
vai junto para lá). No QA local de 2026-10-22 isso resolveu Disk quota
exceeded, com o /tmp cheio de restos de testes. O teste de scrollback que
dependia do comprimento desse caminho não depende mais (52 §14.47).

```bash
TMPDIR=/var/tmp bash scripts/verificar.sh --completo --estrito
```

Prova adicional local de correlação de sondas na ponte Qt (Linux, build da
UI já concluído; não roda dentro do gate padrão):

```bash
python3 scripts/verificar_probe_response_identity.py --build-dir build/dev-local
```

O runner liga os mesmos objetos de produção com um main de fixture, usa
CoreClient real e um core IPC falso que responde fora de ordem. Grava os
relatórios e artefatos em `build/qa-artifacts/`, com temporários próprios nessa
partição. Nenhuma sonda/hardware é acessada. Não executá-lo durante
`verificar-cpp.sh` nem enquanto a UI ainda compila. Detalhes no
[52 §14.33](../roadmaps/52-arquitetura-executavel-da-0.4.md#1433-041h4b2b4a2--desfechos-de-sondas-correlacionados-no-qt-desenho-2026-10-22).

Prova adicional local da gravação nativa por sonda (Linux; exige `cmake`,
`ninja`, `arm-none-eabi-gcc` e probe-rs 0.32 no PATH; fora do gate padrão):

```bash
cargo build -p kinein-core
python3 scripts/verificar_probe_rs_nativo.py --core target/debug/kinein-core
```

O runner sobe o core real por stdio com variáveis `PROBE_RS_*` no ambiente,
faz Configurar e build de um projeto CMake arm-none-eabi e consulta o probe-rs
real (`mi meta`, `chip info`, `list`). Só uma listagem de fixture e o
`download` passam por um wrapper que grava argv e ambiente: nenhuma sonda é
aberta e nada é gravado em chip. O parser real recebe o argv planejado com um
seletor que não casa com sonda nenhuma. Relatório em `build/qa-artifacts/`.
Detalhes no [52 §14.34](../roadmaps/52-arquitetura-executavel-da-0.4.md#1434-041h4b2b4b1--probe-rs-nativo-alvo-cmake-sonda-explícita-e-efeitos-herdados-desenho-2026-10-22).

Provas locais dos wrappers nativos (Linux; fora do gate padrão):

```bash
python3 scripts/verificar_idf_py_nativo.py --core target/debug/kinein-core
python3 scripts/verificar_platformio_nativo.py --core target/debug/kinein-core
python3 scripts/verificar_platformio_home.py --core target/debug/kinein-core
```

- **`idf.py`.** Usa um ativador e um `idf.py` falsos, que gravam argv e
  ambiente, e não exige ESP-IDF instalado (52 §14.40).
- **PlatformIO.** Exige o `pio` (Core 6) no PATH, com a plataforma
  espressif32 e o framework Arduino instalados.
  - Passa 17 projetos-fixture pela prévia real, entre aceitos e recusados,
    incluindo uma plataforma dentro do projeto, que não pode ser carregada.
  - Roda um `run.start` real, em que o `pio run` compila e o esptool falha
    na porta, que é um arquivo comum. Nenhuma placa é tocada.
  - O core roda com `PLATFORMIO_*` e `ESPTOOL_*` no ambiente, e nenhuma pode
    chegar ao `pio` (52 §14.44).

- **PIO Home.** Exige o `pio` (Core 6). O core real sobe o PIO Home
  (`platformio.homeStart`) com um proxy HTTP morto no ambiente. Confere:
  - 200 na página da sessão e 403 na raiz;
  - o reuso da sessão;
  - a porta fechada depois de `platformio.homeStop` e depois de o core sair
    (52 §14.45).

Os relatórios ficam em `build/qa-artifacts/`.

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
scripts/verificar-qml-tokens.sh              # raio/fonte/duracao/cor literais no QML
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
scripts/verificar-fiacao-ipc.sh              # metodo sem cliente, evento sem tratador, sinal sem
                                             # ouvinte, sinal QML sem onX, elo do despacho C++ sem
                                             # chamador (2026-09-18)
cmake --build --preset dev-local             # UI debug (KINEIN_PRESET_DEBUG)
scripts/verificar-binario-abre.sh --preset dev-local          # o binario que saiu do build ABRE
cargo build --release -p kinein-core -p kinein-adapter-sqlite  # o adaptador fica ao lado do core
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
Desde 2026-10-24, o Qt do projeto é o 6.12 (40 §5). O módulo
`cmake/KineinQt.cmake` procura a instalação oficial mais recente em
`~/Qt/6.12.*/gcc_64`, ou o prefixo explícito `-DKINEIN_QT_PREFIX=/caminho`.
Recusa outra versão e grava `kinein-qt-bins.txt` em cada build. O qmllint,
runner QML, qmlformat e qmlls da exercitação saem dessa instalação.
Após instalar ou trocar o Qt, reconfigure com `cmake --fresh --preset
dev-local` e repita para `dev-local-release` e `linux-clang-debug-strict`.
Instalação e dependências no [14](14-ambiente-de-desenvolvimento.md).

Execucao manual da IDE:

```bash
./scripts/kinein-vectis
```

Quando o preset local ainda nao existir/configurar:

```bash
cmake --preset dev-local
cmake --preset dev-local-release
```

# 02 — Estrutura do Repositório

> **Classe: ESTADO** (`../README.md`). Tem que ser verdade hoje. Remedido no
> fim do dia **2026-09-10**, com o gate completo verde. Se divergir do código, o código
> vence e este documento se corrige no mesmo gesto.
>
> **A remedição de 2026-09-10 achou este documento MENTINDO, e o pior caso não
> era um número — era uma pasta.** Ele listava `templates/` com cinco scaffolds
> (`cpp-console-strict`, `cpp-qt-qml-strict`, `java-backend-strict`,
> `python-backend-strict`, `embedded-linux-strict`); **essa pasta não existe e
> nunca existiu neste checkout**. Os templates são escritos por código, em
> `crates/kinein-core/src/workspace/create.rs`. Uma sessão que fosse "editar o
> template" procuraria um diretório ausente, e a saída provável seria criá-lo —
> inventando um segundo mecanismo para o que já tem um.
>
> **O resto do desvio, medido:** a árvore de `docs/` listava oito arquivos que
> não existem mais (`00-product-vision.md`, `01-architecture.md`,
> `04-command-system.md`, `05-design-system.md`, `07`–`10`) e nenhuma das nove
> pastas que existem; `commands.rs` virou pasta; onze pastas de domínio do core
> e sete handlers estavam ausentes; o `kinein-protocol` aparecia com doze
> módulos e tem trinta; a lista de pastas do `ui/qml` tinha seis de vinte e
> três; e `prompts/` estava na raiz, quando mora em `docs-privada/`.
>
> **Por que ele envelheceu calado:** este documento não tem número solto, e o
> `verificar-docs.sh` só confere número. Árvore de arquivos é afirmação sobre o
> disco tanto quanto um número é — e não havia quem a conferisse. Ver §4.

## 1. A árvore, como ela é

```text
kinein-vectis/
├── README.md  MANUAL.md  Tutorial.md      os TRES markdown publicos
├── AGENTS.md  GUIAIA.md  PONTO_ATUAL.md   continuidade interna
├── COMO_EXECUTAR.md
├── Cargo.toml  rust-toolchain.toml  deny.toml
├── .gitignore  .editorconfig
│
├── crates/
│   ├── kinein-core/src/
│   │   ├── main.rs              binario: chama run_stdio
│   │   ├── lib.rs               Core + dispatch de handle_request + re-exports
│   │   ├── runtime.rs           laco JSON-RPC sobre stdio
│   │   ├── rpc.rs               erros JSON-RPC + parse de params
│   │   ├── handlers.rs          o modulo que agrega os handlers por dominio
│   │   ├── cargo.rs cmake.rs format.rs run.rs test.rs tools.rs process.rs
│   │   ├── cdb.rs               compilation database do C/C++: onde esta e se envelheceu
│   │   ├── fswatch.rs           notify debounced + mudanca externa
│   │   ├── probe.rs runconfig.rs settings.rs size.rs serial.rs
│   │   │
│   │   ├── handlers/            roteadores por dominio (blocos impl Core)
│   │   │   └── build cargo cmake configaction datasource debug draft format fs
│   │   │      git grafana jobs library probe run runconfig serial settings setup
│   │   │      sim syntax terminal toolchain tools workspace
│   │   │
│   │   ├── build/               mod parse
│   │   ├── commands/            mod build editor git ide run
│   │   ├── configaction/        catalog availability plan + um planejador por
│   │   │                        arquivo editado (cmakelists, presets, cargotoml,
│   │   │                        builddir), mais parametros, rigor, remover, error
│   │   ├── dap/                 wire parse reader session target
│   │   ├── datasource/          connection store secret introspect sqlite mongo
│   │   │                        mongo_infer
│   │   ├── db/                  rascunhos em SQLite (WAL)
│   │   ├── fsops/               confine ops search find replace transaction walk error
│   │   ├── git/                 operations parse
│   │   ├── grafana/             client store
│   │   ├── jobs/                context manager
│   │   ├── lang/                Tree-sitter local: registry service positions
│   │   │                        outline folding
│   │   ├── library/             catalog availability applied
│   │   ├── lsp/                 manager session sync server framing parse
│   │   │                        parse_symbols transaction edit uri types
│   │   ├── setup/               catalog distro
│   │   ├── sim/                 catalogo entradas entradas_sistema formula corrida
│   │   │                        corrida_sistema integrador sistema exata invariante
│   │   │                        persistencia
│   │   │   └── oraculo/         mod (a fachada) programa (o Python embutido)
│   │   │                        processo (o transporte) portao (o que se aceita)
│   │   ├── terminal/            session state render input error
│   │   ├── toolchain/           catalog store
│   │   ├── workspace/           detect open create session recent error
│   │   └── tests/               testes de integracao, um arquivo por dominio
│   │
│   ├── kinein-protocol/src/     um modulo por dominio, re-exportado plano do lib.rs
│   │   └── rpc command core tools workspace fs run terminal lsp syntax git build
│   │      cargo cmake configaction datasource debug diagnostic draft format
│   │      grafana job library probe runconfig settings setup sim sim_corrida
│   │      sim_sistema toolchain
│   │
│   ├── kinein-config/src/lib.rs
│   └── kinein-cli/src/          main.rs lib.rs commands.rs error.rs
│
├── ui/
│   ├── CMakeLists.txt           qt_add_qml_module: o QML_FILES e' a lista do que
│   │                            o modulo ENTREGA (ver verificar-qml-alcance.sh)
│   ├── src/                     ponte C++: CoreClient (um .cpp por dominio),
│   │                            realce do editor, clipboard, chrome de janela
│   ├── assets/icons/tree/       SVGs autorais de pasta/C/C++/Rust
│   └── qml/
│       ├── Main.qml  Theme.qml  StatusColors.qml
│       ├── app/                 composition: AppDomains, AppRouters
│       ├── ipc/                 routers por dominio (<X>EventRouter/<X>RequestRouter)
│       ├── shell/               toolbar, rail, layout, overlays, status, menus
│       ├── components/          KvIcon/KvButton/KvTooltip reutilizaveis
│       ├── editor/              renderer, controllers, outline/folding
│       ├── panels/bottom/       tool windows inferiores
│       ├── workspace/           picker, Start Screen e Project Health
│       └── command/ configaction/ datasource/ debug/ diagnostics/ embedded/
│           git/ grafana/ jobs/ library/ project/ runtime/ search/ settings/
│           setup/ sim/ toolchain/
│
├── scripts/                     gates, sondas, ambiente, launcher, packaging
│   └── qml-harness/             tst_*.qml — logica QML headless
├── packaging/appimage/          Containerfile e receita do AppImage
├── cmake/                       politicas estritas da propria UI
├── dist/                        saida unica: AppImage/checksum/instalador
├── schemas/                     ipc, settings, project, workspace, recent-workspaces
├── imagens/
│
├── docs/                        LIDA EM TODA SESSAO
│   └── adr/ arquitetura/ build/ iconografia/ integracoes/ roadmaps/ seguranca/
│      specs/ tooling/
├── docs-privada/                continuidade interna
│   └── ContextoIA.md diario/ prompts/
└── docs-legada/                 superado ou cancelado; nao implementar dali
```

**Não existe `templates/`.** Criar projeto novo escreve os arquivos a partir de
`crates/kinein-core/src/workspace/create.rs`. Template novo é código lá, com
teste em `crates/kinein-core/src/tests/workspace.rs` — não é arquivo numa pasta
de scaffold, e não há mecanismo de template externo.

## 2. Os quatro crates, e por que são quatro

```text
kinein-core       toda a logica: build, run, debug, LSP, git, terminal, fs, jobs
kinein-protocol   os TIPOS do contrato. Um modulo por dominio, re-exportado plano
kinein-config     configuracao tipada
kinein-cli        cliente fino do protocolo; nao replica o core
```

O core é o cérebro. Começar por Rust foi o que evitou que a lógica da IDE
ficasse presa ao Qt/C++ cedo demais — a UI conversa por JSON-RPC sobre stdio e
não linka nada do core.

**O caminho de crescimento é `função → arquivo → pasta → crate`**
(`ARCHITECTURE.md` §6). Hoje o core é **um crate só**, e isso está certo
enquanto couber: nenhum dos crates futuros já nomeados foi criado. As 18 pastas
de domínio acima nasceram do passo `arquivo → pasta`, cada uma quando um arquivo
único deixou de fazer uma coisa só.

## 3. Nomes e convenções

Pacote com hífen, crate com underscore:

```toml
[package]
name = "kinein-core"
```

```rust
use kinein_core::...
```

```text
diretorios/packages          kebab-case
crates/imports/modulos Rust  snake_case
tipos Rust                   PascalCase
funcoes                      snake_case
```

## 4. O `Cargo.toml` raiz — a FONTE é o arquivo

O workspace declara `resolver = "2"`, `edition = "2024"`, `rust-version`
e os lints estritos em `[workspace.lints]`, herdados por todos os crates.

**Não há cópia deles aqui, e a ausência é deliberada.** Uma cópia de
configuração num documento é exatamente o que envelhece calado: a versão
anterior desta seção dizia `missing_docs = "warn"` quando o arquivo já dizia
`deny`, e listava três lints a menos do que existem. O que os lints exigem, e
por quê, está em [`06-strict-mode.md`](06-strict-mode.md); o que eles **são**
está em `Cargo.toml`, que é o único lugar onde a resposta não pode envelhecer.

```bash
sed -n '/\[workspace.lints/,/^\[profile/p' Cargo.toml
```

## 5. Como conferir que esta página ainda é verdade

Nenhuma afirmação aqui depende de acreditar nela:

```bash
find crates/kinein-core/src -mindepth 1 -maxdepth 1 -type d | sort
ls crates/kinein-protocol/src/*.rs | wc -l          # modulos do protocolo
ls crates/kinein-core/src/handlers/*.rs | wc -l     # handlers
find ui/qml -mindepth 1 -maxdepth 1 -type d | sort
find docs -mindepth 1 -maxdepth 1 -type d | sort
ls -d templates 2>&1                                # tem de FALHAR
```

**A lição desta remedição, e ela vale para o repositório inteiro:** o gate de
veracidade confere número, e este documento não tinha nenhum. **Árvore de
arquivos é afirmação sobre o disco tanto quanto um número é.** A última vez que
esta página foi tocada foi em 2026-08-30, e mesmo então só nas duas linhas do
`terminal/` e do `lsp/` que a fatia daquele dia mexeu — a `templates/` inexistente
já estava aqui e passou por todas as revisões desde então. Quando um documento
descreve estrutura, os comandos acima são o que o mantém honesto.

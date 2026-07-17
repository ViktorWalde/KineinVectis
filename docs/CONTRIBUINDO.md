# Contribuindo com a Kinein Vectis

Guia para quem vai **alterar ou implementar** algo no projeto. Ele responde a
uma pergunta só: *onde olhar para fazer a mudança certa no lugar certo.*

> **Antes de propor arquitetura, split ou reorganização: leia
> [`arquitetura/ARCHITECTURE.md`](arquitetura/ARCHITECTURE.md) inteiro, e meça.**
> É contrato, não consulta, e é verificado por catraca. O padrão observado neste
> repositório é que o problema é **regra não cumprida, não regra ausente** — em
> 2026-07-16, 7 arquivos do core e 20 da UI violavam regras que já estavam
> escritas lá. Propor desenho novo sem medir é o erro mais caro possível aqui;
> ver §1.1 e §1.3 daquele documento.

Para **usar** a IDE, o documento é o [`../MANUAL.md`](../MANUAL.md). Para
**instalar/distribuir**, o [`../Tutorial.md`](../Tutorial.md).

---

## 1. O que o projeto é (e o que ele recusa ser)

A Kinein Vectis é uma IDE Linux-first para C, C++ e Rust: interface nativa em
Qt/QML conversando por JSON-RPC local com um core em Rust.

O princípio que explica quase toda decisão do repositório:

> **A IDE orquestra ferramentas maduras. Ela não reimplementa compilador,
> language server, build system, debugger nem emulador de terminal.**

Consequências práticas: a inteligência semântica é do clangd e do
rust-analyzer; o modelo de projeto vem do CMake File API e do `cargo metadata`;
o debug é DAP; o terminal usa o emulador do Alacritty. Quando existe ferramenta
aberta, madura e gratuita, a resposta certa é integrá-la — não escrever a nossa.

Também é rígido por padrão: warnings quebram o build, `unsafe` é proibido,
`unwrap`/`expect` fora de teste são recusados, e não há telemetria.

## 2. Arquitetura em uma tela

```text
Qt/QML  ─ apresenta, interage, exibe estado
   ↓ signal/property → controller QML → método da fachada
CoreClient (C++) ─ única ponte da UI
   ↓ JSON-RPC local tipado (kinein-protocol)
Rust Core ─ roteia, valida, mantém estado
   ↓ API interna
Serviços de domínio ─ workspace, fs, lsp, build, run, git, terminal…
   ↓ operação longa vira Job cancelável
Ferramentas externas ─ cargo, cmake, clangd, rust-analyzer, lldb-dap, git…
   ↑ evento tipado → router IPC no QML → controller → visual
```

A regra curta: **a UI apresenta; o Core decide; o serviço executa; Jobs
acompanham; Events notificam.**

Proibições que sustentam isso — não são estilo, são estrutura:

- a UI **nunca** chama ferramenta externa nem toca o filesystem do workspace;
- a UI **não** contém regra de negócio nem faz parsing de saída de ferramenta;
- todo dado entre UI e Core passa por tipos versionados do `kinein-protocol`;
- o core Rust **não** depende de Qt.

Detalhe obrigatório antes de escrever código novo:
[`arquitetura/ARCHITECTURE.md`](arquitetura/ARCHITECTURE.md).

## 3. Onde mexer

| Quero mudar… | Olhe aqui |
| --- | --- |
| Comportamento visual/layout | a spec da área em [`specs/`](specs/) → o componente em `ui/qml/` |
| Um contrato entre UI e core | [`arquitetura/03-ipc-protocol.md`](arquitetura/03-ipc-protocol.md) → `crates/kinein-protocol/src/<dominio>.rs` |
| Lógica de um domínio | `crates/kinein-core/src/<dominio>/` + `handlers/<dominio>.rs` |
| Editor, completion, navegação | spec `EDITOR_LANGUAGE_INTELLIGENCE` + [`roadmaps/25-syntax-tree-semantic-foundation.md`](roadmaps/25-syntax-tree-semantic-foundation.md) |
| Terminal | [`roadmaps/26-terminal-rendering-parity-roadmap.md`](roadmaps/26-terminal-rendering-parity-roadmap.md) + [`adr/ADR-0004-alacritty-terminal-emulator.md`](adr/ADR-0004-alacritty-terminal-emulator.md) |
| Build, Run, Test, Debug | spec `PRODUCT_FLOWS_BUILD_RUN_DEBUG` + [`build/22-compilacao-c-cpp-rust.md`](build/22-compilacao-c-cpp-rust.md) |
| Git | `crates/kinein-core/src/git/` + `ui/qml/git/` |
| Configurações/persistência | `crates/kinein-config` + `schemas/` (formato novo exige schema) |
| Adotar uma ferramenta externa | [`integracoes/README.md`](integracoes/README.md) |
| Ícones e sistema visual | [`iconografia/README.md`](iconografia/README.md) |
| Segurança de dados (save, drafts) | [`seguranca/23-rede-de-seguranca.md`](seguranca/23-rede-de-seguranca.md) |

O índice completo da documentação é o [`README.md`](README.md) desta pasta.

## 4. Preparar o ambiente

```bash
bash scripts/instalar-ambiente.sh   # dependências por distro
bash scripts/verificar.sh           # gate completo (deve ficar verde)
bash scripts/instalar-atalho.sh     # atalho "Kinein Vectis (Desenvolvimento)"
./scripts/kinein-vectis             # executa pelo checkout
```

Requisitos e detalhes em [`build/14-development-environment.md`](build/14-development-environment.md);
os comandos oficiais em [`build/COMANDOS_BUILD_VERIFICACAO.md`](build/COMANDOS_BUILD_VERIFICACAO.md).

## 5. O ritual de uma mudança

```text
1. DESENHAR   contrato, arquivos, testes e o que fica FORA — antes do código.
2. IMPLEMENTAR  protocolo → serviço no core → handler fino → testes → UI burra.
3. PROVAR     bash scripts/verificar.sh  (gate completo, para no 1º erro)
4. VER        para UI/terminal, o aceite é o gesto na tela real. Automação
              verde não é aceite visual.
5. SINCRONIZAR  contrato, schema, manual e arquitetura afetados.
```

Se a operação for longa, ela **vira um Job cancelável** — nunca bloqueia a UI.
Se houver estado visual, escreva um harness QML em `scripts/qml-harness/`.

### Armadilhas conhecidas do gate

- `scripts/verificar.sh | tail` **mascara o exit code**: confira o texto
  `✗ FALHOU`, não o código de saída do pipeline.
- `verificar-qml.sh` lê metadados de tipo do build; ao adicionar uma
  `property` nova, rode antes:
  `cmake --build build/linux-clang-debug-strict --target kinein-vectis`.
- `cargo test` **não** recompila `target/debug/kinein-core`: rode
  `cargo build -p kinein-core` antes de sondas e2e.

## 6. Adicionar uma integração (o caminho dos "plugins")

Não existe extension host nem código de terceiros carregado em runtime. Uma
integração é o core orquestrando uma ferramenta madura por contrato tipado.
O processo inteiro — modos A–D, gate de auditoria (licença, telemetria, rede,
pin), níveis L0–L10 e o checklist de 10 passos — está em
[`integracoes/README.md`](integracoes/README.md).

Antes de criar ou alterar uma funcionalidade de IDE, é obrigatório estudar a
implementação oficial e atual de uma referência pertinente (Code OSS, IntelliJ
IDEA Community, Zed, Lapce, NetBeans) e registrar revisão, lições e adaptação.
A referência autoriza **estudo**; não autoriza copiar função, traduzir
mecanicamente entre linguagens nem transplantar runtime.

## 7. Convenções

```text
Commits     Conventional Commits (feat:, fix:, docs:, refactor:, test:, chore:)
Rust        fmt + clippy pedantic -D warnings; erros tipados com contexto
C++/QML     clang-tidy Werror; qmllint estrito, zero warnings
Nomes       produto Kinein Vectis · core kinein-core · crate kinein_core
Atalhos     todo atalho com F-key tem alternativa sem F-key
Idioma      português no desenvolvimento; `qsTr` em toda string de UI
```

Nunca: telemetria, envio de código a serviço externo sem ação explícita do
usuário, formato de configuração sem schema, ou relaxar o strict mode sem
registrar o motivo.

## 8. Escopo atual

O foco é C, C++ e Rust em Linux x86_64. Windows é uma frente futura separada.
Ferramentas de outras linguagens ficam adiadas — um ambiente C/C++/Rust bem
feito já é trabalhoso, e diluir isso é o erro que o projeto evita
deliberadamente.

Licença: MIT **ou** Apache-2.0. Uma contribuição entra sob essa licença dupla;
código copyleft não é aceito no core.

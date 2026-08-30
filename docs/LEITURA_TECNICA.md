# Leitura técnica da Kinein Vectis

> **Classe: ESTADO** (`docs/README.md`). Tem que ser verdade hoje. Todo número
> aqui foi **medido em 2026-08-30** com a toolchain fixada e o gate completo
> verde, e está datado por isso. Se divergir do código, o código vence e este documento se corrige no
> mesmo gesto.
>
> **Para que serve:** dar em uma leitura o que hoje exige abrir dez documentos —
> o que o projeto é, o que existe de verdade, onde o peso está, onde a
> arquitetura está sob tensão e o que a direção escolhida custa. Não substitui
> `ARCHITECTURE.md` (contrato) nem `PONTO_ATUAL.md` (fila).

## 1. O que é, em uma frase honesta

Uma IDE Linux-first para **C, C++ e Rust**, que **orquestra ferramentas
consolidadas** em vez de reimplementá-las, com frontend Qt/QML e um core Rust
separados por JSON-RPC sobre stdio.

O que ela deliberadamente **não** é, e cada "não" é uma decisão registrada:

```text
nao tem host de extensoes       nao ha runtime de plugin de terceiro
nao tem Electron/Node/WebView   a UI e' Qt/QML nativa
nao tem telemetria              nenhuma. Nao e' opt-out, e' ausencia
nao tem IA embutida             cancelado em 2026-07-17: o usuario roda
                                claude/codex no terminal, que a IDE ja tem
nao reimplementa compilador,    clangd, rust-analyzer, CMake, Cargo, GDB/LLDB,
LSP nem debugador               ripgrep e fd sao orquestrados, nao substituidos
```

## 2. As três camadas, e onde está o peso (medido em 2026-08-29)

```text
Qt/QML  ──── IPC JSON-RPC (stdio, uma linha por mensagem) ──── Rust Core
```

| Camada | Linhas | Arquivos | O que carrega |
| --- | ---: | ---: | --- |
| `crates/kinein-core` | 25.823 | 96 | Toda a lógica: build, run, debug, LSP, git, terminal, fs, jobs |
| `ui/qml` | 23.227 | 139 | Apresentação e estado visual |
| `crates/kinein-protocol` | 3.879 | 22 | Os tipos do contrato, um módulo por domínio |
| `ui/src` (C++) | 4.115 | 21 | Ponte fina: `CoreClient`, realce, clipboard, chrome de janela |
| `scripts/` | 4.071 | 26 | Gates, sondas, ambiente, packaging |

**O fato que mais surpreende quem chega:** a documentação tem **62.697 linhas em
87 arquivos** — mais do que o core e a UI **somados**. Isso é uma escolha
consciente (o projeto é conduzido por sessões que trocam de contexto), mas cobra
um preço, e é a razão de existirem as três árvores e o gate de veracidade.

**A camada C++ é fina de propósito e isso é o desenho certo.** 4.115 linhas para
uma ponte: se ela engordar, é sinal de que lógica de negócio vazou da UI ou do
core para o meio.

## 3. O que existe de verdade

**109 métodos IPC** roteados, **17 domínios** no core, **378 testes** Rust
verdes (medido em 2026-08-30). Protocolo `0.62.0`.

Domínios do core, por profundidade real:

```text
SOLIDO      fsops     confinamento ao root, escrita atomica, transacao com
                      rollback, search/replace com walk unico e teste de paridade
            workspace deteccao de projeto, sessao, recentes, criacao por template
            lsp       manager, framing, parse, transacao de WorkspaceEdit
            git       operacoes reais contra repositorio, 12 testes de integracao
            jobs      cancelamento cooperativo, progresso, drain no shutdown

MEDIO       build/run/test/format/cmake/cargo   orquestracao + parse de saida
            dap       sessao de debug; 4 testes de integracao
            db        rascunhos em SQLite (WAL); a rede de seguranca de dados

            cdb       diagnostico da compilation database do C/C++: onde ela
                      esta, se envelheceu e qual arquivo a invalidou (0.62.0)

            terminal  pasta desde 2026-08-30: session (PTY/ciclo de vida),
                      state (grid VT), render (o contrato que o QML le), input
                      (o que a roda significa) e error. Era 955 linhas de codigo
                      num arquivo so; saiu da catraca. tests/terminal.rs, os 8
                      testes de integracao, nao mudou uma linha no corte.
            draft     handler sem arquivo de teste proprio; coberto de lado
                      por tests/workspace.rs e tests/fs.rs desde 2026-08-29
```

**O que mudou em 2026-08-29/30, e é o que destrava o resto:** o terminal deixou
de ser o único domínio grande sem teste de integração. A rede veio **antes** do
corte, de propósito — e provou o valor no dia seguinte, quando o upgrade do
`portable-pty` (0.8 → 0.9, para sair de uma dependência abandonada desde 2017)
passou sem uma única mudança nos testes.

## 4. Cinco fatos que mudam decisão

**1. O gate é o produto, não cerimônia.** **Treze** verificações, e **cada uma
nasceu de uma falha que passou verde por todas as outras** (`ARCHITECTURE.md` §4 regra
11). Não se cria gate aqui por gosto de rigor; cria-se quando uma classe de erro
não tem quem reclame. A recíproca também vale: gate que nunca reprovou não está
provado, está sem evidência — por isso cada um é testado por mutação.

**2. A catraca de arquitetura congela 21 arquivos e só deixa diminuir.** Ela não
é limite duro. O critério é **responsabilidade**; linhas são só o detector de
fumaça. Quando dispara há três suspeitos nesta ordem: **a sua mudança, a
categoria, o arquivo** — e medido em 2026-07-16/17, o terceiro errou em dois de
três casos.

**3. O maior débito bloqueia por área, não em geral.** `EditorController.qml`
(1.070/400) bloqueia qualquer feature de editor. Quem toca a área, paga a
dela antes — foi o que aconteceu com o terminal, pago em 2026-08-30.

**4. O core não escreve nada fora do workspace sem gesto explícito.** Desde
2026-08-29 a persistência global entra por `Core::enable_persistence`, chamada só
pelo processo real. O padrão seguro deixou de depender de disciplina — antes
disso, a suíte de testes apagou a lista de projetos recentes do autor.

**5. A cadeia de dependências é verificada, não afirmada.** Desde 2026-08-30 o
`cargo-deny` está no gate: licenças restritas a permissivas (MIT, Apache-2.0,
ISC, CC0-1.0, Unicode-3.0), advisories negados, origens conhecidas. A primeira
execução achou uma dependência **abandonada desde 2017** no caminho do terminal.
Ferramenta externa com licença copyleft (o GDB é GPL-3) é **executada como
processo**, nunca linkada.

**6. Referência profissional é obrigatória e cópia é proibida.** Toda feature de
IDE exige estudar Code OSS, IntelliJ IDEA Community, Zed, Lapce ou NetBeans e
**registrar** o que foi aprendido. Importa-se invariante, modo de falha e
estratégia de teste; nunca código, runtime ou modelo interno.

## 5. Onde a arquitetura está sob tensão

**Composition root da UI.** `Main.qml` é função do número de domínios, não da
qualidade do código. O projeto já inventou os roteadores (`<X>EventRouter` para
o que o core manda, `<X>RequestRouter` para o que a UI pede) e eles resolveram
parte; o resto é binding e bloco de host, que **é** trabalho de composition root.
Chegar abaixo do limite exige módulos por domínio — decisão registrada como
proposta em `arquitetura/27`, **não** implementada.

**O core tem 17 handlers e um `lib.rs` de 361 linhas** (limite 500) — ele
**saiu do débito em 2026-08-30**, quando ~140 linhas do domínio `tools` que
moravam ali voltaram para `handlers/tools.rs`. Quem cobrou foi a catraca, ao
reprovar UMA linha de outra fatia: a §4 regra 9 manda olhar a mudança, a
categoria e o arquivo nessa ordem, e aqui o culpado era o terceiro.

O caminho previsto no contrato é `função → arquivo → pasta → crate`, com os
nomes dos crates futuros já escolhidos. Nenhum foi criado ainda: hoje é um crate
só, e isso está certo enquanto couber.

**A UI tem um padrão que o core não tinha.** `WorkspaceUiResetter` é dono único
do "esqueça tudo do workspace anterior" — exatamente o padrão que faltava no core
e que causou o bug da store de rascunhos órfã. Vale citar como precedente
interno: quando um estado precisa mudar junto, ele precisa de **um dono**.

## 6. O que a direção escolhida custa

A ordem decidida em 2026-07-17 é **profundidade antes de superfície**:

```text
L1     plataforma de integracao (`integration` v1)
L2-L4  C/C++/Rust SOLIDOS: diagnostico, teste, cobertura, Project Graph, DAP
L5     RemoteContext — Docker entra AQUI, como contexto remoto
L5.5   banco de dados
L6     embarcados
```

E a decisão que define o custo: **Docker e banco são NATIVOS**, domínios do core
como `git` e `lsp`, não plugins de terceiro. O autor aceitou explicitamente que
isso é demorado. O que isso significa em concreto:

- **não há atalho de ecossistema.** Sem host de extensões, cada vertical é
  código Rust deste repositório, com testes deste repositório e passando pelos
  mesmos gates. Uma IDE com host de plugins terceiriza esse custo; esta o paga.
- **a referência funcional de banco não é o IntelliJ Community** — ele não tem
  Database Tools (é Ultimate). Verificado em 2026-07-17. O idioma visual do
  Community continua sendo a referência; o cliente de banco a auditar é o DBeaver.
- **cobertura antes de superfície é a trava certa.** "IDE com Docker e sem
  cobertura de teste é demo" é a formulação do próprio autor, e é o que põe
  L2–L4 antes de L5.

## 7. Como verificar tudo que está escrito aqui

```bash
scripts/instalar-ambiente.sh    # bootstrap: toolchain, Qt6, presets, build dirs
scripts/verificar.sh            # o gate completo, para no primeiro erro
```

Nenhuma afirmação deste documento depende de acreditar nele: cada número sai de
`git ls-files`, `cargo test` ou `scripts/verificar-arquitetura.sh`. Em conflito,
**o código vence**.

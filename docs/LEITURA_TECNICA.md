# Leitura técnica da Kinein Vectis

> **Classe: ESTADO** (`docs/README.md`). Tem que ser verdade hoje. Todo número
> aqui foi **medido em 2026-08-30**, e os da §3 e da §7 remedidos em
> **2026-09-02**, com a toolchain fixada e o gate completo verde; está datado
> por isso. Se divergir do código, o código vence e este documento se corrige no
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

**139 métodos IPC** roteados e **35 eventos**, **17 domínios** no core (mais os
módulos de arquivo único), **563 testes** Rust verdes e **24 harnesses QML**
(medido em 2026-09-04). Protocolo `0.78.0`. O gate tem **18 verificações**.

Domínios do core, por profundidade real:

```text
SOLIDO      fsops     confinamento ao root, escrita atomica, transacao com
                      rollback, search/replace com walk unico e teste de
                      paridade. Desde 2026-09-02 (0.65.0) a busca varre o
                      CONTEUDO, nao linha a linha: query com \n acha, e o
                      preview mostra o trecho inteiro com ` ⏎ ` no lugar das
                      quebras — e por isso a recusa de \n pode sair
            workspace deteccao de projeto, sessao, recentes, criacao por template
            lsp       manager (operacoes interativas), session (qual executavel,
                      subir/reiniciar/encerrar), sync (o TEXTO), framing, parse,
                      transacao de WorkspaceEdit. Desde 2026-09-02 os testes
                      sobem um servidor FALSO e olham o wire: didOpen/didChange/
                      didClose deixaram de ser afirmacao. Um configure
                      bem-sucedido fecha os documentos C/C++ abertos, e a UI os
                      reabre com o buffer real
            git       operacoes reais contra repositorio, 12 testes de integracao
            jobs      cancelamento cooperativo, progresso, drain no shutdown

MEDIO       build/run/test/format/cmake/cargo   orquestracao + parse de saida
            dap       sessao de debug; 4 testes de integracao. Pasta por
                      responsabilidade desde 2026-09-03: wire (o transporte),
                      parse (interpretar a resposta, o unico testavel sem
                      subir processo), reader (a thread leitora) e session.
                      Desde 0.66.0 tem `evaluate` (watches) e breakpoint com
                      `condition`/`hitCondition`
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

            toolchain qual executavel cumpre cada papel (compilador C/C++,
                      gerador, cmake, cargo), persistido em
                      .kinein/toolchain.json. Sem escolha, o PATH decide — o
                      comportamento historico. A escolha vira argumento de
                      cmake e executavel do build (0.64.0). Desde 0.67.0 a
                      escolha e' do KIT, nao do workspace: um kit e' um preset
                      mais sysroot e triple do alvo, e o schema 1 migra para o
                      kit padrao NA LEITURA, sem perder escolha de ninguem

            library   catalogo CURADO de bibliotecas C/C++ (0.68.0): licenca
                      verificada na fonte, versao pinada e a frase do que cada
                      uma faz. Dominio STATELESS — nao depende do Core nem de
                      workspace. NAO escreve arquivo de build: devolve plano,
                      e quem escreve e' o configaction

            configaction  as 18 Configuration Actions (16 da spec de MVP §12
                      mais findPackage e fetchContent, que o dominio library
                      nomeia), com
                      preview e consentimento (0.63.0). Pasta desde que nasceu:
                      catalogo (a tabela), disponibilidade (mede o workspace),
                      plano (o que sera escrito) e um planejador por arquivo
                      editado — CMakeLists.txt, CMakePresets.json, Cargo.toml e
                      o build dir. Nao executa ferramenta nem duplica dominio:
                      `cargo.check` devolve o job que ja existia.
```

**O que mudou em 2026-08-29/30, e é o que destrava o resto:** o terminal deixou
de ser o único domínio grande sem teste de integração. A rede veio **antes** do
corte, de propósito — e provou o valor no dia seguinte, quando o upgrade do
`portable-pty` (0.8 → 0.9, para sair de uma dependência abandonada desde 2017)
passou sem uma única mudança nos testes.

## 4. Cinco fatos que mudam decisão

**1. O gate é o produto, não cerimônia.** **Dezesseis** verificações (2026-09-03), e **cada uma
nasceu de uma falha que passou verde por todas as outras** (`ARCHITECTURE.md` §4 regra
11). Não se cria gate aqui por gosto de rigor; cria-se quando uma classe de erro
não tem quem reclame. A recíproca também vale: gate que nunca reprovou não está
provado, está sem evidência — por isso cada um é testado por mutação.

**2. A catraca de arquitetura congela 1 arquivo (medido em 2026-09-04) e só
deixa diminuir.** Ela não é limite duro. O critério é **responsabilidade**; linhas são só o detector de
fumaça. Quando dispara há três suspeitos nesta ordem: **a sua mudança, a
categoria, o arquivo** — e medido em 2026-07-16/17, o terceiro errou em dois de
três casos.

**3. O maior débito bloqueia por área, não em geral.** Era o
`EditorController.qml`, e a área do editor ficou travada por ele até
2026-09-02, quando quatro donos nasceram do corte (1.070 → 791; ver
[arquitetura/32](arquitetura/32-editor-por-responsabilidade.md)). O que resta
dele **não é mais implementação misturada** — é a fachada única do editor. A
pergunta que ficou aberta não era "como cortar mais", era por que o
`ShellWorkspaceHost.qml` precisava de 92 propriedades do editor. **O autor
respondeu em 2026-09-03: corta-se o host primeiro** (saída (a); `arquitetura/32`
§8.4). Feito no mesmo dia — o `ShellEditorHost.qml` nasceu com a fiação do painel
e o `ShellWorkspaceHost` caiu de 576 para 407 (medição de 2026-09-03) — e
continuado em **2026-09-04**, quando o mesmo movimento desceu um nível: o
`EditorPane` perdeu 31 propriedades e 21 sinais de puro repasse para o
`ShellEditorOverlayHost.qml`, e caiu de 538 para 226. Nesse dia a catraca foi de
**8 arquivos para 1**; ver
[roadmaps/39](roadmaps/39-divida-tecnica-paga.md).

Nenhum limite foi levantado em nenhuma das duas datas. O `EditorController`
segue **congelado em 791/400** — é o único item restante, e as duas saídas
(dissolver a fachada, ou corrigir a categoria) estão medidas no
[roadmaps/39](roadmaps/39-divida-tecnica-paga.md) §6, esperando decisão do
autor.

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

**O core tem 19 handlers e um `lib.rs` de 406 linhas** (limite 500) — ele
**saiu do débito em 2026-08-30**, quando ~140 linhas do domínio `tools` que
moravam ali voltaram para `handlers/tools.rs`. Quem cobrou foi a catraca, ao
reprovar UMA linha de outra fatia: a §4 regra 9 manda olhar a mudança, a
categoria e o arquivo nessa ordem, e aqui o culpado era o terceiro.

O caminho previsto no contrato é `função → arquivo → pasta → crate`, com os
nomes dos crates futuros já escolhidos. Nenhum foi criado ainda: hoje é um crate
só, e isso está certo enquanto couber.

**A fronteira de thread do job tem três saídas, não duas.** O
`arquitetura/04` §3 registrava que um job não alcança o `Core` e listava duas
formas de atravessar: um objeto compartilhado ou uma requisição nova da UI. Em
2026-09-02 apareceu a terceira, e é a mais barata: **o evento que o job emite já
volta ao dono do estado** — o loop principal o recebe antes de repassá-lo à UI.
Reagir ali (`Core::observe_notification`) não cria estado compartilhado nem um
segundo dono. Foi assim que a reabertura de documentos após o `cmake.configure`
saiu sem um `Arc<Atomic…>`.

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
L5.5   banco de dados — relacional E temporal (TimescaleDB), mais Grafana,
       decididos NATIVOS e plug and play em 2026-09-03. A licenca do
       Grafana (AGPL-3.0) decide a FORMA: HTTP API, nunca embutido.
       Levantamento em docs/integracoes/37
L6     embarcados — REORDENADO para cima em 2026-09-03, e decidido PLUG
       AND PLAY: a IDE detecta a sonda, deduz o alvo e roda build/flash/
       debug sem edicao manual; quando nao deduz, diz o que faltou.
       Registro em docs/integracoes/README.md e roadmaps/35 §5
```

E a decisão que define o custo: **Docker e banco são NATIVOS**, domínios do core
como `git` e `lsp`, não plugins de terceiro. O autor aceitou explicitamente que
isso é demorado. O que isso significa em concreto:

- **não há atalho de ecossistema.** Sem host de extensões, cada vertical é
  código Rust deste repositório, com testes deste repositório e passando pelos
  mesmos gates. Uma IDE com host de plugins terceiriza esse custo; esta o paga.
- **credencial de banco ainda não tem onde morar.** O `.kinein/` guarda
  rascunho e toolchain em TEXTO PURO; senha ali seria regressão de segurança,
  não feature. O projeto tem rede contra perda de dado (`seguranca/23`) e
  **não tem cofre**. Registrado em `roadmaps/35` §7.3: nenhuma conexão a banco
  entra antes dessa pergunta ter dono.
- **a referência funcional de banco não é o IntelliJ Community** — ele não tem
  Database Tools (é Ultimate). Verificado em 2026-07-17. O idioma visual do
  Community continua sendo a referência; o cliente de banco a auditar é o DBeaver.
- **cobertura antes de superfície é a trava certa.** "IDE com Docker e sem
  cobertura de teste é demo" é a formulação do próprio autor, e é o que põe
  L2–L4 antes de L5.

## 7. Para onde o projeto vai

A ordem das etapas até o MVP está em
[roadmaps/30-caminho-para-o-mvp.md](roadmaps/30-caminho-para-o-mvp.md),
decidida pelo autor em 2026-08-30 e medida item por item contra a spec de MVP.
Medido em **2026-09-02**, os **21 itens do MVP essencial estão fechados**: 20
existem e 1 é item morto (a linha de IA, cancelada). As Configuration Actions —
o único que faltava — entraram na etapa 2. O resto da lista é dívida de
honestidade e de hardening, não funcionalidade.

**Essa lista fechou em 2026-09-02** — 9 etapas feitas, a 6 paga em parte com uma
decisão que ficou em aberto e foi **respondida em 2026-09-03**. O sucessor é
[roadmaps/34-depois-do-mvp.md](roadmaps/34-depois-do-mvp.md): as quatro frentes
do pós-MVP — dívida que cobra pedágio, atrito diário medido, profundidade (TR2)
e a simulação —, cada item com o comando que mede se ainda está pendente. A
recomendação de ordem começava por **uma pergunta ao autor**, não por código: o
`EditorController.qml` em 791/400 (§3.2). Respondida em 2026-09-03 — a fila
agora abre na fatia do `ShellWorkspaceHost.qml` (34 §7, etapa 11.1), **entregue
em 2026-09-03**.

O horizonte mais distante, e **ainda não arquitetado**, está em
[roadmaps/31-simulacao-fisica-matematica.md](roadmaps/31-simulacao-fisica-matematica.md):
montar simulações física/matemática por layout e desenhá-las em OpenGL. É
estudo, não plano de execução.

Como o processo da UI e o do core conversam — boot, threads, ordem garantida,
crash e recuperação — está em
[arquitetura/04-boot-e-comunicacao.md](arquitetura/04-boot-e-comunicacao.md).

## 8. Como verificar tudo que está escrito aqui

```bash
scripts/instalar-ambiente.sh    # bootstrap: toolchain, Qt6, presets, build dirs
scripts/verificar.sh            # o gate completo, para no primeiro erro
python3 scripts/sonda_drafts.py    # rede de seguranca, contra o binario real
python3 scripts/sonda_terminal.py  # terminal e2e: grid, resize, historico
python3 scripts/sonda_soak.py      # soak: memoria, fds, threads, filhos, latencia
bash scripts/empacotar-appimage.sh # gera o AppImage (Podman + rede); o
                                   # verificar.sh valida o que houver em dist/
```

Nenhuma afirmação deste documento depende de acreditar nele: cada número sai de
`git ls-files`, `cargo test` ou `scripts/verificar-arquitetura.sh`. Em conflito,
**o código vence**.

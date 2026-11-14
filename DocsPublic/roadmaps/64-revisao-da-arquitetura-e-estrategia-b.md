# 64 — A revisão da arquitetura e a estratégia B

> **Classe: PLANO** (`DocsPublic/README.md`), com medidas datadas. É o dono
> da revisão de concepção de 2026-11-14 e do plano da estratégia B: o
> diagnóstico medido, a arquitetura-alvo, a correção dos riscos da própria
> estratégia, a ordem e o aceite de cada mudança estrutural.
>
> **Situação em 2026-11-14.** Decisão do autor de 2026-11-14: a 0.4 segue a
> **estratégia B** (reconstrução arquitetural seletiva). O bloco do D-19 do
> [63](63-regras-e-plano-de-engenharia-da-0.4.md) (ENG-40 a ENG-46) não
> começa como está escrito. As nove decisões do §11 foram tomadas no mesmo
> dia, todas como recomendadas. A Parte A do 63 continua sendo o
> regulamento. O B0 (§8) fechou no mesmo dia: os requisitos moram no 19, e o
> 63 foi reconciliado. Também no mesmo dia, o autor decidiu que **a 0.4 é a
> última versão que reconstrói de forma sistemática os fundamentos da Kinein
> Vectis** (D-27 do 63, D18 do 19). Por isso o motor de edição deixou de ser
> "pós-V1" e virou a DP-10 (§11), ainda aberta, que segura as fases do
> editor. Nenhuma fase de código começou.
>
> **Ponto de partida.** Commit `9198888c`, protocolo 0.223.0. Toda medida
> deste documento é de 2026-11-14, nessa árvore, salvo data própria.

## 0. Como ler

O §1 responde à pergunta da revisão. O §2 traz o que foi medido e
reproduzido, com o método. O §3 lista os problemas por gravidade, cada um com
a solução consolidada que este plano adota. O §4 aponta os requisitos que
faltavam, agora no 19. O §5 é a arquitetura-alvo; o §5.15 traz o Zed como
referência de engenharia. O §6 lista os riscos da estratégia B e a correção
de cada um. O §7 diz o que acontece com cada item do 63. O §8 é a
ordem, com aceite e reversão. O §9 traz as provas. O §10 classifica os
componentes. O §11 traz as decisões do autor. O §12 diz o que não foi
medido.

## 1. A pergunta e a resposta

A pergunta: o projeto corrige uma implementação imperfeita de uma
arquitetura adequada, ou preserva uma concepção inadequada?

**A resposta é mista, e a evidência separa as duas partes.**

- **O nível macro é adequado e fica:**
  - dois processos (UI Qt/QML e core Rust), com o core dono dos fatos;
  - um protocolo tipado no `kinein-protocol`;
  - ferramentas maduras orquestradas como processos;
  - Jobs com cancelamento;
  - QML que só envia intenção.
- **O nível intermediário tem quatro decisões que não servem ao produto
  declarado.** Elas foram tomadas quando a IDE era de uso pessoal, com 17
  requisitos ([19](../arquitetura/19-compromissos-de-arquitetura.md),
  revisão de 2026-07-09):
  1. **Execução:** o laço do core bloqueia por padrão. Sair dele é escolha
     de cada handler, e nada tem prazo.
  2. **Ponte:** um sinal difundido por resultado, sem correlação com o
     pedido, sobre um contrato tipado de um lado só.
  3. **Políticas transversais:** lançar processo, confiança no projeto,
     acesso a arquivo, troca de workspace e persistência não têm ponto
     único. Cada local de chamada refaz a política.
  4. **Escopo:** sem camadas de suporte. Tudo vive no mesmo processo e na
     mesma régua.

A consequência mais grave é de segurança, e foi reproduzida (§2.1): o modo
"não confiável" executa código do projeto por cinco caminhos.

**A estratégia B troca essas quatro costuras e preserva o resto.** A troca é
incremental, atrás da superfície IPC que os testes já exercitam. A
reconstrução ampla foi descartada porque o que precisa mudar é fino. Também
foi descartada a continuação do 63 como está: ele endurece verificadores de
texto em volta de estruturas que o próprio 63 troca depois. Os §5 e §6
mostram que cada ponto tem solução consolidada na indústria, e o plano usa
essas soluções, não mecanismos próprios.

**O princípio que limita a B (decisão do autor de 2026-11-14, D-27 do 63).**
A 0.4 deve ser a última versão em que a Kinein Vectis precisa reconstruir
sistematicamente os seus fundamentos. A B não é a primeira de uma série de
reconstruções: cada costura que ela troca fecha com decisão datada e
imposição verificada, e da 0.5 em diante o produto cresce por pontos de
extensão. A lista dos fundamentos está na D18 do
[19](../arquitetura/19-compromissos-de-arquitetura.md). O motor de edição
entra nela, e por isso a escolha dele virou a DP-10 (§11), com avaliação e
prova de conceito dentro da 0.4.

## 2. O que foi medido e reproduzido em 2026-11-14

**Ambiente.**
- Binário `kinein-core` release, compilado do `9198888c`.
- Rust 1.96.1, na máquina de medição do projeto: 16 núcleos, 21 GiB de RAM,
  sem swap.
- O core roda como a UI o roda: um processo filho, JSON-RPC por linha no
  stdin e no stdout.
- `HOME` e `XDG_*` apontam para uma pasta temporária, para nada tocar o
  estado da pessoa.
- Os projetos de teste ficam em pastas temporárias. O projeto grande é um
  clone do próprio repositório no `9198888c`: 706 commits, 2.086 arquivos
  rastreados.

**Método do bloqueio do laço.** O laço do core é um só. O pedido medido sai
seguido de um `core.ping`, que só é atendido depois que o pedido libera o
laço. O atraso do ping é o tempo em que o laço ficou preso.

### 2.1 Código do projeto executado sem confiança (INF-23)

Cada "binário do projeto" do experimento é um script que só grava uma marca
numa pasta temporária. O projeto abre sem confiança; a resposta do
`workspace.open` traz `trusted: false`. Os pedidos são os que a ponte manda
sozinha depois do `workspace.open` (`ui/src/core_client_dispatch.cpp`:
`cmakeStatus`, `cmakePresetsList`, `cargoMetadata`, `runConfigList`,
`gitStatus`), mais o primeiro pedido LSP de um arquivo.

| # | Vetor | Gatilho | Resultado |
| --- | --- | --- | --- |
| E1 | `<raiz>/.venv/bin/python --version` | o próprio `workspace.open`: `activate_workspace` → `configure_python_lsp` → `python_env` (`crates/kinein-core/src/python/env.rs`, `versao_de`). O `python_env` também é chamado pelo contexto do índice (`index/context/mod.rs`); a origem da segunda execução não foi isolada | **executou, 2 vezes** |
| E2 | `core.fsmonitor` do `.git/config` do projeto | `git.status`, que a ponte manda sozinha depois de abrir | **executou** |
| E3 | `build.rustc` e `build.rustc-wrapper` do `.cargo/config.toml` | `cargo.metadata` do core (`cargo metadata --no-deps`) | não executou |
| E4 | `build.rustc-wrapper` do `.cargo/config.toml` | o primeiro pedido LSP de um `.rs` (medidos `lsp.semanticTokens` e `lsp.didChange`): o core sobe o rust-analyzer "sem confiança", e os comandos do cargo que ele roda chamam o wrapper (a marca registra `<wrapper> <rustc> -vV`; qual comando do cargo não foi isolado) | **executou, em 0,5 s** |
| E5 | o compilador C fixado por caminho absoluto no `.kinein/toolchain.json` do projeto, com um `compile_commands.json` que nomeia o mesmo caminho | o primeiro pedido LSP de um `.c`: `configure_clangd_from_toolchain` não consulta a confiança, o caminho vai ao `--query-driver` e o clangd o executa para extrair os includes (`-E -v -x c -`) | **executou, 2 vezes** |
| E6 | `<raiz>/.venv/bin/python`, entregue ao basedpyright como `python.pythonPath` (`handlers/python.rs`) | o primeiro pedido LSP de um `.py` (medidos `lsp.didChange` e `lsp.semanticTokens`): o basedpyright 1.40.2 sobe e roda o interpretador com `-c` e `-I -c`, para ler os caminhos de busca | **executou, 6 vezes** |

**Conferência direta com o cargo.** Fora do core, com marcas novas a cada
comando, no mesmo projeto: `cargo metadata --no-deps` não executa o wrapper.
`cargo metadata` sem `--no-deps`, `cargo rustc -- --print cfg` e
`cargo check` executam.

**Conclusão.** Cinco vetores executaram (E1, E2, E4, E5 e E6). Desligar build
scripts, macros procedurais e o check ao salvar, que é o que o
[contrato do workspace](../arquitetura/03-dominios/workspace.md) descreve,
não impede o rust-analyzer de executar código do projeto. O mesmo vale para
o interpretador do `.venv` (pelo core e pelo basedpyright), para o `git` e
para o clangd com o kit do projeto. A extensão do rust-analyzer para o VS Code se declara sem suporte a
workspace não confiável, com o motivo: ele invoca binários definidos pela
configuração e os da toolchain.

**Limite do E5.** O caminho do compilador no `toolchain.json` tem de ser
absoluto (`kits::is_installation`). Quem ataca precisa acertar onde o
projeto será aberto, ou apontar um arquivo que já exista na máquina.

**O que a leitura do código mostra seguro, sem experimento próprio.** O
compilador automático, o do `CMakeCache.txt`, só ganha identidade quando é o
mesmo arquivo de um candidato detectado (`ProjectCache::automatic`, em
`toolchain/project.rs`). Sem identidade, ele não vai ao `--query-driver`.

**Observado, sem exercitar como vetor.** No E6, o servidor do ruff (0.17.4)
também subiu sem confiança. O projeto do experimento não tinha configuração
do ruff, e o que ele faz com uma configuração hostil não foi medido (§12).

**Abrir grava no projeto.** O `workspace.open` de qualquer pasta, confiável
ou não, deixou nela dois arquivos:
- `.kinein/workspace.json`, com o caminho absoluto da raiz. Em produção,
  nada o lê de volta (`workspace/open.rs` só o grava).
- `.kinein/kinein.db`, de 12 KiB, sem rascunho nenhum salvo.

### 2.2 O bloqueio do laço por método

O projeto grande foi aberto e indexado. Cada linha é a mediana e o máximo de
5 execuções.

| Método | Laço, mediana | Laço, máximo | Bytes da resposta |
| --- | --- | --- | --- |
| `setup.list` | 517,8 ms | 830,8 ms | 8.675 |
| `configAction.list` | 55,1 ms | 71,3 ms | 30.558 |
| `workspace.open` | 42,1 ms (uma execução) | — | 290 |
| `format.text` (`lib.rs` inteiro) | 30,4 ms | 37,8 ms | 29.640 |
| `cargo.metadata` | 29,9 ms | 52,6 ms | 1.006 |
| `git.blame` (`lib.rs`) | 22,0 ms | 24,7 ms | 58.984 |
| `library.list` | 16,7 ms | 18,3 ms | 5.942 |
| `fs.search` (`Core`) | 12,3 ms | 12,7 ms | 66.855 |
| `fs.findFiles` (`core`) | 11,0 ms | 13,6 ms | 3.228 |
| `git.status` | 7,7 ms | 149,6 ms | 124 |
| `git.log` (200 commits) | 7,4 ms | 8,2 ms | 51.672 |
| `git.log` (sem limite), `python.status`, `project.model`, `git.fileDiff`, `cmake.status`, `serial.list` | 3 a 7 ms | até 7,5 ms | — |
| Os outros 21 medidos | < 3 ms | 46,2 ms no `fs.read`; até 3,1 ms nos demais | — |

`tools.detect`, `probe.list`, `dfu.list` e `container.list` respondem
depois, fora do laço (1.426 ms, 128 ms, 61 ms e 248 ms até a resposta), e o
laço fica livre.

**Leitura.**
- Neste projeto e nesta máquina, o laço aguenta a maioria dos métodos.
- `setup.list` roda o `--version` de cada ferramenta do guia em série, no
  laço (`setup::guides` → `detector.detect`).
- O `workspace.open` passou de 1,5 ms em 2026-07-16 para 42,1 ms, contra o
  orçamento de 50 ms do [21 §M4.2](21-roadmap-de-longo-prazo.md). O projeto
  medido cresceu no mesmo período, então os dois números não são da mesma
  árvore.
- O problema não é a média. É o caso sem limite do §2.3.

### 2.3 Uma ferramenta lenta prende o core inteiro

O teste usou um `git` falso, primeiro no `PATH`, que espera 6 s e chama o
`git` real.

- O `git.status` levou **18,0 s**: são três chamadas ao `git` em série
  (`is_inside_work_tree`, `workspace_prefix` e o `status`, em
  `git/operations.rs`), sem prazo.
- O `core.ping`, mandado 0,2 s depois, respondeu em **17,8 s**.
- O `job.cancel`, mandado junto, também respondeu em **17,8 s**.

Enquanto um processo externo não responde, nenhum pedido anda, nem o
cancelamento. Os casos reais da mesma classe:
- `git` num sistema de arquivos de rede, ou com um `fsmonitor` que trava;
- um `.venv/bin/python` quebrado no `workspace.open`;
- um adaptador DAP que não responde, que segura o `debug.*` por até 10 s
  (`dap/session.rs`, `REQUEST_TIMEOUT`).

### 2.4 `syntaxTree.update`: o buffer inteiro na ida, a árvore inteira na volta

Mediana e máximo de 5 versões seguidas, cada uma com 1 caractere a mais:

| Linhas | Enviado por pedido | Laço, mediana | Laço, máximo | Resposta por pedido |
| --- | --- | --- | --- | --- |
| 2.500 | 89 KiB | 42,9 ms | 73,4 ms | 1.000 KiB |
| 5.000 | 181 KiB | 84,8 ms | 113,2 ms | 2.037 KiB |
| 20.000 | 726 KiB | 345,0 ms | 395,2 ms | 8.218 KiB |

**Leitura.**
- Cada edição custa o arquivo inteiro na ida e um JSON de 1 a 8 MiB na
  volta, que a UI ainda precisa ler.
- A medida de 2026-07-16 (21 §M4.2), do `scripts/medir-performance.sh`, era
  de 281 ms e 1.138 KB por atualização, num fonte de 56 KB. As condições das
  duas medidas não foram comparadas (arquivo, binário, máquina). O que as
  duas mostram igual é a resposta com cerca de 20 vezes o tamanho do
  arquivo.
- O RSS do core era de **25.000 KiB** depois de abrir e indexar. Ao final
  da rodada (os pedidos do §2.2 e as 15 atualizações), era de
  **68.292 KiB**, acima do orçamento de 60 MB do 21 §M4.2. A medida não
  separa quanto veio de cada pedido.

### 2.5 O terminal (ponto encerrado: não muda)

O teste rodou `seq 1 300000` num terminal de 160×50. Saíram 4 eventos
`event.terminal.render` em 0,13 s, o maior com 9 KiB. A grade vai por quadro
e só a parte visível. Esse modelo fica.

### 2.6 O peso dos drivers de banco no processo do core

`cargo tree -p kinein-core -e normal` lista 254 dependências únicas. Sem
`mongodb`, `postgres`, `tokio-postgres` e `odbc-api`, sobram 129. Os quatro
drivers trazem **125 das 254 dependências (49%)** para dentro do processo que
decide os fatos do editor. A [ADR-0010](../decisoes-adr/ADR-0010-drivers-em-processos-versionados.md)
decidiu em 2026-10-07 tirá-los dali e segue com "implementação pendente".

### 2.7 O mecanismo de imposição pelo compilador, provado

O protótipo é um workspace Cargo de dois crates, numa pasta temporária, com
o Rust 1.96.1 e o clippy 0.1.96 do projeto:

- O crate do lançador tem um `clippy.toml` próprio e é o único que chama
  `std::process::Command::new`.
- O crate consumidor tem `disallowed_methods = "forbid"` na tabela `[lints]`
  e herda o `clippy.toml` da raiz, que proíbe o `Command::new`.

| Forma plantada no consumidor | Resultado |
| --- | --- |
| `std::process::Command::new` direto | `error: use of a disallowed method` |
| o mesmo, com `#[allow(clippy::disallowed_methods)]` | `error[E0453]: allow(clippy::disallowed_methods) incompatible with previous forbid` |
| a chamada pelo crate do lançador | passa |

A regra "só o lançador cria processo" não precisa de leitor nem de catraca:
o compilador a impõe, e a fuga pelo `allow` não compila.

**O mesmo mecanismo para a leitura e a escrita de arquivo (o ENG-47 e o
ENG-02 do 63).** Segundo protótipo, no mesmo arranjo: o crate dono tem o
`clippy.toml` próprio e é o único que lê e grava; a raiz proíbe
`std::fs::read`, `read_to_string`, `write`, `File::open`, `File::create` e
`OpenOptions::open`. Cada forma foi plantada sozinha no consumidor.

| Forma plantada no consumidor | Resultado |
| --- | --- |
| a leitura pelo crate dono | passa |
| `std::fs::read_to_string`, `std::fs::read`, `std::fs::write` | `error: use of a disallowed method` |
| `std::fs::File::open`, direto ou importado com `use std::fs::File` | `error: use of a disallowed method` |
| `std::fs::OpenOptions::new().read(true).open(...)` | `error: use of a disallowed method` |
| a leitura com `#[allow(clippy::disallowed_methods)]` | `error[E0453]` |
| a função guardada num apelido (`let g = std::fs::read_to_string::<&Path>`) | `error: use of a disallowed method` |

O que o protótipo não cobre: `std::fs::read_dir`, `metadata`, `copy`,
`rename` e `remove_file` não estavam na lista. Entram na lista do B7 pelo
mesmo mecanismo, e a lista final é a do catálogo do `kinein_dir` (§5.9).

### 2.8 Medidas estáticas da revisão

- **Core:**
  - `kinein-core`: 537 arquivos e 151.597 linhas. Pelo corte heurístico
    entre produção e teste, ~83,8 mil linhas de produção.
  - `Core`: 32 campos, definidos na raiz do crate.
  - `impl Core` em 73 arquivos de produção.
  - Cerca de 250 funções de resposta: ~106 com `&mut self` e ~204 com
    `&self`.
  - 38 pontos de adiamento em 19 arquivos de handler e ~36 `jobs.spawn`.
  - Zero declarações `trait` nos crates do projeto.
- **Processos:** ~122 `Command::new` de produção em 28 módulos. Os dois
  executores comuns (`owned_child`, `process.rs`) são usados por parte deles.
- **Ponte:**
  - `ui/src/core_client.h`: 201 `Q_INVOKABLE` e 220 sinais.
  - 263 comparações `method == QStringLiteral(...)` em `ui/src`.
  - `m_pendingMethods` guarda só id → método, ao lado de quatro mapas
    pendentes específicos, e não há prazo nem cancelamento de pedido.
- **Contrato:** `schemas/ipc.schema.json` parou no protocolo 0.62.0 e nada o
  referencia. O gate de fiação lê o código por expressão regular.
- **Confiança:** consultada em quatro lugares do core
  (`handlers/lsp/toolchain.rs`, `handlers/lsp/verification.rs`,
  `handlers/platformio.rs`, `handlers/syntax_check.rs`) e num da QML
  (`ui/qml/workspace/ProjectHealthController.qml`, o configure automático).
- **Testes:** 663 chamadas a `handle_request(` e 622 testes em `src/tests/`.
  Esses testes exercitam a superfície IPC e sobrevivem à reestruturação
  interna.
- **Scripts:** 366 arquivos rastreados em `scripts/`, com 19.150 linhas de
  Python e shell.
- **`.gitignore` dos modelos de projeto** (`workspace/cmake_template.rs`,
  `workspace/create.rs`): o de C ignora `/.kinein/` inteiro; o de C++ e o de
  Python não o ignoram. Nesses dois, um `git add .` leva para o repositório o
  `kinein.db`, com o texto não salvo dos rascunhos, e o `session.json`.
- **Commits:** 706 desde 2026-07-04. Dos últimos 400, 107 são `feat`, 94
  `fix`, 92 `docs` e 5 `refactor`.

## 3. Os problemas, por gravidade, e a solução adotada

A severidade segue a escala do [62 §3.4](62-base-publica-0.4.md). O número
do item é o do inventário do 62.

| Item | Problema | Severidade | Solução consolidada adotada | Onde no plano |
| --- | --- | --- | --- | --- |
| INF-23 | Código do projeto executado sem confiança (E1, E2, E4, E5, E6) | crítica | O modo restrito dos IDEs maduros: sem confiança, não executa nada cujo programa ou comportamento venha do projeto (o Safe Mode do IntelliJ desliga VCS e importação de build; o Restricted Mode do VS Code desliga tarefas, depuração e as extensões sem suporte, o rust-analyzer entre elas). A política mora no core, no lançador. | §5.5, B2 |
| ENG-50 | O laço bloqueia por padrão; trabalho sem prazo nem cancelamento | alta | Classe de execução declarada na rota; trabalho fora do laço com prazo, cancelamento (o `$/cancelRequest` do LSP, com `RequestCancelled`) e fila por recurso; guarda que reprova E/S no laço nos testes, como o StrictMode do Android | §5.3, B3 |
| ENG-51 | Processo criado em ~122 lugares, sem política única | alta | Um lançador dono do `Command` e do PTY, imposto por `disallowed_methods = "forbid"` (§2.7) | §5.4, B2 |
| ENG-52 | O core não registra nada: nem duração, nem processo, nem fila | alta | `tracing` com um span por pedido, por processo e por job; arquivo rotativo no estado XDG; o requisito da [especificação da arquitetura interna](../especificacoes/arquitetura-interna-core-ipc-jobs.md) §23, que não tinha dono | §5.10, B1 |
| ENG-53 | A ponte não correlaciona resposta e pedido; pedido sem prazo nem cancelamento | alta | Um objeto de resposta por pedido, no molde do `QNetworkReply` do Qt (`finished`, `failed`, `cancel()`, dono em C++) | §5.6, B5 |
| ENG-54 | Contrato tipado só no Rust; schema parado; dublês dos harnesses sem conferência | alta | Fonte única no `kinein-protocol`; JSON Schema gerado (`schemars`) e conferido como artefato gerado; constantes C++ geradas; dublês validados contra o schema (teste de contrato) | §5.7, B4 |
| ENG-55 | `syntaxTree.update` com o buffer inteiro na ida e 1 a 8 MiB na volta por tecla | alta | Sincronia incremental como o `didChange` incremental do LSP; `Tree::edit` e reanálise incremental do tree-sitter fora do laço; volta só com as faixas mudadas (`changed_ranges`); versão velha descartada | §5.11, B8 |
| ENG-56 | O `.kinein` mistura configuração do projeto, estado da pessoa e cache; dois dos três modelos não o ignoram no git | alta | As três classes do XDG Base Directory (config, state, cache), com o estado e o cache por projeto fora da árvore dele, como o `workspaceStorage` do VS Code; a divisão compartilhável/local do JetBrains (`dataSources.xml` e `dataSources.local.xml`); a pasta da ferramenta com `.gitignore` próprio, como `.pytest_cache` e `.ruff_cache` | §5.9, B7 |
| ENG-57 | Drivers de banco no processo do core: 125 de 254 dependências | média | A ADR-0010 executada: adaptador em processo, com contrato versionado. O precedente já existe: `kinein-adapter-sqlite` | §5.13, B9 |
| ENG-58 | Requisitos parados em 2026-07-09; requisitos do 9 (log, armazenamento) sem dono nem verificação | alta | Linha de base de requisitos em tabela, cada requisito com métrica, método de medida e o teste ou gate que o impõe | 19 (desde o B0); a imposição, na fase de cada requisito |
| ENG-60 | O motor de edição e a posse do documento e da inteligência de linguagem ficavam para "pós-V1", contra o D-27 | alta | Avaliação com prova de conceito isolada antes de decidir, com critérios de refutação escritos antes da prova | DP-10 (§11), §9 |
| ENG-59 | Adaptador DAP e motor de gravação escolhidos por `match` de texto | baixa | Tabela de descritores, no molde do `ServerSpec` do LSP (`lsp/registry.rs`) | §5.12, B10 |

Os itens ENG-10 a ENG-17 e ENG-20 a ENG-25 do 63 continuam válidos: tabela
de rotas, camadas, estado por domínio e ponte por domínio. O §7 diz o que
muda em cada um.

## 4. Os requisitos que faltavam

O [19](../arquitetura/19-compromissos-de-arquitetura.md) é o dono dos
requisitos. A tabela aceita na DP-03 (§11) foi levada ao 19 no B0, na seção
"Requisitos mensuráveis", com a referência de medida, a coluna "quem impõe"
e o estado de cada requisito em 2026-11-14. As camadas de suporte (DP-07) e
a extensibilidade também moram lá. Este documento cita os requisitos pelo
ID (RNF-EXE-01, RNF-SEG-01 e os demais) e não repete a tabela, para ela ter
um dono só.

## 5. A arquitetura-alvo

### 5.1 Processos e direção das dependências

```text
UI (Qt/QML)  --JSON-RPC por linha-->  kinein-core  --processos-->  ferramentas
                                         |                         (git, cmake, cargo,
                                         |                          compiladores, sondas)
                                         +--LSP/DAP-->  servidores de linguagem e adaptadores
                                         +--contrato versionado-->  adaptadores de periferia
                                                                    (Banco: ADR-0010)
```

**Crates e direção permitida.** O Cargo impõe a direção, e um ciclo não
compila.

```text
kinein-protocol     tipos do fio e a tabela de métodos             (sem dependência de projeto)
kinein-base         platform, fsops, kinein_dir, events, framing   -> protocol
kinein-process      o lançador: Command, PTY, confiança, prazo     -> base
kinein-core         domínios + borda (rotas, runtime, rpc)         -> base, process, protocol
kinein-adapter-*    adaptadores de periferia, processos próprios   -> protocol (o deles)
```

Os domínios ficam dentro do `kinein-core`, com estado privado por módulo. O
crate por domínio fica fora: sem necessidade medida, ele só custaria tempo de
build.

### 5.2 O que cada componente decide

| Componente | Decide | Nunca decide |
| --- | --- | --- |
| QML | apresentação e intenção | fato, regra, caminho efetivo, confiança |
| Ponte (C++) | transporte, correlação, prazo do pedido, vida da resposta | regra de domínio |
| Borda do core | rota, classe de execução, prazo padrão, cancelamento, registro | regra de domínio |
| Domínio | regra e estado do domínio | como se cria processo, como se lê o `.kinein` |
| Lançador | origem, confiança, ambiente, prazo, teto de saída, grupo de processos | regra de domínio |
| `kinein_dir` | caminho, classe e esquema de cada arquivo próprio | conteúdo de domínio |

### 5.3 O modelo de execução

A tabela de rotas do ENG-10 ganha a classe de cada método:

```rust
pub(crate) enum Execution {
    /// Só memória do domínio; roda no laço. Orçamento do RNF-EXE-01.
    State(fn(&mut Ctx, Request) -> Response),
    /// Processo, disco do projeto ou rede: fora do laço, com prazo,
    /// cancelamento e fila por recurso. Volta ao laço só para aplicar.
    Work(fn(&Ctx, Request) -> Plan),
    /// Operação longa e visível: um Job (o JobManager de hoje).
    Job(fn(&mut Ctx, Request) -> Response),
}
```

O `Ctx` é o contexto que o roteador monta para a rota (§5.8): o estado do
domínio dela e os serviços que ela declara, nunca o `Core` inteiro. O nome e
a forma exata saem da prova do B3 (§9).

**A forma do handler de trabalho:**
1. **Preparar, no laço:** uma cópia imutável do que o trabalho precisa,
   mais a geração do workspace e do documento.
2. **Trabalhar, fora do laço:** uma função pura, mais o lançador.
3. **Aplicar, no laço:** a continuação de hoje (`Continuation`). Ela só
   aplica se a geração ainda for a mesma. Senão, devolve o erro
   `ContentModified`, como o LSP.

O ajudante tipado das três fases torna a migração de cada handler mecânica.

**Fila por recurso.** Dois trabalhos sobre o mesmo recurso (o repositório
git, a pasta de build, o documento) rodam em série. Recursos diferentes rodam
em paralelo. O pool tem tamanho fixo (os núcleos), sem thread por pedido.

**Prazo e cancelamento:**
- Toda rota tem prazo padrão pela classe, e o pedido pode pedir outro.
- `core.cancel { id }` cancela qualquer pedido, como o `$/cancelRequest` do
  LSP.
- O pedido cancelado responde `RequestCancelled`, e o prazo vencido,
  `DeadlineExceeded`.
- O trabalho cancelado mata o grupo de processos pelo lançador. O
  `OwnedChild` de hoje já faz SIGTERM e depois SIGKILL.

**A guarda do laço.**
- A thread do laço tem uma marca.
- O lançador, o leitor de arquivo do projeto e a espera de LSP/DAP conferem
  a marca. Nos testes e no build de depuração, E/S no laço é falha imediata,
  com o método no texto.
- No release, vira um evento no registro.
- O ponto não é medir depois: uma regressão para o laço reprova no primeiro
  teste que a exercita.

**Testes.** O runtime real, com threads e canais, roda nos testes por um
`TestRuntime` sobre tubos em memória. As ferramentas falsas (o `fake_exe`
existente) ganham trava controlada pelo teste, para forçar intercalações sem
`sleep` (R-TEST-03). O modo em linha de hoje (`run_json_lines`, sem canais)
deixa de ser o caminho dos testes de integração.

### 5.4 O lançador

O lançador é o único dono de `std::process::Command` e de
`portable_pty::CommandBuilder` (o terminal, em `terminal/session.rs`). O
`clippy.toml` do workspace proíbe os dois, e o `kinein-core` declara
`disallowed_methods = "forbid"` (§2.7).

```rust
pub struct Launch {
    program: Program,          // IdeTool(detectada) | FromProject(caminho) | ProjectConfigured(ferramenta)
    args: Vec<OsString>,
    cwd: PathBuf,
    env: Environment,          // a lista permitida do owned_child
    deadline: Duration,        // obrigatório
    output_cap: usize,         // obrigatório
    cancel: CancelToken,
}
```

- **Origem e confiança.**
  - `FromProject` e `ProjectConfigured` exigem um workspace confiável.
    Sem confiança, o lançamento volta `WorkspaceUntrusted { capability }`
    sem criar processo.
  - O `git` é `ProjectConfigured`: o `.git/config` define `fsmonitor`,
    filtros e hooks.
  - O `cargo` também é: o `.cargo/config.toml` define o `rustc` e o wrapper.
  - O interpretador do `.venv` é `FromProject`.
  - O compilador fixado por caminho no kit é `FromProject` enquanto o caminho
    vier de um arquivo da árvore do projeto (E5). Com o §5.9, o caminho de
    instalação passa a morar no estado da pessoa, fora da árvore.
- **Argumento que nomeia programa.** O servidor de linguagem recebe
  caminhos que ele mesmo executa (`--query-driver`, `pythonPath`). Esses
  argumentos passam pela mesma regra de origem: sem confiança, o lançador
  recusa o caminho que veio do projeto.
- **Registro.** Todo lançamento abre um span com programa, origem, decisão,
  duração, código de saída e se o prazo venceu.
- **Testes.** A API do lançador para testes cria repositórios e
  ferramentas falsas. Os testes também deixam de chamar o `Command` direto.
- **O git interno.** Todo comando do git que a IDE roda sozinha (status,
  log, blame, diff de fundo) sai do lançador com a mesma forma, como o Zed
  faz em `crates/git/src/repository.rs` (§5.15):
  - `--no-optional-locks`: o status de fundo não disputa o `index.lock` com
    o terminal da pessoa;
  - `--no-pager` e `GIT_TERMINAL_PROMPT=0`, posto depois do ambiente do
    projeto: comando de fundo nunca espera entrada;
  - `-c core.fsmonitor=false` e `-c log.showSignature=false` sempre, como lá.

  Sem confiança, o Git segue desligado (§5.5, DP-02), e a forma acima não
  substitui o desligamento. O Zed, sem confiança, roda o git com uma lista
  de chaves neutralizadas (`core.hooksPath`, `credential.helper`,
  `diff.external` e outras). Pela leitura do código no `703b5ac`, a lista
  não cobre os filtros `filter.<nome>.clean` e `smudge`, um dos caminhos do
  E2 (§2.1). Uma lista de chaves é aberta por natureza; o desligamento é
  fechado.

### 5.5 O modo restrito

| Capacidade | Sem confiança | Com confiança |
| --- | --- | --- |
| Ler, navegar, editar, buscar | sim | sim |
| Índice e sintaxe (tree-sitter, sem processo) | sim | sim |
| clangd | sobe, com `--query-driver` só de compilador detectado pela IDE; o caminho fixado no projeto fica de fora (E5) | sobe, com o kit inteiro |
| rust-analyzer | não sobe | sobe |
| basedpyright com o interpretador do projeto | sobe sem o interpretador do projeto (DP-02): o `pythonPath` do projeto fica de fora (E6) | sobe |
| Git (status, log, blame, diff) | desligado, com aviso "confie no projeto para ver o Git" | sim |
| `cargo`, `cmake`, build, testes, executar, depurar, gravar | pede confiança no gesto (DP-02) | sim |
| `--version` de interpretador do projeto | não | sim |

**A regra mora no core, no lançador.** A QML só mostra o estado e pede o
gesto. O configure automático que a QML segura hoje passa a ser recusado pelo
core. A R-ARQ-12 já pede isso ("a UI não repete regra do core").

A confiança segue no estado global (`trusted-projects.json`, pelo caminho
canônico da raiz), como hoje. O plano acrescenta as "pastas confiáveis"
do IntelliJ, que tornam confiáveis os projetos abertos de dentro delas
(DP-02).

### 5.6 A ponte

```text
CoreTransport (C++)          processo, quadros, ids, prazo, cancelamento, reinício
  └ CoreReply (QObject)      um por pedido: finished(result), failed(error), cancel()
<Dominio>Client (C++)        métodos tipados que devolvem CoreReply; eventos do domínio
<Dominio>Domain.qml          controladores sem IPC (como hoje)
```

- **O `CoreReply`.**
  - Segue o molde do `QNetworkReply`: o transporte é o dono e o apaga com
    `deleteLater` depois de entregar.
  - O pedido guarda o `QPointer` de quem pediu. Se quem pediu morre, o
    transporte manda `core.cancel`.
  - Vencido o prazo do lado da UI, o `failed` sai com `DeadlineExceeded`.
- **O que some:**
  - os mapas `m_pending*`;
  - os ramos por método em `core_client_dispatch*.cpp`;
  - o `requestFailed` difundido para 17 roteadores;
  - o `requestId` enxertado sinal a sinal.
- **Eventos:** cada `<Dominio>Client` assina os eventos do domínio pelo
  nome, por tabela.
- **Quem alcança a ponte:** a QML de área não a recebe. Só os roteadores a
  recebem, como propriedade obrigatória. Fora deles, a referência não
  existe, e o ENG-22 vira estrutura, não verificação.

### 5.7 O contrato com fonte única

O `kinein-protocol` declara a tabela de métodos uma vez: nome, tipo dos
parâmetros, tipo do resultado, classe de execução padrão e domínio. Dela
saem:

1. **No Rust:** um teste confere que cada método tem rota, com a classe
   declarada. Rota sem método não compila.
2. **O JSON Schema:** `schemas/` é gerado do código com `schemars`. Um passo
   do gate confere que o gerado está igual ao versionado, e o schema nunca
   mais fica parado.
3. **No C++:** um cabeçalho gerado com o nome de cada método, os campos de
   parâmetro e a tabela de domínio. A ponte do `<Dominio>Client` usa as
   constantes, e um nome errado não compila.
4. **Nos harnesses:** o cliente falso gerado valida cada pedido e cada
   resposta de mentira contra o schema. Um dublê que diverge do contrato
   reprova o harness. Essa conferência substitui a catraca de sinais
   redeclarados do ENG-44.

A QML continua dinâmica. O que se ganha é nome e forma conferidos na borda,
não tipo completo. O §6, item 7, trata o resto.

### 5.8 O estado e a troca de workspace

- **O `Core` sai da raiz do crate (R-ARQ-07 e R-ARQ-08 do 63, D-28).** No
  Rust, um item privado é visível no módulo que o define e em todos os
  descendentes. Com o `Core` definido no `lib.rs`, todo módulo do crate
  descende da raiz e vê todos os 32 campos (§2.8): a regra antiga ("`impl
  Core` só no `lib.rs`") juntava as funções num arquivo, mas deixava o estado
  aberto ao crate inteiro.
- **A forma adotada:**
  - o `Core` mora em módulo próprio (`core/`), com campos privados;
  - cada domínio é dono do próprio estado, em tipo próprio com campos
    privados no módulo dele;
  - o roteador entrega ao handler, por parâmetro, só o estado do domínio
    dele e os serviços que a rota declara (lançador, jobs, eventos).

  É o padrão do `State` com `FromRef` do axum: o handler declara o que
  recebe, e quem monta o contexto é um ponto só. O compilador impõe a
  separação; nenhum gate de texto é preciso para ela.
- **O que não muda:** os testes IPC (`handle_request`) seguem a superfície
  de hoje; a mudança é interna ao roteador e aos handlers.
- A troca de workspace é um gancho de domínio (`workspace_changed`, no
  §C.2 do 63). `activate_workspace` e `deactivate_workspace` deixam de nomear
  o estado de todos.
- Uma geração por workspace e por documento descarta resultado velho (§5.3).

### 5.9 A persistência

**Três classes, pelo XDG Base Directory, mais a árvore de build.** No Windows, o `platform` já
resolve as pastas de configuração e de estado; a de cache entra com o B7.

| Classe | Onde | O quê |
| --- | --- | --- |
| Configuração do projeto, compartilhável | `<raiz>/.kinein/` | o que um colega deve receber pelo git |
| Estado da pessoa no projeto | `$XDG_STATE_HOME/kinein-vectis/projetos/<id>/` | sessão, rascunhos, histórico, escolhas locais |
| Cache do projeto | `$XDG_CACHE_HOME/kinein-vectis/projetos/<id>/` | derivado e recriável |
| Árvore de build | `<raiz>/.kinein/build`, `presets/<preset>` (convenção do CMake, como o `cmake-build-*` do CLion) | saída do build, com `.gitignore` próprio |

O `<id>` é o hash do caminho canônico da raiz, como o `workspaceStorage` do
VS Code e o banco do Zed, que guarda o workspace pelos caminhos locais
(`crates/workspace/src/persistence.rs`). Um arquivo `projeto.json` dentro da
pasta guarda o caminho, para a listagem e a limpeza de projetos que sumiram.

**O estado segue o projeto renomeado ou movido (D-28 do 63; D15 do 19).**
Só o hash do caminho perde o estado quando a pasta muda de nome. A solução:
- o `projeto.json` guarda também a identidade do diretório raiz:
  dispositivo, inode e data de criação (`statx`); no Windows, número de
  série do volume, identificador do arquivo e data de criação;
- ao abrir um caminho sem pasta de estado, o core procura uma pasta cujo
  caminho gravado sumiu e cuja identidade é a do diretório aberto, e a adota
  com o caminho novo;
- a adoção automática exige a identidade completa e o caminho antigo
  ausente; cópia, clone novo e movimento entre volumes começam do zero;
- a pasta que sobra vai para a lista de projetos que sumiram, com "adotar"
  e "remover"; nada é apagado sem o gesto da pessoa.

Um identificador gravado dentro do projeto (o `ProjectId` do JetBrains) foi
descartado: obrigaria a gravar na pasta ao abrir, viria de um arquivo que o
projeto controla e seria dividido entre cópias (D15 do 19).

**Classificação dos arquivos de hoje** (dono pelo código; a divisão campo a
campo dos arquivos mistos é o primeiro passo do B7):

| Arquivo sob `.kinein` | Dono | Classe |
| --- | --- | --- |
| `settings.json` | `settings.rs` | projeto |
| `runconfigs.json` | `runconfig.rs` | projeto; o `activeId` é estado da pessoa |
| `toolchain.json` | `kits/store.rs` | dividir: kit, alvo e chip são do projeto; caminho de instalação é local, e sair da árvore fecha o E5 pela estrutura |
| `datasources.json` | `datasource/store.rs` | dividir como o `dataSources.xml` e o `dataSources.local.xml`; segredo segue no cofre |
| `grafana.json` | `grafana/store.rs` | projeto; o token segue no cofre |
| `remotes.json` | `remote/store.rs` | estado da pessoa (alvos SSH dela) |
| `remote-contacts.json` | `remote/contacts.rs` | estado da pessoa |
| `session.json` | `workspace/session.rs` | estado da pessoa |
| `kinein.db` (rascunhos) | `db/mod.rs` | estado da pessoa |
| `consoles/v1/` | `datasource/console.rs` | estado da pessoa (o JetBrains guarda consoles fora do projeto) |
| `python.json` | `python/mod.rs` | estado da pessoa (o interpretador escolhido é um caminho local) |
| `datasources.schema1.json` | `datasource/store.rs` | estado da pessoa (cópia da migração) |
| `workspace.json` | `workspace/open.rs` | deixa de ser gravado: é derivado, traz o caminho absoluto da raiz e nada o lê em produção (§2.1) |
| `coverage.lcov` | `coverage/mod.rs` | cache |
| `configuration.json`, marcas do configure (`rigor`) | `cmake_configuration.rs` | árvore de build |
| `toolchains/*.cmake` | `configaction/catalog` | projeto (o arquivo gerado que o build usa) |
| `remote-mirror.json` | `remote/mirror.rs` | fica na raiz do espelho, que já mora no cache |

**Migração.**
- Ao abrir, o core move o arquivo de estado ou de cache do lugar antigo para
  o novo, pelo guarda do `kinein_dir`: cópia, releitura e, só então, remoção.
- O `.kinein/.gitignore` passa a ser escrito pela IDE, com o que não é
  compartilhável.
- Abrir uma pasta sem confiança deixa de gravar nela: o `.kinein` só nasce
  quando a pessoa grava uma configuração do projeto.
- O arquivo que só muda de lugar mantém o esquema. O arquivo misto que se
  divide (o `toolchain.json`, o `datasources.json` e o `runconfigs.json`
  com o `activeId`) ganha versão nova de esquema nas duas metades. O
  binário novo lê a versão antiga; o binário anterior, diante da versão
  nova, avisa e preserva, pela R-DADOS-03 do 63. A DP-05 levantou a Parte F,
  item 3, do 63 para isso.

### 5.10 A observabilidade

- **Um span `tracing` por pedido,** aberto pelo roteador (ponto único) com:
  método, id, classe, espera na fila, tempo no laço, tempo total e desfecho.
  Também um span por lançamento (§5.4) e um por job.
- **O destino:** um arquivo JSON por linha em
  `$XDG_STATE_HOME/kinein-vectis/log/`, com rotação por tamanho, e o nível
  por `KINEIN_LOG`.
- **Sem segredo:** o Banco e o Remoto passam pelo mesmo filtro de redação do
  cofre.
- **Uso:**
  - a tabela do §2.2 passa a sair do registro de uso real, e não de um
    experimento;
  - os testes usam um assinante em memória para conferir os orçamentos do
    §4;
  - o 21 §M4.2 ganha o histograma por método.

### 5.11 A sintaxe incremental

- **O primeiro envio do documento é inteiro.** Depois, só as edições:
  `syntaxTree.didChange { path, version, edits: [{ range, text }] }`.
- **O core aplica `Tree::edit` e reanalisa de forma incremental**, fora do
  laço (classe trabalho, fila por documento).
- **A volta traz só as faixas que mudaram** (`changed_ranges`) e a versão.
  A resposta de versão velha é descartada nos dois lados.
- **A UI aplica faixas no realce.** O custo do lado da UI é medido antes e
  depois com o `KINEIN_PERF_TYPING*`.
- **Espera a DP-10 (§11).** O desenho acima vale para o motor de hoje (Qt
  Quick `TextEdit` com `QSyntaxHighlighter`, D5 do 19). Se a DP-10 trocar o
  motor, quem guarda o documento e manda as edições muda, e o B8 é
  reescrito sobre a decisão, em vez de ser feito duas vezes.

### 5.12 Os adaptadores

O DAP (`dap/adapter.rs`) e a gravação (`flash/mod.rs`) passam do `match` de
texto para uma tabela de descritores, como o `ServerSpec` do LSP. Um
adaptador novo vira uma entrada na tabela, com o teste de extensão da
R-TEST-05. Interface pública de plugin fica fora (§4).

### 5.13 A periferia fora do processo do core

- **O Banco segue a ADR-0010:** um processo adaptador por família de
  driver, com contrato versionado.
  - O core continua dono da seleção, da política, do segredo, do catálogo,
    dos jobs e da autorização de prévia.
  - O `kinein-adapter-sqlite` é o precedente.
  - O processo adaptador nasce pelo lançador (§5.4).
- **O `cargo-deny` proíbe os crates de driver** no `kinein-core`
  (RNF-ESC-01).

### 5.14 A verificação

**Ordem de preferência:**
1. **O compilador:** grafo de crates, privacidade, `forbid` de
   `disallowed_methods`, enum exaustivo.
2. **Artefato gerado conferido:** o schema e o cabeçalho C++.
3. **Teste de contrato:** os dublês validados.
4. **Gate de texto,** só onde os três primeiros não alcançam: QML, shell e
   docs.

**Os gates de texto restantes leem pelo tree-sitter.** São as mesmas
gramáticas que o produto já compila (C, C++, Rust e Python no `kinein-core`),
mais `tree-sitter-qmljs` e `tree-sitter-bash`, numa ferramenta Rust do
workspace. É o "um leitor por linguagem" da R-GATE-11, com gramáticas que o
ecossistema mantém, e não leitores próprios. A R-GATE-01 ("antes de qualquer
build") passa a permitir o binário da ferramenta, guardado pelo cargo
(DP-06).

### 5.15 O Zed como referência de engenharia

O Zed é um editor em Rust, de código aberto, que resolve em produção
problemas que a B também resolve: limites entre editor, projeto e
ferramentas; trabalho assíncrono sem prender a interface; documento com
análise incremental; ciclo de vida de LSP e de DAP; e testes determinísticos
de concorrência. Ele entra no plano como **referência**, não como molde.

**Fonte.** Código do repositório do Zed no commit `703b5ac` (2026-11-14),
lido e não executado. Os caminhos abaixo são desse commit.

**O que ele mostra, e onde a B usa:**

| Tema | O que o Zed faz | Onde | Uso na Kinein |
| --- | --- | --- | --- |
| Limites entre editor, projeto e ferramentas | O `Project` é um agregado de lojas por responsabilidade (`BufferStore`, `WorktreeStore`, `LspStore`, `GitStore`, `DapStore`, `TaskStore`, `ToolchainStore`). Cada loja é dona do próprio estado; as outras a alcançam por referência explícita | `crates/project/src/project.rs` | Confirma o estado por domínio com dono único (§5.8, ENG-15, B6) |
| Trabalho assíncrono sem prender a interface | Executor de primeiro plano (a interface) e executor de fundo. A tarefa é um valor: soltá-la cancela o trabalho, e manter trabalho sem dono exige `detach()` explícito | `crates/scheduler/src/executor.rs` (`Task`), `crates/gpui/src/executor.rs` | A classe de execução e o cancelamento por dono (§5.3, B3): trabalho que ninguém espera não continua rodando |
| Operação longa | O pedido ao servidor de linguagem cancelado por queda do dono manda `$/cancelRequest` sozinho | `crates/lsp/src/lsp.rs` (`cancel_on_drop`) | O `CoreReply` destruído gera `core.cancel` (§5.6, B5) |
| Documento e análise incremental | A reanálise tenta 1 ms em linha (10 ms nos testes) e, passado o prazo, continua no fundo, aplicando a árvore quando termina | `crates/language/src/buffer.rs` (`sync_parse_timeout`) | Orçamento explícito para o trabalho em linha e o resto fora (§5.11, RNF-EDI-01) |
| Sincronia com o servidor de linguagem | Guarda as últimas versões enviadas de cada documento (`OLD_VERSIONS_TO_RETAIN = 10`) para traduzir resposta de versão velha | `crates/project/src/lsp_store.rs` | Geração por documento e descarte do velho (§5.3), a decidir com a DP-10 |
| Ciclo de vida do DAP | Adaptador de depuração como implementação de um traço (`DebugAdapter`), registrado por nome | `crates/dap/src/adapters.rs` | Tabela de descritores (§5.12, B10) |
| Confiança no projeto | Modo restrito por árvore de trabalho; o git interno com `core.fsmonitor=false` sempre e chaves neutralizadas sem confiança | `crates/project/src/trusted_worktrees.rs`, `crates/git/src/repository.rs` | O modo restrito (§5.5) e o git interno (§5.4); a Kinein desliga o Git sem confiança, em vez de neutralizar chaves |
| Testes de concorrência | Agendador de testes determinístico: `run_until_parked`, permissão explícita para estacionar e a semente `SEED` para repetir uma ordem | `crates/scheduler/src/test_scheduler.rs`, `crates/gpui/src/test.rs`, `crates/gpui/src/platform/test/dispatcher.rs` | O `TestRuntime` (§5.3) e a camada de concorrência do 62 §3.5: a semente que reprovou é a que se repete |
| Estado do workspace | Banco SQLite com o workspace pelos caminhos locais | `crates/workspace/src/persistence.rs` | A chave pelo caminho (§5.9), com o rastreio de identidade que o Zed não tem |

**O que não se transpõe sem validar na Kinein.** A Kinein tem dois
processos e uma fronteira IPC; o Zed é um processo com um runtime próprio
(`gpui`). Por isso:
- a tarefa que cancela ao cair vale, na Kinein, do lado da ponte (o
  `CoreReply`) e do lado do core (o trabalho fora do laço); não há
  `gpui` nos dois lados;
- o agendador determinístico do Zed controla todas as tarefas do processo;
  o `TestRuntime` da Kinein controla o core, e a UI segue testada pelos
  harnesses;
- o orçamento de 1 ms da análise em linha é número do Zed, não da Kinein: o
  valor daqui sai da medida do B1;
- toda técnica tirada do Zed entra pela prova da fase dela (§9), com o
  mesmo aceite medido de qualquer outra.

## 6. Os riscos da estratégia B, e a correção de cada um

| # | Risco | Correção | O que resta |
| --- | --- | --- | --- |
| 1 | B mexe na concorrência, onde os testes de hoje são cegos: sem canais, tudo roda em linha (`runtime/services.rs`) | `TestRuntime` com o runtime real; ferramentas falsas com trava; guarda de E/S no laço; a camada "ordem temporal e concorrência" do [62 §3.5](62-base-publica-0.4.md) com repetição em lote | O agendamento real do sistema não é determinístico; a repetição em lote mede a intermitência, como o INF-22 já mostrou |
| 2 | Trabalho fora do laço abre corrida de estado | Fila por recurso; geração por workspace e documento; aplicar só no laço; `ContentModified` para o velho | Mais estados para pensar; a classe declarada na rota os confina |
| 3 | ~106 handlers `&mut self` a reescrever | O ajudante das três fases; migração por domínio, os testes da superfície IPC intactos (R-TEST-07); primeiro os infratores medidos (§2.2, §2.3) | Volume de migração; a ordem do §8 o divide |
| 4 | ~122 locais de processo, e a fuga pelo `allow` | `forbid` (§2.7): a lista de erros do compilador é a lista exata da migração, e a fuga não compila | O PTY (`portable_pty::CommandBuilder`) entra na mesma lista |
| 5 | Crates novos aumentam o tempo de build | Dois crates (`base`, `process`), não um por domínio; `cargo build --timings` antes e depois, com veredito | Mudança de visibilidade (`pub(crate)` para `pub`) |
| 6 | O `CoreReply` cria vida de objeto na QML | Dono em C++ (`deleteLater`), `QPointer` de quem pediu, cancelamento quando quem pediu morre | Os dublês precisam emular a resposta; o dublê é gerado do contrato (§5.7) |
| 7 | A QML é dinâmica, e o contrato gerado não a tipa | Validação de schema na borda nos harnesses e no build de depuração; constantes geradas para os nomes | Erro de forma na QML aparece no harness, não no build; o harness da área é obrigatório (R-IPC-02) |
| 8 | O documento de requisitos vira outro texto longo | Uma tabela; uma linha por requisito; a coluna "quem impõe" preenchida ou NÃO VERIFICADO explícito; o dono é o 19 | Os valores foram aceitos na DP-03 e só mudam por decisão datada |
| 9 | B atrasa a etapa C | As primeiras fatias (registro, lançador e confiança, classes de execução) tocam os domínios que o C1 usa (cmake, build, test, índice, toolchain); o C1 retoma sobre as costuras novas, sem refazer depois | Atraso de calendário, aceito pela decisão do autor |
| 10 | Dois planos (63 e 64) contradizem o D-01 | Este documento diz, item a item, o que substitui (§7). Com a DP-01, o 63 fica com o regulamento (Parte A) e aponta para cá nas Partes C e E | Até o B0 reconciliar o texto do 63, ele tem a nota de situação no topo |
| 11 | "Toda dívida" com linha de chegada móvel: o leitor melhor revela mais dívida (D-22) | O compilador conta a dívida das regras que passam a ele, e a conta é exata e finita desde o primeiro dia; os gates de texto restantes leem pelo tree-sitter | Nada da dívida sai da 0.4; o ENG-37 (identificadores) vem por último, pelo rename do rust-analyzer (R-PROC-03), depois que a estrutura assenta |
| 12 | A ferramenta do gate precisa de build antes dos passos estáticos | O binário fica no cache do cargo; os passos estáticos rodam em paralelo depois dele; o custo é medido no ENG-03 | A primeira rodada num checkout limpo paga o build da ferramenta |
| 13 | A Parte F do 63 congela o fio e os arquivos gravados | O protocolo é 0.x e a UI e o core saem juntos (D10 do 19): o fio muda no mesmo commit, com o contrato; arquivo gravado muda de lugar com migração, cópia e releitura | A DP-05 levantou os dois itens; o texto da Parte F mudou no B0 |
| 14 | O D-27 alarga a 0.4: o motor de edição, que ficava para depois, entra nela | A DP-10 decide com avaliação e prova isolada (§9); só as fases que dependem do motor esperam (§8); B1, B2 e B3, que fecham segurança, concorrência e IPC, não esperam | Calendário mais longo, aceito pela decisão do autor; o risco de a prova consumir o tempo das fases críticas é tratado pela ordem: a prova não toca o produto e não passa na frente do B2 |
| 15 | Trocar o motor de edição muda o domínio do editor, a sincronia da sintaxe e o dono do cliente LSP | O B4 (contrato), o B5 (piloto da ponte), o B6 (domínio `lsp`), o B8 e o ENG-24 começam depois da DP-10 na parte do editor; o resto deles segue | Se a DP-10 trocar o motor, parte do LSP do core vira outra coisa; o código de produção só sai quando a substituição estiver validada |
| 16 | Uma técnica do Zed copiada sem medir | Toda técnica do §5.15 entra pela prova da fase, com aceite medido | O Zed é um processo; a Kinein são dois (§5.15) |

## 7. O que acontece com cada item do 63

| Item do 63 | Destino na estratégia B |
| --- | --- |
| ENG-01 (ferramenta de mutação) | fica; os catálogos ganham os mutantes de cada item da §8 |
| ENG-02 (cada dono monta o caminho do `.kinein`) | absorvido pelo catálogo com classe (§5.9) e pelo `forbid` de leitura fora do guarda |
| ENG-03, ENG-05, ENG-06, ENG-07 | ficam, independentes da arquitetura |
| ENG-10 (tabela de rotas) | fica, com a classe de execução (§5.3) |
| ENG-11 (reações), ENG-13, ENG-14, ENG-15, ENG-17 | ficam (§5.8) |
| ENG-12 (camadas) | a base passa a crate (§5.1), e o grafo do Cargo impõe a R-ARQ-02; a borda e os domínios seguem no mesmo crate, e a R-ARQ-03 e a R-ARQ-04 seguem no `module_map.py`, lendo pelo tree-sitter (R-GATE-11, R-GATE-13) |
| ENG-16 (tipo do fio como tipo interno) | fica a decisão D-07 |
| ENG-20 (`CoreClient` por domínio) | fica, com o `CoreReply` e o contrato gerado (§5.6, §5.7); a parte do editor espera a DP-10 |
| ENG-21 a ENG-25 | ficam; o ENG-22 vira estrutura (§5.6); o ENG-24 (a fachada do `EditorController.qml`) espera a DP-10 |
| ENG-30 a ENG-36, ENG-38 | ficam |
| ENG-37 (identificadores) | fica, por último no Rust, pelo rename do rust-analyzer |
| ENG-39 (adiados do Windows) | fica na etapa 10 do 62 §3 |
| ENG-40 (linha de base sem folga, só desce) | fica a regra (R-GATE-09 e R-GATE-10), aplicada só às catracas de texto que sobram |
| ENG-41 (um leitor por linguagem) | trocado: o tree-sitter na ferramenta do gate (§5.14) |
| ENG-42 (gates que leem Rust) | trocado pelo compilador onde a regra é de estrutura; o que sobra lê pelo tree-sitter |
| ENG-43 (QML e C++) | fica, lendo pelo tree-sitter |
| ENG-44 (sinais redeclarados da ponte) | trocado pelo dublê gerado e validado (§5.7) |
| ENG-45 (gate falha fechado), ENG-46 (arquivos que nenhum gate lê) | ficam, independentes da arquitetura |
| ENG-47 (leitura sem dono) | trocado pelo `forbid` de `std::fs::read*` e `File::open` fora dos leitores guardados; os 6 pendentes do D-21 morrem por ele |
| INF-06, INF-16, INF-19, INF-22 | ficam; o INF-16 (mapa de módulos) diminui com as camadas por crate |

## 8. A ordem, com aceite e reversão

Cada fatia é um item no 62 §3.4 e um commit (R-PROC-02), com o gate rápido
verde. Sempre que uma fatia criar uma guarda, ela leva um mutante no
catálogo. A ordem segue a dependência: medir antes de mexer, e a segurança
primeiro.

| Fase | O quê | Aceite objetivo | Reversão |
| --- | --- | --- | --- |
| B0 | Decisões do §11; requisitos no 19; 63 reconciliado | As DP-01 a DP-09 decididas e registradas; os gates de docs verdes | Documento |
| B1 | Registro (`tracing`, §5.10) e a guarda do laço (§5.3), sem mudar comportamento | A tabela do §2.2 sai do registro; um `Command` plantado num handler de estado reprova um teste pela guarda | Desligar o assinante; a guarda é só de teste e depuração |
| B2 | O lançador e o modo restrito (§5.4, §5.5); fecha o INF-23 primeiro, nos cinco vetores, e migra os ~122 locais | O projeto hostil do §2.1 é fixture: zero marcas para o `workspace.open`, para cada pedido automático da ponte e para o primeiro arquivo de cada linguagem; `forbid` ativo; `Command::new` plantado não compila | Reverter por domínio; o `forbid` só entra quando a lista do compilador chega a zero |
| B3 | Classes de execução, prazo, `core.cancel` e fila por recurso (§5.3); migração dos infratores (`git.*`, `setup.list`, as sondas do `workspace.open`, `debug.*`) e depois do resto | O experimento do §2.3 é teste: `core.ping` e cancelamento em ≤ 50 ms com o git falso travado; RNF-EXE-01 no registro | A rota volta para a classe estado (é uma linha na tabela) |
| B4 | Contrato com fonte única (§5.7) | Mutantes: método renomeado só no Rust reprova o build C++; campo trocado num dublê reprova o harness; schema velho reprova o gate | O gate de fiação de hoje segue até a geração cobrir todos os domínios |
| B5 | A ponte com o `CoreReply` e os clientes por domínio (§5.6): piloto com dois domínios, depois todos | Dois pedidos iguais e simultâneos chegam cada um a quem pediu (harness); quem pediu destruído gera `core.cancel`; os mapas `m_pending*` e os ramos por método somem do domínio migrado | O piloto vive ao lado do `CoreClient` até o domínio fechar |
| B6 | Estrutura do core: ENG-11, ENG-13 a ENG-15 e ENG-17, com os crates do §5.1 | Um import da base para um domínio não compila; o `Core` sem campo de domínio público; `cargo build --timings` antes e depois | Mover de volta é mecânico; os testes IPC não mudam |
| B7 | Persistência em três classes, com migração (§5.9) | Projeto com os arquivos antigos abre, migra com cópia e releitura e não perde nada (testes de adversidade do R-DADOS); nenhum arquivo de pessoa sob `.kinein`; `.kinein/.gitignore` gerado | O binário anterior lê o lugar antigo; a cópia fica até a releitura confirmar |
| B8 | Sintaxe incremental (§5.11) | RNF-EDI-01 e RNF-MEM-01 medidos e no teste; `KINEIN_PERF_TYPING*` sem regressão | O pedido inteiro de hoje fica como caminho de reserva até a UI migrar |
| B9 | O Banco fora do processo (ADR-0010, §5.13) | Dependências do core sem os drivers (`cargo tree`, `cargo-deny`); os testes do Banco pelo adaptador; o adaptador morto não derruba o core | O adaptador por família pode voltar a ser biblioteca, se o contrato provar caro |
| B10 | Descritores para DAP e gravação (§5.12) | Teste de extensão: um adaptador de mentira entra só pela tabela | Mecânica |
| B11 | O resto do 63: ENG-21 a ENG-25, ENG-30 a ENG-36, ENG-38, ENG-03, ENG-05, ENG-06, ENG-45, ENG-46; ENG-37 por último | O aceite de cada item no 63 | O de cada item |

**Estado do B0 em 2026-11-14: pronto.** As nove decisões estão tomadas; os
requisitos moram no 19, com a D13 à D18; o 63 tem as regras da DP-09 na
Parte A, a Parte F com a DP-05, a R-GATE-01 com a DP-06, a R-ARQ-07 e a
R-ARQ-08 emendadas (D-28) e as Partes C e E apontando para cá.

**A DP-10 segura só a parte do editor.** Até ela ser decidida:
- B1, B2 e B3 seguem sem esperar. São eles que fecham as falhas críticas de
  segurança, concorrência e IPC, e nenhum depende do motor de edição;
- B4, B5 e B6 seguem nos outros domínios: a geração do contrato, o piloto
  da ponte e os crates não escolhem o editor nem o domínio `lsp` como
  piloto;
- B7, B9 e B10 seguem;
- B8, o ENG-24 e a parte do editor do ENG-20 esperam.

A avaliação do ENG-60 e a prova de conceito correm fora do produto (§9) e não
passam na frente do B2.

Depois do B11, a etapa C retoma do C1-47. O ENG-39 fica na etapa 10 do 62
§3.

**Independentes da decisão, podem correr em qualquer ponto:** ENG-07 (CI,
quando a conta destravar), ENG-45, ENG-46, INF-22 e os gates de docs do
ENG-06.

**Adiados para não fazer duas vezes:** ENG-40 a ENG-44, o ENG-20 na forma
atual e o ENG-37.

## 9. As provas

**Feitas nesta revisão:** os experimentos do §2.1 ao §2.6 e o protótipo do
§2.7. Na fase que os cobre, cada experimento vira teste permanente: o §2.1
no B2, o §2.3 no B3, o §2.4 no B8.

**Provas de conceito antes de cada fase estrutural:**

| Prova | Pergunta que responde | Quando |
| --- | --- | --- |
| A classe de execução no domínio `git`, com o git falso travado | O ajudante das três fases cabe sem mudar os testes IPC do git? | início do B3 |
| O `CoreReply` em dois domínios | Quantos arquivos e linhas custa um método novo nas duas formas? | início do B5 |
| A tabela de métodos gerando schema e cabeçalho para um domínio | A geração cobre os tipos do protocolo sem exceção manual? | início do B4 |
| Os crates `base` e `process` | Quanto muda o tempo de build e de teste? | início do B6 |
| A ferramenta do gate com tree-sitter lendo QML | O `tree-sitter-qmljs` lê os 447 arquivos QML sem erro de análise? | antes de trocar o ENG-41 |
| O motor de edição (ENG-60) | Um motor candidato cabe na UI Qt Quick, na confiança do core e nos fluxos do editor sem regressão? Os critérios de refutação são escritos antes da prova, e a prova vive fora do produto | antes da DP-10 |

**Fluxos verticais que ficam equivalentes,** com os testes de despacho que já
existem:
- abrir um projeto CMake C, configurar, compilar e ver o diagnóstico;
- editar com LSP (servidor falso);
- sessão DAP com pilha e variáveis;
- terminal: abrir, escrever e redimensionar;
- proposta e gravação com ferramenta falsa;
- prévia de consulta do Banco.

## 10. Preservar, refatorar, substituir, investigar

| Categoria | Componentes |
| --- | --- |
| **Preservar** | Separação UI/core e o fio JSON-RPC; `kinein-protocol` e os códigos de erro; JobManager; `owned_child`; `kinein_dir` e `fsops`; gerenciador LSP e sessão DAP; a lógica de `flash`, `toolchain`, `build`, `cmake_presets` e `serial`; o terminal (§2.5); os testes da superfície IPC; os controladores QML e os harnesses; a ferramenta de mutação; os lints estritos; a regra "o CMake é a verdade" |
| **Refatorar** (mesmo desenho, outro lugar) | Despacho para a tabela de rotas com classe; `Core` para estado por domínio; troca de workspace por gancho; camadas para crates; a confiança da QML para o core; DAP e gravação para descritores |
| **Substituir** | Bloqueio por padrão no laço; sinal difundido por resultado; contrato espelhado à mão e o schema parado; `Command::new` espalhado; o `.kinein` como pasta única; o `syntaxTree.update` inteiro; os drivers de banco no processo; os gates de texto para o que o compilador impõe |
| **Investigar** | O custo do realce na UI (o JSON de 1 MiB por tecla não foi medido do lado da UI); o motor de edição e o dono do documento e do cliente LSP, decididos na 0.4 pela DP-10, com o ENG-60 (D-27); a divisão campo a campo dos arquivos mistos do `.kinein` (B7) |

## 11. As decisões do autor

Decisão do autor de 2026-11-14: as nove primeiras foram aceitas como
recomendadas. A DP-10 nasceu no mesmo dia, do D-27 do 63, e está aberta.

| ID | Pergunta | Decisão (2026-11-14) |
| --- | --- | --- |
| DP-01 | Onde mora o plano da B? | Neste documento. O 63 fica com a Parte A (regulamento) e aponta para cá nas Partes C e E |
| DP-02 | O modo restrito: o gesto (Configurar, Compilar, Executar, Depurar) sem confiança pede confiança ou executa? O basedpyright sem confiança sobe sem o interpretador do projeto ou não sobe? Pastas confiáveis? | O gesto sem confiança pede confiança, como no VS Code e no IntelliJ. O basedpyright sobe sem o interpretador do projeto. As pastas confiáveis entram |
| DP-03 | Os requisitos do §4 e seus valores | Aceitos. A tabela vai ao 19 no B0 |
| DP-04 | A ordem da §8, com o INF-23 no início do B2 (um bloqueador real fura a fila, regra 6 do [00](../00-comece-aqui.md)) | Aceita |
| DP-05 | Levantar a Parte F do 63, itens 1 (o fio) e 3 (os arquivos gravados), para os itens da B, com contrato primeiro e migração com cópia | Aceito |
| DP-06 | Verificação: o compilador primeiro, o tree-sitter na ferramenta do gate no lugar dos leitores próprios do ENG-41, e a R-GATE-01 emendada | Aceita |
| DP-07 | Camadas de suporte (núcleo e periferia) e a ADR-0010 dentro da 0.4 (B9) | Aceitas |
| DP-08 | A persistência em três classes, com o estado da pessoa e o cache fora da árvore do projeto | Aceita. Emenda do mesmo dia (D-28 do 63): o estado segue o projeto renomeado ou movido no mesmo volume, pela identidade do diretório (§5.9) |
| DP-09 | Novas regras na Parte A do 63: classe de execução na rota, só o lançador cria processo, a confiança mora no core, todo pedido tem span, a classe de cada arquivo próprio | Aceitas. Entraram na Parte A no B0 |
| DP-10 | O motor de edição da Kinein: o de hoje (Qt Quick `TextEdit`, D5 do 19) ou outro, e quem é dono do documento e do cliente LSP | **Aberta.** A avaliação é o ENG-60 do 62 §3.4, com prova de conceito isolada (§9). O candidato preferido do autor é o Neovim como motor embutido, com a decisão orientada por evidência. A avaliação reabre de forma explícita a exclusão do Neovim embutido do mandato de 2026-07-11 ([21](21-roadmap-de-longo-prazo.md) §M5.4), e a tensão entre o modo modal e a identidade JetBrains (RNF8 do 19) é critério de refutação. A migração do produto só começa com aprovação explícita do autor depois da prova |

## 12. O que não foi medido

- O custo do lado da UI no `syntaxTree.update`: leitura do JSON e realce.
- O tempo de build com os crates novos (prova do B6).
- O custo de migrar a ponte (prova do B5).
- O servidor do ruff com uma configuração hostil no projeto: ele sobe sem
  confiança (§2.1), e o efeito da configuração não foi exercitado. O aceite
  do B2 cobre cada servidor que sobe sem confiança.
- Os outros campos do kit que nomeiam programa ou comando (`debugServer`,
  `toolchainFile`): são usados no gesto de depurar e de configurar, que o
  modo restrito passa a segurar (DP-02). Não foram exercitados.
- Os pedidos que a QML manda sozinha ao abrir, além dos cinco da ponte
  citados no §2.1: o aceite do B2 os enumera pelo registro do B1.
- O comportamento de outros comandos do `git` além do `status` com
  `fsmonitor` e filtros. Pela solução escolhida (Git desligado sem
  confiança), a medida não muda a decisão.
- Nada desta revisão rodou no Windows nem no contêiner da CI.
- O projeto de referência é o próprio repositório. Projetos maiores (SDKs de
  embarcados com dezenas de milhares de arquivos) não foram medidos, e o B1
  passa a medi-los pelo registro.
- O Zed (§5.15) foi lido, não executado. Nenhum número dele foi medido
  aqui; o comportamento do git dele com um filtro hostil também não.
- O rastreio de identidade do §5.9 não foi prototipado: a estabilidade do
  inode e da data de criação depois de um `mv` no mesmo volume é da
  documentação do `statx` e do Windows, e o B7 a prova em teste nos
  sistemas de arquivos suportados.
- As chamadas de arquivo fora da lista do protótipo do §2.7 (`read_dir`,
  `metadata`, `copy`, `rename`, `remove_file`).

**Referências das soluções adotadas:**
- [JetBrains — Project security (Safe Mode)](https://www.jetbrains.com/help/idea/project-security.html)
- [VS Code — Workspace Trust](https://code.visualstudio.com/docs/editing/workspaces/workspace-trust)
- [VS Code — Workspace Trust para extensões](https://code.visualstudio.com/api/extension-guides/workspace-trust)
- [SonarSource — Securing developer tools: Git integrations](https://www.sonarsource.com/blog/securing-developer-tools-git-integrations)
- [Language Server Protocol 3.17 — cancelamento e `ContentModified`](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/)
- [XDG Base Directory Specification](https://specifications.freedesktop.org/basedir-spec/latest/)
- [Clippy — configuração e `disallowed_methods`](https://doc.rust-lang.org/clippy/configuration.html)
- [Qt — `QNetworkReply`](https://doc.qt.io/qt-6/qnetworkreply.html)
- [tree-sitter — análise incremental](https://tree-sitter.github.io/tree-sitter/using-parsers/)
- [`tracing`](https://docs.rs/tracing) e [`schemars`](https://docs.rs/schemars)
- [rust-analyzer — `untrustedWorkspaces` no `package.json` da extensão](https://github.com/rust-lang/rust-analyzer/blob/master/editors/code/package.json)
- [Zed — código-fonte](https://github.com/zed-industries/zed) (lido no commit `703b5ac`)
- [axum — `State` e `FromRef`](https://docs.rs/axum/latest/axum/extract/struct.State.html)
- [Linux — `statx(2)`](https://man7.org/linux/man-pages/man2/statx.2.html) e [Windows — `GetFileInformationByHandle`](https://learn.microsoft.com/windows/win32/api/fileapi/nf-fileapi-getfileinformationbyhandle)
- [JetBrains — árvores de trabalho do git e o `ProjectId`](https://www.jetbrains.com/help/idea/use-git-worktrees.html)

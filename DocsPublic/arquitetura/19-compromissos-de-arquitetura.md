# 19 — Requisitos e trade-offs de arquitetura

> **Status:** ativo
> **Prioridade:** referência permanente (ler antes de decisão estrutural)
> **Fonte de verdade:** os requisitos e o *porquê* das decisões. O *como* está em
> `DocsPublic/arquitetura/ARCHITECTURE.md`; o *o quê/quando* no
> [mapa de versões](../roadmaps/57-mapa-de-versoes-ate-a-1.0.md) e em `DocsPublic/roadmaps/55-backend-para-ui-ux.md`; o visual-alvo em `DocsPublic/especificacoes/`
> **Ultima revisao:** 2026-11-14, pela revisão da arquitetura
> ([64](../roadmaps/64-revisao-da-arquitetura-e-estrategia-b.md)): os
> requisitos não funcionais ganharam a tabela mensurável; a D3, a D4 e a D5
> foram revistas; a D9, a D10 e a D11 foram anotadas; a D13 à D18 são novas. O
> que não foi tocado segue com a data que traz (2026-07-09).

## Por que este documento existe

O projeto avançou rápido e as decisões de arquitetura, requisitos funcionais
e não funcionais ficaram implícitas em código e commits. Este doc as torna
explícitas — cada trade-off registra o ganho, o custo **aceito de forma
consciente** e o gatilho para revisitar. Pragmatismo aqui significa: a
solução mais simples que atende o requisito; complexidade nova só entra
comprando algo mensurável (desempenho, correção ou manutenção).

## Requisito inegociável: UI/UX dos specs

**A parte visual não é negociável.** Layout, espaçamento, cor, iconografia,
copy e comportamento visual seguem estritamente os `.md` de `DocsPublic/especificacoes/`
(entrada: `indice-das-especificacoes.md`). Funcionalidade pode ser fatiada,
adiada ou simplificada; o visual **não se inventa nem se "melhora" de
passagem** — divergência de spec é bug ou tarefa explícita com o usuário
ciente. (Reafirmado aqui em 2026-07-09.)

## Requisitos funcionais (resumo por área)

Detalhe e sequência vivem nos roadmaps (o [`21`](../roadmaps/21-roadmap-de-longo-prazo.md) para M4–M7); aqui só
o mapa com estado em 2026-07-09:

```text
RF1  Workspace: abrir/criar/fechar projeto, browse confinado      [feito]
RF2  Editor: abas, salvar, highlight sintático + semântico        [feito]
RF3  Arquivos: tree, criar/renomear/apagar, busca conteúdo/nome   [feito]
RF4  Build/Test/Quality como jobs canceláveis com Problems        [feito]
RF5  LSP: diagnostics, hover, completion, definition, references,
     rename, semantic tokens                                      [feito]
RF6  Run + terminal PTY integrados                                [feito]
RF7  Ambiente: detecção de ferramentas + scan como job            [feito]
RF8  Project Health mínimo (banner acionável)                     [feito]
RF9  Formatação orquestrada (rustfmt/clang-format, Ctrl+Alt+L)    [feito]
RF10 Code actions/quick fixes LSP                                 [M1.3]
RF11 Go-to-symbol arquivo/workspace                               [M1]
RF12 Sessão por workspace (reabrir abas)                          [M1]
RF13 CMake presets/targets + Cargo metadata/features na UI        [M2]
RF14 Run configurations persistidas                               [M2]
RF15 Debugger via DAP (lldb-dap/gdb)                              [M2]
RF16 Git integrado (status/diff/stage/commit)                     [M3]
RF17 Settings com schema/migração + Strict/Balanced/Relaxed       [M4]
```

O mapa acima é o registro de 2026-07-09. O estado vivo de cada área na
versão em curso está na tabela do
[62 §3](../roadmaps/62-base-publica-0.4.md) e no
[mapa de versões](../roadmaps/57-mapa-de-versoes-ate-a-1.0.md).

## Requisitos não funcionais (explícitos a partir de agora)

```text
RNF1 Privacidade: 100% local/offline; ZERO telemetria; nada sai da
     máquina sem ação explícita do usuário (leitura-tecnica.md §1).
RNF2 Segurança de workspace: todo acesso a arquivo pelo core é
     canonicalizado e confinado à raiz aberta; a UI nunca toca disco.
     Sem a confiança da pessoa, nada que o projeto define é executado
     (RNF-SEG-01 e RNF-SEG-02, abaixo).
RNF3 Responsividade: a UI nunca bloqueia em trabalho pesado; operação
     longa vira job cancelável com eventos. O laço do core só faz
     trabalho de estado: processo, disco do projeto e rede rodam fora
     dele, com prazo e cancelamento (RNF-EXE-01 e RNF-EXE-02, abaixo).
RNF4 Robustez: erro estruturado no protocolo com mensagem humana;
     falha de ferramenta externa nunca derruba core nem UI; erros da IDE
     logados em ~/.cache/kinein-vectis/logs/. O core registra pedido,
     processo e job (RNF-OBS-01, abaixo).
RNF5 Manutenibilidade: strict mode máximo com gate único fix-first
     (Rust: clippy pedantic/nursery -D warnings, missing_docs; C++:
     clang-format/tidy Werror; QML: qmllint estrito zero warnings);
     domínios separados, handler fino, teste de comportamento no core.
RNF6 Portabilidade: Linux-first, agnóstico de distro (Arch é o alvo
     principal); bootstrap por scripts/instalar-ambiente.sh; nada de
     caminho hardcoded de distro em código.
RNF7 Performance percebida: startup rápido (LSP/ferramentas sob demanda,
     nunca no boot da UI); highlight semântico assíncrono com debounce.
     Orçamento formal de medição entra no M4 (21 §M4.2). Editar não
     custa o arquivo inteiro (RNF-EDI-01 e RNF-MEM-01, abaixo).
RNF8 UX: memória muscular JetBrains (atalhos/fluxos) + visual dos specs.
```

Até 2026-11-14, a RNF3 dizia "requests síncronos só quando comprovadamente
curtos (< ~300ms típicos, ex.: format.text)". Nada conferia esse limite, e a
medida daquele dia mostrou o laço preso pelo tempo da ferramenta
([64](../roadmaps/64-revisao-da-arquitetura-e-estrategia-b.md) §2.2 e §2.3;
ENG-50 e ENG-58 do [62 §3.4](../roadmaps/62-base-publica-0.4.md)).

### Requisitos mensuráveis (decisão do autor de 2026-11-14)

Cada requisito abaixo dá métrica, método de medida e dono da imposição a um
RNF acima. A revisão da arquitetura os mediu e propôs (64 §2 e §4), e a
decisão do autor de 2026-11-14 os aceitou (DP-03 do 64 §11). O valor de uma
métrica só muda por decisão datada, registrada nesta tabela.

**Referência de medida:**
- o clone do repositório no commit fixado pelo item;
- a máquina de medição do projeto;
- o contêiner da CI, quando o ENG-07 rodar.

| ID | Requisito | Métrica | Medida | Quem impõe | Estado em 2026-11-14 |
| --- | --- | --- | --- | --- | --- |
| RNF-EXE-01 | Trabalho de estado roda no laço; processo, disco do projeto e rede nunca rodam | Pedido da classe estado: p99 ≤ 10 ms e máximo ≤ 50 ms na referência | Os spans do registro, por método | Guarda de E/S no laço nos testes; a classe na rota (R-ARQ-14 do [63](../roadmaps/63-regras-e-plano-de-engenharia-da-0.4.md)) | Não cumprido (64 §2.2) e NÃO VERIFICADO; a guarda nasce no B1, e as classes, no B3 |
| RNF-EXE-02 | Todo pedido tem prazo e é cancelável | `core.ping` e o cancelamento respondem em ≤ 50 ms com uma ferramenta presa | O experimento do 64 §2.3 como teste | Teste com ferramenta falsa travada por trava, sem `sleep` | Não cumprido (64 §2.3) e NÃO VERIFICADO; B3 |
| RNF-SEG-01 | Sem confiança, nada que o projeto define é executado | Zero marcas no projeto hostil, para cada pedido automático da ponte e para o primeiro arquivo de cada linguagem | Os experimentos do 64 §2.1 como fixture | O lançador e o teste do projeto hostil (R-ARQ-15 e R-ARQ-16 do 63) | Não cumprido (INF-23) e NÃO VERIFICADO; B2 |
| RNF-SEG-02 | O que é executado diz de onde veio | Todo processo registrado com programa, origem e decisão de confiança | O span do lançador | O tipo do lançador: a origem é campo obrigatório | Não cumprido e NÃO VERIFICADO; B2 |
| RNF-OBS-01 | Todo pedido, processo e job deixa registro estruturado e sem segredo | 100% dos pedidos com span; zero segredo no registro | Teste com assinante de `tracing`; teste de redação de segredo | O registro do roteador, num ponto só (R-ARQ-17 do 63) | Não cumprido (ENG-52) e NÃO VERIFICADO; B1 |
| RNF-DADOS-05 | Estado da pessoa e cache não moram na árvore do projeto; a árvore de build segue a convenção do CMake | Zero arquivo por pessoa sob o `.kinein`; `.kinein/.gitignore` cobre o que não é compartilhável | Catálogo do `kinein_dir` com a coluna classe | Teste do catálogo (R-DADOS-05 do 63) | Não cumprido (ENG-56) e NÃO VERIFICADO; B7 |
| RNF-CON-01 | O contrato tem fonte única | Renomear um método ou campo só no Rust reprova o build C++ ou um teste dos dublês | Mutante no catálogo do ENG-01 | Artefatos gerados conferidos | Não cumprido (ENG-54) e NÃO VERIFICADO; B4 |
| RNF-EDI-01 | Editar não custa o arquivo inteiro | Edição de 1 caractere num arquivo de 20 mil linhas: trabalho do core p95 ≤ 16 ms e volta ≤ 4 KiB mais a faixa mudada | Os spans; o tamanho da resposta | Teste com o arquivo de 20 mil linhas | Não cumprido (64 §2.4) e NÃO VERIFICADO; B8 |
| RNF-MEM-01 | O orçamento de RSS do 21 §M4.2 vale com edição | RSS do core ≤ 60 MB depois de abrir, indexar e editar o arquivo de 20 mil linhas | `/proc/<pid>/status` | Passo do `medir-performance.sh` com veredito | Não cumprido na rodada medida (64 §2.4) e NÃO VERIFICADO; B8 |
| RNF-ESC-01 | Capacidade de periferia não pesa no núcleo | Dependências do processo do core sem os drivers de banco | `cargo tree` | `cargo-deny` com proibição dos crates de driver no `kinein-core` | Não cumprido (64 §2.6) e NÃO VERIFICADO; B9 |

As fases B1 a B9 são as do 64 §8. O item que cria a imposição troca o estado
da linha, com a data e a medida.

**Camadas de suporte (DP-07 do 64 §11):**
- **Núcleo:** as jornadas de C, C++, Rust e Python, embarcados (build,
  gravação, depuração), editor, terminal, Git, busca e Jobs.
- **Periferia:** Banco, Grafana, Remoto e contêineres.

A periferia segue no produto e na 0.4. A diferença está em onde mora: as
dependências pesadas dela saem do processo do core, e a falha dela não
derruba o núcleo (D16).

**Extensibilidade:** a [especificação da arquitetura interna](../especificacoes/arquitetura-interna-core-ipc-jobs.md)
§22.1 já decidiu "sem sistema público de plugins no MVP". Na 0.4, a extensão
é interna, por tabela de descritores, com o teste de extensão da R-TEST-05
do 63.

## Decisões de arquitetura e seus trade-offs

Formato: decisão → ganho → custo aceito → quando revisitar.

### D1. UI Qt/QML e core Rust em processos separados via JSON-RPC (stdio)

- **Ganho:** isolamento de falha (core morre ≠ UI trava), linguagem certa
  para cada camada, testabilidade do core por linhas JSON puras (as sondas
  de M1.1/M1.2 provaram o valor), caminho aberto para CLI/clientes futuros.
- **Custo aceito:** serialização em cada chamada; dois toolchains e dois
  builds; contrato para manter (`DocsPublic/arquitetura/03` + schemas).
- **Revisitar:** transporte vira Unix socket/gRPC apenas se surgir segundo
  cliente simultâneo ou payloads grandes mensuráveis (não antes).

### D2. Orquestrar ferramentas maduras, nunca reimplementar

rustfmt/clang-format, clangd/rust-analyzer, cargo/cmake/ninja, fd, script(1).

- **Ganho:** qualidade de ferramenta de década no dia 1; manutenção mínima;
  comportamento idêntico ao que o usuário já confia no terminal.
- **Custo aceito:** dependência de PATH/versões da máquina (mitigado por
  tools.detect/environment.scan + modo degradado + instalar-ambiente.sh);
  variação de saída entre versões de ferramenta.
- **Revisitar:** nunca por princípio; exceções pontuais só com
  registro aqui.

### D3. Core síncrono de request único + JobManager para o resto

O loop stdio atende um request por vez; build/test/quality/scan são jobs em
thread com eventos.

- **Ganho:** dispatch trivial de raciocinar/testar (sem async runtime, sem
  locks espalhados); jobs dão cancelamento e progresso uniformes.
- **Custo aceito:** um request síncrono lento bloqueia a fila (por isso a
  regra RNF3: longo = job; format.text ficou síncrono por ser curto).
- **Revisitar:** se um request legitimamente curto passar a estourar o
  orçamento (medir antes; a resposta certa pode ser virar job, não async).
- **Revista em 2026-11-14 (decisão do autor):** o gatilho disparou. Na
  medida da revisão da arquitetura (64 §2.2 e §2.3), o laço fica preso pelo
  tempo da ferramenta, e o cancelamento espera junto (ENG-50). O modelo
  passa a ser o do 64 §5.3. A rota declara a classe de execução, e só o
  trabalho de estado roda no laço. Processo, disco do projeto e rede rodam
  fora dele, com prazo, cancelamento e fila por recurso, e voltam ao laço
  só para aplicar o resultado. O JobManager fica com a operação longa e
  visível.
  - **Ganho:** uma ferramenta lenta não para a IDE, e todo pedido tem
    prazo e cancelamento (RNF-EXE-01, RNF-EXE-02). O laço segue sendo o
    único dono do estado, sem locks espalhados.
  - **Custo aceito:** mais estados para pensar (resultado velho, fila por
    recurso, geração por workspace e por documento), a migração dos
    handlers, e testes que rodam o runtime real, com threads.
  - **Revisitar:** se a fila por recurso ou o pool de tamanho fixo virar
    gargalo medido no registro (RNF-OBS-01).

### D4. QML por domínio: composition root + controllers + roteadores IPC

Em 2026-07-09: `Main.qml` com 336 linhas; visual recebe `property`/emite
`signal`; estado em controllers; eventos em `ipc/*`; `CoreClient` fachada
única (DocsPublic/arquitetura/17).

- **Ganho:** fim do god-file de 4.6k linhas; mudança local fica local;
  qmllint estrito vira viável.
- **Custo aceito:** mais arquivos e indireção; fio manual de sinais no
  composition root a cada feature.
- **Revisitar:** não regride; se o fio manual doer, gerar/agrupar — nunca
  voltar estado para componente visual.
- **Revista em 2026-11-12 e em 2026-11-14 (decisão do autor):** o fio
  manual doeu. O `CoreClient` deixa de ser fachada única e vira um
  transporte e um objeto por domínio (D-05 do
  [63](../roadmaps/63-regras-e-plano-de-engenharia-da-0.4.md)), com um
  objeto de resposta por pedido e o contrato gerado (D14).

### D5. Editor sobre TextEdit + QSyntaxHighlighter (não treesitter/engine própria)

- **Ganho:** editor funcional imediato; highlight sintático por regex +
  spans semânticos do LSP cobrem a leitura diária de Rust/C++.
- **Custo aceito:** sem multi-cursor real, minimap, split, parsing
  incremental; regex é aproximação (o LSP corrige por cima).
- **Revisitar:** pós-V1, se edição avançada virar o gargalo do daily use —
  aí avaliar engine de editor dedicada (decisão grande, nova entrada aqui).
- **Revista em 2026-11-14 (decisão do autor):** o "revisitar pós-V1" deixa
  de valer. O motor de edição é fundamento, e pela D18 a escolha fecha na
  0.4. A avaliação do Neovim como motor (ENG-60 do
  [62 §3.4](../roadmaps/62-base-publica-0.4.md); DP-10 do 64 §11) decide,
  com prova de conceito, se o motor muda. Até a decisão, o realce também
  recebe a árvore sintática do core (`syntaxTree.update`, tree-sitter), que
  hoje vai e volta inteira a cada edição (ENG-55); a sincronia incremental
  do 64 §5.11 espera a DP-10, para não ser feita duas vezes. O custo do
  realce do lado da UI não foi medido (64 §12).

### D6. Semantic tokens: full-file com debounce, ancorados por linha

- **Ganho:** zero sincronização incremental (delta encoding, shifts de
  edição); implementação pequena e correta na prática.
- **Custo aceito:** ao inserir/remover linhas, cores podem ficar ~600ms +
  roundtrip deslocadas até o refresh; requisição full a cada pausa de
  digitação (payload pequeno em arquivos típicos).
- **Revisitar:** se arquivos grandes (>5k linhas) mostrarem custo medido,
  adotar range/delta do protocolo LSP.

### D7. Formatação: ferramenta direta, síncrona, sobre o buffer (M1.1)

Design da fatia M1.1. Resumo do trade-off:
independência de LSP vivo e zero side effect em disco, ao custo de manter
seleção de formatter por extensão no core.

### D8. Strict mode máximo com gate fix-first

- **Ganho:** regressão aparece no gate, não no uso; código homogêneo;
  degraus novos (qmllint 117→0) provaram o método fix-first.
- **Custo aceito:** fricção por entrega (minutos de gate completo); recusas
  do clippy pedantic exigem justificar exceções raras.
- **Revisitar:** relaxar exige motivo registrado; a direção
  preferida é subir degraus ([`06-modo-estrito.md`](06-modo-estrito.md), a escada de rigor).

### D9. Testes: comportamento no core; UI sem harness automatizado (hoje)

- **Ganho:** 175+ testes rápidos no caminho crítico (dispatch, fs, jobs,
  LSP parse, format); sondas stdio validam ponta-a-ponta com ferramentas
  reais.
- **Custo aceito:** lógica QML (controllers) e pintura do highlighter só
  têm smoke offscreen + validação empírica — regressão visual pode passar.
- **Revisitar:** degrau 4 da escada de rigor (Qt Quick Test para
  controllers) quando uma regressão de UI escapar OU antes do M2 (o que
  vier primeiro).
- **Anotada em 2026-11-14:** o gatilho já disparou. A lógica QML tem
  harnesses em `scripts/qml-harness`, que rodam no gate, e as camadas finais
  de teste são as do [62 §3.5](../roadmaps/62-base-publica-0.4.md). Os
  números acima são de 2026-07-09. Os dublês dos harnesses passam a ser
  validados contra o contrato (D14).

### D10. Protocolo 0.x sem compatibilidade retroativa formal

- **Ganho:** liberdade de evoluir contrato rápido enquanto o único cliente
  é a própria UI (mesma árvore, mesmo commit).
- **Custo aceito:** nenhum cliente externo pode depender do protocolo antes
  do V1; toda mudança exige DocsPublic/arquitetura/03 + schemas sincronizados no mesmo commit.
- **Revisitar:** congelar/versionar de verdade quando existir consumidor
  fora do repositório (CLI pública, plugins).
- **Confirmado em 2026-11-12 (decisão do autor):** os tipos de dado do
  protocolo seguem como tipos internos dos domínios, como troca consciente
  ([63](../roadmaps/63-regras-e-plano-de-engenharia-da-0.4.md) §B.3, ENG-16).
  O envelope do JSON-RPC fica só na borda do core (R-ARQ-04 de lá).
- **Anotada em 2026-11-14:** "schemas sincronizados no mesmo commit" não
  tinha quem conferisse, e o `schemas/ipc.schema.json` parou no protocolo
  0.62.0 (64 §2.8, ENG-54). O schema passa a ser gerado do código e
  conferido (D14, RNF-CON-01).

### D11. Sem telemetria; diagnóstico por log local

- **Ganho:** privacidade absoluta (RNF1), confiança, zero infra.
- **Custo aceito:** melhoria guiada só por uso próprio e reports manuais.
- **Revisitar:** não revisitar; decisão de produto permanente (leitura-tecnica.md §1).
- **Anotada em 2026-11-14:** o log local era só o de erros da UI. O core
  passa a ter registro local estruturado por pedido, processo e job, sem
  segredo (RNF-OBS-01, 64 §5.10). Nada sai da máquina: a RNF1 segue.

### D12. UI atual → specs por convergência gradual, não remake big-bang

Decisão de 2026-07-09, confirmada com o usuário. Plano vinculante completo
(regras, inventário de divergências, ordem C0–C6 e definition of done):
`DocsPublic/roadmaps/20-convergencia-com-a-especificacao-de-ui.md`.

- **Ganho:** IDE utilizável durante toda a transição (dogfooding contínuo,
  que é o critério dos marcos); risco de regressão proporcional ao tamanho
  da fatia (não há teste visual automatizado — D9); reaproveita a separação
  visual/lógica já paga em DocsPublic/arquitetura/17; UI nova (Main Toolbar, Assistente,
  Start Screen) nasce conforme spec de graça.
- **Custo aceito:** estados intermediários visualmente híbridos (área nova
  conforme convivendo com área antiga) até a C6.
- **Revisitar:** gatilhos objetivos na seção "Gatilho de remake" de
  DocsPublic/roadmaps/20 (acoplamento visual pior que o mapeado na C1; estouro de 2x em
  duas fatias C seguidas; híbrido atrapalhando o uso diário — relato do
  usuário).

### D13. A confiança no projeto é decidida no core, por um lançador único

Decisão do autor de 2026-11-14 (64 §5.4 e §5.5; INF-23, ENG-51). Um lançador
é o único dono da criação de processo. Sem a confiança da pessoa, nada cujo
programa ou comportamento venha do projeto é executado: é o modo restrito do
VS Code e do IntelliJ.

- **Ganho:** abrir um projeto desconhecido não executa código dele
  (RNF-SEG-01); toda execução diz de onde veio (RNF-SEG-02); prazo, teto de
  saída e cancelamento ficam num ponto só.
- **Custo aceito:** sem confiança, o Git, o rust-analyzer e o interpretador
  do projeto ficam desligados, e o gesto de configurar, compilar, executar
  ou depurar pede confiança; todo ponto que cria processo migra para o
  lançador.
- **Revisitar:** se um servidor de linguagem ganhar um modo que não executa
  nada do projeto, ele pode subir sem confiança; ou se o modo restrito
  atrapalhar a leitura de projetos, por relato de uso.

### D14. Contrato com fonte única e resposta por pedido na ponte

Decisão do autor de 2026-11-14 (64 §5.6 e §5.7; ENG-53, ENG-54). A tabela de
métodos mora no `kinein-protocol`, e dela saem o JSON Schema, o cabeçalho
C++ e os dublês dos harnesses. Na ponte, cada pedido devolve um objeto de
resposta próprio, no molde do `QNetworkReply`.

- **Ganho:** nome ou campo divergente entre o Rust, o C++ e os dublês
  reprova no build ou no harness (RNF-CON-01); dois pedidos iguais chegam
  cada um a quem pediu; o pedido tem prazo e cancelamento do lado da UI.
- **Custo aceito:** geração de código no build; a QML segue dinâmica, e o
  erro de forma nela aparece no harness, e não no build; a vida do objeto
  de resposta passa a ser cuidada na ponte.
- **Revisitar:** se a geração não cobrir os tipos do protocolo sem exceção
  manual (a prova do início do B4, 64 §9).

### D15. Persistência em três classes

Decisão do autor de 2026-11-14 (64 §5.9; ENG-56). A configuração do projeto
fica sob o `.kinein`. O estado da pessoa e o cache saem da árvore do projeto
e vão para as pastas de estado e de cache do XDG, numa pasta por projeto.

A pasta por projeto é achada pelo hash do caminho canônico da raiz, como o
`workspaceStorage` do VS Code. Para o estado seguir o projeto renomeado ou
movido (decisão do autor de 2026-11-14, D-28 do 63):
- Cada pasta guarda, no `projeto.json`, o caminho e a identidade do
  diretório raiz no sistema de arquivos: dispositivo, inode e data de
  criação. No Windows, são o número de série do volume, o identificador do
  arquivo e a data de criação.
- Ao abrir um caminho sem pasta de estado, o core procura uma pasta cujo
  caminho gravado sumiu e cuja identidade é a do diretório aberto. É o mesmo
  diretório, renomeado ou movido no mesmo volume, e o estado vai junto, com
  o caminho atualizado.
- A adoção automática exige a identidade completa e o caminho antigo
  ausente. Sem um dos dois, vale só o caminho, e nada é adotado por
  engano.
- Cópia, clone novo e movimento entre volumes começam com estado novo. A
  pasta antiga fica na lista de projetos que sumiram, com "adotar" e
  "remover", e nada é apagado sem o gesto da pessoa.
- Por que não um identificador gravado dentro do projeto, como o
  `ProjectId` do `.idea/workspace.xml` do JetBrains: ele exigiria gravar na
  pasta ao abrir; viria de um arquivo que o projeto controla; e duas cópias
  com o mesmo arquivo dividiriam o estado. É o problema que a ajuda do
  JetBrains descreve para uma árvore de trabalho nova do git: o
  `workspace.xml` versionado duplica o `ProjectId`, e a correção é apagar o
  arquivo para o IDE gerar outro.

- **Ganho:** o estado da pessoa não vai para o repositório; abrir uma pasta
  não grava nela; o caminho local de uma ferramenta deixa de morar num
  arquivo que o projeto controla (RNF-DADOS-05); renomear ou mover a pasta
  no mesmo volume não perde rascunho, sessão nem escolha local.
- **Custo aceito:** a migração dos arquivos que já existem; o estado de uma
  cópia ou de um movimento entre volumes não segue sozinho (a pessoa adota
  pela lista); e o rastreio depende de o sistema de arquivos dar inode e
  data de criação estáveis, senão só o caminho vale.
- **Revisitar:** se a adoção pela lista aparecer como dor de uso relatada,
  ou se um sistema de arquivos suportado não der identidade estável.

### D16. Núcleo e periferia: a periferia fora do processo do core

Decisão do autor de 2026-11-14 (64 §5.13; ENG-57). Executa a
[ADR-0010](../decisoes-adr/ADR-0010-drivers-em-processos-versionados.md)
dentro da 0.4: um processo adaptador por família de driver de banco, com
contrato versionado.

- **Ganho:** as dependências pesadas e a falha de um driver não pesam no
  núcleo nem o derrubam (RNF-ESC-01).
- **Custo aceito:** um contrato versionado e um processo a mais por família
  de driver, e uma ida e volta entre processos em cada consulta.
- **Revisitar:** se o contrato provar caro, o adaptador de uma família pode
  voltar a ser biblioteca (64 §8, B9).

### D17. A regra de estrutura é imposta pelo compilador antes do gate de texto

Decisão do autor de 2026-11-14 (64 §2.7 e §5.14; R-GATE-13 do 63). A ordem
de preferência é: o compilador, o artefato gerado e conferido, o teste de
contrato e, só onde os três não alcançam, o gate de texto, que lê pelo
tree-sitter.

- **Ganho:** a regra não depende de leitor próprio, a fuga não compila, e a
  dívida de cada regra é contada exata.
- **Custo aceito:** crates a mais, com o tempo de build ainda por medir;
  visibilidade mais aberta entre os crates; e a ferramenta do gate precisa
  de build antes dos passos estáticos.
- **Revisitar:** pelo veredito da prova dos crates no início do B6 (64 §9).

### D18. A 0.4 é a última reconstrução sistemática dos fundamentos

Decisão do autor de 2026-11-14 (D-27 do
[63](../roadmaps/63-regras-e-plano-de-engenharia-da-0.4.md)). **A 0.4 deve
ser a última versão em que a Kinein Vectis precisa reconstruir
sistematicamente os seus fundamentos.**

Fundamento é o que todo domínio usa e que não se troca por extensão:
- o modelo de processos e de execução (D1, D3);
- o contrato e a ponte (D14);
- a confiança e a criação de processo (D13);
- a persistência (D15);
- o registro (RNF-OBS-01);
- a estrutura do core e os pontos de extensão;
- a verificação (D17);
- o motor de edição, com a posse do documento e da inteligência de
  linguagem (D5).

Cada fundamento fecha na 0.4, com decisão datada e imposição verificada. Da
0.5 em diante, linguagem, toolchain, SDK, adaptador de depuração, motor de
gravação, painel e método IPC entram por um ponto de extensão (tabela,
descritor ou registro) sem mexer num fundamento.

- **Ganho:** as versões seguintes crescem por extensão, com custo
  proporcional ao recurso novo e não ao tamanho do produto; o colaborador
  novo encontra fundamentos estáveis; a dívida de arquitetura deixa de se
  acumular entre versões.
- **Custo aceito:** a 0.4 fica maior e mais longa, e a etapa C espera; toda
  decisão que antes ficava para depois (o motor de edição é a maior) precisa
  de medida e de prova dentro da 0.4.
- **Revisitar:** só por gatilho medido que nenhum ponto de extensão absorve
  (por exemplo, uma plataforma nova que o modelo de processos não
  comporta), com ADR. Nunca por conveniência de uma entrega.

## Como usar este documento

```text
1. Decisão estrutural nova (ou exceção a uma existente) entra aqui no
   mesmo formato: ganho / custo aceito / gatilho de revisita.
2. Custo aceito começou a doer de verdade (medido, não intuído)? Esse é o
   gatilho: reabrir a decisão numa fatia própria, nunca "de passagem".
3. Requisito novo (funcional ou não) primeiro ganha linha aqui; depois
   vira fatia no roadmap da versão (o [`57`](../roadmaps/57-mapa-de-versoes-ate-a-1.0.md) diz qual).
4. Requisito mensurável: o valor só muda por decisão datada do autor, na
   tabela; o item que cria a imposição troca o estado da linha, com a
   data e a medida. Linha sem imposição diz NÃO VERIFICADO.
```

# 63 — Regras e plano de engenharia da 0.4

> **Classe.** A **Parte A é CONTRATO**: regra, sem número medido, e só muda
> por decisão registrada. As **partes B a H são PLANO**: têm medida datada e
> acompanham a execução. Um documento só, por decisão do autor de 2026-11-12
> (D-01), para a regra e o plano não se contradizerem em dois lugares.
>
> **Para quem executa.** Leia a Parte G, depois a Parte A, e siga a Parte E na
> ordem. O que este documento não traz está num dono que ele aponta pela seção.
>
> **Ponto de partida.** Commit `6baddbdf`, protocolo 0.221.0. Toda medida
> deste documento é de 2026-11-12, nessa árvore, salvo data própria.

## 0. As decisões (decisão do autor, 2026-11-12)

| ID | Decisão |
| --- | --- |
| D-01 | Regras e plano num documento só: este. A Parte A é o regulamento; os outros donos apontam para ela. |
| D-02 | **Nenhuma dívida passa da 0.4.** A 0.4 paga toda a dívida técnica e de arquitetura inventariada. Catraca nova só guarda uma migração: o critério de pronto é zero, dentro da 0.4. |
| D-03 | Nenhum escopo de versão ou de etapa começa sem um plano sólido e estruturado: regras, inventário, ordem e aceite. |
| D-04 | `Core`: tabela de rotas em todos os domínios, handlers fora do `impl Core` e estado por domínio com dono próprio. |
| D-05 | `CoreClient`: um transporte comum e um objeto por domínio, com despacho por tabela. Um piloto de dois domínios mede e ordena; todos os domínios migram. Muda a regra 5 da [ARCHITECTURE §3.1](../arquitetura/ARCHITECTURE.md). |
| D-06 | `EditorController.qml`: a fachada é dissolvida; a exceção de tamanho acaba. |
| D-07 | Toda lista congelada é paga, inclusive os identificadores em português e os adiados do Windows. Só o tipo do fio usado como tipo interno fica como troca consciente, escrita, com gatilho (ENG-16). |
| D-08 | O INF-15 é o primeiro item da execução. |
| D-09 | A UI não repete regra do core: a partida pergunta ao `kinein-core`, num modo de uma resposta. Quem executa mede antes; acima de 50 ms na partida, para e pergunta. |
| D-10 | Gate: os passos estáticos primeiro e em paralelo; índice de palavras nas três checagens lentas; cache do clang-tidy só no `--rapido`; contêiner da CI no repositório; os gates de docs leem o que o git vê. |
| D-11 | Ordem: a engenharia inteira deste documento vem antes de retomar a etapa C. Os adiados do Windows ficam na etapa 10 do [62 §3](62-base-publica-0.4.md). |
| D-12 | INF-06: o prazo entra no teste como parâmetro. |
| D-13 | Na abertura do INF-15 (2026-11-12): a leitura guardada vale também para o `remote-contacts.json`, o `datasources.schema1.json` e as marcas do configure dentro do `.kinein`; o `stateProblem` nomeia também a entrada que não é arquivo regular nem pasta; o INF-17 (escritas que o INF-13 não cobriu) entra logo depois do INF-15 ([62 §3.4](62-base-publica-0.4.md)). |
| D-14 | Depois do INF-15 (2026-11-13): o `CMakeLists.txt` e os outros arquivos do sistema de build são a verdade do projeto; o `.kinein` só complementa, sem atrapalhar nem tomar o lugar deles. O INF-18 entra logo depois do INF-17, pela R-DADOS-04: o arquivo do projeto e o da ferramenta são lidos como o sistema de build os lê, e o guarda só evita que a leitura prenda e diz o motivo, sem o limite de 1 MiB. O limite de cada arquivo do `.kinein` é declarado no catálogo do ENG-02, que passa a ler o LCOV da cobertura (INF-19) ([62 §3.4](62-base-publica-0.4.md)). |
| D-15 | Na abertura do INF-18 (2026-11-13): o INF-20 (as marcas `.kinein-rigor` e `.kinein-preset` e o marcador do espelho fora do guarda, que o INF-13 e o INF-15 não cobriram) entra logo, antes do INF-18. Na abertura do INF-20, no mesmo dia: ele cobre também as remoções sob o `.kinein` (o registro da configuração e a pasta de build do reparo, pelo `kinein_dir::remove`), a query do file-api e as marcas que o configure que deu certo grava e apaga ([62 §3.4](62-base-publica-0.4.md)). |
| D-16 | Na abertura do INF-18 (2026-11-13), depois da medida: o item cobre todo leitor de produção de arquivo do projeto ou de ferramenta que abre sem conferir o tipo, não só os do `CMakeLists.txt`, dos presets, do `CMakeCache.txt` e das marcas, num commit. O motivo do arquivo recusado vai no campo novo `projectProblem` do `workspace.open` e do `cmake.status`, com o caminho a partir do projeto, e a UI o mostra na faixa de saúde. O detector de frameworks, a CDB e a resposta do file-api leem o arquivo inteiro, como o sistema de build e o clangd; o índice, a busca, a substituição e a lista de dependências do compilador são trabalho da IDE e mantêm o teto próprio (§A.9). No fechamento, no mesmo dia: o sinal novo da ponte, `workspaceProjectProblem`, fica redeclarado no cliente falso dos harnesses do roteador e do `stateProblem` e sai na ENG-23 com os outros (R-TEST-06); o harness da faixa prova o controller sem cliente falso, e a medida da ENG-23 segue em 36 ([62 §3.4](62-base-publica-0.4.md)). |
| D-17 | No gate rápido do INF-18 (2026-11-13): o INF-21 (o verificador de idioma lia a string crua de bytes do Rust e a string crua do C++ como código) entra logo, num commit próprio, antes de fechar o INF-18 ([62 §3.4](62-base-publica-0.4.md)). |
| D-18 | Depois do INF-18 (2026-11-13): o aceite do INF-16 passa a ser medido na árvore do 3952b8ad, 213 arestas e 0 ciclos, com o critério do protótipo. O número da revisão (196, no 61934697) envelheceu com os itens da Fase 0, e as arestas falsas da revisão eram 10, não 11 ([62 §3.4](62-base-publica-0.4.md)). |

---

# Parte A — As regras

## A.0 Como ler uma regra

Cada regra tem um ID estável, a frase que se verifica, o porquê com a
evidência, o alcance e a verificação. Quatro pontos valem para todas:

1. **Exceção.** Só por decisão do autor, com data e motivo, registrada na
   tabela da §A.9 e, quando um gate precisa conhecê-la, no arquivo que a regra
   nomeia. Exceção que não está na §A.9 é defeito.
2. **Verificação por gate.** O passo citado reprova o commit. Quando o passo
   ainda não existe, a regra cita o item da Parte E que o cria; até esse item
   fechar, a regra é conferida pelo checklist do commit (§G.3).
3. **Verificação por checklist.** Quem executa confere antes do commit e
   registra na entrada do [40.7](40.7-registro-das-entregas.md); o autor
   confere ao ler o commit, antes do push.
4. **Dono único.** A regra mora aqui. Outro documento aponta para o ID e não
   a repete com outras palavras.

## A.1 Arquitetura

**R-ARQ-01 — O grafo de dependências entre os módulos do `kinein-core` não tem ciclo.**
- Porquê: um ciclo novo já foi descoberto só no gate, no fim de um item
  (INF-13), e o extrator do mapa media errado nos dois sentidos (INF-16).
- Alcance: código de produção de `crates/kinein-core/src`.
- Verificação: `scripts/module_map.py --check` (passo existente; o extrator
  certo e a lista de ciclos conhecidos vazia vêm do INF-16).

**R-ARQ-02 — Módulo da base só depende de módulo da base.**
- Porquê: o guarda compartilhado do `.kinein` só pôde servir a todos porque é
  folha (INF-13); infraestrutura que mora dentro de um domínio obriga os
  outros a depender dele (ENG-12).
- Alcance: os módulos da lista `base` declarada em `scripts/module_map.py`.
- Verificação: `scripts/module_map.py --check` (regra nova, ENG-12).

**R-ARQ-03 — Nenhum módulo fora da borda depende da borda.** A borda é
`handlers`, `runtime`, `outcome` e `rpc`.
- Porquê: a borda conhece todos os domínios; quem depende dela herda todos.
- Alcance: `crates/kinein-core/src`.
- Verificação: `scripts/module_map.py --check` (ENG-12).

**R-ARQ-04 — Só a borda nomeia o envelope de resposta do JSON-RPC.** Domínio
devolve valor e erro próprios; a borda os traduz.
- Porquê: domínio que monta a resposta do protocolo conhece o transporte, e
  ajudante de transporte que conhece o domínio cresce a cada domínio (ENG-12).
- Alcance: `JsonRpcResponse`, `JsonRpcError` e `JsonRpcErrorCode` fora da borda.
- Verificação: conferência de vocabulário no `scripts/module_map.py --check`
  (ENG-12).

**R-ARQ-05 — Um método IPC entra no core por uma entrada na tabela de rotas
do domínio dele. O `lib.rs` não nomeia método.**
- Porquê: a cadeia central cresce um ramo por domínio e chegou ao limite do
  arquivo; todo commit de IPC edita a mesma função (ENG-10).
- Alcance: `crates/kinein-core/src/lib.rs` e as tabelas dos domínios.
- Verificação: teste do registro (toda rota uma vez só; o registro é igual à
  lista do [03](../arquitetura/03-protocolo-ipc.md)) no `cargo test`, e
  vocabulário do `lib.rs` no `module_map.py --check` (ENG-10).

**R-ARQ-06 — A reação a um evento e à troca de workspace é registrada pelo
domínio que reage. A função central não nomeia evento nem estado de domínio.**
- Porquê: a observação de eventos e a troca de workspace são duas cadeias que
  crescem um ramo por domínio (ENG-11).
- Alcance: `lib.rs` e `handlers/workspace.rs`.
- Verificação: teste de extensão (R-TEST-05) e vocabulário no
  `module_map.py --check` (ENG-11).

**R-ARQ-07 — `impl Core` existe só no `lib.rs`. Handler é função livre.**
- Porquê: com o `impl` espalhado, todo handler alcança todo campo, e o
  acoplamento real fica invisível (ENG-14).
- Alcance: `crates/kinein-core/src`, fora de teste.
- Verificação: contagem no `module_map.py --check`, com a catraca de migração
  do ENG-14 até zero.

**R-ARQ-08 — Cada campo do `Core` é um serviço declarado ou o estado de um
domínio. Um domínio tem um campo de estado, e um arquivo de handler só toca o
estado do próprio domínio, os serviços e os estados que o mapa declara para
ele.**
- Porquê: campo de domínio solto no `Core` é estado sem dono (ENG-15).
- Alcance: `struct Core` e `handlers/`.
- Verificação: `module_map.py --check`, contra a lista declarada nele, que o
  mapa gerado publica (ENG-15).

**R-ARQ-09 — A ponte é um transporte e um objeto por domínio. Pedido,
resposta, evento e sinal de um domínio moram no objeto dele; o transporte não
nomeia método.**
- Porquê: uma classe só recebe uma declaração por método e por sinal, e o
  despacho cresce um ramo por nome (UI-02, ENG-20).
- Alcance: `ui/src`.
- Verificação: `scripts/verificar-fiacao-ipc.sh`, com vocabulário do
  transporte e a catraca de migração do ENG-20 até zero.

**R-ARQ-10 — QML fora de `ui/qml/ipc` e do módulo do domínio não acessa a
ponte.**
- Porquê: a regra já está na [ARCHITECTURE §5](../arquitetura/ARCHITECTURE.md)
  e nenhum gate a conferia (ENG-22).
- Alcance: `ui/qml`.
- Verificação: `scripts/verificar-fiacao-ipc.sh` (ENG-22).

**R-ARQ-11 — Um motivo da faixa de saúde é uma unidade numa lista ordenada.
O controller da faixa não nomeia motivo.**
- Porquê: a função de prioridade ganha um ramo por motivo (UI-02).
- Alcance: `ui/qml/workspace`.
- Verificação: teste de extensão no harness da faixa (UI-02).

**R-ARQ-12 — Uma regra do core não é reimplementada na UI nem em ajudante de
gate ou de teste: quem precisa da resposta pergunta ao core ou lê a saída do
produto.**
- Porquê: a cópia fica para trás quando o dono muda (INF-08, INF-14).
- Alcance: `ui/`, `scripts/` e ajudantes de teste.
- Verificação: checklist do desenho e do commit (§G.3, item 4).

**R-ARQ-13 — Arquivo no limite se divide por responsabilidade. Limite não
sobe, categoria não muda e linha de base não sobe.** O commit da divisão diz
qual responsabilidade saiu e qual palavra o arquivo deixou de nomear.
- Porquê: o limite virou alvo, e dividir por tamanho só espalha o problema
  (ENG-25; [ARCHITECTURE §4](../arquitetura/ARCHITECTURE.md), regra 9).
- Alcance: todo arquivo que `scripts/verificar-arquitetura.sh` mede.
- Verificação: `scripts/verificar-arquitetura.sh` para o limite; checklist do
  commit (§G.3, item 6) para a mensagem.

## A.2 Dados da pessoa

**R-DADOS-01 — Toda leitura e toda escrita sob o `.kinein` passa pelo
`kinein_dir`, e todo arquivo próprio está declarado no catálogo dele. Só o
`kinein_dir` nomeia a pasta.**
- Porquê: cada dono fazia por conta própria, e o defeito de um não era
  corrigido nos outros (INF-12, INF-13, INF-15).
- Alcance: código de produção do core.
- Verificação: vocabulário no `module_map.py --check` (ENG-02); até lá, os
  testes do INF-13 e do INF-15.

**R-DADOS-02 — O core não segue link, não abre o que não é arquivo regular e
não bloqueia numa leitura dentro do `.kinein` do projeto.**
- Porquê: um repositório clonado com um FIFO no lugar de um arquivo prende a
  IDE inteira (INF-15); um link grava fora do projeto (INF-13).
- Alcance: todo arquivo do catálogo.
- Verificação: o teste de adversidade do catálogo, por entrada, com mutantes
  (INF-15, ENG-02).

**R-DADOS-03 — A IDE não grava por cima do arquivo que não entende.** Versão
de esquema desconhecida, JSON quebrado ou texto que não é UTF-8: ela avisa e
preserva.
- Porquê: configuração da pessoa já foi sobrescrita em silêncio (INF-07,
  INF-11, INF-12).
- Alcance: todo arquivo do catálogo.
- Verificação: o mesmo teste de adversidade do catálogo (ENG-02).

**R-DADOS-04 — O core lê o arquivo do projeto e o da ferramenta como o
sistema de build os lê, sem bloquear.** Segue o link, que é da pessoa, e não
impõe limite de tamanho próprio; abre sem bloquear e confere no descritor que
é arquivo regular antes de ler. O que não passa não prende a IDE: o projeto
abre, o motivo aparece com o caminho, e nada do `.kinein` toma o lugar do
arquivo recusado.
- Porquê: um FIFO no lugar do `CMakeLists.txt` da raiz ou do
  `CMakeCache.txt` da `.kinein/build/` prende o `workspace.open` (INF-18); o
  `CMakeLists.txt` é a verdade do projeto, e a IDE só complementa (D-14).
- Alcance: código de produção do core que lê arquivo do projeto aberto ou o
  que uma ferramenta gravou para ele, inclusive na pasta de build dentro do
  `.kinein`, que não é do catálogo: o `CMakeLists.txt`, os presets, o
  `CMakeCache.txt` e as marcas do configure numa pasta de build fora do
  `.kinein`.
- Verificação: o teste de adversidade do INF-18, com mutantes.

## A.3 IPC e contrato

**R-IPC-01 — O contrato vem antes do código.** Uma mudança de IPC entra com
a seção do domínio no [03](../arquitetura/03-protocolo-ipc.md), a nota no
[03.1](../arquitetura/03.1-notas-por-versao-do-protocolo.md) e o
`PROTOCOL_VERSION`, no mesmo commit, escritos antes do handler.
- Porquê: método roteado fora da lista e versão sem nota já aconteceram por
  esquecimento ([ARCHITECTURE §5](../arquitetura/ARCHITECTURE.md), passo 12).
- Alcance: todo commit que muda o fio.
- Verificação: `scripts/verificar-fiacao-ipc.sh` (lista do 03 igual às rotas)
  e a conferência da versão contra a nota do topo do 03.1 (ENG-10).

**R-IPC-02 — Todo método tem cliente, todo evento tem tratador e todo sinal
tem ouvinte. Entrada em lista de aceitos leva motivo e data.**
- Porquê: botão que não faz nada passa verde em todos os outros gates.
- Alcance: core, ponte e QML.
- Verificação: `scripts/verificar-fiacao-ipc.sh` (existente); motivo e data
  das listas pelo ENG-36.

## A.4 Testes

**R-TEST-01 — A prova usa a ferramenta real. Ferramenta ausente é NÃO PROVADO,
nunca verde calado; antes de release e na CI, NÃO PROVADO reprova.**
- Porquê: resumo verde com ciclos que não rodaram
  ([07 §1.1](../contribuindo/07-fluxo-e-responsabilidades-dos-gates.md)).
- Alcance: todo teste e todo gate de integração.
- Verificação: o protocolo NÃO PROVADO do orquestrador, com `--estrito`.

**R-TEST-02 — Toda guarda nova tem ao menos um mutante no catálogo
versionado, que compila e é reprovado pelo teste da guarda. Mutante roda numa
cópia, nunca no checkout vivo.**
- Porquê: teste verde porque a guarda nunca foi exercida; e os mutantes eram
  scripts avulsos sobre a árvore de trabalho (ENG-01).
- Alcance: todo item que cria ou muda uma guarda.
- Verificação: `scripts/mutantes.py --conferir` no gate rápido e
  `--rodar <item>` antes do commit do item (ENG-01).

**R-TEST-03 — Teste não espera tempo fixo para sincronizar: espera uma
condição, com prazo, pelo ajudante único. Prazo de produção entra no teste
como parâmetro.**
- Porquê: testes reprovaram sob carga sem defeito no produto (INF-06), e
  espera fixa atrasa o gate e esconde corrida (ENG-30).
- Alcance: testes Rust, harnesses QML e scripts de prova.
- Verificação: conferência no gate rápido (ENG-30).

**R-TEST-04 — Teste e gate não leem nem gravam o diretório pessoal, a
configuração nem o shell de quem os roda.**
- Porquê: testes já dependeram do shell e do `settings.json` da pessoa
  (INF-01, INF-07).
- Alcance: `cargo test`, harnesses e gates.
- Verificação: o orquestrador isola o ambiente de cada rodada (ENG-03).

**R-TEST-05 — Todo ponto de extensão tem um teste de extensão permanente: um
caso só de teste entra sem editar a parte fechada.**
- Porquê: mecanismo fechado só se prova fechado quando um caso novo entra por
  fora ([00](../00-comece-aqui.md), regra 3).
- Alcance: tabela de rotas, reações, catálogo do `.kinein`, objetos de
  domínio da ponte e motivos da faixa.
- Verificação: os testes de extensão no `cargo test` e nos harnesses.

**R-TEST-06 — Ajudante de teste tem um dono: não existe ajudante idêntico em
dois arquivos de teste, e harness não redeclara sinal da ponte.**
- Porquê: a correção feita numa cópia não chega à outra (INF-08, INF-13).
- Alcance: testes Rust e `scripts/qml-harness`.
- Verificação: conferências no gate rápido (ENG-31, ENG-23).

**R-TEST-07 — Refatoração não edita os testes que a provam.** Os testes
existentes passam iguais antes e depois; a única edição aceita é o nome que o
compilador obrigou a trocar, listado na mensagem do commit.
- Porquê: teste ajustado junto com o código prova o código novo, não a
  equivalência.
- Alcance: todo item da Parte E marcado como refatoração.
- Verificação: checklist do commit (§G.3, item 5).

## A.5 Processo

**R-PROC-01 — Escopo de versão ou de etapa só começa com o plano dele
escrito:** as regras que valem, o inventário da dívida da área no
[62 §3.4](62-base-publica-0.4.md), a ordem e o aceite (D-03).
- Porquê: dívida descoberta no meio da etapa fica mais cara e entra sem ordem.
- Alcance: cada versão e cada etapa do 62 §3.
- Verificação: checklist da abertura (§G.1); o cabeçalho do
  [40](40-estado-e-continuidade.md) aponta o plano vigente.

**R-PROC-02 — Um commit fecha um item. Achado novo vira item próprio no 62
§3.4 antes de qualquer correção, e não cresce o commit em curso.**
- Porquê: item que nasce da prova de outro já inchou um commit.
- Alcance: todo commit.
- Verificação: checklist do commit (§G.3, item 1).

**R-PROC-03 — Renomear é pelo compilador ou pela ferramenta do repositório,
nunca por troca de texto no crate inteiro.**
- Porquê: uma troca às cegas já alterou o que não devia.
- Alcance: todo rename.
- Verificação: checklist do commit (§G.3, item 7).

**R-PROC-04 — Decisão reversível é do autor: fica registrada no dono, com
data, antes do código.**
- Porquê: decisão tomada no meio do código não tem onde ser revista.
- Alcance: todo item.
- Verificação: checklist do commit (§G.3, item 2); os casos previstos estão
  em §G.4.

**R-PROC-05 — O gate rápido passa verde, sozinho, antes de cada commit.**
Sem build pesado nem mutantes ao mesmo tempo.
- Porquê: carga paralela já deu falso vermelho, e commit sem gate já quebrou
  o item seguinte.
- Alcance: todo commit de código.
- Verificação: o resumo do gate na entrada do 40.7.

## A.6 Dependências

**R-DEP-01 — Licença, advisory, ban e origem de toda dependência Rust passam
pelo `cargo-deny` em todo gate. Exceção no `deny.toml` leva data e motivo no
próprio arquivo.**
- Porquê: a política de licença já morou num arquivo que ninguém rodava.
- Alcance: `Cargo.lock` inteiro.
- Verificação: `scripts/verificar-deny.sh` (existente).

**R-DEP-02 — Advisory novo é item próprio, com commit próprio, antes de
continuar o item em curso.**
- Porquê: o gate reprova sem mudança local, e a correção não pertence ao item
  que estava aberto (INF-03, INF-10).
- Alcance: todo advisory acusado.
- Verificação: checklist do commit (§G.3, item 1).

Ferramenta ou dependência nova continua pelo registro de componente das
[integrações](../integracoes/README.md), que é o dono dessa regra.

## A.7 Docs

**R-DOC-01 — Número sem data é afirmação sobre agora e tem de bater com o
disco.** Número de outro dia leva a data.
- Porquê: documento já afirmou tamanho de arquivo várias vezes menor que o
  real ([ARCHITECTURE §1.1](../arquitetura/ARCHITECTURE.md)).
- Alcance: todo `.md` que não é registro datado.
- Verificação: `scripts/verificar-docs.sh` (existente).

**R-DOC-02 — Documento público só cita documento público, sem conta pessoal
nem nota de sessão.**
- Porquê: texto que manda o leitor a um arquivo que ele não tem confunde
  quem chega para colaborar.
- Alcance: todo arquivo que o git publica.
- Verificação: `scripts/verificar-links-docs.sh` (existente).

**R-DOC-03 — Um fato mora num dono, e os outros apontam pela seção. Regra de
engenharia mora nesta Parte A.**
- Porquê: a mesma regra aparecia em vários documentos com palavras
  diferentes, e cada cópia envelhecia sozinha.
- Alcance: `DocsPublic/` e os comentários de cabeçalho dos gates.
- Verificação: checklist do commit (§G.3, item 8).

**R-DOC-04 — Os gates de docs leem exatamente os arquivos que o git vê:**
rastreados e novos não ignorados.
- Porquê: o gate local lia arquivos que a CI não tem, e o veredito dependia
  da máquina (ENG-06).
- Alcance: `verificar-docs.sh`, `verificar-links-docs.sh` e a conferência de
  privacidade.
- Verificação: os três gates de docs (ENG-06).

## A.8 Gate e CI

**R-GATE-01 — Os passos estáticos rodam antes de qualquer build, em
paralelo.**
- Porquê: checagem barata reprovava no fim, depois de minutos (ENG-03).
- Alcance: todo perfil do `scripts/verificar.sh`.
- Verificação: `scripts/test_verification_profiles.py` (ENG-03).

**R-GATE-02 — Gate folha não grava no checkout e não chama outro gate
folha.**
- Porquê: passo que grava na árvore muda o que o passo ao lado lê
  ([07 §3](../contribuindo/07-fluxo-e-responsabilidades-dos-gates.md)).
- Alcance: todo script que o orquestrador chama.
- Verificação: o orquestrador compara o estado do git antes e depois (ENG-03).

**R-GATE-03 — Toda regra de gate tem um defeito plantado no catálogo, que ela
reprova.**
- Porquê: gate que nunca reprovou não se sabe se funciona
  ([ARCHITECTURE §4](../arquitetura/ARCHITECTURE.md), regra 11).
- Alcance: toda regra de gate, nova ou mudada.
- Verificação: `scripts/mutantes.py --rodar gate` (ENG-01).

**R-GATE-04 — Mudança de tempo no gate não muda veredito:** os mesmos
achados, antes e depois, sobre os defeitos plantados.
- Porquê: ordem, cache e paralelismo podem esconder uma falha sem que nada
  reclame.
- Alcance: toda mudança de ordem, cache, paralelismo ou algoritmo num gate.
- Verificação: `scripts/mutantes.py --rodar gate` nos dois lados, registrado
  no commit da mudança.

**R-GATE-05 — Cache não decide no `--completo` nem na CI.**
- Porquê: o cache pode errar; a prova de release e a da CI têm de refazer
  tudo (ENG-05).
- Alcance: todo cache de resultado de gate.
- Verificação: `scripts/test_verification_profiles.py` (ENG-05).

**R-GATE-06 — A CI roda um perfil do mesmo orquestrador, no ambiente
declarado no repositório. Nenhuma checagem existe só no YAML.**
- Porquê: "CI vermelha" tem de se reproduzir na máquina de quem desenvolve
  (ENG-07).
- Alcance: `.github/workflows/` e `packaging/ci/`.
- Verificação: `scripts/test_verification_profiles.py` lê o `ci.yml`
  (ENG-07); `scripts/ci-local.sh` roda o mesmo ambiente.

**R-GATE-07 — Catraca de migração tem zero como critério de pronto e é
apagada quando chega a zero.**
- Porquê: lista congelada sem prazo vira dívida permanente (D-02).
- Alcance: toda linha de base criada para guardar uma migração.
- Verificação: a tabela da §B.2 e o checklist do fechamento da etapa (§E.5).

**R-GATE-08 — Rodada vermelha da CI tem a causa lida e registrada no 40
antes do commit seguinte.**
- Porquê: a CI ficou vermelha por semanas sem que a causa fosse lida (ENG-07).
- Alcance: toda rodada da CI.
- Verificação: checklist do commit (§G.3, item 9).

## A.9 Exceções registradas

| Regra | Exceção | Decisão |
| --- | --- | --- |
| R-DEP-01 | Licença AGPL-3.0-only dos crates próprios e Zlib do `foldhash`, no `deny.toml` | Autor; a do `foldhash`, em 2026-11-09 |
| R-ARQ-04 | Os tipos de dado do protocolo seguem como tipos internos dos domínios (ENG-16) | Autor, 2026-11-12 (D-07) |
| R-DADOS-04 | O teto próprio do trabalho da IDE sobre os arquivos do projeto: o índice não analisa o arquivo acima de 4 MiB e o lista com o motivo; a busca e a substituição pulam o arquivo acima de 1 MiB, o teto do `fs.read`, e a substituição para nos 16 MiB somados; a lista de dependências que o compilador grava na verificação de uma edição é recusada acima de 1 MiB, e a captura dos cabeçalhos dessa verificação, acima de 64 MiB | Autor, 2026-11-13 (D-16) |

---

# Parte B — O inventário da dívida (medido em 2026-11-12)

O registro de cada item está no [62 §3.4](62-base-publica-0.4.md), no
formato dele. Aqui ficam a evidência medida e o destino. Destino de todos,
salvo o ENG-16: **paga na 0.4** (D-02).

## B.1 Os itens (medido em 2026-11-12)

| ID | Dívida | Evidência | Sev. |
| --- | --- | --- | --- |
| INF-15 | Leitura do `.kinein` sem conferir o tipo | Um FIFO no lugar de 5 dos 10 arquivos lidos prende o `workspace.open` (62 §3.4) | alta |
| INF-17 | Escritas do `.kinein` que o INF-13 não cobriu | Medido no 61934697: o `datasource.save` grava o `datasources.json` na pasta de fora do `.kinein` ligado (62 §3.4) | alta |
| INF-20 | Marcas do configure, remoções e marcador do espelho fora do guarda | Medido no deb3ed5b: um FIFO no `.kinein-rigor` prende o `build.run`; com `.kinein/presets` ligado, o reparo da pasta de build apaga a pasta de fora inteira (62 §3.4) | alta |
| INF-21 | Verificador de idioma lê a string crua de bytes do Rust e a do C++ como código | Medido no cfae82b9: o texto do `br#"…"#` vira identificador, e depois de `br#"a"b"#` ou de `R"(a"b)"` o resto da linha não é lido (62 §3.4) | baixa |
| INF-18 | Arquivos do projeto e das ferramentas lidos sem conferir o tipo | Medido no 61934697: um FIFO no lugar do `CMakeLists.txt` da raiz ou do `CMakeCache.txt` da `.kinein/build/` prende o `workspace.open` (62 §3.4) | alta |
| INF-19 | LCOV da cobertura lido fora do `kinein_dir` | O `.kinein/coverage.lcov` é lido de volta com `fs::read_to_string`, sem limite; o de 1 MiB não serve a ele (62 §3.4) | baixa |
| ENG-01 | Mutantes avulsos, sobre o checkout vivo | Sem catálogo; numa cópia, o build frio do teste do core leva 69,1 s e cada mutante 11,3 s | média |
| INF-16 | O mapa de módulos mede errado nos dois sentidos | Mede 184 arestas e 2 ciclos; o código de produção tem 196 arestas e 0 ciclos. Os dois ciclos conhecidos só existem em comentário; 10 arestas vêm de comentário ou teste; 22 arestas reais em `use crate::{...}` não são vistas. No 3952b8ad (2026-11-13): 200 arestas e os 2 ciclos, contra 213 e 0, com 9 falsas e as 22 perdidas (D-18) | média |
| ENG-02 | Cada dono monta o caminho do `.kinein` | 33 arquivos de produção do core têm o literal; o `kinein_dir` não tem leitura nem catálogo | alta |
| ENG-03 | Checagem barata no fim do gate; folha que grava no checkout | 19 passos estáticos somam 61,7 s e rodam por último; o autoteste dos identificadores cria uma pasta temporária dentro do checkout | média |
| ENG-04 | Três checagens com um regex por nome sobre o texto todo | Fiação IPC 14,1 s, funções mortas 11,0 s, alcance 10,6 s; protótipo com índice: 11,07 s → 0,37 s, mesmo resultado | baixa |
| ENG-05 | clang-tidy sem cache | 52 s por rodada, com ou sem mudança em C++ | baixa |
| ENG-06 | Gate de docs lê arquivo que o git não vê | `verificar-docs.sh` varre o disco; os outros dois gates de docs já leem pelo git | média |
| ENG-07 | A CI nunca executou | 32 rodadas, de 2026-10-19 a 2026-11-12, todas com 0 passos em 2 s: o job não chega a iniciar no runner, por um bloqueio da conta no GitHub, fora do repositório. O `--ci-rapida --estrito` nunca foi provado no Ubuntu 24.04 | alta |
| ENG-10 | Roteamento do core por cadeia | `lib.rs` em 500/500: um `match` e 40 `or_else`; 35 dos últimos 300 commits o editaram | alta |
| ENG-11 | Reações centrais | `observe_notification` reage a 10 eventos por nome; `handlers/workspace.rs` toca 16 dos 32 campos do `Core` | média |
| ENG-12 | Infraestrutura dentro de domínio; borda que conhece domínio | O `EventSender` e o enquadramento `Content-Length` moram no `lsp` e são usados por 9 módulos; o `fswatch` tem uma cópia do tipo do remetente; o `rpc.rs` tem 12 funções de erro de 6 domínios; 6 arquivos de domínio (banco, serial e terminal) nomeiam o envelope de erro | média |
| ENG-13 | `workspace` com duas responsabilidades | Criação de projeto (4 arquivos, traz `configaction`, `process` e `tools`) e estado do workspace (6 arquivos, só a base) | média |
| ENG-14 | `impl Core` espalhado | 73 arquivos de produção; 30 dos 72 arquivos de handler tocam 0 ou 1 campo | alta |
| ENG-15 | Estado sem dono no `Core` | 32 campos, escritos duas vezes (a struct e o construtor); 6 são do banco, 5 de embarcados, 3 de linguagem | alta |
| ENG-16 | Tipo do fio como tipo interno | 191 arquivos de domínio usam tipo do `kinein-protocol` | — |
| UI-02 | Faixa de saúde e despacho crescem um ramo por motivo | `update()` com 23 `apply()` em 376/400; `handleWorkspaceOpened` com 6 `emit` e 7 pedidos fixos | média |
| ENG-20 | `CoreClient` como classe única | 201 `Q_INVOKABLE`, cerca de 220 sinais, 287 ramos por nome de método em 22 arquivos; o cabeçalho foi editado em 40 dos últimos 300 commits e em 44% dos commits de IPC | alta |
| ENG-21 | Raízes de composição no limite | `Main.qml` 397/400, `AppDomains.qml` 377/400; o módulo `<X>Domain` do [27](../arquitetura/27-modulos-por-dominio.md) §4 segue aberto | média |
| ENG-22 | QML de área fala com a ponte | 8 arquivos fora de `ui/qml/ipc`, 99 acessos | média |
| ENG-23 | Cliente falso repetido | 36 sinais da ponte redeclarados em 3 ou mais harnesses (`requestFailed` em 17); o `workspaceProjectProblem` do INF-18 está em 2 (D-16) | média |
| ENG-24 | Fachada do `EditorController.qml` | 791/400; 127 membros usados em 250 pontos de 22 arquivos (eram 123 em 15 em 2026-09-03) | média |
| ENG-25 | Limite usado como alvo | 65 arquivos a 90% ou mais do limite, 21 a 99% ou mais; são tocados 7,2 vezes cada em 300 commits, contra 2,6 dos outros | média |
| INF-06 | Prazo de relógio nos testes da verificação de edição | 5 testes reprovaram sob carga (62 §3.4) | média |
| ENG-30 | Espera fixa em teste | Cerca de 45 `sleep` fora de laço nos testes Rust (`syntax_check.rs` 600 e 900 ms, `datasource/resolve.rs` 1500 e 1800 ms); 46 `time.sleep` em 19 scripts | média |
| ENG-31 | Ajudante de teste duplicado | 19 ajudantes com corpo idêntico em 2 ou mais arquivos, 23 cópias | baixa |
| ENG-32 | A UI repete a regra do `settings.json` na partida | `ui/src/startup_web.cpp`, com duas diferenças documentadas (INF-14) | média |
| ENG-33 | Funções QML sem consumidor | 12 na linha de base | baixa |
| ENG-34 | Derivações repetidas na QML | 11 na linha de base | baixa |
| ENG-35 | Literais fora do `Theme` | 7 entradas na linha de base | baixa |
| ENG-36 | Supressões e listas de aceitos | 25 `#[allow(`, 7 `#![allow(`, 13 `NOLINT`, 24 `shellcheck disable`; 8 entradas nas listas de aceitos da fiação | baixa |
| ENG-37 | Identificadores em português | 3321 pares arquivo e palavra, 14.074 ocorrências (eram 4044 pares em 2026-10-01) | média |
| ENG-38 | Checklist de riscos sem conferência | Os 10 riscos do [16](../arquitetura/16-checklist-de-riscos-ocultos.md) não têm, cada um, a verificação apontada | média |
| ENG-39 | Testes adiados no Windows | 76 testes e 50 `cfg(unix)` congelados | média |

Os itens C-01 a C-03, CPP-01, UI-01 e os C1 abertos seguem no 62 §3.4 e na
etapa deles; este plano não os altera.

## B.2 Linhas de base e listas, uma a uma (medido em 2026-11-12)

| Lista | Hoje | Destino | Item |
| --- | --- | --- | --- |
| `scripts/arquitetura-baseline.txt` | 1 arquivo | Zero; o arquivo fica vazio | ENG-24 |
| `scripts/module_map.py` (ciclos conhecidos) | 2 | Zero; a lista é apagada | INF-16 |
| `scripts/qml-mortas-baseline.txt` | 12 | Zero; a linha de base é apagada | ENG-33 |
| `scripts/qml-duplicacao-baseline.txt` | 11 | Zero; a linha de base é apagada | ENG-34 |
| `scripts/qml-tokens-baseline.txt` | 7 | Zero; a linha de base é apagada | ENG-35 |
| `scripts/identifier-language-baseline.txt` | 3321 | Zero; a linha de base é apagada | ENG-37 |
| `scripts/adiados-windows-baseline.txt` | 76 e 50 | Zero, na etapa 10 | ENG-39 |
| Listas de aceitos em `scripts/verificar_fiacao_ipc.py` | 8 | Cada entrada ligada, apagada ou mantida com motivo e data | ENG-36 |
| Supressões no código | 69 | Cada uma removida ou com o motivo na linha | ENG-36 |
| Catracas de migração novas (ENG-14, ENG-15, ENG-20, ENG-22, ENG-23, ENG-30) | criadas pelo item | Zero, e apagadas (R-GATE-07) | o próprio |

## B.3 O que fica como está, com decisão

- **ENG-16 — tipo do fio como tipo interno: troca consciente** (D-07).
  Separar os dois sem versão a manter é mecanismo sem usuário
  ([00](../00-comece-aqui.md), regra 3; [19](../arquitetura/19-compromissos-de-arquitetura.md),
  D10). O envelope não entra na troca: R-ARQ-04 vale. Gatilho para rever: um
  cliente fora do repositório, sessão remota entre versões diferentes, ou a 1.0.
- **Exceções do `deny.toml`:** mantidas; têm decisão e motivo no arquivo.
- **`scripts/identifier-language-allowlist.txt`:** nomes de API e de
  ferramenta, com motivo. Não é dívida.
- **Arquivo de teste fora do limite de linhas:** a
  [ARCHITECTURE §4](../arquitetura/ARCHITECTURE.md) já exclui teste da
  catraca, de propósito. Sem regra nova de tamanho para teste.

---

# Parte C — A arquitetura-alvo

Os esboços de código desta parte mostram a forma, e não são contrato: o
primeiro domínio de cada item confirma a forma. Se ela não servir, quem
executa para e pergunta (§G.4).

## C.1 O core: três camadas

```text
borda     handlers  runtime  outcome  rpc        conhece os domínios; ninguém a conhece
domínio   todos os outros módulos                dependem da base e de outros domínios, sem ciclo
base      platform  stderr_tail  msvc  fsops  process  owned_child  kinein_dir  events  framing
```

Invariantes:

1. O grafo não tem ciclo (R-ARQ-01).
2. A base só conhece a base (R-ARQ-02). `events` (o remetente de eventos) e
   `framing` (o enquadramento `Content-Length`) saem do `lsp` para a base:
   `jobs`, `terminal`, `dap` e `platformio` deixam de conhecer o `lsp` por
   causa deles, e o `fswatch` deixa de ter a própria cópia do tipo.
3. Ninguém fora da borda conhece a borda (R-ARQ-03). O `rpc` fica só com o
   que é do envelope; o erro de cada domínio vai para o handler dele.
4. Só a borda nomeia o envelope de resposta (R-ARQ-04).
5. Só o `kinein_dir` nomeia o `.kinein` (R-DADOS-01).
6. O `workspace` é só o estado do workspace aberto; a criação de projeto é
   outro módulo, que o `workspace` não conhece.

A lista das camadas mora em `scripts/module_map.py`, que a confere e a
publica no [01](../arquitetura/01-mapa-de-modulos.md).

## C.2 O core: os pontos de extensão

Um domínio se descreve no próprio módulo, e um registro único lista os
domínios:

```rust
pub(crate) struct Route {
    pub method: &'static str,
    pub handle: fn(&mut Core, Option<Value>, Option<&Value>) -> RequestOutcome,
}
pub(crate) struct Domain {
    pub name: &'static str,
    pub routes: &'static [Route],          // método -> handler
    pub reactions: &'static [Reaction],    // evento observado -> reação
    pub workspace_changed: Option<fn(&mut Core)>,
}
pub(crate) const DOMAINS: &[&Domain] = &[&git::DOMAIN, /* um por linha */];
```

- O `lib.rs` valida o pedido, procura o método no registro e chama. Não
  nomeia método nem evento (R-ARQ-05, R-ARQ-06).
- Um método novo é uma linha na tabela do domínio. Um domínio novo é um
  módulo e uma linha no registro.
- O `Core` guarda os serviços declarados (os que muitos domínios usam, como
  `jobs`, `detector`, `workspace` e `events`) e um estado por domínio, cujo
  tipo mora no domínio (R-ARQ-08). O construtor não repete a lista de campos.
- Handler é função livre no arquivo do domínio (R-ARQ-07).

## C.3 Os dados da pessoa: o catálogo do `.kinein`

O `kinein_dir` declara cada arquivo próprio (nome, versão de esquema, dono e
limite de tamanho, D-14) e é o único caminho de leitura e de escrita. Um arquivo novo é uma entrada no
catálogo, e herda o teste de adversidade: FIFO, link, pasta no lugar do
arquivo, tamanho acima do limite, esquema desconhecido, JSON quebrado e texto
que não é UTF-8 (R-DADOS-01 a R-DADOS-03).

## C.4 A ponte e a QML

```text
CoreClient (transporte)   processo, enquadramento, ids, pedidos pendentes, log
<Dominio>Client           pedidos, sinais, tabela de respostas e de eventos do domínio
<Dominio>Domain.qml       instancia o cliente, os roteadores e o controller do domínio
```

Invariantes:

1. O transporte não nomeia método, evento nem domínio (R-ARQ-09). O objeto do
   domínio se registra nele.
2. Um resultado novo é uma entrada na tabela do domínio; um campo novo do
   `workspace.open` chega ao consumidor sem um `emit` por campo.
3. O que cada domínio pede depois de abrir o workspace é registrado pelo
   domínio, e não por uma função do transporte.
4. Só os roteadores de `ui/qml/ipc` e o `<Dominio>Domain.qml` acessam a ponte
   (R-ARQ-10). O resto da QML fala com o controller.
5. O `EditorController.qml` deixa de ser fachada: cada consumidor fala com o
   dono, conforme o [32](../arquitetura/32-editor-por-responsabilidade.md) §9.
6. A faixa de saúde escolhe o primeiro motivo levantado de uma lista
   ordenada; cada motivo é um arquivo, com as próprias entradas (R-ARQ-11).

## C.5 O que mudou nos donos

A [ARCHITECTURE.md](../arquitetura/ARCHITECTURE.md) (§3.1 regra 5, §4 e §5),
o [35](../arquitetura/35-crescer-sem-god-object.md) (§5.2 e §5.3) e o
[27](../arquitetura/27-modulos-por-dominio.md) (cabeçalho) registram, cada um,
a decisão de 2026-11-12 e apontam para cá. O guia da ARCHITECTURE §5 é
reescrito pelo item que entrega cada ponto de extensão (ENG-10, ENG-20).

---

# Parte D — O gate

## D.1 Regras novas ou mudadas (medido em 2026-11-12)

| Regra | O que pega | Falso positivo medido | Custo | Como reprova |
| --- | --- | --- | --- | --- |
| Extrator do mapa (INF-16) | Ciclo e aresta reais, também em `use crate::{...}` | No 3952b8ad: 2 ciclos e 9 arestas falsos, 22 arestas perdidas. Protótipo: 0 falsos, 213 arestas (196 no 61934697) | 0,4 s, como hoje | Qualquer ciclo; a lista de conhecidos deixa de existir |
| Camadas e vocabulário (ENG-12, ENG-02, ENG-10) | Base que conhece domínio; envelope fora da borda; `.kinein` fora do `kinein_dir`; método no `lib.rs` | Nenhum: cada ocorrência de hoje é a dívida do item (33, 6 e as rotas) | Menos de 1 s, no mesmo passo do mapa | Reprova direto depois do item; sem catraca |
| `impl Core`, campos do `Core`, ponte, QML fora do `ipc` (ENG-14, ENG-15, ENG-20, ENG-22) | Volta do god object | Nenhum por construção: contagem exata | Menos de 1 s | Catraca de migração que só desce; em zero, vira regra direta e a catraca é apagada |
| Estado do git antes e depois (ENG-03) | Folha que grava no checkout | Nenhum: o checkout ficou igual depois das 2 rodadas medidas do bloco | 11 ms por conferência | Reprova e mostra o arquivo |
| Espera fixa em teste (ENG-30) | `sleep` fora do ajudante | A conferência ignora texto de script embutido (6 das 45 ocorrências de hoje) | Menos de 1 s | Catraca de migração até zero |
| Ajudante idêntico (ENG-31) | Cópia de ajudante de teste | Nenhum: só corpo idêntico, de 80 caracteres ou mais | Menos de 1 s | Reprova direto depois do item |
| Catálogo de mutantes (ENG-01) | Troca que não se aplica mais | Nenhum: conferência exata de texto | Milissegundos | Reprova e mostra a entrada |
| Versão contra a nota (ENG-10) | `PROTOCOL_VERSION` sem a nota do 03.1 | A medir no item | Milissegundos | Reprova direto |

Nenhuma dessas regras mede linha de arquivo. O limite de linhas continua
sendo só o alarme que manda olhar (R-ARQ-13).

## D.2 O tempo (medido em 2026-11-12)

Gate rápido: 222 s. Por passo, em série: `cargo test` 54 s, clang-tidy 52 s,
exercitação 30 s, qmllint 12 s, harnesses 8 s e os 19 passos estáticos 61,7 s.

| Passo estático | Sozinho |
| --- | --- |
| fiação IPC | 14,1 s |
| identificadores (9,5 s são o autoteste) | 11,4 s |
| funções QML mortas | 11,0 s |
| alcance QML | 10,6 s |
| shellcheck | 4,2 s |
| docs, links | 2,5 s e 2,1 s |
| perfis, fmt, deny | 1,4 s, 1,2 s e 1,5 s |
| os outros 9 | menos de 0,5 s cada |

| Mudança | Antes | Depois | Estado |
| --- | --- | --- | --- |
| Bloco estático em paralelo, no início (ENG-03) | 61,7 s, no fim | 16,0 s, no início | medido |
| Índice de palavras em três checagens (ENG-04) | 11,07 s (funções mortas) | 0,37 s, mesmo resultado | medido em uma; as outras duas têm o mesmo perfil |
| Autoteste dos identificadores sobre uma árvore de fixture (ENG-03) | 9,5 s | a medir | — |
| Cache do clang-tidy (ENG-05) | 52 s | cerca de 2 s sem mudança em C++ (o pré-processamento das 52 unidades leva 1,2 s) | estimado |

**Meta**, a conferir por quem executa: gate rápido em até 125 s sem mudança
em C++ e até 175 s com; a primeira falha estática em até 10 s. O `cargo test`
não entra na meta: os 38 testes de 5 s ou mais somam 287 s de 567 s e são
provas com ferramenta real (R-TEST-01).

**O que não muda:** a ordem continua sendo só do orquestrador; o commit
sempre passa pelo `--rapido` inteiro; não há perfil que pule passo pela
mudança. O bloco estático não roda junto do `cargo test`.

## D.3 A prova de que nada perdeu força

O catálogo `gate` do ENG-01 tem ao menos um defeito plantado por passo: a
função morta, o componente que ninguém abre, o método sem cliente, o ciclo
por import agrupado, o arquivo acima do limite, o número sem data que mente,
o link morto, a menção a nota local, o identificador em português, o erro de
shell, o achado do clang-tidy e o `NOLINT` removido. Cada mudança de D.2 roda
o catálogo antes e depois, na cópia: os mesmos passos reprovam, com a mesma
mensagem (R-GATE-04). A composição (todo passo em todo perfil que o inclui, e
a falha de um passo paralelo não se perde) é provada pelo
`scripts/test_verification_profiles.py`.

## D.4 O gate local e a CI

A CI roda `bash scripts/verificar.sh --ci-rapida --estrito` dentro do
contêiner de `packaging/ci/`, e `scripts/ci-local.sh` roda o mesmo contêiner
com o `podman` (ENG-07). O perfil é um subconjunto do gate local, escolhido no
orquestrador ([07 §2.1](../contribuindo/07-fluxo-e-responsabilidades-dos-gates.md)).
O desbloqueio da conta no GitHub não é do repositório: é do autor.

---

# Parte E — A execução

## E.1 A ordem

```text
Fase 0  dados e gate     INF-15 → INF-17 → INF-20 → INF-21 → INF-18 → ENG-01 → INF-16 → ENG-02 → ENG-03 → ENG-04 → ENG-05 → ENG-06 → ENG-07
Fase 1  core             ENG-10 → ENG-11 → ENG-12 → ENG-13 → ENG-14 → ENG-15
Fase 2  ponte e QML      UI-02 → ENG-20 → ENG-21 → ENG-22 → ENG-23 → ENG-24 → ENG-25
Fase 3  testes e listas  INF-06 → ENG-30 → ENG-31 → ENG-32 → ENG-33 → ENG-34 → ENG-35 → ENG-36 → ENG-37 → ENG-38
depois                   a etapa C retoma (C1-47, C1-48, C1-46, C1-07) pelo 62 §3; ENG-39 na etapa 10
```

Um item só começa com o anterior fechado e o gate rápido verde. Cada item
abaixo diz o que fazer, o que cabe em cada commit, o aceite, os mutantes e o
pronto. **Aceite comum** a todos: os testes existentes passam iguais antes e
depois (R-TEST-07), o gate rápido passa, e a entrada do 40.7 traz as medidas.

## E.2 Fase 0 — dados e gate

**INF-15 — a leitura guardada do `.kinein`.**
- Fazer: `kinein_dir::read`, que abre sem bloquear e sem seguir link, confere
  no descritor que é arquivo regular e respeita um limite de tamanho. As dez
  leituras do item passam por ela; as conferências próprias de cada dono saem.
  No Windows não há FIFO: vale a pasta e o link, provados na etapa 10.
- Commits: um.
- Aceite: a reprodução do 62 §3.4 com o FIFO, o link, a pasta e o arquivo
  acima do limite no lugar de cada um dos dez; o `workspace.open` e o
  `settings.get` respondem no prazo e o problema aparece como o do INF-13. O
  teste abre o outro lado do FIFO se a leitura prender, para nunca travar.
- Mutantes: abrir bloqueando; seguir o link; não conferir o tipo; não aplicar
  o limite; um dono lendo direto.
- Pronto: nenhuma leitura de arquivo do `.kinein` fora do `kinein_dir`.

**INF-17 — as escritas que o INF-13 não cobriu** (D-13).
- Fazer: o `datasources.json`, a cópia `datasources.schema1.json` e o
  `remote-contacts.json` gravam pelo `kinein_dir::write`; a conferência
  própria do `datasource/store.rs` sai.
- Commits: um.
- Aceite: o `.kinein` e cada arquivo ligados, pelo IPC e no dono, com a
  pasta de fora vazia no fim; o erro do método diz o motivo do guarda.
- Mutantes: cada dono gravando direto.
- Pronto: o `datasource/store.rs` e o `remote/contacts.rs` não gravam no
  `.kinein` fora do `kinein_dir`.

**INF-20 — as marcas, as remoções e o marcador que o INF-13 e o INF-15 não
cobriram** (D-15).
- Fazer: medir antes o FIFO no lugar do `.kinein-rigor` no build, o link no
  lugar dele e do `.kinein-preset` depois de um configure, e as remoções e a
  query pelo link. O guarda ganha o `kinein_dir::remove`, que confere o
  caminho como a escrita. Na pasta de build dentro do `.kinein`, as marcas do
  rigor, do preset, da tentativa e do kit e a query do file-api são lidas,
  gravadas e apagadas pelo guarda; o `begin_configure` confere a do preset e
  a do rigor com a trava e as outras marcas, antes do `CMake`. O registro da
  configuração é apagado pelo guarda; o reparo da pasta de build confere o
  caminho na prévia e ao aplicar, e apaga pelo guarda. O `remote-mirror.json`
  é lido e gravado pelo guarda. Numa pasta de build fora do `.kinein`, as
  marcas seguem para o INF-18.
- Commits: um.
- Aceite: o FIFO no lugar do `.kinein-rigor` não prende o build, que recusa
  com o motivo; o FIFO e o link no lugar de cada marca recusam o configure
  antes do `CMake`, sem criar o alvo; com `.kinein/presets` ligado, o reparo
  recusa na prévia e ao aplicar, e o build recusa sem criar a query, com a
  pasta de fora intacta; com o `.kinein` ligado, a volta à escolha
  automática não apaga o registro de fora; o marcador do espelho ligado e
  como FIFO não é usado nem gravado fora. O teste abre o outro lado do FIFO
  se a leitura prender.
- Mutantes: cada marca, a query, o registro, a pasta do reparo e o marcador
  lidos, gravados ou apagados direto; o `begin_configure` sem conferir cada
  marca nova; o reparo sem conferir; o `kinein_dir::remove` sem conferir.
- Pronto: nenhum arquivo nem pasta da IDE sob o `.kinein` é lido, gravado ou
  apagado sem a conferência do guarda, salvo o LCOV (INF-19, no ENG-02). O
  literal `.kinein` que sobra fora do `kinein_dir`, em donos que já
  conferem o caminho, é a migração do ENG-02.

**INF-21 — a string crua no verificador de idioma** (D-17).
- Fazer: o `strip_c_like` pula inteira a string crua do Rust (`r`, `br`, com
  ou sem `#`) e a do C++ (`R"fim(…)fim"`, com os prefixos), quando ela não
  vem colada num nome.
- Commits: um.
- Aceite: cada forma com aspas dentro sai sem deixar palavra, e o
  identificador depois dela na mesma linha é lido; o nome raw do Rust e o
  nome terminado em `r` ou `R` colado numa string seguem como código; o gate
  inteiro não reprova o texto da string e reprova o identificador em
  português depois dela; a contagem do legado não muda.
- Mutantes: sem o prefixo `b`; sem a string crua do C++; o C++ fechando sem
  o delimitador e o Rust sem os `#`; sem conferir o caractere de antes; sem
  os prefixos do C++.
- Pronto: nenhuma string crua do Rust ou do C++ é lida como código.

**INF-18 — a leitura dos arquivos do projeto e das ferramentas** (D-14,
D-16, R-DADOS-04).
- Fazer: antes, medir e listar os leitores; o do `CMakeLists.txt` na
  abertura é o detector de frameworks (`project/detect.rs`), confirmado
  pelo backtrace. Uma leitura comum no `fsops`, que segue o link, abre sem
  bloquear, confere no descritor que é arquivo regular e lê sem limite
  próprio. Todo leitor de produção de arquivo do projeto ou de ferramenta
  passa por ela (D-16): o `CMakeLists.txt` e os presets com os includes, o
  `CMakeCache.txt` (`ProjectCache::read` e os outros), as marcas do
  configure fora do `.kinein`, o `.clangd`, o `Cargo.toml`, o
  `platformio.ini`, o `pyproject.toml`, o `setup.py`, a CDB, a resposta do
  file-api, os artefatos e o ELF, o índice, a busca, a substituição, o git,
  os rascunhos, o flash e o debug. Os presets com um arquivo recusado não
  chamam o `cmake --list-presets`, que pararia no FIFO. O configure recusa
  o `CMakeCache.txt` que não é arquivo regular antes do `CMake`. O
  `projectProblem` do `workspace.open` e do `cmake.status` diz o primeiro
  arquivo recusado entre os que a abertura e o status leem, e a faixa de
  saúde o mostra. O detector, a CDB e o file-api perdem o teto próprio; o
  do trabalho da IDE fica (§A.9). Se a medida em projetos reais pedir um
  teto contra abuso, ele volta ao autor (§G.4). No Windows não há FIFO:
  vale a pasta, provada na etapa 10.
- Commits: um.
- Aceite: o FIFO e a pasta no lugar de cada arquivo que a abertura e o
  status do CMake leem, pelo IPC: o `workspace.open` e o `cmake.status`
  respondem no prazo, o `projectProblem` diz o caminho e o motivo, e nada
  do `.kinein` toma o lugar do arquivo; o `CMakeLists.txt` ligado a um
  arquivo de fora e o `CMakeCache.txt` acima de 1 MiB são lidos como antes;
  cada leitor fora da abertura, no dono, com o FIFO e a pasta, sem prender.
  O teste abre o outro lado do FIFO se a leitura prender.
- Mutantes: abrir bloqueando; não conferir o tipo; aplicar o limite de
  1 MiB; não seguir o link; cada leitor lendo direto; os presets chamando o
  `CMake` com um arquivo recusado; o configure sem conferir o cache; o
  `projectProblem` sem um dos arquivos.
- Pronto: nenhuma leitura de arquivo do projeto ou de ferramenta que abra
  sem conferir o tipo.

**ENG-01 — a ferramenta de mutação.**
- Fazer: `scripts/mutantes.py` e um catálogo por item em `scripts/mutantes/`.
  Cada entrada: arquivo, texto exato, troca e o teste que deve reprovar.
  `--conferir` confere que cada texto ocorre uma vez (entra no bloco
  estático). `--rodar <item>` espelha a árvore de trabalho numa cópia fora do
  checkout, com diretório de build próprio, roda o teste sem a troca (tem de
  passar) e com ela, e classifica: não compila, sobreviveu ou reprovado. Só
  "reprovado" serve.
- Commits: a ferramenta com o catálogo do INF-15, seu primeiro usuário; depois
  o catálogo `gate` (D.3).
- Aceite: o estado do git do checkout é igual antes e depois de `--rodar`.
- Mutantes da própria ferramenta: uma troca que não compila conta como
  falha; uma que sobrevive reprova a rodada; um texto que ocorre duas vezes
  reprova o `--conferir`.
- Pronto: os mutantes de cada item seguinte moram no catálogo. Os mutantes
  antigos, descritos no 40.7, migram na camada "Mutação das guardas" do
  [62 §3.5](62-base-publica-0.4.md).

**INF-16 — o extrator do mapa de módulos.**
- Fazer: o extrator ignora comentário e texto literal, corta no primeiro
  `#[cfg(test)]`, exclui `tests.rs` e `tests/`, e lê os grupos
  `use crate::{a, b::{c}}`. A lista de ciclos conhecidos é apagada. O
  [01](../arquitetura/01-mapa-de-modulos.md) é regenerado no mesmo commit.
- Commits: um.
- Aceite: 213 arestas e 0 ciclos na árvore do 3952b8ad (medido em
  2026-11-13, D-18); se um commit antes do INF-16 mudar as dependências do
  core, o executor mede de novo pelo critério do Fazer antes de começar; o
  resto do mapa sem diferença.
- Defeitos plantados: um ciclo fechado por import agrupado reprova; a menção
  num comentário e a dependência só no `tests.rs` não criam aresta.
- Pronto: R-ARQ-01 reprova qualquer ciclo.

**ENG-02 — o catálogo do `.kinein`.**
- Fazer: a lista declarada dos arquivos próprios no `kinein_dir`; todo
  caminho sob a pasta nasce nele; a conferência de vocabulário (o literal só
  no `kinein_dir`); o teste de adversidade que percorre o catálogo (§C.3).
  Cada entrada declara o limite de tamanho, e o `kinein_dir::read` lê com
  ele; o LCOV da cobertura entra com o limite dele e é lido de volta pelo
  guarda (INF-19, D-14).
- Commits: o catálogo e o teste; depois um por grupo de donos migrados.
- Aceite: o teste de extensão: uma entrada só de teste herda as adversidades
  sem editar o guarda.
- Mutantes: uma entrada fora do teste; um dono montando o caminho à mão; o
  limite único no lugar do da entrada.
- Pronto: 33 → 0 arquivos com o literal fora do `kinein_dir`; R-DADOS-01
  reprova direto; o LCOV acima de 1 MiB é lido inteiro, e o link e o FIFO no
  lugar dele são recusados.

**ENG-03 — o bloco estático, o checkout intacto e o ambiente isolado.**
- Fazer: os 19 passos estáticos rodam logo depois do autoteste dos perfis, em
  paralelo, com a saída de cada um inteira e na ordem declarada; todos os
  que falharam são mostrados. O orquestrador compara o estado do git no
  início, depois do bloco e no fim. O autoteste dos identificadores usa uma
  árvore de fixture no diretório temporário. O orquestrador isola o
  diretório pessoal e a configuração da rodada, mantendo o Rust instalado.
- Commits: a ordem e o paralelismo; a conferência do checkout com o autoteste
  dos identificadores; o isolamento.
- Aceite: D.3 antes e depois; o veredito é o mesmo com e sem o ambiente
  isolado à mão; o pico de memória do bloco fica registrado.
- Defeitos plantados: um passo estático depois do build reprova o autoteste
  dos perfis; uma folha que grava no checkout reprova; a falha de um passo
  paralelo sai no código de saída.
- Pronto: a primeira falha estática em até 10 s (medida no 40.7).

**ENG-04 — o índice de palavras.**
- Fazer: `verificar_qml_mortas.py`, `verificar_qml_alcance.py` e
  `verificar_fiacao_ipc.py` montam um índice de palavras por arquivo, em vez
  de um regex por nome.
- Commits: um por script.
- Aceite: a mesma saída na árvore e nos defeitos plantados de D.3.
- Pronto: cada um abaixo de 1 s (medida no 40.7).

**ENG-05 — o cache do clang-tidy.**
- Fazer: no `--rapido`, uma unidade com resultado limpo guardado não roda de
  novo. A chave junta a versão do clang-tidy, o comando da unidade, os
  `.clang-tidy` que valem e os bytes de cada arquivo que ela inclui (a lista
  vem do compilador; o `NOLINT` mora em comentário, então a saída
  pré-processada não serve). Só resultado limpo é guardado, na pasta de
  build. O `--completo` ignora o cache.
- Commits: um.
- Defeitos plantados: achado numa unidade guardada; achado num cabeçalho que
  ela inclui; `NOLINT` removido; `.clang-tidy` mudado; e uma entrada falsa no
  cache, que o `--completo` ainda reprova.
- Pronto: R-GATE-05 no autoteste dos perfis; a medida com e sem mudança em
  C++ no 40.7.

**ENG-06 — os gates de docs pelo git.**
- Fazer: o `verificar-docs.sh` lista os `.md` como os outros dois gates de
  docs: rastreados e novos não ignorados.
- Commits: um.
- Defeitos plantados: número errado num `.md` rastreado e num novo não
  ignorado reprovam; num ignorado, não.
- Pronto: R-DOC-04.

**ENG-07 — a CI.**
- Fazer: `packaging/ci/Containerfile` (Ubuntu 24.04 com imagem fixada, a
  lista de pacotes que hoje está no `ci.yml`, o Rust do `rust-toolchain.toml`
  e o cargo-deny da versão do `ci.yml`); `scripts/ci-local.sh`, no molde do
  `scripts/empacotar-appimage-portatil.sh`; o `ci.yml` usa o mesmo
  `Containerfile`. O autoteste dos perfis confere que o `ci.yml` só chama o
  orquestrador. O [07 §2.1](../contribuindo/07-fluxo-e-responsabilidades-dos-gates.md)
  registra que a CI nunca executou e por quê.
- Commits: o contêiner e o script; o `ci.yml`.
- Aceite: `scripts/ci-local.sh` verde. Cada falha que o Ubuntu 24.04 revelar
  é item próprio (R-PROC-02).
- Pronto: verde local no contêiner. A primeira rodada verde no GitHub depende
  do desbloqueio da conta e é registrada no 40 quando acontecer.

## E.3 Fase 1 — o core

**ENG-10 — a tabela de rotas.**
- Fazer: o registro de §C.2. A tabela é consultada antes da cadeia; cada
  domínio migra com o `or_else` dele saindo no mesmo commit, e nenhum método
  fica nos dois. No fim, o `match` e a cadeia saem do `lib.rs`. O
  `verificar_fiacao_ipc.py` e o `module_map.py` passam a ler a lista do 03,
  que o teste do registro amarra às rotas, em vez do regex sobre os braços.
  Entra a conferência da versão contra a nota do topo do 03.1.
- Commits: o mecanismo com o primeiro domínio; um por domínio; a retirada da
  cadeia; a conferência da versão.
- Aceite: os testes de despacho iguais; o teste de extensão, com um domínio
  só de teste registrado sem editar `lib.rs` nem o mecanismo.
- Mutantes: rota duplicada; rota fora do 03; método no 03 sem rota; versão
  sem nota.
- Pronto: 40 → 0 `or_else`; o `lib.rs` sem nome de método; a
  [ARCHITECTURE §5](../arquitetura/ARCHITECTURE.md), passo 2, reescrita.

**ENG-11 — as reações registradas.**
- Fazer: as reações do `observe_notification` e a troca de workspace do
  `handlers/workspace.rs` viram entradas do domínio que reage (§C.2).
- Commits: os eventos; a troca de workspace.
- Aceite: os testes de workspace e de eventos iguais; o teste de extensão com
  uma reação e uma troca só de teste.
- Mutantes: uma reação fora do registro; a troca de workspace que pula um
  domínio.
- Pronto: o `lib.rs` sem nome de evento; `handlers/workspace.rs` sem estado
  de outro domínio.

**ENG-12 — as camadas.**
- Fazer: `events` e `framing` na base; os erros por domínio saem do `rpc.rs`
  para os handlers; os 6 arquivos de domínio deixam de nomear o envelope; a
  lista das camadas e o vocabulário no `module_map.py`.
- Commits: `events`; `framing`; o `rpc.rs`; o envelope; a conferência.
- Defeitos plantados: base que importa domínio; domínio que importa a borda;
  envelope num domínio.
- Pronto: R-ARQ-02, R-ARQ-03 e R-ARQ-04 reprovam direto.

**ENG-13 — o `workspace` em dois.**
- Fazer: a criação de projeto (`create`, `c_options`, `cmake_template`,
  `project_options`) vira módulo próprio; o `workspace` fica com o estado.
- Commits: um. Renomear pelo compilador (R-PROC-03).
- Pronto: o `workspace` só depende da base; o mapa regenerado.

**ENG-14 — os handlers fora do `impl Core`.**
- Fazer: cada método de handler vira função livre no arquivo do domínio,
  ligada pela tabela. Começar pelos arquivos que tocam 0 ou 1 campo. Entra a
  catraca de migração dos arquivos com `impl Core`.
- Commits: um por domínio.
- Aceite: os testes iguais; nenhum arquivo de teste editado além dos nomes
  que o compilador exigir.
- Pronto: 73 → 1 arquivo com `impl Core` (o `lib.rs`); a catraca apagada; a
  [08](../contribuindo/08-convencoes-codigo-testes-commits.md) corrigida.

**ENG-15 — o estado por domínio.**
- Fazer: os campos de um domínio viram o estado dele, com o tipo no módulo do
  domínio; os serviços ficam declarados no `module_map.py`; o construtor
  deixa de repetir os campos. Entra a conferência de R-ARQ-08 e a matriz
  handler × estado no mapa gerado.
- Commits: um por domínio, começando pelo banco (6 campos).
- Aceite: os testes iguais; o teste de extensão do ENG-10 ganha um domínio
  com estado.
- Mutantes: campo não declarado; handler que toca estado alheio sem a
  declaração.
- Pronto: todo campo do `Core` é serviço declarado ou estado de um domínio;
  a catraca apagada.

## E.4 Fase 2 — a ponte e a QML

Antes do UI-02, medir o primeiro quadro e a digitação com
`scripts/medir-performance.sh`: é a base de comparação da fase (Parte F).

**UI-02 — a faixa de saúde e o despacho.**
- Fazer: cada motivo como arquivo próprio, com as entradas dele; a lista
  ordenada num arquivo de composição; o `update()` escolhe o primeiro
  levantado. A metade do despacho fecha com o domínio `workspace` do ENG-20.
- Commits: o mecanismo com dois motivos; os outros motivos; a metade do
  despacho, no ENG-20.
- Aceite: os harnesses da faixa e do roteador iguais; o teste de extensão com
  um motivo só de teste; e um campo novo do `workspace.open`, só de teste,
  chegando ao consumidor sem editar o despacho. Prova na UI nativa.
- Mutantes: a ordem trocada; um motivo fora da lista.
- Pronto: o controller da faixa sem nome de motivo.

**ENG-20 — a ponte por domínio.**
- Fazer: o transporte e o objeto de domínio de §C.4. **Piloto:** `workspace`
  e `git`; medir o custo de cada um (arquivos, harnesses, tempo) e registrar.
  Depois, todos os domínios, do menor para o maior, com a API antiga do
  domínio saindo no mesmo commit. O `verificar_fiacao_ipc.py` e o
  `module_map.py` acompanham, com os mesmos achados em D.3.
- Commits: o transporte com o `git`; o `workspace` (fecha o UI-02); um por
  domínio.
- Aceite: os harnesses iguais; o teste de extensão com um domínio só de
  teste; primeiro quadro e digitação sem piora.
- Mutantes: resultado sem entrada na tabela; evento sem tratador; pedido
  seguinte à abertura fora do registro.
- Pronto: 287 → 0 ramos por nome de método; o transporte sem nome de método;
  o cabeçalho do transporte dentro do limite comum. A retirada da categoria
  "cabeçalho que só declara" do `verificar-arquitetura.sh` é decisão do autor
  no fechamento (§G.4).

**ENG-21 — o módulo de domínio na UI.**
- Fazer: `<Dominio>Domain.qml` instancia o cliente, os roteadores e o
  controller; `AppDomains.qml`, `AppRouters.qml` e `Main.qml` viram listas de
  domínios ([27](../arquitetura/27-modulos-por-dominio.md) §4).
- Commits: um por domínio, junto ou logo depois do commit dele no ENG-20.
- Pronto: um domínio novo na UI é um arquivo e uma linha.

**ENG-22 — a QML de área sem a ponte.**
- Fazer: as leituras (`workspaceRoot` é a maior) vêm do controller; as
  chamadas passam pelo roteador. Entra a conferência de R-ARQ-10.
- Commits: um por arquivo de área.
- Pronto: 8 → 0 arquivos; R-ARQ-10 reprova direto.

**ENG-23 — o cliente falso com dono.**
- Fazer: o piloto do ENG-20 mede as duas formas: o cliente real sobre um
  transporte falso, ou um falso por domínio conferido contra o cabeçalho. A
  escolha é decisão do autor (§G.4). Os harnesses param de redeclarar sinal.
- Commits: o mecanismo; um por grupo de harnesses.
- Pronto: 36 → 0 sinais redeclarados, com o `workspaceProjectProblem` (D-16);
  R-TEST-06 reprova direto.

**ENG-24 — a fachada do editor.**
- Fazer: cada consumidor passa a falar com o dono
  ([32](../arquitetura/32-editor-por-responsabilidade.md) §9); o membro da
  fachada sai quando o último consumidor sai.
- Commits: um por área consumidora.
- Aceite: os harnesses do editor iguais; prova na UI nativa (digitar, salvar,
  completar, formatar, reabrir).
- Pronto: `scripts/arquitetura-baseline.txt` vazio; o 40 §2 e §5 e o 32 §8
  atualizados.

**ENG-25 — os arquivos no limite.**
- Fazer: remedir depois das fases 1 e 2. Para cada arquivo a 99% ou mais,
  registrar no 40.7 a responsabilidade misturada, ou que ele faz uma coisa
  só, com a palavra do vocabulário que prova. Dividir, por responsabilidade,
  os que misturam (R-ARQ-13).
- Commits: um por divisão.
- Pronto: todo arquivo a 99% ou mais tem o registro; nenhuma divisão por
  tamanho.

## E.5 Fase 3 — testes, listas e supressões

**INF-06 — o prazo como parâmetro.**
- Fazer: o teste que não é sobre o prazo recebe um prazo folgado; o que é
  sobre o prazo usa um compilador falso que nunca termina, com prazo curto.
- Mutantes: o prazo ignorado; o prazo que não cancela.
- Pronto: 50 rodadas seguidas sob carga plena, sem falha.

**ENG-30 — a espera fixa.**
- Fazer: o ajudante único de espera por condição; onde o tempo é o estímulo
  (debounce, granularidade do relógio do arquivo), a passagem de tempo com o
  motivo escrito na chamada. O mesmo nos scripts de prova. Entra a
  conferência.
- Commits: o ajudante com o primeiro arquivo; um por domínio.
- Pronto: nenhum `sleep` de sincronização; a catraca apagada.

**ENG-31 — os ajudantes de teste.**
- Fazer: os 19 ajudantes idênticos vão para o dono comum dos testes; entra a
  conferência.
- Pronto: R-TEST-06 reprova direto.

**ENG-32 — a regra do `settings.json` na partida.**
- Fazer: medir antes o custo de chamar o `kinein-core` num modo de uma
  resposta antes do `QGuiApplication`. Até 50 ms: a partida usa a resposta do
  core e a leitura em `ui/src/startup_web.cpp` sai. Acima: parar e perguntar
  (D-09). O modo novo é linha de comando, não IPC; a forma dele é registrada
  no [04](../arquitetura/04-boot-e-comunicacao.md) antes do código.
- Aceite: os casos do `tst_startup_web` iguais, agora contra o core.
- Pronto: `ui/src` sem cópia da regra.

**ENG-33, ENG-34, ENG-35 — as três listas da QML.**
- Fazer: cada função morta é apagada ou ligada; ligar o que "devia estar em
  uso" (`setBreakpointCondition`, por exemplo) é decisão do autor (§G.4).
  Cada derivação repetida ganha um dono. Cada literal vai para o `Theme`.
- Pronto: as três linhas de base apagadas; os três gates sem catraca.

**ENG-36 — supressões e aceitos.**
- Fazer: cada `#[allow]`, `NOLINT` e `shellcheck disable` é removido ou fica
  com o motivo na linha; cada entrada de lista de aceitos é ligada, apagada
  ou fica com motivo e data. Entra a conferência do motivo.
- Pronto: supressão sem motivo reprova.

**ENG-37 — os identificadores em português.**
- Fazer: renomear por `scripts/rename_identifiers.py` e pelo compilador, uma
  pasta por commit. Nome que vai ao fio ou a arquivo gravado não muda de
  forma: se mudar, é contrato primeiro (R-IPC-01) e parar para perguntar.
- Pronto: a linha de base apagada; identificador em português reprova direto.

**ENG-38 — a auditoria do checklist de riscos.**
- Fazer: para cada um dos 10 riscos do
  [16](../arquitetura/16-checklist-de-riscos-ocultos.md), apontar o teste ou
  o gate que o cobre hoje. O que não tiver vira item próprio no 62 §3.4.
- Pronto: a tabela risco → verificação no próprio 16.

**ENG-39 — os adiados do Windows.** Na etapa 10, conforme o
[60](60-windows-e-edicao-especial.md) §3.3. Pronto: linha de base apagada.

**Pronto da engenharia inteira:** toda linha da §B.2 no destino; nenhuma
catraca de migração de pé (R-GATE-07); o gate completo estrito verde; a meta
de D.2 medida; o 62 §3.4 com cada item fechado e a evidência.

---

# Parte F — O que não muda

1. **O fio.** Nenhum item desta engenharia muda método, parâmetro, resultado
   ou evento. Se um item precisar mudar, é contrato primeiro (R-IPC-01) e
   quem executa para e pergunta.
2. **O comportamento visível.** Telas, textos, atalhos e ordem das ações
   ficam iguais, salvo decisão do autor.
3. **Os arquivos gravados.** Formato e versão de esquema dos arquivos do
   `.kinein` e da configuração global ficam iguais.
4. **O desempenho.** Primeiro quadro e digitação não pioram contra a base
   medida no início da fase 2.
5. **A regra de produto.** O CMake segue como fonte da verdade, e a IDE
   orquestra os motores maduros sem reimplementá-los (62 §3).
6. **Os limites e as categorias da catraca,** salvo a decisão prevista no
   ENG-20.

---

# Parte G — Como executar

## G.1 Antes do primeiro item

Conferir o checkout pelo git (`git status`, `git log`), e não por texto. Ler
esta página, o [62 §3.4](62-base-publica-0.4.md) do item e o dono que ele
aponta. O gate rápido (`bash scripts/verificar.sh --rapido`; os perfis estão
em [build/comandos](../build/comandos-de-build-e-verificacao.md)) tem de
estar verde antes de começar. O ritual de uma mudança, passo a passo, é o do
[contribuindo/03](../contribuindo/03-o-ritual-de-uma-fatia.md); esta parte só
acrescenta as regras e as paradas.

## G.2 O ciclo de um item

| Passo | O que fazer | Regras |
| --- | --- | --- |
| 1 | Conferir o registro do item no 62 §3.4; achado novo vira item antes de tudo | R-PROC-02 |
| 2 | Se o fio muda: 03, 03.1 e `PROTOCOL_VERSION` antes do handler | R-IPC-01 |
| 3 | Escrever o teste de aceite e o de extensão sobre a base verde | R-TEST-01, R-TEST-05, R-TEST-03 |
| 4 | Fazer a mudança, no menor commit que fecha o item | R-ARQ-*, R-DADOS-*, R-PROC-03 |
| 5 | Registrar os mutantes no catálogo e rodá-los na cópia | R-TEST-02, R-GATE-03 |
| 6 | Se a mudança é visual: prova na UI nativa isolada | Parte F, 2 |
| 7 | Gate rápido, sozinho, até o verde | R-PROC-05 |
| 8 | Atualizar os donos: o estado no 62 §3.4, o 40 (o cabeçalho cabe em 60 linhas) e a entrada do 40.7 com as medidas | R-DOC-01, R-DOC-03 |
| 9 | Checklist do commit (§G.3) e commit | todas as de checklist |

## G.3 O checklist do commit

1. O commit fecha um item, e a mensagem cita o ID. Nenhum achado novo foi
   corrigido de carona (R-PROC-02, R-DEP-02).
2. Toda decisão reversível tomada está no dono, com data e "decisão do
   autor" (R-PROC-04).
3. Nenhuma linha de base subiu, nenhuma lista de aceitos ou supressão nasceu
   sem decisão na §A.9 (A.0).
4. Nenhuma regra do core foi copiada para a UI, para um script ou para um
   ajudante (R-ARQ-12).
5. Em refatoração, os testes existentes não foram editados, salvo os nomes
   que o compilador exigiu, listados na mensagem (R-TEST-07).
6. Em divisão de arquivo, a mensagem diz a responsabilidade que saiu e a
   palavra que o arquivo deixou de nomear (R-ARQ-13).
7. Todo rename foi pelo compilador ou pela ferramenta (R-PROC-03).
8. O fato novo está num dono só; os outros apontam (R-DOC-03).
9. Se a CI rodou vermelha, a causa está no 40 (R-GATE-08).

## G.4 Quando parar e perguntar

Pergunta curta, com a recomendação primeiro. Sempre que:

- a forma esboçada na Parte C não servir no primeiro domínio;
- um item precisar mudar o fio, um arquivo gravado ou o comportamento visível;
- um limite, uma categoria ou uma linha de base precisar mudar (inclusive a
  categoria do cabeçalho, no fim do ENG-20);
- o piloto do ENG-20 fechar: a ordem dos domínios e a forma do cliente falso
  (ENG-23) são decididas com a medida;
- a chamada da partida passar de 50 ms (ENG-32);
- uma função da linha de base da QML parecer que devia estar em uso (ENG-33);
- um rename tocar nome que vai ao fio ou a arquivo gravado (ENG-37);
- o contêiner da CI revelar falha que não é do ambiente (ENG-07);
- um teste existente só passar se for editado;
- a execução achar algo fora deste plano (Parte H).

---

# Parte H — Os riscos

## H.1 Como desfazer

Cada commit fecha um item ou um domínio de um item, com o gate verde: desfaz
com `git revert` desse commit, sem tocar nos vizinhos.

| Passo | Risco | Como se vê | Como desfazer |
| --- | --- | --- | --- |
| INF-15, ENG-02 | Arquivo válido recusado pela leitura guardada | Projeto real que abria passa a mostrar problema | Reverter o commit do dono; o guarda fica |
| ENG-03 | Falha de um passo paralelo perdida | O defeito plantado de D.3 passa verde | Reverter o paralelismo; a ordem nova fica |
| ENG-05 | Cache que esconde achado | O `--completo` reprova o que o `--rapido` passou | Desligar o cache pela variável do script e reverter |
| ENG-10, ENG-11 | Método ou reação perdidos na migração | O teste do registro e a fiação IPC reprovam | Reverter o commit do domínio: a cadeia volta para ele |
| ENG-14, ENG-15 | Troca de ordem na inicialização ou na troca de workspace | Testes de workspace e exercitação | Reverter o commit do domínio |
| ENG-20, ENG-21 | Ordem dos pedidos depois de abrir o workspace; sinal sem ouvinte | Harnesses, fiação IPC, exercitação e a prova na UI nativa | Reverter o commit do domínio: a API antiga dele volta inteira |
| ENG-24 | Regressão no editor | Harnesses do editor e a prova nativa | Reverter o commit da área consumidora |
| ENG-32 | Partida mais lenta ou sem core | Medida do primeiro quadro; o caso do core ausente | Reverter: a leitura em C++ volta |
| ENG-37 | Rename que muda o fio ou arquivo gravado | Testes de protocolo e de reabertura | Reverter o commit da pasta |

Um domínio migrado nunca fica pela metade num commit: a parte antiga dele sai
no mesmo commit em que a nova entra.

## H.2 O que está fora do plano

Achado novo durante a execução: registrar como item próprio no 62 §3.4, com
evidência e severidade, sem corrigir de carona (R-PROC-02). Se ele muda a
ordem, o desenho ou uma regra deste documento, a execução para nesse ponto e
o item volta para a revisão de arquitetura com o autor. Este documento só
muda por essa revisão; a Parte A, só por decisão registrada na §0.

## H.3 O que este plano não mediu

- O resultado real do `--ci-rapida --estrito` no Ubuntu 24.04 (ENG-07).
- O custo de migrar um domínio da ponte (o piloto do ENG-20 mede).
- O custo da chamada do core na partida (ENG-32 mede antes).
- A cobertura de cada risco do 16 (ENG-38 audita).
- O pico de memória do bloco estático em paralelo (ENG-03 registra).

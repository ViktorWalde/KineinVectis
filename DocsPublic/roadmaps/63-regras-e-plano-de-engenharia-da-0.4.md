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
| D-19 | Depois do INF-18 (2026-11-13), com a medida dos gates no 2e5fa164 (§D.5): o endurecimento das catracas e dos gates entra antes do catálogo `gate` do ENG-01, que seria montado sobre verificadores que deixaram passar 84 de 120 defeitos plantados. A ferramenta do ENG-01 entra primeiro, só com o catálogo do INF-15: ela não depende de catraca nenhuma, e é ela que roda os defeitos plantados do bloco. Depois vêm ENG-40, ENG-41, INF-16, ENG-42, ENG-43, ENG-44, ENG-45 e ENG-46, e o catálogo `gate` fecha o bloco (§E.1). |
| D-20 | Na mesma revisão (2026-11-13): quatro regras novas na Parte A: R-GATE-09 (linha de base sem folga), R-GATE-10 (linha de base só desce, e nenhum comando a cria nem a sobe), R-GATE-11 (um leitor por linguagem) e R-GATE-12 (gate falha fechado). A verificação da R-TEST-06 passa a citar a catraca do ENG-44. |
| D-21 | Na mesma revisão (2026-11-13): mutante equivalente não é estado final. Cada um dos 10 registrados (2 no INF-17, 2 no INF-20 e 6 no INF-18) vira pendência com dono: uma regra estática o reprova, ou o código que ele troca sai. Os 4 do INF-17 e do INF-20 são da conferência de vocabulário do ENG-02; os 6 do INF-18, do ENG-47, item novo. A lista de pendentes é catraca de migração (R-GATE-07). A R-TEST-02 e a verificação da R-DADOS-04 mudam junto. |
| D-22 | Na mesma revisão (2026-11-13): a dívida que o leitor certo revela não entra em linha de base que já existe. Entra em catraca de migração nova, criada pelo item que troca o leitor, com zero como pronto e no dono que já existe: ENG-14 (`lib.rs`), ENG-17 (item novo, `terminal/session.rs`), ENG-33, ENG-34, ENG-35, ENG-37 e ENG-39 (§B.2). O bloco só endurece gate: não corrige código de produto. O ENG-04 é absorvido pelo ENG-43. O tempo em milissegundos e a rede do `deny` vão para o ENG-03; a sensibilidade dos gates de docs, para o ENG-06; o conteúdo das supressões e dos aceitos segue no ENG-36. |

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
- Verificação: o teste de adversidade do INF-18, com mutantes; a conferência
  de que o conteúdo de arquivo só é lido por um leitor com dono (ENG-47).

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
cópia, nunca no checkout vivo.** Mutante que compila e sobrevive não fica
registrado como equivalente: uma regra estática o reprova, ou o código que
ele troca sai (D-21).
- Porquê: teste verde porque a guarda nunca foi exercida; e os mutantes eram
  scripts avulsos sobre a árvore de trabalho (ENG-01). O mutante dito
  equivalente é uma guarda que nada confere.
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
- Verificação: conferências no gate rápido (ENG-31; para o sinal da ponte, a
  catraca de migração do ENG-44, que o ENG-23 leva a zero).

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

**R-GATE-09 — Linha de base não tem folga: a contagem medida é igual à
gravada, chave por chave.** Contagem acima reprova. Contagem abaixo também
reprova, e a linha desce no mesmo commit. O mesmo vale para lista de exceção
de gate: a entrada que não casa mais com nada reprova.
- Porquê: a base com folga deixa a dívida voltar sem que nada reclame
  (ENG-40, ENG-45).
- Alcance: toda linha de base e toda lista de exceção que um gate lê.
- Verificação: `scripts/ratchet.py`, o único que compara linha de base
  (ENG-40); a conferência das listas de exceção (ENG-45).

**R-GATE-10 — Linha de base só desce, e nenhum comando a cria nem a sobe.**
Base nova só nasce pelo item que a Parte E nomeia, com a decisão da §0 no
cabeçalho do arquivo.
- Porquê: os comandos de atualizar regravavam a base com a dívida nova dentro
  (ENG-40).
- Alcance: toda linha de base, e todo comando do repositório que grava uma.
- Verificação: `scripts/ratchet.py --history` confere cada base contra a
  história do git (ENG-40); a tabela da §B.2 lista as bases.

**R-GATE-11 — Cada linguagem tem um leitor só, com dono, e todo gate que lê
código lê por ele.** O leitor separa código, comentário e texto literal, e é
provado por um corpus das formas difíceis da linguagem.
- Porquê: cada verificador tinha o seu regex, e o mesmo erro de leitura
  aparecia em vários (INF-21, INF-16, ENG-41).
- Alcance: todo gate que lê Rust, QML e JavaScript, C++ ou shell. O Python é
  lido pela árvore sintática da própria linguagem.
- Verificação: o corpus do leitor no bloco estático, e a conferência de que
  nenhum verificador lê arquivo de código fora do leitor (ENG-41).

**R-GATE-12 — Gate falha fechado.** Leitor que quebra, arquivo do alcance que
não se lê, conjunto vazio onde a regra espera itens e ferramenta ou rede
ausente nunca resultam em verde: reprovam, ou saem como NÃO PROVADO.
- Porquê: passo que dá verde sem ter conferido não se distingue do que
  conferiu (ENG-45).
- Alcance: todo passo do orquestrador.
- Verificação: os defeitos plantados do ENG-45 no catálogo `gate`; o
  protocolo NÃO PROVADO do orquestrador (R-TEST-01).

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
| ENG-04 | Três checagens com um regex por nome sobre o texto todo | Fiação IPC 14,1 s, funções mortas 11,0 s, alcance 10,6 s; protótipo com índice: 11,07 s → 0,37 s, mesmo resultado. Absorvido pelo ENG-43 (D-22) | baixa |
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
| ENG-40 | Linha de base com folga, e comando que a sobe | Medido no 2e5fa164 (2026-11-13): só 1 das 6 catracas reprova abaixo da base; 3 comandos de atualizar sobem a base sem conferir e 2 sobem quando a base é apagada antes (62 §3.4) | alta |
| ENG-41 | Um leitor de código por verificador | Medido no 2e5fa164 (2026-11-13): 14 de 15 formas legítimas plantadas foram reprovadas; o idioma reprova 4 de 4 e deixa passar 7 de 10 defeitos (62 §3.4) | alta |
| ENG-42 | Gates que leem Rust errado | Medido no 2e5fa164 (2026-11-13): arquitetura 3 de 8 defeitos, transição 1 de 7, adiados 3 de 10; 2 arquivos acima do limite fora da base (62 §3.4) | alta |
| ENG-43 | Gates que leem QML e C++ errado | Medido no 2e5fa164 (2026-11-13): mortas 1 de 7 defeitos, duplicação 2 de 10, tokens 5 de 14, alcance 1 de 6, auto-referência 2 de 6, propriedades 3 de 6, atalhos 2 de 5, fiação 3 de 8 (62 §3.4) | alta |
| ENG-44 | R-TEST-06 sem gate para o sinal da ponte | Medido no 2e5fa164 (2026-11-13): 19 harnesses redeclaram 340 sinais da ponte, e nenhum passo confere (62 §3.4) | média |
| ENG-45 | Gate que dá verde sem conferir; exceção sem conferência | Medido no 2e5fa164 (2026-11-13): a arquitetura passa com o leitor quebrado; o `qml-logica` dá verde com 0 testes; uma entrada velha nos aceitos da fiação passa (62 §3.4) | alta |
| ENG-46 | Arquivos que nenhum gate lê | Medido no 2e5fa164 (2026-11-13): os 8 arquivos C++ de `ui/tests` fora do clang-format e do clang-tidy, com 13 erros de formatação em 2 deles; script novo não rastreado e 2 formas de shebang fora do shellcheck; 2 `.pyc` rastreados apesar do `.gitignore`, com caminho pessoal dentro (62 §3.4) | média |
| ENG-47 | Leitura de conteúdo sem dono; mutantes ditos equivalentes | Medido no 2e5fa164 (2026-11-13): 19 leituras diretas em 14 arquivos de produção do core; 10 mutantes registrados como equivalentes (62 §3.4) | média |
| ENG-17 | `terminal/session.rs` acima do limite, fora da base | Medido no 2e5fa164 (2026-11-13): 661 linhas de produção, contadas como 135 (62 §3.4) | média |

Os itens C-01 a C-03, CPP-01, UI-01 e os C1 abertos seguem no 62 §3.4 e na
etapa deles; este plano não os altera.

## B.2 Linhas de base e listas, uma a uma (medido em 2026-11-13, no 2e5fa164)

O número vivo de cada base é o do arquivo e o da saída do gate. Esta tabela é
o registro da medida, com o comando; quem executa mede de novo ao abrir o
item.

| Lista | Medida | Comando | Destino | Item |
| --- | --- | --- | --- | --- |
| `scripts/arquitetura-baseline.txt` | 1 arquivo | `grep -c . <base>` | Zero; a base é apagada | ENG-24 |
| `scripts/module_map.py` (ciclos conhecidos) | 2 | `python3 scripts/module_map.py --check` | Zero; a lista é apagada | INF-16 |
| `scripts/qml-mortas-baseline.txt` | 12 | `grep -c . <base>` | Zero; a base é apagada | ENG-33 |
| `scripts/qml-duplicacao-baseline.txt` | 11 | `grep -c . <base>` | Zero; a base é apagada | ENG-34 |
| `scripts/qml-tokens-baseline.txt` | 7; uma é a definição do token no `Theme.qml` e sai no ENG-43 | `grep -vc '^#' <base>` | Zero; a base é apagada | ENG-35 |
| `scripts/identifier-language-baseline.txt` | 3319 pares, 14.067 ocorrências (eram 3321 e 14.074 em 2026-11-12) | `grep -vc '^#' <base>`; a soma da terceira coluna | Zero; a base é apagada | ENG-37 |
| `scripts/adiados-windows-baseline.txt` | 76 testes e 50 `cfg(unix)` | o próprio arquivo | Zero, na etapa 10 | ENG-39 |
| Listas de aceitos em `scripts/verificar_fiacao_ipc.py` | 8 | `python3 scripts/verificar_fiacao_ipc.py` ("8 excecoes ditas") | Cada entrada ligada, apagada ou mantida com motivo e data | ENG-36 |
| Supressões no código | 71: 25 `#[allow(`, 8 `#![allow(`, 13 `NOLINT` em `ui/`, 25 `shellcheck disable` (eram 69 em 2026-11-12); mais 3 `noqa` e `type: ignore` em Python, que a primeira medida não contou | `git grep -c` de cada forma, por extensão | Cada uma removida ou com o motivo na linha | ENG-36 |
| Catracas de migração novas (ENG-14, ENG-15, ENG-20, ENG-22, ENG-23, ENG-30) | criadas pelo item | — | Zero, e apagadas (R-GATE-07) | o próprio |

**Catracas de migração do D-22**, criadas pelo item que troca o leitor. A
medida é a do protótipo de leitor da revisão, fora do repositório, no
2e5fa164 (2026-11-13); o item que cria a catraca mede de novo, pelo critério
do Fazer dele, e grava o que medir.

| Catraca | Criada por | Medida | Zerada por |
| --- | --- | --- | --- |
| Arquitetura, o que o corte escondia | ENG-42 | 2 arquivos: `crates/kinein-core/src/lib.rs` com 567 linhas de produção e `crates/kinein-core/src/terminal/session.rs` com 661 | ENG-14 e ENG-17 |
| Funções QML mortas, citadas só em texto literal | ENG-43 | 1: `DataSourceController.qml::introspect` | ENG-33 |
| Derivações repetidas em outra forma | ENG-43 | 9 chaves a mais (20 contra 11) e 8 valores de `case` repetidos | ENG-34 |
| Cores e fonte fora do `Theme` em outra forma | ENG-43 | 7 cores (4 `property color` e 3 `Qt.rgba` com número) e 1 `font.family` | ENG-35 |
| Identificadores que o leitor escondia, e os arquivos fora do escopo | ENG-41 | 8 pares em `remote/discover.rs`; 11 pares em 9 arquivos `.c` | ENG-37 |
| Adiados no Windows em outra forma | ENG-42 | 3 retornos cedo por `cfg!(…)` em teste | ENG-39 |
| Sinal da ponte redeclarado em harness | ENG-44 | 19 arquivos, 340 redeclarações | ENG-23 |
| Leitura direta de conteúdo no core | ENG-47 | 19 leituras em 14 arquivos | ENG-47 |
| Mutantes pendentes | ENG-01 | 10 | ENG-02 (4) e ENG-47 (6) |

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
| Catraca comum (ENG-40), de 2026-11-13 | Contagem acima e abaixo da base, chave nova, chave que sumiu; base que subiu na história | Nenhum por construção: comparação exata, chave por chave | Milissegundos por base | Reprova e diz a linha a descer; nenhum comando sobe |
| Leitor por linguagem (ENG-41), de 2026-11-13 | A forma difícil lida errado: comentário, texto literal, `#[cfg(test)]`, import agrupado, bloco de uma linha | No 2e5fa164: 14 de 15 formas legítimas reprovadas pelos leitores de hoje; o corpus do leitor mede o de cada forma | A medir no item | O corpus reprova a leitura diferente da esperada |
| Sinal da ponte em harness (ENG-44), de 2026-11-13 | Harness que redeclara sinal do `CoreClient` | Nenhum por construção: nome exato contra o cabeçalho | Menos de 1 s | Catraca de migração até zero |
| Falha fechada e exceção conferida (ENG-45), de 2026-11-13 | Passo que não leu, conjunto vazio, entrada de exceção que não casa mais | A medir no item | Milissegundos | Reprova, ou NÃO PROVADO |
| Leitura com dono (ENG-47), de 2026-11-13 | `fs::read`, `fs::read_to_string` e `File::open` fora dos leitores guardados | Nenhum: cada ocorrência de hoje é a dívida do item (19) | Menos de 1 s, no passo do mapa | Catraca de migração até zero; depois reprova direto |

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
| Índice de palavras em três checagens (ENG-43, que absorveu o ENG-04) | 11,07 s (funções mortas) | 0,37 s, mesmo resultado | medido em uma; as outras duas têm o mesmo perfil |
| Autoteste dos identificadores sobre uma árvore de fixture (ENG-03) | 9,5 s | a medir | — |
| Cache do clang-tidy (ENG-05) | 52 s | cerca de 2 s sem mudança em C++ (o pré-processamento das 52 unidades leva 1,2 s) | estimado |

**Meta**, a conferir por quem executa: gate rápido em até 125 s sem mudança
em C++ e até 175 s com; a primeira falha estática em até 10 s. O `cargo test`
não entra na meta: os 38 testes de 5 s ou mais somam 287 s de 567 s e são
provas com ferramenta real (R-TEST-01).

**O que não muda:** a ordem continua sendo só do orquestrador; o commit
sempre passa pelo `--rapido` inteiro; não há perfil que pule passo pela
mudança. O bloco estático não roda junto do `cargo test`.

**O uso da máquina (medido em 2026-11-13, no 2e5fa164, 16 núcleos).** Cada
passo rodado sozinho numa cópia aquecida; "núcleos" é o tempo de CPU dividido
pelo de parede.

| Passo | Parede | CPU | Núcleos | O que a medida mostra | Item |
| --- | --- | --- | --- | --- | --- |
| `verificar-cpp.sh` | 39,4 s | 546,3 s | 13,9 | Já usa a máquina; o que falta é não refazer | ENG-05 |
| `cargo test --workspace` | 43,9 s | 226,7 s | 5,2 | Fora da meta (prova com ferramenta real) | — |
| `verificar-exercitacao.sh` | 29,5 s | 2,7 s | 0,09 | 9 `sleep` fixos somam 29 s: o passo é espera | ENG-30 |
| `verificar-qml.sh` | 12,4 s | 12,6 s | 1,0 | Um processo só para os 447 arquivos | ENG-03 |
| autoteste dos identificadores | 13,2 s | 13,2 s | 1,0 | Um núcleo | ENG-03 |
| funções mortas, alcance e fiação | 11,2 s, 10,8 s e 11,5 s | o mesmo | 1,0 cada | Um regex por nome, em série | ENG-43 |
| `verificar-qml-logica.sh` | 9,0 s | 45,4 s | 5,0 | Já roda em paralelo | — |
| `verificar-shell.sh` | 4,3 s | 5,6 s | 1,3 | Um arquivo por vez | ENG-03 |
| `verificar-binario-abre.sh` (só no `--completo`, duas vezes) | 38,5 s | 23,1 s | 0,6 | Espera; a causa não foi lida nesta revisão | ENG-03 |

Dos 201 s do `--rapido`, 93 s são de passos que usam um núcleo ou menos dos
16. Nenhum número de "depois" entra aqui sem medida: cada item acima mede o
seu e registra no 40.7.

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

**Desde o D-19 (2026-11-13)**, o catálogo `gate` é a soma dos defeitos
plantados dos itens ENG-40 a ENG-46 e dos outros que plantam defeito em
gate: INF-16, ENG-03, ENG-05, ENG-06, ENG-36 e ENG-47. Cada entrada diz o
veredito esperado: o defeito reprova, e a forma legítima passa. O catálogo
inteiro roda no `--completo`, e o autor o roda quando quiser: a saída diz,
por regra, quantos defeitos ela reprova. As metas, por regra de gate:

| Meta | Critério |
| --- | --- |
| Sensibilidade | Ao menos 95% dos defeitos plantados nas formas difíceis da linguagem reprovam; no catálogo `gate`, 100% |
| Precisão | Ao menos 95% do que a regra reprova na árvore é defeito; no catálogo, toda forma legítima passa |
| Folga | Zero, chave por chave (R-GATE-09) |
| Base | Nenhum comando a cria nem a sobe (R-GATE-10) |
| Determinismo | A mesma saída em toda rodada, com outra semente de hash, outro locale e outro fuso |
| Custo | Medido em milissegundos, por passo, no resumo da rodada (ENG-03) |

As formas difíceis são: comentário, texto literal, string crua,
`#[cfg(test)]`, import agrupado, macro, here-doc, binding de várias linhas,
bloco de uma linha e o nome que só começa ou termina igual. A medida de
partida de cada regra está na §D.5.

## D.4 O gate local e a CI

A CI roda `bash scripts/verificar.sh --ci-rapida --estrito` dentro do
contêiner de `packaging/ci/`, e `scripts/ci-local.sh` roda o mesmo contêiner
com o `podman` (ENG-07). O perfil é um subconjunto do gate local, escolhido no
orquestrador ([07 §2.1](../contribuindo/07-fluxo-e-responsabilidades-dos-gates.md)).
O desbloqueio da conta no GitHub não é do repositório: é do autor.

## D.5 Os passos, um a um (medido em 2026-11-13, no 2e5fa164)

Como foi medido, numa cópia da árvore fora do checkout:

- **Defeitos e formas legítimas.** 135 sondas, em 16 passos: cada uma
  aplicada sozinha na cópia, o comando do passo rodado, a cópia desfeita. A
  sonda é uma forma difícil da linguagem (§D.3), não um defeito comum: mede o
  que o leitor do passo deixa passar. Dos 120 defeitos, 36 reprovaram; das 15
  formas legítimas, 14 foram reprovadas por engano. A lista de cada passo
  está no item que o endurece (§E.2) e no [62 §3.4](62-base-publica-0.4.md).
- **Folga e comando.** A contagem posta abaixo da base, e o comando de
  atualizar rodado com uma dívida nova na árvore.
- **Custo.** Uma rodada aquecida de `bash scripts/verificar.sh --completo`:
  282 s nos 37 passos e 201 s nos 32 do `--rapido`. O orquestrador mede em
  segundos inteiros e lista só os 5 maiores; 15 passos saem com 0 s. Os passos
  estáticos foram medidos sozinhos, 3 vezes cada, em milissegundos; a tabela
  traz a mediana.
- **Determinismo.** Os 17 passos estáticos, 3 rodadas com `PYTHONHASHSEED` 1,
  2 e 3, `LC_ALL` `C.UTF-8`, `C` e `pt_BR.UTF-8`, e outro fuso na terceira:
  a saída e o código de saída foram iguais em todos. O `verificar-deny.sh`
  reprovou uma rodada do gate por falta de rede e passou nas duas seguintes,
  na mesma árvore.

"Não medido" quer dizer que esta revisão não plantou defeito no passo: o
catálogo `gate` do ENG-01 mede (§H.3).

| # | Passo | Promete | Como lê | Defeitos reprovados | Legítimas reprovadas | Base e comando | Custo |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | `test_verification_profiles.py` | Perfis, fail-fast e NÃO PROVADO do orquestrador | Autoteste | não medido | — | — | 1 s |
| 2 | os diretórios de build existem | O preset está configurado | — | não medido | — | — | 0 s |
| 3 | `cmake --build --preset dev-local` | A UI de debug compila | O compilador | não medido | — | — | 0 s, aquecido |
| 4 | `cargo fmt --all --check` | Formatação do Rust | O rustfmt | não medido | — | — | 1 s |
| 5 | `cargo test --workspace` | Os testes do Rust passam | O compilador | os mutantes de cada item | — | — | 44 s |
| 6 | `cargo clippy` | Nenhum diagnóstico | O compilador | não medido | — | — | 0 s, aquecido |
| 7 | `verificar-deny.sh` | Licenças, avisos e fontes das dependências | O cargo-deny | não medido | — | Depende da rede: reprovou uma rodada sem ela | 1,5 s |
| 8 | `verificar-shell.sh` | ShellCheck nos scripts | Só os rastreados; 4 formas de shebang | 1 de 5 | — | Lista de exceção | 4,3 s |
| 9 | `verificar-appimage.sh` | Invariantes do AppImage | — | não medido | — | — | 0 s |
| 10 | `verificar-cpp.sh` | clang-format e clang-tidy do C++ | As ferramentas, só em `ui/src` | não medido; `ui/tests` fica fora | — | A exceção de falso positivo só vale no clang-tidy 18; a máquina tem o 23 | 39 s |
| 11 | `verificar-cpp-testes.sh` | Os testes C++ passam | O CTest | não medido | — | — | 0 s |
| 12 | `verificar-qml.sh` | qmllint sem aviso | O qmllint, com os tipos do build | 4 de 4 dos defeitos do passo 15 | 0 de 2 | — | 12 s |
| 13 | `check_identifier_language.py` | Identificador novo em português reprova | Leitor próprio de C-like, `ast` no Python, leitor de linha no shell; 7 extensões; palavra de 4 letras ou mais | 3 de 10 | 4 de 4 | Aceita abaixo e avisa; o comando recusa subir, mas recria a base apagada com a dívida nova | 2,2 s; 15 s com o autoteste |
| 14 | `verificar-qml-fiacao.sh` | Binding auto-referente `x: x` | Regex por linha; o bloco tem de abrir sozinho na linha | 2 de 6 | — | Sem base | 0,1 s |
| 15 | `verificar-qml-propriedades.sh` | Binding para propriedade que o componente não tem | Regex e recuo; tabela das propriedades do Qt mantida à mão | 3 de 6 | 2 de 2 | Sem base | 0,4 s |
| 16 | `verificar-qml-mortas.sh` | Função QML que ninguém menciona | Só `function` com 4 espaços, só `.qml`; menção por palavra no texto cru | 1 de 7 | — | Aceita abaixo e avisa; o comando sobe | 11,3 s |
| 17 | `verificar-qml-duplicacao.sh` | A mesma derivação em 2 arquivos ou mais | Regex `nome === "valor"` no texto cru; chave pelo nome do arquivo | 2 de 10 | 1 de 1 | Aceita abaixo sem avisar; o comando sobe, e a mensagem de falha o sugere | 0,2 s |
| 18 | `verificar-qml-tokens.sh` | Raio, fonte, duração e cor fora do `Theme` | 4 regex; tira só o comentário de linha | 5 de 14 | 1 de 1 | Aceita abaixo e avisa; o comando recusa subir, mas recria a base apagada | 0,2 s |
| 19 | `verificar-qml-alcance.sh` | Componente entregue que nenhuma tela alcança | Regex de instância no texto cru; trecho de nome no C++ | 1 de 6 | — | Lista de exceção | 11,0 s |
| 20 | `verificar-fiacao-ipc.sh` | Método, evento e sinal com dono nas duas pontas | Regex no texto cru de Rust, C++ e QML | 3 de 8 | 1 de 1 | 8 aceitos, sem conferir se ainda casam | 11,7 s |
| 21 | `verificar-exercitacao.sh` | O core real contra as ferramentas da máquina | Executa | não medido | — | — | 29 s |
| 22 | `verificar-embarcado.sh` | Depuração embarcada no QEMU | Executa | não medido | — | — | 1 s |
| 23 | `verificar-python-debug.sh` | MicroPython e depuração Python | Executa | NÃO PROVADO na máquina da medida, sem o debugpy | — | — | 3 s |
| 24 | `verificar-clangd-cross.sh` | O clangd acha os cabeçalhos do kit cross | Executa | não medido | — | — | 0 s |
| 25 | `verificar-atalhos.sh` | Atalho, paleta e menu coerentes, sem colisão | Regex; exige a linha `Shortcut {` | 2 de 5 | — | Sem base | 0,05 s |
| 26 | `verificar-docs.sh` | Contagem de linhas sem data igual à do arquivo | Regex perto do nome; varre o disco; tolera 2; a data vizinha isenta | 1 de 7 | 0 de 1 | Sem base | 2,6 s |
| 27 | `module_map.py --check` | Mapa igual ao código; nenhum ciclo novo | Regex `crate::x` no texto cru | 1 de 2 | 4 de 4 | 2 ciclos conhecidos, os dois falsos | 0,4 s |
| 28 | `verificar-links-docs.sh` | Link relativo com alvo; doc público sem nota local | Regex do link em linha; a privacidade só em `.md` | 4 de 9 | — | Sem base | 2,2 s |
| 29 | `verificar-arquitetura.sh` | Limite de linhas por categoria; a dívida só desce | Corta no primeiro texto `#[cfg(test)]`; isenta `tests.rs` e `tests/` pelo nome | 3 de 8 | — | Aceita abaixo sem avisar; o comando sobe, e grava a base vazia quando um arquivo não se lê | 0,08 s |
| 30 | `verificar-adiados-windows.sh` | Testes adiados e `cfg(unix)` só descem | Grep de `cfg_attr(` em várias linhas e de `#[cfg(unix)]` | 3 de 10 | 1 de 1 | Folga zero; sem comando | 0,06 s |
| 31 | `verificar-transicao-workspace.sh` | A troca do `workspace` tem um dono só | Regex `self.workspace` por linha; para na primeira linha com `#[cfg(test)]` | 1 de 7 | — | Sem base | 0,08 s |
| 32 | `verificar-qml-logica.sh` | Os harnesses da QML passam | Executa | não medido; com um filtro que não casa, dá verde com 0 testes | — | — | 9 s |
| 33 | `verificar-binario-abre.sh` (debug) | O binário abre até o primeiro frame, sem aviso | Executa | não medido | — | Lista de exceção | 38 s |
| 34 | `check_terminal_quiet.py` | Abrir pela linha de comando não imprime nada | Executa | não medido | — | — | 4 s |
| 35 | `cargo build --release` | O core compila em release | O compilador | não medido | — | — | 0 s, aquecido; 181 s frio |
| 36 | `cmake --build` (release) | A UI compila em release | O compilador | não medido | — | — | 0 s, aquecido; 255 s frio |
| 37 | `verificar-binario-abre.sh` (release) | O binário de release abre | Executa | não medido | — | Lista de exceção | 39 s |

Nenhum dos 16 passos medidos cumpre a meta de sensibilidade nas formas
difíceis, e 7 reprovam forma legítima. Só o passo 30 tem folga zero. O
determinismo é cumprido pelos 17 estáticos; o custo só é medido em segundos.

---

# Parte E — A execução

## E.1 A ordem

```text
Fase 0  dados            INF-15 → INF-17 → INF-20 → INF-21 → INF-18
        gate (D-19)      ENG-01 (a ferramenta) → ENG-40 → ENG-41 → INF-16 → ENG-42 → ENG-43 → ENG-44 → ENG-45 → ENG-46 → ENG-01 (o catálogo gate)
        dados e gate     ENG-02 → ENG-47 → ENG-03 → ENG-05 → ENG-06 → ENG-07
Fase 1  core             ENG-10 → ENG-11 → ENG-12 → ENG-13 → ENG-14 → ENG-15 → ENG-17
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
- Fazer, desde o D-19 e o D-21 (2026-11-13):
  - A entrada é uma lista de edições (troca de texto exato ou arquivo novo),
    porque o defeito de gate costuma pedir um arquivo novo e a linha que o
    entrega.
  - A entrada diz o veredito esperado, `reprova` ou `passa`, e o trecho da
    mensagem que a reprovação tem de trazer. Reprovar por outro motivo (a
    ferramenta que quebrou, o passo que não achou o `cargo`) conta como
    falha da rodada: na medida de 2026-11-13, uma rodada inteira do mapa
    "reprovou" porque o passo quebrou, não porque viu o defeito.
  - O que reprova pode ser um teste do `cargo`, um harness da QML, um teste
    C++ ou um passo do gate. Toda entrada roda: a rodada que pula uma entrada
    por falta de quem a rode reprova (a rodada do INF-18 deixou os mutantes
    da QML de fora até ser corrigida).
  - `--rodar` usa uma cópia por entrada e roda as entradas em paralelo, até o
    número de núcleos; a de passo estático copia só os arquivos rastreados.
    No fim imprime, por regra: plantados, reprovados, formas legítimas e as
    que passaram. Sai com erro se qualquer número não for o esperado.
  - `--conferir` reprova também o passo do orquestrador que não tem entrada
    no catálogo `gate` (R-GATE-03).
  - A seção de pendentes: os 10 mutantes registrados como equivalentes
    entram com o item que os mata (ENG-02 ou ENG-47). Enquanto o dono está
    aberto, sobreviver é o esperado; quando ele fecha, a entrada passa para o
    catálogo do dono e tem de reprovar. É catraca de migração: 10 → 0, e a
    seção é apagada. Nenhuma entrada nova nasce pendente.
  - O `--completo` roda `--rodar gate` inteiro. O `--rapido` roda só o
    `--conferir`.
- Commits: a ferramenta com o catálogo do INF-15, seu primeiro usuário, e a
  seção de pendentes; no fim do bloco do D-19, o catálogo `gate` (D.3) e o
  passo no `--completo`.
- Aceite: o estado do git do checkout é igual antes e depois de `--rodar`. O
  catálogo `gate` reprova 100% dos defeitos e deixa passar 100% das formas
  legítimas, e o tempo da rodada inteira fica registrado no 40.7 (a medida de
  partida: 135 sondas em série, com os leitores de hoje, 6 min 5 s).
- Mutantes da própria ferramenta: uma troca que não compila conta como
  falha; uma que sobrevive reprova a rodada; um texto que ocorre duas vezes
  reprova o `--conferir`; uma reprovação sem o trecho esperado reprova a
  rodada; uma forma legítima que reprova, também; um passo sem entrada
  reprova o `--conferir`.
- Pronto: os mutantes de cada item seguinte moram no catálogo. Os mutantes
  antigos, descritos no 40.7, migram na camada "Mutação das guardas" do
  [62 §3.5](62-base-publica-0.4.md). O autor roda `--rodar gate` e lê, por
  regra, quantos defeitos ela reprova.

**ENG-40 — a catraca comum** (D-19, D-20).
- Fazer: `scripts/ratchet.py`, o único que lê e compara linha de base. O
  nome é em inglês porque o módulo é importado, e o nome vira identificador.
  - As bases vão para `scripts/baselines/`, num formato só:
    `chave<TAB>contagem`, com um cabeçalho que diz a regra, a decisão da §0
    que criou a base e o item que a zera.
  - Cada verificador entrega só a medida, chave → contagem. O módulo compara
    chave por chave e reprova com a mesma mensagem em todos: a contagem
    subiu; a chave é nova; a contagem desceu (desça a linha); a chave sumiu
    (apague a linha).
  - Uma chave mora numa base só. A regra pode ter a base herdada e a de
    migração do D-22; o que não está em nenhuma é dívida nova e reprova.
  - Um comando só grava: `scripts/ratchet.py --lower <regra>`. Ele regrava
    com as contagens menores, recusa chave nova e contagem maior, e não cria
    base. Os `--atualizar-baseline` e o `--update-baseline` saem dos
    verificadores.
  - `scripts/ratchet.py --history`, no bloco estático: para cada base, do
    commit que a criou até o `HEAD`, nenhuma contagem subiu e nenhuma chave
    entrou; todo arquivo de `scripts/baselines/` tem o cabeçalho. Sem a
    história (clone raso), sai como NÃO PROVADO.
  - Base que falta, linha malformada e chave repetida reprovam.
- Commits: o módulo com o autoteste e a catraca dos adiados, a única que já
  reprova abaixo, como primeiro usuário; um por catraca migrada
  (arquitetura, mortas, duplicação, tokens, identificadores); o `--history`.
- Aceite: cada verificador diz na árvore o mesmo que dizia, salvo o texto da
  catraca; cada base migrada tem as chaves e as contagens da §B.2; nenhum
  verificador grava no checkout.
- Defeitos plantados, em cada uma das 6 catracas: a contagem de uma chave
  acima da base; abaixo da base; uma chave nova; uma chave da base que sumiu;
  o `--lower` com dívida nova na árvore (recusa e não grava); o `--lower`
  com a base apagada (recusa e não cria); a base apagada; uma linha
  malformada. No `--history`: um commit de teste, numa cópia, que sobe uma
  contagem; outro que acrescenta uma chave; uma base sem cabeçalho. Forma
  legítima: a contagem que desce junto com a linha passa.
- Pronto: R-GATE-09 e R-GATE-10 reprovam direto; nenhum script fora do
  `ratchet.py` lê nem grava linha de base.

**ENG-41 — o leitor de cada linguagem** (D-19, D-20).
- Fazer: o pacote `scripts/readers/`, com um leitor por linguagem e um dono
  só: Rust, QML e JavaScript, C++ e shell.
  - Todo leitor devolve as regiões do arquivo (código, comentário e texto
    literal) com a posição preservada: o verificador escolhe o que lê, e a
    mensagem cita a linha certa.
  - O de Rust devolve também a visão de produção, sem cada item marcado com
    `#[cfg(test)]`, tirado item a item, e sem o arquivo que o pai declara
    com `#[cfg(test)] mod`; e os caminhos de cada `use`, com os grupos
    abertos. O nome `tests.rs` não decide nada.
  - O de QML devolve também o esqueleto: cada bloco `Tipo { }` com o tipo, o
    `id`, as propriedades, os bindings, os sinais e as funções, em qualquer
    recuo, numa linha ou em várias.
  - O de shell é o leitor de linha de hoje, movido, com o here-doc.
  - O leitor devolve também as palavras das regiões de código, por arquivo:
    é o índice que o ENG-04 pedia, e sai de uma leitura só.
  - O Python segue lido pelo módulo `ast`.
  - O corpus mora em `scripts/readers/corpus/`: um arquivo por forma, com a
    leitura esperada ao lado, e roda no bloco estático.
  - O primeiro consumidor é o `check_identifier_language.py`: o leitor
    próprio dele sai, e o escopo passa a ser todo arquivo de código
    rastreado (entram `.c`, `.cmake`, `.zsh`, `.ps1`, `.in` e os scripts sem
    extensão).
  - Parser de compilador não entra: o passo estático roda antes de qualquer
    build e sem ferramenta externa (R-GATE-01).
- Commits: um por leitor, com o corpus dele; o idioma sobre os leitores, com
  a base herdada descendo e a catraca de migração do ENG-37 nascendo.
- Aceite: o corpus inteiro lido como o esperado. No idioma, os pares que o
  leitor antigo lia de comentário ou de texto literal saem da base herdada
  (8, pelo protótipo); os que ele escondia (8, em `remote/discover.rs`) e os
  dos arquivos que entram no escopo (11 pares em 9 arquivos `.c`) vão para a
  catraca de migração; a palavra inglesa que aparecer entra no vocabulário
  pelo `--accept`. O custo do passo fica registrado no 40.7 (hoje 2,2 s).
- Defeitos plantados no corpus, cada forma com a leitura esperada:
  - Rust: comentário de linha, de bloco e de bloco aninhado; string com
    quebra de linha; string crua com `#`, de bytes e de C; literal de
    caractere com aspa e o lifetime; `#[cfg(test)]` num item no meio do
    arquivo, num `mod tests;`, num `mod tests { }`, num comentário e numa
    string; `use crate::{a, b::{c}}`; macro com chaves.
  - QML e JavaScript: comentário de linha e de bloco; aspa simples e dupla;
    template com `${}` aninhado; regex com aspa; bloco de uma linha;
    `Tipo { // nota`; `prop: Tipo {`; os grupos `font { }` e `anchors { }`;
    binding de várias linhas; `;` no fim; recuo de 2 e de 8 espaços.
  - C++: string crua com delimitador; literal de caractere; separador de
    dígitos.
  - Shell: here-doc com e sem aspas; `read -r`; `local` sem atribuição;
    comentário depois de código.
- Defeitos plantados no idioma: identificador em português num `.rs`, depois
  de um comentário de bloco aninhado, num `.c`, num `.cmake`, num `.ps1`,
  num script `.in`, na variável de um `read -r` e num `local` sem
  atribuição. Formas legítimas, que hoje reprovam: português numa string
  Rust de várias linhas, dentro de um comentário de bloco aninhado, numa
  string QML de aspa simples e numa string depois de uma regex com aspa.
- Fora do item: a palavra de 3 letras. O verificador só lê palavra de 4 ou
  mais; com 3, a medida achou 313 palavras fora do vocabulário, quase todas
  palavra-chave. A triagem é do ENG-37.
- Pronto: R-GATE-11 no idioma; nenhum leitor próprio no
  `check_identifier_language.py`.

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
- Desde o D-19 (2026-11-13): o extrator lê pelo leitor de Rust do ENG-41, e
  o "corta no primeiro `#[cfg(test)]`" do Fazer passa a ser a visão de
  produção, item a item. O corte é o erro de leitura que escondeu dois
  arquivos da catraca de arquitetura (ENG-42): 25 arquivos do core têm
  produção depois da primeira marca. O aceite não muda: no 3952b8ad e no
  2e5fa164, as duas formas dão 213 arestas e 0 ciclos (medido em 2026-11-13).
  Defeitos plantados a mais: o import que fecha um ciclo depois de um item
  `#[cfg(test)]` no meio do arquivo reprova; o `crate::x` numa string e num
  `#[cfg(test)] mod` de dentro do arquivo não criam aresta.
- Pronto: R-ARQ-01 reprova qualquer ciclo.

**ENG-42 — os gates que leem Rust** (D-19, D-22).
- Fazer: passam a ler pelo leitor de Rust do ENG-41.
  - `verificar-arquitetura.sh`: conta as linhas de produção, item a item; o
    arquivo que o leitor não lê reprova. A contagem tem um dono só, que o
    `verificar-docs.sh` também usa.
  - `verificar-transicao-workspace.sh`: toda escrita no `workspace` e nos
    `drafts` do `Core`, em qualquer forma: atribuição numa linha ou em duas,
    `take`, `replace`, `insert`, `mem::take`, `mem::replace`, `mem::swap` e
    o `&mut` entregue; por `self` ou por outro nome, que é a forma da função
    livre do ENG-14.
  - `verificar-adiados-windows.sh`: toda forma de adiar um teste:
    `cfg_attr(…, ignore)` numa linha ou em várias, com ou sem fatia;
    `cfg(not(windows))`, `cfg(all(unix, …))`, `target_os` e `target_family`
    sobre um teste; e o retorno cedo por `cfg!(…)` dentro de um teste.
  - O lado Rust do `verificar_fiacao_ipc.py`: o braço de rota numa função
    livre; o nome de evento que só aparece em teste não conta.
- Commits: um por gate. O da arquitetura e o dos adiados criam a catraca de
  migração do D-22 com o que medirem.
- Aceite, na árvore: a transição acha as 4 escritas do dono
  (`handlers/workspace.rs`) e nenhuma outra; os adiados seguem em 76 e 50
  nas chaves de hoje; a base herdada da arquitetura segue com 1 arquivo. As
  catracas de migração nascem com a medida do item (pelo protótipo, em
  2026-11-13: `lib.rs` com 567 e `terminal/session.rs` com 661 linhas de
  produção; 3 retornos cedo por `cfg!(…)` em teste).
- Defeitos plantados:
  - Arquitetura: um arquivo novo acima do limite; o mesmo com as linhas
    depois de um comentário que cita `#[cfg(test)]`, depois de uma string
    que cita, e depois de um item de teste no meio; produção num arquivo
    chamado `tests.rs`; um arquivo em dívida que cresce uma linha; uma QML
    acima do limite; um arquivo ilegível na árvore.
  - Transição: a atribuição fora do dono; `mem::take`; por outro nome; numa
    função livre com `core: &mut Core`; depois de um comentário que cita
    `#[cfg(test)]`; `Option::insert`; a atribuição em duas linhas.
  - Adiados: `cfg_attr(windows, ignore)` sem fatia; numa linha só;
    `cfg(not(windows))`; `cfg(all(unix, test))`; `target_os`;
    `target_family`; o retorno cedo por `cfg!(windows)`; uma fatia
    desconhecida. Forma legítima: `#[cfg(unix)]` num comentário não conta.
  - Fiação: o braço de rota numa função livre, sem cliente. Forma legítima:
    o nome de evento só num arquivo de teste.
- Pronto: os quatro sem leitor próprio; as metas da §D.3 medidas no catálogo.

**ENG-43 — os gates que leem QML e C++** (D-19, D-22; absorve o ENG-04).
- Fazer: passam a ler pelo leitor de QML, com o esqueleto, e pelo de C++; a
  busca por nome usa o índice de palavras do leitor.
  - Funções mortas: a função em qualquer recuo e nos `.js`; a menção só
    conta em código. A chamada montada em texto (o `KvLocalWebPage.qml`
    monta QML numa string) é lida como código quando o texto é passado a
    `Qt.createQmlObject`.
  - Duplicação: a mesma comparação em qualquer forma (`===`, `!==`, `==`,
    `!=`, invertida, aspa simples, valor com maiúscula ou sublinhado, acesso
    por colchete, `indexOf`) e o `case "valor":`; a chave usa o caminho.
  - Tokens: a cor em qualquer forma (aspa simples, `Qt.rgba` com número,
    cor por nome, `property color`); a fonte no grupo `font { }`, o
    `pointSize` e a `font.family`; o raio entre parênteses e por atribuição.
    O `Theme.qml` fica fora inteiro: ele é o dono dos tokens, e hoje a
    definição `radius` dele está na base como dívida.
  - Alcance: o componente é alcançado a partir das raízes, não por ser
    citado; a instância só conta em código; o nome inteiro no C++; a linha
    do CMake com comentário.
  - Auto-referência: `x: x` em qualquer bloco do esqueleto.
  - Atalhos: o `Shortcut` em qualquer forma de bloco; o `case` só em código.
  - Fiação, lado QML e C++: cliente, ouvinte e tratador só em código; o
    tratador `onX` conta no bloco do tipo que declara o sinal ou num
    `Connections`; a entrada de aceito que não casa mais reprova (ENG-45).
  - Propriedades: decisão do autor antes de mexer (§G.4). A medida: o
    qmllint do passo 12 reprova as 4 formas plantadas e aceita as 2
    legítimas, e o verificador próprio reprova 3 de 6 e as 2 legítimas; mas
    o passo 12 não roda na `--ci-rapida`, que não tem Qt, e o verificador
    próprio roda.
- Commits: um por gate; cada um cria a catraca de migração do que revela.
- Aceite, na árvore: as bases herdadas iguais (12, 11 e 7, com a definição
  do `Theme.qml` saindo dos tokens); o alcance segue em 447 de 447; a
  fiação, com os mesmos totais. As catracas de migração nascem com a medida
  do item (§B.2). Custo: funções mortas, alcance e fiação abaixo de 1 s
  cada, contra 11,2 s, 10,8 s e 11,5 s, medido no 40.7.
- Defeitos plantados:
  - Funções mortas: a função sem uso; a de recuo 8; a citada só num
    comentário; só numa string; a função numa propriedade; num `.js`.
  - Duplicação: `!==`; invertida; `==`; `switch` e `case`; valor com
    sublinhado; aspa simples; `indexOf`; colchete; a que espalha para um
    terceiro arquivo. Forma legítima: a comparação num comentário.
  - Tokens: `font { pixelSize }`; cor com aspa simples; `Qt.rgba` com
    número; cor por nome; `property color`; `radius: (6)`; `pointSize`; a
    atribuição em JavaScript; `font.family`. Forma legítima: o literal num
    comentário de bloco.
  - Alcance: o componente que ninguém instancia; instanciado só num
    comentário; só numa string; o de nome que é trecho de outro no C++; a
    linha do CMake com comentário; dois componentes que só se instanciam um
    ao outro.
  - Auto-referência: com comentário no fim da linha; num bloco de uma
    linha; num bloco aberto por `prop: Tipo {`; num bloco aberto com
    comentário; com `;` no fim.
  - Atalhos: o item de menu cujo `case` só existe num comentário; a mesma
    sequência num `Shortcut { id: …` e num `Shortcut` de uma linha.
  - Fiação: o cliente só num comentário; o ouvinte só num comentário; o
    sinal sem tratador cujo nome outro componente trata (`clicked`).
- Limite da regra, dito: a função morta que tem o mesmo nome de uma função
  viva de outro objeto não é vista. Separar as duas pede o tipo do objeto,
  que a leitura do texto não tem (§H.3).
- Pronto: nenhum dos sete com leitor próprio; as metas da §D.3 medidas no
  catálogo.

**ENG-44 — a catraca do sinal da ponte** (D-19, D-20).
- Fazer: uma conferência no bloco estático: para cada arquivo de
  `scripts/qml-harness`, quantos sinais do `CoreClient` ele redeclara. Os
  sinais vêm do `ui/src/core_client.h`, pelo leitor de C++; as declarações,
  do esqueleto da QML. A catraca de migração, por arquivo, nasce com a
  medida.
- Commits: um.
- Aceite: 19 arquivos e 340 redeclarações (medido em 2026-11-13, no
  2e5fa164); o item mede de novo e grava o que medir.
- Defeitos plantados: um harness novo que redeclara um sinal da ponte; um
  harness da base que redeclara mais um; a redeclaração com espaços a mais.
  Formas legítimas: o sinal próprio do harness; o sinal da ponte citado num
  comentário.
- Pronto: a parte do sinal da R-TEST-06 tem gate; o ENG-23 leva a zero e
  apaga a catraca.

**ENG-45 — a falha fechada e a exceção conferida** (D-19, D-20).
- Fazer:
  - Todo passo reprova quando o leitor quebra ou um arquivo do alcance não
    se lê. Hoje a arquitetura passa: o leitor quebra, a lista sai vazia e o
    script segue.
  - O passo que espera itens reprova com o conjunto vazio: o
    `verificar-qml-logica.sh` com um filtro que não casa, os verificadores
    da QML com 0 arquivos, o shellcheck com 0 scripts. Todo passo diz no
    fim quantos itens leu.
  - Toda lista de exceção de gate tem folga zero: a entrada que não casa
    mais com nada reprova. Vale para os aceitos da fiação, para a exceção de
    falso positivo e a lista de versões do `verificar-cpp.sh`, e para as
    listas do `verificar-qml-alcance.sh`, do `verificar-shell.sh` e do
    `verificar-binario-abre.sh`.
  - A exceção que só vale numa versão de ferramenta sai como NÃO PROVADO
    fora dela: a do clang-tidy 18 nunca foi exercida na máquina da medida,
    que tem o 23, nem na `--ci-rapida`, que pula o passo.
- Commits: a falha do leitor; o conjunto vazio; as listas de exceção.
- Defeitos plantados: um arquivo ilegível no alcance de cada verificador
  que lê a árvore; o filtro que não casa com nada; a pasta dos harnesses
  vazia; uma entrada velha em cada lista de exceção.
- Pronto: R-GATE-12 reprova direto; a parte das listas da R-GATE-09.

**ENG-46 — os arquivos que nenhum gate lê** (D-19).
- Fazer:
  - O C++ de `ui/tests` entra no clang-format e no clang-tidy do
    `verificar-cpp.sh`. A formatação desses arquivos é a dívida do item (13
    erros em 2 dos 8, medido em 2026-11-13).
  - O `verificar-shell.sh` lê os scripts rastreados e os novos não
    ignorados, e reconhece o script pelo shebang em qualquer forma.
  - Os 2 `.pyc` saem do índice do git, e uma conferência reprova o arquivo
    rastreado que o `.gitignore` ignora.
  - O caminho do diretório pessoal não aparece em arquivo rastreado nenhum:
    a conferência de privacidade, que hoje lê só `.md`, passa a ler todo
    arquivo de texto rastreado para essa forma.
- Commits: um por parte.
- Defeitos plantados: um teste C++ mal formatado; um achado do clang-tidy
  em `ui/tests`; um script novo, não rastreado, com erro; um script sem
  extensão com `#!/usr/bin/bash` e outro com `#! /bin/sh`; um `.pyc`
  rastreado; o caminho pessoal num comentário de script.
- Pronto: cada defeito plantado reprova; `git ls-files` sem arquivo que o
  `.gitignore` ignora.

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
  lugar dele são recusados. Os 4 mutantes pendentes do INF-17 e do INF-20
  (o dono que grava direto e o que monta o caminho à mão) reprovam pela
  conferência de vocabulário e saem da seção de pendentes (D-21).

**ENG-47 — a leitura de conteúdo com dono** (D-21).
- Fazer: uma conferência no passo do mapa, pelo leitor de Rust: no código de
  produção do core, `fs::read`, `fs::read_to_string` e `File::open` só
  aparecem nos leitores guardados (`kinein_dir` e `fsops/project_file`) e
  num leitor de arquivo do sistema, na `platform`. Cada leitura direta de
  hoje passa para o leitor do tipo dela: do `.kinein`, do projeto e da
  ferramenta, ou do sistema. Entra como catraca de migração, por arquivo;
  em zero, vira regra direta.
- Commits: a conferência com a catraca; um por domínio migrado.
- Aceite: os testes iguais (R-TEST-07). A medida de partida, pelo protótipo
  de leitor, em 2026-11-13, no 2e5fa164: 19 leituras em 14 arquivos de
  produção (`coverage`, `datasource`, `platform`, `remote`, `serial`,
  `settings`, `setup`, `terminal`, `toolchain` e `workspace`). No texto cru,
  com os testes de dentro dos arquivos, o
  `git grep -cE 'fs::read(_to_string)?\(|File::open\('` em
  `crates/kinein-core/src`, sem `tests/` e `tests.rs`, soma 109.
- Mutantes: os 6 pendentes do INF-18 (os alvos do `CMakeLists.txt`, as
  bibliotecas ligadas, a CDB do contexto, a substituição, a gravação
  conferida e os rascunhos lendo direto) passam a reprovar pela conferência.
- Defeitos plantados: `fs::read_to_string` novo num domínio; `File::open`
  depois de um item `#[cfg(test)]` no meio do arquivo. Formas legítimas: a
  leitura direta num teste; a citação num comentário.
- Pronto: 19 → 0, e a catraca apagada; 10 → 0 mutantes pendentes, e a seção
  apagada.

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
- Desde o D-22 (2026-11-13), entram no item:
  - O orquestrador mede cada passo em milissegundos e lista todos no resumo,
    com o tempo de CPU ao lado do de parede. Hoje mede em segundos inteiros
    e lista os 5 maiores: 15 dos 37 passos saem com 0 s.
  - O `verificar-deny.sh` sem rede sai como NÃO PROVADO, e não como
    reprovado nem como verde: em 2026-11-13 ele reprovou uma rodada por não
    resolver o nome do servidor do banco de avisos e passou nas duas
    seguintes, na mesma árvore.
  - O qmllint roda em lotes paralelos (hoje 12,4 s num núcleo só), e o
    shellcheck, um arquivo por núcleo (4,3 s).
  - A espera do `verificar-binario-abre.sh` (38,5 s de parede com 0,6
    núcleo, duas vezes no `--completo`) é lida e registrada: o que é prova e
    o que é espera.
  - A conta de passos estáticos é medida de novo: o bloco do D-19 muda os 19
    de 2026-11-12.
  - Defeitos plantados a mais: o `deny` sem rede não dá verde nem reprova a
    árvore; um passo que leva mais que o registrado aparece no resumo.
- Pronto: a primeira falha estática em até 10 s (medida no 40.7).

**ENG-04 — o índice de palavras.** Absorvido pelo ENG-43 (D-22): o índice é
uma saída do leitor do ENG-41, e os três scripts trocam de leitor e de busca
no mesmo commit. A meta do item segue no ENG-43: cada um abaixo de 1 s.

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
- Desde o D-22 (2026-11-13), entra no item a sensibilidade dos dois gates de
  docs, medida no 2e5fa164: o `verificar-docs.sh` reprovou 1 de 7 defeitos, e
  o `verificar-links-docs.sh`, 4 de 9.
  - `verificar-docs.sh`: o número vale com o caminho ou só com o nome do
    arquivo; antes ou depois do nome; a data só isenta o número da mesma
    frase; a tolerância é zero; entram os arquivos `.py` e `.sh`. A conta de
    linhas é a do dono único do ENG-42.
  - `verificar-links-docs.sh`: o link por referência, a âncora dentro do
    arquivo e o link em HTML; a conferência de privacidade reprova também a
    menção a modelo e a sessão.
  - Defeitos plantados a mais: o número errado só com o nome do arquivo;
    antes do arquivo; perto de uma data que não é dele; errado por 2 linhas;
    de um script Python; o link morto por referência; a âncora que não
    existe; o link morto em HTML; a menção a modelo e a sessão. Forma
    legítima: o número certo com o caminho inteiro.
  - Fora do item, e sem gate: o número que não é contagem de linhas (pares,
    arestas, testes). Ele entra no documento como registro, com data, hash e
    comando (R-DOC-01), e o número vivo fica no arquivo e na saída do gate,
    como na §B.2. A conferência é a do checklist (§H.3).
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
  é item próprio (R-PROC-02). O `ci.yml` baixa a história inteira, porque o
  `ratchet.py --history` sai como NÃO PROVADO num clone raso e o `--estrito`
  reprova (ENG-40).
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
  [08](../contribuindo/08-convencoes-codigo-testes-commits.md) corrigida. O
  `lib.rs` sai da catraca de migração da arquitetura (D-22): em 2026-11-13,
  no 2e5fa164, ele tinha 567 linhas de produção contra o limite de 500, e a
  catraca contava 500 porque o `#[cfg(test)] mod tests;` está na linha 501.

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

**ENG-17 — o `terminal/session.rs` acima do limite** (D-22).
- Fazer: dividir por responsabilidade (R-ARQ-13). Em 2026-11-13, no
  2e5fa164, o arquivo tinha 661 linhas de produção contra o limite de 500, e
  a catraca contava 135, porque há um `#[cfg(test)]` num método na linha
  135. O `TerminalManager` junta o ciclo das sessões (abrir, lançar e
  fechar), a entrada (a escrita, o mouse e a roda) e a vista (redimensionar,
  rolar e limpar). O executor registra no 40.7 a responsabilidade que sai e
  a palavra que o arquivo deixa de nomear.
- Commits: um por responsabilidade que sai.
- Aceite: os testes do terminal iguais (R-TEST-07).
- Pronto: a chave sai da catraca de migração da arquitetura; com o `lib.rs`
  do ENG-14, a catraca é apagada.

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
  R-TEST-06 reprova direto. Pela catraca do ENG-44, que mede por arquivo:
  19 → 0 harnesses e 340 → 0 redeclarações (medido em 2026-11-13, no
  2e5fa164; os 36 são os sinais redeclarados em 3 ou mais harnesses, na
  mesma data); a catraca é apagada.

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
- Medida de 2026-11-13, no 2e5fa164: o `verificar-exercitacao.sh` leva 29,5 s
  de parede com 2,7 s de CPU, e os 9 `sleep` fixos dele somam 29 s. O passo
  entra no item, com a espera por condição no lugar de cada um.
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
- Desde o D-22 (2026-11-13): cada item paga também a catraca de migração
  que o ENG-43 criou para a regra dele (§B.2): a função citada só em texto
  literal; as derivações em outra forma e os `case` repetidos; as cores e a
  fonte em outra forma.
- Pronto: as três linhas de base e as três catracas de migração apagadas; os
  três gates sem catraca.

**ENG-36 — supressões e aceitos.**
- Fazer: cada `#[allow]`, `NOLINT` e `shellcheck disable` é removido ou fica
  com o motivo na linha; cada entrada de lista de aceitos é ligada, apagada
  ou fica com motivo e data. Entra a conferência do motivo.
- Desde o D-22 (2026-11-13): entram os `noqa` e os `type: ignore` do Python,
  que a primeira medida não contou (§B.2). A entrada de aceito que não casa
  mais já reprova pelo ENG-45; aqui fica o conteúdo de cada uma. Defeito
  plantado: a supressão sem motivo ao lado, em Rust, C++, shell e Python.
- Pronto: supressão sem motivo reprova.

**ENG-37 — os identificadores em português.**
- Fazer: renomear por `scripts/rename_identifiers.py` e pelo compilador, uma
  pasta por commit. Nome que vai ao fio ou a arquivo gravado não muda de
  forma: se mudar, é contrato primeiro (R-IPC-01) e parar para perguntar.
- Desde o D-22 (2026-11-13): o item paga também a catraca de migração do
  ENG-41 (os pares que o leitor antigo escondia e os dos arquivos que
  entraram no escopo) e faz a triagem da palavra de 3 letras: o vocabulário
  ganha as inglesas, e o verificador passa a ler palavra de 3 letras.
- Pronto: a linha de base e a catraca de migração apagadas; identificador em
  português reprova direto, também com 3 letras.

**ENG-38 — a auditoria do checklist de riscos.**
- Fazer: para cada um dos 10 riscos do
  [16](../arquitetura/16-checklist-de-riscos-ocultos.md), apontar o teste ou
  o gate que o cobre hoje. O que não tiver vira item próprio no 62 §3.4.
- Pronto: a tabela risco → verificação no próprio 16.

**ENG-39 — os adiados do Windows.** Na etapa 10, conforme o
[60](60-windows-e-edicao-especial.md) §3.3. Pronto: linha de base apagada, e
também a catraca de migração do ENG-42, com as formas de adiar que a
contagem de hoje não vê (D-22).

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
   sem decisão na §A.9 (A.0). Depois do ENG-40, a base é conferida pelo gate
   (R-GATE-10); a lista de aceitos e a supressão seguem neste item.
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
- uma catraca de migração precisar nascer fora das que a §B.2 lista, ou a
  medida de uma das do D-22 passar da que a §B.2 registra;
- um defeito plantado da lista de um item não puder reprovar, ou uma forma
  legítima não puder passar: o item não fecha com a meta da §D.3 abaixo do
  dito, e a entrada não sai do catálogo para o item fechar;
- o ENG-43 chegar ao passo de propriedades: aposentar o verificador próprio
  e ficar com o qmllint, que a `--ci-rapida` não roda, ou mantê-lo sobre o
  leitor para a CI;
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

Da revisão de 2026-11-13 (D-19):

- A sensibilidade dos outros 21 passos (§D.5, "não medido"), que executam
  ferramenta ou produto: só o qmllint recebeu defeito plantado, e foram 4. O
  catálogo `gate` do ENG-01 mede.
- A sensibilidade a defeito comum. As 135 sondas são formas difíceis,
  escolhidas para achar o que o leitor deixa passar: os 30% medidos não
  dizem quanto cada passo pega no dia a dia.
- As catracas de migração do D-22 foram medidas com um protótipo de leitor,
  fora do repositório. O número que vale é o que o item medir com o leitor
  do ENG-41.
- O custo de cada passo depois do endurecimento, e o da rodada inteira do
  catálogo `gate` em paralelo.
- A causa dos 38,5 s do `verificar-binario-abre.sh` (ENG-03 lê).
- Nada disto rodou no Ubuntu 24.04 nem no Windows.

O que fica sem gate, dito:

- A função QML morta com o mesmo nome de uma função viva de outro objeto
  (ENG-43): a leitura do texto não tem o tipo do objeto.
- O número de documento que não é contagem de linhas (ENG-06): entra como
  registro datado, e a conferência é a do checklist.
- A criação de uma linha de base: o gate confere o cabeçalho e que ela só
  desce depois de criada (R-GATE-10); que ela devia nascer, quem confere é o
  autor, ao ler o commit.

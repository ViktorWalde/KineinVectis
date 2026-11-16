# 66 — Os gates antes do código da estratégia B

> **Classe: PLANO** (`DocsPublic/README.md`), com medidas datadas. **É uma
> proposta e aguarda a decisão do autor.** Decisão do autor de 2026-11-15:
> nenhuma fase de código da estratégia B (do B1 em diante,
> [64](64-revisao-da-arquitetura-e-estrategia-b.md) §8) começa sobre gate
> frouxo. Este documento responde à pergunta que a decisão abre: o que
> fecha antes do B1 sem fazer duas vezes o que o B4 e o B6 trocam pelo
> compilador e por artefato gerado. Ele mede, classifica cada regra e
> propõe a ordem. Não muda a Parte A do
> [63](63-regras-e-plano-de-engenharia-da-0.4.md) nem a ordem do 64 §8: as
> duas só mudam por decisão do autor (63 §H.2 e DP-04 do 64 §11). As
> perguntas estão no §11.
>
> **Ponto de partida.** Commit `cbb1f649`, protocolo 0.223.0. Toda medida é
> de 2026-11-15, numa cópia da árvore fora do checkout, na máquina de
> medição do projeto (16 núcleos; Rust 1.96.1, Qt 6.12.0, clang 23), salvo
> data própria. "Lido" e "medido" são marcados separados. Nenhum código,
> teste, linha de base ou gate mudou.

## 0. Como ler

O §1 é a resposta curta. O §2 traz as medidas. O §3 define as classes; o
§4 e o §5 são as tabelas, por regra da Parte A e por passo do
`scripts/verificar.sh`. O §6 é o desenho proposto da ferramenta do gate. O
§7 diz o que o B1, o B2 e o B3 precisam, e o §8, a CI. O §9 é a ordem
proposta, com aceite, evidência e reversão. O §10 é o risco do intervalo.
O §11 são as perguntas ao autor, e o §12, o que não foi medido.

Os donos não mudam. A regra mora na Parte A do 63. O que fazer em cada
item do gate, com os defeitos plantados, mora no 63 §E.2. O destino de cada
item pela estratégia B mora no 64 §7, e a dívida, no
[62 §3.4](62-base-publica-0.4.md). Este documento aponta para eles e
acrescenta três coisas: a medida de 2026-11-15, a classe de cada regra e a
ordem proposta.

## 1. A pergunta e a resposta

**A pergunta.** O 64 §8 adiou o ENG-40 ao ENG-44 "para não fazer duas
vezes", porque o B4 e o B6 trocam parte deles. Com o gate antes do B1: o
que fecha agora, o que espera o compilador ou o artefato gerado, e o que
guarda cada regra até lá?

**A resposta, em sete pontos.**

1. **Os verificadores são os de 2026-11-13.** Nenhum leitor mudou desde o
   `2e5fa164`. As 135 sondas rodaram de novo: 36 dos 120 defeitos reprovam
   e 14 das 15 formas legítimas são reprovadas por engano, sonda por sonda
   como antes (§2.1).
2. **As gramáticas tree-sitter leem o repositório, menos o C++.** O
   `tree-sitter-qmljs` lê os 447 arquivos de `ui/qml` sem nenhum nó de
   erro, que é a prova pedida pelo 64 §9. Shell, Rust e Python também
   saem sem erro. O C++ não: 20 dos 87 arquivos têm nó de erro, quase
   todos nas macros do Qt (§2.2). O que os gates leem no C++ da ponte vai
   para o artefato gerado, e não para o gate de texto (§6).
3. **Os cinco testes do INF-24 e o INF-22 têm causa medida, e nenhuma se
   corrige com prazo maior** (§2.3). Duas causas estão no produto: a
   releitura do contexto de compilador não tem ordem, e uma carga antiga
   cai por cima da nova (INF-22); a falha de `exec` numa sessão de
   terminal aborta o processo filho, e o sistema grava um core dump antes
   de a sessão fechar (INF-29). Duas estão no teste: o prazo de produção
   fixo no caminho do handler (INF-06) e a espera por um evento que três
   produtores emitem.
4. **Um achado novo de gate: o passo dinâmico exercita o binário errado.**
   Com o alvo do cargo fora da árvore, como no comando do gate, o
   `verificar-exercitacao.sh` compila o core num lugar e exercita o
   `target/debug/kinein-core` da árvore. Com um core plantado que não
   sobe, o passo deu verde (INF-31, §2.4).
5. **A classificação** (§4): das 55 regras da Parte A, 10 fecham agora
   num gate de texto pela ferramenta; 13 vão ao compilador, a artefato
   gerado ou a teste de contrato numa fase da B; 14 são independentes da
   arquitetura; 6 são teste do próprio item; 12 são checklist.
6. **Antes do B1, a proposta é** (§9): a suíte confiável; o teto de
   memória da ferramenta de mutação; a ferramenta do gate com a catraca
   comum e os consumidores de Rust, de QML e de shell; o gate que falha
   fechado (ENG-45), os arquivos que nenhum gate lê (ENG-46), o
   orquestrador (ENG-03, com o INF-31), os gates de docs (ENG-06), a parte
   da CI que é do repositório (ENG-07) e o catálogo `gate`.
7. **Não se faz na forma antiga, nem antes do B1:** o ENG-44 (vai ao dublê
   gerado, B4), o ENG-47 e o vocabulário do ENG-02 (vão ao `forbid`, B7),
   as contagens do ENG-12, do ENG-14 e do ENG-15 (vão ao grafo de crates e
   à privacidade, B6). O intervalo fica com risco explícito, e a lista do
   compilador na fase é exata (§10).

## 2. O que foi medido em 2026-11-15

### 2.1 Os verificadores não mudaram desde a revisão de 2026-11-13

`git diff --stat 2e5fa164..cbb1f649 -- scripts crates ui` lista 9 arquivos,
todos em `scripts/`: a ferramenta de mutação, o teste dela e os três
catálogos; duas listas de palavras do idioma; duas linhas no
`test_verification_profiles.py`; e cinco linhas no `verificar.sh`, o passo
`mutantes.py --conferir`. Nenhum verificador mudou, e nada mudou em
`crates/` nem em `ui/`.

As 135 sondas do 63 §D.5 rodaram de novo na árvore do `cbb1f649`, uma por
vez, cada uma aplicada sozinha numa cópia: 36 dos 120 defeitos reprovaram,
e 14 das 15 formas legítimas foram reprovadas por engano. O veredito de
cada sonda é igual ao de 2026-11-13. A rodada levou 8 min 3 s.

| Passo (63 §D.5) | Defeitos reprovados | Legítimas reprovadas |
| --- | --- | --- |
| 8, `verificar-shell.sh` | 1 de 5 | — |
| 13, `check_identifier_language.py` | 3 de 10 | 4 de 4 |
| 14, `verificar-qml-fiacao.sh` | 2 de 6 | — |
| 15, `verificar-qml-propriedades.sh` | 3 de 6 | 2 de 2 |
| 16, `verificar-qml-mortas.sh` | 1 de 7 | — |
| 17, `verificar-qml-duplicacao.sh` | 2 de 10 | 1 de 1 |
| 18, `verificar-qml-tokens.sh` | 5 de 14 | 1 de 1 |
| 19, `verificar-qml-alcance.sh` | 1 de 6 | — |
| 20, `verificar-fiacao-ipc.sh` | 3 de 8 | 1 de 1 |
| 25, `verificar-atalhos.sh` | 2 de 5 | — |
| 26, `verificar-docs.sh` | 1 de 7 | 0 de 1 |
| 27, `module_map.py --check` | 1 de 2 | 4 de 4 |
| 28, `verificar-links-docs.sh` | 4 de 9 | — |
| 29, `verificar-arquitetura.sh` | 3 de 8 | — |
| 30, `verificar-adiados-windows.sh` | 3 de 10 | 1 de 1 |
| 31, `verificar-transicao-workspace.sh` | 1 de 7 | — |

A folga das bases, os comandos que as sobem e o determinismo são os do 63
§D.5, medidos em 2026-11-13 sobre os mesmos arquivos; não foram medidos de
novo.

### 2.2 As gramáticas tree-sitter leem o repositório (a prova do 64 §9)

O 64 §9 pede uma prova antes de trocar o ENG-41: "o `tree-sitter-qmljs`
lê os 447 arquivos QML sem erro de análise?". Lê. A medida é de um
protótipo fora do repositório: um binário Rust com o `tree-sitter` 0.26.11
do workspace, que analisa cada arquivo rastreado e conta os nós de erro e
os nós faltantes da árvore.

| O quê | Gramática | Arquivos | Com nó de erro | Análise |
| --- | --- | --- | --- | --- |
| QML de `ui/qml` | `tree-sitter-qmljs` 0.3.1 (MIT) | 447 | 0 | 267,9 ms |
| As outras QML rastreadas (harnesses e fixtures) | a mesma | 200 | 0 | 117,7 ms |
| JavaScript da UI | `tree-sitter-javascript` 0.25.0 (MIT) | 4 | 4, um nó cada, na linha 1 | — |
| Shell | `tree-sitter-bash` 0.25.1 (MIT) | 49: 48 `.sh` e 1 sem extensão | 0 | 40,1 ms |
| Rust de `crates/` | `tree-sitter-rust` 0.24.2, já no workspace | 610 | 0 | 742,5 ms |
| Python | `tree-sitter-python` 0.25.0, já no workspace | 69 | 0 | 88,8 ms |
| C++ | `tree-sitter-cpp` 0.23.4, já no workspace | 87 | 20, com 301 nós de erro e 71 faltantes | 245,4 ms |
| C | `tree-sitter-c` 0.24.2, já no workspace | 9 | 1, o `typeof` do C23 numa fixture | — |
| Markdown | `tree-sitter-md` 0.5.4 (MIT) | 228 | 0 | 2742,7 ms |

O que a medida mostra além da contagem:

- **JavaScript.** O nó de erro dos 4 arquivos `.js` é a diretiva
  `.pragma library` da linha 1, que é do QML e não do JavaScript. Com essa
  linha em branco, os 4 saem sem erro. Lidos pela gramática de QML, os
  mesmos 4 dão 93 nós de erro: a gramática certa para eles é a de
  JavaScript, com a diretiva tratada.
- **C++.** Os erros estão nas macros do Qt (`Q_PROPERTY(… READ … NOTIFY
  …)`, `slots:`, `Q_UNUSED(x)` sem ponto e vírgula) e em 3 arquivos em
  outras formas. O `ui/src/core_client.h` sozinho tem 253 nós de erro. A
  gramática não tem pré-processador, e o cabeçalho da ponte é o arquivo
  que os gates mais precisam ler.
- **Linhas de produção, item a item.** Pela gramática do Rust, tirando
  cada item marcado com `#[cfg(test)]`: o `crates/kinein-core/src/lib.rs`
  tem 554 linhas de produção, e o
  `crates/kinein-core/src/terminal/session.rs`, 657 (medido em
  2026-11-15). O protótipo de 2026-11-13 dava 567 e 661; a diferença são
  as linhas dos atributos. Os dois continuam acima do limite de 500 e
  fora da base.
- **As formas difíceis.** Um corpus de cerca de 30 formas foi lido como o
  esperado, sem nó de erro: comentário de linha, de bloco e aninhado;
  string de aspa simples, de várias linhas, crua e de bytes; regex com
  aspa; template aninhado; bloco de uma linha; `prop: Tipo {`; o grupo
  `font { }`; o literal de caractere `'"'`; o lifetime; o `use` agrupado;
  o `#[cfg(test)]` item a item; o here-doc com e sem aspas; o `read -r` e
  o `local`.
- **Markdown.** A gramática aceita qualquer texto, e zero nós de erro não
  prova a leitura. Ela separa 2161 links em linha, 93 por referência, 96
  curtos, 74 definições de referência, 8 autolinks, 34.005 trechos de
  código e 3279 blocos de código. O tempo é o de uma passada ingênua, uma
  análise por parágrafo.
- **Custo.** O build frio do protótipo, com o download das gramáticas,
  levou 16,8 s. A análise de todos os arquivos de Rust, QML, shell e
  Python soma cerca de 1,3 s num núcleo só.

`tree-sitter-qmljs` 0.3.1 tem ABI 14, e o `tree-sitter` 0.26.11 aceita da
13 à 15. As gramáticas novas são MIT, licença que o `deny.toml` já aceita.

### 2.3 A suíte do core: os cinco testes do INF-24 e o INF-22

A suíte é `cargo test -q -p kinein-core --lib --all-features --locked`,
1566 testes. Em 2026-11-14 duas de cinco rodadas ficaram vermelhas. Em
2026-11-15 foram 25 rodadas, todas verdes: 5 sem instrumentação e 10
instrumentadas, com outra carga de cerca de 2 dos 16 núcleos na máquina
(48,0 s a 49,0 s cada); e 10 instrumentadas com a máquina livre (40,5 s a
43,0 s). A intermitência não se reproduziu em série nesse dia. A causa de
cada teste saiu de outra forma: a medida do mecanismo dentro da suíte, e a
dose e a resposta num parâmetro, numa cópia instrumentada. A
instrumentação só imprime e lê variáveis de ambiente; sem elas, o
comportamento é o do `cbb1f649`.

| Teste | O que se vê quando falha | Causa medida | De quem é |
| --- | --- | --- | --- |
| `c1_existing_cancelled_configure_retains_obsolete_state_until_successful_retry` (INF-22) | o `staleBecause` sem o `release local` | uma releitura mais velha do contexto troca depois da mais nova | produto |
| `the_compiledb_action_runs_pio_and_hands_the_new_cdb_to_the_index` | o `index.context` sem a CDB nova | o teste espera o primeiro `event.index.finished`, e o primeiro vem do watcher | teste |
| `c_host_keeps_build_lsp_context_and_verification_on_the_same_configuration` e `c_standard_drives_gcc_clang_cdb_context_and_tests_without_changing_other_targets` | `notVerified` com `obsolete: true` | o prazo de 5 s da verificação é fixo no handler, e a suíte gasta até 3,2 s dele | teste (INF-06) |
| `a_failed_program_releases_the_port_after_the_terminal_reports_its_exit` | a sessão não fecha em 4 s | o filho aborta na falha do `exec`, e a sessão só fecha depois do core dump | produto (INF-29) e ambiente do teste |

**O INF-22: a carga antiga cai por cima da nova.** Cada
`reload_index_context` (`crates/kinein-core/src/handlers/index.rs`) cria
um job que carrega o contexto e o troca no índice. A troca só confere a
raiz do workspace: não há ordem entre os jobs. No começo do configure de
`release local`, duas releituras nascem com poucos milissegundos de
diferença. Pelo que cada uma devolve, a mais velha carrega o contexto com
a CDB de `debug local`, e a mais nova, o contexto sem CDB, que é o certo.

- 32 cópias do teste ao mesmo tempo, 60 rodadas: 24 falhas em 1920
  execuções. Em todas, a primeira leitura respondeu "a origem ou o
  conteúdo da CDB mudou; recarregue/reconfigure", que é a resposta do
  contexto antigo, e de 1 ms a 18 ms depois, com os eventos pendentes
  observados, respondeu "configure não concluído (preset release local)".
- O mesmo lote de novo, registrando a ordem das trocas: 11 falhas em
  1920. Em 11 de 11, a 4ª releitura do processo trocou depois da 5ª.
- A sequência forçada: com a 4ª releitura segura 100 ms ou 140 ms entre
  carregar e trocar, 10 falhas em 10, com a mesma resposta. Atrasar todas
  as releituras por igual (300 ms ou 3 s) não reproduz: 0 em 5 cada.
- 16 cópias ao mesmo tempo: 0 em 192 neste dia (2 em 192 em 2026-11-14).

No teste, a releitura do fim do configure conserta o estado em
milissegundos. No produto, quando a carga antiga é a última a trocar, o
contexto velho fica até o próximo evento que dispara uma releitura.

**O teste do PlatformIO: o evento não diz de quem é.** O
`event.index.finished` sai de três produtores: a indexação inicial, a
releitura do contexto e o incremento do watcher (`reindex_changed_paths`),
que não relê o contexto. O evento não traz a causa. O teste espera o
primeiro depois do job.

- 16 cópias ao mesmo tempo, 20 rodadas: 6 falhas em 320. Nas 6, o primeiro
  `event.index.finished` veio depois de um `event.fs.changed`, e o
  contexto estava certo de 2 ms a 3 ms depois.
- A sequência forçada: com a releitura 300 ms mais lenta, 10 falhas em 10,
  com a mesma ordem de eventos; sem o atraso, 0 em 10.

O produto fica certo sozinho; o teste lê cedo.

**Os dois testes da verificação de edição: o prazo fixo.** O prazo de 5 s
é uma constante em `crates/kinein-core/src/handlers/lsp/verification.rs`,
e o teste não o recebe como parâmetro. É a forma do INF-06, que o D-12 do
63 já decidiu, num caminho que o INF-06 não cita.

- Cada verificação leva cerca de 1,2 s com o teste sozinho. Dentro da
  suíte, em 560 verificações de 10 rodadas com a outra carga: mediana de
  1362 ms, máximo de 3227 ms. Com a máquina livre, máximo de 2388 ms.
  Com 16 cópias do mesmo teste: máximo de 3421 ms.
- Dose e resposta, com o prazo por variável: a 300 ms e a 1000 ms, 3
  falhas em 3, com o `notVerified` e o `obsolete: true` do INF-24; a
  1500 ms, 1 em 3; a 2500 ms, 0 em 3.

Dois defeitos do produto apareceram nessa medida e são o INF-30:

- O prazo estourado sai como obsoleto. Em 4 verificações a 1000 ms e a
  1500 ms, a comparação tinha terminado, a conferência final do contexto
  passou do prazo, e a resposta virou `notVerified` com `obsolete: true`.
  Lido no handler: com `obsolete`, o texto que a pessoa vê é "Edição ou
  contexto obsoleto; consulte as ações novamente.". Nada estava obsoleto.
- A verificação só conclui quando as duas compilações, a de antes e a de
  depois, caem no mesmo segundo do relógio; senão, repete o par. Com o
  compilador 600 ms mais lento por chamada (o par leva 1,36 s), ela nunca
  conclui: com o prazo de 5 s, `notVerified` em 5,4 s; com 30 s, 18 pares
  repetidos e `notVerified` em 30,5 s. Com 250 ms a mais por chamada, 7
  de 10 verificações passaram de 5 s. Uma unidade de `ui/src` leva de 2,4 s
  a 3,6 s no `clang++ -fsyntax-only` com as flags do build (4 unidades, 5
  vezes cada): nenhuma unidade C++ do próprio produto se verifica.

**O teste do serial: o core dump.** O teste roda um programa cujo
interpretador não existe. O `exec` falha, e o filho aborta com "fatal
runtime error: assertion failed: output.write(&bytes).is_ok(), aborting".
Lido na dependência: o `portable-pty` 0.9.0 fecha todos os descritores
acima do 2 antes do `exec`, inclusive o canal por onde a biblioteca
padrão do Rust avisaria o pai da falha, e a biblioteca aborta. O sistema
grava então o core dump do filho, que é uma cópia do processo de teste, e
a sessão só fecha depois.

- Dentro da suíte, com a outra carga: a sessão fechou entre 3079 ms e
  3534 ms, em 10 rodadas. O prazo do teste é 4000 ms.
- Dentro da suíte, com a máquina livre: de 2058 ms a 2120 ms com o core
  dump, e de 505 ms a 564 ms com o limite de core do processo em zero (5
  rodadas cada).
- Sozinho: de 1011 ms a 1162 ms com o core dump, 535 ms sem.
- Cada rodada da suíte deixa um core dump no registro do sistema de quem
  a roda, de 21 MiB a 26 MiB compactado. Eram 118 registros do binário de
  teste desde 2026-11-10, contados antes destas medidas. As medidas desta
  revisão deixaram outros 135; os 131 arquivos que o sistema guarda somam
  2,83 GiB, e a política dele os apaga em 3 dias.

### 2.4 O binário que o passo dinâmico exercita (INF-31)

O `scripts/verificar-exercitacao.sh` roda `cargo build -q -p kinein-core`
e depois usa `target/debug/kinein-core`, relativo à árvore. O comentário
do script diz o motivo do build: "um core ANTIGO provaria o codigo de
ontem". Com `CARGO_TARGET_DIR` apontando para fora, o cargo grava em
outro lugar, e o passo exercita o binário que estiver na árvore.

Medido numa cópia, com um core plantado que sai com o código 3 quando
sobe sem argumento:

| Como o passo rodou | Saída | Binário exercitado |
| --- | --- | --- |
| `CARGO_TARGET_DIR` fora da árvore | 0, "os endpoints com ferramenta externa responderam" | o da árvore, de 2026-11-13, sem o defeito |
| sem a variável | 1, "uma ferramenta real recusou o que o core pediu" | o recém-compilado, com o defeito |

Lido, não medido: o `verificar_embarcado.py`, o
`verificar_python_debug.py` e o `verificar_micropython_porta.py` usam o
mesmo caminho fixo, e a UI que o `verificar_binario_abre.py` abre procura
o core em `target/debug/` a partir da raiz. O comando do gate rápido usa
o alvo do cargo fora da árvore, para não misturar o build do gate com o
de quem desenvolve: é a combinação que falha. Na árvore do `cbb1f649` o
defeito não mudou veredito, porque `crates/` não muda desde 2026-11-13.

### 2.5 O gate rápido, sozinho

`bash scripts/verificar.sh --rapido`, sozinho, no checkout limpo do
`cbb1f649`, com o ambiente isolado e o alvo do cargo fora da árvore: verde
na primeira rodada, em 220,6 s, com 1 NÃO PROVADO (o ciclo de depurar
Python, sem o debugpy na máquina). O resumo traz segundos inteiros e só
as cinco etapas mais longas: `cargo test --workspace` 54 s,
`verificar-cpp.sh` 45 s, `verificar-exercitacao.sh` 30 s,
`check_identifier_language.py` 15 s e `verificar-qml.sh` 12 s. Em
2026-11-13 a rodada aquecida levou 201 s.

### 2.6 A CI

Lido: o `.github/workflows/ci.yml` roda `bash scripts/verificar.sh
--ci-rapida --estrito` num `ubuntu-24.04`, instala os pacotes e o
`cargo-deny` 0.20.2 no próprio YAML e faz o checkout com a profundidade
padrão, de um commit. Não existe `packaging/ci/` nem `scripts/ci-local.sh`.
A máquina de medição tem o `podman` 6.2.0 e nenhuma imagem do Ubuntu.
Nada da CI foi executado nesta revisão (§12).

## 3. As classes

| Classe | O que quer dizer |
| --- | --- |
| **a** | Fecha agora, num gate de texto pela ferramenta do gate, lendo pela gramática tree-sitter (R-GATE-11). Fica como gate de texto depois da B |
| **b** | Vai ao compilador, a artefato gerado ou a teste de contrato numa fase da B (R-GATE-13). A tabela diz a fase e o que guarda a regra até lá |
| **c** | Independente da arquitetura: ENG-01, ENG-03, ENG-05, ENG-06, ENG-07, ENG-45, ENG-46 e INF-16 |
| **T** | Teste do próprio item, com mutantes (R-TEST-02). Não é regra de gate de texto |
| **L** | Checklist do desenho ou do commit (63 §G.3) |

As metas são as do 63 §D.3, por regra de gate: sensibilidade (95% nas
formas difíceis; 100% no catálogo `gate`), precisão (95%; toda forma
legítima passa), folga zero, base que nenhum comando cria nem sobe,
determinismo e custo em milissegundos no resumo da rodada. Nas tabelas,
"não atende" traz a medida; "—" quer dizer que a meta não se aplica; "não
medido" quer dizer que nenhum defeito foi plantado no passo.

**O princípio proposto para o intervalo.** A regra que vai ao compilador
ou a artefato gerado não ganha catraca de texto provisória. A dívida que
entrar no intervalo aparece inteira na lista de erros do compilador na
fase que a impõe; o custo é o trabalho a mais, linear, e nada fica
escondido. Ganham guarda de texto no intervalo só três casos: o gate que
já existe e que a ferramenta corrige com uma regra curta sobre o leitor
que fica; a regra cuja fase não enumera sozinha a dívida; e a regra que o
B1 ao B3 vão exercitar todo dia (§10).

## 4. A tabela por regra da Parte A

As colunas das metas: **S** sensibilidade, **P** precisão, **F** folga,
**B** base e comando, **D** determinismo, **C** custo. As medidas são as do
§2.1 e as do 63 §D.5. A meta do custo não é atendida por nenhum passo: o
orquestrador mede em segundos inteiros e lista só os cinco maiores (§5).
O determinismo dos passos estáticos foi medido em 2026-11-13.

### 4.1 Arquitetura

| Regra | Conferida hoje por | Classe | S | P | F | B | D | Destino, e o que a guarda até lá |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| R-ARQ-01, sem ciclo | passo 27, `module_map.py --check` | a, c (INF-16) | não atende: 1 de 2 | não atende: 4 de 4 legítimas reprovadas | não atende: 2 ciclos conhecidos que não existem | — | atende | Fecha agora: o extrator lê pela gramática, e a lista de ciclos é apagada |
| R-ARQ-02, a base só depende da base | nada; checklist | b | — | — | — | — | — | O grafo de crates, no B6. Até lá, risco explícito (§10) |
| R-ARQ-03, ninguém depende da borda | nada; checklist | a | — | — | — | — | — | Regra nova sobre o extrator do INF-16; a dívida que ela medir vai à catraca de migração do ENG-12, zerada no B6 |
| R-ARQ-04, só a borda nomeia o envelope | nada; checklist | a | — | — | — | — | — | Como a R-ARQ-03: vocabulário sobre o leitor de Rust |
| R-ARQ-05, rota por tabela | nada; checklist | b | — | — | — | — | — | O teste do registro, no B3. Até lá, a catraca de arquitetura corrigida segura o `lib.rs`, que só desce |
| R-ARQ-06, reação registrada pelo domínio | nada; checklist | b | — | — | — | — | — | O teste de extensão, no B6. Até lá, risco explícito |
| R-ARQ-07, `Core` fora da raiz | nada; checklist | b | — | — | — | — | — | A privacidade do compilador, no B6. Até lá, risco explícito, no lugar da contagem que a regra cita (pergunta 4) |
| R-ARQ-08, estado por domínio | nada; checklist | b | — | — | — | — | — | Como a R-ARQ-07 |
| R-ARQ-09, a ponte por domínio | nada; checklist | b | — | — | — | — | — | A estrutura do B5. Até lá, risco explícito |
| R-ARQ-10, QML de área não acessa a ponte | nada; checklist | b | — | — | — | — | — | A estrutura do B5. Até lá, a catraca de migração do ENG-22 sobre o leitor de QML (pergunta 3) |
| R-ARQ-11, motivo da faixa numa lista | nada | T | — | — | — | — | — | O teste de extensão do UI-02, no B11 |
| R-ARQ-12, a regra do core não se repete | checklist | L | — | — | — | — | — | — |
| R-ARQ-13, o limite de linhas | passo 29, `verificar-arquitetura.sh`; a mensagem, checklist | a | não atende: 3 de 8 | não medido | não atende: aceita abaixo sem avisar | não atende: o comando sobe, e grava a base vazia com um arquivo ilegível | atende | Fecha agora: linhas de produção item a item; dois arquivos entram em catraca de migração (§2.2) |
| R-ARQ-14, classe de execução | nada | b | — | — | — | — | — | A guarda do laço, no B1, e o teste do registro, no B3 |
| R-ARQ-15, só o lançador cria processo | nada | b | — | — | — | — | — | O `forbid` do clippy, no B2. O intervalo é só o B1, que não cria processo |
| R-ARQ-16, a confiança no core | nada | b, T | — | — | — | — | — | O teste do projeto hostil e os mutantes da guarda, no B2. O INF-23 segue aberto, crítico, até lá |
| R-ARQ-17, registro estruturado | nada | T | — | — | — | — | — | Os testes do B1 |

Do passo 31, `verificar-transicao-workspace.sh`, não há regra na Parte A:
ele guarda "a troca do `workspace` tem um dono só", que a privacidade da
R-ARQ-07 passa a impor no B6. Classe b. Mede 1 de 7. Até o B6, a proposta
é corrigi-lo sobre o leitor de Rust, com as formas do 63 §E.2, e apagá-lo
no B6.

### 4.2 Dados da pessoa

| Regra | Conferida hoje por | Classe | S | P | F | B | D | Destino, e o que a guarda até lá |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| R-DADOS-01, tudo pelo `kinein_dir` | os testes do INF-13 e do INF-15 | b | não atende: 4 mutantes pendentes sobrevivem | — | — | — | — | O `forbid` de `std::fs` e o catálogo, no B7. Até lá, risco explícito: os 4 pendentes ficam no catálogo de pendentes |
| R-DADOS-02, não segue link nem bloqueia | os testes de adversidade do INF-15, com 24 mutantes | T | — | — | — | — | — | — |
| R-DADOS-03, não grava por cima do que não entende | os testes do INF-07, do INF-11 e do INF-12 | T | — | — | — | — | — | O teste do catálogo, no B7 |
| R-DADOS-04, lê como o sistema de build | os testes do INF-18 | b, T | não atende: 6 mutantes pendentes sobrevivem | — | — | — | — | O `forbid` de `std::fs::read*` e de `File::open`, no B7. Até lá, risco explícito |
| R-DADOS-05, a classe de armazenamento | nada | T | — | — | — | — | — | O teste do catálogo, no B7 |

### 4.3 IPC e contrato

| Regra | Conferida hoje por | Classe | S | P | F | B | D | Destino, e o que a guarda até lá |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| R-IPC-01, o contrato antes do código | passo 20, `verificar-fiacao-ipc.sh`; a versão, checklist | b | não atende: 3 de 8 no passo | não atende: 1 de 1 | — | — | atende | O contrato com fonte única, no B4. Até lá, o lado Rust do passo 20 corrigido sobre o leitor de Rust |
| R-IPC-02, cliente, tratador e ouvinte | passo 20 | a no lado QML; b no lado Rust e C++ | não atende: 3 de 8 | não atende: 1 de 1 | não atende: 8 aceitos sem conferir se ainda casam | — | atende | O lado QML fecha agora. O lado C++ fica como está até o B4 (§6) |

### 4.4 Testes

| Regra | Conferida hoje por | Classe | S | P | F | B | D | Destino, e o que a guarda até lá |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| R-TEST-01, ferramenta real e NÃO PROVADO | passo 1 e o protocolo do orquestrador | c (ENG-45) | não medido | — | — | — | não medido | O `qml-logica` dá verde com 0 testes, e o INF-31 dá verde com o binário errado: os dois entram no ENG-45 e no ENG-03 |
| R-TEST-02, mutante por guarda | `mutantes.py --conferir`, depois do passo 31 | c (ENG-01) | não medido | — | — | — | não medido | Faltam o teto de memória do `--rodar` e o catálogo `gate` |
| R-TEST-03, espera por condição | nada | a | — | — | — | — | — | A conferência do ENG-30 sobre o leitor de Rust, como catraca de migração (pergunta 3) |
| R-TEST-04, o teste não toca a máquina da pessoa | o ambiente isolado à mão, no comando do gate | c (ENG-03) | não atende: a suíte grava um core dump por rodada no sistema | — | — | — | — | O orquestrador isola o ambiente; o executor de testes zera o limite de core (§9, G1) |
| R-TEST-05, teste de extensão | nada | T | — | — | — | — | — | Com cada ponto de extensão: B3, B5, B6, B7 e UI-02 |
| R-TEST-06, ajudante com um dono | nada | a nos ajudantes; b no sinal da ponte | — | — | — | — | — | Os ajudantes: a conferência do ENG-31, no B11. O sinal: o dublê gerado, no B4, com risco explícito até lá |
| R-TEST-07, refatoração não edita teste | checklist | L | — | — | — | — | — | — |

### 4.5 Processo, dependências e docs

| Regra | Conferida hoje por | Classe | S | P | F | B | D | Destino, e o que a guarda até lá |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| R-PROC-01 a R-PROC-05 | checklist | L | — | — | — | — | — | — |
| R-DEP-01, o `cargo-deny` | passo 7, `verificar-deny.sh` | c (ENG-03) | não medido | — | — | — | não atende: reprovou uma rodada sem rede | Sem rede, NÃO PROVADO, no ENG-03 |
| R-DEP-02, advisory é item próprio | checklist | L | — | — | — | — | — | — |
| R-DOC-01, número sem data | passo 26, `verificar-docs.sh` | c (ENG-06) | não atende: 1 de 7 | atende: 0 de 1 | não atende: tolera 2 linhas | — | atende | ENG-06 |
| R-DOC-02, só documento público | passo 28, `verificar-links-docs.sh`, e a conferência de privacidade | c (ENG-06) | não atende: 4 de 9 | não medido | — | — | atende | ENG-06 |
| R-DOC-03, um fato num dono | checklist | L | — | — | — | — | — | — |
| R-DOC-04, os arquivos que o git vê | os três gates de docs | c (ENG-06) | não atende: o `verificar-docs.sh` varre o disco | — | — | — | — | ENG-06 |

### 4.6 Gate e CI

| Regra | Conferida hoje por | Classe | Atende hoje? | Destino |
| --- | --- | --- | --- | --- |
| R-GATE-01, estáticos antes do build | `test_verification_profiles.py` | c (ENG-03) | não: os estáticos rodam por último | ENG-03, com o build da ferramenta do gate antes deles |
| R-GATE-02, folha não grava no checkout | nada | c (ENG-03) | não: o autoteste do idioma cria uma pasta temporária dentro do checkout (medido em 2026-11-12) | ENG-03 |
| R-GATE-03, defeito plantado por regra | nada | c (ENG-01) | não: o catálogo `gate` não existe | O catálogo `gate`, no fim do bloco |
| R-GATE-04, tempo não muda veredito | nada | c (ENG-01) | não | Com o catálogo `gate` |
| R-GATE-05, cache não decide | nada | c (ENG-05) | — : não há cache | ENG-05 |
| R-GATE-06, a CI roda o orquestrador | nada | c (ENG-07) | não: os pacotes e o `cargo-deny` só existem no YAML | ENG-07 (§8) |
| R-GATE-07, catraca de migração termina em zero | a tabela do 63 §B.2 | L | — | — |
| R-GATE-08, rodada vermelha da CI lida | checklist | L | — | Depois do desbloqueio da conta |
| R-GATE-09, base sem folga | cada verificador, do seu jeito | a | não: só 1 das 6 catracas reprova abaixo da base | O módulo comum de catraca da ferramenta (ENG-40) |
| R-GATE-10, a base só desce | cada verificador | a | não: 3 comandos sobem a base e 2 a recriam | O mesmo módulo, com a conferência contra a história do git |
| R-GATE-11, um leitor por linguagem | nada | a | não: um leitor por verificador | A ferramenta, com o corpus (ENG-41 pelo 64 §7) |
| R-GATE-12, falha fechado | nada | c (ENG-45) | não: 4 casos medidos em 2026-11-13, mais o INF-31 | ENG-45 e ENG-03 |
| R-GATE-13, o meio mais forte | checklist | L | — | — |

**A soma,** pela classe principal de cada uma das 55 regras:

- classe a, 10: R-ARQ-01, 03, 04 e 13; R-IPC-02; R-TEST-03 e 06;
  R-GATE-09, 10 e 11;
- classe b, 13: R-ARQ-02, 05 a 10, 14, 15 e 16; R-DADOS-01 e 04; R-IPC-01;
- classe c, 14: R-TEST-01, 02 e 04; R-DEP-01; R-DOC-01, 02 e 04;
  R-GATE-01 a 06 e 12;
- classe T, 6: R-ARQ-11 e 17; R-DADOS-02, 03 e 05; R-TEST-05;
- classe L, 12: R-ARQ-12; R-TEST-07; R-PROC-01 a 05; R-DEP-02; R-DOC-03;
  R-GATE-07, 08 e 13.

Três regras têm duas classes: a R-ARQ-01 é também do INF-16 (c); a
R-IPC-02 e a R-TEST-06 têm um lado que vai ao artefato gerado no B4 (b).

## 5. A tabela por passo do `scripts/verificar.sh`

A numeração é a do 63 §D.5; o `mutantes.py --conferir` entrou depois do
passo 31. Custo: a medida de 2026-11-13, de cada passo sozinho.

| # | Passo | Classe | S | P | F e B | D | Custo | O que muda |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | `test_verification_profiles.py` | c | não medido | — | — | não medido | 1 s | ENG-03 |
| 2, 3 | os diretórios de build e o build da UI | — | não medido | — | — | — | 0 s, aquecido | — |
| 4 | `cargo fmt --all --check` | — | não medido | — | — | — | 1 s | — |
| 5 | `cargo test --workspace` | c | os mutantes de cada item | — | — | não atende: INF-24 e INF-22 | 44 s | A suíte confiável (§9, G1) |
| 6 | `cargo clippy` | — | não medido | — | — | — | 0 s, aquecido | Recebe o `forbid` da R-ARQ-15 (B2) e o das R-DADOS (B7) |
| 7 | `verificar-deny.sh` | c | não medido | — | — | não atende | 1,5 s | ENG-03: sem rede, NÃO PROVADO |
| 8 | `verificar-shell.sh` | c | não atende: 1 de 5 | não medido | lista de exceção sem conferência | atende | 4,3 s | ENG-46: todo script que o git vê, por qualquer shebang |
| 9 | `verificar-appimage.sh` | — | não medido | — | — | — | 0 s | — |
| 10 | `verificar-cpp.sh` | c | não medido; `ui/tests` fica fora | — | a exceção só vale no clang-tidy 18 | — | 39 s | ENG-46, ENG-45 e ENG-05 |
| 11 | `verificar-cpp-testes.sh` | — | não medido | — | — | — | 0 s | — |
| 12 | `verificar-qml.sh`, o qmllint | — | atende no que foi plantado: 4 de 4 | atende: 0 de 2 | — | — | 12 s | Dono das propriedades (pergunta 5) |
| 13 | `check_identifier_language.py` | a | não atende: 3 de 10 | não atende: 4 de 4 | não atende: recria a base apagada | atende | 2,2 s | ENG-41: primeiro consumidor da ferramenta |
| 14 | `verificar-qml-fiacao.sh` | a | não atende: 2 de 6 | não medido | sem base | atende | 0,1 s | ENG-43 |
| 15 | `verificar-qml-propriedades.sh` | — | não atende: 3 de 6 | não atende: 2 de 2 | sem base | atende | 0,4 s | Pergunta 5: apagar |
| 16 | `verificar-qml-mortas.sh` | a | não atende: 1 de 7 | não medido | não atende: o comando sobe | atende | 11,3 s | ENG-43 |
| 17 | `verificar-qml-duplicacao.sh` | a | não atende: 2 de 10 | não atende: 1 de 1 | não atende: aceita abaixo sem avisar; o comando sobe | atende | 0,2 s | ENG-43 |
| 18 | `verificar-qml-tokens.sh` | a | não atende: 5 de 14 | não atende: 1 de 1 | não atende: recria a base apagada | atende | 0,2 s | ENG-43 |
| 19 | `verificar-qml-alcance.sh` | a | não atende: 1 de 6 | não medido | lista de exceção sem conferência | atende | 11,0 s | ENG-43 |
| 20 | `verificar-fiacao-ipc.sh` | a e b | não atende: 3 de 8 | não atende: 1 de 1 | não atende: 8 aceitos sem conferência | atende | 11,7 s | ENG-42 e ENG-43 nos lados Rust e QML; o lado C++ espera o B4 |
| 21 | `verificar-exercitacao.sh` | c | não medido | — | — | não medido | 29 s | INF-31; os 9 `sleep` fixos são do ENG-30 |
| 22 a 24 | embarcado, Python e clangd cross | c | não medido | — | — | não medido | 0 s a 3 s | INF-31 |
| 25 | `verificar-atalhos.sh` | a | não atende: 2 de 5 | não medido | sem base | atende | 0,05 s | ENG-43 |
| 26 | `verificar-docs.sh` | c | não atende: 1 de 7 | atende: 0 de 1 | não atende: tolera 2 | atende | 2,6 s | ENG-06 |
| 27 | `module_map.py --check` | a, c | não atende: 1 de 2 | não atende: 4 de 4 | não atende: 2 ciclos falsos | atende | 0,4 s | INF-16 |
| 28 | `verificar-links-docs.sh` | c | não atende: 4 de 9 | não medido | sem base | atende | 2,2 s | ENG-06 |
| 29 | `verificar-arquitetura.sh` | a | não atende: 3 de 8 | não medido | não atende: aceita abaixo; o comando sobe | atende | 0,08 s | ENG-42 |
| 30 | `verificar-adiados-windows.sh` | a | não atende: 3 de 10 | não atende: 1 de 1 | atende: folga zero, sem comando | atende | 0,06 s | ENG-42; a base zera na etapa 10 |
| 31 | `verificar-transicao-workspace.sh` | b | não atende: 1 de 7 | não medido | sem base | atende | 0,08 s | ENG-42 agora; apagado no B6 |
| — | `mutantes.py --conferir` | c | não medido | — | — | não medido | não medido sozinho | ENG-01 |
| 32 | `verificar-qml-logica.sh` | c | não medido; dá verde com 0 testes | — | — | não medido | 9 s | ENG-45 |
| 33, 34 | `verificar-binario-abre.sh` e `check_terminal_quiet.py`, só no `--completo` | c | não medido | — | lista de exceção | não medido | 38 s e 4 s | INF-31 (lido) |
| 35 a 37 | os builds de release e o binário de release, só no `--completo` | — | não medido | — | — | — | 0 s aquecido; 39 s | — |

Nenhum dos 16 passos com sonda atende a meta de sensibilidade, e 7
reprovam forma legítima. O custo em milissegundos por passo, no resumo,
não existe para nenhum (ENG-03).

## 6. A ferramenta do gate: o desenho proposto

A DP-06 e a R-GATE-11 já decidiram o quê: um leitor por linguagem, pelas
gramáticas tree-sitter, numa ferramenta Rust do workspace. O que segue é
o como, para a decisão do autor antes da implementação.

- **Um crate do workspace, com um binário.** Só ele depende das
  gramáticas; os crates do produto não dependem dele. O
  `tree-sitter-qmljs` e o `tree-sitter-bash` entram pelo registro de
  componente das [integrações](../integracoes/README.md).
- **Três partes, fechadas uma para a outra.** Os leitores, um por
  linguagem, que devolvem a árvore e as regiões de código, de comentário e
  de texto literal. O módulo de catraca, o único que lê e compara linha de
  base: lê a base, compara chave por chave, reprova acima e abaixo, só
  grava para baixo, com a mesma mensagem em toda regra, e confere a base
  contra a história do git. As regras, uma por módulo, registradas numa
  tabela: regra nova é um arquivo e uma linha.
- **O leitor falha fechado.** Arquivo do alcance que não se lê, e árvore
  com nó de erro, reprovam com o caminho e a linha (R-GATE-12). Na árvore
  de hoje, isso dá zero reprovações em QML, shell, Rust e Python (§2.2).
- **O corpus é teste do crate.** Uma forma por arquivo, com a leitura
  esperada ao lado, no `cargo test` da ferramenta. Os defeitos plantados e
  as formas legítimas de cada regra são testes da regra; o catálogo `gate`
  do `scripts/mutantes.py` os aplica numa cópia e roda o passo.
- **Cada verificador sai no commit em que a regra entra.** Não ficam dois
  leitores para a mesma regra, nem um script que só repassa a chamada.
- **O JavaScript da UI** é lido pela gramática de JavaScript, com a
  diretiva da linha 1 tratada pelo leitor e provada no corpus (§2.2).
- **O Python** passa a ser lido pelo `tree-sitter-python`, que o produto
  já compila, e não pelo módulo `ast`: a ferramenta é Rust. É uma emenda à
  R-GATE-11 (pergunta 4).
- **O C++ fica fora do gate de texto de estrutura.** A gramática não lê 20
  dos 87 arquivos. O que os gates precisam do C++ da ponte tem dono mais
  forte: os sinais e os métodos da ponte saem do contrato gerado, no B4.
  Até lá, o lado C++ do passo 20 fica como está: ele conta 220 sinais no
  cabeçalho, e o `moc` do Qt lista 220 no JSON que grava na árvore de
  build (medido em 2026-11-15, no build `dev-local`). Para o idioma, o
  C++ só precisa do nível léxico (comentário, texto literal,
  identificador), e isso volta ao autor (pergunta 4).
- **Os gates de docs seguem em Python.** Os defeitos medidos neles não são
  de leitura de Markdown: nome ambíguo, número antes do arquivo, tolerância
  de 2 linhas, alcance pelo disco. A gramática de Markdown fica como
  opção para os links por referência e em HTML, sem recomendação.

O 63 §E.2 foi escrito para leitores próprios em Python. Com a ferramenta,
mudam nele três pontos, e nenhum muda o aceite: o corpus mora no crate, e
não em `scripts/readers/corpus/`; o leitor de shell é a gramática, e não o
leitor de linha de hoje movido; e "o passo estático roda sem ferramenta
externa" passa a valer com o binário da ferramenta já construído
(R-GATE-01 emendada pela DP-06).

## 7. O que o B1, o B2 e o B3 precisam

| Fase | O gate que o aceite dela usa | Tem de existir antes da fase? |
| --- | --- | --- |
| B1, registro e guarda do laço | A suíte em que vermelho é defeito; a repetição em lote | Sim, as duas |
| | Os mutantes da guarda, pelo `scripts/mutantes.py` | Sim: o teto de memória do `--rodar` |
| | A catraca de arquitetura lendo certo: o `lib.rs`, onde o registro e a guarda entram, tem 554 linhas de produção contra o limite de 500, e hoje o gate o conta como 500 | Sim |
| | O teste com assinante em memória e o de redação de segredo | Não: nascem no B1 |
| B2, lançador e modo restrito | O projeto hostil do 64 §2.1 como fixture | Não: é o primeiro commit do B2, e a lista dos pedidos automáticos sai do registro do B1 |
| | O `Command::new` plantado que não compila | Sim: é defeito plantado num passo, e não mutante de teste. Precisa do catálogo `gate` |
| | A lista da migração: 150 `Command::new` em 79 arquivos de produção, 34 em 23 de teste, e 2 `CommandBuilder` num arquivo (contagem do texto cru, em 2026-11-15) | Não: a lista exata é a do compilador |
| | Nenhum processo órfão no fim | Não: nasce com o lançador, que é quem conhece os filhos |
| | O INF-29, a falha de `exec` sem aborto | Não: entra com a sessão de terminal no lançador |
| B3, classes de execução | As ferramentas falsas com trava e o runtime real nos testes (64 §6, risco 1) | Não: é a prova do começo do B3 (64 §9) |
| | O ajudante único de espera por condição (R-TEST-03) | Sim: o B3 escreve testes de concorrência, e hoje há 124 `recv_timeout` em 58 arquivos de teste e 70 `thread::sleep` em 35 (contagem do texto cru, em 2026-11-15) |
| | O teste do registro, que confere a classe de cada rota | Não: nasce no B3 |

**Um cuidado para o aceite do B3.** O 64 §8 pede "`core.ping` e
cancelamento em ≤ 50 ms com o git falso travado". Um teste com esse
limite de relógio dentro da suíte de 16 threads repete o INF-24. A
proposta: dentro da suíte, o teste prova a ordem, sem relógio (o `ping`
responde enquanto o git falso ainda está travado, e a trava só solta
depois); o limite de 50 ms é medido num passo próprio, sozinho, com p95.

**A camada "ordem temporal e concorrência" do 62 §3.5** pede sequências
forçadas e repetição em lote. As duas foram usadas nesta revisão, à mão,
e acharam a causa do INF-22. A repetição em lote entra como instrumento
do repositório antes do B1 (§9, G1). As sequências forçadas ficam com o
runtime de teste do B3.

## 8. A CI (ENG-07)

| O quê | De quem é | Fica pronto antes do desbloqueio? |
| --- | --- | --- |
| O desbloqueio da conta | do autor | — |
| O ambiente declarado: a imagem em `packaging/ci/`, com os pacotes e o `cargo-deny` que hoje só existem no YAML | do repositório | Sim |
| `scripts/ci-local.sh`, que roda o mesmo contêiner com o `podman` | do repositório | Sim |
| O `ci.yml` que só chama o contêiner e o orquestrador (R-GATE-06) | do repositório | Sim |
| O `test_verification_profiles.py` lendo o `ci.yml` | do repositório | Sim |
| A história inteira no checkout da CI: a conferência da base contra a história do git (R-GATE-10) não funciona com a profundidade padrão, de um commit | do repositório | Sim |
| O build da ferramenta do gate na CI | do repositório | Sim: ela já tem o Rust |
| A primeira rodada real, com a causa de cada vermelho lida (R-GATE-08) | do autor e do repositório | Não |

O Ubuntu 24.04 traz o Qt 6.4, abaixo do 6.5 em que o
`verificar_qml.py` confia: na CI, o qmllint sai como NÃO PROVADO, e o
perfil `--ci-rapida` hoje nem o roda. Enquanto a CI não tiver um Qt que
sirva, nenhuma regra de QML que dependa do Qt é conferida nela; as regras
de QML da ferramenta do gate são, porque só precisam do Rust.

Rodar o contêiner na máquina de medição pede baixar a imagem do Ubuntu, o
que depende da autorização do autor (pergunta 6).

## 9. A ordem proposta do bloco

Um item por commit, cada um depois do anterior fechado e do gate rápido
verde, sozinho. Em todo item: os defeitos plantados e as formas legítimas
como testes; os mutantes rodados numa cópia; as metas medidas de novo; os
donos atualizados. A base só desce: se a leitura certa revelar dívida, o
item para, e o número volta ao autor (D-22).

| # | Item | O que faz | Aceite | Evidência | Reversão |
| --- | --- | --- | --- | --- | --- |
| G1.1 | INF-24, o serial | O executor dos testes zera o limite de core do processo de teste. O defeito do produto é o INF-29, no B2 | Dentro da suíte, a sessão fecha em até 1 s (medido sem o core dump: 505 ms a 564 ms; o prazo de 4 s não muda). Nenhum registro novo de core dump depois da suíte | As rodadas em lote, antes e depois | Um commit |
| G1.2 | INF-06, com os dois testes do INF-24 | O prazo da verificação vira parâmetro também no caminho do handler. O teste que não é sobre o prazo recebe um prazo folgado; o que é sobre o prazo usa um compilador falso que nunca termina | Com o compilador da medida 250 ms mais lento por chamada, em que 7 de 10 verificações passam de 5 s, os dois testes passam. 50 rodadas da suíte sob carga, sem falha (63 §E.5) | A dose e a resposta do §2.3, de novo | Um commit |
| G1.3 | INF-22 | A troca do contexto passa a ter ordem: a carga mais velha não cai por cima da mais nova | O teste de unidade da troca fora de ordem, e o mutante que tira a comparação, que tem de reprovar. 32 cópias, 60 rodadas: 0 em 1920 (hoje, 24 e 11) | A sequência forçada do §2.3: 10 de 10 antes, 0 de 10 depois | Um commit |
| G1.4 | INF-24, o PlatformIO | O teste espera a condição (o contexto com a CDB nova), pelo ajudante único de espera, que nasce aqui | Com a releitura 300 ms mais lenta, 0 em 10 (hoje, 10 em 10). 16 cópias, 60 rodadas: 0 em 960 (hoje, 6 em 320) | A sequência forçada | Um commit |
| G1.5 | A repetição em lote | O script do repositório que roda a suíte N vezes e um teste K vezes ao mesmo tempo, com a taxa por teste | O script acha, na árvore de antes do G1.3, a falha do INF-22 | A saída do script nos dois lados | Um commit |
| G2 | ENG-01, o teto | O `--rodar` com teto de memória | A rodada dos pendentes, com 4 cópias, dentro do teto | A memória medida durante a rodada | Um commit |
| G3.1 | ENG-41, a ferramenta | O crate, os leitores de Rust, QML, JavaScript, shell e Python, e o corpus | O corpus inteiro lido como o esperado; zero nós de erro nos arquivos rastreados dessas linguagens | O `cargo test` da ferramenta; o custo do build e da leitura | Apagar o crate |
| G3.2 | ENG-40, a catraca comum | O módulo de catraca, com a conferência contra a história do git | Os defeitos plantados do 63 §E.2 (ENG-40): acima reprova, abaixo reprova, nenhum comando sobe nem recria | Os defeitos plantados | Um commit |
| G3.3 | O idioma | Primeiro consumidor (63 §E.2, ENG-41) | O do 63 §E.2: a base herdada desce, e o que o leitor revelar volta ao autor antes de virar catraca de migração | As sondas do passo 13: 10 de 10 e 0 de 4 | Um commit; o verificador antigo volta |
| G3.4 | INF-16 | O extrator do mapa de módulos | 213 arestas e 0 ciclos, ou a medida de novo se as dependências mudaram; a lista de ciclos apagada | As sondas do passo 27: 2 de 2 e 0 de 4 | Um commit |
| G3.5 | ENG-42 | Arquitetura, adiados, transição e o lado Rust da fiação, um commit cada | O do 63 §E.2 | As sondas de cada passo | Um commit cada |
| G3.6 | ENG-43 | Os gates de QML, um commit cada; absorve o ENG-04 | O do 63 §E.2; o custo dos três passos de 11 s cai para a ordem da leitura (§2.2) | As sondas de cada passo | Um commit cada |
| G4 | ENG-45 e ENG-46 | O gate que falha fechado; os arquivos que nenhum gate lê | O do 63 §E.2 | Os defeitos plantados | Um commit cada |
| G5 | ENG-03, com o INF-31 | Os estáticos antes e em paralelo, depois do build da ferramenta; o custo em milissegundos por passo; o estado do git antes e depois; o ambiente isolado; sem rede, NÃO PROVADO; o passo dinâmico exercita o binário que a rodada compilou e diz qual | O do 63 §E.2. Para o INF-31: o core plantado do §2.4 reprova com o alvo do cargo fora da árvore | O catálogo antes e depois (R-GATE-04) | Um commit por mudança |
| G6 | ENG-06 | Os gates de docs | O do 63 §E.2: as sondas dos passos 26 e 28 | 7 de 7 e 9 de 9 | Um commit |
| G7 | ENG-07, a parte do repositório | O §8 | O `ci-local.sh` roda o perfil da CI no contêiner e dá o mesmo veredito do `ci.yml` | A rodada no contêiner | Um commit |
| G8 | O catálogo `gate` | A soma dos defeitos plantados (63 §D.3) | 100% dos defeitos reprovam e toda forma legítima passa | A saída por regra | Um commit |
| — | B1 | — | — | — | — |

O ENG-05, o cache do clang-tidy, só muda custo e pode vir depois do G8 ou
com o B11, onde o 64 §8 o deixou.

**O que fica onde está.** O ENG-44 (B4), o ENG-47 e o ENG-02 (B7), o
ENG-12, o ENG-14 e o ENG-15 (B6), o ENG-20 e o ENG-22 (B5), o ENG-30 ao
ENG-38 (B11, com o ENG-37 por último), o ENG-39 (etapa 10) e o INF-28
(antes da fase D do [65](65-avaliacao-do-neovim-como-motor-de-edicao.md)).
O INF-30 é do verificador de correções ([62 §4](62-base-publica-0.4.md)) e
não é do gate: onde ele entra é do autor.

## 10. O risco do intervalo

O que fica sem gate de texto entre o fim deste bloco e a fase que impõe a
regra, pela proposta do §3:

| Regra | Imposta em | O que pode entrar no intervalo | Por que o risco é aceitável |
| --- | --- | --- | --- |
| R-ARQ-02 | B6, o grafo de crates | Uma dependência nova da base para um domínio | O import proibido não compila no B6; a lista é exata |
| R-ARQ-06 | B6 | Um ramo novo na função central de reações | A catraca de arquitetura segura o tamanho do `lib.rs` |
| R-ARQ-07 e R-ARQ-08 | B6, a privacidade | Um handler novo que toca campo de outro domínio | O campo privado não compila no B6. A contagem de texto que a regra cita seria feita agora e apagada no B6 |
| R-ARQ-09 | B5 | Um método novo na classe única da ponte | O B2 e o B3 criam métodos de qualquer jeito; o B5 migra todos |
| R-DADOS-01 e R-DADOS-04 | B7, o `forbid` | Uma leitura ou escrita direta nova | O `forbid` lista todas; os 10 mutantes pendentes seguem no catálogo, à vista |
| O sinal da R-TEST-06 | B4, o dublê gerado | Um harness novo que redeclara sinal (hoje, 340 redeclarações em 19 arquivos, medido em 2026-11-13) | O dublê gerado troca todas de uma vez |
| O lado C++ da R-IPC-02 | B4 | Um sinal ou método da ponte sem dono, numa forma que o regex não lê | O lado QML e o lado Rust passam a ler certo agora; o contrato gerado cobre o C++ |

Em todos, o checklist do commit (63 §G.3) segue valendo, e a regra é
conferida por quem executa e pelo autor.

**Onde a proposta é guardar com texto, apesar da fase** (pergunta 3):

- o passo 31, a transição do `workspace`: já existe, e corrigi-lo é uma
  regra curta sobre o leitor de Rust;
- a R-ARQ-10, QML de área que acessa a ponte: o B2 mexe na QML da
  confiança, e a estrutura do B5 não enumera sozinha os acessos de uma
  linguagem dinâmica. É a catraca de migração do ENG-22, que nasce com a
  medida do item (em 2026-11-12: 8 arquivos e 99 acessos);
- a R-TEST-03, a espera fixa em teste: o B1 ao B3 escrevem testes de
  concorrência. É a catraca de migração do ENG-30, que nasce com a medida
  do item.

As duas últimas criam linha de base nova, e isso é decisão do autor
(R-GATE-10).

## 11. As perguntas ao autor

Cada uma traz a recomendação primeiro.

1. **A ordem do bloco e o que vem antes do B1.** Recomendo a ordem do §9
   inteira antes do B1: G1 a G8. A alternativa mais curta deixa o G3.6, os
   gates de QML, para antes do B2: o B1 só mexe no core. Não recomendo,
   porque a decisão de 2026-11-15 é não começar fase sobre gate frouxo, e
   a diferença é de poucos commits.
2. **O INF-24 e o INF-22.** Recomendo os dois antes do B1, pelos itens
   G1.1 a G1.5, nenhum com prazo maior. Dois defeitos do produto
   apareceram na medida: a ordem da releitura do contexto (INF-22)
   corrige-se agora, numa mudança pequena e na direção do B3; a falha de
   `exec` que aborta (INF-29) corrige-se no B2, com o lançador, e até lá o
   executor dos testes zera o limite de core. O INF-30 não é do gate:
   onde entra?
3. **O que fica no gate de texto até o B4 e o B6.** Recomendo o princípio
   do §3: sem catraca de texto provisória para o que o compilador vai
   enumerar, com três exceções (§10). Duas delas criam linha de base nova,
   a do ENG-22 e a do ENG-30: o autor as aceita?
4. **A Parte A.** A proposta pede quatro emendas, pela revisão do 63 §H.2:
   - R-GATE-09 e R-GATE-10: a verificação cita `scripts/ratchet.py`; passa
     a citar o módulo de catraca da ferramenta do gate;
   - R-GATE-11: o Python lido pelo `tree-sitter-python`, e não pelo
     `ast`; e o C++ fora do alcance dos gates de estrutura, com o idioma
     lendo o C++ só no nível léxico da gramática, provado no corpus;
   - R-ARQ-07 e R-ARQ-08: tirar o "até lá, a contagem no `module_map.py
     --check`", e ficar o risco explícito do §10;
   - R-TEST-04: acrescentar o sistema de quem roda (o registro de core
     dumps) ao que o teste não grava.

   E uma regra nova, a R-GATE-14: "o passo que executa o produto executa
   o binário que a mesma rodada compilou, e diz qual" (INF-31). Recomendo
   as quatro emendas e a regra nova.
5. **O passo de propriedades** (63 §G.4). Recomendo apagar o
   `verificar-qml-propriedades.sh`: ele reprova 2 de 2 formas legítimas e
   3 de 6 defeitos, e o qmllint reprova os 4 que recebeu e aceita as 2. A
   CI sem Qt deixa de conferir propriedade de QML, e o perfil diz isso.
6. **A CI.** Recomendo fazer a parte do repositório no G7. Para provar o
   contêiner aqui, preciso baixar a imagem do Ubuntu 24.04 com o `podman`:
   o autor autoriza?
7. **O lugar deste plano.** Recomendo este documento como dono do bloco,
   com uma linha "G" no 64 §8, antes do B1, e a DP-11 no 64 §11
   registrando a decisão. As alternativas são uma seção nova no 64 ou o
   D-30 do 63.

## 12. O que não foi medido

- A folga das bases, os comandos que as sobem e o determinismo dos passos
  estáticos: valem as medidas de 2026-11-13, sobre os mesmos arquivos.
- A sensibilidade dos passos que executam ferramenta ou produto (63 §H.3):
  o catálogo `gate` mede.
- A ordem das trocas nas 24 falhas do primeiro lote do INF-22; no segundo
  lote, com a ordem registrada, foram 11 de 11.
- O INF-22 no produto em uso: quanto tempo o contexto velho fica quando a
  carga antiga é a última. Só o teste foi medido.
- O INF-31 nos outros passos dinâmicos e na UI: lido, não medido.
- Se uma versão mais nova do `portable-pty` corrige a falha de `exec`.
- A leitura do C++ no nível léxico contra o lexer do compilador.
- As gramáticas de CMake e de PowerShell, para os arquivos que o idioma
  passa a ler (63 §E.2).
- A ferramenta do gate em build de depuração, e o tempo dela dentro do
  gate.
- O perfil `--ci-rapida --estrito` num contêiner do Ubuntu 24.04.
- Quantas dependências da base para um domínio existem hoje (R-ARQ-02).
- A taxa de falha da suíte em 2026-11-15: 0 em 25 não diz que a
  intermitência acabou. As causas do §2.3 seguem na árvore.

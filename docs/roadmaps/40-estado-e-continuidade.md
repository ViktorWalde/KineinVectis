# 40 — Onde o projeto está, e por onde continuar

> **Classe: ESTADO** (`docs/README.md`). Remedido em **2026-09-11**, com o gate
> completo verde e **a IDE do checkout abrindo de novo** (§7.7). O dia anterior
> teve onze commits: o domínio `sim` inteiro (motor, tela e oráculo), o 19º
> gate, a checagem de unidades, a correção de categoria do `core_client.h`, a
> varredura e a análise de arquitetura. Se divergir do código, o código vence e
> este documento se corrige no mesmo gesto.
>
> **AVISO que a sessão de 2026-09-05 aprendeu na pele:** "gate verde" tem prazo
> de validade de uma atualização de sistema. Ao abrir a sessão o gate
> **reprovava**, e não era código — o Qt do sistema subiu de 6.11.1 para 6.11.2
> em 2026-09-04 22:21, e as duas árvores de build guardavam o caminho absoluto
> antigo. `cmake --preset dev-local` **e** `cmake --preset dev-local-release`,
> seguidos de rebuild, resolvem. **Cada árvore carrega o caminho por si; consertar
> uma não conserta a outra.** Detalhe em [`31`](31-simulacao-fisica-matematica.md) §8.0.
>
> **E esse remédio era INCOMPLETO — medido em 2026-09-11 (§7.7).** Reconfigurar
> troca o caminho da biblioteca e **não invalida objeto nenhum**: onze objetos
> compilados na tarde de 2026-09-04, contra os headers do 6.11.1, continuaram
> no link porque o rpm instala header com mtime de maio e o ninja compara mtime.
> O gate ficou verde e a IDE abortava ao abrir. O remédio inteiro para troca de
> Qt é reconfigurar **e** recompilar o que ficou velho — o 20º gate agora diz
> quais objetos são, e imprime o comando.
>
> **COMECE POR AQUI ao retomar.** Ele substitui o
> [`38`](38-divida-restante-e-continuidade.md) nesse papel; o 38 vira registro
> de como a fila estava quando a dívida foi paga.
>
> **Regra zero vale aqui como em tudo:** antes de aceitar qualquer item como
> pendente, MEÇA. Cada seção carrega o comando.

## 1. O estado, em números

```bash
bash scripts/verificar.sh                 # 22 verificacoes
cat scripts/arquitetura-baseline.txt      # a catraca
cargo test -q --workspace

# metodos IPC roteados. O `grep -v '^event.'` NAO e' cosmetico: sem ele a
# contagem pega dois nomes de EVENTO num `match` dentro de #[cfg(test)] em
# handlers/build.rs, e devolve 130 onde ha' 128.
grep -rhoE '"[a-z][a-zA-Z]*\.[a-zA-Z][a-zA-Z.]*"\s*(\||=>)' \
     crates/kinein-core/src/handlers/ crates/kinein-core/src/lib.rs \
  | grep -oE '"[a-z][a-zA-Z]*\.[a-zA-Z][a-zA-Z.]*"' | tr -d '"' \
  | grep -v '^event\.' | sort -u | wc -l

# eventos. Cinco nascem de format!("event.{domain}.*") em handlers/build.rs,
# com domain em {build, quality}: um grep de literal NAO OS VE e devolve 36
# onde ha' 41. Os cinco sao build.started/output/diagnostic e
# quality.started/output.
{ grep -rhoE '"event\.[a-zA-Z.]+"' crates/kinein-core/src/ | tr -d '"'
  for d in build quality; do
    for s in started output diagnostic finished; do echo "event.$d.$s"; done
  done
} | sort -u | wc -l
```

```text
protocolo   0.93.0
testes      703 Rust + 33 harnesses QML
metodos     140 IPC roteados, 43 eventos
dominios    33, e os 33 documentados no arquitetura/03
catraca     1 arquivo em debito
gate        22 verificacoes
```

> **O par `metodos`/`eventos` foi CORRIGIDO DE NOVO em 2026-09-06, e desta vez
> o erro não estava no número — estava no COMANDO.** Ele dizia `130` e `36`;
> medidos com o comando certo, são **128 e 41**. As duas falhas têm a mesma
> causa: grep de literal não sabe o que o literal é. Uma contava dois nomes de
> evento como se fossem método porque eles aparecem num `match` de teste; a
> outra não via cinco eventos porque eles são montados com `format!`.
>
> **Isto é a segunda vez que este par mente** — em 2026-09-04 ele dizia `139` e
> `35`. Da primeira vez a culpa foi de um número escrito sem comando que o
> reproduzisse. Desta vez havia comando, ele rodava, e ainda assim mentia.
> **Comando que reproduz não é o mesmo que comando que mede certo**, e a defesa
> agora é o comentário dentro de cada um dizendo o que ele exclui e por quê.

> **RECONFERIDO em 2026-09-06** com `bash scripts/verificar.sh` verde. O que
> bateu com o disco: 0.85.0, 639 testes, 28 harnesses, 18 verificações, catraca
> com 1 arquivo. O que **não** bateu foi o par `metodos`/`eventos`, e a causa
> está na nota acima. A medição das três candidatas está na
> [`31`](31-simulacao-fisica-matematica.md) §19, e a **escolha do autor** está
> na §7 deste documento.
>
> **Medido em 2026-09-05**, com o gate completo verde e a etapa 28 com CODIGO:
> o dominio `sim`, a tela dele e o INTEGRADOR. Os tres harnesses QML novos sao
> `tst_sim_controller` (a logica de montar a equacao), `tst_sim_run_controller`
> (a escolha numerica: metodo, passo, amostragem), `tst_sim_layout` (a GEOMETRIA
> da tela, na forma do `tst_configaction_layout`: a tabela de ligacao nao pode
> sumir) e `tst_sim_plot` (a ESCALA do grafico nos casos degenerados: trilha
> vazia, um ponto so', e a curva CONSTANTE, onde `max - min` vale zero e um
> grafico ingenuo divide por zero e some sem erro).
>
> **Nove metodos `sim.*`:** `catalog`, `inspectFormula`, `checkFormula`,
> `evaluate`, `estimate`, `run`, `list`, `save` e `forget`. O `estimate` existe porque a IDE MOSTRA o
> custo antes de rodar e contar passos e' regra de negocio — a UI pergunta e
> desenha, nao calcula (`ARCHITECTURE` §2).
>
> **O integrador e' verificado por ORDEM DE CONVERGENCIA**, que e' o
> procedimento da ASME V&V 20 e nao "o resultado parece razoavel": Euler mede
> 1,00, RK4 mede 3,85 na faixa onde isso e' mensuravel. E a faixa esta escrita
> na assercao, porque medir o RK4 entre 1e-4 e 1e-5 daria 0,34 e reprovaria um
> codigo correto.
>
> **E o core foi exercitado por STDIO, contra o binario de verdade**, em
> 2026-09-05. Ele reproduz as medicoes que guiaram o desenho inteiro:
>
> ```text
> metodo                calculado          erro contra o exato
> Euler explicito     -3.382195262            3.106309e+00
> Euler simpletico    -0.236244794            3.964147e-02
> Runge-Kutta 4       -0.275935335            4.906781e-05
> exato               -0.275886266940653
> ```
>
> Os tres erros batem com o que foi medido no rascunho ANTES de existir codigo
> (§8 do roadmaps/31: 3,11 / 3,96e-2 / 4,91e-5), e o valor exato bate com o
> `dsolve` do SymPy ate' a 14a casa. **A IDE mede o que a analise prometeu.**
>
> **E a simulacao SOBREVIVE a sessao**, em `.kinein/simulacoes/`, uma por
> arquivo, com `schemaVersion` e arquivo invalido tratado como ausente. O
> arquivo guarda o que o autor MONTOU — conceito, formula, ligacao, valores,
> metodo, passo — e **nenhuma linha do que a maquina produziu**. Isso e' gate,
> nao intencao: uma mutacao que injeta `"trail"` no arquivo derruba o teste
> `o_arquivo_guarda_o_que_o_autor_montou_e_nada_do_que_a_maquina_produziu`.
>
> **A catraca reprovou durante esta fatia**, e o corte foi por RESPONSABILIDADE.
> Medicao observada em 2026-09-05: ao ganhar o painel de simulacao o
> `ShellOverlays.qml` chegou a 309 linhas, acima do limite de 300 que a catraca
> guarda. Os cinco paineis de "Ambiente do projeto" (bibliotecas, banco,
> simulacao, observabilidade, instalacao) sairam para o
> `ShellEnvironmentOverlays.qml` — eles tem a mesma forma, o mesmo ciclo
> abrir/fechar, e sao o agrupamento que o menu ja' usa.
> O salto de 581 para 608 testes e de 121 para 125 metodos e' a **primeira
> fatia de codigo da etapa 28**: a forma ALGEBRICA, com `sim.catalog`,
> `sim.inspectFormula`, `sim.checkFormula` e `sim.evaluate`. O protocolo subiu
> de `0.82.0` para `0.83.0` pela mesma regra que o MongoDB e o Grafana seguiram:
> dominio novo sobe o minor.

> **O par `metodos` foi CORRIGIDO em 2026-09-04.** Ele dizia `139 IPC roteados,
> 35 eventos` e nenhum dos dois batia com o disco: medidos, sao 121 e 36. O
> numero entrou numa sessao anterior sem comando que o reproduzisse, e por isso
> envelheceu sozinho — exatamente o vicio que o `verificar-docs.sh` existe para
> pegar, e que ele nao pegou porque so' confere numeros que sabe medir. Os
> comandos acima ficam junto para o proximo que ler nao ter de confiar.

## 2. A dívida da catraca: 1 arquivo, e ele tem decisão do autor

```text
ui/qml/editor/EditorController.qml   791/400
```

**Não corte sem falar com o autor.** A decisão de mantê-lo congelado está em
[`../arquitetura/32`](../arquitetura/32-editor-por-responsabilidade.md) §8.4, e
as duas saídas estão medidas em [`39`](39-divida-tecnica-paga.md) §6: dissolver
a fachada (~190 pontos de chamada em 15 arquivos) ou corrigir a categoria.
Mexer em limite é decisão explícita dele, nunca do assistente.

## 3. O que esta sessão entregou (2026-09-03/04)

**Dívida paga: 8 arquivos → 1** — cada corte com a pergunta que o justifica em
[`39`](39-divida-tecnica-paga.md).

**Dois gates nasceram, os dois de relato de uso:**

```text
verificar-atalhos.sh       o atalho que a paleta ANUNCIA e' o que a IDE OBEDECE,
                           e todo item de menu tem tratamento no host
verificar-exercitacao.sh   o core contra as ferramentas REAIS desta maquina
```

**Frente de banco (etapa 26), completa até a introspecção:**

```text
26.1  perfil sem campo de senha, cofre decidido ANTES (../seguranca/40)
26.2  driver `postgres` 0.19.14, datasource.test como job
26.3  UI: painel, dialogo de senha por `secretRequired`, nunca por texto
26.4  introspeccao: esquemas, tabelas e colunas do information_schema
```

**Ambiente C/C++ e Rust:**

```text
6 acoes novas de REGIME de compilacao   rigor, sanitizers, OpenMP, .hex/.bin,
                                        alvo embarcado do Cargo
biblioteca com ciclo fechado            ativar E desativar, com a bolinha
                                        dizendo o que esta no PROJETO
campo que EXPLICA e SUGERE              valores reais lidos do projeto
toolchain automatico e VISIVEL          escolhe e mostra que escolheu
setup.list                              passo a passo OFICIAL por distro,
                                        com fonte e data
lista UNIFICADA "Ambiente do projeto"   biblioteca e acao lado a lado, com
                                        filtro por LINGUAGEM (nao por
                                        ferramenta) — o Cargo deixou de estar
                                        atras do C/C++
```

**A previa do `CMakeLists.txt` deixou de sumir** (roadmaps/35 §9.3.14). O autor
pediu polimento de tamanho; a medicao achou defeito: numa tela 1366x768, com uma
acao de seis campos, a caixa que mostra o arquivo media **10 pixels** — e o
botao "Ativar" continuava habilitado. A previa ganhou dono
(`ConfigActionDiffView`), o cabecalho ganhou teto de 40% com rolagem, o dialogo
cresceu de 1020x660 para 1100x820 e a caixa ganhou barra de rolagem.

```text
1920x1080, 3 campos    214px (~12 linhas)  ->  374px (~22 linhas)
1366x768,  6 campos     10px (NENHUMA)     ->  285px (~17 linhas)
```

**A observabilidade entrou: o Grafana pela API** (roadmaps/35 §9.6). Domínio
`grafana.*` com quatro métodos e um evento, cliente HTTP próprio (`ureq`: +5
crates, todas `MIT OR Apache-2.0`; o TLS foi ligado horas depois, quando o autor
decidiu a política de licença — §9.7.7), e o cruzamento que justifica o domínio
existir:

```text
dev  ->  kinein-dev      banco `kinein` em localhost:5432
```

Exercitado contra um **Grafana 13.0.2 de verdade**, incluindo os quatro modos de
falha (token inválido, sem token, porta errada, `https` — que na época era
recusado por falta de TLS e hoje conecta). Os dashboards
abrem no navegador — a licença AGPL decide a forma, e a IDE nunca embute.

**E a etapa 27 FECHOU, com o MongoDB** (roadmaps/35 §9.7), com as OITO
decisões do autor respondidas antes da primeira linha de código. O que ele traz
que nenhum motor anterior trazia:

```text
duas verdades    DECLARADO (validador `$jsonSchema`) ou INFERIDO de amostra,
                 e a tela NUNCA deixa os dois parecidos
o custo na tela  o `$sample` varre a colecao inteira quando N nao e' menor que
                 5% dela — a IDE conta ANTES e diz qual caminho vai acontecer
tetos de RAM     2.000 campos, 8 niveis, 10 elementos de array; quando um morde,
                 a colecao aparece com o aviso em vez de parecer completa
segunda forma    `DataSourceCollections` ao lado da arvore relacional, com
                 profundidade, tipo PLURAL e presenca em %
```

Exercitado contra um **MongoDB 8.2.12** real, e a exercitação achou o de sempre:
com o servidor em contêiner rootless, **`localhost` não conecta** (resolve para
IPv6 e o driver não cai para IPv4). A mensagem agora diz `tente 127.0.0.1`.

**Dois defeitos apareceram mexendo no código, não em gate:**

```text
Ctrl+O nao abria nada       `shellController` era usado pelo GlobalShortcuts e
                            NUNCA ligado pelo Main.qml; a chamada caia num
                            `null` em silencio. O menu funcionava, a tecla nao
requestFailed sem `code`    o cliente Qt descartava o codigo do erro, forcando
                            a UI a casar por TEXTO — exatamente o que o
                            comentario do `SecretRequired` existe para evitar
```

**E o harness passou a enxergar tela.** O `verificar-qml-logica.sh` monta um
espelho plano do modulo `KineinVectis` a partir das fontes; ate' 2026-09-04 so'
dava para testar componente que nao usa `Theme`, e geometria era zona sem
cobertura. O `tst_configaction_layout.qml` e o primeiro morador — provado por
mutacao: tirar o teto derruba a caixa a 70px, devolver o dialogo ao tamanho
antigo derruba a 249px, e cada mutacao acende um bit diferente.

## 3.5 O que a sessão de 2026-09-05 entregou: a etapa 28, em duas metades

**A etapa 28 pedia responder as sete perguntas da §5 do
[`31`](31-simulacao-fisica-matematica.md) e PRODUZIR ARQUITETURA.** A sessão fez
isso e foi além, em duas metades separadas por uma instrução do autor.

**Primeira metade — medir e perguntar, com código ZERO.** Por instrução dele:
medição antes, perguntas com opções depois, e nenhuma linha escrita até as sete
respostas existirem. A arquitetura saiu daí, em
[`../arquitetura/34`](../arquitetura/34-simulacao-por-conceito.md).

**Segunda metade — a primeira fatia de código**, quando ele mandou seguir:
domínio `sim` com nove métodos, catálogo de 17 conceitos, ligação explícita de
variáveis, integrador verificado por ordem de convergência, tela com gráfico 2D,
e persistência em `.kinein/simulacoes/`.

**O que a medição achou, e que nenhuma suposição teria achado:**

```text
evalexpr trocou de licenca     o avaliador de expressao mais usado do
                               ecossistema Rust (9,96M downloads) era MIT ate' a
                               v11 e virou AGPL-3.0-only na v12 (2024-10-17).
                               Quem adotou antes e rodou `cargo update` passou a
                               distribuir AGPL sem decidir nada. O deny.toml
                               reprova — e so' pega porque roda a cada build
mexprp reprova pela transitiva a licenca DELE e' MPL-2.0; quem reprova e' o
                               `rug`/`gmp-mpfr-sys` (GMP/MPFR), LGPL-3.0+
compilar custa 140x mais       do que rende: 490 ms de `cargo` por edicao da
                               formula para economizar 3 ms de conta num
                               pendulo. O ponto de virada e' 21,5 MILHOES de
                               avaliacoes
o exmex tem 4 armadilhas       e uma e' SILENCIOSA: `var_names()` devolve em
                               ordem ALFABETICA, nao a da formula. Quem casar
                               por ordem de leitura calcula a fisica errada sem
                               erro nenhum
a memoria compartilhada        nao e' exigencia do transporte. A §5.1.1 dizia
nao era necessaria             "250 MB/s, exige segundo canal" — isso vale para
                               frame CRU. Comprimido, um grafico 1920x1080 a
                               30fps sao 1,42 MB/s: tres vezes a grade do
                               terminal, que o JSON-RPC ja' carrega hoje
uom nao serve com formula      ele checa em tempo de COMPILACAO; formula digitada
digitada pelo usuario          pelo usuario nao tem tipo Rust nenhum
"exato" nao existe             Euler com dt=0.1 erra por 3,11 onde a resposta e'
                               -0,276 — onze vezes a propria resposta. E mais
                               passos pode PIORAR: RK4 com dt=1e-7 e' 108x pior
                               que com dt=1e-5
o Maxima nao mostra passos     pedido oficial #103: "won't fix". Nem o SymPy,
                               para EDO. A narracao algebrica de EDO nao existe
                               em ferramenta auditavel — o Wolfram tem, com
                               heuristica propria e POR FORA do motor que calcula
symbolica e' Pylance de novo   unico CAS serio em Rust, "fonte publica, proibido
                               copiar ou distribuir sem permissao expressa"
```

**E "tudo" virou número.** O pedido *"matematica/fisica basica, I, II, III e IV
— ou seja tudo"* foi mapeado contra as cinco formas do motor, no Apêndice A do
[`../arquitetura/34`](../arquitetura/34-simulacao-por-conceito.md), e a tabela
de lá é a fonte que se reconta sozinha:

```text
100 conceitos          8 disciplinas, de Matematica basica a Fisica IV
 66% forma ALGEBRICA   dois tercos de "tudo" NAO precisa de integrador nenhum
 88% com solucao       o oraculo tem com que comparar em 88 dos 100 — e' este
     fechada           numero que sustenta o "exato e preciso"
  9% forma EDP         a que o autor pos na etapa, e a que custa mais que as
                       outras quatro somadas. Nao reabre a decisao: informa a
                       ORDEM de construcao dentro da etapa, que segue aberta
```

**Uma correção minha, registrada porque premissa errada não some sozinha:**
ofereci o Maxima como caminho para o passo a passo algébrico e o autor escolheu
com base nisso. Exercitar as duas ferramentas **antes de escrever código** mostrou
que a premissa estava errada. O CAS entrou na função que ele de fato cumpre —
**oráculo de exatidão**, não narrador. Registro em
[`31`](31-simulacao-fisica-matematica.md) §13.

## 4. O que está aberto

```text
27  bancos relacional/temporal/nao-     FECHADA em 2026-09-04. Postgres,
    relacional, e o Grafana               TimescaleDB por nome, SQLite, MongoDB
                                          (com a segunda forma de exibicao) e o
                                          Grafana pela HTTP API. Falta so' o que
                                          ficou registrado como fatia PROPRIA:
                                          executar consulta, escrever, e o TLS
                                          do `postgres` — ver roadmaps/35 §9.7.10
--  TLS do `postgres`                     a LICENCA ja' esta' decidida (o
                                          `deny.toml` aceita as duas de 2026-09-04
                                          e o `ureq` e o `mongodb` ja' cifram).
                                          Falta o conector, que muda a chamada de
                                          conexao — fatia propria
28  simulacao fisica/matematica         as SETE perguntas da §5 do roadmaps/31
                                          respondidas, ARQUITETURA em
                                          ../arquitetura/34, e a PRIMEIRA FATIA
                                          DE CODIGO entregue em 2026-09-05:
                                          forma algebrica + integrador (EDO 1a e
                                          2a ordem) + tela + grafico 2D +
                                          persistencia. Medicao em roadmaps/31
                                          §8 a §18. O que falta esta' abaixo
--  sim: SISTEMA_EDO                      FECHADA. O motor saiu em 2026-09-06
                                          (§7.1) e a TELA foi ligada nele em
                                          2026-09-07 (§7.2) — antes disso o motor
                                          existia sem porta, e os tres conceitos
                                          apareciam na lista caindo na tela
                                          ESCALAR. Falta so' salvar, abaixo
--  sim: salvar um SISTEMA                 RECUSADO com motivo na tela, em
                                          2026-09-07. O `SimSaved` do protocolo
                                          carrega UMA formula e UMA ligacao, e um
                                          sistema tem uma de cada por componente:
                                          gravar assim escreveria uma montagem
                                          que nao volta. Fatia propria — muda o
                                          protocolo, o `persistencia.rs` e os
                                          testes dele
--  sim: motor de EDP                     onda, calor, Laplace, Schrodinger — a
                                          parte de CAMPO da Fisica II, III e IV.
                                          DECISAO DO AUTOR de que entra nesta
                                          etapa (2026-09-05). Custo medido em
                                          roadmaps/31 §16: malha, condicao de
                                          contorno, e a ESTABILIDADE como gate.
                                          REMEDIDA em 2026-09-06 (§19.2), e o
                                          preco subiu: o interpretado e' 90,3x o
                                          compilado, nao 7,6x, entao 201x201 por
                                          1 s sao SEIS MINUTOS no motor de hoje.
                                          "EDP sempre compila" virou PRE-REQUISITO
                                          e o motor compilado NAO EXISTE. Duas
                                          boas noticias na mesma medicao: o quadro
                                          de campo CABE no canal atual se for
                                          quantizado (1,62 MB/s a 30 fps em
                                          201x201), e a parede de estabilidade so'
                                          aparece depois de N passos que dependem
                                          da inicial — 146 com um pico, 911 com
                                          uma gaussiana
--  sim: o oraculo SymPy                  FECHADO em 2026-09-10 (§7.3). A coluna
                                          `exato` passou a ter PROCEDENCIA, e com
                                          o SymPy presente ela responde pela
                                          equacao DIGITADA. Exercitado contra o
                                          binario real: os tres casos em que a
                                          IDE mentia agora batem com o erro de
                                          verdade, e o quarto (nao linear) cai na
                                          solucao do conceito AVISANDO. Falta so'
                                          o que esta abaixo
--  sim: o oraculo da forma VETORIAL       o `dsolve` sobre SISTEMA nao foi
                                          medido, e entrar sem medir e' o oposto
                                          do que este dominio faz. Hoje a tela do
                                          sistema carrega a ressalva, e o sinal
                                          honesto dela e' o INVARIANTE
--  sim: `sim.run` deveria ser JOB         ele e' sincrono e pode levar minutos
                                          (teto de 100 milhoes de passos); o
                                          oraculo acrescenta 5 s no pior caso.
                                          Nao e' regressao desta fatia — e' um
                                          desenho que ficou visivel por causa
                                          dela
--  sim: o SymPy como oraculo (registro)  hoje a IDE so' sabia o erro dos TRES
                                          conceitos cuja solucao fechada alguem
                                          escreveu a mao. Com o SymPy, ela
                                          saberia o de qualquer equacao que o
                                          `dsolve` resolva. Decidido em
                                          2026-09-05; falta a deteccao do
                                          processo e a racionalizacao dos
                                          coeficientes (roadmaps/31 §15.5: o
                                          `dsolve` quebra com float).
                                          MEDIDA em 2026-09-06 (§19.3): o SymPy
                                          NAO ESTA nesta maquina (a medicao de
                                          2026-09-05 usou o que hoje falta), o
                                          `dsolve` TRAVA no pendulo nao
                                          linearizado (>20 s, exige teto de
                                          tempo), e o `exmex` LE a saida dele com
                                          diferenca de 6,9e-18 — uma ida por
                                          FORMULA, nao por ponto. Cobertura nova
                                          no catalogo de hoje: ZERO. O valor dele
                                          e' consertar o defeito da §19.0
--  sim: unidades CHECADAS                FECHADO em 2026-09-10 (§7.4) para as
                                          formas que INTEGRAM. Falta a ALGEBRICA,
                                          e o motivo e' de catalogo: ela nao
                                          declara a unidade do RESULTADO
                                          (`energia-cinetica` declara `m` e `v`,
                                          nao o joule), entao so' metade da
                                          checagem seria possivel — e meia
                                          checagem numa tela que promete conferir
                                          e' pior que nenhuma
--  sim: `kinein-sim` e a vista 3D        o processo separado com OpenGL
                                          offscreen. Depende do SISTEMA_EDO ou
                                          da EDP existirem — antes disso nao ha'
                                          trajetoria 3D nem campo para desenhar
--  sim: janela em resolucao cheia        a trilha e' amostrada; pedir um trecho
                                          em detalhe re-executa aquele pedaco
                                          (arquitetura/34 §7.1)
--  core_client.h em 500/500            FECHADO em 2026-09-10 (§7.5), por
                                         DECISAO DO AUTOR e pela saida (b): a
                                         categoria estava errada, nao o arquivo.
                                         Um header que so' DECLARA nao tem
                                         logica para esconder, e o limite dele
                                         passou a ser 2x o que ele declara — nao
                                         um numero escolhido. A saida (a) foi
                                         MEDIDA e descartada: 422 linhas de
                                         declaracao contra 63 de comentario, e
                                         mover comentario compraria ~33 linhas
                                         contra a convencao da linguagem
--  A IDE DO CHECKOUT NAO ABRE            FECHADO em 2026-09-11 (§7.7). Achado em
                                          2026-09-10 como prioridade 1: os dois
                                          builds abortavam (SIGABRT, display real
                                          e offscreen) com os 19 gates verdes,
                                          e o AppImage abria. A HIPOTESE do dia
                                          — "QML compilado em AOT contra o Qt
                                          6.11.2" — estava ERRADA na causa e
                                          certa no sintoma: o chamador era o
                                          AOT, mas o culpado eram 11 OBJETOS
                                          compilados antes da troca de Qt que o
                                          ninja nunca recompilou (rpm instala
                                          header com mtime de maio). Violacao de
                                          ODR no `copyAppend` inline. Nasceu o
                                          20o gate: `verificar-binario-abre.sh`
--  FECHADOS nesta passada, e ficam       a coluna `exato` que respondia por
    aqui so' como registro                OUTRA equacao (§19.0) — consertada em
                                          2026-09-10 pela PROCEDENCIA, §7.3; e o
                                          `03-ipc` sem cobertura de 5 dominios,
                                          escrito em 2026-09-06. Conferido em
                                          2026-09-10: dos 130 metodos roteados,
                                          ZERO estao ausentes do documento
25  handshake DAP com probe-rs           PARCIAL: precisa de sonda fisica ou
                                         alvo QEMU. O resto do ciclo de
                                         embarcado esta' provado no CORE; na
                                         TELA, nada dele chega (medido em
                                         2026-09-11: menu de toolchain sem
                                         `debugAdapter`, kit sem `chip`,
                                         `probe.list` sem consumidor). As DOZE
                                         decisoes do autor para fechar a frente
                                         estao no roadmaps/35 §5.7, e a ordem
                                         e' fio -> QEMU -> tela -> polimento
--  ESP32 NA MESA: conectividade         LEVANTADO em 2026-09-11 (§7.12,
    bare metal                           integracoes/38). O autor plugou um
                                         ESP32 na tarde do dia em que a §5.7 do
                                         35 dizia que nao podia. MEDIDO: e' um
                                         ESP32-D0WD-V3 CLASSICO (Xtensa) atras
                                         de CP2102 em /dev/ttyUSB0, dialout
                                         OK, auto-reset OK, flash 4 MB — sem
                                         USB-JTAG embutido. Gravar e monitorar
                                         sao exercitaveis HOJE; depurar exige
                                         ESP-Prog + openocd-esp32 + esp-gdb, e
                                         NAO esta' na mesa. O core nao sabe o
                                         que e' porta serial. Seis fatias
                                         propostas (E1 serial.list -> E3
                                         monitor -> E5 identidade Espressif ->
                                         E2 permissao por canal -> E4 gravar
                                         como JOB com motor por familia; E6
                                         debug bloqueado por hardware).
                                         DECIDIDO pelo autor na mesma noite
                                         (38 §6), com a regra "padrao de
                                         mercado, solucao pronta, nada
                                         escrito do zero": monitor = PROCESSO
                                         (`espflash monitor` / `tio`) no
                                         painel de terminal, MPL-2.0 NAO
                                         entra; gravar = CONFIGURACAO DE
                                         EXECUCAO, nao dominio novo; ordem
                                         E1 -> E3 -> E5 -> E4 -> E2; um C3/C6
                                         vai para a mesa (USB-JTAG embutido,
                                         probe-rs sem fork). E1 `serial.list`
                                         FEITA na mesma noite (§7.13, 0.91.0),
                                         exercitada contra o ESP32 real. E3
                                         (monitor como processo) FEITA em
                                         2026-09-12 (§7.15). O autor pediu uma
                                         ORGANIZACAO antes de seguir: a trilha
                                         PROFUNDA de embarcados e' o roadmaps/42
                                         — oito pilares, pronto POR FAMILIA. O
                                         PILAR 0 (o MODELO do projeto) teve a
                                         primeira fatia em 2026-09-12 (§7.16):
                                         dominio `project`, 9 frameworks com
                                         evidencia, SDKs, artefatos, alvo. O
                                         PROXIMO e' o que falta do P0 (42 §3):
                                         o modelo por ALVO/preset, o
                                         flasher_args.json e a tabela de
                                         particoes LIDOS, o map file, e o
                                         P1 (setup com as ferramentas). As
                                         tres decisoes do 42 §7 foram
                                         TOMADAS em 2026-09-12: so' o ESP32
                                         classico na mesa (o resto fecha no
                                         gate e fica dito como nao exercitado);
                                         Raspberry Pi OS como alvo Linux; P0
                                         primeiro
--  O ECOSSISTEMA INTEIRO, em ordem      MAPEADO em 2026-09-11 (roadmaps/41), a
    linear: embarcados + Python +        pedido do autor: "nada deve ficar de
    MicroPython                          fora", com o VS Code (Python, C/C++,
                                         Rust, Cortex-Debug, probe-rs,
                                         PlatformIO, ESP-IDF, Pico) como
                                         referencia de FUNCIONALIDADE e a
                                         ferramenta aberta por tras como
                                         implementacao, licenca lida no
                                         arquivo. Seis blocos: A fecha o canal
                                         serial e o ciclo Espressif (E3->E5->
                                         E4->E2, + setup + saida do teste); B e'
                                         Python inteiro (Tree-sitter ->
                                         interpretador -> basedpyright -> ruff
                                         -> debugpy -> pytest); C MicroPython
                                         (mpremote, stubs, firmware); D
                                         profundidade (RTT, SVD, memoria,
                                         RTOS, clang-tidy, cobertura, Renode);
                                         E frameworks (ESP-IDF, pico-sdk,
                                         Zephyr, PlatformIO); F o grande
                                         (Jupyter, SSH, Docker). O PROXIMO
                                         continua sendo A1 = E3, o monitor
--  guias de instalacao para arch/suse   a fonte oficial dos tres projetos NAO
                                         cobre essas familias; entrar exige
                                         fonte de comunidade, marcada como tal
--  SSH REMOTO nativo                    DECIDIDO pelo autor em 2026-09-11,
                                         NAO arquitetado. Abrir e trabalhar num
                                         workspace REMOTO por SSH — editar,
                                         compilar, rodar e depurar na maquina do
                                         outro lado —, nativo como Docker e
                                         banco sao nativos (§5), nunca plugin.
                                         Antes de qualquer codigo, MEDIR: onde o
                                         core assume caminho LOCAL (fs, fswatch,
                                         run, terminal, build, dap, toolchain
                                         resolvem no disco desta maquina), o que
                                         vira canal remoto e o que continua
                                         local, e a licenca do transporte (o
                                         `ssh` do sistema como PROCESSO, ou uma
                                         crate — a decidir com fonte). E' fatia
                                         grande e propria; entra na fila sem
                                         data de execucao
```

## 5. As decisões registradas que NÃO se reabrem

```text
IA na IDE                    fora de escopo (2026-07-17)
simulacao: quem calcula      processo `kinein-sim` separado calcula E desenha; a
                             IDE pinta o frame como IMAGEM 2D (2026-09-03)
simulacao: catalogo          DUAS CAMADAS — o nome que o usuario conhece por
                             cima, a forma matematica por baixo (2026-09-05)
simulacao: a formula         o usuario DIGITA a equacao dentro do conceito que
                             escolheu, e a IDE alerta NA HORA da digitacao
                             quando os dois nao batem (2026-09-05)
simulacao: exato             nao existe. Integracao numerica tem erro, e a IDE
                             MOSTRA o erro em vez de prometer exatidao. O SymPy
                             entra como ORACULO, nunca como narrador (2026-09-05)
simulacao: a grade           basica + I a IV, matematica E fisica. A EDP entra
                             como QUINTA forma ja' nesta etapa, porque a parte
                             de campo da Fisica II/III/IV nao cabe nas outras
                             quatro (2026-09-05)
simulacao: unidades          declaradas E CHECADAS em runtime pelo
                             `check_dimensions` do SymPy. REVERTE a decisao de
                             rotulo-sem-checagem tomada horas antes: ela se
                             apoiava no `uom`, que checa em COMPILACAO; o SymPy
                             checa em EXECUCAO, que e' quando a formula do
                             usuario existe (2026-09-05)
simulacao: o raciocinio      so' existe para INTEGRAL (`integral_steps`). Nas
                             demais formas a IDE mostra substituicao numerica,
                             solucao fechada com o erro, e o metodo nomeado —
                             e DIZ que nao tem derivacao, em vez de inventar
simulacao: DOIS caminhos     o catalogo com formula digitada, E o usuario
                             escrevendo o proprio codigo. Rodar o codigo dele
                             JA' FUNCIONA (run.*, runConfig.*, event.run.output,
                             fswatch): falta so' ligar a saida ao desenho
                             (2026-09-05)
simulacao: a saida do codigo DECLARADA na run config, nunca adivinhada. A
                             heuristica de reconhecimento foi MEDIDA (zero falso
                             positivo contra cmake, cargo, g++, ping, df) e
                             recusada por preferencia do autor: sem magia. A IDE
                             nao linka, nao exige header, nao toca no codigo dele
simulacao: 2D ou 3D          o CONCEITO declara a vista natural, e o usuario
                             pode trocar (2026-09-05). E' a UNICA excecao ao
                             principio abaixo, e esta' registrada como excecao
simulacao: NADA ADIVINHADO   principio SUPERIOR as outras decisoes (2026-09-05).
                             Campo comeca VAZIO; o usuario preenche metodo, dt,
                             motor, amostragem e o papel de cada variavel; a IDE
                             calcula e MOSTRA, e nunca altera um numero dele.
                             REVOGOU duas decisoes do mesmo dia: a IDE escolher
                             o motor, e a IDE corrigir o dt instavel da EDP
symbolica (CAS Rust)         PROIBIDO — fonte visivel, uso proprietario, licenca
                             a adquirir. Categoria Pylance (2026-09-05)
Python                       ENTRA como vertical nativa — DECISAO DO AUTOR em
                             2026-09-11 (roadmaps/41), REVERTENDO o "adiado" de
                             2026-08-30. Sem anuncio parcial: a tela so' diz
                             "Python" quando a cadeia inteira funcionar (41 §5, B8)
Pylance                      PROIBIDO (licenca) — continua; o motor e' basedpyright
Docker e banco               NATIVOS, nao plugins. Docker/Podman IMPLEMENTADO
                             em 2026-09-12 (§7.14): dominio `container`
EditorConfig                 auditado com resultado NEGATIVO (2026-07-16)
Grafana embutido             PROIBIDO (AGPL) — integracao por HTTP API
TLS na IDE                   ENTRA (autor, 2026-09-04): `subtle` (BSD-3-Clause) e
                             `webpki-roots` (CDLA-Permissive-2.0) estao na
                             allowlist, com justificativa datada no deny.toml
esquema de documento         DECLARADO quando ha' validador, INFERIDO de amostra
                             quando nao — e a tela nunca deixa os dois parecidos
custo de leitura             sempre VISIVEL: quantos documentos foram lidos e se
                             a leitura obrigou o servidor a varrer tudo
EditorController             congelado ate' decisao do autor (../arquitetura/32 §8.4)
senha em disco               PROIBIDA: a IDE guarda o PERFIL (../seguranca/40)
comando de instalacao        so' com FONTE OFICIAL citada e datada; sem fonte,
                             a IDE mostra o site e diz que nao tem passo a passo
dimensionamento da UI        AUTOMATICO pelo conteudo, com PISO e com o teto da
                             janela (autor, 2026-09-04). Altura fixa corta
                             conteudo; automatica sem piso faz a tela piscar
UI/UX moderna e minimalista  ETAPA A PARTE, depois do pente-fino (autor,
                             2026-09-04). Nao se antecipa em fatia de
                             funcionalidade, e nao reabre decisao registrada
embarcados: escopo           ARM Cortex-M via probe-rs nesta etapa; RISC-V e
                             ESP32 depois; AVR/Arduino NAO entra (2026-09-11)
embarcados: ponte GDB        ENTRA como `gdb -i dap`, 2o candidato do papel
                             debugAdapter — o GDB fala DAP desde a v14, e isso
                             derruba o "protocolo novo inteiro" do 36 §3
                             (2026-09-11)
embarcados: prova            QEMU no gate com fixture propria; sonda real so'
                             quando plugada, e ate' la' NAO PROVADO e dito
                             (2026-09-11)
embarcados: deducao          `probe-rs info` SUGERE, o usuario CONFIRMA; a IDE
                             nunca escolhe o chip calada (2026-09-11)
embarcados: templates        catalogo CURADO com fonte e licenca, nunca
                             linker script adivinhado (2026-09-11)
```

## 6. A lacuna que não é técnica, e continua sendo a mais cara

**O registro de saídas do dogfooding continua VAZIO**
(`docs-privada/diario/19-registro-de-saidas.md`).

E esta sessão deu a prova mais forte que existe de que ele importa: **cinco
defeitos reais** — a busca por arquivo quebrada pelo `fd` 10.4.2, o atalho da
biblioteca que formatava o arquivo, os dezoito alvos de link que não aceitam
link, o `CMAKE_CXX_STANDARD` escrito onde não faz efeito, e a prévia do
`CMakeLists.txt` com 10 pixels numa tela de notebook — **foram achados por
frases do autor usando a IDE**, não pelos dezoito gates.

O quinto ensina uma coisa a mais: **a frase apontou o lugar, a medição achou o
tamanho.** O autor disse *"a caixa ficou com um dimensionamento pequeno"* e
pediu polimento. Instanciar o diálogo real fora do app e medir mostrou que, na
tela dele com uma ação de seis campos, a caixa não era pequena — era
**inexistente**, com o botão de consentimento habilitado do lado. Relato de uso
diz **onde** olhar; medir diz **quanto**. Aceitar a frase como veredito teria
produzido um ajuste cosmético e deixado o defeito de pé.

Uma frase de uso vale mais que uma refatoração da lista.

## 7. A escolha do autor para a etapa 28 — 2026-09-06

**Ele escolheu a recomendação medida.** A ordem, registrada:

```text
1. SISTEMA_EDO       a forma vetorial. ENTREGUE em 2026-09-06 (§7.1), e a TELA
                     ligada nela em 2026-09-07 (§7.2)
2. o oraculo SymPy   ENTREGUE em 2026-09-10 — ver §7.3
3. a EDP             por ultimo, e o motor compilado e' fatia PROPRIA antes dela
```

**O que sustenta a ordem está medido em [`31`](31-simulacao-fisica-matematica.md)
§19**, e o resumo de uma linha por candidata:

```text
SISTEMA_EDO   o motor interpretado de hoje BASTA (3,67 s para 100 voltas de
              orbita, 3,50 s para 60 s de pendulo duplo). Nao arrasta subsistema
              nenhum, e e' o unico que abre a vista 3D
oraculo       cobertura NOVA no catalogo de hoje: zero. O valor dele e' consertar
              o defeito da §19.0, e ele cresce com o catalogo — por isso depois
EDP           o interpretado e' 90,3x o compilado, nao 7,6x. "EDP sempre compila"
              virou PRE-REQUISITO, e o motor compilado nao existe
```

**O pedido veio com uma condição, e ela foi cumprida antes:** *"atualize/sincronize
toda a documentação antes"*. O que a sincronização de 2026-09-06 fez:

```text
arquitetura/03   os CINCO dominios ausentes ganharam secao (command, setup,
                 datasource, grafana, sim), mais o `core.*` que faltava. As
                 duas listas foram REFEITAS a partir do codigo: a de metodos
                 listava 66 de 128 sem dizer que era parcial
o par de numeros CORRIGIDO pela segunda vez, e desta vez o erro estava no
                 COMANDO: 128 metodos e 41 eventos (§1)
LEITURA_TECNICA  classe ESTADO, e estava muito velho: core listado com 25.823
                 linhas e tem 43.750; o par de numeros errado com data ao lado
arquitetura/04   413 testes -> 639, em dois lugares
```

### 7.1 O que a forma SISTEMA_EDO entregou — 2026-09-06

**Dois métodos novos, `sim.checkSystem` e `sim.runSystem`**, e o protocolo subiu
de `0.85.0` para `0.86.0`. O desenho está em
[`../arquitetura/34`](../arquitetura/34-simulacao-por-conceito.md) §13, escrito
**antes** do código.

```text
tres conceitos novos   orbita de dois corpos, pendulo duplo, massa-mola acoplada
tres metodos           Euler, simpletico e RK4, os tres vetoriais
oraculo                solucao fechada onde e' honesto (a orbita CIRCULAR), e
                       INVARIANTE onde nao ha' — energia e momento angular
grafico                componentes no tempo, e trajetoria no plano declarado
```

**O core reproduz a medição que guiou o desenho**, exercitado por stdio contra o
binário de verdade em 2026-09-06 — órbita circular de raio verdadeiro 1, dez
voltas, `dt=0,01`:

```text
metodo                raio final     deriva de energia   deriva de |L|
Euler explicito         1.647957            2,032e-01       2,975e-01
Euler simpletico        1.000024            2,800e-10       1,554e-15
Runge-Kutta 4           1.000000            8,727e-11       8,727e-11
```

Os três números batem com o que a §19.1.1 do
[`31`](31-simulacao-fisica-matematica.md) mediu **antes de existir código**.

**A decisão que mais custou, e o que ela paga:** o método simplético exige saber
qual componente é posição de qual velocidade, e num sistema de primeira ordem
genérico esse par não existe. Deduzi-lo seria a dedução que erra calada, então
**o conceito declara o pareamento** — e um conceito que não declara **não oferece
o método**, com o motivo na recusa. É o que separa `1,647957` de `1,000024`.

**Três defeitos apareceram durante a fatia, e nenhum veio de gate:**

```text
o oraculo era perguntado    a corrida para em `passos * dt`, e o oraculo era
pelo instante ERRADO        avaliado na `duracao` PEDIDA. Com duracao = 2*pi e
                            dt = 0,01 a diferenca e' 3,2e-3 — e o "erro"
                            mostrado passava a incluir uma diferenca de TEMPO
                            que nao e' erro de integracao. Fez um RK4 medir
                            ordem 0,81. CORRIGIDO nas duas formas, escalar e
                            vetorial: a escalar carregava o mesmo defeito sem
                            aparecer, porque `duracao = 10` com `dt = 0,1` da'
                            exatamente 100 passos
as equacoes do pendulo      escritas de memoria, passaram na compilacao e no
duplo estavam ERRADAS       checador, e reprovaram no INVARIANTE: a energia
                            derivou 2,18 onde deveria ficar parada. Derivadas
                            de novo com o SymPy e conferidas contra a
                            lagrangiana em 2.000 pontos (maior diferenca
                            7,1e-15). **O invariante pegou o que nenhum outro
                            gate pegaria** — e' exatamente para isso que ele
                            existe
o conceito normalizava      `g`, `l` e `m` iguais a 1 embutidos no catalogo e'
a fisica em silencio        a IDE decidindo fisica, que a §2.1 proibe. Achado
                            por um gate que ja' existia: `catalogo_declara_
                            conceitos_com_fonte_datada` cobra grandeza
                            declarada, e o pendulo nao declarava nenhuma
```

**Cinco mutações provam os gates do core** (simplético vira explícito, ligação
casada por nome, pareamento deixa de ser exigido, tolerância do oráculo frouxa,
oráculo no instante pedido) **e três provam o do QML**. Uma sexta mutação foi
recusada por dar falso negativo — o compilador a pegou antes do teste, que é a
armadilha já registrada.

**E o gate do QML ensinou uma coisa nova sobre gates.** Duas mutações **não
mataram**, e as duas foram registradas:

```text
teto de comprimento     `indice < values.length` era REDUNDANTE — o acesso fora
                        da faixa ja' chega como `undefined`. SAIU do codigo:
                        linha que nenhuma mutacao mata nao defende nada
guarda de `undefined`   tambem nao mata, porque em JS `undefined < x` e
                        `undefined > x` sao os dois falsos. FICOU, com o motivo
                        escrito: a alternativa e' depender de comportamento
                        implicito da linguagem
```

A linha que **de fato** defende é `!isFinite` nos extremos, e foi preciso mutar
três candidatas para descobrir qual era.

### 7.2 A tela foi ligada no motor — 2026-09-07

**A fatia anterior entregou motor sem porta, e o gate inteiro ficou verde.** O
`SimPlotSystem.qml` estava no `QML_FILES`, tinha harness próprio que passava, e
não era instanciado em lugar nenhum do app; o `SimSystemController` nascia no
`AppDomains` e ninguém o lia. Medido contra o binário real antes do conserto:

```text
sim.catalog        devolve os 3 conceitos odeSystem, e a lista NAO filtra por forma
sim.checkFormula   concept=orbita-dois-corpos, formula="mu*2"  ->  {"ok": true}
sim.evaluate       -> "Orbita de dois corpos = mu*2"  =  2
```

**Pior que a ausência:** a IDE oferecia órbita, pêndulo duplo e massa-mola
acoplada, deixava a borda ficar verde numa fórmula algébrica qualquer e devolvia
um número — enquanto o integrador que resolve os três ficava inalcançável.

**O que a ligação trouxe:**

```text
o painel escolhe o RAMO pelo CATALOGO   `system.isSystem`, nunca olhando a formula
4 arquivos novos de tela                SimSystemAuthoring (n equacoes, n
                                        ligacoes, n estados iniciais),
                                        SimComponentEquation (uma equacao),
                                        SimSystemAccuracy (os DOIS sinais de
                                        exatidao) e SimSystemResultView (a
                                        procedencia e os dois modos de grafico)
1 dono para o formato do numero         `SimFormat`, singleton. A regra ia ser
                                        copiada para o segundo arquivo, e copia
                                        de derivacao e' o que o
                                        verificar-qml-duplicacao.sh persegue
o simpletico RECUSA com motivo          conceito sem pareamento declarado nao
                                        oferece o metodo, e a tela diz por que
salvar um sistema RECUSA com motivo     o `SimSaved` carrega UMA formula; um
                                        sistema tem uma por componente
```

**O gate que nasceu, e por que ele não existia.** Nenhuma das dezoito
verificações via o buraco, e a razão é estrutural: **cada uma confere o
componente por si.** O `qmllint` lê um arquivo; o de propriedades confere o
binding onde ele está escrito; o harness instancia o que o teste pediu. Faltava
alguém perguntando se **alguma tela chega ali** — que é a pergunta do usuário.

```text
verificar-qml-alcance.sh   19o gate. Todo .qml do QML_FILES e' instanciado em
                           outro arquivo do modulo, ou — se for `pragma
                           Singleton` — usado pelo nome. Sem baseline:
                           entregue e inalcancavel e' ZERO
tst_sim_system_panel       30o harness. Conceito de sistema nao cai na tela
                           escalar, ha' uma equacao e um estado inicial por
                           componente, o grafico vetorial aparece com o
                           resultado, e o simpletico e' recusado com motivo
```

**A catraca reprovou no meio da fatia, e o corte foi por RESPONSABILIDADE:** o
`SimSystemResultView` chegou a 324/300 e o que estava misturado eram duas
perguntas — *"o quanto isto erra"* e *"como isto se desenha"*. A §13.5 já as
tratava separadas no desenho, e o `SimSystemAccuracy` nasceu daí. Nenhum limite
foi levantado.

Os dois foram provados por mutação. No gate, nos dois sentidos: tirar o
`SimPlotSystem` da tela **reproduz o achado original**, e tirar o `pragma
Singleton` do `StatusColors` pega o outro caminho. No harness, três mutações —
sumir com o `SimSystemAuthoring`, deixar o campo escalar visível sempre, e calar
a recusa do simplético — derrubam três assertivas diferentes.

**A lição, e ela generaliza:** *"o motor passa nos testes"* e *"o usuário alcança
o motor"* são afirmações diferentes, e este repositório só tinha rede para a
primeira. Uma fatia que entrega core, protocolo, ponte C++, roteador e
controller **ainda não entregou nada** enquanto nenhuma tela instancia o
componente — e é o tipo de coisa que uma frase de uso acharia em cinco segundos
(§6) e que dezoito gates não acharam em um dia.

**Achado de quebra, e não é desta fatia:** o `EditorUnsavedChangesDialog.qml`
(198 linhas, do commit de fundação) não está no `QML_FILES` e ninguém o
referencia. Ele não chega ao binário, então o gate novo não o vê. Fica
registrado para decisão: ligar ou remover.

### 7.3 O oráculo entrou, e a coluna `exato` parou de mentir — 2026-09-10

**A segunda escolha da §7.** O que ela conserta é o defeito de veracidade da
§19.0 do [`31`](31-simulacao-fisica-matematica.md): a coluna `exato` vinha de
`catalogo::exata(conceito.id)` e **não olhava a fórmula digitada**, enquanto o
`sim.checkFormula` a aprovava porque confere ligação, não física.

**A decisão de desenho, e ela não era óbvia: o conserto não é esconder o
número.** Sem SymPy na máquina, a solução do conceito continua sendo a melhor
resposta disponível — o que faltava era **dizer que é ela**. Nasceu daí o
`SimAccuracySource`, e não um `if` que apaga a coluna.

**Exercitado contra o binário real**, com SymPy 1.14.0 numa venv:

```text
formula digitada          exato agora        proced.    erro abs     erro rel
-(k/m)*x - (c/m)*v          0.032128320      oracle    3.732e-06    1.161e-04
(k/m)*x - (c/m)*v       83178.343751029      oracle    1.474e+00    1.773e-05
-(k/m)*x - 2*(c/m)*v        0.006879277      oracle    3.222e-07    4.684e-05
-(k/m)*x*x*x - (c/m)*v      0.032128320     concept    8.634e-02    2.687e+00
```

**As três primeiras batem com a coluna "erro REAL" da §19.0**, que tinha sido
medida com o `dsolve` resolvendo à mão o que fora digitado. **A mentira de
78.000x acabou.** A quarta é a §19.3.5 acontecendo: o `dsolve` responde
`NotImplementedError`, a IDE cai na solução do conceito **e avisa**.

**O segundo achado da §19.0 também entrou:** o erro relativo, ao lado do
absoluto. Repare na segunda linha — `1,474` sobre 83 mil é `1,8e-5`, uma
integração excelente; a mesma tela mostrava só o absoluto.

**O que a fatia trouxe:**

```text
sim/oraculo.rs           processo externo, teto de 5 s, racionalizacao e o
                         PORTAO: `var_names()` tem de ser exatamente ["t"]
Core::set_oraculo        a dependencia e' INJETADA, nao descoberta — senao a
                         suite passa a depender de a maquina ter SymPy
SimAccuracySource        a procedencia, no protocolo. Foi a AUSENCIA deste campo
                         que fez a coluna mentir
SimAccuracyProvenance    a linha na tela, num dono so' para as duas formas
fake_sympy_oracle.py     um `python3` FALSO que grava o que recebeu. O cenario
                         viaja por ARGUMENTO: `unsafe` e' proibido aqui, e
                         escrever variavel de ambiente virou `unsafe` na edicao
                         2024 — e a trava esta certa, porque ambiente e' estado
                         global e teste que o escreve contamina o vizinho
```

**Cinco mutações provam os gates do core** (o portão para de conferir
`var_names`; some a troca de `**`; a exatidão ignora o oráculo; o teto vira 20x
maior; os parâmetros viajam como float) **e três provam o do QML** (some o
`SimAccuracyProvenance`; a tela para de reconhecer a procedência do oráculo;
some o erro relativo).

**Custo medido:** 425–506 ms por corrida, dos quais ~200 ms são o `import
sympy`. Uma ida por **fórmula**, nunca por ponto.

**O que ele NÃO cobre, e está dito na tela:** a forma vetorial. O `dsolve` sobre
sistema não foi medido, e entrar sem medir é o oposto do que este domínio faz —
a tela do sistema carrega a ressalva, e o sinal honesto dela continua sendo o
**invariante**, medido na trajetória do próprio autor.

**E uma coisa que a fatia tornou visível sem ser culpa dela:** `sim.run` é
síncrono e bloqueia o laço do core. Ele já podia levar minutos (o teto é 100
milhões de passos); o oráculo acrescenta 5 s no pior caso. Virar job é fatia
própria, e vale para os dois.

### 7.4 As unidades passaram a ser CHECADAS — 2026-09-10

**Decisão do autor em 2026-09-05, e ela dependia do oráculo existir.** A decisão
original desta etapa era rótulo SEM checagem, tomada sobre a medição de que o
`uom` checa em tempo de COMPILAÇÃO e uma fórmula digitada não tem tipo Rust
nenhum. O SymPy checa em EXECUÇÃO, que é quando a fórmula do usuário existe.

Entrou **na mesma ida do oráculo**: o processo custa ~200 ms de `import` antes
de qualquer conta.

**Três camadas, e a terceira é a que pega o caso difícil:**

```text
1. argumento de transcendente   `sin(x)` com `x` em metros
2. os TERMOS entre si           `x + x^3` nao se soma
3. o LADO ESQUERDO              a equacao tem de ser da grandeza do estado
                                dividida pelo tempo elevado a ordem — e' o que
                                pega `-(k/m)*x*x*x` SOZINHO, coerente consigo
                                mesmo e que nao e' uma aceleracao
```

**Na forma vetorial ela vale mais**, e a razão é aritmética: são `n` equações.
Medido — `vx' = x` (a posição no lugar da velocidade) **passa no
`sim.checkSystem`**, porque ele confere ligação e não física, e sai `wrongSide`
aqui.

**A medição achou TRÊS armadilhas antes do código, e as três dariam veredito
errado em silêncio** (`31` §19.5):

```text
substituir pela UNIDADE crua   `a*x - b*v` vira `u - u = 0`, e a dimensao de
faz os termos CANCELAREM       zero e' 1
o `check_dimensions` fica      medido: ele ACEITA `length^3/time^2 +
CEGO com simbolo livre         length/time^2`. A defesa NAO e' usa-lo
o expoente volta FLOAT         `length^1.00000000000000` != `length^1` num
                               dicionario, e as quatro equacoes CERTAS da
                               orbita foram reprovadas por isso
```

**E uma quarta, que mudou o protocolo com o processo.** O `dsolve` do pêndulo
não linearizado não volta, o teto de 5 s o mata, e **o veredito de unidade
morria junto** — pronto em 3 ms, dizendo exatamente o que estava errado. O
processo passou a responder em **duas linhas, a barata primeiro**, e o core lê
linha a linha; o teto não descarta mais o que já chegou. Medido contra o
binário real, com SymPy 1.14.0:

```text
formula                  unidades              ms      procedencia do exato
-(k/m)*sin(x)            dimensionalArgument   5024    concept (com a ressalva)
-(k/m)*x*x*x             wrongSide             5018    concept (com a ressalva)
```

**O limite vai na tela junto com o recurso:** unidade que fecha não quer dizer
física certa — `E = m·v²` sem o meio passa, porque coerência dimensional não vê
constante adimensional.

**A catraca reprovou dois arquivos, e os dois cortes foram por
RESPONSABILIDADE:** o `oraculo.rs` em 714/500 virou pasta (`mod` a fachada,
`programa` o Python embutido, `processo` o transporte, `portao` o que se
aceita de volta), e o `sim_corrida.rs` do protocolo em 509/500 perdeu a forma
vetorial para o `sim_sistema.rs` — o mesmo corte que o core já tinha entre
`corrida.rs` e `corrida_sistema.rs`, pela mesma razão. Nenhum limite levantado.

**O que ficou de fora, e por quê:** a forma **algébrica**. Ela não declara a
unidade do RESULTADO (`energia-cinetica` declara `m` e `v`, não o joule), então
só metade da checagem seria possível — e meia checagem numa tela que promete
conferir é pior que nenhuma. Fechar isso é trabalho de tabela, não de motor.

  protocolo  0.88.0 — `SimDimensionCheck` e `SimDimensionVerdict`
  testes     666 Rust (+8) e 31 harnesses; 3 mutacoes no core, 2 no QML

### 7.5 O `core_client.h` saiu de 500/500 — a CATEGORIA estava errada, 2026-09-10

**Decisão do autor**, e ela vem depois da medição descartar a outra saída.

O arquivo bateu em **exatamente 500/500** e travaria a próxima assinatura IPC que
a UI consumisse — três dos itens abertos acrescentam método. As duas saídas
estavam escritas desde 2026-09-06: **(a)** mover comentário para junto da
implementação, ou **(b)** corrigir a categoria.

**A medição matou a (a).** O header tem **422 linhas de declaração pura contra 63
de comentário**; mover os `///` das declarações compraria ~33 linhas e custaria a
convenção da própria linguagem — corte por TAMANHO, que a `ARCHITECTURE` §4
regra 9 recusa em qualquer arquivo.

**O que ele é, medido:** 289 itens declarados (129 `Q_INVOKABLE`, 18
`Q_PROPERTY`, 142 `void`) em 423 linhas — **1,46 linha por item**. É o argumento
do composition root aplicado a C++: o tamanho é função do **contrato** que ele
espelha. E a saída padrão do composition root ("dividir a composição por área")
não existe aqui: um `QObject` é **uma** classe, e classe não se divide em dois
arquivos.

**O limite não virou um número escolhido**, e essa é a parte que importa. Fixar
700 porque 500 não coube seria levantar limite para caber. Ele é **2× o que o
arquivo declara**:

```text
so' cresce DECLARANDO        hoje 578 contra 500 — a folga de 78 linhas e' o
                             espaco entre a densidade medida (1,46) e o teto (2)
comentario sem declaracao    come a folga e REPROVA
logica que entra no header   derruba o arquivo para 500 NA HORA, porque ele
                             deixa de ser desta categoria
```

Provado por mutação nas três direções: um corpo de função reprova em 502/500;
noventa linhas de comentário reprovam em 590/578; trinta declarações novas
passam.

**A catraca continua com UM arquivo em débito** — o `EditorController.qml` em
791/400, que segue congelado por decisão registrada (`../arquitetura/32` §8.4).
Nenhum limite foi levantado nesta passada: corrigiu-se a categoria de um arquivo
que nunca teve lógica, que é a segunda vez que isso acontece (a primeira foi o
`ui/qml/app/`, que nunca foi QML visual).

### 7.6 O registro de saídas: o que foi feito, e o que continua faltando

**O registro continua VAZIO, e isso está certo** — não porque ninguém olhou,
mas porque a regra do arquivo é estreita de propósito:

```text
E' SAIDA        abandonei a Kinein e abri outra ferramenta para terminar
NAO E' SAIDA    bug que contornei DENTRO da Kinein — isso vai para a fila
                normal. "Misturar os dois esvazia o valor deste arquivo"
```

**Um assistente não pode preenchê-lo.** Uma saída é o autor largando a IDE no
meio de uma tarefa; escrever entrada sem isso ter acontecido é inventar o dado
que o arquivo existe para coletar. O que a sessão de 2026-09-10 pôde fazer foi o
oposto disso: **medir**, e mandar o achado para o lugar certo.

**E o achado é grande.** Ao tentar rodar a IDE para conferir a tela da
simulação, os dois builds do checkout abortaram — com display real e offscreen —
enquanto o AppImage abre. Prioridade 1, anterior a esta sessão, e **com o gate
de dezenove verificações verde**. Está registrado na §4, com os dois contornos
que o isolam.

**A leitura honesta, e ela não mudou desde 2026-09-03:** o registro vazio não
significa que a IDE substituiu o VS Code. Significa que a semana de
desenvolvimento C/C++ e Rust dentro dela — o critério do TR1 — ainda não
aconteceu. E agora há um motivo medido para ela não ter acontecido: **quem
compila do checkout não consegue abrir a IDE nesta máquina.** *(Removido em
2026-09-11, §7.7: os três binários do checkout abrem, e o gate passou a
executá-los.)*

### 7.7 A IDE do checkout voltou a abrir — e a hipótese registrada estava errada, 2026-09-11

**A regra zero mandou medir a §4 antes de aceitá-la, e a primeira medição
confirmou o sintoma e derrubou a causa.** `exit=134` nos dois builds, offscreen
e com display; `exit=0` no AppImage. Até aí, igual ao registro. O que o gdb
acrescentou foi **um número que não batia**: a pilha apontava o assert em
`qarraydataops.h:286`, e no header em disco (6.11.2) a linha 286 é `private:` —
o `copyAppend` com seus quatro `Q_ASSERT` está na 301–304. O binário tinha sido
compilado contra um header que **não é o que está no disco**.

**A causa, medida em quatro passos:**

```text
1. onze objetos de 2026-09-04 14:23    main.cpp.o, clipboard, documentation, os
   nas TRES arvores de build           cinco editor_highlighter*, debug_flags,
                                       window_chrome_controller e um qrc — todos
                                       anteriores ao Qt subir para 6.11.2 as
                                       22:21 do mesmo dia
2. o ninja os da' como VALID           `ninja -t deps` compara MTIME; o rpm
                                       instala o header com o mtime de quando o
                                       PACOTE foi construido: 2026-05-11. Header
                                       "mais velho" que o objeto = nada a fazer
3. dois objetos definem o mesmo        `QGenericArrayOps<QVariant>::copyAppend`
   inline, em duas versoes             e' template inline (COMDAT). So' dois .o
                                       o instanciam: `editor_highlighter_
                                       folding.cpp.o` (VELHO, 6.11.1, assert na
                                       286) e o `GlobalShortcuts` compilado em
                                       AOT (NOVO, 6.11.2, assert na 301)
4. o linker dobra na versao velha      no 6.11.1 a classe HERDA de
                                       QArrayDataPointer e `this` e' o dado; no
                                       6.11.2 ela guarda `m_ptr`. O chamador novo
                                       passa um ponteiro para o ponteiro, o
                                       callee velho le como se fosse o dado, e
                                       `!this->isShared() || b == e` estoura na
                                       primeira `QVariantList{"Alt+F12","Ctrl+`"}`
                                       que o QML compilado monta
```

O `this->` no texto do assert é a prova final: a sintaxe do 6.11.2 é
`that()->`. **E os dois contornos registrados na §4 explicam-se pela mesma
causa:** `QV4_FORCE_INTERPRETER` e `QML_DISABLE_DISK_CACHE` desligam o AOT, que
era o **único chamador novo** daquele inline — a versão velha nunca recebia o
layout que não entende.

**Por que o stderr estava vazio, e isso custou uma sessão:** o Qt do Fedora é
compilado com journald. Sem tty, o assert vai para o journal, e quem roda por
`timeout` ou por pipe vê um SIGABRT mudo. `journalctl --user _COMM=kinein-vectis`
tinha a linha o tempo todo. `QT_FORCE_STDERR_LOGGING=1` a traz de volta — o
`atualizar-tudo.sh` já usava, o smoke novo também.

**O conserto, medido:** remover os onze objetos e relinkar — 9,5 s no
`dev-local`. Os três binários do checkout abrem; a cópia do anterior, guardada
antes do conserto, continua abortando:

```text
build/dev-local                       primeiro frame em 239 ms   exit 0
build/linux-clang-debug-strict        primeiro frame em 548 ms   exit 0
build/dev-local-release               primeiro frame em 235 ms   exit 0
o binario de 2026-09-10 (copia)       SIGABRT                    exit 134
```

**A lição que corrige o registro:** o remédio da §8.0 do
[`31`](31-simulacao-fisica-matematica.md) — reconfigurar as duas árvores depois
da troca de Qt — **resolvia o gate e não a IDE**. Reconfigurar troca o caminho
da biblioteca no `build.ninja`; não toca em objeto. Só `--clean-first` (que o
`atualizar-tudo.sh` faz e o `verificar.sh` não) ou remover os objetos velhos
resolve. O ninja não tem modo por ctime; a defesa é gate.

**O gate que nasceu — o 20º, e a pergunta que ele faz é a que faltava:**

```text
verificar-binario-abre.sh   depois de CADA `cmake --build` do verificar.sh:
                            1. nenhum objeto da arvore e' mais velho (mtime) que
                               a chegada ao disco (ctime) de uma dependencia
                               FORA da arvore de build que o ninja registrou
                               para ele — o que esta' dentro e' do ninja, e a
                               primeira rodada do gate provou isso com um falso
                               positivo num gerado. Se for, diz qual header,
                               quando chegou, e imprime o comando que remove
                               exatamente aqueles objetos
                            2. o binario chega ao primeiro frame offscreen e
                               sai com 0 — o MESMO mecanismo do smoke do
                               AppImage (KINEIN_PERF_MARKER + KINEIN_PERF_EXIT),
                               que existia desde o M4.2 e nunca foi apontado
                               para o binario do checkout
```

O ctime é a hora em que o inode entrou **neste** disco; nenhum gerenciador de
pacote o falsifica, e é o que o ninja não olha. A primeira pergunta existe
porque a segunda, sozinha, dá um SIGABRT — e foi um SIGABRT sem explicação que
produziu a hipótese errada da véspera.

**O critério afinou duas vezes depois de nascer.** A primeira rodada excluiu
dependências DENTRO da árvore de build (o ninja as gerencia). A segunda
(2026-09-11, ao editar o próprio `CMakeLists.txt`) restringiu ao TRAP exato: a
dependência parece velha para o ninja (`mtime <= objeto`, então ele não
recompila) **e** chegou depois (`ctime > objeto`). Um arquivo do repo que o
autor acabou de editar tem `mtime > objeto` — o ninja o recompila sozinho, e
não é este gate que cuida disso. Sem essa segunda trava, editar um `.md` no
`CMakeLists` reprovava um `mocs_compilation` que o próximo build já conserta.

**Provado por mutação nas duas metades:** `touch -d` num objeto para antes da
instalação do Qt reprova em (1) e nomeia `QtQuick/qtquickglobal.h` chegando às
22:21:19; o comando impresso remove o objeto, o rebuild recompila, e o gate
volta a passar. O binário de 2026-09-10 no lugar do relinkado reprova em (2)
mostrando o assert — que agora aparece, porque o smoke força o stderr.

```text
gate    20 verificacoes (+1)
codigo  scripts/verificar-binario-abre.sh, scripts/verificar_binario_abre.py,
        dois passos no verificar.sh — nenhuma linha de core, protocolo ou QML
```

**O que NÃO mudou:** protocolo `0.88.0`, 666 testes, 31 harnesses, catraca com
um arquivo. O defeito era da árvore de build, não do fonte — e por isso nenhum
`git log` o explicava.

### 7.8 Frente F, fatia 1 — o fio: a tela passa a alcançar o que o core já tinha, 2026-09-11

**As doze decisões estão no [`35`](35-ambiente-cpp-embarcados-simulacao.md)
§5.7; a ordem escolhida foi fio → QEMU → tela → polimento.** Esta é a primeira,
e ela não escreve motor nenhum: liga à tela três coisas que existiam no core
desde 2026-09-03 e que nenhuma tela pedia.

```text
papel `debugAdapter`   o menu de toolchain listava 5 papeis e omitia o sexto:
                       ninguem escolhia o probe-rs pela tela. Uma linha
`chip` do kit          o `toolchainSetKit` do C++ nao levava `chip`, e a
                       resposta nao o devolvia: so' a CLI gravava um. Agora
                       viaja nos dois sentidos (ponte, controller, roteadores)
`probe.list`           ganhou consumidor: `EmbeddedController` + painel
                       "Embarcados" em Ambiente do projeto (Ctrl+Alt+M, menu
                       e paleta), com a sonda reconhecida, a dica do core, a
                       saida CRUA quando nada foi reconhecido, e os tres campos
                       do kit (chip, alvo, sysroot) EDITAVEIS num gesto so' —
                       `toolchain.setKit` e' uma escrita, nao tres
```

**Exercitado contra o core real por stdio:** `probe.list` devolve
`probes`/`toolAvailable`/`rawOutput`/`hint` — os nomes que o C++ lê — com a
saída sem escapes ANSI; `toolchain.setKit { chip }` grava e `toolchain.get`
devolve `STM32F401CC`; o papel `debugAdapter` oferece `lldb-dap` e `probe-rs`
nesta máquina, com `lldb-dap` automático.

**Provado por mutação:** três no `EmbeddedController` (a linha da sonda esquece
a família; o erro de outro domínio acende este painel; trocar de workspace não
esquece a sonda) e uma no fio — tirar o `EmbeddedPanelHost` do
`ShellEnvironmentOverlays` reprova no 19º gate, que é exatamente o buraco que
esta fatia fecha.

```text
gate      20 verificacoes, 32 harnesses (+tst_embedded)
codigo    core: 1 descriptor de paleta (`probe.list`, Ctrl+Alt+M)
          C++:  core_client_probe.cpp (novo), `chip` na ponte da toolchain
          QML:  embedded/ (controller, painel, host, campo), 2 roteadores,
                `debugAdapter` no menu, `chip` no controller da toolchain
catraca   nenhum limite tocado; AppDomains em 390/400 — a proxima fatia que
          precisar de dominio novo la' corta por responsabilidade
```

**O que esta fatia NÃO faz, e está dito na tela:** não grava, não roda, não
sobe o depurador — isso é a fatia 3. E o `probe-rs` continua sem sonda no USB
desta máquina, então o painel mostra hoje o estado "nenhuma sonda conectada"
com a dica do core.

### 7.9 Frente F, fatia 2 — o QEMU e a ponte: o ciclo de embarcado provado sem placa, 2026-09-11

**A ordem da §5.7 do [`35`](35-ambiente-cpp-embarcados-simulacao.md) era fio →
QEMU → tela → polimento.** A fatia 1 (§7.8) ligou a tela ao que o core tinha; a
2 dá ao core o que faltava para depurar o que **não está na máquina**, e prova
o ciclo inteiro no emulador.

**O achado que encurtou a fatia, medido antes de escrever:** o `integracoes/36`
§3 dizia que OpenOCD/QEMU exigiriam "uma ponte a mais" ou "protocolo novo
inteiro". **O GDB fala DAP nativamente desde a v14** (`/usr/share/doc/gdb/NEWS`,
*"Changes in GDB 14"*), por stdin/stdout, com `-i dap` — medido aqui no gdb 17.2
do Fedora, multiarch (`arm`, `riscv:rv32`). A ponte é um **segundo candidato do
papel `debugAdapter`**, não um cliente GDB-remote no core. O 36 §3 foi corrigido.

```text
protocolo 0.89.0   `remoteTarget` (host:porta, vai no `target remote`) e
                   `debugServer` (comando que a IDE sobe, `{program}` = ELF) no
                   toolchain.setKit e no ToolchainResult
dap/adapter.rs     saiu do session.rs: QUAL adaptador, com que argumentos, e que
                   PEDIDO — `attach` quando ha' remoteTarget, `launch` senao.
                   O `gdb` ganhou `-i dap` + dois `-iex` (medido: sem desligar o
                   debuginfod e o confirm, o attach TRAVA mudo perguntando)
dap/server.rs      o processo servidor: sobe com `exec` antes do adaptador,
                   ESPERA a porta abrir (QEMU em 0,05 s), morre com a sessao
gdb candidato      o papel debugAdapter agora oferece lldb-dap, probe-rs e gdb
escopo preferido   o GDB lista `Registers` PRIMEIRO; o core agora prefere
                   Globals/Locals — senao a tela de variaveis mostrava r0..pc
```

**O ciclo provado contra o core REAL, por stdio, no QEMU `lm3s6965evb`** (uma
fixture bare-metal NOSSA em `scripts/fixtures/embarcado/`, não do usuário):
`setKit(gdb, remoteTarget, debugServer)` → `setBreakpoints` → `debug.start`
sobe o QEMU e faz `attach` → `continue` para no breakpoint em `main.c:4` →
`evaluate contador` lê `0`, depois `1` → as variáveis **não** são registradores
→ `stop` → `finished(exitCode 0)`, e o QEMU morreu com a sessão. 19 ms do
`continue` ao `stop`.

**Provado por mutação, e uma mutação foi descartada por medição:**

```text
attach vira launch       o ciclo nunca para (sem servidor a que conectar)
escopo = primeiro barato  a tela mostra r0..pc no lugar de `contador`
Drop do servidor sem kill o teste de server.rs mede o TEMPO do drop: sem kill
                          ele espera 30 s o `sleep` sair sozinho
DESCARTADA: `sh -c` sem   nao muda nada — o shell exec-otimiza comando unico, e
  `exec`                  o QEMU ainda morre. O `exec` defende o caso de comando
                          COMPOSTO; o kill do Drop defende o OpenOCD, que NAO se
                          mata sozinho no pacote `k` do GDB (o QEMU se mata)
```

**O limite honesto, dito aqui e no código:** o QEMU se mata sozinho ao receber
o pacote `k` do GDB no `disconnect`, então o ciclo no QEMU **não** prova o
`kill` do servidor no `Drop` — quem prova é o teste de `server.rs`, que mede o
tempo do drop. O `kill` existe para o **OpenOCD**, que não se mata, e esse
caminho fica NÃO PROVADO até haver uma sonda. Sonda física e ESP32 continuam
fora desta passada (decisão do autor, §5.7).

```text
gate      21 verificacoes (+verificar-embarcado.sh), roda no gate so' onde
          arm-none-eabi-gcc, qemu-system-arm e gdb existem — ausente nao reprova
codigo    protocolo: remoteTarget/debugServer; core: dap/adapter.rs e
          dap/server.rs novos, gdb no catalogo, escopo preferido; fixture e
          driver stdio. Nenhuma linha de QML
testes    672 Rust (+6: adapter, server, escopo); mutacoes: 3 no ciclo de
          embarcado + 1 no Drop do servidor
```

### 7.10 Frente F, fatia 4.1 — o clangd enxerga o compilador cross, 2026-09-11

**Primeiro dos quatro itens de polimento da §5.7.** A fatia 3 (painel com
Gravar/Rodar/RTT via probe-rs) foi PAUSADA por decisão do autor até haver uma
sonda — ela não é exercitável no QEMU. O polimento, que é, segue.

**O que a medição achou, e derrubou meia premissa:** o clangd 22 do Fedora
acha `<stdint.h>` de um alvo `arm-none-eabi` sozinho (ele traz um
`-internal-isystem .../arm-none-eabi/include`). O `.c` de bare metal **não**
fica vermelho. Mas o `.cpp` fica: os cabeçalhos de `libstdc++` do GCC ARM
(`<array>`, `<cstdint>` em `/usr/lib/gcc/arm-none-eabi/15.2.0/.../c++`) o clangd
não encontra — **32 erros** medidos num arquivo de quatro linhas. Com
`--query-driver=<cross>` na allowlist, o clangd pergunta ao GCC seus `-isystem`
e os 32 somem.

```text
Toolchain::clangd_args   `--background-index` sempre; mais
                         `--query-driver=<caminho do cross>` quando o
                         compilador C ou C++ EFETIVO do kit nao e' um nativo
                         (clang/gcc/clangxx/gxx). Regra por EXCLUSAO: cada alvo
                         cross novo entra sem tocar aqui
fiacao                   configure_clangd_from_toolchain roda no workspace.open
                         e em toolchain.set/setKit; vale na PROXIMA subida do
                         servidor cpp (o server vivo nao e' trocado, mesmo
                         contrato do use_server_command)
seguranca                o clangd EXIGE a allowlist explicita — rodar driver
                         arbitrario e' risco. So' entra o compilador que o
                         usuario escolheu, nunca um glob aberto
```

**Provado em dois níveis:** três testes de unidade sobre `clangd_args` (nativo
sem driver; C++ cross com o caminho resolvido; C+C++ cross numa lista só sem
repetir), mutação-provados (contar o cross como nativo derruba os dois); e o
`verificar-clangd-cross.sh` (22º gate), que roda o **clangd de verdade** contra
o `arm-none-eabi-g++` real e mede 32 erros sem o driver, 0 com ele — e se
declara NÃO CONCLUSIVO onde o clangd achar tudo sozinho, em vez de passar em
falso. É a lição do `probe.rs`/`fd`: fixture não vê mudança de ferramenta.

```text
gate    22 verificacoes (+verificar-clangd-cross.sh)
testes  675 Rust (+3). Nenhuma linha de QML
```

**Faltam três itens de polimento** (§5.7), todos exercitáveis sem sonda:
tamanho de flash/RAM após o build (`arm-none-eabi-size`), estado do udev +
passo oficial, e o monitor serial UART.

### 7.11 Frente F, fatia 4.2 — o tamanho do ELF depois do build, 2026-09-11

**Segundo dos quatro itens de polimento.** Quanto o firmware ocupa de
não-volátil e de RAM — o número que decide se o próximo commit ainda cabe no
chip.

```text
build.size (0.90.0)   sincrono (o size le um arquivo em ms). `{ program? }`:
                      ausente, resolve o ELF como o debug.start. Roda
                      `<prefix>size -A` (prefixo do cross do kit) e devolve
                      secoes + regioes do linker script com a fracao usada
core/size.rs          parse do SysV e do bloco MEMORY do `.ld`; regiao = soma
                      das secoes ALOCADAS por endereco (`.comment`/
                      `.ARM.attributes`, addr 0, NAO contam — senao 35 B da
                      string de versao virariam "flash usada")
Toolchain::binutils_prefix  `arm-none-eabi-gcc` -> `arm-none-eabi-`; de onde
                      sai o size, o objcopy, o objdump do alvo
UI                    painel Embarcados: botao "Medir tamanho" e uma barra por
                      regiao (usado/capacidade), amarela e com aviso a partir
                      de 90%. Sem linker script legivel, mostra os totais por
                      secao
```

**Exercitado contra o `arm-none-eabi-size` real** (dobrado no gate de embarcado,
que já compila a fixture): FLASH 132/262144, SRAM 4/65536, ferramenta
`arm-none-eabi-size` escolhida pelo prefixo do kit. Provado por mutação: 5
testes de unidade sobre o parse; anular a exclusão de seção não-alocada faz a
FLASH medir 212 em vez de 132 e o teste cai (a mutação por dead-code foi
descartada — o compilador a pega antes do teste, a armadilha já registrada).

```text
protocolo 0.90.0 — BuildSizeParams, SizeReport
testes  680 Rust (+5). QML: painel Embarcados ganhou a seção de tamanho
```

**Faltam dois itens de polimento** (§5.7): estado do udev + passo oficial, e o
monitor serial UART.

### 7.12 O ESP32 chegou à mesa — o levantamento da conectividade bare metal, 2026-09-11

**Na tarde do mesmo dia em que a §5.7 do [`35`](35-ambiente-cpp-embarcados-simulacao.md)
registrou *"ESP32-C3/C6/S3 existe mas NÃO pode ser plugado agora"*, o autor
plugou um ESP32** e pediu três coisas: o mapa de como um microcontrolador bare
metal se conecta, o que existe de open source para cada elo, e se o que vale
para o ESP32 vale para STM32 e Raspberry Pi. A resposta é o
[`integracoes/38`](../integracoes/38-conectividade-bare-metal.md) — documento,
não código, pela mesma regra que fez o 36 vir antes da etapa 22.

**Medido antes de ler qualquer fonte, e a medição mudou a pergunta:**

```text
o que esta' no USB   ESP32-D0WD-V3 rev v3.1 — o ESP32 CLASSICO (Xtensa LX6),
                     nao um S3/C3/C6. Ponte CP2102 (10c4:ea60) -> /dev/ttyUSB0
                     root:dialout 0660; o autor esta' em dialout; o esptool
                     5.3.1 conectou, leu chip e flash (4 MB) e resetou por RTS
                     sem ninguem apertar BOOT
o que isso implica   o classico NAO tem USB-JTAG embutido: so' tem o canal
                     SERIAL. Gravar (esptool/espflash) e monitorar (UART
                     115200) sao exercitaveis hoje; DEPURAR exige 4 GPIOs + um
                     adaptador FT2232H (ESP-Prog) + o fork openocd-esp32 + o
                     esp-gdb xtensa — o gdb do Fedora nao tem xtensa, e o
                     OpenOCD upstream desta maquina conhece o alvo mas NAO
                     grava nele (sem flash bank, sem program_esp)
quem mais olha       o ModemManager EXAMINOU a porta 4 s depois do plug
a porta              (journal: "couldn't check support ... not supported by
                     any plugin") — ID_MM_CANDIDATE=1 sem ID_MM_DEVICE_IGNORE.
                     brltty instalado, inativo, sem regra para 10c4
permissao USB        o 60-openocd.rules do Fedora e' o upstream SEM
                     GROUP="plugdev" (so' TAG+="uaccess"); o 69-probe-rs.rules
                     oficial nao esta' instalado e usa plugdev + uaccess;
                     plugdev nao existe nesta maquina. O passo oficial e o que
                     a distro ja' fez NAO sao o mesmo — a fatia 4.3 tem de
                     dizer qual vale
o core               NAO sabe o que e' porta serial: o unico ttyUSB0 nos
                     fontes e' um teste negativo do dap/server.rs
```

**O mapa que generaliza** (38 §2 e §5): todo bare metal chega por até três
canais — **A serial** (ROM bootloader + console + DTR/RTS), **B depuração**
(SWD/JTAG por sonda ou embutido) e **C massa/DFU** (UF2, `dfu-util`) — mais
uma camada de **identidade** (VID:PID → família do elo → chip lido **pelo
canal**). O que vale para as cinco colunas (ESP32 clássico, S3/C3/C6, STM32,
RP2040/RP2350, Pi) é o canal A inteiro, a identidade, o mecanismo de permissão
e o canal B, que já está feito. O que **não** vale é o protocolo do
bootloader — SLIP da Espressif, AN3155/DFU da ST, picoboot/UF2 da Raspberry
Pi — e a IDE não implementa nenhum: orquestra a ferramenta oficial como
processo. **A Raspberry Pi 4/5 como computador não é este domínio**: é o item
"SSH remoto" da §4.

**A licença decidiu uma forma, de novo:** o `espflash` (MIT OR Apache-2.0)
como **crate** não passa no `deny.toml` — a serial dele é o `serialport`
4.10.1, **MPL-2.0**, e a política é "nada de copyleft, nem LGPL". Admitir MPL
é decisão do autor; até lá, `esptool` (GPL-2.0) e `espflash` entram iguais:
processo filho, saída em evento, `NO_COLOR` injetado no `Command`. O monitor
serial abre o tty com `rustix` (já transitivo) — sem crate novo.

**O que entra na fila** (38 §6, proposta): E1 `serial.list` → E3 monitor UART
(a fatia 4.4) → E5 identidade Espressif (`chip-id`/`flash-id` como o
`probe-rs info`) → E2 permissão por canal (a fatia 4.3, redesenhada) → E4
gravar como JOB com motor por família. E6, o debug do clássico, fica
**bloqueado por hardware** ao lado da fatia 3. **Nenhuma das doze decisões da
§5.7 foi reaberta por este documento**; a de *escopo* ("RISC-V/ESP32 depois") é
a que o autor vai ter de rever, e a nota datada no 35 diz isso.

```text
gate      22 verificacoes, inalterado — nenhuma linha de codigo
docs      integracoes/38 (novo), indices em docs/README e integracoes/README,
          nota datada no 35 §5.7, esta secao e a entrada da §4
provado   nada gravado no chip: so' leituras (chip-id, flash-id). O 38 §7 diz
          o que NAO foi provado, item a item
```

### 7.13 E1 — `serial.list`: a IDE passa a ver a porta serial, 2026-09-11

**Primeira das fatias decididas na §7.12.** O core não sabia o que era uma
porta serial (o único `ttyUSB0` nos fontes era um teste negativo do
`dap/server.rs`); agora enumera as USB, e o painel Embarcados as mostra.

```text
serial.list (0.91.0)  /sys/class/tty/ttyUSB* e ttyACM*; sobe ate' o diretorio
                      USB com idVendor (ABI do kernel) para VID:PID, nomes,
                      serial e bInterfaceNumber; driver pelo link; by-id de
                      /dev/serial/by-id; ModemManager por `udevadm info` +
                      /proc/*/comm (null sem udevadm, nao false)
core/serial.rs        NUNCA abre a porta — abrir aciona DTR/RTS e reseta a
                      placa. Permissao MEDIDA com access(2) via `rustix`
                      (+0 crates: ja' era transitiva), porque access honra a
                      ACL do `uaccess`; stat sozinho mentiria
familia               VID:PID -> o que o ELO e' (ponte CP210x, USB-JTAG da
                      Espressif, VCP do ST-Link, Debug Probe...), nunca o chip
                      atras da ponte. Fontes no 38 §5
handler               NAO exige workspace: nao ha' ferramenta do kit aqui
UI                    EmbeddedSerialView (dono proprio; o painel estava em
                      260/300), controller com ports/portsHint/portsBusy, o
                      aviso do ModemManager so' com vivo + candidata + sem
                      IGNORE, e a dica de acesso sem `sudo` que a IDE rodasse
```

**Exercitado contra o core real com o ESP32 plugado:** a resposta bate campo
a campo com a medição manual da tarde — `/dev/ttyUSB0`, CP2102 `10c4:ea60`,
`cp210x`, `if00`, `crw-rw---- dialout` com `readableWritable: true`, e
`modemManager { candidate: true, ignored: false, running: true }`. O chip não
foi resetado. O `verificar-exercitacao.sh` agora pede `serial.list` ao core
real (sem placa: lista vazia com `ports` presente).

**Provado por mutação** (todas com o compilador calado): Rust — parar a subida
do sysfs na interface (reprova nos dois testes de sysfs); trocar `access(2)`
por "bits do grupo no modo" (reprova no caso `0o060`, que é dono sem bits);
trocar vid/pid na família (reprova). QML — o aviso do MM ignorar `running`;
`refresh()` não pedir as portas; trocar de workspace esquecer a sonda e não a
porta. **O limite honesto do teste de acesso:** o caso real do `uaccess` (nó
de root com ACL nomeada) não se monta sem root — medido: numa ACL o dono é
julgado pela entrada do dono; quem prova é a exercitação contra `/dev/ttyUSB0`.

```text
protocolo 0.91.0 — SerialPortInfo, SerialAccess, ModemManagerState
testes  685 Rust (+5, em tests/serial.rs), harness tst_embedded (+16 assercoes)
gate    22 verificacoes; exercitacao ganhou serial.list
```

### 7.14 Containers: Docker e Podman viraram domínio nativo de verdade, 2026-09-12

**A decisão era de 2026-07-17** ([`28`](28-plataforma-de-plugins-e-verticais.md)
§0: *"vão ser cidadãos nativos"*) e até ontem tinha **zero código**. O autor a
priorizou hoje, com um pedido a mais: *"deverá ter um ícone/atalho/visual para
auxiliar no uso, para ativar a ferramenta"* — para Docker **e** para o Grafana,
que já era nativo mas só se alcançava por menu, paleta e Ctrl+Alt+O.

**Medido antes de escrever, e mudou o desenho:** nesta máquina `docker` é o
shim **`podman-docker`** (imprime *"Emulate Docker CLI using podman"* em
stderr); não há Docker Engine; há **Podman 5.8.4 rootless**, socket do usuário
ativo, `podman-compose` 1.6.0 e `podman compose` delegando. Exatamente o
"Podman é alternativa a auditar" do 28 §4 — então o domínio nasceu com **motor**
(docker | podman) atrás da mesma CLI, e a detecção **pergunta ao binário** em
vez de confiar no nome.

```text
protocolo 0.92.0   container.status/list/images (sincronos), action e compose
                   (JOBS, event.container.finished), open (logs|shell numa ABA
                   DE TERMINAL pelo open_command que existia sem chamador)
core/container/    mod.rs: deteccao, status (versao, rootless, socket,
                   responde, compose, passo oficial), comandos; parse.rs: as
                   DUAS formas de JSON (array do Podman, objeto-por-linha do
                   Docker) numa lista so', saida crua sempre
tools.rs           docker, podman, podman-compose no catalogo (tools.detect)
UI                 ContainerController/Panel/ListView/PanelHost + 2 roteadores;
                   AppDomains cortado por RESPONSABILIDADE: os donos do
                   "Ambiente do projeto" foram para AppEnvironmentDomains (ele
                   estava em 390/400); RuntimeController: aba com TITULO
                   opcional (o comando que roda nela)
icone/atalho       SideRail ganhou dois botoes — `container` e `observability`
                   (glifos proprios em KvIconGlyphs.js; nenhum e' marca
                   registrada) — que abrem os paineis SEM projeto; menu
                   Ambiente > Containers...; paleta `container.list`;
                   Ctrl+Alt+W (C e D ja' tinham dono na UI)
invariantes do 28  a UI nunca chama docker (so' o core); acao = job cancelavel;
                   permissao visivel (status e' a tela de ATIVAR: instalar,
                   grupo docker, daemon, podman.socket — impresso, nunca sudo);
                   rm SEM -f; compose up e' -d
```

**Exercitado contra o Podman real:** `status` bate com a medição manual
(podman 5.8.4, emulated, rootless, socket, compose); `list` devolve os 5
containers do autor com nomes, estado, status humano e portas; `images` os 12
(o Podman em `--format json` **não** traz `repository`/`tag` — vem de `Names`,
e o parser aprendeu isso na exercitação, não no teste); o ciclo **start →
logs em aba → rm** num container descartável (`podman create … true`) rodou
por jobs com `event.container.finished { ok: true }` e o container sumiu. O
`verificar-exercitacao.sh` agora pede `container.status` e `container.list`.

**Provado por mutação** (compilador calado): Rust — o shim tratado como Docker
(reprova na detecção); `rm -f` (reprova); `host_ip` ignorado nas portas
(reprova). QML — job falho sem motivo na tela; `isRunning` pelo status humano;
trocar de workspace esquecer o motor. **O que NÃO foi provado:** um Docker
Engine real (a forma `{{json .}}` é da documentação); `compose up/down` de
verdade (só a montagem do comando); o shell dentro do container (só o comando).

```text
testes  693 Rust (+8 em tests/container.rs), 33 harnesses (+tst_container)
docs    03-ipc (dominio + indice), 02 (container/), 28 §4 (nota), 41 (F3)
```

**O que fica para a fatia seguinte deste domínio, e é o que o 28 §4 chama de
contexto remoto:** *dev containers* de verdade — abrir o workspace **dentro**
do container (path mapping, `devcontainer.json` como formato de entrada, build
e LSP do outro lado). É o mesmo contrato do SSH remoto da §4, e os dois devem
nascer da mesma abstração, não de duas.

### 7.15 E3 — o monitor serial como processo numa aba de terminal, 2026-09-12

**A decisão era de 2026-09-11** (38 §6: processo pronto, nunca código serial
nosso — a forma da extensão oficial da Espressif para o VS Code) e a aba com
título que o domínio `container` acabara de dar ao `RuntimeController` era
exatamente o que faltava.

```text
papel `serialMonitor` (kit)   candidatos tio, picocom, minicom, espflash — a
                              ordem e' a preferencia automatica; a UI lista o
                              papel no menu de toolchain desde o nascimento
serial.monitor (0.92.0)       { device, baud? } -> aba de terminal com o
                              comando no titulo. Chip Espressif no kit E
                              espflash detectado -> `espflash monitor --elf`;
                              escolha FIXADA vence; sem monitor nenhum,
                              TOOL_NOT_FOUND com o que instalar
linhas de comando             lidas na fonte: espflash e' `--monitor-baud` —
                              o `--baud` dele e' o de GRAVACAO (pegadinha que
                              a fonte mostrou); tio/picocom `-b`; minicom `-D -b`
UI                            botao de monitor por porta no painel Embarcados
                              (so' com acesso a porta); a aba abre e o painel
                              fecha
```

**Exercitado no ESP32 real:** `serial.monitor { device: /dev/ttyUSB0 }` →
`/usr/bin/picocom -b 115200 /dev/ttyUSB0` numa aba (`picocom` e `minicom` são
os detectados nesta máquina; `tio` e `espflash` não estão), e o render da aba
mostrou o banner do picocom com a porta aberta. Nenhuma linha de código serial
foi escrita. **Provado por mutação:** `--baud` no lugar de `--monitor-baud`;
espflash para qualquer chip; ignorar a escolha fixada — as três reprovam.

```text
testes  695 Rust (+2), tst_embedded (+2 assercoes); exercitacao pede serial.monitor
```

### 7.16 Pilar 0, primeira fatia — o MODELO do projeto embarcado, 2026-09-12

**A resposta em código à pergunta do autor** ("a IDE já lê todo o projeto?",
[`42`](42-trilha-profunda-embarcados.md) §1): até hoje o `workspace` olhava
marcadores só na raiz e não conhecia framework de embarcado nenhum. Nasceu o
domínio `project` (protocolo 0.93.0).

```text
project.model / event.project.changed   o que o projeto E', com evidencia
detect.rs     9 frameworks — ESP-IDF, Zephyr, pico-sdk, PlatformIO, STM32Cube,
              Rust embarcado, MicroPython, Yocto, Buildroot — reconhecidos ate'
              3 niveis abaixo, LENDO o marcador (project.cmake, find_package
              (Zephyr), pico_sdk_init, `import machine`...), ignorando pastas
              de saida, com o detalhe que o arquivo diz (IDF_TARGET, PICO_BOARD,
              DeviceId, triple, MACHINE, ambientes do PlatformIO)
sdk.rs        o que cada framework exige e se esta' aqui — variavel, pasta
              padrao ou binario; ~/.espressif/tools e' vasculhado para a
              toolchain que so' o export.sh poe no PATH; o passo oficial em
              cada falta; nenhum processo roda
artifacts.rs  ELF/bin/hex/uf2/map dos build dirs de cada framework, do mais
              novo ao mais velho; flasher_args.json; tabela de particoes;
              memory.x; linker scripts da fonte; o ELF sem extensao do cargo
              pelo magico
alvo          kit > framework; familia; motores por familia — e o ESP32
              classico recebe "sem depurador: ESP-Prog" em vez de um probe-rs
              que nao funcionaria
evento        emitido em activate_workspace (open e createProject) e ao fim de
              event.cmake.finished / event.build.finished: a tela SEGUE o modelo
tela          EmbeddedProjectView no painel Embarcados: framework · evidencia ·
              detalhe; alvo com as linhas de evidencia; ✓/✗ por SDK com o
              passo; as dicas
fixtures      scripts/fixtures/projetos/<framework>/ — reais e minimas, as
              que os testes leem
```

**Exercitado contra o core real** com a fixture ESP-IDF como workspace: o
evento chega na abertura com `espIdf · CMakeLists.txt · IDF_TARGET esp32c3`,
o alvo `esp32c3/espressif` com `esptool/espflash/probe-rs`, `esptool` achado
em `~/.local/bin`, ESP-IDF e a toolchain riscv **faltando** com o passo — que
é exatamente o estado desta máquina. **Provado por mutação** (compilador
calado): descer em `build/` (a cópia gerada venceria); o framework vencer o
kit; pasta "achada" sem existir; e no QML: trocar workspace sem esquecer o
modelo, `refresh` sem pedir, resumo do alvo sem o depurador.

**Segunda fatia, no mesmo dia:** `flasher_args.json` e `partitions.csv`
passaram de localizados a **lidos** (`project/esp.rs`), com a forma tirada da
fonte do ESP-IDF (`flasher_args.json.in` + `project_include.cmake`; guia
*Partition Tables*): a receita com caminhos absolutos, nome e cifragem por
imagem, e a tabela com os offsets em branco resolvidos como o
`gen_esp32part.py` (a primeira em tabela+0x1000, `app` a 64 KB). Linha que
não se lê vai em `unreadable`. Provado por mutação: caminho não resolvido,
linha ilegível sumindo, `app` alinhada a 4 KB — as três reprovam. A vista do
projeto mostra "receita: N imagens, flash 4MB dio · N partições, app 1 MB em
0x10000".

**O que falta do P0** ([`42`](42-trilha-profunda-embarcados.md) §3): o modelo
por **alvo/preset** (hoje é um por workspace), o map file interpretado, a
detecção de SDK que hoje o modelo declara "não medido" (alvo rustup), e o
consumo da partição `app` pelo `build.size` (hoje ele só lê `.ld`).

```text
protocolo 0.93.0 — project.*, event.project.changed, FlashRecipe, PartitionTable
testes  703 Rust (+8, tests/project.rs), tst_embedded (+9 assercoes)
gate    exercitacao pede project.model
```

## 8. A VARREDURA de 2026-09-10 — o que está entregue e não chega à tela

Feita a pedido do autor, antes do pente-fino. **Ela procura uma classe só, e é a
que já bateu duas vezes nesta etapa:** algo que o core calcula, a ponte
transporta, e nenhuma tela mostra. Foi assim que o motor vetorial ficou um dia
sem porta, e foi assim que a coluna `exato` respondeu por outra equação.

**Os comandos que a reproduzem** — nenhuma linha abaixo pede confiança:

```bash
# metodos IPC roteados no core que o C++ nunca pede
# sinais do CoreClient que ninguem escuta (descontando NOTIFY de Q_PROPERTY)
# Q_INVOKABLE que o QML nunca chama
# sinais QML declarados sem tratador
```

### 8.1 O que é calculado e nunca aparece — prioridade real

```text
event.test.output      EMITIDO e DESCARTADO. O `JobsEventRouter` roteia
                       `onBuildOutput` e NAO roteia `onTestOutput`; o
                       `JobsController` tem `handleBuildOutput` e nao tem o par.
                       O `BuildPanel` tem `outputModel`; o `TestsPanel` tem
                       `casesModel` e `summary`, e nao tem saida nenhuma.

                       O ESTRAGO: num teste que falha, a razao esta' na SAIDA —
                       a mensagem do panico, a diferenca da assercao, o erro do
                       compilador. A tela mostra "falhou" e engole o porque.

                       Conferido: o core emite esta saida SO' em
                       `event.test.output` (handlers/build.rs:170); ela nao
                       chega por `event.job.output` nem por outro caminho.

event.test.started     idem, e o build mostra o dele: o comando que rodou nao
                       aparece nos testes

event.quality.started  idem, na analise

event.quality.output   pior: descartado no PROPRIO C++, com `return true` e sem
                       emitir sinal nenhum. Ele nao chega nem a ter dono

environmentScan        `Started`, `Tool` e `Finished` nao tem ouvinte. O painel
  (tres sinais)        de Ferramentas recebe a lista PRONTA; o progresso da
                       varredura, que existe, nao aparece
```

### 8.2 Superfície morta — o protocolo afirma o que ninguém consome

```text
sim.inspectFormula    round-trip COMPLETO e sem uso: metodo no protocolo,
                      handler no core, `simInspectFormula` no C++ e o sinal
                      `simFormulaInspected` — e o QML nunca chama nenhum deles.
                      A tela usa `sim.checkFormula`, que ja' devolve `variables`

cmake.presets.list    roteados no core e nao pedidos nem pela UI nem pela CLI
job.list
probe.list

dataSourceTestAccepted  o `jobId` do teste de conexao e da sonda do Grafana e'
grafanaProbeAccepted    emitido e ninguem escuta
```

**Não confundir com superfície morta:** `core.shutdown` e `workspace.status` não
são pedidos pelo C++ **porque quem os usa é a `kinein-cli`**. Ela é cliente do
protocolo tanto quanto a UI, e a varredura que olhasse só a UI os acusaria por
engano.

### 8.3 Menor

```text
GitPanel.branchMenuDismissRequested        sinal QML declarado sem tratador
TerminalScrollController.sessionChanged
```

### 8.4 O que a varredura NÃO cobre, e vale dizer

Ela mede **fiação**: o que existe e não está ligado. Ela não mede se a tela que
está ligada mostra a coisa certa, nem se o que aparece é legível — isso é o
pente-fino, e ele precisa da IDE **abrindo** (§4, prioridade 1).

**E a classe da §8.1 não tem gate.** O 19º pega componente QML entregue e não
instanciado; o 20º (2026-09-11) pega binário que compila e não abre; falta o
andar de cima — **evento que o core emite e nenhuma tela consome**. É o mesmo critério, uma camada acima, e os cinco achados de hoje
teriam saído dele automaticamente.

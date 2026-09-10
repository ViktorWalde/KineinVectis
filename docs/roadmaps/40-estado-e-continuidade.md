# 40 — Onde o projeto está, e por onde continuar

> **Classe: ESTADO** (`docs/README.md`). Remedido em **2026-09-10**, com o gate
> completo verde. Se divergir do código, o código vence e este documento se
> corrige no mesmo gesto.
>
> **AVISO que a sessão de 2026-09-05 aprendeu na pele:** "gate verde" tem prazo
> de validade de uma atualização de sistema. Ao abrir a sessão o gate
> **reprovava**, e não era código — o Qt do sistema subiu de 6.11.1 para 6.11.2
> em 2026-09-04 22:21, e as duas árvores de build guardavam o caminho absoluto
> antigo. `cmake --preset dev-local` **e** `cmake --preset dev-local-release`,
> seguidos de rebuild, resolvem. **Cada árvore carrega o caminho por si; consertar
> uma não conserta a outra.** Detalhe em [`31`](31-simulacao-fisica-matematica.md) §8.0.
>
> **COMECE POR AQUI ao retomar.** Ele substitui o
> [`38`](38-divida-restante-e-continuidade.md) nesse papel; o 38 vira registro
> de como a fila estava quando a dívida foi paga.
>
> **Regra zero vale aqui como em tudo:** antes de aceitar qualquer item como
> pendente, MEÇA. Cada seção carrega o comando.

## 1. O estado, em números

```bash
bash scripts/verificar.sh                 # 19 verificacoes
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
protocolo   0.88.0
testes      666 Rust + 31 harnesses QML
metodos     130 IPC roteados, 41 eventos
dominios    30, e os 30 documentados no arquitetura/03
catraca     1 arquivo em debito
gate        19 verificacoes
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
--  A IDE DO CHECKOUT NAO ABRE, e     ACHADO em 2026-09-10. Prioridade 1
    o gate inteiro fica verde         (crash), e NAO e' regressao desta
                                      sessao: o binario de release de
                                      2026-09-06, anterior a tudo, aborta
                                      igual.

                                      MEDIDO, com display real E offscreen:

                                        build/dev-local            SIGABRT
                                        build/linux-clang-debug-   SIGABRT
                                          strict
                                        dist/*.AppImage            ABRE

                                      O AppImage empacota o proprio Qt; o
                                      checkout usa o do sistema, 6.11.2. A
                                      pilha aponta o QML compilado em AOT:
                                      `GlobalShortcuts.qml` montando um
                                      `QList<QVariant>` a partir de
                                      initializer list, e o assert estoura
                                      dentro do `QGenericArrayOps`.

                                      ISOLADO por dois contornos INDEPENDENTES,
                                      os dois em runtime:

                                        QV4_FORCE_INTERPRETER=1   abre
                                        QML_DISABLE_DISK_CACHE=1  abre
                                        QT_ENABLE_REGEXP_JIT=0    aborta
                                                                  (controle)

                                      POR QUE O GATE NAO VE: o
                                      `verificar-appimage.sh` roda o smoke do
                                      ARTEFATO em dist/, que empacota outro Qt
                                      — e ele passa. Nada executa o binario que
                                      sai do `cmake --build`. E' a mesma classe
                                      de falha do 19o gate, num eixo novo:
                                      "compila" e "abre" sao afirmacoes
                                      diferentes.

                                      NAO E' SAIDA de dogfooding (o registro
                                      recusa bug contornado dentro da Kinein);
                                      e' bug, e mora aqui
--  FECHADOS nesta passada, e ficam       a coluna `exato` que respondia por
    aqui so' como registro                OUTRA equacao (§19.0) — consertada em
                                          2026-09-10 pela PROCEDENCIA, §7.3; e o
                                          `03-ipc` sem cobertura de 5 dominios,
                                          escrito em 2026-09-06. Conferido em
                                          2026-09-10: dos 130 metodos roteados,
                                          ZERO estao ausentes do documento
25  handshake DAP com probe-rs           PARCIAL: precisa de sonda fisica ou
                                         alvo QEMU. O resto do ciclo de
                                         embarcado esta' provado
--  guias de instalacao para arch/suse   a fonte oficial dos tres projetos NAO
                                         cobre essas familias; entrar exige
                                         fonte de comunidade, marcada como tal
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
Python                       adiado
Pylance                      PROIBIDO (licenca)
Docker e banco               NATIVOS, nao plugins
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
compila do checkout não consegue abrir a IDE nesta máquina.**
